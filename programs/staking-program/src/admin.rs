use crate::{Errors, RewardAlgorithm, StakingPool};
use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_interface::{
    transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked,
};

#[derive(Accounts)]
pub struct TransferAdmin<'info> {
    pub admin: Signer<'info>,

    #[account(
        mut,
        has_one = admin @ Errors::AdminNotMatch
    )]
    pub staking_pool: Box<Account<'info, StakingPool>>,
}

impl<'info> TransferAdmin<'info> {
    pub fn process(&mut self, new_admin: Pubkey) -> Result<()> {
        self.staking_pool.pending_admin = new_admin;

        msg!("NewAdmin: {}", new_admin);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct AcceptAdmin<'info> {
    pub pending_admin: Signer<'info>,

    #[account(
        mut,
        has_one = pending_admin @ Errors::PendingAdminNotMatch
    )]
    pub staking_pool: Box<Account<'info, StakingPool>>,
}

impl<'info> AcceptAdmin<'info> {
    pub fn process(&mut self) -> Result<()> {
        self.staking_pool.admin = self.staking_pool.pending_admin;
        self.staking_pool.pending_admin = Pubkey::default();

        msg!("AcceptAdmin: {}", self.staking_pool.admin);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Config<'info> {
    pub admin: Signer<'info>,

    #[account(
        mut,
        has_one = admin @ Errors::AdminNotMatch
    )]
    pub staking_pool: Box<Account<'info, StakingPool>>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, AnchorSerialize, AnchorDeserialize)]
pub struct ConfigParams {
    pub min_stake_amount: Option<u64>,
    pub reward_rate: Option<u64>,
    pub unbonding_seconds: Option<u64>,
    pub reward_algorithm: Option<RewardAlgorithm>,
}

impl<'info> Config<'info> {
    pub fn process(&mut self, config_params: ConfigParams) -> Result<()> {
        if let Some(min_stake_amount) = config_params.min_stake_amount {
            self.staking_pool.min_stake_amount = min_stake_amount;
            msg!("min_stake_amount: {}", min_stake_amount);
        }
        if let Some(reward_rate) = config_params.reward_rate {
            self.staking_pool.update_pool()?;

            self.staking_pool.reward_rate = reward_rate;
            msg!("reward_rate: {}", reward_rate);
        }
        if let Some(unbonding_seconds) = config_params.unbonding_seconds {
            self.staking_pool.unbonding_seconds = unbonding_seconds;
            msg!("unbonding_seconds: {}", unbonding_seconds);
        }
        if let Some(reward_algorithm) = config_params.reward_algorithm {
            self.staking_pool.update_pool()?;

            self.staking_pool.reward_algorithm = reward_algorithm.clone();
            msg!("reward_algorithm: {:?}", reward_algorithm);
        }

        Ok(())
    }
}

#[derive(Accounts)]
pub struct AddRewards<'info> {
    pub admin: Signer<'info>,

    #[account(mut)]
    pub staking_pool: Box<Account<'info, StakingPool>>,

    #[account(
        mut,
        associated_token::mint = token_mint,
        associated_token::authority = admin,
        associated_token::token_program = token_program,
    )]
    pub admin_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = token_mint,
        associated_token::authority = staking_pool,
        associated_token::token_program = token_program,
    )]
    pub pool_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    pub token_mint: Box<InterfaceAccount<'info, Mint>>,

    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> AddRewards<'info> {
    pub fn process(&mut self, amount: u64) -> Result<()> {
        require_gt!(amount, 0, Errors::ParamsNotMatch);

        self.staking_pool.update_pool()?;

        let transfer_to_pool_cpi_context = CpiContext::new(
            self.token_program.to_account_info(),
            TransferChecked {
                from: self.admin_token_account.to_account_info(),
                mint: self.token_mint.to_account_info(),
                to: self.pool_token_account.to_account_info(),
                authority: self.admin.to_account_info(),
            },
        );
        transfer_checked(
            transfer_to_pool_cpi_context,
            amount,
            self.token_mint.decimals,
        )?;

        self.staking_pool.total_reward += amount;

        msg!("AddRewards: {}", amount);
        Ok(())
    }
}
