use {
    anchor_lang::{
        solana_program::{
            self,
            instruction::{AccountMeta, Instruction},
            pubkey::Pubkey,
        },
        InstructionData, ToAccountMetas,
    },
    litesvm::LiteSVM,
    solana_keypair::Keypair,
    solana_message::{Message, VersionedMessage},
    solana_signer::Signer,
    solana_transaction::versioned::VersionedTransaction,
    spl_associated_token_account_interface::{
        address::get_associated_token_address_with_program_id,
        instruction::create_associated_token_account,
    },
    spl_token_2022_interface::{instruction::transfer_checked, ID as TOKEN_2022_ID},
    transfer_hook_vault as program,
};

fn send(
    svm: &mut LiteSVM,
    ixs: &[Instruction],
    payer: &Keypair,
    signers: &[&Keypair],
) -> litesvm::types::TransactionResult {
    svm.expire_blockhash();
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(ixs, Some(&payer.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), signers).unwrap();
    svm.send_transaction(tx)
}

fn load_program(svm: &mut LiteSVM) {
    let program_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/deploy/transfer_hook_vault.so");
    let bytes = std::fs::read(program_path)
        .expect("run `anchor build` before running LiteSVM tests");
    svm.add_program(program::id(), &bytes).unwrap();
}

fn transfer_hook_remaining_metas(
    extra_account_meta_list: Pubkey,
    vault_state: Pubkey,
) -> Vec<AccountMeta> {
    vec![
        AccountMeta::new_readonly(extra_account_meta_list, false),
        AccountMeta::new_readonly(vault_state, false),
        AccountMeta::new_readonly(program::id(), false),
    ]
}

fn append_transfer_hook_accounts(
    ix: &mut Instruction,
    extra_account_meta_list: Pubkey,
    vault_state: Pubkey,
) {
    ix.accounts
        .extend(transfer_hook_remaining_metas(extra_account_meta_list, vault_state));
}

#[test]
fn test_vault_hook_flow() {
    let mut svm = LiteSVM::new();
    let admin = Keypair::new();
    let user = Keypair::new();
    let stranger = Keypair::new();
    let program_id = program::id();
    let system_program = solana_program::system_program::id();

    load_program(&mut svm);
    svm.airdrop(&admin.pubkey(), 10_000_000_000).unwrap();
    svm.airdrop(&user.pubkey(), 1_000_000_000).unwrap();
    svm.airdrop(&stranger.pubkey(), 1_000_000_000).unwrap();

    let (vault_state, _) = Pubkey::find_program_address(&[program::VAULT_STATE_SEED], &program_id);
    let (vault_authority, _) =
        Pubkey::find_program_address(&[program::VAULT_AUTHORITY_SEED], &program_id);
    let (mint, _) = Pubkey::find_program_address(&[program::MINT_SEED], &program_id);
    let (vault, _) =
        Pubkey::find_program_address(&[program::VAULT_TOKEN_ACCOUNT_SEED], &program_id);
    let (extra_account_meta_list, _) = Pubkey::find_program_address(
        &[b"extra-account-metas", mint.as_ref()],
        &program_id,
    );

    let initialize_ix = Instruction::new_with_bytes(
        program_id,
        &program::instruction::Initialize {}.data(),
        program::accounts::Initialize {
            admin: admin.pubkey(),
            vault_state,
            vault_authority,
            system_program,
        }
        .to_account_metas(None),
    );
    send(&mut svm, &[initialize_ix], &admin, &[&admin]).expect("initialize failed");

    let initialize_vault_ix = Instruction::new_with_bytes(
        program_id,
        &program::instruction::InitializeVaultTokenAccounts {}.data(),
        program::accounts::InitializeVaultTokenAccounts {
            admin: admin.pubkey(),
            vault_state,
            vault_authority,
            transfer_hook_program: program_id,
            mint,
            vault,
            token_program: TOKEN_2022_ID,
            system_program,
        }
        .to_account_metas(None),
    );
    send(&mut svm, &[initialize_vault_ix], &admin, &[&admin])
        .expect("initialize_vault_token_accounts failed");

    let initialize_transfer_hook_ix = Instruction::new_with_bytes(
        program_id,
        &program::instruction::InitializeTransferHook {}.data(),
        program::accounts::InitializeExtraAccountMetaList {
            payer: admin.pubkey(),
            extra_account_meta_list,
            mint,
            system_program,
        }
        .to_account_metas(None),
    );
    send(&mut svm, &[initialize_transfer_hook_ix], &admin, &[&admin])
        .expect("initialize_transfer_hook failed");

    let add_to_whitelist_ix = Instruction::new_with_bytes(
        program_id,
        &program::instruction::AddToWhitelist {
            user: user.pubkey(),
        }
        .data(),
        program::accounts::WhitelistOperations {
            admin: admin.pubkey(),
            vault_state,
            system_program,
        }
        .to_account_metas(None),
    );
    send(&mut svm, &[add_to_whitelist_ix], &admin, &[&admin])
        .expect("add_to_whitelist failed");

    let user_ata = get_associated_token_address_with_program_id(
        &user.pubkey(),
        &mint,
        &TOKEN_2022_ID,
    );
    let stranger_ata = get_associated_token_address_with_program_id(
        &stranger.pubkey(),
        &mint,
        &TOKEN_2022_ID,
    );
    let create_user_ata = create_associated_token_account(
        &admin.pubkey(),
        &user.pubkey(),
        &mint,
        &TOKEN_2022_ID,
    );
    let create_stranger_ata = create_associated_token_account(
        &admin.pubkey(),
        &stranger.pubkey(),
        &mint,
        &TOKEN_2022_ID,
    );
    send(
        &mut svm,
        &[create_user_ata, create_stranger_ata],
        &admin,
        &[&admin],
    )
    .expect("create token accounts failed");

    let mint_amount = 100_000_000;
    let mint_token_ix = Instruction::new_with_bytes(
        program_id,
        &program::instruction::MintToken {
            amount: mint_amount,
        }
        .data(),
        program::accounts::MintToken {
            admin: admin.pubkey(),
            recipient: user.pubkey(),
            mint,
            recipient_token_account: user_ata,
            vault_authority,
            vault_state,
            token_program: TOKEN_2022_ID,
        }
        .to_account_metas(None),
    );
    send(&mut svm, &[mint_token_ix], &admin, &[&admin]).expect("mint_token failed");

    let deposit_amount = 40_000_000;
    let mut whitelisted_deposit_ix = transfer_checked(
        &TOKEN_2022_ID,
        &user_ata,
        &mint,
        &vault,
        &user.pubkey(),
        &[],
        deposit_amount,
        6,
    )
    .unwrap();
    append_transfer_hook_accounts(&mut whitelisted_deposit_ix, extra_account_meta_list, vault_state);
    send(&mut svm, &[whitelisted_deposit_ix], &user, &[&user])
        .expect("whitelisted transfer into vault failed");

    let fund_stranger_amount = 1_000_000;
    let mut fund_stranger_ix = transfer_checked(
        &TOKEN_2022_ID,
        &user_ata,
        &mint,
        &stranger_ata,
        &user.pubkey(),
        &[],
        fund_stranger_amount,
        6,
    )
    .unwrap();
    append_transfer_hook_accounts(&mut fund_stranger_ix, extra_account_meta_list, vault_state);
    send(&mut svm, &[fund_stranger_ix], &user, &[&user])
        .expect("non-vault transfer should pass through hook");

    let mut stranger_deposit_ix = transfer_checked(
        &TOKEN_2022_ID,
        &stranger_ata,
        &mint,
        &vault,
        &stranger.pubkey(),
        &[],
        fund_stranger_amount,
        6,
    )
    .unwrap();
    append_transfer_hook_accounts(&mut stranger_deposit_ix, extra_account_meta_list, vault_state);
    let result = send(&mut svm, &[stranger_deposit_ix], &stranger, &[&stranger]);
    assert!(
        result.is_err(),
        "non-whitelisted transfer into vault should fail"
    );
}
