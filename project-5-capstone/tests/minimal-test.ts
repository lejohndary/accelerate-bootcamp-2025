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
} from '@solana/spl-token';
import { assert } from "chai";

describe("project-5-capstone-minimal", () => {
  anchor.setProvider(anchor.AnchorProvider.env());

  const provider = anchor.getProvider() as anchor.AnchorProvider;
  const program = anchor.workspace.Project5Capstone as Program<any>;
  
  const authority = Keypair.generate();
  const yesMint = Keypair.generate();
  const noMint = Keypair.generate();
  
  // PDA for the pool
  const [poolPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("pool"), authority.publicKey.toBuffer()],
    program.programId
  );
  
  it("Airdrop SOL to authority", async () => {
    const authorityAirdrop = await provider.connection.requestAirdrop(
      authority.publicKey,
      2 * LAMPORTS_PER_SOL
    );
    await provider.connection.confirmTransaction(authorityAirdrop);
    console.log(`Authority: ${authority.publicKey.toString()}`);
    console.log(`Yes Mint: ${yesMint.publicKey.toString()}`);
    console.log(`No Mint: ${noMint.publicKey.toString()}`);
    console.log(`Pool PDA: ${poolPda.toString()}`);
  });

  it("Initialize betting pool", async () => {
    const disputePeriodSeconds = 86400; // 1 day
    const disputeThreshold = 1_000_000; // 1 token
    const poolName = "Test Prediction Pool";
    const poolDescription = "Will BTC reach $100k by end of 2025?";
    
    // End time 1 hour from now
    const currentTime = Math.floor(Date.now() / 1000);
    const endTime = currentTime + 3600;
    
    try {
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
      console.log("Pool initialized successfully!");
      console.log(`Name: ${poolData.name}`);
      console.log(`Description: ${poolData.description}`);
    } catch (error) {
      console.error("Error initializing pool:", error);
      throw error;
    }
  });
}); 