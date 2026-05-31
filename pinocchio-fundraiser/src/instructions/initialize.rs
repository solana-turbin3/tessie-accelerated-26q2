use pinocchio::{
    AccountView, ProgramResult,
    cpi::{Seed, Signer},
    error::ProgramError,
    sysvars::{Sysvar, clock::Clock, rent::Rent},
};
use pinocchio_pubkey::derive_address;
use pinocchio_system::instructions::CreateAccount;

use crate::{
    MIN_AMOUNT_TO_RAISE,
    instructions::{FundraiserError, read_u64},
    state::Fundraiser,
};

pub fn process_initialize(accounts: &mut [AccountView], data: &[u8]) -> ProgramResult {
    let [
        maker,
        mint_to_raise,
        fundraiser,
        vault,
        system_program,
        token_program,
        _associated_token_program,
        ..,
    ] = accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if data.len() < 9 {
        return Err(ProgramError::InvalidInstructionData);
    }
    if !maker.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let amount_to_raise = read_u64(data, 0)?;
    let duration = data[8];
    let mint = pinocchio_token::state::Mint::from_account_view(mint_to_raise)?;
    let min_amount = MIN_AMOUNT_TO_RAISE
        .checked_mul(10_u64.pow(mint.decimals() as u32))
        .ok_or(ProgramError::ArithmeticOverflow)?;

    if amount_to_raise <= min_amount {
        return Err(FundraiserError::InvalidAmount.into());
    }

    let bump = data
        .get(9)
        .copied()
        .ok_or(ProgramError::InvalidInstructionData)?;
    let seeds = [b"fundraiser".as_ref(), maker.address().as_ref(), &[bump]];
    let expected = derive_address(&seeds, None, &crate::ID.to_bytes());
    if expected != *fundraiser.address().as_array() {
        return Err(ProgramError::InvalidSeeds);
    }

    let bump_bytes = [bump];
    let signer_seeds = [
        Seed::from(b"fundraiser"),
        Seed::from(maker.address().as_array()),
        Seed::from(bump_bytes.as_ref()),
    ];

    CreateAccount {
        from: maker,
        to: fundraiser,
        lamports: Rent::get()?.try_minimum_balance(Fundraiser::LEN)?,
        space: Fundraiser::LEN as u64,
        owner: &crate::ID,
    }
    .invoke_signed(&[Signer::from(&signer_seeds)])?;

    pinocchio_associated_token_account::instructions::Create {
        funding_account: maker,
        account: vault,
        wallet: fundraiser,
        mint: mint_to_raise,
        token_program,
        system_program,
    }
    .invoke()?;

    let clock = Clock::get()?;
    let fundraiser_state = Fundraiser::from_account_view(fundraiser)?;
    fundraiser_state.set_maker(maker.address());
    fundraiser_state.set_mint_to_raise(mint_to_raise.address());
    fundraiser_state.set_amount_to_raise(amount_to_raise);
    fundraiser_state.set_current_amount(0);
    fundraiser_state.set_time_started(clock.unix_timestamp);
    fundraiser_state.duration = duration;
    fundraiser_state.bump = bump;

    Ok(())
}
