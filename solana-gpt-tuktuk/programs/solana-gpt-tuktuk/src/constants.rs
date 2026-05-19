use anchor_lang::prelude::*;

#[constant]
pub const SEED: &str = "anchor";

// MagicBlock Solana GPT Oracle identity signer used to verify callback responses.
pub const GPT_ORACLE_IDENTITY: Pubkey = pubkey!("A1ooMmN1fz6LbEFrjh6GukFS2ZeRYFzdyFjeafyyS7Ca");

pub const GPT_ORACLE_PROGRAM_ID: Pubkey = pubkey!("LLMrieZMpbJFwN52WgmBNMxYojrpRVYXdC1RCweEbab");
