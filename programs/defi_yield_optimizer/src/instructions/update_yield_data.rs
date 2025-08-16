use anchor_lang::prelude::*;
use crate::{constants::*, error::VaultError, state::*};

#[derive(Accounts)]
pub struct UpdateYieldData<'info> {
    #[account(
        mut,
        seeds = [VAULT_SEED, vault.stablecoin_mint.as_ref()],
        bump = vault.vault_bump,
        constraint = vault.authority == authority.key() @ VaultError::Unauthorized
    )]
    pub vault: Account<'info, Vault>,

    #[account(
        mut,
        seeds = [PROTOCOL_ADAPTER_SEED, vault.key().as_ref(), protocol_adapter.protocol_program_id.as_ref()],
        bump,
        constraint = protocol_adapter.vault == vault.key()
    )]
    pub protocol_adapter: Account<'info, ProtocolAdapter>,

    pub authority: Signer<'info>,
}

pub fn handler(
    ctx: Context<UpdateYieldData>,
    current_apy: u32,
    available_liquidity: u64,
) -> Result<()> {
    let protocol_adapter = &mut ctx.accounts.protocol_adapter;
    let clock = Clock::get()?;

    protocol_adapter.update_yield_data(current_apy, available_liquidity, clock.unix_timestamp);

    msg!("Yield data updated: {} bps APY", current_apy);

    Ok(())
}