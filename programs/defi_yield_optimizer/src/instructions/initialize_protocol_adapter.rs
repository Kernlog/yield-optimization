use anchor_lang::prelude::*;
use crate::{constants::*, error::VaultError, state::*};

#[derive(Accounts)]
pub struct InitializeProtocolAdapter<'info> {
    #[account(
        mut,
        seeds = [VAULT_SEED, vault.stablecoin_mint.as_ref()],
        bump = vault.vault_bump,
        constraint = vault.authority == authority.key() @ VaultError::Unauthorized
    )]
    pub vault: Account<'info, Vault>,

    #[account(
        init,
        payer = authority,
        space = ProtocolAdapter::LEN,
        seeds = [PROTOCOL_ADAPTER_SEED, vault.key().as_ref(), protocol_program_id.key().as_ref()],
        bump
    )]
    pub protocol_adapter: Account<'info, ProtocolAdapter>,

    /// CHECK: External protocol program ID
    pub protocol_program_id: AccountInfo<'info>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<InitializeProtocolAdapter>,
    protocol_type: u8,
    max_allocation_percentage: u8,
) -> Result<()> {
    require!(
        max_allocation_percentage <= MAX_REBALANCING_PERCENTAGE,
        VaultError::AllocationExceedsMaximum
    );

    let protocol_type = match protocol_type {
        0 => ProtocolType::Kamino,
        1 => ProtocolType::Drift,
        2 => ProtocolType::Meteora,
        3 => ProtocolType::Marinade,
        4 => ProtocolType::Jito,
        5 => ProtocolType::Sanctum,
        _ => ProtocolType::Other,
    };

    let protocol_adapter = &mut ctx.accounts.protocol_adapter;
    let clock = Clock::get()?;

    protocol_adapter.vault = ctx.accounts.vault.key();
    protocol_adapter.protocol_program_id = ctx.accounts.protocol_program_id.key();
    protocol_adapter.protocol_type = protocol_type;
    protocol_adapter.current_apy = 0;
    protocol_adapter.available_liquidity = 0;
    protocol_adapter.deposited_amount = 0;
    protocol_adapter.last_update_timestamp = clock.unix_timestamp;
    protocol_adapter.max_allocation_percentage = max_allocation_percentage;
    protocol_adapter.is_active = true;
    protocol_adapter.protocol_specific_data = [0u8; 64];
    protocol_adapter.created_at = clock.unix_timestamp;
    protocol_adapter.updated_at = clock.unix_timestamp;

    msg!("Protocol adapter initialized: {:?}", protocol_type);

    Ok(())
}