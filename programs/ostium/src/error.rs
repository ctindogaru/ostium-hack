use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    // 0
    #[msg("This account has already been initialized.")]
    AlreadyInitialized,
    // 1
    #[msg("This account has not been initialized.")]
    NotInitialized,
    // 2
    #[msg("This position is not opened.")]
    PositionNotOpened,
    // 3
    #[msg("The amount that you are trying to withdraw is higher than your account balance.")]
    InsufficientFunds,
    // 3
    #[msg("You do not have access to this resource.")]
    PermissionDenied,
}
