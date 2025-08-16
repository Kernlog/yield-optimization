use anchor_lang::prelude::*;
use anchor_spl::{
    token_interface::{Mint, TokenAccount, TokenInterface, mint_to, transfer, MintTo, Transfer},
};
use crate::{constants::*, error::VaultError, state::*};

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(
        mut,
        seeds = [VAULT_SEED, vault.stablecoin_mint.as_ref()],
        bump = vault.vault_bump
    )]
    pub vault: Account<'info, Vault>,

    #[account(
        init_if_needed,
        payer = depositor,
        space = UserAccount::LEN,
        seeds = [USER_ACCOUNT_SEED, depositor.key().as_ref(), vault.key().as_ref()],
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
        constraint = depositor_token_account.owner == depositor.key(),
        constraint = depositor_token_account.mint == vault.stablecoin_mint
    )]
    pub depositor_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = depositor,
        token::mint = vault_shares_mint,
        token::authority = depositor,
    )]
    pub depositor_shares_account: InterfaceAccount<'info, TokenAccount>,

    #[account(mut)]
    pub depositor: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn handler(ctx: Context<Deposit>, amount: u64) -> Result<()> {
    let vault = &mut ctx.accounts.vault;
    let user_account = &mut ctx.accounts.user_account;
    let clock = Clock::get()?;

    require!(!vault.is_paused, VaultError::VaultPaused);
    require!(
        amount >= vault.minimum_deposit,
        VaultError::DepositBelowMinimum
    );
    require!(
        vault.total_deposits + amount <= vault.maximum_total_deposit,
        VaultError::VaultCapacityReached
    );

    let shares_to_mint = vault.calculate_shares_to_mint(amount)?;

    transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.depositor_token_account.to_account_info(),
                to: ctx.accounts.vault_token_account.to_account_info(),
                authority: ctx.accounts.depositor.to_account_info(),
            },
        ),
        amount,
    )?;

    let vault_key = vault.key();
    let seeds = &[
        VAULT_AUTHORITY_SEED,
        vault_key.as_ref(),
        &[ctx.bumps.vault_authority],
    ];
    let signer = &[&seeds[..]];

    mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.vault_shares_mint.to_account_info(),
                to: ctx.accounts.depositor_shares_account.to_account_info(),
                authority: ctx.accounts.vault_authority.to_account_info(),
            },
            signer,
        ),
        shares_to_mint,
    )?;

    vault.total_deposits = vault.total_deposits
        .checked_add(amount)
        .ok_or(VaultError::MathOverflow)?;
    vault.total_shares_minted = vault.total_shares_minted
        .checked_add(shares_to_mint)
        .ok_or(VaultError::MathOverflow)?;
    vault.updated_at = clock.unix_timestamp;

    if user_account.created_at == 0 {
        user_account.owner = ctx.accounts.depositor.key();
        user_account.vault = vault.key();
        user_account.created_at = clock.unix_timestamp;
    }
    user_account.update_deposit(shares_to_mint, amount, clock.unix_timestamp);

    msg!("Deposit: {} tokens, {} shares", amount, shares_to_mint);

    Ok(())
}