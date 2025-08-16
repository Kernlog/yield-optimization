use anchor_lang::prelude::*;
use crate::{constants::*, error::VaultError, state::*};

#[derive(Accounts)]
pub struct CompoundRewards<'info> {
    #[account(
        mut,
        seeds = [VAULT_SEED, vault.stablecoin_mint.as_ref()],
        bump = vault.vault_bump,
        constraint = vault.authority == authority.key() @ VaultError::Unauthorized
    )]
    pub vault: Account<'info, Vault>,

    pub authority: Signer<'info>,
}

pub fn handler(ctx: Context<CompoundRewards>) -> Result<()> {
    let vault = &mut ctx.accounts.vault;
    let clock = Clock::get()?;

    require!(!vault.is_paused, VaultError::VaultPaused);

    // In a real implementation, this would:
    // 1. Claim rewards from all protocol adapters
    // 2. Convert rewards to stablecoins
    // 3. Reinvest the converted stablecoins
    // 4. Update total deposits and yield earned

    vault.last_compound_timestamp = clock.unix_timestamp;
    vault.updated_at = clock.unix_timestamp;

    msg!("Rewards compounded");

    Ok(())
}