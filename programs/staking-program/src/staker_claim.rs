use crate::{helper, Errors, StakeAccount, StakingPool};
use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_interface::{
    transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked,
};

#[derive(Accounts)]
pub struct Claim<'info> {
    pub user: SystemAccount<'info>,

    #[account(mut)]
    pub staking_pool: Box<Account<'info, StakingPool>>,

    #[account(
        mut,
        seeds = [
            helper::VAULT_SEED,
            &staking_pool.key().to_bytes(),
        ],
        bump = staking_pool.vault_seed_bump
    )]
    pub pool_vault_signer: SystemAccount<'info>,

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
        mut,
        associated_token::mint = token_mint,
        associated_token::authority = user,
        associated_token::token_program = token_program,
    )]
    pub user_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = token_mint,
        associated_token::authority = pool_vault_signer,
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
    pub fn process(&mut self) -> Result<()> {
        require_gt!(self.stake_account.reward, 0, Errors::ClaimAmountZero);

        self.staking_pool.update_pool()?;

        self.stake_account
            .update_reward(self.staking_pool.reward_per_share)?;

        self.stake_account
            .update_reward_debt(self.staking_pool.reward_per_share)?;

        let claim_amount = self.stake_account.reward;
        self.stake_account.reward = 0;

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
                    authority: self.pool_vault_signer.to_account_info(),
                },
                &[&[
                    helper::VAULT_SEED,
                    &self.staking_pool.key().to_bytes(),
                    &[self.staking_pool.vault_seed_bump],
                ]],
            ),
            claim_amount,
            self.token_mint.decimals,
        )?;

        emit!(EventClaim {
            user: self.user.key(),
            claim_amount,
            staking_pool: self.staking_pool.key()
        });
        Ok(())
    }
}
