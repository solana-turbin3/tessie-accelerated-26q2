#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use litesvm::LiteSVM;
    use litesvm_token::{
        CreateAssociatedTokenAccount, CreateMint, MintTo,
        spl_token::{self},
    };
    use solana_clock::Clock;
    use solana_instruction::{AccountMeta, Instruction};
    use solana_keypair::Keypair;
    use solana_message::Message;
    use solana_native_token::LAMPORTS_PER_SOL;
    use solana_pubkey::Pubkey;
    use solana_signer::Signer;
    use solana_transaction::Transaction;

    const TOKEN_PROGRAM_ID: Pubkey = spl_token::ID;
    const FUNDRAISER_LEN: usize = 90;
    const CONTRIBUTOR_LEN: usize = 9;

    fn program_id() -> Pubkey {
        Pubkey::from(crate::ID)
    }

    fn so_path() -> PathBuf {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        for subdir in &["sbpf-solana-solana", "sbf-solana-solana"] {
            let p = manifest_dir
                .join("target")
                .join(subdir)
                .join("release/pinocchio_fundraiser.so");
            if p.exists() {
                return p;
            }
        }
        manifest_dir.join("target/deploy/pinocchio_fundraiser.so")
    }

    fn setup() -> (LiteSVM, Keypair) {
        let mut svm = LiteSVM::new();
        let maker = Keypair::new();
        svm.airdrop(&maker.pubkey(), 10 * LAMPORTS_PER_SOL)
            .expect("Airdrop failed");

        let program_data = std::fs::read(so_path())
            .expect("Failed to read fundraiser .so; run `cargo build-sbf` first");
        svm.add_program(program_id(), &program_data)
            .expect("Failed to add program");

        (svm, maker)
    }

    fn fundraiser_pda(maker: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(&[b"fundraiser", maker.as_ref()], &program_id())
    }

    fn contributor_pda(fundraiser: &Pubkey, contributor: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[b"contributor", fundraiser.as_ref(), contributor.as_ref()],
            &program_id(),
        )
    }

    fn ata_program() -> Pubkey {
        "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL"
            .parse()
            .unwrap()
    }

    fn system_program() -> Pubkey {
        solana_sdk_ids::system_program::ID
    }

    fn read_token_balance(svm: &LiteSVM, ata: &Pubkey) -> u64 {
        let account = svm.get_account(ata).expect("token account not found");
        let bytes: [u8; 8] = account.data[64..72].try_into().unwrap();
        u64::from_le_bytes(bytes)
    }

    fn read_fundraiser_amounts(svm: &LiteSVM, fundraiser: &Pubkey) -> (u64, u64) {
        let account = svm.get_account(fundraiser).expect("fundraiser not found");
        assert_eq!(account.data.len(), FUNDRAISER_LEN);
        let target = u64::from_le_bytes(account.data[64..72].try_into().unwrap());
        let current = u64::from_le_bytes(account.data[72..80].try_into().unwrap());
        (target, current)
    }

    fn read_contributor_amount(svm: &LiteSVM, contributor_account: &Pubkey) -> u64 {
        let account = svm
            .get_account(contributor_account)
            .expect("contributor account not found");
        assert_eq!(account.data.len(), CONTRIBUTOR_LEN);
        u64::from_le_bytes(account.data[0..8].try_into().unwrap())
    }

    fn initialize(
        svm: &mut LiteSVM,
        maker: &Keypair,
        mint: &Pubkey,
        amount_to_raise: u64,
        duration_days: u8,
    ) -> (Pubkey, Pubkey) {
        let (fundraiser, bump) = fundraiser_pda(&maker.pubkey());
        let vault = spl_associated_token_account::get_associated_token_address(&fundraiser, mint);
        let data = [
            vec![0],
            amount_to_raise.to_le_bytes().to_vec(),
            vec![duration_days, bump],
        ]
        .concat();

        let ix = Instruction {
            program_id: program_id(),
            accounts: vec![
                AccountMeta::new(maker.pubkey(), true),
                AccountMeta::new_readonly(*mint, false),
                AccountMeta::new(fundraiser, false),
                AccountMeta::new(vault, false),
                AccountMeta::new_readonly(system_program(), false),
                AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
                AccountMeta::new_readonly(ata_program(), false),
            ],
            data,
        };

        let msg = Message::new(&[ix], Some(&maker.pubkey()));
        let tx = Transaction::new(&[maker], msg, svm.latest_blockhash());
        svm.send_transaction(tx).expect("initialize failed");

        (fundraiser, vault)
    }

    fn contribute(
        svm: &mut LiteSVM,
        contributor: &Keypair,
        mint: &Pubkey,
        fundraiser: &Pubkey,
        contributor_ata: &Pubkey,
        vault: &Pubkey,
        amount: u64,
    ) -> Pubkey {
        let (contributor_account, contributor_bump) =
            contributor_pda(fundraiser, &contributor.pubkey());
        let data = [
            vec![1],
            amount.to_le_bytes().to_vec(),
            vec![contributor_bump],
        ]
        .concat();

        let ix = Instruction {
            program_id: program_id(),
            accounts: vec![
                AccountMeta::new(contributor.pubkey(), true),
                AccountMeta::new_readonly(*mint, false),
                AccountMeta::new(*fundraiser, false),
                AccountMeta::new(contributor_account, false),
                AccountMeta::new(*contributor_ata, false),
                AccountMeta::new(*vault, false),
                AccountMeta::new_readonly(system_program(), false),
                AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
            ],
            data,
        };

        let msg = Message::new(&[ix], Some(&contributor.pubkey()));
        let tx = Transaction::new(&[contributor], msg, svm.latest_blockhash());
        svm.send_transaction(tx).expect("contribute failed");

        contributor_account
    }

    fn make_contributor(svm: &mut LiteSVM, payer: &Keypair, mint: &Pubkey, amount: u64) -> Keypair {
        let contributor = Keypair::new();
        svm.airdrop(&contributor.pubkey(), LAMPORTS_PER_SOL)
            .expect("contributor airdrop failed");
        let contributor_ata = CreateAssociatedTokenAccount::new(svm, payer, mint)
            .owner(&contributor.pubkey())
            .send()
            .unwrap();
        MintTo::new(svm, payer, mint, &contributor_ata, amount)
            .send()
            .unwrap();
        contributor
    }

    fn contributor_ata(owner: &Pubkey, mint: &Pubkey) -> Pubkey {
        spl_associated_token_account::get_associated_token_address(owner, mint)
    }

    #[test]
    fn test_initialize() {
        let (mut svm, maker) = setup();
        let mint = CreateMint::new(&mut svm, &maker)
            .decimals(6)
            .authority(&maker.pubkey())
            .send()
            .unwrap();

        let (fundraiser, vault) = initialize(&mut svm, &maker, &mint, 10_000_000, 2);

        let fundraiser_account = svm.get_account(&fundraiser).expect("fundraiser missing");
        assert_eq!(fundraiser_account.owner, program_id());
        assert_eq!(fundraiser_account.data.len(), FUNDRAISER_LEN);

        let vault_account = svm.get_account(&vault).expect("vault missing");
        assert_eq!(vault_account.owner, TOKEN_PROGRAM_ID);
        assert_eq!(read_token_balance(&svm, &vault), 0);

        let (target, current) = read_fundraiser_amounts(&svm, &fundraiser);
        assert_eq!(target, 10_000_000);
        assert_eq!(current, 0);
    }

    #[test]
    fn test_contribute() {
        let (mut svm, maker) = setup();
        let mint = CreateMint::new(&mut svm, &maker)
            .decimals(6)
            .authority(&maker.pubkey())
            .send()
            .unwrap();
        let contributor = make_contributor(&mut svm, &maker, &mint, 5_000_000);
        let contributor_ata = contributor_ata(&contributor.pubkey(), &mint);
        let (fundraiser, vault) = initialize(&mut svm, &maker, &mint, 10_000_000, 2);

        let contributor_account = contribute(
            &mut svm,
            &contributor,
            &mint,
            &fundraiser,
            &contributor_ata,
            &vault,
            1_000_000,
        );

        assert_eq!(read_token_balance(&svm, &contributor_ata), 4_000_000);
        assert_eq!(read_token_balance(&svm, &vault), 1_000_000);
        assert_eq!(
            read_contributor_amount(&svm, &contributor_account),
            1_000_000
        );
        let (_, current) = read_fundraiser_amounts(&svm, &fundraiser);
        assert_eq!(current, 1_000_000);
    }

    #[test]
    fn test_refund_after_failed_fundraiser() {
        let (mut svm, maker) = setup();
        let mint = CreateMint::new(&mut svm, &maker)
            .decimals(6)
            .authority(&maker.pubkey())
            .send()
            .unwrap();
        let contributor = make_contributor(&mut svm, &maker, &mint, 5_000_000);
        let contributor_ata = contributor_ata(&contributor.pubkey(), &mint);
        let (fundraiser, vault) = initialize(&mut svm, &maker, &mint, 10_000_000, 1);
        let contributor_account = contribute(
            &mut svm,
            &contributor,
            &mint,
            &fundraiser,
            &contributor_ata,
            &vault,
            1_000_000,
        );

        let mut clock = svm.get_sysvar::<Clock>();
        clock.unix_timestamp += 2 * 86_400;
        svm.set_sysvar::<Clock>(&clock);

        let ix = Instruction {
            program_id: program_id(),
            accounts: vec![
                AccountMeta::new(contributor.pubkey(), true),
                AccountMeta::new(maker.pubkey(), false),
                AccountMeta::new_readonly(mint, false),
                AccountMeta::new(fundraiser, false),
                AccountMeta::new(contributor_account, false),
                AccountMeta::new(contributor_ata, false),
                AccountMeta::new(vault, false),
                AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
                AccountMeta::new_readonly(system_program(), false),
            ],
            data: vec![3],
        };

        let msg = Message::new(&[ix], Some(&contributor.pubkey()));
        let tx = Transaction::new(&[&contributor], msg, svm.latest_blockhash());
        svm.send_transaction(tx).expect("refund failed");

        assert_eq!(read_token_balance(&svm, &contributor_ata), 5_000_000);
        assert_eq!(read_token_balance(&svm, &vault), 0);
        assert!(svm.get_account(&contributor_account).is_none());
        let (_, current) = read_fundraiser_amounts(&svm, &fundraiser);
        assert_eq!(current, 0);
    }

    #[test]
    fn test_check_contributions_after_target_met() {
        let (mut svm, maker) = setup();
        let mint = CreateMint::new(&mut svm, &maker)
            .decimals(6)
            .authority(&maker.pubkey())
            .send()
            .unwrap();
        let (fundraiser, vault) = initialize(&mut svm, &maker, &mint, 4_000_000, 2);

        for _ in 0..10 {
            let contributor = make_contributor(&mut svm, &maker, &mint, 500_000);
            let ata = contributor_ata(&contributor.pubkey(), &mint);
            contribute(
                &mut svm,
                &contributor,
                &mint,
                &fundraiser,
                &ata,
                &vault,
                400_000,
            );
        }

        let maker_ata = contributor_ata(&maker.pubkey(), &mint);
        let ix = Instruction {
            program_id: program_id(),
            accounts: vec![
                AccountMeta::new(maker.pubkey(), true),
                AccountMeta::new_readonly(mint, false),
                AccountMeta::new(fundraiser, false),
                AccountMeta::new(vault, false),
                AccountMeta::new(maker_ata, false),
                AccountMeta::new_readonly(system_program(), false),
                AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
                AccountMeta::new_readonly(ata_program(), false),
            ],
            data: vec![2],
        };

        let msg = Message::new(&[ix], Some(&maker.pubkey()));
        let tx = Transaction::new(&[&maker], msg, svm.latest_blockhash());
        svm.send_transaction(tx)
            .expect("check contributions failed");

        assert_eq!(read_token_balance(&svm, &maker_ata), 4_000_000);
        assert!(svm.get_account(&vault).is_none());
        assert!(svm.get_account(&fundraiser).is_none());
    }
}
