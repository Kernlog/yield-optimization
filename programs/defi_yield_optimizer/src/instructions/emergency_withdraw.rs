use anchor_lang::prelude::*;
use crate::{constants::*, error::VaultError, state::*};

#[derive(Accounts)]
pub struct EmergencyWithdraw<'info> {
    #[account(
        mut,
        seeds = [VAULT_SEED, vault.stablecoin_mint.as_ref()],
        bump = vault.vault_bump,
        constraint = vault.authority == authority.key() @ VaultError::EmergencyWithdrawUnauthorized
    )]
    pub vault: Account<'info, Vault>,

    pub authority: Signer<'info>,
}

pub fn handler(ctx: Context<EmergencyWithdraw>) -> Result<()> {
    let vault = &mut ctx.accounts.vault;
    let clock = Clock::get()?;

    // Pause the vault to prevent further deposits
    vault.is_paused = true;
    vault.updated_at = clock.unix_timestamp;

    // In a real implementation, this would:
    // 1. Withdraw all funds from all protocol adapters
    // 2. Return funds to the vault token account
    // 3. Allow users to withdraw their proportional share

    msg!("Emergency withdrawal initiated");

    Ok(())
}