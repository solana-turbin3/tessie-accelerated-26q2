use pinocchio::{
    AccountView, ProgramResult,
    cpi::{Seed, Signer},
    error::ProgramError,
};
use pinocchio_pubkey::derive_address;

use crate::{
    instructions::{FundraiserError, assert_token_account, close_program_account},
    state::Fundraiser,
};

pub fn process_check_contributions(accounts: &mut [AccountView], _data: &[u8]) -> ProgramResult {
    let [
        maker,
        mint_to_raise,
        fundraiser,
        vault,
        maker_ata,
        system_program,
        token_program,
        _associated_token_program,
        ..,
    ] = accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if !maker.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let (bump, amount_to_raise) = {
        let fundraiser_state = Fundraiser::from_account_view(fundraiser)?;
        if fundraiser_state.maker() != maker.address() {
            return Err(ProgramError::InvalidAccountData);
        }
        if fundraiser_state.mint_to_raise() != mint_to_raise.address() {
            return Err(ProgramError::InvalidAccountData);
        }

        (fundraiser_state.bump, fundraiser_state.amount_to_raise())
    };

    assert_fundraiser_pda(maker.address(), bump, fundraiser.address())?;
    let vault_amount = assert_token_account(vault, fundraiser.address(), mint_to_raise.address())?;
    if vault_amount < amount_to_raise {
        return Err(FundraiserError::TargetNotMet.into());
    }

    if maker_ata.is_data_empty() {
        pinocchio_associated_token_account::instructions::Create {
            funding_account: maker,
            account: maker_ata,
            wallet: maker,
            mint: mint_to_raise,
            token_program,
            system_program,
        }
        .invoke()?;
    }

    assert_token_account(maker_ata, maker.address(), mint_to_raise.address())?;

    let bump_bytes = [bump];
    let signer_seeds = [
        Seed::from(b"fundraiser"),
        Seed::from(maker.address().as_array()),
        Seed::from(bump_bytes.as_ref()),
    ];

    pinocchio_token::instructions::Transfer::new(vault, maker_ata, fundraiser, vault_amount)
        .invoke_signed(&[Signer::from(&signer_seeds)])?;

    pinocchio_token::instructions::CloseAccount::new(vault, maker, fundraiser)
        .invoke_signed(&[Signer::from(&signer_seeds)])?;

    close_program_account(fundraiser, maker)
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
