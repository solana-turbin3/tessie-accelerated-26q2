pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;
use spl_discriminator::SplDiscriminate;
use spl_tlv_account_resolution::state::ExtraAccountMetaList;
use spl_transfer_hook_interface::instruction::ExecuteInstruction;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("HaH6qJkNS6WkxjyjtKCNNRqLo6WLioubocXYXJjPCJ9Y");

#[program]
pub mod transfer_hook_vault {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        ctx.accounts.initialize(ctx.bumps)
    }

    pub fn initialize_vault_token_accounts(
        ctx: Context<InitializeVaultTokenAccounts>,
    ) -> Result<()> {
        ctx.accounts.initialize_vault_token_accounts()
    }

    pub fn add_to_whitelist(ctx: Context<WhitelistOperations>, user: Pubkey) -> Result<()> {
        ctx.accounts.add_to_whitelist(user)
    }

    pub fn remove_from_whitelist(ctx: Context<WhitelistOperations>, user: Pubkey) -> Result<()> {
        ctx.accounts.remove_from_whitelist(user)
    }

    pub fn mint_token(ctx: Context<MintToken>, amount: u64) -> Result<()> {
        ctx.accounts.mint_token(amount)
    }

    pub fn initialize_transfer_hook(
        ctx: Context<InitializeExtraAccountMetaList>,
    ) -> Result<()> {
        let extra_account_metas = InitializeExtraAccountMetaList::extra_account_metas()?;

        ExtraAccountMetaList::init::<ExecuteInstruction>(
            &mut ctx
                .accounts
                .extra_account_meta_list
                .try_borrow_mut_data()?,
            &extra_account_metas,
        )?;

        Ok(())
    }

    #[instruction(discriminator = ExecuteInstruction::SPL_DISCRIMINATOR_SLICE)]
    pub fn transfer_hook(ctx: Context<TransferHook>, amount: u64) -> Result<()> {
        ctx.accounts.transfer_hook(amount)
    }
}
