import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import {
  createCronJob,
  cronJobTransactionKey,
  getCronJobForName,
  init as initCron,
} from "@helium/cron-sdk";
import {
  compileTransaction,
  init,
  taskQueueAuthorityKey,
} from "@helium/tuktuk-sdk";
import {
  Connection,
  Keypair,
  LAMPORTS_PER_SOL,
  PublicKey,
  SystemProgram,
  Transaction,
  TransactionInstruction,
} from "@solana/web3.js";
import * as fs from "fs";
import * as os from "os";
import * as path from "path";
import { ErStateAccount } from "../target/types/er_state_account";

type Args = {
  cronName: string;
  queueName: string;
  walletPath: string;
  rpcUrl: string;
  taskQueue: string;
  schedule: string;
  newData: number;
  fundingAmount: number;
};

const DEFAULT_TASK_QUEUE = "CMreFdKxT5oeZhiX8nWTGz9PtXM1AMYTh6dGR2UzdtrA";

function readArgs(): Args {
  const raw = process.argv.slice(2);
  const values = new Map<string, string>();

  for (let i = 0; i < raw.length; i += 1) {
    const arg = raw[i];
    if (!arg.startsWith("--")) {
      continue;
    }

    const key = arg.slice(2);
    const value = raw[i + 1];
    if (!value || value.startsWith("--")) {
      throw new Error(`Missing value for --${key}`);
    }

    values.set(key, value);
    i += 1;
  }

  return {
    cronName: values.get("cronName") || "magicblock-state-cron",
    queueName: values.get("queueName") || "magicblock-state-scheduler",
    walletPath: values.get("walletPath") || "~/.config/solana/id.json",
    rpcUrl: values.get("rpcUrl") || "https://api.devnet.solana.com",
    taskQueue: values.get("taskQueue") || DEFAULT_TASK_QUEUE,
    schedule: values.get("schedule") || "0 * * * * *",
    newData: Number(values.get("newData") || 777),
    fundingAmount: Number(
      values.get("fundingAmount") || 0.01 * LAMPORTS_PER_SOL
    ),
  };
}

function expandHome(filePath: string): string {
  if (filePath === "~") {
    return os.homedir();
  }

  if (filePath.startsWith("~/")) {
    return path.join(os.homedir(), filePath.slice(2));
  }

  return filePath;
}

function loadWallet(walletPath: string): anchor.Wallet {
  const keypairPath = expandHome(walletPath);
  const secret = JSON.parse(fs.readFileSync(keypairPath, "utf8"));
  return new anchor.Wallet(Keypair.fromSecretKey(Uint8Array.from(secret)));
}

async function main() {
  const argv = readArgs();
  const wallet = loadWallet(argv.walletPath);
  const provider = new anchor.AnchorProvider(
    new Connection(argv.rpcUrl, "confirmed"),
    wallet,
    { commitment: "confirmed" }
  );

  anchor.setProvider(provider);

  const program = anchor.workspace.erStateAccount as Program<ErStateAccount>;
  const tuktukProgram = await init(provider);
  const cronProgram = await initCron(provider);
  const taskQueue = new PublicKey(argv.taskQueue);
  const user = wallet.publicKey;
  const userAccount = PublicKey.findProgramAddressSync(
    [Buffer.from("user"), user.toBuffer()],
    program.programId
  )[0];

  console.log("Scheduler:", argv.queueName);
  console.log("Wallet:", user.toBase58());
  console.log("RPC URL:", argv.rpcUrl);
  console.log("Task queue:", taskQueue.toBase58());
  console.log("User account:", userAccount.toBase58());

  const userAccountInfo = await provider.connection.getAccountInfo(userAccount);
  if (!userAccountInfo) {
    console.log("Initializing Magicblock user account...");
    await program.methods
      .initialize()
      .accountsPartial({
        user,
        userAccount,
        systemProgram: SystemProgram.programId,
      })
      .rpc();
    console.log("User account initialized.");
  } else {
    console.log("User account already exists.");
  }

  const taskQueueAuthorityPda = taskQueueAuthorityKey(taskQueue, user)[0];
  const taskQueueAuthorityInfo = await provider.connection.getAccountInfo(
    taskQueueAuthorityPda
  );

  if (!taskQueueAuthorityInfo) {
    console.log("Initializing TukTuk task queue authority...");
    await tuktukProgram.methods
      .addQueueAuthorityV0()
      .accounts({
        payer: user,
        queueAuthority: user,
        taskQueue,
      })
      .rpc({ skipPreflight: true });
    console.log("Task queue authority initialized.");
  } else {
    console.log("Task queue authority already exists.");
  }

  const existingCronJob = await getCronJobForName(cronProgram, argv.cronName);
  let cronJob: PublicKey;

  if (!existingCronJob) {
    console.log("Creating cron job...");
    const {
      pubkeys: { cronJob: cronJobPubkey },
    } = await (
      await createCronJob(cronProgram, {
        tuktukProgram,
        taskQueue,
        args: {
          name: argv.cronName,
          schedule: argv.schedule,
          freeTasksPerTransaction: 0,
          numTasksPerQueueCall: 1,
        },
      })
    ).rpcAndKeys({ skipPreflight: false });

    cronJob = cronJobPubkey;

    console.log(
      "Funding cron job with",
      argv.fundingAmount / LAMPORTS_PER_SOL,
      "SOL"
    );
    const fundingTx = new Transaction().add(
      SystemProgram.transfer({
        fromPubkey: user,
        toPubkey: cronJob,
        lamports: argv.fundingAmount,
      })
    );
    await provider.sendAndConfirm(fundingTx);

    const scheduledUpdateInstruction = new TransactionInstruction({
      keys: [
        { pubkey: user, isSigner: false, isWritable: false },
        { pubkey: userAccount, isSigner: false, isWritable: true },
      ],
      data: program.coder.instruction.encode("scheduledUpdate", {
        newData: new anchor.BN(argv.newData),
      }),
      programId: program.programId,
    });

    const { transaction, remainingAccounts } = compileTransaction(
      [scheduledUpdateInstruction],
      []
    );

    console.log("Adding scheduled update transaction to cron job...");
    await cronProgram.methods
      .addCronTransactionV0({
        index: 0,
        transactionSource: {
          compiledV0: [transaction],
        },
      })
      .accounts({
        payer: user,
        cronJob,
        cronJobTransaction: cronJobTransactionKey(cronJob, 0)[0],
      })
      .remainingAccounts(remainingAccounts)
      .rpc({ skipPreflight: true });
    console.log("Cron job created.");
  } else {
    cronJob = existingCronJob;
    console.log("Cron job already exists.");
  }

  console.log("Cron job address:", cronJob.toBase58());
  console.log(
    `scheduledUpdate(${argv.newData}) will be posted using schedule "${argv.schedule}".`
  );
  console.log("To stop the cron job:");
  console.log(
    `tuktuk -u ${argv.rpcUrl} -w ${argv.walletPath} cron-transaction close --cron-name ${argv.cronName} --id 0`
  );
  console.log(
    `tuktuk -u ${argv.rpcUrl} -w ${argv.walletPath} cron close --cron-name ${argv.cronName}`
  );
}

main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error(error);
    process.exit(1);
  });
