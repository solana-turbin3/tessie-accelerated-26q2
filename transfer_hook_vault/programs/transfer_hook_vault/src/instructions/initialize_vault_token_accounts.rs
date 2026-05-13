use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::{
    error::ErrorCode, VaultState, MINT_SEED, VAULT_AUTHORITY_SEED,
    VAULT_STATE_SEED, VAULT_TOKEN_ACCOUNT_SEED,
};

#[derive(Accounts)]
pub struct InitializeVaultTokenAccounts<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        mut,
        seeds = [VAULT_STATE_SEED],
        bump = vault_state.bump,
    )]
    pub vault_state: Account<'info, VaultState>,
    /// CHECK: This PDA is the mint authority, vault authority, and permanent delegate.
    #[account(
        seeds = [VAULT_AUTHORITY_SEED],
        bump = vault_state.vault_authority_bump,
    )]
    pub vault_authority: UncheckedAccount<'info>,
    /// CHECK: The transfer hook program is this program.
    #[account(address = crate::ID)]
    pub transfer_hook_program: UncheckedAccount<'info>,
    #[account(
        init,
        payer = admin,
        seeds = [MINT_SEED],
        bump,
        mint::decimals = 6,
        mint::authority = vault_authority,
        mint::token_program = token_program,
        extensions::transfer_hook::authority = admin,
        extensions::transfer_hook::program_id = transfer_hook_program,
        extensions::permanent_delegate::delegate = vault_authority,
    )]
    pub mint: InterfaceAccount<'info, Mint>,
    #[account(
        init,
        payer = admin,
        seeds = [VAULT_TOKEN_ACCOUNT_SEED],
        bump,
        token::mint = mint,
        token::authority = vault_authority,
        token::token_program = token_program,
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

impl<'info> InitializeVaultTokenAccounts<'info> {
    pub fn initialize_vault_token_accounts(&mut self) -> Result<()> {
        if self.admin.key() != self.vault_state.admin {
            return Err(ErrorCode::UnauthorizedAdmin.into());
        }

        if self.vault_state.mint != Pubkey::default()
            || self.vault_state.vault != Pubkey::default()
        {
            return Err(ErrorCode::VaultTokenAccountsAlreadyInitialized.into());
        }

        self.vault_state.mint = self.mint.key();
        self.vault_state.vault = self.vault.key();

        Ok(())
    }
}
