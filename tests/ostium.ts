import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { assert } from "chai";
import { TOKEN_PROGRAM_ID, Token } from "@solana/spl-token";
import { Ostium } from "../target/types/ostium";
import { OSTIUM_SEED } from "./utils";
import _ from "lodash";

const { SystemProgram } = anchor.web3;

describe("ostium", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const connection = provider.connection;
  const wallet = provider.wallet;
  const program = anchor.workspace.Ostium as Program<Ostium>;
  const state: anchor.web3.Keypair = anchor.web3.Keypair.generate();
  const usdcOwner: anchor.web3.Keypair = anchor.web3.Keypair.generate();
  const user: anchor.web3.Keypair = anchor.web3.Keypair.generate();
  let usdcAccount;
  let pdaAccount;
  let usdc;
  let pda;
  let bump;
  let managerPda;
  let managerBump;

  const TOKEN_DECIMALS = 6;
  const TOKEN_MINT_AMOUNT = 1_000_000 * 10 ** TOKEN_DECIMALS;
  const DEPOSIT_AMOUNT = 1_000 * 10 ** TOKEN_DECIMALS;
  const WITHDRAW_AMOUNT = 500 * 10 ** TOKEN_DECIMALS;

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

    await airdropSolTokens(connection, user);

    [managerPda, managerBump] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("position-manager"), user.publicKey.toBuffer()],
      program.programId
    );

    await program.methods
      .initializePositionManager()
      .accounts({
        positionManager: managerPda,
        signer: user.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .signers([user])
      .rpc();

    let managerAccount = await program.account.positionManager.fetch(
      managerPda
    );
    assert(managerAccount.isInitialized === true);
    assert(managerAccount.owner.equals(user.publicKey));
    assert(managerAccount.balance.eq(new anchor.BN(0)));
    assert(managerAccount.noOfPositions.eq(new anchor.BN(0)));
  });

  it("deposit", async () => {
    await airdropSolTokens(connection, usdcOwner);
    usdc = await Token.createMint(
      connection,
      usdcOwner,
      usdcOwner.publicKey,
      null,
      TOKEN_DECIMALS,
      TOKEN_PROGRAM_ID
    );
    usdcAccount = await usdc.createAccount(usdcOwner.publicKey);
    await usdc.mintTo(usdcAccount, usdcOwner, [], TOKEN_MINT_AMOUNT);
    pdaAccount = await usdc.createAccount(pda);

    let accountInfo;
    accountInfo = await usdc.getAccountInfo(usdcAccount);
    assert(accountInfo.amount == TOKEN_MINT_AMOUNT);
    accountInfo = await usdc.getAccountInfo(pdaAccount);
    assert(accountInfo.amount == 0);

    await program.methods
      .deposit(new anchor.BN(DEPOSIT_AMOUNT))
      .accounts({
        positionManager: managerPda,
        transferFrom: usdcAccount,
        transferTo: pdaAccount,
        authority: usdcOwner.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([usdcOwner])
      .rpc();

    let managerAccount = await program.account.positionManager.fetch(
      managerPda
    );
    assert(managerAccount.balance.eq(new anchor.BN(DEPOSIT_AMOUNT)));

    accountInfo = await usdc.getAccountInfo(usdcAccount);
    assert(accountInfo.amount == TOKEN_MINT_AMOUNT - DEPOSIT_AMOUNT);
    accountInfo = await usdc.getAccountInfo(pdaAccount);
    assert(accountInfo.amount == DEPOSIT_AMOUNT);
  });

  it("withdraw", async () => {
    let accountInfo;
    accountInfo = await usdc.getAccountInfo(usdcAccount);
    assert(accountInfo.amount == TOKEN_MINT_AMOUNT - DEPOSIT_AMOUNT);
    accountInfo = await usdc.getAccountInfo(pdaAccount);
    assert(accountInfo.amount == DEPOSIT_AMOUNT);

    await program.methods
      .withdraw(new anchor.BN(WITHDRAW_AMOUNT))
      .accounts({
        positionManager: managerPda,
        state: state.publicKey,
        transferFrom: pdaAccount,
        transferTo: usdcAccount,
        authority: pda,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();

    let managerAccount = await program.account.positionManager.fetch(
      managerPda
    );
    assert(
      managerAccount.balance.eq(new anchor.BN(DEPOSIT_AMOUNT - WITHDRAW_AMOUNT))
    );

    accountInfo = await usdc.getAccountInfo(usdcAccount);
    assert(
      accountInfo.amount == TOKEN_MINT_AMOUNT - DEPOSIT_AMOUNT + WITHDRAW_AMOUNT
    );
    accountInfo = await usdc.getAccountInfo(pdaAccount);
    assert(accountInfo.amount == DEPOSIT_AMOUNT - WITHDRAW_AMOUNT);
  });

  it("openPosition/closePosition", async () => {
    const QUANTITY = 10;
    const LEVERAGE = 50;

    let managerAccount = await program.account.positionManager.fetch(
      managerPda
    );
    let [positionPda, _positionBump] =
      await anchor.web3.PublicKey.findProgramAddress(
        [
          Buffer.from("position"),
          user.publicKey.toBuffer(),
          managerAccount.noOfPositions.toBuffer("le", 8),
        ],
        program.programId
      );

    await program.methods
      .openPosition(new anchor.BN(QUANTITY), LEVERAGE)
      .accounts({
        positionManager: managerPda,
        position: positionPda,
        signer: user.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .signers([user])
      .rpc();

    let positionAccount = await program.account.position.fetch(positionPda);
    assert(positionAccount.isInitialized === true);
    assert(positionAccount.owner.equals(user.publicKey));
    assert(positionAccount.entryPrice.eq(new anchor.BN(1650)));
    assert(positionAccount.quantity.eq(new anchor.BN(QUANTITY)));
    assert(positionAccount.leverage === LEVERAGE);
    assert(_.isEqual(positionAccount.status, { open: {} }));

    managerAccount = await program.account.positionManager.fetch(managerPda);
    assert(managerAccount.noOfPositions.eq(new anchor.BN(1)));

    const priceFeed: anchor.web3.Keypair = anchor.web3.Keypair.generate();
    await program.methods
      .closePosition()
      .accounts({
        positionManager: managerPda,
        position: positionPda,
        priceAccountInfo: priceFeed.publicKey,
      })
      .rpc();

    positionAccount = await program.account.position.fetch(positionPda);
    assert(positionAccount.exitPrice.eq(new anchor.BN(1800)));
    assert(_.isEqual(positionAccount.status, { closed: {} }));
  });
});

const airdropSolTokens = async (connection, wallet) => {
  const airdrop_sig = await connection.requestAirdrop(
    wallet.publicKey,
    anchor.web3.LAMPORTS_PER_SOL
  );
  await connection.confirmTransaction(airdrop_sig, "confirmed");
};
