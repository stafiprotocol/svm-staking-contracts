use crate::{helper, Errors, StakingPool, UnstakeAccount};
use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_interface::{
    transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked,
};

#[derive(Accounts)]
pub struct Withdraw<'info> {
    pub user: SystemAccount<'info>,

    #[account(mut)]
    pub rent_payer: Signer<'info>,

    #[account(mut)]
    pub staking_pool: Box<Account<'info, StakingPool>>,

    #[account(
        mut,
        close = user,
        has_one = staking_pool @Errors::InvalidUnstakeAccount,
        has_one = user @Errors::InvalidUnstakeAccount,
    )]
    pub unstake_account: Account<'info, UnstakeAccount>,

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
pub struct EventWithdraw {
    pub user: Pubkey,
    pub unstake_account: Pubkey,
    pub withdraw_amount: u64,
    pub staking_pool: Pubkey,
}

impl<'info> Withdraw<'info> {
    pub fn process(&mut self) -> Result<()> {
        let timestamp = Clock::get()?.unix_timestamp as u64;
        require_gt!(
            self.unstake_account.amount,
            0,
            Errors::UnstakeAccountAmountZero
        );
        require_gte!(
            timestamp,
            self.unstake_account.withdrawable_timestamp,
            Errors::UnstakeAccountNotClaimable
        );

        let withdraw_amount = self.unstake_account.amount;

        require_gte!(
            self.pool_token_account.amount,
            withdraw_amount,
            Errors::PoolBalanceNotEnough
        );

        self.unstake_account.amount = 0;

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
                    &self.staking_pool.token_mint.key().to_bytes(),
                    &self.staking_pool.creator.key().to_bytes(),
                    &[self.staking_pool.index],
                    &[self.staking_pool.pool_seed_bump],
                ]],
            ),
            withdraw_amount,
            self.token_mint.decimals,
        )?;

        emit!(EventWithdraw {
            user: self.user.key(),
            unstake_account: self.unstake_account.key(),
            withdraw_amount,
            staking_pool: self.staking_pool.key()
        });
        Ok(())
    }
}
