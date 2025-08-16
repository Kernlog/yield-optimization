use anchor_lang::prelude::*;

#[error_code]
pub enum VaultError {
    #[msg("Deposit amount is below minimum threshold")]
    DepositBelowMinimum,
    
    #[msg("Vault has reached maximum deposit limit")]
    VaultCapacityReached,
    
    #[msg("Insufficient vault shares for withdrawal")]
    InsufficientShares,
    
    #[msg("Invalid fee configuration")]
    InvalidFeeConfiguration,
    
    #[msg("Protocol adapter already initialized")]
    AdapterAlreadyInitialized,
    
    #[msg("Maximum number of protocol adapters reached")]
    MaxAdaptersReached,
    
    #[msg("Invalid protocol type")]
    InvalidProtocolType,
    
    #[msg("Allocation percentage exceeds maximum allowed")]
    AllocationExceedsMaximum,
    
    #[msg("Insufficient liquidity for rebalancing")]
    InsufficientLiquidity,
    
    #[msg("Rebalancing cooldown period not met")]
    RebalancingCooldownActive,
    
    #[msg("No yield improvement found for rebalancing")]
    NoYieldImprovement,
    
    #[msg("Unauthorized access")]
    Unauthorized,
    
    #[msg("Math overflow error")]
    MathOverflow,
    
    #[msg("Invalid token mint")]
    InvalidTokenMint,
    
    #[msg("Vault is paused")]
    VaultPaused,
    
    #[msg("Emergency withdrawal only allowed by vault authority")]
    EmergencyWithdrawUnauthorized,
    
    #[msg("Invalid withdrawal amount")]
    InvalidWithdrawalAmount,
    
    #[msg("Protocol adapter not found")]
    AdapterNotFound,
    
    #[msg("Rebalancing failed")]
    RebalancingFailed,
}