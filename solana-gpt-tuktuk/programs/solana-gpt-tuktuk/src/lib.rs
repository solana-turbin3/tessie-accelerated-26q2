#![allow(ambiguous_glob_reexports)]

pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("BhvCFy3z2YyZsbL8oKJ9NoJWDuJmX7PTsJTQAWxwM6SY");

#[program]
pub mod solana_gpt_tuktuk {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, prompt: String) -> Result<()> {
        ctx.accounts.initialize(prompt, &ctx.bumps)
    }

    pub fn receive_gpt_response(ctx: Context<ReceiveGptResponse>, response: String) -> Result<()> {
        ctx.accounts.receive_gpt_response(response)
    }

    pub fn schedule_gpt(ctx: Context<ScheduleGpt>, task_id: u16) -> Result<()> {
        ctx.accounts.schedule_gpt(task_id, &ctx.bumps)
    }
}
