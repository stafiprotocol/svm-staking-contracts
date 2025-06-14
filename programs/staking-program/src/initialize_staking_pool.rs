pub use crate::errors::Errors;
pub use crate::StakingPool;
use crate::{helper, RewardAlgorithm};
use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_interface::{
    transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked,
};

#[derive(Accounts)]
pub struct InitializeStakingPool<'info> {
    pub admin: Signer<'info>,

    #[account(mut)]
    pub rent_payer: Signer<'info>,

    pub token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        init,
        space = 8 + std::mem::size_of::<StakingPool>(),
        payer = rent_payer,
        rent_exempt = enforce,
        seeds = [
            helper::POOL_SEED,
            &token_mint.key().to_bytes(),
            &admin.key().to_bytes(),
        ],
        bump,
    )]
    pub staking_pool: Box<Account<'info, StakingPool>>,

    #[account(
        init_if_needed,
        payer = rent_payer,
        associated_token::mint = token_mint,
        associated_token::authority = admin,
        associated_token::token_program = token_program,
    )]
    pub admin_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        init,
        payer = rent_payer,
        associated_token::mint = token_mint,
        associated_token::authority = staking_pool,
        associated_token::token_program = token_program,
    )]
    pub pool_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

#[repr(C)]
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct InitializeStakingPoolParams {
    pub reward_rate: u64,
    pub total_reward: u64,
    pub unbonding_seconds: u64,
    pub reward_algorithm: RewardAlgorithm,
}

impl<'info> InitializeStakingPool<'info> {
    pub fn process(
        &mut self,
        params: InitializeStakingPoolParams,
        pool_seed_bump: u8,
    ) -> Result<()> {
        require_gt!(params.reward_rate, 0, Errors::ParamsNotMatch);
        require_gt!(params.unbonding_seconds, 0, Errors::ParamsNotMatch);

        if params.total_reward > 0 {
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
                params.total_reward,
                self.token_mint.decimals,
            )?;
        }

        self.staking_pool.set_inner(StakingPool {
            creator: self.admin.key(),
            admin: self.admin.key(),
            pending_admin: Pubkey::default(),
            pool_seed_bump,
            token_mint: self.token_mint.key(),
            min_stake_amount: helper::DEFAULT_MIN_STAKE_AMOUNT,
            total_stake: 0,
            total_reward: params.total_reward,
            reward_rate: params.reward_rate,
            reward_algorithm: params.reward_algorithm,
            last_reward_timestamp: 0,
            reward_per_share: 0,
            unbonding_seconds: params.unbonding_seconds,
            _reserved: [0u8; 256],
        });

        Ok(())
    }
}
