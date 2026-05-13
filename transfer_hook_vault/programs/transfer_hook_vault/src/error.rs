use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Only the vault admin can perform this action")]
    UnauthorizedAdmin,
    #[msg("The whitelist is full")]
    WhitelistFull,
    #[msg("The user is already whitelisted")]
    UserAlreadyWhitelisted,
    #[msg("The user is not whitelisted")]
    UserNotWhitelisted,
    #[msg("The vault token accounts have already been initialized")]
    VaultTokenAccountsAlreadyInitialized,
}
