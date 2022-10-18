const {
  Connection,
  sendAndConfirmTransaction,
  Keypair,
  Transaction,
  SystemProgram,
  PublicKey,
  TransactionInstruction,
  SYSVAR_RENT_PUBKEY
} = require("@solana/web3.js");
const { readFile } = require("mz/fs");
const { TOKEN_PROGRAM_ID, createMint, mintTo, createAccount, getOrCreateAssociatedTokenAccount } = require('@solana/spl-token');
// const serum = require("@project-serum/serum");

const {
  DexInstructions,
  TokenInstructions,
  OpenOrdersPda,
  MARKET_STATE_LAYOUT_V3,
  encodeInstruction,
  OpenOrders,
  Market
} = require("@project-serum/serum");

var assert = require('chai').assert;
const BN = require("bn.js");

describe("Serum Client", () => {

  let feePayer;
  let programId;
  let marketKP;
  let requestQueueKP;
  let eventQueueKP;
  let bids;
  let asks;
  let user1;
  let user2;
  let open_orders_account_user1;
  let open_orders_account_user2;
  let connection;
  let Token1;
  let Token2;
  let user1_price_acc;
  let user1_coin_acc;
  let user2_price_acc;
  let user2_coin_acc;
  let user3_coin_acc;
  let user3_price_acc;
  let market;
  let market_coin_account;
  let market_price_account;
  let vaultOwnerAndNonce;
  // const USER1_PRICE_ACC_AMOUNT = 120;
  // const USER2_COIN_ACC_AMOUNT = 140;
  // const ORDER_1_LIMIT_PRICE = 55;
  // const ORDER_1_AMOUNT = 2;
  // const ORDER_2_LIMIT_PRICE = 55;
  // const ORDER_2_AMOUNT = 20;
  const BASE_LOT_SIZE = 1;
  const QUOTE_LOT_SIZE = 1;
  const TOKEN_1_DECIMALS = 0;
  const TOKEN_2_DECIMALS = 0;

  beforeEach(async () => {

    const args = process.argv.slice(2);
  
    if (args.length > 3) {
      string = await readFile(args[4], {
        encoding: "utf8",
      });
      console.log("Loaded Keypair from ", args[4]);
      const sk = Uint8Array.from(JSON.parse(string));
      feePayer = Keypair.fromSecretKey(sk);
    } else {
      feePayer = new Keypair();
    }
  
    programId = new PublicKey(args[3]);
    marketKP = new Keypair();
    requestQueueKP = new Keypair();
    eventQueueKP = new Keypair();
    bids = new Keypair();
    asks = new Keypair();
    user1 = feePayer;
    user2 = new Keypair();
    user3 = new Keypair();
  
    connection = new Connection("http://127.0.0.1:8899", 'processed');
    const airdrop_sig = await connection.requestAirdrop(feePayer.publicKey, 2e9);
    const latestBlockHash = await connection.getLatestBlockhash();
    await connection.confirmTransaction({
      blockhash: latestBlockHash.blockhash,
      lastValidBlockHeight: latestBlockHash.lastValidBlockHeight,
      signature: airdrop_sig,
    });
    const airdrop_sig2 = await connection.requestAirdrop(user2.publicKey, 2e9);
    await connection.confirmTransaction({
      blockhash: latestBlockHash.blockhash,
      lastValidBlockHeight: latestBlockHash.lastValidBlockHeight,
      signature: airdrop_sig2,
    });
    const airdrop_sig3 = await connection.requestAirdrop(user3.publicKey, 2e9);
    await connection.confirmTransaction({
      blockhash: latestBlockHash.blockhash,
      lastValidBlockHeight: latestBlockHash.lastValidBlockHeight,
      signature: airdrop_sig3,
    });
  
    Token1 = await createMint(
      connection,
      feePayer,
      feePayer.publicKey,
      null,
      TOKEN_1_DECIMALS,
      undefined,
      undefined,
      TOKEN_PROGRAM_ID
    );
    Token2 = await createMint(
      connection,
      feePayer,
      feePayer.publicKey,
      null,

      TOKEN_2_DECIMALS,
      undefined,
      undefined,
      TOKEN_PROGRAM_ID
    );
  
    user1_price_acc = await createAccount(
      connection,
      user1,
      Token1,
      user1.publicKey
    )
    user1_coin_acc = await createAccount(
      connection,
      user1,
      Token2,
      user1.publicKey
    )
    user2_coin_acc = await createAccount(
      connection,
      user2,
      Token2,
      user2.publicKey
    )
    user2_price_acc = await createAccount(
      connection,
      user2,
      Token1,
      user2.publicKey
    )
    user3_coin_acc = await createAccount(
      connection,
      user3,
      Token2,
      user3.publicKey
    )
    user3_price_acc = await createAccount(
      connection,
      user3,
      Token1,
      user3.publicKey
    )

    vaultOwnerAndNonce = await getVaultOwnerAndNonce(
      marketKP.publicKey,
      programId
    )

    market_coin_account = await getOrCreateAssociatedTokenAccount(
      connection,
      feePayer,
      Token2,
      vaultOwnerAndNonce[0],
      true
    );
    
    market_price_account = await getOrCreateAssociatedTokenAccount(
      connection,
      feePayer,
      Token1,
      vaultOwnerAndNonce[0],
      true
    );

    const tx = new Transaction();

    tx.add(
      SystemProgram.createAccount({
        fromPubkey: feePayer.publicKey,
        newAccountPubkey: marketKP.publicKey,
        lamports: await connection.getMinimumBalanceForRentExemption(
          388
        ),
        space: 388,
        programId: programId,
      }),
      SystemProgram.createAccount({
        fromPubkey: feePayer.publicKey,
        newAccountPubkey: requestQueueKP.publicKey,
        lamports: await connection.getMinimumBalanceForRentExemption(5120 + 12),
        space: 5120 + 12,
        programId: programId,
      }),
      SystemProgram.createAccount({
        fromPubkey: feePayer.publicKey,
        newAccountPubkey: eventQueueKP.publicKey,
        lamports: await connection.getMinimumBalanceForRentExemption(262144 + 12),
        space: 262144 + 12,
        programId: programId,
      }),
      SystemProgram.createAccount({
        fromPubkey: feePayer.publicKey,
        newAccountPubkey: bids.publicKey,
        lamports: await connection.getMinimumBalanceForRentExemption(65536 + 12),
        space: 65536 + 12,
        programId: programId,
      }),
      SystemProgram.createAccount({
        fromPubkey: feePayer.publicKey,
        newAccountPubkey: asks.publicKey,
        lamports: await connection.getMinimumBalanceForRentExemption(65536 + 12),
        space: 65536 + 12,
        programId: programId,
      }),
      DexInstructions.initializeMarket({
        market: marketKP.publicKey,
        requestQueue: requestQueueKP.publicKey,
        eventQueue: eventQueueKP.publicKey,
        bids: bids.publicKey,
        asks: asks.publicKey,
        baseVault: market_coin_account.address,
        quoteVault: market_price_account.address,
        baseMint: Token2,
        quoteMint: Token1,
        baseLotSize: new BN(BASE_LOT_SIZE),
        quoteLotSize: new BN(QUOTE_LOT_SIZE),
        feeRateBps: 0,
        vaultSignerNonce: vaultOwnerAndNonce[1],
        quoteDustThreshold: new BN(100),
        programId: programId,
      })
    );

    const txid = await sendAndConfirmTransaction(
      connection,
      tx,
      [feePayer, marketKP, requestQueueKP, eventQueueKP, bids, asks],
      {
        skipPreflight: true,
        preflightCommitment: "confirmed",
        confirmation: "confirmed",
      }
    );

    market = await Market.load(connection, marketKP.publicKey, {}, programId);

    console.log('Initialization Complete: Market: ', market);
  })


  // This test creates an open order account for the user1,
  // Then, the user1 places a BUY order.
  it.only("Creates Orderbook of Limit Orders", async () => {

    const tx = new Transaction();

    const open_orders_account_user1 = new Keypair();

    const USER1_PRICE_BALANCE = 850*(10 ** TOKEN_1_DECIMALS)
    const ORDER_1_PRICE = 41
    const ORDER_1_QTY = 2
    const ORDER_2_PRICE = 40
    const ORDER_2_QTY = 3
    const ORDER_3_PRICE = 39
    const ORDER_3_QTY = 4

    const USER1_ORDER_SUM = (ORDER_1_PRICE*ORDER_1_QTY + ORDER_2_PRICE*ORDER_2_QTY + ORDER_3_PRICE*ORDER_3_QTY) * (10 ** TOKEN_1_DECIMALS)

    await mintTo(
      connection,
      feePayer,
      Token1,
      user1_price_acc,
      feePayer,
      USER1_PRICE_BALANCE
    )

    tx.add(
      await OpenOrders.makeCreateAccountTransaction(
        connection,
        marketKP.publicKey,
        user1.publicKey,
        open_orders_account_user1.publicKey,
        programId,
      ),
    ).add(
      market.makePlaceOrderInstruction(connection, {
        owner: feePayer.publicKey,
        payer: user1_price_acc,
        side: "buy",
        price: ORDER_1_PRICE,
        size: ORDER_1_QTY,
        orderType: 'limit',
        clientId: undefined,
        openOrdersAddressKey: open_orders_account_user1.publicKey,
        openOrdersAccount: undefined,
        feeDiscountPubkey: undefined
      })
    ).add(
      market.makePlaceOrderInstruction(connection, {
        owner: feePayer.publicKey,
        payer: user1_price_acc,
        side: "buy",
        price: ORDER_2_PRICE,
        size: ORDER_2_QTY,
        orderType: 'limit',
        clientId: undefined,
        openOrdersAddressKey: open_orders_account_user1.publicKey,
        openOrdersAccount: undefined,
        feeDiscountPubkey: undefined
      })
    ).add(
      market.makePlaceOrderInstruction(connection, {
        owner: feePayer.publicKey,
        payer: user1_price_acc,
        side: "buy",
        price: ORDER_3_PRICE,
        size: ORDER_3_QTY,
        orderType: 'limit',
        clientId: undefined,
        openOrdersAddressKey: open_orders_account_user1.publicKey,
        openOrdersAccount: undefined,
        feeDiscountPubkey: undefined
      })
    )

    let txid = await sendAndConfirmTransaction(
      connection,
      tx,
      [user1, open_orders_account_user1],
      {
        skipPreflight: true,
        preflightCommitment: "confirmed",
        confirmation: "confirmed",
      }
    );
  
    market = await Market.load(connection, marketKP.publicKey, {}, programId);

    const base_vault_acc = await connection.getTokenAccountBalance(new PublicKey(market._decoded.baseVault.toBase58()))
    const quote_vault_acc = await connection.getTokenAccountBalance(new PublicKey(market._decoded.quoteVault.toBase58()))
    const user1_price_acc_info = (await connection.getTokenAccountBalance(user1_price_acc)).value.amount;

    // This assert checks that the base_vault account is still empty
    assert.equal(parseInt(base_vault_acc.value.amount), 0);
    // This checks that the quote_vault is now equal to the PRICE*AMOUNT of the user1's order
    assert.equal(parseInt(quote_vault_acc.value.amount), USER1_ORDER_SUM);
    // This checks to make sure that the user1's Price token account has decremented the correct amount
    assert.equal(user1_price_acc_info, (USER1_PRICE_BALANCE) - USER1_ORDER_SUM);

    const tx2 = new Transaction();
    const open_orders_account_user2 = new Keypair();
    const USER2_COIN_BALANCE = 23*(10 ** TOKEN_2_DECIMALS)
    const ORDER_4_PRICE = 42
    const ORDER_4_QTY = 3
    const ORDER_5_PRICE = 43
    const ORDER_5_QTY = 4
    const ORDER_6_PRICE = 44
    const ORDER_6_QTY = 5
    const USER2_ORDER_SUM = (ORDER_4_QTY + ORDER_5_QTY + ORDER_6_QTY) * (10 ** TOKEN_2_DECIMALS)

    await mintTo(
      connection,
      feePayer,
      Token2,
      user2_coin_acc,
      feePayer,
      USER2_COIN_BALANCE
    )

    tx2.add(
      await OpenOrders.makeCreateAccountTransaction(
        connection,
        marketKP.publicKey,
        user2.publicKey,
        open_orders_account_user2.publicKey,
        programId,
      ),
    ).add(
      market.makePlaceOrderInstruction(connection, {
        owner: user2.publicKey,
        payer: user2_coin_acc,
        side: "sell",
        price: ORDER_4_PRICE,
        size: ORDER_4_QTY,
        orderType: 'limit',
        clientId: undefined,
        openOrdersAddressKey: open_orders_account_user2.publicKey,
        openOrdersAccount: undefined,
        feeDiscountPubkey: undefined
      })
    ).add(
      market.makePlaceOrderInstruction(connection, {
        owner: user2.publicKey,
        payer: user2_coin_acc,
        side: "sell",
        price: ORDER_5_PRICE,
        size: ORDER_5_QTY,
        orderType: 'limit',
        clientId: undefined,
        openOrdersAddressKey: open_orders_account_user2.publicKey,
        openOrdersAccount: undefined,
        feeDiscountPubkey: undefined
      })
    ).add(
      market.makePlaceOrderInstruction(connection, {
        owner: user2.publicKey,
        payer: user2_coin_acc,
        side: "sell",
        price: ORDER_6_PRICE,
        size: ORDER_6_QTY,
        orderType: 'limit',
        clientId: undefined,
        openOrdersAddressKey: open_orders_account_user2.publicKey,
        openOrdersAccount: undefined,
        feeDiscountPubkey: undefined
      })
    )

    let txid2 = await sendAndConfirmTransaction(
      connection,
      tx2,
      [user2, open_orders_account_user2],
      {
        skipPreflight: true,
        preflightCommitment: "confirmed",
        confirmation: "confirmed",
      }
    );

    market = await Market.load(connection, marketKP.publicKey, {}, programId);

    const base_vault_acc2 = await connection.getTokenAccountBalance(new PublicKey(market._decoded.baseVault.toBase58()))
    //const quote_vault_acc2 = await connection.getTokenAccountBalance(new PublicKey(market._decoded.quoteVault.toBase58()))
    const user2_coin_acc_info2 = (await connection.getTokenAccountBalance(user2_coin_acc)).value.amount;

    // This checks that the base_vault is now equal to the QTY of the user2's order
    assert.equal(parseInt(base_vault_acc2.value.amount), USER2_ORDER_SUM);
    // This checks to make sure that the user2's Coin account has decremented the correct amount
    assert.equal(user2_coin_acc_info2, (USER2_COIN_BALANCE) - USER2_ORDER_SUM);
    
    assert.equal(Number(market._decoded.quoteFeesAccrued), 0);


    // Send take

    const tx3 = new Transaction();
    const SEND_TAKE_PRICE = 42
    const SEND_TAKE_QTY = 3
    const USER_3_PRICE_BALANCE = 230*(10 ** TOKEN_2_DECIMALS)

    await mintTo(
      connection,
      user3,
      Token1,
      user3_price_acc,
      feePayer,
      USER_3_PRICE_BALANCE
    )


    const price = priceNumberToLots(SEND_TAKE_PRICE, TOKEN_2_DECIMALS, TOKEN_1_DECIMALS, BASE_LOT_SIZE, QUOTE_LOT_SIZE)
    const max_native_pc_qty = new BN(QUOTE_LOT_SIZE).mul(baseSizeNumberToLots(SEND_TAKE_QTY, TOKEN_2_DECIMALS, BASE_LOT_SIZE)).mul(priceNumberToLots(SEND_TAKE_PRICE, TOKEN_2_DECIMALS, TOKEN_1_DECIMALS, BASE_LOT_SIZE, QUOTE_LOT_SIZE))
    const max_native_pc_qty_including_fees = max_native_pc_qty.add(new BN(Math.ceil(0.0004 * max_native_pc_qty.toNumber())))
    const max_coin_qty = baseSizeNumberToLots(SEND_TAKE_QTY, TOKEN_2_DECIMALS, BASE_LOT_SIZE)

    const SEND_TAKE_DATA = Buffer.concat([
      Buffer.from(new Uint8Array([0])),
      Buffer.from(new Uint8Array((new BN(13)).toArray("le", 4))),
      Buffer.from(new Uint8Array((new BN(0)).toArray("le", 4))), //side = buy
      Buffer.from(new Uint8Array((price.toArray("le", 8)))), //limit_price
      Buffer.from(new Uint8Array(max_coin_qty.toArray("le", 8))), //max_coin
      Buffer.from(new Uint8Array(max_native_pc_qty_including_fees.toArray("le", 8))), // max_price
      Buffer.from(new Uint8Array((new BN(1)).toArray("le", 8))), //min_coin
      Buffer.from(new Uint8Array((new BN(1)).toArray("le", 8))), //min_price
      Buffer.from(new Uint8Array((new BN(60000)).toArray("le", 2))), // limit
    ])

    // console.log('send_take: max_native_pc_qty',max_native_pc_qty.toNumber())
    // console.log('send_take: max_native_pc_qty_including_fees',max_native_pc_qty_including_fees.toNumber())
    // console.log('send_take: max_coin_qty',max_coin_qty.toNumber())
    // console.log('send_take: price',price.toNumber())

    const takeIx = new TransactionInstruction({
      keys: [
        {
          pubkey: marketKP.publicKey,
          isSigner: false,
          isWritable: true,
        },
        {
          pubkey: requestQueueKP.publicKey,
          isSigner: false,
          isWritable: true,
        },
        {
          pubkey: eventQueueKP.publicKey,
          isSigner: false,
          isWritable: true,
        },
        {
          pubkey: bids.publicKey,
          isSigner: false,
          isWritable: true,
        },
        {
          pubkey: asks.publicKey,
          isSigner: false,
          isWritable: true,
        },
        {
          pubkey: user3_coin_acc,
          isSigner: false,
          isWritable: true,
        },
        {
          pubkey: user3_price_acc,
          isSigner: false,
          isWritable: true,
        },
        {
          pubkey: user3.publicKey,
          isSigner: true,
          isWritable: false,
        },
        {
          pubkey: market_coin_account.address,
          isSigner: false,
          isWritable: true,
        },
        {
          pubkey: market_price_account.address,
          isSigner: false,
          isWritable: true,
        },
        {
          pubkey: TOKEN_PROGRAM_ID,
          isSigner: false,
          isWritable: false,
        },
        {
          pubkey: vaultOwnerAndNonce[0],
          isSigner: false,
          isWritable: false,
        },
      ],
      programId,
      data: SEND_TAKE_DATA,
    })

    tx3.add(takeIx);

    // let txid3 = await sendAndConfirmTransaction(
    //   connection,
    //   tx3,
    //   [user3]

    let txid3 = await sendAndConfirmTransaction(
      connection,
      tx3,
      [user3],
      {
        skipPreflight: true,
        preflightCommitment: "confirmed",
        confirmation: "confirmed",
      }
    );

    market = await Market.load(connection, marketKP.publicKey, {}, programId);
    console.log('marketKP.publicKey', marketKP.publicKey.toBase58())

    // console.log('0.0004 * max_native_pc_qty_including_fees.toNumber()', 0.0004 * max_native_pc_qty.toNumber())

    // assert.equal(market._decoded.quoteFeesAccrued.toNumber(), Math.ceil(0.0004 * max_native_pc_qty.toNumber()))

    // console.log('market._decoded', market._decoded.quoteFeesAccrued.toNumber())


  });

  // This test creates an open order account for the user2, and then places a limit SELL order
  it("Creates Limit Order that Takes", async () => {

    const tx = new Transaction();
    const open_orders_account_user1 = new Keypair();
    const USER1_PRICE_BALANCE = 850*(10 ** TOKEN_1_DECIMALS)
    const ORDER_1_PRICE = 41
    const ORDER_1_QTY = 2
    const USER1_ORDER_SUM = (ORDER_1_PRICE*ORDER_1_QTY) * (10 ** TOKEN_1_DECIMALS)
    await mintTo(
      connection,
      feePayer,
      Token1,
      user1_price_acc,
      feePayer,
      USER1_PRICE_BALANCE
    )
    tx.add(
      await OpenOrders.makeCreateAccountTransaction(
        connection,
        marketKP.publicKey,
        user1.publicKey,
        open_orders_account_user1.publicKey,
        programId,
      ),
    ).add(
      market.makePlaceOrderInstruction(connection, {
        owner: user1.publicKey,
        payer: user1_price_acc,
        side: "buy",
        price: ORDER_1_PRICE,
        size: ORDER_1_QTY,
        orderType: 'limit',
        clientId: undefined,
        openOrdersAddressKey: open_orders_account_user1.publicKey,
        openOrdersAccount: undefined,
        feeDiscountPubkey: undefined
      })
    )
    let txid = await sendAndConfirmTransaction(
      connection,
      tx,
      [user1, open_orders_account_user1],
      {
        skipPreflight: true,
        preflightCommitment: "confirmed",
        confirmation: "confirmed",
      }
    );

    const tx2 = new Transaction();
    const open_orders_account_user2 = new Keypair();
    const USER2_COIN_BALANCE = 23*(10 ** TOKEN_2_DECIMALS)
    const ORDER_2_PRICE = 41
    const ORDER_2_QTY = 2
    const USER2_ORDER_SUM = (ORDER_2_QTY) * (10 ** TOKEN_2_DECIMALS)

    await mintTo(
      connection,
      feePayer,
      Token2,
      user2_coin_acc,
      feePayer,
      USER2_COIN_BALANCE
    )

    tx2.add(
      await OpenOrders.makeCreateAccountTransaction(
        connection,
        marketKP.publicKey,
        user2.publicKey,
        open_orders_account_user2.publicKey,
        programId,
      ),
    ).add(
      market.makePlaceOrderInstruction(connection, {
        owner: user2.publicKey,
        payer: user2_coin_acc,
        side: "sell",
        price: ORDER_2_PRICE,
        size: ORDER_2_QTY,
        orderType: 'limit',
        clientId: undefined,
        openOrdersAddressKey: open_orders_account_user2.publicKey,
        openOrdersAccount: undefined,
        feeDiscountPubkey: undefined
      })
    )

    let txid2 = await sendAndConfirmTransaction(
      connection,
      tx2,
      [user2, open_orders_account_user2],
      {
        skipPreflight: true,
        preflightCommitment: "confirmed",
        confirmation: "confirmed",
      }
    );

    market = await Market.load(connection, marketKP.publicKey, {}, programId);

    const base_vault_acc = await connection.getTokenAccountBalance(new PublicKey(market._decoded.baseVault.toBase58()))
    //const quote_vault_acc2 = await connection.getTokenAccountBalance(new PublicKey(market._decoded.quoteVault.toBase58()))
    const user2_coin_acc_info2 = (await connection.getTokenAccountBalance(user2_coin_acc)).value.amount;

    // This checks that the base_vault is now equal to the QTY of the user2's order
    assert.equal(parseInt(base_vault_acc.value.amount), 0);
    // This checks to make sure that the user2's Coin account has decremented the correct amount
    assert.equal(user2_coin_acc_info2, (USER2_COIN_BALANCE) - USER2_ORDER_SUM);


  });


  it("Sends Take - Bid", async () => { 

    const tx5 = new Transaction();

    processSenduser2 = new Keypair();
    
    processSenduser2_coin_acc = await getOrCreateAssociatedTokenAccount(
      connection,
      feePayer,
      Token2,
      processSenduser2.publicKey
    )

    processSenduser2_price_acc = await getOrCreateAssociatedTokenAccount(
      connection,
      feePayer,
      Token1,
      processSenduser2.publicKey
    ) 

    await mintTo(
      connection,
      feePayer,
      Token1,
      processSenduser2_price_acc.address,
      feePayer,
      SEND_user2_PRICE_ACC_AMOUNT
    )

    let processSenduser2_coin_acc_info = await connection.getTokenAccountBalance(new PublicKey(processSenduser2_coin_acc.address.toBase58()))
    let processSenduser2_price_acc_info = await connection.getTokenAccountBalance(new PublicKey(processSenduser2_price_acc.address.toBase58()))

    assert.equal(processSenduser2_coin_acc_info.value.amount, 0)
    assert.equal(processSenduser2_price_acc_info.value.amount, SEND_user2_PRICE_ACC_AMOUNT)


    const takeIx = new TransactionInstruction({
      keys: [
        {
          pubkey: marketKP.publicKey,
          isSigner: false,
          isWritable: true,
        },
        {
          pubkey: requestQueueKP.publicKey,
          isSigner: false,
          isWritable: true,
        },
        {
          pubkey: eventQueueKP.publicKey,
          isSigner: false,
          isWritable: true,
        },
        {
          pubkey: bids.publicKey,
          isSigner: false,
          isWritable: true,
        },
        {
          pubkey: asks.publicKey,
          isSigner: false,
          isWritable: true,
        },
        {
          pubkey: processSenduser2_coin_acc.address,
          isSigner: false,
          isWritable: true,
        },
        {
          pubkey: processSenduser2_price_acc.address,
          isSigner: false,
          isWritable: true,
        },
        {
          pubkey: processSenduser2.publicKey,
          isSigner: true,
          isWritable: false,
        },
        {
          pubkey: market_coin_account.address,
          isSigner: false,
          isWritable: true,
        },
        {
          pubkey: market_price_account.address,
          isSigner: false,
          isWritable: true,
        },
        {
          pubkey: TOKEN_PROGRAM_ID,
          isSigner: false,
          isWritable: false,
        },
        {
          pubkey: vaultOwner,
          isSigner: false,
          isWritable: false,
        },
      ],
      programId,
      data: SEND_TAKE_DATA_1,
    })

    tx5.add(takeIx)

    let txid5 = await sendAndConfirmTransaction(
      connection,
      tx5,
      [feePayer, processSenduser2],
      {
        skipPreflight: true,
        preflightCommitment: "confirmed",
        confirmation: "confirmed",
      }
    );

    processSenduser2_coin_acc_info = await connection.getTokenAccountBalance(new PublicKey(processSenduser2_coin_acc.address.toBase58()))
    processSenduser2_price_acc_info = await connection.getTokenAccountBalance(new PublicKey(processSenduser2_price_acc.address.toBase58()))

    assert.isAbove(parseInt(processSenduser2_coin_acc_info.value.amount), 0)
    assert.equal(parseInt(processSenduser2_price_acc_info.value.amount), SEND_user2_PRICE_ACC_AMOUNT-SEND_TAKE_PRICE_AMOUNT)

    market = await serum.Market.load(connection, marketKP.publicKey, {}, programId);

    // These asserts ensure that the market's base deposits calculations are correct
    assert.equal(parseInt(market._decoded.baseDepositsTotal.toString()), ORDER_2_AMOUNT - ORDER_1_AMOUNT - SEND_TAKE_COIN_AMOUNT)
    assert.isAbove(parseInt(market._decoded.quoteDepositsTotal.toString()), 0)


    const tx6 = new Transaction();

    tx6.add(
      market.makeConsumeEventsInstruction([open_orders_account_user2.publicKey], 100)
    )

    let txid6 = await sendAndConfirmTransaction(
      connection,
      tx6,
      [feePayer],
      {
        skipPreflight: true,
        preflightCommitment: "confirmed",
        confirmation: "confirmed",
      }
    );

    for (let openOrders of await market.findOpenOrdersAccountsForOwner(
      connection,
      user2.publicKey,
    )) {
      if (openOrders.baseTokenFree > 0 || openOrders.quoteTokenFree > 0) {
  
        await market.settleFunds(
          connection,
          user2,
          openOrders,
          user2_coin_acc.address,
          user2_price_acc.address,
        );
      }
    };

    user2_coin_acc_info = await connection.getTokenAccountBalance(new PublicKey(user2_coin_acc.address.toBase58()))
    user2_price_acc_info = await connection.getTokenAccountBalance(new PublicKey(user2_price_acc.address.toBase58()))

    // This assert ensures that the user2, upon settle funds, gets the correct amount of price currency, as it corresponds to the send_take order
    assert.isAbove(parseInt(user2_price_acc_info.value.amount), SEND_TAKE_PRICE_AMOUNT)
    
  });

  const LEFTOVER_user1_PRICE_ACC_AMOUNT = 10
  const MINT_TO_user1_PRICE_ACC = 250
  const ORDER_7_LIMIT_PRICE = 50
  const ORDER_7_AMOUNT = 3
  const ORDER_8_COIN_AMOUNT = 4
  const ORDER_8_PRICE_AMOUNT = 201
  const SEND_TAKE_DATA_2 = Buffer.concat([
    Buffer.from(new Uint8Array([0])),
    Buffer.from(new Uint8Array((new BN(13)).toArray("le", 4))), // instruction number
    Buffer.from(new Uint8Array((new BN(1)).toArray("le", 4))), //side
    Buffer.from(new Uint8Array((new BN(ORDER_7_LIMIT_PRICE)).toArray("le", 8))), //limit_price
    Buffer.from(new Uint8Array((new BN(ORDER_8_COIN_AMOUNT)).toArray("le", 8))), //max_coin
    Buffer.from(new Uint8Array((new BN(ORDER_8_PRICE_AMOUNT)).toArray("le", 8))), // max_price
    Buffer.from(new Uint8Array((new BN(1)).toArray("le", 8))), //min_coin
    Buffer.from(new Uint8Array((new BN(1)).toArray("le", 8))), //min_price
    Buffer.from(new Uint8Array((new BN(60000)).toArray("le", 2))), // limit
  ])

  // This test ensures that send take works for ASKs as well as bids
  it("Sends Take - Ask", async () => {

    await mintTo(
      connection,
      feePayer,
      Token1,
      user1_price_acc.address,
      feePayer,
      MINT_TO_user1_PRICE_ACC
    )

    user1_price_acc_info = await connection.getTokenAccountBalance(new PublicKey(user1_price_acc.address.toBase58()))

    assert.equal(parseInt(user1_price_acc_info.value.amount), LEFTOVER_user1_PRICE_ACC_AMOUNT + MINT_TO_user1_PRICE_ACC)

    const tx7 = new Transaction();

    tx7.add(
      market.makePlaceOrderInstruction(connection, {
        owner: feePayer.publicKey,
        payer: user1_price_acc.address,
        side: "buy",
        price: ORDER_7_LIMIT_PRICE,
        size: ORDER_7_AMOUNT,
        orderType: 'limit',
        clientId: undefined,
        openOrdersAddressKey: open_orders_account_user1.publicKey,
        openOrdersAccount: undefined,
        feeDiscountPubkey: undefined
      })
    );

    let txid7 = await sendAndConfirmTransaction(
      connection,
      tx7,
      [feePayer],
      {
        skipPreflight: true,
        preflightCommitment: "confirmed",
        confirmation: "confirmed",
      }
    );

    user1_price_acc_info = await connection.getTokenAccountBalance(new PublicKey(user1_price_acc.address.toBase58()))

    assert.equal(parseInt(user1_price_acc_info.value.amount), LEFTOVER_user1_PRICE_ACC_AMOUNT + MINT_TO_user1_PRICE_ACC - (ORDER_7_LIMIT_PRICE*ORDER_7_AMOUNT))

    const tx8 = new Transaction();

    const takeIx = new TransactionInstruction({
      keys: [
        {
          pubkey: marketKP.publicKey,
          isSigner: false,
          isWritable: true,
        },
        {
          pubkey: requestQueueKP.publicKey,
          isSigner: false,
          isWritable: true,
        },
        {
          pubkey: eventQueueKP.publicKey,
          isSigner: false,
          isWritable: true,
        },
        {
          pubkey: bids.publicKey,
          isSigner: false,
          isWritable: true,
        },
        {
          pubkey: asks.publicKey,
          isSigner: false,
          isWritable: true,
        },
        {
          pubkey: processSenduser2_coin_acc.address,
          isSigner: false,
          isWritable: true,
        },
        {
          pubkey: processSenduser2_price_acc.address,
          isSigner: false,
          isWritable: true,
        },
        {
          pubkey: processSenduser2.publicKey,
          isSigner: true,
          isWritable: false,
        },
        {
          pubkey: market_coin_account.address,
          isSigner: false,
          isWritable: true,
        },
        {
          pubkey: market_price_account.address,
          isSigner: false,
          isWritable: true,
        },
        {
          pubkey: TOKEN_PROGRAM_ID,
          isSigner: false,
          isWritable: false,
        },
        {
          pubkey: vaultOwner,
          isSigner: false,
          isWritable: false,
        },
      ],
      programId,
      data: SEND_TAKE_DATA_2,
    })

    tx8.add(takeIx)

    let processSenduser2_coin_acc_info = await connection.getTokenAccountBalance(new PublicKey(processSenduser2_coin_acc.address.toBase58()))
    let processSenduser2_price_acc_info = await connection.getTokenAccountBalance(new PublicKey(processSenduser2_price_acc.address.toBase58()))

    let coin_amt_before = processSenduser2_coin_acc_info.value.amount
    let price_amt_before = processSenduser2_price_acc_info.value.amount

    console.log('processSenduser2_coin_acc.value.amount before', processSenduser2_coin_acc_info.value.amount)
    console.log('processSenduser2_price_acc.value.amount before', processSenduser2_price_acc_info.value.amount)

    market = await serum.Market.load(connection, marketKP.publicKey, {}, programId);

    console.log('market baseDepositsTotal', market._decoded.baseDepositsTotal.toString())
    console.log('market quoteDepositsTotal', market._decoded.quoteDepositsTotal.toString())

    let txid8 = await sendAndConfirmTransaction(
      connection,
      tx8,
      [feePayer, processSenduser2],
      {
        skipPreflight: true,
        preflightCommitment: "confirmed",
        confirmation: "confirmed",
      }
    );

    processSenduser2_coin_acc_info = await connection.getTokenAccountBalance(new PublicKey(processSenduser2_coin_acc.address.toBase58()))
    processSenduser2_price_acc_info = await connection.getTokenAccountBalance(new PublicKey(processSenduser2_price_acc.address.toBase58()))

    let coin_amt_after = processSenduser2_coin_acc_info.value.amount
    let price_amt_after = processSenduser2_price_acc_info.value.amount

    // assert.equal(parseInt(coin_amt_after), parseInt(coin_amt_before) - ORDER_8_COIN_AMOUNT)
    // assert.isAbove(parseInt(price_amt_after), parseInt(price_amt_before))

    console.log('processSenduser2_coin_acc.value.amount', processSenduser2_coin_acc_info.value.amount)
    console.log('processSenduser2_price_acc.value.amount', processSenduser2_price_acc_info.value.amount)

    market = await serum.Market.load(connection, marketKP.publicKey, {}, programId);

    console.log('market baseDepositsTotal', market._decoded.baseDepositsTotal.toString())
    console.log('market quoteDepositsTotal', market._decoded.quoteDepositsTotal.toString())

    console.log('marketKP.pk', marketKP.publicKey.toBase58())
    console.log('marketKP', marketKP)
    console.log('vaultOwner', vaultOwner.toBase58())

    user1_price_acc = await getOrCreateAssociatedTokenAccount(
      connection,
      user1,
      Token1,
      user1.publicKey
    )
    user1_coin_acc = await getOrCreateAssociatedTokenAccount(
      connection,
      user1,
      Token2,
      user1.publicKey
    )
    console.log('token1 PRICE TOKEN PK', Token1.toBase58())
    console.log('token2 COIN TOKEN PK', Token2.toBase58())
    console.log('user1PriceAccamount', user1_price_acc.address.toBase58())
    console.log('user1PriceAccamount', user1_price_acc.amount)
    console.log('user1CoinAccamount', user1_coin_acc.address.toBase58())
    console.log('user1CoinAccamount', user1_coin_acc.amount)
    console.log('user1.publicKey', user1.publicKey.toBase58())
    console.log('user1', user1)


    await mintTo(
      connection,
      feePayer,
      Token1,
      user1_price_acc.address,
      feePayer,
      1130
    )

    await mintTo(
      connection,
      user2,
      Token2,
      user2_coin_acc.address,
      feePayer,
      10000
    )

    const tx9 = new Transaction();

    tx9.add(
      market.makePlaceOrderInstruction(connection, {
        owner: feePayer.publicKey,
        payer: user1_price_acc.address,
        side: "buy",
        price: 52,
        size: 10,
        orderType: 'limit',
        clientId: undefined,
        openOrdersAddressKey: open_orders_account_user1.publicKey,
        openOrdersAccount: undefined,
        feeDiscountPubkey: undefined
      })
    ).add(
      market.makePlaceOrderInstruction(connection, {
        owner: feePayer.publicKey,
        payer: user1_price_acc.address,
        side: "buy",
        price: 50,
        size: 8,
        orderType: 'limit',
        clientId: undefined,
        openOrdersAddressKey: open_orders_account_user1.publicKey,
        openOrdersAccount: undefined,
        feeDiscountPubkey: undefined
      })
    ).add(
      market.makePlaceOrderInstruction(connection, {
        owner: feePayer.publicKey,
        payer: user1_price_acc.address,
        side: "buy",
        price: 53,
        size: 5,
        orderType: 'limit',
        clientId: undefined,
        openOrdersAddressKey: open_orders_account_user1.publicKey,
        openOrdersAccount: undefined,
        feeDiscountPubkey: undefined
      })
    );

    let txid9 = await sendAndConfirmTransaction(
      connection,
      tx9,
      [feePayer],
      {
        skipPreflight: true,
        preflightCommitment: "confirmed",
        confirmation: "confirmed",
      }
    );
  
    console.log('Transaction 9 complete', txid9);

    const tx10 = new Transaction();


    tx10.add(
      market.makePlaceOrderInstruction(connection, {
        owner: user2.publicKey,
        payer: user2_coin_acc.address,
        side: "sell",
        price: 57,
        size: 5,
        orderType: 'limit',
        clientId: undefined,
        openOrdersAddressKey: open_orders_account_user2.publicKey,
        openOrdersAccount: undefined,
        feeDiscountPubkey: undefined
      })
    ).add(
      market.makePlaceOrderInstruction(connection, {
        owner: user2.publicKey,
        payer: user2_coin_acc.address,
        side: "sell",
        price: 58,
        size: 8,
        orderType: 'limit',
        clientId: undefined,
        openOrdersAddressKey: open_orders_account_user2.publicKey,
        openOrdersAccount: undefined,
        feeDiscountPubkey: undefined
      })
    );

    let txid10 = await sendAndConfirmTransaction(
      connection,
      tx10,
      [user2],
      {
        skipPreflight: true,
        preflightCommitment: "confirmed",
        confirmation: "confirmed",
      }
    );
  
    console.log('Transaction 10 complete', txid10);


  });

})


async function getVaultOwnerAndNonce(marketPublicKey, dexProgramId = DEX_PID) {
  console.log('marketPublicKey', marketPublicKey)
  console.log('dexProgramId', dexProgramId)
  const nonce = new BN(0);
  while (nonce.toNumber() < 255) {
    try {
      const vaultOwner = await PublicKey.createProgramAddress(
        [marketPublicKey.toBuffer(), nonce.toArrayLike(Buffer, "le", 8)],
        dexProgramId
      );
      return [vaultOwner, nonce];
    } catch (e) {
      nonce.iaddn(1);
    }
  }
  throw new Error("Unable to find nonce");
}

function priceNumberToLots(price, baseDecimals, quoteDecimals, baseLot, quoteLot) {
  console.log('WTF>>>', price, baseDecimals, quoteDecimals, baseLot, quoteLot)
  baseDecimals = new BN(baseDecimals);
  quoteDecimals = new BN(quoteDecimals);
  baseLot = new BN(baseLot);
  quoteLot = new BN(quoteLot);
  return new BN(
    Math.round(
      (price *
        Math.pow(10, quoteDecimals) *
        baseLot.toNumber()) /
      (Math.pow(10, baseDecimals) *
        quoteLot.toNumber()),
    ),
  );
}

function baseSizeNumberToLots(size, baseDecimals, baseLot) {
  baseDecimals = new BN(baseDecimals);
  baseLot = new BN(baseLot);
  const native = new BN(
    Math.round(size * Math.pow(10, baseDecimals)),
  );
  // rounds down to the nearest lot size
  return native.div(baseLot);
}




