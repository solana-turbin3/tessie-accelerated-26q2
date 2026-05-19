import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { init, taskKey, taskQueueAuthorityKey } from "@helium/tuktuk-sdk";
import { SolanaGptTuktuk } from "../target/types/solana_gpt_tuktuk";

describe("solana-gpt-tuktuk", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.solanaGptTuktuk as Program<SolanaGptTuktuk>;

  const gptOracleProgram = new anchor.web3.PublicKey(
    "LLMrieZMpbJFwN52WgmBNMxYojrpRVYXdC1RCweEbab"
  );
  const taskQueue = new anchor.web3.PublicKey(
    process.env.TUKTUK_TASK_QUEUE ||
      "BbGDaZKP6w3XE1vMoiHXxY8yDWAf4B2fQa72mBP57YvE"
  );

  const gptResponse = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("gpt-response"), provider.publicKey.toBuffer()],
    program.programId
  )[0];
  const queueAuthority = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("queue_authority")],
    program.programId
  )[0];
  const oracleCounter = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("counter")],
    gptOracleProgram
  )[0];

  const getOracleContextAndInteraction = async () => {
    const counterAccount = await provider.connection.getAccountInfo(
      oracleCounter
    );
    if (!counterAccount) {
      throw new Error("GPT Oracle counter is not initialized on this cluster");
    }

    const counter = new anchor.BN(
      counterAccount.data.subarray(8, 12),
      "le"
    ).toNumber();
    const counterBytes = Buffer.alloc(4);
    counterBytes.writeUInt32LE(counter);

    const contextAccount = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("test-context"), counterBytes],
      gptOracleProgram
    )[0];
    const interaction = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("interaction"),
        queueAuthority.toBuffer(),
        contextAccount.toBuffer(),
      ],
      gptOracleProgram
    )[0];

    return { contextAccount, interaction };
  };

  it("initializes a GPT response account", async () => {
    const prompt = "Explain Solana in one short sentence.";

    const tx = await program.methods
      .initialize(prompt)
      .accountsPartial({
        authority: provider.publicKey,
        gptResponse,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    console.log("initialize tx:", tx);
    const account = await program.account.gptResponse.fetch(gptResponse);
    console.log("prompt:", account.prompt);
  });

  it("schedules a GPT Oracle request with TukTuk", async () => {
    const tuktukProgram = await init(provider);
    const taskId = Number(process.env.TUKTUK_TASK_ID || 0);
    const { contextAccount, interaction } =
      await getOracleContextAndInteraction();

    const tx = await program.methods
      .scheduleGpt(taskId)
      .accountsPartial({
        authority: provider.publicKey,
        gptResponse,
        oracleCounter,
        interaction,
        contextAccount,
        taskQueue,
        taskQueueAuthority: taskQueueAuthorityKey(taskQueue, queueAuthority)[0],
        task: taskKey(taskQueue, taskId)[0],
        queueAuthority,
        systemProgram: anchor.web3.SystemProgram.programId,
        tuktukProgram: tuktukProgram.programId,
      })
      .rpc({ skipPreflight: true });

    console.log("schedule tx:", tx);
    console.log("gptResponse:", gptResponse.toBase58());
    console.log("task:", taskKey(taskQueue, taskId)[0].toBase58());
  });
});
