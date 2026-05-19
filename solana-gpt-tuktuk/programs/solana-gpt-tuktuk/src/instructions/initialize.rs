use anchor_lang::prelude::*;

use crate::state::GptResponse;

#[derive(Accounts)]
#[instruction(prompt: String)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        init,
        payer = authority,
        space = 8 + GptResponse::INIT_SPACE,
        seeds = [b"gpt-response", authority.key().as_ref()],
        bump
    )]
    pub gpt_response: Account<'info, GptResponse>,
    pub system_program: Program<'info, System>,
}

impl<'info> Initialize<'info> {
    pub fn initialize(&mut self, prompt: String, bumps: &InitializeBumps) -> Result<()> {
        self.gpt_response.set_inner(GptResponse {
            authority: self.authority.key(),
            prompt,
            response: String::new(),
            bump: bumps.gpt_response,
        });

        Ok(())
    }
}
