import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { assert } from "chai";
import { TOKEN_PROGRAM_ID, Token } from "@solana/spl-token";
import { Ostium } from "../target/types/ostium";
import { airdropSolTokens, OSTIUM_SEED } from "./utils";
import _ from "lodash";

const { SystemProgram } = anchor.web3;

describe("ostium", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const connection = provider.connection;
  const admin = provider.wallet;
  const program = anchor.workspace.Ostium as Program<Ostium>;

  const user: anchor.web3.Keypair = anchor.web3.Keypair.generate();
  let userAccount;
  let usdc;

  let ostiumPda;
  let ostiumBump;
  let ostiumPdaAccount;
  let feeCollectorAccount;

  let positionManagerPda;
  let positionManagerBump;

  const TOKEN_DECIMALS = 6;
  const USER_MINT_AMOUNT = 1_000_000 * 10 ** TOKEN_DECIMALS;
  const OSTIUM_MINT_AMOUNT = 100_000 * 10 ** TOKEN_DECIMALS;
  const DEPOSIT_AMOUNT = 1_000 * 10 ** TOKEN_DECIMALS;
  const WITHDRAW_AMOUNT = 500 * 10 ** TOKEN_DECIMALS;

  it("initialize", async () => {
    [ostiumPda, ostiumBump] = await anchor.web3.PublicKey.findProgramAddress(
      [OSTIUM_SEED],
      program.programId
    );

    await program.methods
      .initialize(ostiumBump)
      .accounts({
        state: ostiumPda,
        signer: admin.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    const stateAccount = await program.account.state.fetch(ostiumPda);
    assert.ok(stateAccount.ostiumBump === ostiumBump);
    assert.ok(stateAccount.isInitialized === true);

    await airdropSolTokens(connection, user);

    [positionManagerPda, positionManagerBump] =
      await anchor.web3.PublicKey.findProgramAddress(
        [Buffer.from("position-manager"), user.publicKey.toBuffer()],
        program.programId
      );

    await program.methods
      .initializePositionManager()
      .accounts({
        positionManager: positionManagerPda,
        signer: user.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .signers([user])
      .rpc();

    let managerAccount = await program.account.positionManager.fetch(
      positionManagerPda
    );
    assert(managerAccount.isInitialized === true);
    assert(managerAccount.owner.equals(user.publicKey));
    assert(managerAccount.noOfPositions.eq(new anchor.BN(0)));
  });

  it("end-to-end testing", async () => {
    // ------- INITIAL SETUP -------

    usdc = await Token.createMint(
      connection,
      user,
      user.publicKey,
      null,
      TOKEN_DECIMALS,
      TOKEN_PROGRAM_ID
    );
    userAccount = await usdc.createAccount(user.publicKey);
    await usdc.mintTo(userAccount, user, [], USER_MINT_AMOUNT);
    ostiumPdaAccount = await usdc.createAccount(ostiumPda);
    await usdc.mintTo(ostiumPdaAccount, user, [], OSTIUM_MINT_AMOUNT);
    feeCollectorAccount = await usdc.createAccount(ostiumPda);

    let accountInfo;
    accountInfo = await usdc.getAccountInfo(userAccount);
    assert(accountInfo.amount == USER_MINT_AMOUNT);
    accountInfo = await usdc.getAccountInfo(ostiumPdaAccount);
    assert(accountInfo.amount == OSTIUM_MINT_AMOUNT);
    accountInfo = await usdc.getAccountInfo(feeCollectorAccount);
    assert(accountInfo.amount == 0);

    // ------- OPEN POSITION -------

    const UNITS_IN_ONE_QUANTITY = 100_000_000;
    const UNITS_IN_ONE_LEVERAGE = 100;
    const QUANTITY = 10 * UNITS_IN_ONE_QUANTITY;
    const LEVERAGE = 50 * UNITS_IN_ONE_LEVERAGE;
    const ENTRY_PRICE = 1650 * 10 ** TOKEN_DECIMALS;
    const EXIT_PRICE = 1800 * 10 ** TOKEN_DECIMALS;

    let managerAccount = await program.account.positionManager.fetch(
      positionManagerPda
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

    const priceFeed: anchor.web3.Keypair = anchor.web3.Keypair.generate();
    await program.methods
      .openPosition(new anchor.BN(QUANTITY), new anchor.BN(LEVERAGE), {
        long: {},
      })
      .accounts({
        positionManager: positionManagerPda,
        position: positionPda,
        priceAccountInfo: priceFeed.publicKey,
        feeCollector: feeCollectorAccount,
        transferFrom: userAccount,
        transferTo: ostiumPdaAccount,
        signer: user.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([user])
      .rpc();

    let positionAccount = await program.account.position.fetch(positionPda);
    let feeInQuantity =
      (((QUANTITY * LEVERAGE) / UNITS_IN_ONE_LEVERAGE) * 5) / 10_000;
    let feeInUsdc = positionAccount.entryPrice
      .mul(new anchor.BN(feeInQuantity))
      .div(new anchor.BN(UNITS_IN_ONE_QUANTITY));

    assert(positionAccount.isInitialized === true);
    assert(positionAccount.owner.equals(user.publicKey));
    assert(positionAccount.asset.equals(priceFeed.publicKey));
    assert(
      positionAccount.quantity.eq(
        new anchor.BN(QUANTITY).sub(new anchor.BN(feeInQuantity))
      )
    );
    assert(positionAccount.leverage.eq(new anchor.BN(LEVERAGE)));
    let initial_collateral = positionAccount.quantity
      .mul(positionAccount.entryPrice)
      .div(new anchor.BN(UNITS_IN_ONE_QUANTITY));
    assert(positionAccount.collateral.eq(initial_collateral));
    assert(positionAccount.entryPrice.eq(new anchor.BN(ENTRY_PRICE)));
    assert(positionAccount.exitPrice.eq(new anchor.BN(0)));
    assert(_.isEqual(positionAccount.posStatus, { open: {} }));
    assert(_.isEqual(positionAccount.posType, { long: {} }));

    managerAccount = await program.account.positionManager.fetch(
      positionManagerPda
    );
    assert(managerAccount.noOfPositions.eq(new anchor.BN(1)));

    accountInfo = await usdc.getAccountInfo(userAccount);
    assert(
      accountInfo.amount ==
        USER_MINT_AMOUNT - initial_collateral.toNumber() - feeInUsdc.toNumber()
    );
    accountInfo = await usdc.getAccountInfo(ostiumPdaAccount);
    assert(
      accountInfo.amount == OSTIUM_MINT_AMOUNT + initial_collateral.toNumber()
    );
    accountInfo = await usdc.getAccountInfo(feeCollectorAccount);
    assert(accountInfo.amount == feeInUsdc.toNumber());

    // ------- DEPOSIT COLLATERAL -------

    await program.methods
      .depositCollateral(new anchor.BN(DEPOSIT_AMOUNT))
      .accounts({
        position: positionPda,
        transferFrom: userAccount,
        transferTo: ostiumPdaAccount,
        signer: user.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([user])
      .rpc();

    positionAccount = await program.account.position.fetch(positionPda);
    assert(
      positionAccount.collateral.eq(
        new anchor.BN(DEPOSIT_AMOUNT).add(initial_collateral)
      )
    );

    accountInfo = await usdc.getAccountInfo(userAccount);
    assert(
      accountInfo.amount ==
        USER_MINT_AMOUNT -
          positionAccount.collateral.toNumber() -
          feeInUsdc.toNumber()
    );
    accountInfo = await usdc.getAccountInfo(ostiumPdaAccount);
    assert(
      accountInfo.amount ==
        OSTIUM_MINT_AMOUNT + positionAccount.collateral.toNumber()
    );

    // ------- WITHDRAW COLLATERAL -------

    await program.methods
      .withdrawCollateral(new anchor.BN(WITHDRAW_AMOUNT))
      .accounts({
        position: positionPda,
        state: ostiumPda,
        transferFrom: ostiumPdaAccount,
        transferTo: userAccount,
        signer: user.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([user])
      .rpc();

    positionAccount = await program.account.position.fetch(positionPda);
    assert(
      positionAccount.collateral.eq(
        new anchor.BN(DEPOSIT_AMOUNT - WITHDRAW_AMOUNT).add(initial_collateral)
      )
    );

    accountInfo = await usdc.getAccountInfo(userAccount);
    assert(
      accountInfo.amount ==
        USER_MINT_AMOUNT -
          positionAccount.collateral.toNumber() -
          feeInUsdc.toNumber()
    );
    accountInfo = await usdc.getAccountInfo(ostiumPdaAccount);
    assert(
      accountInfo.amount ==
        OSTIUM_MINT_AMOUNT + positionAccount.collateral.toNumber()
    );

    // ------- CLOSE POSITION -------

    await program.methods
      .closePosition(100)
      .accounts({
        position: positionPda,
        state: ostiumPda,
        priceAccountInfo: priceFeed.publicKey,
        transferFrom: ostiumPdaAccount,
        transferTo: userAccount,
        signer: user.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([user])
      .rpc();

    positionAccount = await program.account.position.fetch(positionPda);
    assert(positionAccount.exitPrice.eq(new anchor.BN(EXIT_PRICE)));
    assert(_.isEqual(positionAccount.posStatus, { closed: {} }));

    let pnl =
      ((((EXIT_PRICE - ENTRY_PRICE) * LEVERAGE) / UNITS_IN_ONE_LEVERAGE) *
        (QUANTITY - feeInQuantity)) /
      UNITS_IN_ONE_QUANTITY;
    accountInfo = await usdc.getAccountInfo(userAccount);
    assert(accountInfo.amount == USER_MINT_AMOUNT + pnl - feeInUsdc.toNumber());
    accountInfo = await usdc.getAccountInfo(ostiumPdaAccount);
    assert(accountInfo.amount == OSTIUM_MINT_AMOUNT - pnl);

    // ------- COLLECT FEES -------

    let adminAccount = await usdc.createAccount(admin.publicKey);
    accountInfo = await usdc.getAccountInfo(adminAccount);
    assert(accountInfo.amount == 0);

    await program.methods
      .collectFees(feeInUsdc)
      .accounts({
        state: ostiumPda,
        transferFrom: feeCollectorAccount,
        transferTo: adminAccount,
        signer: admin.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();

    accountInfo = await usdc.getAccountInfo(adminAccount);
    assert(accountInfo.amount == feeInUsdc.toNumber());
  });
});
