use pinocchio::{
    AccountView, ProgramResult,
    cpi::{Seed, Signer},
    error::ProgramError,
    sysvars::{Sysvar, clock::Clock, rent::Rent},
};
use pinocchio_pubkey::derive_address;
use pinocchio_system::instructions::CreateAccount;

use crate::{
    MAX_CONTRIBUTION_PERCENTAGE, PERCENTAGE_SCALER, SECONDS_TO_DAYS,
    instructions::{FundraiserError, assert_token_account, read_u64},
    state::{Contributor, Fundraiser},
};

pub fn process_contribute(accounts: &mut [AccountView], data: &[u8]) -> ProgramResult {
    let [
        contributor,
        mint_to_raise,
        fundraiser,
        contributor_account,
        contributor_ata,
        vault,
        _system_program,
        _token_program,
        ..,
    ] = accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if !contributor.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let amount = read_u64(data, 0)?;
    if amount <= 1 {
        return Err(FundraiserError::ContributionTooSmall.into());
    }

    let (maker, fundraiser_bump, amount_to_raise, time_started, duration) = {
        let fundraiser_state = Fundraiser::from_account_view(fundraiser)?;
        if fundraiser_state.mint_to_raise() != mint_to_raise.address() {
            return Err(ProgramError::InvalidAccountData);
        }

        (
            *fundraiser_state.maker(),
            fundraiser_state.bump,
            fundraiser_state.amount_to_raise(),
            fundraiser_state.time_started(),
            fundraiser_state.duration,
        )
    };

    assert_fundraiser_pda(&maker, fundraiser_bump, fundraiser.address())?;
    assert_token_account(
        contributor_ata,
        contributor.address(),
        mint_to_raise.address(),
    )?;
    assert_token_account(vault, fundraiser.address(), mint_to_raise.address())?;

    let max_contribution = amount_to_raise
        .checked_mul(MAX_CONTRIBUTION_PERCENTAGE)
        .ok_or(ProgramError::ArithmeticOverflow)?
        / PERCENTAGE_SCALER;
    if amount > max_contribution {
        return Err(FundraiserError::ContributionTooBig.into());
    }

    let elapsed_days = elapsed_days(time_started)?;
    if elapsed_days >= duration {
        return Err(FundraiserError::FundraiserEnded.into());
    }

    let contributor_bump = data
        .get(8)
        .copied()
        .ok_or(ProgramError::InvalidInstructionData)?;
    let contributor_seeds = [
        b"contributor".as_ref(),
        fundraiser.address().as_ref(),
        contributor.address().as_ref(),
        &[contributor_bump],
    ];
    let expected_contributor = derive_address(&contributor_seeds, None, &crate::ID.to_bytes());
    if expected_contributor != *contributor_account.address().as_array() {
        return Err(ProgramError::InvalidSeeds);
    }

    if contributor_account.is_data_empty() {
        let bump_bytes = [contributor_bump];
        let signer_seeds = [
            Seed::from(b"contributor"),
            Seed::from(fundraiser.address().as_array()),
            Seed::from(contributor.address().as_array()),
            Seed::from(bump_bytes.as_ref()),
        ];

        CreateAccount {
            from: contributor,
            to: contributor_account,
            lamports: Rent::get()?.try_minimum_balance(Contributor::LEN)?,
            space: Contributor::LEN as u64,
            owner: &crate::ID,
        }
        .invoke_signed(&[Signer::from(&signer_seeds)])?;
    }

    {
        let contributor_state = Contributor::from_account_view(contributor_account)?;
        let new_total = contributor_state
            .amount()
            .checked_add(amount)
            .ok_or(ProgramError::ArithmeticOverflow)?;
        if new_total > max_contribution {
            return Err(FundraiserError::MaximumContributionsReached.into());
        }
    }

    pinocchio_token::instructions::Transfer::new(contributor_ata, vault, contributor, amount)
        .invoke()?;

    let fundraiser_state = Fundraiser::from_account_view(fundraiser)?;
    fundraiser_state.add_current_amount(amount)?;

    let contributor_state = Contributor::from_account_view(contributor_account)?;
    contributor_state.bump = contributor_bump;
    contributor_state.add_amount(amount)?;

    Ok(())
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
