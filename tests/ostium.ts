import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { assert } from "chai";
import { TOKEN_PROGRAM_ID, Token } from "@solana/spl-token";
import { Ostium } from "../target/types/ostium";
import { OSTIUM_SEED } from "./utils";
const { SystemProgram } = anchor.web3;

describe("ostium", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const connection = provider.connection;
  const wallet = provider.wallet;
  const program = anchor.workspace.Ostium as Program<Ostium>;
  const state: anchor.web3.Keypair = anchor.web3.Keypair.generate();
  let pda;
  let bump;

  it("initialize", async () => {
    [pda, bump] = await anchor.web3.PublicKey.findProgramAddress(
      [OSTIUM_SEED],
      program.programId
    );
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
    const TOKEN_MINT_AMOUNT = 1_000_000 * 10 ** TOKEN_DECIMALS;
    const DEPOSIT_AMOUNT = 1_000 * 10 ** TOKEN_DECIMALS;

    const usdcOwner: anchor.web3.Keypair = anchor.web3.Keypair.generate();
    await airdropSolTokens(connection, usdcOwner);
    const usdc = await Token.createMint(
      connection,
      usdcOwner,
      usdcOwner.publicKey,
      null,
      TOKEN_DECIMALS,
      TOKEN_PROGRAM_ID
    );
    const usdcAccount = await usdc.createAccount(usdcOwner.publicKey);
    await usdc.mintTo(usdcAccount, usdcOwner, [], TOKEN_MINT_AMOUNT);
    const pdaAccount = await usdc.createAccount(pda);

    let accountInfo;
    accountInfo = await usdc.getAccountInfo(usdcAccount);
    assert(accountInfo.amount == TOKEN_MINT_AMOUNT);
    accountInfo = await usdc.getAccountInfo(pdaAccount);
    assert(accountInfo.amount == 0);

    await program.methods
      .deposit(new anchor.BN(DEPOSIT_AMOUNT))
      .accounts({
        state: state.publicKey,
        transferFrom: usdcAccount,
        transferTo: pdaAccount,
        authority: usdcOwner.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([usdcOwner])
      .rpc();

    accountInfo = await usdc.getAccountInfo(usdcAccount);
    assert(accountInfo.amount == TOKEN_MINT_AMOUNT - DEPOSIT_AMOUNT);
    accountInfo = await usdc.getAccountInfo(pdaAccount);
    assert(accountInfo.amount == DEPOSIT_AMOUNT);
  });
});

const airdropSolTokens = async (connection, wallet) => {
  const airdrop_sig = await connection.requestAirdrop(
    wallet.publicKey,
    anchor.web3.LAMPORTS_PER_SOL
  );
  await connection.confirmTransaction(airdrop_sig, "confirmed");
};
