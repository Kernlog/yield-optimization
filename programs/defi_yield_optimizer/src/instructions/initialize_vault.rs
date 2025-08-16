use anchor_lang::prelude::*;
use anchor_spl::{
    token_interface::{Mint, TokenAccount, TokenInterface},
};
use crate::{constants::*, error::VaultError, state::*};

#[derive(Accounts)]
#[instruction(vault_bump: u8)]
pub struct InitializeVault<'info> {
    #[account(
        init,
        payer = authority,
        space = Vault::LEN,
        seeds = [VAULT_SEED, stablecoin_mint.key().as_ref()],
        bump
    )]
    pub vault: Account<'info, Vault>,

    #[account(
        init,
        payer = authority,
        seeds = [VAULT_SHARES_SEED, vault.key().as_ref()],
        bump,
        mint::decimals = USDC_DECIMALS,
        mint::authority = vault_authority,
        mint::freeze_authority = vault_authority,
    )]
    pub vault_shares_mint: InterfaceAccount<'info, Mint>,

    /// CHECK: Vault authority PDA
    #[account(
        init,
        payer = authority,
        seeds = [VAULT_AUTHORITY_SEED, vault.key().as_ref()],
        bump,
        space = 8
    )]
    pub vault_authority: AccountInfo<'info>,

    #[account(
        init,
        payer = authority,
        token::mint = stablecoin_mint,
        token::authority = vault_authority,
    )]
    pub vault_token_account: InterfaceAccount<'info, TokenAccount>,

    pub stablecoin_mint: InterfaceAccount<'info, Mint>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn handler(
    ctx: Context<InitializeVault>,
    vault_bump: u8,
    management_fee: u16,
    performance_fee: u16,
    minimum_deposit: u64,
    maximum_total_deposit: u64,
) -> Result<()> {
    require!(
        management_fee <= MAX_MANAGEMENT_FEE,
        VaultError::InvalidFeeConfiguration
    );
    require!(
        performance_fee <= MAX_PERFORMANCE_FEE,
        VaultError::InvalidFeeConfiguration
    );

    let vault = &mut ctx.accounts.vault;
    let clock = Clock::get()?;

    vault.authority = ctx.accounts.authority.key();
    vault.vault_bump = vault_bump;
    vault.stablecoin_mint = ctx.accounts.stablecoin_mint.key();
    vault.vault_shares_mint = ctx.accounts.vault_shares_mint.key();
    vault.total_deposits = 0;
    vault.total_shares_minted = 0;
    vault.management_fee = management_fee;
    vault.performance_fee = performance_fee;
    vault.minimum_deposit = minimum_deposit;
    vault.maximum_total_deposit = maximum_total_deposit;
    vault.last_rebalance_timestamp = clock.unix_timestamp;
    vault.last_compound_timestamp = clock.unix_timestamp;
    vault.total_yield_earned = 0;
    vault.current_allocation = [ProtocolAllocation {
        protocol_adapter: Pubkey::default(),
        allocated_amount: 0,
        allocation_percentage: 0,
    }; MAX_PROTOCOL_ADAPTERS];
    vault.is_paused = false;
    vault.created_at = clock.unix_timestamp;
    vault.updated_at = clock.unix_timestamp;

    msg!("Vault initialized");

    Ok(())
}