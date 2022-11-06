import * as anchor from "@project-serum/anchor";

export const OSTIUM_SEED = Buffer.from("ostium");

export const airdropSolTokens = async (connection, wallet) => {
  const airdrop_sig = await connection.requestAirdrop(
    wallet.publicKey,
    anchor.web3.LAMPORTS_PER_SOL
  );
  await connection.confirmTransaction(airdrop_sig, "confirmed");
};
