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
  TOKEN_2022_PROGRAM_ID,
} from "@solana/spl-token";
import { assert } from "chai";

describe("defi_yield_optimizer - Devnet Tests", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.DefiYieldOptimizer as Program<DefiYieldOptimizer>;
  
  const authority = provider.wallet as anchor.Wallet;
  let stablecoinMint: PublicKey;
  let vault: PublicKey;
  let vaultBump: number;
  let vaultSharesMint: PublicKey;
  let vaultAuthority: PublicKey;
  let vaultTokenAccount: PublicKey;
  let vaultTokenKeypair: Keypair;

  const MANAGEMENT_FEE = 50;
  const PERFORMANCE_FEE = 1000;
  const MINIMUM_DEPOSIT = new anchor.BN(1000000);
  const MAXIMUM_TOTAL_DEPOSIT = new anchor.BN(1000000000000);

  before(async () => {
    const mintKeypair = Keypair.generate();
    stablecoinMint = await createMint(
      provider.connection,
      authority.payer,
      authority.publicKey,
      null,
      6,
      mintKeypair
    );

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

    vaultTokenKeypair = Keypair.generate();
    vaultTokenAccount = vaultTokenKeypair.publicKey;
  });

  describe("Initialize Vault", () => {
    it("should initialize the vault successfully", async () => {
      await program.methods
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
        .signers([vaultTokenKeypair])
        .rpc();

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

  describe("Protocol Adapter", () => {
    let protocolAdapter: PublicKey;
    const mockProtocolProgramId = Keypair.generate().publicKey;

    before(() => {
      [protocolAdapter] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("protocol_adapter"),
          vault.toBuffer(),
          mockProtocolProgramId.toBuffer()
        ],
        program.programId
      );
    });

    it("should initialize a protocol adapter", async () => {
      await program.methods
        .initializeProtocolAdapter(0, 30)
        .accounts({
          vault,
          protocolAdapter,
          protocolProgramId: mockProtocolProgramId,
          authority: authority.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .rpc();

      const adapterAccount = await program.account.protocolAdapter.fetch(protocolAdapter);
      assert.equal(adapterAccount.vault.toString(), vault.toString());
      assert.equal(adapterAccount.maxAllocationPercentage, 30);
      assert.equal(adapterAccount.isActive, true);
    });

    it("should update yield data for protocol adapter", async () => {
      const currentApy = 1200;
      const availableLiquidity = new anchor.BN(100000000000);

      await program.methods
        .updateYieldData(currentApy, availableLiquidity)
        .accounts({
          vault,
          protocolAdapter,
          authority: authority.publicKey,
        })
        .rpc();

      const adapterAccount = await program.account.protocolAdapter.fetch(protocolAdapter);
      assert.equal(adapterAccount.currentApy, currentApy);
      assert.equal(adapterAccount.availableLiquidity.toNumber(), availableLiquidity.toNumber());
    });
  });

  describe("Admin Functions", () => {
    it("should allow authority to update vault config", async () => {
      const newManagementFee = 100;
      const newPerformanceFee = 1500;

      await program.methods
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
        .rpc();

      const vaultAccount = await program.account.vault.fetch(vault);
      assert.equal(vaultAccount.managementFee, newManagementFee);
      assert.equal(vaultAccount.performanceFee, newPerformanceFee);
    });

    it("should enforce rebalancing cooldown", async () => {
      try {
        await program.methods
          .rebalance()
          .accounts({
            vault,
            authority: authority.publicKey,
          })
          .rpc();
        assert.fail("Should have failed due to cooldown");
      } catch (error) {
        assert.include(error.toString(), "RebalancingCooldownActive");
      }

      const vaultAccount = await program.account.vault.fetch(vault);
      assert.isAbove(vaultAccount.lastRebalanceTimestamp.toNumber(), 0);
    });

    it("should allow authority to compound rewards", async () => {
      await program.methods
        .compoundRewards()
        .accounts({
          vault,
          authority: authority.publicKey,
        })
        .rpc();

      const vaultAccount = await program.account.vault.fetch(vault);
      assert.isAbove(vaultAccount.lastCompoundTimestamp.toNumber(), 0);
    });
  });
});