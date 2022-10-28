use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    // 0
    #[msg("The account has already been initialized.")]
    AlreadyInitialized,
}