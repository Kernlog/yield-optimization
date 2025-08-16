use anchor_lang::prelude::*;
use anchor_spl::{
    token_interface::{Mint, TokenAccount, TokenInterface, burn, transfer, Burn, Transfer},
};
use crate::{constants::*, error::VaultError, state::*};

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(
        mut,
        seeds = [VAULT_SEED, vault.stablecoin_mint.as_ref()],
        bump = vault.vault_bump
    )]
    pub vault: Account<'info, Vault>,

    #[account(
        mut,
        seeds = [USER_ACCOUNT_SEED, withdrawer.key().as_ref(), vault.key().as_ref()],
        bump
    )]
    pub user_account: Account<'info, UserAccount>,

    #[account(
        mut,
        constraint = vault_shares_mint.key() == vault.vault_shares_mint
    )]
    pub vault_shares_mint: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        constraint = vault_token_account.owner == vault_authority.key()
    )]
    pub vault_token_account: InterfaceAccount<'info, TokenAccount>,

    /// CHECK: Vault authority PDA
    #[account(
        seeds = [VAULT_AUTHORITY_SEED, vault.key().as_ref()],
        bump
    )]
    pub vault_authority: AccountInfo<'info>,

    #[account(
        mut,
        constraint = withdrawer_token_account.owner == withdrawer.key(),
        constraint = withdrawer_token_account.mint == vault.stablecoin_mint
    )]
    pub withdrawer_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        constraint = withdrawer_shares_account.owner == withdrawer.key(),
        constraint = withdrawer_shares_account.mint == vault.vault_shares_mint
    )]
    pub withdrawer_shares_account: InterfaceAccount<'info, TokenAccount>,

    #[account(mut)]
    pub withdrawer: Signer<'info>,

    pub token_program: Interface<'info, TokenInterface>,
}

pub fn handler(ctx: Context<Withdraw>, shares_amount: u64) -> Result<()> {
    let vault = &mut ctx.accounts.vault;
    let user_account = &mut ctx.accounts.user_account;
    let clock = Clock::get()?;

    require!(!vault.is_paused, VaultError::VaultPaused);
    require!(shares_amount > 0, VaultError::InvalidWithdrawalAmount);
    require!(
        user_account.shares_owned >= shares_amount,
        VaultError::InsufficientShares
    );

    let withdrawal_amount = vault.calculate_withdrawal_amount(shares_amount)?;

    burn(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Burn {
                mint: ctx.accounts.vault_shares_mint.to_account_info(),
                from: ctx.accounts.withdrawer_shares_account.to_account_info(),
                authority: ctx.accounts.withdrawer.to_account_info(),
            },
        ),
        shares_amount,
    )?;

    let vault_key = vault.key();
    let seeds = &[
        VAULT_AUTHORITY_SEED,
        vault_key.as_ref(),
        &[ctx.bumps.vault_authority],
    ];
    let signer = &[&seeds[..]];

    transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.vault_token_account.to_account_info(),
                to: ctx.accounts.withdrawer_token_account.to_account_info(),
                authority: ctx.accounts.vault_authority.to_account_info(),
            },
            signer,
        ),
        withdrawal_amount,
    )?;

    vault.total_deposits = vault.total_deposits
        .checked_sub(withdrawal_amount)
        .ok_or(VaultError::MathOverflow)?;
    vault.total_shares_minted = vault.total_shares_minted
        .checked_sub(shares_amount)
        .ok_or(VaultError::MathOverflow)?;
    vault.updated_at = clock.unix_timestamp;

    user_account.update_withdrawal(shares_amount, withdrawal_amount, clock.unix_timestamp);

    msg!("Withdraw: {} shares, {} tokens", shares_amount, withdrawal_amount);

    Ok(())
}