use anchor_lang::prelude::*;

#[account]
pub struct UserAccount {
    pub owner: Pubkey,
    pub vault: Pubkey,
    pub shares_owned: u64,
    pub total_deposited: u64,
    pub total_withdrawn: u64,
    pub last_deposit_timestamp: i64,
    pub last_withdrawal_timestamp: i64,
    pub realized_gains: i64,
    pub deposit_count: u32,
    pub withdrawal_count: u32,
    pub created_at: i64,
    pub updated_at: i64,
}

impl UserAccount {
    pub const LEN: usize = 8 +
        32 + // owner
        32 + // vault
        8 + // shares_owned
        8 + // total_deposited
        8 + // total_withdrawn
        8 + // last_deposit_timestamp
        8 + // last_withdrawal_timestamp
        8 + // realized_gains
        4 + // deposit_count
        4 + // withdrawal_count
        8 + // created_at
        8; // updated_at

    pub fn update_deposit(&mut self, shares: u64, amount: u64, timestamp: i64) {
        self.shares_owned = self.shares_owned.saturating_add(shares);
        self.total_deposited = self.total_deposited.saturating_add(amount);
        self.deposit_count = self.deposit_count.saturating_add(1);
        self.last_deposit_timestamp = timestamp;
        self.updated_at = timestamp;
    }

    pub fn update_withdrawal(&mut self, shares: u64, amount: u64, timestamp: i64) {
        self.shares_owned = self.shares_owned.saturating_sub(shares);
        self.total_withdrawn = self.total_withdrawn.saturating_add(amount);
        self.withdrawal_count = self.withdrawal_count.saturating_add(1);
        self.last_withdrawal_timestamp = timestamp;
        self.updated_at = timestamp;
        
        let net_amount = amount as i64 - (self.total_deposited as i64);
        if net_amount > 0 {
            self.realized_gains = self.realized_gains.saturating_add(net_amount);
        }
    }
}