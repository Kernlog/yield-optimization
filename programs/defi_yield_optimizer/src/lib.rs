use anchor_lang::prelude::*;

pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use instructions::*;

declare_id!("DGqtQj1izTNEooEmZVwjMXtbuwfWex3HmZVkHHXeyYPF");

#[program]
pub mod defi_yield_optimizer {
    use super::*;

    pub fn initialize_vault(
        ctx: Context<InitializeVault>,
        vault_bump: u8,
        management_fee: u16,
        performance_fee: u16,
        minimum_deposit: u64,
        maximum_total_deposit: u64,
    ) -> Result<()> {
        instructions::initialize_vault::handler(
            ctx,
            vault_bump,
            management_fee,
            performance_fee,
            minimum_deposit,
            maximum_total_deposit,
        )
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        instructions::deposit::handler(ctx, amount)
    }

    pub fn withdraw(ctx: Context<Withdraw>, shares_amount: u64) -> Result<()> {
        instructions::withdraw::handler(ctx, shares_amount)
    }

    pub fn initialize_protocol_adapter(
        ctx: Context<InitializeProtocolAdapter>,
        protocol_type: u8,
        max_allocation_percentage: u8,
    ) -> Result<()> {
        instructions::initialize_protocol_adapter::handler(
            ctx,
            protocol_type,
            max_allocation_percentage,
        )
    }

    pub fn update_yield_data(
        ctx: Context<UpdateYieldData>,
        current_apy: u32,
        available_liquidity: u64,
    ) -> Result<()> {
        instructions::update_yield_data::handler(ctx, current_apy, available_liquidity)
    }

    pub fn rebalance(ctx: Context<Rebalance>) -> Result<()> {
        instructions::rebalance::handler(ctx)
    }

    pub fn compound_rewards(ctx: Context<CompoundRewards>) -> Result<()> {
        instructions::compound_rewards::handler(ctx)
    }

    pub fn emergency_withdraw(ctx: Context<EmergencyWithdraw>) -> Result<()> {
        instructions::emergency_withdraw::handler(ctx)
    }

    pub fn update_vault_config(
        ctx: Context<UpdateVaultConfig>,
        new_management_fee: Option<u16>,
        new_performance_fee: Option<u16>,
        new_minimum_deposit: Option<u64>,
        new_maximum_total_deposit: Option<u64>,
    ) -> Result<()> {
        instructions::update_vault_config::handler(
            ctx,
            new_management_fee,
            new_performance_fee,
            new_minimum_deposit,
            new_maximum_total_deposit,
        )
    }
}