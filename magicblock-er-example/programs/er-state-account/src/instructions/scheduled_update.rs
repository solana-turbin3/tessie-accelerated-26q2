use anchor_lang::prelude::*;

use crate::state::UserAccount;

#[derive(Accounts)]
pub struct ScheduledUpdate<'info> {
    /// CHECK: The key is only used as the seed for the user account PDA.
    pub user: UncheckedAccount<'info>,
    #[account(
        mut,
        has_one = user,
        seeds = [b"user", user.key().as_ref()],
        bump = user_account.bump,
    )]
    pub user_account: Account<'info, UserAccount>,
}

impl<'info> ScheduledUpdate<'info> {
    pub fn scheduled_update(&mut self, new_data: u64) -> Result<()> {
        self.user_account.data = new_data;

        Ok(())
    }
}
