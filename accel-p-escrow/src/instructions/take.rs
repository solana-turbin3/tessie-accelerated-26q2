use pinocchio::{
    AccountView, Address, ProgramResult,
    cpi::{Seed, Signer},
    error::ProgramError,
};
use pinocchio_pubkey::derive_address;

use crate::state::Escrow;

pub fn process_take_instruction(accounts: &mut [AccountView], _data: &[u8]) -> ProgramResult {
    let [
        taker,
        maker,
        mint_a,
        mint_b,
        escrow_account,
        escrow_ata,
        taker_ata_a,
        taker_ata_b,
        maker_ata_b,
        system_program,
        token_program,
        _associated_token_program,
        ..,
    ] = accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if !taker.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let (bump, amount_to_receive, amount_to_give) = {
        let escrow_state = Escrow::from_account_info(escrow_account)?;

        if escrow_state.maker() != maker.address() {
            return Err(ProgramError::InvalidAccountData);
        }
        if escrow_state.mint_a() != mint_a.address() {
            return Err(ProgramError::InvalidAccountData);
        }
        if escrow_state.mint_b() != mint_b.address() {
            return Err(ProgramError::InvalidAccountData);
        }

        (
            escrow_state.bump,
            escrow_state.amount_to_receive(),
            escrow_state.amount_to_give(),
        )
    };

    assert_escrow_pda(maker.address(), bump, escrow_account.address())?;
    assert_token_account(escrow_ata, escrow_account.address(), mint_a.address())?;
    assert_token_account(taker_ata_b, taker.address(), mint_b.address())?;

    if taker_ata_a.is_data_empty() {
        pinocchio_associated_token_account::instructions::Create {
            funding_account: taker,
            account: taker_ata_a,
            wallet: taker,
            mint: mint_a,
            token_program,
            system_program,
        }
        .invoke()?;
    }

    if maker_ata_b.is_data_empty() {
        pinocchio_associated_token_account::instructions::Create {
            funding_account: taker,
            account: maker_ata_b,
            wallet: maker,
            mint: mint_b,
            token_program,
            system_program,
        }
        .invoke()?;
    }

    assert_token_account(taker_ata_a, taker.address(), mint_a.address())?;
    assert_token_account(maker_ata_b, maker.address(), mint_b.address())?;

    pinocchio_token::instructions::Transfer::new(
        taker_ata_b,
        maker_ata_b,
        taker,
        amount_to_receive,
    )
    .invoke()?;

    let bump_bytes = [bump];
    let signer_seeds = [
        Seed::from(b"escrow"),
        Seed::from(maker.address().as_array()),
        Seed::from(bump_bytes.as_ref()),
    ];
    pinocchio_token::instructions::Transfer::new(
        escrow_ata,
        taker_ata_a,
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
