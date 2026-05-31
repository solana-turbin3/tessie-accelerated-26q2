use pinocchio::{
    AccountView, ProgramResult,
    cpi::{Seed, Signer},
    error::ProgramError,
    sysvars::{Sysvar, clock::Clock},
};
use pinocchio_pubkey::derive_address;

use crate::{
    SECONDS_TO_DAYS,
    instructions::{FundraiserError, assert_token_account, close_program_account},
    state::{Contributor, Fundraiser},
};

pub fn process_refund(accounts: &mut [AccountView], _data: &[u8]) -> ProgramResult {
    let [
        contributor,
        maker,
        mint_to_raise,
        fundraiser,
        contributor_account,
        contributor_ata,
        vault,
        _token_program,
        _system_program,
        ..,
    ] = accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if !contributor.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let (bump, time_started, duration, amount_to_raise) = {
        let fundraiser_state = Fundraiser::from_account_view(fundraiser)?;
        if fundraiser_state.maker() != maker.address() {
            return Err(ProgramError::InvalidAccountData);
        }
        if fundraiser_state.mint_to_raise() != mint_to_raise.address() {
            return Err(ProgramError::InvalidAccountData);
        }

        (
            fundraiser_state.bump,
            fundraiser_state.time_started(),
            fundraiser_state.duration,
            fundraiser_state.amount_to_raise(),
        )
    };

    assert_fundraiser_pda(maker.address(), bump, fundraiser.address())?;

    let contributor_state = Contributor::from_account_view(contributor_account)?;
    let contributor_amount = contributor_state.amount();
    assert_contributor_pda(
        fundraiser.address(),
        contributor.address(),
        contributor_state.bump,
        contributor_account.address(),
    )?;
    assert_token_account(
        contributor_ata,
        contributor.address(),
        mint_to_raise.address(),
    )?;
    let vault_amount = assert_token_account(vault, fundraiser.address(), mint_to_raise.address())?;

    if elapsed_days(time_started)? < duration {
        return Err(FundraiserError::FundraiserNotEnded.into());
    }
    if vault_amount >= amount_to_raise {
        return Err(FundraiserError::TargetMet.into());
    }

    let bump_bytes = [bump];
    let signer_seeds = [
        Seed::from(b"fundraiser"),
        Seed::from(maker.address().as_array()),
        Seed::from(bump_bytes.as_ref()),
    ];

    pinocchio_token::instructions::Transfer::new(
        vault,
        contributor_ata,
        fundraiser,
        contributor_amount,
    )
    .invoke_signed(&[Signer::from(&signer_seeds)])?;

    let fundraiser_state = Fundraiser::from_account_view(fundraiser)?;
    fundraiser_state.sub_current_amount(contributor_amount)?;

    close_program_account(contributor_account, contributor)
}

#[inline(always)]
fn elapsed_days(time_started: i64) -> Result<u8, ProgramError> {
    let elapsed_seconds = Clock::get()?
        .unix_timestamp
        .checked_sub(time_started)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    Ok((elapsed_seconds / SECONDS_TO_DAYS) as u8)
}

#[inline(always)]
fn assert_fundraiser_pda(
    maker: &pinocchio::Address,
    bump: u8,
    fundraiser: &pinocchio::Address,
) -> ProgramResult {
    let seeds = [b"fundraiser".as_ref(), maker.as_ref(), &[bump]];
    let expected = derive_address(&seeds, None, &crate::ID.to_bytes());
    if expected != *fundraiser.as_array() {
        return Err(ProgramError::InvalidSeeds);
    }
    Ok(())
}

#[inline(always)]
fn assert_contributor_pda(
    fundraiser: &pinocchio::Address,
    contributor: &pinocchio::Address,
    bump: u8,
    contributor_account: &pinocchio::Address,
) -> ProgramResult {
    let bump_bytes = [bump];
    let seeds = [
        b"contributor".as_ref(),
        fundraiser.as_ref(),
        contributor.as_ref(),
        bump_bytes.as_ref(),
    ];
    let expected = derive_address(&seeds, None, &crate::ID.to_bytes());
    if expected != *contributor_account.as_array() {
        return Err(ProgramError::InvalidSeeds);
    }

    Ok(())
}
