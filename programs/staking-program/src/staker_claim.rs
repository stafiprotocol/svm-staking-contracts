use crate::{helper, Errors, StakeAccount, StakingPool};
use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_interface::{
    transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked,
};

#[derive(Accounts)]
pub struct Claim<'info> {
    /// CHECK:
    pub user: AccountInfo<'info>,

    #[account(mut)]
    pub rent_payer: Signer<'info>,

    #[account(mut)]
    pub staking_pool: Box<Account<'info, StakingPool>>,

    #[account(
        mut,
        has_one = staking_pool @Errors::InvalidStakeAccount,
        has_one = user @Errors::InvalidStakeAccount,
    )]
    pub stake_account: Box<Account<'info, StakeAccount>>,

    #[account(
        address = staking_pool.token_mint @Errors::TokenMintAccountNotMatch
    )]
    pub token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        init_if_needed,
        payer = rent_payer,
        associated_token::mint = token_mint,
        associated_token::authority = user,
        associated_token::token_program = token_program,
    )]
    pub user_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = token_mint,
        associated_token::authority = staking_pool,
        associated_token::token_program = token_program,
    )]
    pub pool_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

#[event]
pub struct EventClaim {
    pub user: Pubkey,
    pub claim_amount: u64,
    pub staking_pool: Pubkey,
}

impl<'info> Claim<'info> {
    pub fn process(&mut self, restake: bool) -> Result<()> {
        self.staking_pool.update_pool()?;

        self.stake_account
            .update_reward(self.staking_pool.reward_per_share)?;

        let claim_amount = self.stake_account.reward;

        if claim_amount > 0 {
            self.stake_account.reward = 0;

            if restake {
                self.stake_account.amount += claim_amount;
                self.staking_pool.total_stake += claim_amount;
            } else {
                require_gte!(
                    self.pool_token_account.amount,
                    claim_amount,
                    Errors::PoolBalanceNotEnough
                );

                transfer_checked(
                    CpiContext::new_with_signer(
                        self.token_program.to_account_info(),
                        TransferChecked {
                            from: self.pool_token_account.to_account_info(),
                            mint: self.token_mint.to_account_info(),
                            to: self.user_token_account.to_account_info(),
                            authority: self.staking_pool.to_account_info(),
                        },
                        &[&[
                            helper::POOL_SEED,
                            &self.token_mint.key().to_bytes(),
                            &self.staking_pool.creator.key().to_bytes(),
                            &[self.staking_pool.index],
                            &[self.staking_pool.pool_seed_bump],
                        ]],
                    ),
                    claim_amount,
                    self.token_mint.decimals,
                )?;
            }
        }

        self.stake_account
            .update_reward_debt(self.staking_pool.reward_per_share)?;

        emit!(EventClaim {
            user: self.user.key(),
            claim_amount,
            staking_pool: self.staking_pool.key()
        });

        Ok(())
    }
}
