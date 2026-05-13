use anchor_lang::{prelude::*, system_program};

use crate::{
    error::ErrorCode, VaultState, WhitelistEntry, VAULT_STATE_SEED,
};

#[derive(Accounts)]
pub struct WhitelistOperations<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        mut,
        seeds = [VAULT_STATE_SEED],
        bump = vault_state.bump,
    )]
    pub vault_state: Account<'info, VaultState>,
    pub system_program: Program<'info, System>,
}

impl<'info> WhitelistOperations<'info> {
    pub fn add_to_whitelist(&mut self, user: Pubkey) -> Result<()> {
        if self.admin.key() != self.vault_state.admin {
            return Err(ErrorCode::UnauthorizedAdmin.into());
        }

        let user_already_exists = self
            .vault_state
            .whitelist
            .iter()
            .any(|entry| entry.user == user);

        if !user_already_exists {
            self.realloc_whitelist(true)?;
            self.vault_state.whitelist.push(WhitelistEntry { user, amount: 0 });
        }

        Ok(())
    }

    pub fn remove_from_whitelist(&mut self, user: Pubkey) -> Result<()> {
        if self.admin.key() != self.vault_state.admin {
            return Err(ErrorCode::UnauthorizedAdmin.into());
        }

        let user_position = self
            .vault_state
            .whitelist
            .iter()
            .position(|entry| entry.user == user);

        if let Some(pos) = user_position {
            self.vault_state.whitelist.remove(pos);
            self.realloc_whitelist(false)?;
        }

        Ok(())
    }

    pub fn realloc_whitelist(&self, is_adding: bool) -> Result<()> {
        let account_info = self.vault_state.to_account_info();

        if is_adding {
            let new_account_size = account_info
                .data_len()
                .checked_add(WhitelistEntry::SPACE)
                .ok_or(ProgramError::ArithmeticOverflow)?;
            let lamports_required = (Rent::get()?).minimum_balance(new_account_size);
            let rent_diff = lamports_required
                .checked_sub(account_info.lamports())
                .ok_or(ProgramError::ArithmeticOverflow)?;

            let cpi_program = self.system_program.key();
            let cpi_accounts = system_program::Transfer {
                from: self.admin.to_account_info(),
                to: account_info.clone(),
            };
            let cpi_context = CpiContext::new(cpi_program, cpi_accounts);
            system_program::transfer(cpi_context, rent_diff)?;

            account_info.resize(new_account_size)?;
            msg!("VaultState size updated: {}", account_info.data_len());
        } else {
            let new_account_size = account_info
                .data_len()
                .checked_sub(WhitelistEntry::SPACE)
                .ok_or(ProgramError::ArithmeticOverflow)?;
            let lamports_required = (Rent::get()?).minimum_balance(new_account_size);
            let rent_diff = account_info
                .lamports()
                .checked_sub(lamports_required)
                .ok_or(ProgramError::ArithmeticOverflow)?;

            account_info.resize(new_account_size)?;
            msg!("VaultState size downgraded: {}", account_info.data_len());

            let admin_info = self.admin.to_account_info();
            let vault_state_info = self.vault_state.to_account_info();
            let mut admin_lamports = admin_info.try_borrow_mut_lamports()?;
            let mut vault_state_lamports = vault_state_info.try_borrow_mut_lamports()?;
            **admin_lamports = (**admin_lamports)
                .checked_add(rent_diff)
                .ok_or(ProgramError::ArithmeticOverflow)?;
            **vault_state_lamports = (**vault_state_lamports)
                .checked_sub(rent_diff)
                .ok_or(ProgramError::ArithmeticOverflow)?;
        }

        Ok(())
    }
}
