use anchor_lang::prelude::*;
use crate::constants::*;

#[account]
pub struct Vault {
    pub authority: Pubkey,
    pub vault_bump: u8,
    pub stablecoin_mint: Pubkey,
    pub vault_shares_mint: Pubkey,
    pub total_deposits: u64,
    pub total_shares_minted: u64,
    pub management_fee: u16,
    pub performance_fee: u16,
    pub minimum_deposit: u64,
    pub maximum_total_deposit: u64,
    pub last_rebalance_timestamp: i64,
    pub last_compound_timestamp: i64,
    pub total_yield_earned: u64,
    pub current_allocation: [ProtocolAllocation; MAX_PROTOCOL_ADAPTERS],
    pub is_paused: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

impl Vault {
    pub const LEN: usize = 8 + 
        32 + // authority
        1 + // vault_bump  
        32 + // stablecoin_mint
        32 + // vault_shares_mint
        8 + // total_deposits
        8 + // total_shares_minted
        2 + // management_fee
        2 + // performance_fee
        8 + // minimum_deposit
        8 + // maximum_total_deposit
        8 + // last_rebalance_timestamp
        8 + // last_compound_timestamp
        8 + // total_yield_earned
        (48 * MAX_PROTOCOL_ADAPTERS) + // current_allocation
        1 + // is_paused
        8 + // created_at
        8; // updated_at

    pub fn calculate_share_price(&self) -> Result<u64> {
        if self.total_shares_minted == 0 {
            return Ok(10_u64.pow(USDC_DECIMALS as u32));
        }
        
        self.total_deposits
            .checked_mul(10_u64.pow(USDC_DECIMALS as u32))
            .and_then(|result| result.checked_div(self.total_shares_minted))
            .ok_or(crate::error::VaultError::MathOverflow.into())
    }

    pub fn calculate_shares_to_mint(&self, deposit_amount: u64) -> Result<u64> {
        if self.total_shares_minted == 0 {
            return Ok(deposit_amount);
        }

        deposit_amount
            .checked_mul(self.total_shares_minted)
            .and_then(|result| result.checked_div(self.total_deposits))
            .ok_or(crate::error::VaultError::MathOverflow.into())
    }

    pub fn calculate_withdrawal_amount(&self, shares_amount: u64) -> Result<u64> {
        shares_amount
            .checked_mul(self.total_deposits)
            .and_then(|result| result.checked_div(self.total_shares_minted))
            .ok_or(crate::error::VaultError::MathOverflow.into())
    }

    pub fn can_rebalance(&self, current_timestamp: i64) -> bool {
        current_timestamp - self.last_rebalance_timestamp >= REBALANCING_COOLDOWN
    }

    pub fn update_allocations(&mut self, new_allocations: Vec<ProtocolAllocation>) {
        for (i, allocation) in new_allocations.iter().enumerate() {
            if i < MAX_PROTOCOL_ADAPTERS {
                self.current_allocation[i] = *allocation;
            }
        }
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug)]
pub struct ProtocolAllocation {
    pub protocol_adapter: Pubkey,
    pub allocated_amount: u64,
    pub allocation_percentage: u8,
}