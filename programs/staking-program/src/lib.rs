use anchor_lang::{prelude::*, Bumps};

pub mod admin;
pub mod errors;
pub mod helper;
pub mod initialize_staking_pool;
pub mod staker_claim;
pub mod staker_stake;
pub mod staker_unstake;
pub mod staker_withdraw;
pub mod states;

pub use crate::admin::*;
pub use crate::errors::Errors;
pub use crate::helper::*;
pub use crate::initialize_staking_pool::*;
pub use crate::staker_claim::*;
pub use crate::staker_stake::*;
pub use crate::staker_unstake::*;
pub use crate::staker_withdraw::*;
pub use crate::states::*;

declare_id!("ASVEfWrLMRd9YeAWJviTF1CMAd2anTM9o83Y5DNqnmyp");

fn check_context<T: Bumps>(ctx: &Context<T>) -> Result<()> {
    if !check_id(ctx.program_id) {
        return err!(Errors::ProgramIdNotMatch);
    }

    if !ctx.remaining_accounts.is_empty() {
        return err!(Errors::RemainingAccountsNotMatch);
    }

    Ok(())
}

#[program]
pub mod staking_program {

    use super::*;

    // initialize account

    pub fn initialize_staking_pool(
        ctx: Context<InitializeStakingPool>,
        params: InitializeStakingPoolParams,
    ) -> Result<()> {
        check_context(&ctx)?;

        ctx.accounts.process(params, ctx.bumps.staking_pool)?;

        Ok(())
    }

    // admin of stake manager

    pub fn transfer_admin(ctx: Context<TransferAdmin>, new_admin: Pubkey) -> Result<()> {
        check_context(&ctx)?;

        ctx.accounts.process(new_admin)?;

        Ok(())
    }

    pub fn accept_admin(ctx: Context<AcceptAdmin>) -> Result<()> {
        check_context(&ctx)?;

        ctx.accounts.process()?;

        Ok(())
    }

    pub fn config(ctx: Context<Config>, params: ConfigParams) -> Result<()> {
        check_context(&ctx)?;

        ctx.accounts.process(params)?;

        Ok(())
    }

    pub fn add_rewards(ctx: Context<AddRewards>, amount: u64) -> Result<()> {
        check_context(&ctx)?;

        ctx.accounts.process(amount)?;

        Ok(())
    }

    // staker

    pub fn stake(ctx: Context<Stake>, stake_amount: u64) -> Result<()> {
        check_context(&ctx)?;

        ctx.accounts.process(stake_amount)?;

        Ok(())
    }

    pub fn unstake(ctx: Context<Unstake>, unstake_amount: u64) -> Result<()> {
        check_context(&ctx)?;

        ctx.accounts.process(unstake_amount)?;

        Ok(())
    }

    pub fn withdraw(ctx: Context<Withdraw>) -> Result<()> {
        check_context(&ctx)?;

        ctx.accounts.process()?;

        Ok(())
    }

    pub fn claim(ctx: Context<Claim>, restake: bool) -> Result<()> {
        check_context(&ctx)?;

        ctx.accounts.process(restake)?;

        Ok(())
    }
}
