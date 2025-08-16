use anchor_lang::prelude::*;
use crate::{constants::*, error::VaultError, state::*};

#[derive(Accounts)]
pub struct Rebalance<'info> {
    #[account(
        mut,
        seeds = [VAULT_SEED, vault.stablecoin_mint.as_ref()],
        bump = vault.vault_bump,
        constraint = vault.authority == authority.key() @ VaultError::Unauthorized
    )]
    pub vault: Account<'info, Vault>,

    pub authority: Signer<'info>,
}

pub fn handler(ctx: Context<Rebalance>) -> Result<()> {
    let vault = &mut ctx.accounts.vault;
    let clock = Clock::get()?;

    require!(
        vault.can_rebalance(clock.unix_timestamp),
        VaultError::RebalancingCooldownActive
    );

    require!(!vault.is_paused, VaultError::VaultPaused);

    // In a real implementation, this would:
    // 1. Query all protocol adapters for current APY
    // 2. Calculate optimal allocation based on yields and risk parameters
    // 3. Execute withdrawals from lower-yield protocols
    // 4. Execute deposits to higher-yield protocols
    // 5. Update vault allocation state

    // For now, we'll just update the rebalance timestamp
    vault.last_rebalance_timestamp = clock.unix_timestamp;
    vault.updated_at = clock.unix_timestamp;

    msg!("Rebalancing executed");

    Ok(())
}