use crate::{Errors, StakeAccount, StakingPool, UnstakeAccount};
use anchor_lang::{prelude::*, solana_program::system_program};

#[derive(Accounts)]
pub struct Unstake<'info> {
    pub user: Signer<'info>,

    #[account(
        mut,
        owner = system_program::ID
    )]
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
        init,
        space = 8 + std::mem::size_of::<UnstakeAccount>(),
        payer = rent_payer,
        rent_exempt = enforce,
    )]
    pub unstake_account: Box<Account<'info, UnstakeAccount>>,

    pub system_program: Program<'info, System>,
}

#[event]
pub struct EventUnstake {
    pub staker: Pubkey,
    pub unstake_account: Pubkey,
    pub unstake_amount: u64,
    pub staking_pool: Pubkey,
}

impl<'info> Unstake<'info> {
    pub fn process(&mut self, unstake_amount: u64) -> Result<()> {
        require_gt!(unstake_amount, 0, Errors::UnstakeAmountIsZero);

        require_gte!(
            self.stake_account.amount,
            unstake_amount,
            Errors::BalanceNotEnough
        );

        self.staking_pool.update_pool()?;

        self.stake_account
            .update_reward(self.staking_pool.reward_per_share)?;

        self.stake_account.amount -= unstake_amount;

        self.stake_account
            .update_reward_debt(self.staking_pool.reward_per_share)?;

        let current_time = Clock::get()?.unix_timestamp as u64;
        self.unstake_account.set_inner(UnstakeAccount {
            staking_pool: self.staking_pool.key(),
            user: self.user.key(),
            amount: unstake_amount,
            withdrawable_timestamp: current_time + self.staking_pool.unbonding_seconds,
        });

        emit!(EventUnstake {
            staker: self.user.key(),
            unstake_account: self.unstake_account.key(),
            unstake_amount,
            staking_pool: self.staking_pool.key(),
        });

        Ok(())
    }
}
