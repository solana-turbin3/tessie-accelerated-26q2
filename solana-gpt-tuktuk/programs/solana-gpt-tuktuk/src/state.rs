use anchor_lang::prelude::*;

pub const MAX_PROMPT_LEN: usize = 512;
pub const MAX_RESPONSE_LEN: usize = 2048;

#[account]
#[derive(InitSpace)]
pub struct GptResponse {
    pub authority: Pubkey,
    #[max_len(MAX_PROMPT_LEN)]
    pub prompt: String,
    #[max_len(MAX_RESPONSE_LEN)]
    pub response: String,
    pub bump: u8,
}
