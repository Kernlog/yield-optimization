import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { DefiYieldOptimizer } from "../target/types/defi_yield_optimizer";
import { 
  PublicKey, 
  SystemProgram, 
  Keypair,
  LAMPORTS_PER_SOL
} from "@solana/web3.js";
import {
  createMint,
  getAssociatedTokenAddress,
  createAssociatedTokenAccount,
  mintTo,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import { assert } from "chai";

describe("defi_yield_optimizer", () => {
  // Configure the client to use the local cluster
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.DefiYieldOptimizer as Program<DefiYieldOptimizer>;
  
  // Test accounts
  let authority: Keypair;
  let user: Keypair;
  let stablecoinMint: PublicKey;
  let vault: PublicKey;
  let vaultBump: number;
  let vaultSharesMint: PublicKey;
  let vaultAuthority: PublicKey;
  let vaultTokenAccount: PublicKey;
  let userTokenAccount: PublicKey;
  let userSharesAccount: PublicKey;
  let userAccount: PublicKey;

  // Constants for testing
  const MANAGEMENT_FEE = 50; // 0.5%
  const PERFORMANCE_FEE = 1000; // 10%
  const MINIMUM_DEPOSIT = new anchor.BN(1000000); // 1 USDC (6 decimals)
  const MAXIMUM_TOTAL_DEPOSIT = new anchor.BN(1000000000000); // 1M USDC

  before(async () => {
    // Setup test accounts
    authority = Keypair.generate();
    user = Keypair.generate();

    // Helper function to airdrop with retry logic
    const airdropWithRetry = async (publicKey: PublicKey, amount: number, retries = 3) => {
      for (let i = 0; i < retries; i++) {
        try {
          const signature = await provider.connection.requestAirdrop(publicKey, amount);
          await provider.connection.confirmTransaction(signature);
          return;
        } catch (error) {
          console.log(`Airdrop attempt ${i + 1} failed, retrying...`);
          if (i === retries - 1) {
            console.log("Airdrop failed after all retries, continuing with existing balance");
          }
          await new Promise(resolve => setTimeout(resolve, 2000)); // Wait 2 seconds between retries
        }
      }
    };

    // Airdrop SOL to test accounts with smaller amounts to avoid rate limits
    await airdropWithRetry(authority.publicKey, 2 * LAMPORTS_PER_SOL);
    await new Promise(resolve => setTimeout(resolve, 1000)); // Wait between airdrops
    await airdropWithRetry(user.publicKey, 2 * LAMPORTS_PER_SOL);

    // Create mock USDC mint
    stablecoinMint = await createMint(
      provider.connection,
      authority,
      authority.publicKey,
      null,
      6 // USDC decimals
    );

    // Derive PDAs
    [vault, vaultBump] = PublicKey.findProgramAddressSync(
      [Buffer.from("vault"), stablecoinMint.toBuffer()],
      program.programId
    );

    [vaultSharesMint] = PublicKey.findProgramAddressSync(
      [Buffer.from("vault_shares"), vault.toBuffer()],
      program.programId
    );

    [vaultAuthority] = PublicKey.findProgramAddressSync(
      [Buffer.from("vault_authority"), vault.toBuffer()],
      program.programId
    );

    // Create vault token account
    vaultTokenAccount = await createAssociatedTokenAccount(
      provider.connection,
      authority,
      stablecoinMint,
      vaultAuthority,
      { allowOwnerOffCurve: true }
    );

    // Create user token account
    userTokenAccount = await createAssociatedTokenAccount(
      provider.connection,
      authority,
      stablecoinMint,
      user.publicKey
    );

    // Mint some USDC to user
    await mintTo(
      provider.connection,
      authority,
      stablecoinMint,
      userTokenAccount,
      authority,
      10000000000 // 10,000 USDC
    );

    // Derive user account PDA
    [userAccount] = PublicKey.findProgramAddressSync(
      [Buffer.from("user_account"), user.publicKey.toBuffer(), vault.toBuffer()],
      program.programId
    );
  });

  describe("Initialize Vault", () => {
    it("should initialize the vault successfully", async () => {
      const tx = await program.methods
        .initializeVault(
          vaultBump,
          MANAGEMENT_FEE,
          PERFORMANCE_FEE,
          MINIMUM_DEPOSIT,
          MAXIMUM_TOTAL_DEPOSIT
        )
        .accounts({
          vault,
          vaultSharesMint,
          vaultAuthority,
          vaultTokenAccount,
          stablecoinMint,
          authority: authority.publicKey,
          systemProgram: SystemProgram.programId,
          tokenProgram: TOKEN_PROGRAM_ID,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        })
        .signers([authority])
        .rpc();

      console.log("Initialize vault transaction:", tx);

      // Fetch and verify vault state
      const vaultAccount = await program.account.vault.fetch(vault);
      
      assert.equal(vaultAccount.authority.toString(), authority.publicKey.toString());
      assert.equal(vaultAccount.vaultBump, vaultBump);
      assert.equal(vaultAccount.stablecoinMint.toString(), stablecoinMint.toString());
      assert.equal(vaultAccount.managementFee, MANAGEMENT_FEE);
      assert.equal(vaultAccount.performanceFee, PERFORMANCE_FEE);
      assert.equal(vaultAccount.totalDeposits.toNumber(), 0);
      assert.equal(vaultAccount.totalSharesMinted.toNumber(), 0);
      assert.equal(vaultAccount.isPaused, false);
    });
  });

  describe("Deposit", () => {
    const depositAmount = new anchor.BN(5000000000); // 5,000 USDC

    before(async () => {
      // Get user's shares account
      userSharesAccount = await getAssociatedTokenAddress(
        vaultSharesMint,
        user.publicKey
      );
    });

    it("should allow users to deposit stablecoins", async () => {
      const tx = await program.methods
        .deposit(depositAmount)
        .accounts({
          vault,
          userAccount,
          vaultSharesMint,
          vaultTokenAccount,
          vaultAuthority,
          depositorTokenAccount: userTokenAccount,
          depositorSharesAccount: userSharesAccount,
          depositor: user.publicKey,
          systemProgram: SystemProgram.programId,
          tokenProgram: TOKEN_PROGRAM_ID,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        })
        .signers([user])
        .rpc();

      console.log("Deposit transaction:", tx);

      // Verify vault state updated
      const vaultAccount = await program.account.vault.fetch(vault);
      assert.equal(vaultAccount.totalDeposits.toNumber(), depositAmount.toNumber());
      assert.equal(vaultAccount.totalSharesMinted.toNumber(), depositAmount.toNumber());

      // Verify user account created and updated
      const userAccountData = await program.account.userAccount.fetch(userAccount);
      assert.equal(userAccountData.owner.toString(), user.publicKey.toString());
      assert.equal(userAccountData.sharesOwned.toNumber(), depositAmount.toNumber());
      assert.equal(userAccountData.totalDeposited.toNumber(), depositAmount.toNumber());
    });

    it("should reject deposits below minimum", async () => {
      const smallAmount = new anchor.BN(100); // Way below minimum

      try {
        await program.methods
          .deposit(smallAmount)
          .accounts({
            vault,
            userAccount,
            vaultSharesMint,
            vaultTokenAccount,
            vaultAuthority,
            depositorTokenAccount: userTokenAccount,
            depositorSharesAccount: userSharesAccount,
            depositor: user.publicKey,
            systemProgram: SystemProgram.programId,
            tokenProgram: TOKEN_PROGRAM_ID,
            rent: anchor.web3.SYSVAR_RENT_PUBKEY,
          })
          .signers([user])
          .rpc();
        
        assert.fail("Should have rejected deposit below minimum");
      } catch (error) {
        assert.include(error.toString(), "DepositBelowMinimum");
      }
    });
  });

  describe("Withdraw", () => {
    const withdrawShares = new anchor.BN(2000000000); // 2,000 shares

    it("should allow users to withdraw their funds", async () => {
      const tx = await program.methods
        .withdraw(withdrawShares)
        .accounts({
          vault,
          userAccount,
          vaultSharesMint,
          vaultTokenAccount,
          vaultAuthority,
          withdrawerTokenAccount: userTokenAccount,
          withdrawerSharesAccount: userSharesAccount,
          withdrawer: user.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([user])
        .rpc();

      console.log("Withdraw transaction:", tx);

      // Verify vault state updated
      const vaultAccount = await program.account.vault.fetch(vault);
      assert.equal(
        vaultAccount.totalDeposits.toNumber(),
        3000000000 // 5,000 - 2,000 = 3,000 USDC
      );
      assert.equal(
        vaultAccount.totalSharesMinted.toNumber(),
        3000000000 // 5,000 - 2,000 = 3,000 shares
      );

      // Verify user account updated
      const userAccountData = await program.account.userAccount.fetch(userAccount);
      assert.equal(userAccountData.sharesOwned.toNumber(), 3000000000);
      assert.equal(userAccountData.totalWithdrawn.toNumber(), 2000000000);
    });
  });

  describe("Admin Functions", () => {
    it("should allow authority to update vault config", async () => {
      const newManagementFee = 100; // 1%
      const newPerformanceFee = 1500; // 15%

      const tx = await program.methods
        .updateVaultConfig(
          newManagementFee,
          newPerformanceFee,
          null,
          null
        )
        .accounts({
          vault,
          authority: authority.publicKey,
        })
        .signers([authority])
        .rpc();

      console.log("Update vault config transaction:", tx);

      // Verify config updated
      const vaultAccount = await program.account.vault.fetch(vault);
      assert.equal(vaultAccount.managementFee, newManagementFee);
      assert.equal(vaultAccount.performanceFee, newPerformanceFee);
    });

    it("should allow authority to trigger rebalancing", async () => {
      const tx = await program.methods
        .rebalance()
        .accounts({
          vault,
          authority: authority.publicKey,
        })
        .signers([authority])
        .rpc();

      console.log("Rebalance transaction:", tx);

      // Verify rebalance timestamp updated
      const vaultAccount = await program.account.vault.fetch(vault);
      assert.isAbove(vaultAccount.lastRebalanceTimestamp.toNumber(), 0);
    });
  });
});