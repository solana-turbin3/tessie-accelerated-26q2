pub mod check_contributions;
pub mod contribute;
pub mod initialize;
pub mod refund;

pub use check_contributions::*;
pub use contribute::*;
pub use initialize::*;
pub use refund::*;

use pinocchio::error::ProgramError;

pub enum FundraiserInstruction {
    Initialize = 0,
    Contribute = 1,
    CheckContributions = 2,
    Refund = 3,
}

impl TryFrom<&u8> for FundraiserInstruction {
    type Error = ProgramError;

    fn try_from(value: &u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Initialize),
            1 => Ok(Self::Contribute),
            2 => Ok(Self::CheckContributions),
            3 => Ok(Self::Refund),
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }
}

#[repr(u32)]
pub enum FundraiserError {
    TargetNotMet = 100,
    TargetMet = 101,
    ContributionTooBig = 102,
    ContributionTooSmall = 103,
    MaximumContributionsReached = 104,
    FundraiserNotEnded = 105,
    FundraiserEnded = 106,
    InvalidAmount = 107,
}

impl From<FundraiserError> for ProgramError {
    fn from(value: FundraiserError) -> Self {
        ProgramError::Custom(value as u32)
    }
}

#[inline(always)]
pub fn read_u64(data: &[u8], offset: usize) -> Result<u64, ProgramError> {
    if data.len() < offset + 8 {
        return Err(ProgramError::InvalidInstructionData);
    }

    Ok(u64::from_le_bytes(
        data[offset..offset + 8]
            .try_into()
            .map_err(|_| ProgramError::InvalidInstructionData)?,
    ))
}

#[inline(always)]
pub fn close_program_account(
    account: &mut pinocchio::AccountView,
    destination: &mut pinocchio::AccountView,
) -> Result<(), ProgramError> {
    let lamports = account.lamports();
    let new_destination_lamports = destination
        .lamports()
        .checked_add(lamports)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    destination.set_lamports(new_destination_lamports);
    account.set_lamports(0);

    unsafe {
        account.borrow_unchecked_mut().fill(0);
        account.assign(&pinocchio_system::ID);
    }

    Ok(())
}

#[inline(always)]
pub fn assert_token_account(
    account: &pinocchio::AccountView,
    owner: &pinocchio::Address,
    mint: &pinocchio::Address,
) -> Result<u64, ProgramError> {
    let token_account = pinocchio_token::state::Account::from_account_view(account)?;

    if token_account.owner() != owner {
        return Err(ProgramError::IllegalOwner);
    }
    if token_account.mint() != mint {
        return Err(ProgramError::InvalidAccountData);
    }

    Ok(token_account.amount())
}
