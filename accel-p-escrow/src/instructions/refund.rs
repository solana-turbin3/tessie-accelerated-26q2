use pinocchio::{
    AccountView, Address, ProgramResult,
    cpi::{Seed, Signer},
    error::ProgramError,
};
use pinocchio_pubkey::derive_address;

use crate::state::Escrow;

pub fn process_refund_instruction(accounts: &mut [AccountView], _data: &[u8]) -> ProgramResult {
    let [
        maker,
        mint_a,
        escrow_account,
        maker_ata,
        escrow_ata,
        _token_program,
        _system_program,
        ..,
    ] = accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if !maker.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let (bump, amount_to_give) = {
        let escrow_state = Escrow::from_account_info(escrow_account)?;

        if escrow_state.maker() != maker.address() {
            return Err(ProgramError::InvalidAccountData);
        }
        if escrow_state.mint_a() != mint_a.address() {
            return Err(ProgramError::InvalidAccountData);
        }

        (escrow_state.bump, escrow_state.amount_to_give())
    };

    assert_escrow_pda(maker.address(), bump, escrow_account.address())?;
    assert_token_account(maker_ata, maker.address(), mint_a.address())?;
    assert_token_account(escrow_ata, escrow_account.address(), mint_a.address())?;

    let bump_bytes = [bump];
    let signer_seeds = [
        Seed::from(b"escrow"),
        Seed::from(maker.address().as_array()),
        Seed::from(bump_bytes.as_ref()),
    ];
    pinocchio_token::instructions::Transfer::new(
        escrow_ata,
        maker_ata,
        escrow_account,
        amount_to_give,
    )
    .invoke_signed(&[Signer::from(&signer_seeds)])?;

    pinocchio_token::instructions::CloseAccount::new(escrow_ata, maker, escrow_account)
        .invoke_signed(&[Signer::from(&signer_seeds)])?;

    close_escrow_account(escrow_account, maker)
}

fn assert_escrow_pda(maker: &Address, bump: u8, escrow: &Address) -> ProgramResult {
    let seed = [b"escrow".as_ref(), maker.as_ref(), &[bump]];
    let expected = derive_address(&seed, None, &crate::ID.to_bytes());

    if expected != *escrow.as_array() {
        return Err(ProgramError::InvalidSeeds);
    }

    Ok(())
}

fn assert_token_account(
    account: &AccountView,
    expected_owner: &Address,
    expected_mint: &Address,
) -> ProgramResult {
    let token_account = pinocchio_token::state::Account::from_account_view(account)?;

    if token_account.owner() != expected_owner {
        return Err(ProgramError::IllegalOwner);
    }
    if token_account.mint() != expected_mint {
        return Err(ProgramError::InvalidAccountData);
    }

    Ok(())
}

fn close_escrow_account(
    escrow_account: &mut AccountView,
    maker: &mut AccountView,
) -> ProgramResult {
    let escrow_lamports = escrow_account.lamports();
    let maker_lamports = maker
        .lamports()
        .checked_add(escrow_lamports)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    maker.set_lamports(maker_lamports);
    escrow_account.set_lamports(0);

    unsafe {
        escrow_account.borrow_unchecked_mut().fill(0);
        escrow_account.assign(&pinocchio_system::ID);
    }

    Ok(())
}
