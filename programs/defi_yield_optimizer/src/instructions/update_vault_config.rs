use anchor_lang::prelude::*;
use crate::{constants::*, error::VaultError, state::*};

#[derive(Accounts)]
pub struct UpdateVaultConfig<'info> {
    #[account(
        mut,
        seeds = [VAULT_SEED, vault.stablecoin_mint.as_ref()],
        bump = vault.vault_bump,
        constraint = vault.authority == authority.key() @ VaultError::Unauthorized
    )]
    pub vault: Account<'info, Vault>,

    pub authority: Signer<'info>,
}

pub fn handler(
    ctx: Context<UpdateVaultConfig>,
    new_management_fee: Option<u16>,
    new_performance_fee: Option<u16>,
    new_minimum_deposit: Option<u64>,
    new_maximum_total_deposit: Option<u64>,
) -> Result<()> {
    let vault = &mut ctx.accounts.vault;
    let clock = Clock::get()?;

    if let Some(fee) = new_management_fee {
        require!(fee <= MAX_MANAGEMENT_FEE, VaultError::InvalidFeeConfiguration);
        vault.management_fee = fee;
    }

    if let Some(fee) = new_performance_fee {
        require!(fee <= MAX_PERFORMANCE_FEE, VaultError::InvalidFeeConfiguration);
        vault.performance_fee = fee;
    }

    if let Some(min) = new_minimum_deposit {
        vault.minimum_deposit = min;
    }

    if let Some(max) = new_maximum_total_deposit {
        vault.maximum_total_deposit = max;
    }

    vault.updated_at = clock.unix_timestamp;

    msg!("Vault config updated");

    Ok(())
}