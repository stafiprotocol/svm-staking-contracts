pub use crate::errors::Errors;
use crate::helper;
use anchor_lang::prelude::*;

#[account]
#[derive(Debug)]
pub struct StakingPool {
    pub creator: Pubkey,
    pub index: u8,
    pub admin: Pubkey,
    pub pending_admin: Pubkey,
    pub pool_seed_bump: u8,
    pub token_mint: Pubkey,

    pub min_stake_amount: u64,
    pub unbonding_seconds: u64,

    /// For FixedPerTokenPerSecond: per staked smallest unit per second.
    ///
    ///     Reward rate is scaled by 1e12 to support fractional values.
    ///     Reward rate is in **smallest token unit per second(after scaling)**.
    ///
    /// For FixedTotalPerSecond: total reward per second in smallest units.
    ///
    ///     Reward rate is in **smallest token unit per second**.
    pub reward_rate: u64,
    pub reward_algorithm: RewardAlgorithm,

    pub total_stake: u64,
    pub total_reward: u64,
    pub undistributed_reward: u64,
    pub last_reward_timestamp: u64,
    pub reward_per_share: u128,

    /// Reserved space for future upgrades. Do not use.
    pub _reserved: [u8; 256],
}

#[derive(Clone, Copy, Debug, AnchorSerialize, AnchorDeserialize, PartialEq, Eq)]
pub enum RewardAlgorithm {
    FixedPerTokenPerSecond,
    FixedTotalPerSecond,
}

impl StakingPool {
    pub fn calc_new_reward(&self, time_diff: u64) -> Result<u64> {
        match self.reward_algorithm {
            RewardAlgorithm::FixedPerTokenPerSecond => u64::try_from(
                (self.total_stake as u128) * (time_diff as u128) * (self.reward_rate as u128)
                    / helper::REWARD_CALC_BASE,
            )
            .map_err(|_| error!(Errors::CalculationFail)),
            RewardAlgorithm::FixedTotalPerSecond => {
                u64::try_from((time_diff as u128) * (self.reward_rate as u128))
                    .map_err(|_| error!(Errors::CalculationFail))
            }
        }
    }

    pub fn calc_reward_per_share(&self, reward: u64) -> Result<u128> {
        Ok(
            (reward as u128) * helper::REWARD_CALC_BASE / (self.total_stake as u128)
                + self.reward_per_share,
        )
    }

    pub fn calc_reward_debt(&mut self, amount: u64) -> Result<u64> {
        u64::try_from(self.reward_per_share * (amount as u128) / helper::REWARD_CALC_BASE)
            .map_err(|_| error!(Errors::CalculationFail))
    }

    pub fn update_pool(&mut self) -> Result<()> {
        let current_time = Clock::get()?.unix_timestamp as u64;

        if current_time <= self.last_reward_timestamp {
            return Ok(());
        }

        if self.total_stake == 0 {
            self.last_reward_timestamp = current_time;
            return Ok(());
        }

        let time_diff = current_time - self.last_reward_timestamp;
        let mut reward = self.calc_new_reward(time_diff)?;

        if reward > 0 {
            if self.undistributed_reward >= reward {
                self.undistributed_reward -= reward;
            } else {
                reward = self.undistributed_reward;
                self.undistributed_reward = 0;
            }

            self.reward_per_share = self.calc_reward_per_share(reward)?;
        }

        self.last_reward_timestamp = current_time;

        Ok(())
    }
}

#[account]
#[derive(Debug)]
pub struct StakeAccount {
    pub staking_pool: Pubkey,
    pub user: Pubkey,
    pub amount: u64,
    pub reward: u64,
    pub reward_debt: u64,

    /// Reserved space for future upgrades. Do not use.
    pub _reserved: [u8; 128],
}

impl StakeAccount {
    pub fn update_reward(&mut self, reward_per_share: u128) -> Result<()> {
        self.reward = u64::try_from(
            (self.amount as u128) * reward_per_share / helper::REWARD_CALC_BASE
                + (self.reward as u128)
                - (self.reward_debt as u128),
        )
        .map_err(|_| error!(Errors::CalculationFail))?;

        Ok(())
    }

    pub fn update_reward_debt(&mut self, reward_per_share: u128) -> Result<()> {
        self.reward_debt =
            u64::try_from((self.amount as u128) * reward_per_share / helper::REWARD_CALC_BASE)
                .map_err(|_| error!(Errors::CalculationFail))?;

        Ok(())
    }
}

#[account]
#[derive(Debug)]
pub struct UnstakeAccount {
    pub staking_pool: Pubkey,
    pub user: Pubkey,
    pub amount: u64,
    pub withdrawable_timestamp: u64,

    /// Reserved space for future upgrades. Do not use.
    pub _reserved: [u8; 128],
}
