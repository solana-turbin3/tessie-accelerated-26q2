use anchor_lang::prelude::*;

#[account]
pub struct VaultState {
    pub admin: Pubkey,
    pub mint: Pubkey,
    pub vault: Pubkey,
    pub bump: u8,
    pub vault_authority_bump: u8,
    pub whitelist: Vec<WhitelistEntry>,
}

impl VaultState {
    pub const INIT_SPACE: usize = 32 + 32 + 32 + 1 + 1 + 4;
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct WhitelistEntry {
    pub user: Pubkey,
    pub amount: u64,
}

impl WhitelistEntry {
    pub const SPACE: usize = 32 + 8;
}
