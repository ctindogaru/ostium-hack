import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { assert } from "chai";
import { createMint } from "@solana/spl-token";
import { Ostium } from "../target/types/ostium";
import { OSTIUM_SEED } from "./utils";
const { SystemProgram } = anchor.web3;

describe("ostium", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const connection = provider.connection;
  const wallet = provider.wallet;
  const program = anchor.workspace.Ostium as Program<Ostium>;

  it("initialize", async () => {
    const [_, bump] = await anchor.web3.PublicKey.findProgramAddress(
      [OSTIUM_SEED],
      program.programId
    );
    const state: anchor.web3.Keypair = anchor.web3.Keypair.generate();
    await program.methods
      .initialize(bump)
      .accounts({
        state: state.publicKey,
        signer: wallet.publicKey,
        systemProgram: SystemProgram.programId,
        admin: wallet.publicKey,
      })
      .signers([state])
      .rpc();

    const stateAccount = await program.account.state.fetch(state.publicKey);
    assert.ok(stateAccount.bumpSeed === bump);
    assert.ok(stateAccount.isInitialized === true);
    assert.ok(stateAccount.admin.equals(wallet.publicKey));
  });

  it("deposit", async () => {
    const TOKEN_DECIMALS = 6;
    const usdcOwner: anchor.web3.Keypair = anchor.web3.Keypair.generate();
    await airdropSolTokens(connection, usdcOwner);
    // const usdc = await createMint(
    //   connection,
    //   usdcOwner,
    //   usdcOwner.publicKey,
    //   null,
    //   TOKEN_DECIMALS
    // );
  });
});

const airdropSolTokens = async (connection, wallet) => {
  const airdrop_sig = await connection.requestAirdrop(
    wallet.publicKey,
    anchor.web3.LAMPORTS_PER_SOL
  );
  const latestBlockHash = await connection.getLatestBlockhash();
  await connection.confirmTransaction({
    blockhash: latestBlockHash.blockhash,
    lastValidBlockHeight: latestBlockHash.lastValidBlockHeight,
    signature: airdrop_sig,
  });
};
