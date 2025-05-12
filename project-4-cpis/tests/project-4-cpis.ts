import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Project4Cpis } from "../target/types/project_4_cpis";
import {
  Connection,
  Keypair,
  PublicKey,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
} from "@solana/web3.js";
import {
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getAssociatedTokenAddressSync,
  createMint,
  getMint,
  getAccount,
} from "@solana/spl-token";
import { BN } from "bn.js";
import * as assert from "assert";

describe("project-4-cpis", () => {
  // Configure the client to use the local cluster
  anchor.setProvider(anchor.AnchorProvider.env());

  const provider = anchor.AnchorProvider.env();
  const program = anchor.workspace.Project4Cpis as Program<Project4Cpis>;
  const connection = provider.connection;
  
  // Create keypairs for our test
  const payer = anchor.web3.Keypair.generate();
  const mintAuthority = anchor.web3.Keypair.generate();
  const userA = anchor.web3.Keypair.generate();
  const userB = anchor.web3.Keypair.generate();
  
  // We'll create a token mint with 9 decimals
  const decimals = 9;
  let mint: PublicKey;
  let userATokenAccount: PublicKey;
  let userBTokenAccount: PublicKey;

  // Airdrop some SOL to our payer
  before(async () => {
    // Airdrop 2 SOL to the payer
    const airdropSignature = await connection.requestAirdrop(
      payer.publicKey,
      2 * anchor.web3.LAMPORTS_PER_SOL
    );
    await connection.confirmTransaction(airdropSignature);

    // Create a new token mint
    mint = await createMint(
      connection,
      payer,
      mintAuthority.publicKey,
      null,
      decimals,
      undefined,
      undefined,
      TOKEN_PROGRAM_ID
    );
    console.log(`Created new mint: ${mint.toString()}`);

    // Derive the associated token accounts for both users
    userATokenAccount = getAssociatedTokenAddressSync(
      mint,
      userA.publicKey,
      false,
      TOKEN_PROGRAM_ID,
      ASSOCIATED_TOKEN_PROGRAM_ID
    );

    userBTokenAccount = getAssociatedTokenAddressSync(
      mint,
      userB.publicKey,
      false,
      TOKEN_PROGRAM_ID,
      ASSOCIATED_TOKEN_PROGRAM_ID
    );

    // Airdrop some SOL to userA and userB for rent
    const airdropUserA = await connection.requestAirdrop(
      userA.publicKey,
      0.1 * anchor.web3.LAMPORTS_PER_SOL
    );
    await connection.confirmTransaction(airdropUserA);

    const airdropUserB = await connection.requestAirdrop(
      userB.publicKey,
      0.1 * anchor.web3.LAMPORTS_PER_SOL
    );
    await connection.confirmTransaction(airdropUserB);
  });

  it("Creates token accounts, mints tokens, and transfers tokens", async () => {
    // Step 1: Create token account for userA
    console.log("Creating token account for userA...");
    await program.methods
      .createTokenAccount()
      .accounts({
        payer: payer.publicKey,
        owner: userA.publicKey,
        mint: mint,
        tokenAccount: userATokenAccount,
        systemProgram: anchor.web3.SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      })
      .signers([payer])
      .rpc();
    
    // Verify the account was created
    const userAAccount = await getAccount(
      connection,
      userATokenAccount,
      undefined,
      TOKEN_PROGRAM_ID
    );
    console.log(`Created token account for userA: ${userATokenAccount.toString()}`);
    assert.equal(userAAccount.mint.toString(), mint.toString());
    assert.equal(userAAccount.owner.toString(), userA.publicKey.toString());

    // Step 2: Create token account for userB
    console.log("Creating token account for userB...");
    await program.methods
      .createTokenAccount()
      .accounts({
        payer: payer.publicKey,
        owner: userB.publicKey,
        mint: mint,
        tokenAccount: userBTokenAccount,
        systemProgram: anchor.web3.SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      })
      .signers([payer])
      .rpc();
    
    // Verify the account was created
    const userBAccount = await getAccount(
      connection,
      userBTokenAccount,
      undefined,
      TOKEN_PROGRAM_ID
    );
    console.log(`Created token account for userB: ${userBTokenAccount.toString()}`);
    assert.equal(userBAccount.mint.toString(), mint.toString());
    assert.equal(userBAccount.owner.toString(), userB.publicKey.toString());

    // Step 3: Mint tokens to userA
    const mintAmount = new BN(1000 * Math.pow(10, decimals)); // 1000 tokens
    console.log(`Minting ${mintAmount.toString()} tokens to userA...`);
    await program.methods
      .mintTokens(mintAmount)
      .accounts({
        mintAuthority: mintAuthority.publicKey,
        mint: mint,
        tokenAccount: userATokenAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([mintAuthority])
      .rpc();
    
    // Verify the tokens were minted
    const userAAccountAfterMint = await getAccount(
      connection,
      userATokenAccount,
      undefined,
      TOKEN_PROGRAM_ID
    );
    console.log(`UserA balance after mint: ${userAAccountAfterMint.amount.toString()}`);
    assert.equal(userAAccountAfterMint.amount.toString(), mintAmount.toString());

    // Step 4: Transfer tokens from userA to userB
    const transferAmount = new BN(500 * Math.pow(10, decimals)); // 500 tokens
    console.log(`Transferring ${transferAmount.toString()} tokens from userA to userB...`);
    await program.methods
      .tokenTransfer(transferAmount)
      .accounts({
        signer: userA.publicKey,
        mint: mint,
        from: userATokenAccount,
        to: userBTokenAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([userA])
      .rpc();
    
    // Verify the transfer
    const userAAccountAfterTransfer = await getAccount(
      connection,
      userATokenAccount,
      undefined,
      TOKEN_PROGRAM_ID
    );
    const userBAccountAfterTransfer = await getAccount(
      connection,
      userBTokenAccount,
      undefined,
      TOKEN_PROGRAM_ID
    );
    
    console.log(`UserA balance after transfer: ${userAAccountAfterTransfer.amount.toString()}`);
    console.log(`UserB balance after transfer: ${userBAccountAfterTransfer.amount.toString()}`);
    
    // UserA should have 500 tokens left
    assert.equal(
      userAAccountAfterTransfer.amount.toString(),
      mintAmount.sub(transferAmount).toString()
    );
    
    // UserB should have 500 tokens
    assert.equal(
      userBAccountAfterTransfer.amount.toString(),
      transferAmount.toString()
    );
    
    console.log("Token operations completed successfully!");
  });
});
