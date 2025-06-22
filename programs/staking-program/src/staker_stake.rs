use crate::{helper, Errors, StakeAccount, StakingPool};
use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_interface::{
    transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked,
};

#[derive(Accounts)]
pub struct Stake<'info> {
    pub user: Signer<'info>,

    #[account(mut)]
    pub rent_payer: Signer<'info>,

    #[account(mut)]
    pub staking_pool: Box<Account<'info, StakingPool>>,

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
        associated_token::authority = staking_pool,
        associated_token::token_program = token_program,
    )]
    pub pool_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        init_if_needed,
        space = 8 + std::mem::size_of::<StakeAccount>(),
        payer = rent_payer,
        rent_exempt = enforce,
        seeds = [
            helper::STAKE_ACCOUNT_SEED,
            &user.key().to_bytes(),
        ],
        bump,
    )]
    pub stake_account: Box<Account<'info, StakeAccount>>,

    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

#[event]
pub struct EventStake {
    pub staker: Pubkey,
    pub stake_amount: u64,
    pub staking_pool: Pubkey,
}

impl<'info> Stake<'info> {
    pub fn process(&mut self, stake_amount: u64) -> Result<()> {
        require_gte!(
            stake_amount,
            self.staking_pool.min_stake_amount,
            Errors::StakeAmountTooLow
        );

        self.staking_pool.update_pool()?;

        let transfer_to_pool_cpi_context = CpiContext::new(
            self.token_program.to_account_info(),
            TransferChecked {
                from: self.user_token_account.to_account_info(),
                mint: self.token_mint.to_account_info(),
                to: self.pool_token_account.to_account_info(),
                authority: self.user.to_account_info(),
            },
        );
        transfer_checked(
            transfer_to_pool_cpi_context,
            stake_amount,
            self.token_mint.decimals,
        )?;

        self.staking_pool.total_stake += stake_amount;

        if self.stake_account.user == Pubkey::default() {
            self.stake_account.set_inner(StakeAccount {
                staking_pool: self.staking_pool.key(),
                user: self.user.key(),
                amount: stake_amount,
                reward: 0,
                reward_debt: self.staking_pool.calc_reward_debt(stake_amount)?,
                _reserved: [0u8; 128],
            });
        } else {
            self.stake_account
                .update_reward(self.staking_pool.reward_per_share)?;

            self.stake_account.amount += stake_amount;

            self.stake_account
                .update_reward_debt(self.staking_pool.reward_per_share)?;
        }

        emit!(EventStake {
            staker: self.user.key(),
            stake_amount,
            staking_pool: self.staking_pool.key(),
        });
        Ok(())
    }
}
