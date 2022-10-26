import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { assert } from "chai";
import { Ostium } from "../target/types/ostium";
import { OSTIUM_SEED } from "./utils";
const { SystemProgram } = anchor.web3;

describe("ostium", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.Ostium as Program<Ostium>;

  it("initialize", async () => {
    const wallet = provider.wallet;

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

  it("deposit", async () => {});
});
