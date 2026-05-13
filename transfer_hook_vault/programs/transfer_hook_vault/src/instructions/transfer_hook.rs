use std::cell::RefMut;

use anchor_lang::prelude::*;
use anchor_spl::{
    token_2022::spl_token_2022::{
        extension::{
            transfer_hook::TransferHookAccount, BaseStateWithExtensionsMut,
            PodStateWithExtensionsMut,
        },
        pod::PodAccount,
    },
    token_interface::{Mint, TokenAccount},
};

use crate::{error::ErrorCode, VaultState, VAULT_STATE_SEED};

#[derive(Accounts)]
pub struct TransferHook<'info> {
    #[account(
        token::mint = mint,
        token::authority = owner,
    )]
    pub source_token: InterfaceAccount<'info, TokenAccount>,
    pub mint: InterfaceAccount<'info, Mint>,
    #[account(
        token::mint = mint,
    )]
    pub destination_token: InterfaceAccount<'info, TokenAccount>,
    /// CHECK: Source token account owner. It can be a wallet or PDA.
    pub owner: UncheckedAccount<'info>,
    /// CHECK: ExtraAccountMetaList account for this mint.
    #[account(
        seeds = [b"extra-account-metas", mint.key().as_ref()],
        bump
    )]
    pub extra_account_meta_list: UncheckedAccount<'info>,
    #[account(
        seeds = [VAULT_STATE_SEED],
        bump = vault_state.bump,
        has_one = mint,
    )]
    pub vault_state: Account<'info, VaultState>,
}

impl<'info> TransferHook<'info> {
    pub fn transfer_hook(&mut self, _amount: u64) -> Result<()> {
        self.check_is_transferring()?;

        let source_is_vault = self.source_token.key() == self.vault_state.vault;
        let destination_is_vault = self.destination_token.key() == self.vault_state.vault;

        if !source_is_vault && !destination_is_vault {
            return Ok(());
        }

        let interacting_user = if destination_is_vault {
            self.owner.key()
        } else {
            self.destination_token.owner
        };

        let user_is_whitelisted = self
            .vault_state
            .whitelist
            .iter()
            .any(|entry| entry.user == interacting_user);

        if !user_is_whitelisted {
            return Err(ErrorCode::UserNotWhitelisted.into());
        }

        Ok(())
    }

    fn check_is_transferring(&mut self) -> Result<()> {
        let source_token_info = self.source_token.to_account_info();
        let mut account_data_ref: RefMut<&mut [u8]> = source_token_info.try_borrow_mut_data()?;
        let mut account = PodStateWithExtensionsMut::<PodAccount>::unpack(*account_data_ref)?;
        let account_extension = account.get_extension_mut::<TransferHookAccount>()?;

        if !bool::from(account_extension.transferring) {
            panic!("TransferHook: Not transferring");
        }

        Ok(())
    }
}
