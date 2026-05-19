use anchor_lang::solana_program::instruction::{AccountMeta, Instruction};
use anchor_lang::{prelude::*, AnchorSerialize, Discriminator};
use tuktuk_program::{
    compile_transaction,
    tuktuk::{
        cpi::{accounts::QueueTaskV0, queue_task_v0},
        program::Tuktuk,
        types::TriggerV0,
    },
    types::QueueTaskArgsV0,
    TransactionSourceV0,
};

use crate::{GptResponse, GPT_ORACLE_PROGRAM_ID};

#[derive(Accounts)]
pub struct ScheduleGpt<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        seeds = [b"gpt-response", gpt_response.authority.as_ref()],
        bump = gpt_response.bump,
    )]
    pub gpt_response: Account<'info, GptResponse>,
    /// CHECK: The GPT Oracle counter account is validated by the GPT Oracle program.
    #[account(mut)]
    pub oracle_counter: UncheckedAccount<'info>,
    /// CHECK: This is the GPT Oracle interaction PDA passed to the oracle instruction.
    #[account(mut)]
    pub interaction: UncheckedAccount<'info>,
    /// CHECK: This GPT Oracle context account is initialized by the scheduled task.
    #[account(mut)]
    pub context_account: UncheckedAccount<'info>,
    #[account(mut)]
    /// CHECK: The TukTuk task queue is validated by the TukTuk program.
    pub task_queue: UncheckedAccount<'info>,
    /// CHECK: The TukTuk task queue authority is validated by the TukTuk program.
    pub task_queue_authority: UncheckedAccount<'info>,
    /// CHECK: Initialized by the TukTuk CPI.
    #[account(mut)]
    pub task: UncheckedAccount<'info>,
    /// CHECK: PDA signer used as this program's TukTuk queue authority.
    #[account(
        mut,
        seeds = [b"queue_authority"],
        bump
    )]
    pub queue_authority: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
    pub tuktuk_program: Program<'info, Tuktuk>,
}

#[derive(AnchorSerialize)]
struct InteractWithLlmArgs {
    text: String,
    callback_program_id: Pubkey,
    callback_discriminator: [u8; 8],
    account_metas: Option<Vec<GptOracleAccountMeta>>,
}

#[derive(AnchorSerialize)]
struct GptOracleAccountMeta {
    pubkey: Pubkey,
    is_signer: bool,
    is_writable: bool,
}

#[derive(AnchorSerialize)]
struct CreateLlmContextArgs {
    text: String,
}

impl<'info> ScheduleGpt<'info> {
    pub fn schedule_gpt(&mut self, task_id: u16, bumps: &ScheduleGptBumps) -> Result<()> {
        let queue_authority_bump = bumps.queue_authority;
        let queue_authority_key = self.queue_authority.key();

        let create_context_args = CreateLlmContextArgs {
            text: self.gpt_response.prompt.clone(),
        };
        let mut create_context_data = [224, 109, 4, 173, 191, 25, 42, 162].to_vec();
        create_context_args.serialize(&mut create_context_data)?;

        let create_context_instruction = Instruction {
            program_id: GPT_ORACLE_PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(queue_authority_key, true),
                AccountMeta::new(self.oracle_counter.key(), false),
                AccountMeta::new(self.context_account.key(), false),
                AccountMeta::new_readonly(self.system_program.key(), false),
            ],
            data: create_context_data,
        };

        let callback_accounts = vec![GptOracleAccountMeta {
            pubkey: self.gpt_response.key(),
            is_signer: false,
            is_writable: true,
        }];

        let args = InteractWithLlmArgs {
            text: self.gpt_response.prompt.clone(),
            callback_program_id: crate::ID,
            callback_discriminator: crate::instruction::ReceiveGptResponse::DISCRIMINATOR
                .try_into()
                .unwrap(),
            account_metas: Some(callback_accounts),
        };

        let mut data = [2, 54, 5, 16, 87, 123, 219, 132].to_vec();
        args.serialize(&mut data)?;

        let oracle_instruction = Instruction {
            program_id: GPT_ORACLE_PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(queue_authority_key, true),
                AccountMeta::new(self.interaction.key(), false),
                AccountMeta::new_readonly(self.context_account.key(), false),
                AccountMeta::new_readonly(self.system_program.key(), false),
            ],
            data,
        };

        let (compiled_tx, _) = compile_transaction(
            vec![create_context_instruction, oracle_instruction],
            vec![vec![
                b"queue_authority".to_vec(),
                vec![queue_authority_bump],
            ]],
        )
        .unwrap();

        queue_task_v0(
            CpiContext::new_with_signer(
                self.tuktuk_program.to_account_info(),
                QueueTaskV0 {
                    payer: self.authority.to_account_info(),
                    queue_authority: self.queue_authority.to_account_info(),
                    task_queue: self.task_queue.to_account_info(),
                    task_queue_authority: self.task_queue_authority.to_account_info(),
                    task: self.task.to_account_info(),
                    system_program: self.system_program.to_account_info(),
                },
                &[&[b"queue_authority", &[queue_authority_bump]]],
            ),
            QueueTaskArgsV0 {
                trigger: TriggerV0::Now,
                transaction: TransactionSourceV0::CompiledV0(compiled_tx),
                crank_reward: Some(1000001),
                free_tasks: 0,
                id: task_id,
                description: "solana-gpt-oracle request".to_string(),
            },
        )?;

        Ok(())
    }
}
