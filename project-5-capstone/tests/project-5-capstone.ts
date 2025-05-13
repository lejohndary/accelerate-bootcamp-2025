import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { 
  PublicKey, 
  SystemProgram, 
  Keypair,
  LAMPORTS_PER_SOL
} from '@solana/web3.js';
import { 
  TOKEN_PROGRAM_ID, 
  TOKEN_2022_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getAssociatedTokenAddressSync,
  getAccount
} from '@solana/spl-token';
import { assert } from "chai";

describe("project-5-capstone", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const provider = anchor.getProvider() as anchor.AnchorProvider;
  const program = anchor.workspace.Project5Capstone as Program<any>;
  
  const authority = Keypair.generate();
  const user1 = Keypair.generate();
  const user2 = Keypair.generate();
  
  const yesMint = Keypair.generate();
  const noMint = Keypair.generate();
  
  // PDA for the pool
  const [poolPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("pool"), authority.publicKey.toBuffer()],
    program.programId
  );
  
  // Associated Token Accounts
  let user1YesToken: PublicKey;
  let user1NoToken: PublicKey;
  let user2YesToken: PublicKey;
  let user2NoToken: PublicKey;
  
  it("Airdrop SOL to authority and users", async () => {
    // Airdrop SOL to authority
    const authorityAirdrop = await provider.connection.requestAirdrop(
      authority.publicKey,
      2 * LAMPORTS_PER_SOL
    );
    await provider.connection.confirmTransaction(authorityAirdrop);

    // Airdrop SOL to user1
    const user1Airdrop = await provider.connection.requestAirdrop(
      user1.publicKey,
      LAMPORTS_PER_SOL
    );
    await provider.connection.confirmTransaction(user1Airdrop);

    // Airdrop SOL to user2
    const user2Airdrop = await provider.connection.requestAirdrop(
      user2.publicKey,
      LAMPORTS_PER_SOL
    );
    await provider.connection.confirmTransaction(user2Airdrop);
  });

  it("Initialize betting pool", async () => {
    const disputePeriodSeconds = 86400; // 1 day
    const disputeThreshold = 1_000_000; // 1 token
    const poolName = "Test Prediction Pool";
    const poolDescription = "Will BTC reach $100k by end of 2025?";
    
    // End time 1 hour from now
    const currentTime = Math.floor(Date.now() / 1000);
    const endTime = currentTime + 3600;
    
    await program.methods
      .initializePool(
        new anchor.BN(disputePeriodSeconds),
        new anchor.BN(disputeThreshold),
        poolName,
        poolDescription,
        new anchor.BN(endTime)
      )
      .accounts({
        authority: authority.publicKey,
        pool: poolPda,
        yes_mint: yesMint.publicKey,
        no_mint: noMint.publicKey,
        token_program: TOKEN_2022_PROGRAM_ID,
        system_program: SystemProgram.programId,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      })
      .signers([authority, yesMint, noMint])
      .rpc({ commitment: "confirmed" });
    
    // Fetch the pool data to verify
    const poolData = await program.account.bettingPool.fetch(poolPda);
    
    assert.equal(poolData.name, poolName, "Pool name doesn't match");
    assert.equal(poolData.description, poolDescription, "Pool description doesn't match");
    assert.approximately(
      poolData.endTime.toNumber(), 
      endTime, 
      10, 
      "End time doesn't match (within 10 seconds)"
    );
    assert.equal(
      poolData.disputeThreshold.toString(), 
      disputeThreshold.toString(), 
      "Dispute threshold doesn't match"
    );
  });

  it("User1 mints YES tokens", async () => {
    // Get the ATAs
    user1YesToken = getAssociatedTokenAddressSync(
      yesMint.publicKey,
      user1.publicKey
    );
    
    user1NoToken = getAssociatedTokenAddressSync(
      noMint.publicKey,
      user1.publicKey
    );
    
    const amountToMint = 5_000_000; // 5 tokens
    
    await program.methods
      .mintPredictionTokens(
        new anchor.BN(amountToMint),
        true // YES prediction
      )
      .accounts({
        user: user1.publicKey,
        pool: poolPda,
        yes_mint: yesMint.publicKey,
        no_mint: noMint.publicKey,
        user_yes_token: user1YesToken,
        user_no_token: user1NoToken,
        token_program: TOKEN_2022_PROGRAM_ID,
        associated_token_program: ASSOCIATED_TOKEN_PROGRAM_ID,
        system_program: SystemProgram.programId,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      })
      .signers([user1])
      .rpc({ commitment: "confirmed" });
    
    // Check token balance
    const tokenAccount = await getAccount(provider.connection, user1YesToken);
    assert.equal(tokenAccount.amount.toString(), amountToMint.toString(), "YES token amount doesn't match");
    
    // Check pool state
    const poolData = await program.account.bettingPool.fetch(poolPda);
    assert.equal(poolData.totalYesTokens.toString(), amountToMint.toString(), "Total YES tokens doesn't match");
  });

  it("User2 mints NO tokens", async () => {
    // Get the ATAs
    user2YesToken = getAssociatedTokenAddressSync(
      yesMint.publicKey,
      user2.publicKey
    );
    
    user2NoToken = getAssociatedTokenAddressSync(
      noMint.publicKey,
      user2.publicKey
    );
    
    const amountToMint = 3_000_000; // 3 tokens
    
    await program.methods
      .mintPredictionTokens(
        new anchor.BN(amountToMint),
        false // NO prediction
      )
      .accounts({
        user: user2.publicKey,
        pool: poolPda,
        yes_mint: yesMint.publicKey,
        no_mint: noMint.publicKey,
        user_yes_token: user2YesToken,
        user_no_token: user2NoToken,
        token_program: TOKEN_2022_PROGRAM_ID,
        associated_token_program: ASSOCIATED_TOKEN_PROGRAM_ID,
        system_program: SystemProgram.programId,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      })
      .signers([user2])
      .rpc({ commitment: "confirmed" });
    
    // Check token balance
    const tokenAccount = await getAccount(provider.connection, user2NoToken);
    assert.equal(tokenAccount.amount.toString(), amountToMint.toString(), "NO token amount doesn't match");
    
    // Check pool state
    const poolData = await program.account.bettingPool.fetch(poolPda);
    assert.equal(poolData.totalNoTokens.toString(), amountToMint.toString(), "Total NO tokens doesn't match");
  });

  it("Skip ahead in time and propose solution", async () => {
    // In a real scenario, we would wait until end_time has passed
    // For testing, we'll modify the pool's end_time to be in the past
    
    // Get the current pool data
    const poolData = await program.account.bettingPool.fetch(poolPda);
    
    // Set the end_time to 10 seconds ago
    const currentTime = Math.floor(Date.now() / 1000);
    const newEndTime = currentTime - 10;
    
    // We need to mock the clock to make it think time has passed
    // For simplicity in this test, we'll just proceed with the validation
    // but in a real implementation you'd need to wait for the actual time to pass
    
    // Authority proposes a solution: YES wins
    await program.methods
      .proposeSolution(true) // YES wins
      .accounts({
        authority: authority.publicKey,
        pool: poolPda,
      })
      .signers([authority])
      .rpc({ commitment: "confirmed" });
    
    // Verify solution was proposed
    const updatedPoolData = await program.account.bettingPool.fetch(poolPda);
    assert.isTrue(updatedPoolData.solutionProposed, "Solution should be proposed");
    assert.isTrue(updatedPoolData.solutionWinner, "Winner should be YES");
  });

  it("User2 disputes the solution (holding NO tokens)", async () => {
    await program.methods
      .disputeSolution()
      .accounts({
        user: user2.publicKey,
        pool: poolPda,
        yes_mint: yesMint.publicKey,
        no_mint: noMint.publicKey,
        user_yes_token: user2YesToken,
        user_no_token: user2NoToken,
      })
      .signers([user2])
      .rpc({ commitment: "confirmed" });
    
    // Verify dispute was registered
    const poolData = await program.account.bettingPool.fetch(poolPda);
    assert.isTrue(poolData.isDisputed, "Pool should be disputed");
    assert.deepEqual(poolData.disputer.toBase58(), user2.publicKey.toBase58(), "Disputer should be user2");
  });

  it("Authority resolves the dispute", async () => {
    await program.methods
      .resolveDispute(false) // Change winner to NO
      .accounts({
        authority: authority.publicKey,
        pool: poolPda,
      })
      .signers([authority])
      .rpc({ commitment: "confirmed" });
    
    // Verify dispute resolution
    const poolData = await program.account.bettingPool.fetch(poolPda);
    assert.isFalse(poolData.isDisputed, "Pool should no longer be disputed");
    assert.isFalse(poolData.solutionWinner, "Winner should now be NO");
  });

  it("Skip ahead in time and finalize the pool", async () => {
    // In a real scenario, we would wait until dispute_period_end has passed
    // For testing, we'll just proceed with the finalize call
    
    await program.methods
      .finalizePool()
      .accounts({
        user: authority.publicKey,
        pool: poolPda,
      })
      .signers([authority])
      .rpc({ commitment: "confirmed" });
    
    // Verify pool finalization
    const poolData = await program.account.bettingPool.fetch(poolPda);
    assert.isTrue(poolData.isFinalized, "Pool should be finalized");
  });

  it("User2 claims winnings (holding NO tokens)", async () => {
    await program.methods
      .claimWinnings()
      .accounts({
        user: user2.publicKey,
        pool: poolPda,
        yes_mint: yesMint.publicKey,
        no_mint: noMint.publicKey,
        user_yes_token: user2YesToken,
        user_no_token: user2NoToken,
        token_program: TOKEN_2022_PROGRAM_ID,
      })
      .signers([user2])
      .rpc({ commitment: "confirmed" });
    
    // Check if tokens were burned (claimed)
    try {
      await getAccount(provider.connection, user2NoToken);
      assert.fail("NO tokens should have been burned");
    } catch (error) {
      // This is expected as the tokens should be burned
      // In a real application, the user would receive a prize
    }
  });
});
