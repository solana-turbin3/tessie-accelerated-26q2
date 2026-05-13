use anchor_lang::prelude::*;
use anchor_spl::token_interface::{
    mint_to, Mint, MintTo, TokenAccount, TokenInterface,
};

use crate::{
    error::ErrorCode, VaultState, VAULT_AUTHORITY_SEED, VAULT_STATE_SEED,
};

#[derive(Accounts)]
pub struct MintToken<'info> {
    pub admin: Signer<'info>,
    /// CHECK: The recipient only needs to match the token account authority.
    pub recipient: UncheckedAccount<'info>,
    #[account(
        mut,
        mint::token_program = token_program
    )]
    pub mint: InterfaceAccount<'info, Mint>,
    #[account(
        mut,
        token::mint = mint,
        token::authority = recipient,
        token::token_program = token_program
    )]
    pub recipient_token_account: InterfaceAccount<'info, TokenAccount>,
    /// CHECK: This PDA is the mint authority.
    #[account(
        seeds = [VAULT_AUTHORITY_SEED],
        bump = vault_state.vault_authority_bump,
    )]
    pub vault_authority: UncheckedAccount<'info>,
    #[account(
        seeds = [VAULT_STATE_SEED],
        bump = vault_state.bump,
        has_one = mint,
    )]
    pub vault_state: Account<'info, VaultState>,
    pub token_program: Interface<'info, TokenInterface>,
}

impl<'info> MintToken<'info> {
    pub fn mint_token(&mut self, amount: u64) -> Result<()> {
        if self.admin.key() != self.vault_state.admin {
            return Err(ErrorCode::UnauthorizedAdmin.into());
        }

        let recipient_key = self.recipient.key();
        let recipient_is_whitelisted = self
            .vault_state
            .whitelist
            .iter()
            .any(|entry| entry.user == recipient_key);

        if !recipient_is_whitelisted {
            return Err(ErrorCode::UserNotWhitelisted.into());
        }

        let vault_authority_bump = self.vault_state.vault_authority_bump;
        let signer_seeds: [&[&[u8]]; 1] =
            [&[VAULT_AUTHORITY_SEED, &[vault_authority_bump]]];

        let cpi_accounts = MintTo {
            mint: self.mint.to_account_info(),
            to: self.recipient_token_account.to_account_info(),
            authority: self.vault_authority.to_account_info(),
        };

        let cpi_ctx = CpiContext::new_with_signer(
            self.token_program.key(),
            cpi_accounts,
            &signer_seeds,
        );

        mint_to(cpi_ctx, amount)
    }
}
