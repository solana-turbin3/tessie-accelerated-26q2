#![allow(unexpected_cfgs)]

use pinocchio::{
    AccountView, Address, ProgramResult, address::declare_id, entrypoint, error::ProgramError,
};

use crate::instructions::FundraiserInstruction;

mod instructions;
mod state;
mod tests;

entrypoint!(process_instruction);

declare_id!("Eoiuq1dXvHxh6dLx3wh9gj8kSAUpga11krTrbfF5XYsC");

pub const MIN_AMOUNT_TO_RAISE: u64 = 3;
pub const SECONDS_TO_DAYS: i64 = 86_400;
pub const MAX_CONTRIBUTION_PERCENTAGE: u64 = 10;
pub const PERCENTAGE_SCALER: u64 = 100;

pub fn process_instruction(
    program_id: &Address,
    accounts: &mut [AccountView],
    instruction_data: &[u8],
) -> ProgramResult {
    assert_eq!(program_id, &ID);

    let (discriminator, data) = instruction_data
        .split_first()
        .ok_or(ProgramError::InvalidInstructionData)?;

    match FundraiserInstruction::try_from(discriminator)? {
        FundraiserInstruction::Initialize => instructions::process_initialize(accounts, data),
        FundraiserInstruction::Contribute => instructions::process_contribute(accounts, data),
        FundraiserInstruction::CheckContributions => {
            instructions::process_check_contributions(accounts, data)
        }
        FundraiserInstruction::Refund => instructions::process_refund(accounts, data),
    }
}
