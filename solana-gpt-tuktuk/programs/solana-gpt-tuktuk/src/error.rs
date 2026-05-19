use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Custom error message")]
    CustomError,
    #[msg("GPT response is too long for the allocated account")]
    ResponseTooLong,
}
