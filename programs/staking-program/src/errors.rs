use anchor_lang::prelude::*;

#[error_code]
pub enum Errors {
    #[msg("Program id not match")]
    ProgramIdNotMatch,

    #[msg("Remaining accounts not match")]
    RemainingAccountsNotMatch,

    #[msg("Admin not match")]
    AdminNotMatch,

    #[msg("params not match")]
    ParamsNotMatch,

    #[msg("Stake amount too low")]
    StakeAmountTooLow,

    #[msg("Balance not enough")]
    BalanceNotEnough,

    #[msg("Calulation fail")]
    CalculationFail,

    #[msg("Invalid unstake account")]
    InvalidUnstakeAccount,

    #[msg("Invalid stake account")]
    InvalidStakeAccount,

    #[msg("Unstake account not claimable")]
    UnstakeAccountNotClaimable,

    #[msg("Unstake account amount zero")]
    UnstakeAccountAmountZero,

    #[msg("Claim amount zero")]
    ClaimAmountZero,

    #[msg("Pool balance not enough")]
    PoolBalanceNotEnough,

    #[msg("Unstake amount is zero")]
    UnstakeAmountIsZero,

    #[msg("Roken mint account not match")]
    TokenMintAccountNotMatch,

    #[msg("Mint authority not match")]
    MintAuthorityNotMatch,

    #[msg("Pending admin not match")]
    PendingAdminNotMatch,
}
