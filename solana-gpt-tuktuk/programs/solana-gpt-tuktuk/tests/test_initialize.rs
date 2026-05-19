#![cfg(feature = "litesvm-test")]

use {
    anchor_lang::{
        solana_program::{instruction::Instruction, pubkey::Pubkey, system_program},
        AccountDeserialize, InstructionData, ToAccountMetas,
    },
    litesvm::LiteSVM,
    solana_keypair::Keypair,
    solana_message::{Message, VersionedMessage},
    solana_signer::Signer,
    solana_transaction::versioned::VersionedTransaction,
};

#[test]
fn test_initialize() {
    let program_id = solana_gpt_tuktuk::id();
    let payer = Keypair::new();
    let payer_pubkey = payer.pubkey();
    let mut svm = LiteSVM::new();
    let bytes = include_bytes!("../../../target/deploy/solana_gpt_tuktuk.so");
    svm.add_program(program_id, bytes).unwrap();
    svm.airdrop(&payer_pubkey, 1_000_000_000).unwrap();

    let prompt = "Explain Solana in one sentence".to_string();
    let (gpt_response, _bump) =
        Pubkey::find_program_address(&[b"gpt-response", payer_pubkey.as_ref()], &program_id);

    let instruction = Instruction::new_with_bytes(
        program_id,
        &solana_gpt_tuktuk::instruction::Initialize {
            prompt: prompt.clone(),
        }
        .data(),
        solana_gpt_tuktuk::accounts::Initialize {
            authority: payer_pubkey,
            gpt_response,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
    );

    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[instruction], Some(&payer_pubkey), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[payer]).unwrap();

    let res = svm.send_transaction(tx);
    assert!(res.is_ok());

    let account = svm.get_account(&gpt_response).unwrap();
    let mut data = account.data.as_slice();
    let gpt_response_account = solana_gpt_tuktuk::GptResponse::try_deserialize(&mut data).unwrap();

    assert_eq!(gpt_response_account.authority, payer_pubkey);
    assert_eq!(gpt_response_account.prompt, prompt);
    assert_eq!(gpt_response_account.response, "");
}
