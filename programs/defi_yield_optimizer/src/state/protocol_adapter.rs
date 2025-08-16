use anchor_lang::prelude::*;

#[account]
pub struct ProtocolAdapter {
    pub vault: Pubkey,
    pub protocol_program_id: Pubkey,
    pub protocol_type: ProtocolType,
    pub current_apy: u32,
    pub available_liquidity: u64,
    pub deposited_amount: u64,
    pub last_update_timestamp: i64,
    pub max_allocation_percentage: u8,
    pub is_active: bool,
    pub protocol_specific_data: [u8; 64],
    pub created_at: i64,
    pub updated_at: i64,
}

impl ProtocolAdapter {
    pub const LEN: usize = 8 +
        32 + // vault
        32 + // protocol_program_id
        1 + // protocol_type
        4 + // current_apy
        8 + // available_liquidity
        8 + // deposited_amount
        8 + // last_update_timestamp
        1 + // max_allocation_percentage
        1 + // is_active
        64 + // protocol_specific_data
        8 + // created_at
        8; // updated_at

    pub fn update_yield_data(&mut self, apy: u32, liquidity: u64, timestamp: i64) {
        self.current_apy = apy;
        self.available_liquidity = liquidity;
        self.last_update_timestamp = timestamp;
        self.updated_at = timestamp;
    }

    pub fn deposit(&mut self, amount: u64) -> Result<()> {
        self.deposited_amount = self.deposited_amount
            .checked_add(amount)
            .ok_or(crate::error::VaultError::MathOverflow)?;
        Ok(())
    }

    pub fn withdraw(&mut self, amount: u64) -> Result<()> {
        self.deposited_amount = self.deposited_amount
            .checked_sub(amount)
            .ok_or(crate::error::VaultError::MathOverflow)?;
        Ok(())
    }

    pub fn get_effective_apy(&self) -> u32 {
        if !self.is_active || self.available_liquidity == 0 {
            return 0;
        }
        self.current_apy
    }

    pub fn can_deposit(&self, amount: u64) -> bool {
        self.is_active && self.available_liquidity >= amount
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq)]
pub enum ProtocolType {
    Kamino,
    Drift,
    Meteora,
    Marinade,
    Jito,
    Sanctum,
    Other,
}