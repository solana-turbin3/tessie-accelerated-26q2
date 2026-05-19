use anchor_lang::prelude::*;

use crate::{error::ErrorCode, state::GptResponse, GPT_ORACLE_IDENTITY, MAX_RESPONSE_LEN};

#[derive(Accounts)]
pub struct ReceiveGptResponse<'info> {
    #[account(address = GPT_ORACLE_IDENTITY)]
    pub oracle_identity: Signer<'info>,
    #[account(
        mut,
        seeds = [b"gpt-response", gpt_response.authority.as_ref()],
        bump = gpt_response.bump,
    )]
    pub gpt_response: Account<'info, GptResponse>,
}

impl<'info> ReceiveGptResponse<'info> {
    pub fn receive_gpt_response(&mut self, response: String) -> Result<()> {
        require!(
            response.as_bytes().len() <= MAX_RESPONSE_LEN,
            ErrorCode::ResponseTooLong
        );

        self.gpt_response.response = response;

        Ok(())
    }
}
