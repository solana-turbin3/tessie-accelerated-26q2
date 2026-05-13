use anchor_lang::prelude::*;

use crate::{VaultState, VAULT_AUTHORITY_SEED, VAULT_STATE_SEED};

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        init,
        payer = admin,
        space = 8 + VaultState::INIT_SPACE,
        seeds = [VAULT_STATE_SEED],
        bump
    )]
    pub vault_state: Account<'info, VaultState>,
    /// CHECK: This PDA signs for the vault token account later.
    #[account(
        seeds = [VAULT_AUTHORITY_SEED],
        bump
    )]
    pub vault_authority: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

impl<'info> Initialize<'info> {
    pub fn initialize(&mut self, bumps: InitializeBumps) -> Result<()> {
        self.vault_state.set_inner(VaultState {
            admin: self.admin.key(),
            mint: Pubkey::default(),
            vault: Pubkey::default(),
            bump: bumps.vault_state,
            vault_authority_bump: bumps.vault_authority,
            whitelist: Vec::new(),
        });

        Ok(())
    }
}
