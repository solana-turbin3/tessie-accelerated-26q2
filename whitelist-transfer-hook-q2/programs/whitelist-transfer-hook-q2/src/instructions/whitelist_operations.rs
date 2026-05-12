use anchor_lang::prelude::*;

use crate::state::Whitelist;

#[derive(Accounts)]
#[instruction(user: Pubkey)]
pub struct AddToWhitelist<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        init_if_needed,
        payer = admin,
        space = 8 + 1,
        seeds = [b"whitelist", user.as_ref()],
        bump
    )]
    pub whitelist: Account<'info, Whitelist>,
    pub system_program: Program<'info, System>,
}

impl<'info> AddToWhitelist<'info> {
    pub fn add_to_whitelist(&mut self, bumps: AddToWhitelistBumps) -> Result<()> {
        self.whitelist.set_inner(Whitelist {
            bump: bumps.whitelist,
        });

        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(user: Pubkey)]
pub struct RemoveFromWhitelist<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        mut,
        close = admin,
        seeds = [b"whitelist", user.as_ref()],
        bump = whitelist.bump,
    )]
    pub whitelist: Account<'info, Whitelist>,
}

impl<'info> RemoveFromWhitelist<'info> {
    pub fn remove_from_whitelist(&mut self) -> Result<()> {
        Ok(())
    }
}
