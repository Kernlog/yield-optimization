# DeFi Yield Optimizer

A Solana program for automated yield farming that optimizes stablecoin deposits across multiple DeFi protocols.

## Overview

This program implements an automated vault system that manages user deposits across various DeFi protocols on Solana, continuously rebalancing to maximize yield while minimizing risk through diversification and algorithmic optimization.

## Architecture

### Core Components

- **Vault**: Main account storing configuration, total deposits, and protocol allocations
- **User Account**: Tracks individual user positions, deposit history, and share ownership
- **Protocol Adapter**: Manages integration with external DeFi protocols
- **Vault Shares**: SPL tokens representing proportional ownership in the vault

### Key Features

- Automated yield optimization across multiple protocols
- Proportional share-based ownership model
- Configurable management and performance fees
- Rebalancing cooldown periods for security
- Emergency withdrawal mechanisms
- Protocol-agnostic adapter system

## Getting Started

### Prerequisites

- Rust 1.70+
- Solana CLI 1.16+
- Anchor 0.31+
- Node.js 16+

### Installation

```bash
git clone git@github.com:Kernlog/yield-optimization.git
cd yield-optimization
npm install

# Create or copy your wallet file (for devnet testing)
cp ~/.config/solana/id.json ./wallet.json

anchor build
```


### Local Development

```bash
# Start local validator
solana-test-validator

# Deploy program
anchor deploy

# Run tests
anchor test
```

### Devnet Deployment

```bash
# Configure for devnet
solana config set --url devnet

# Ensure wallet.json is set up
cp ~/.config/solana/id.json ./wallet.json

# Deploy to devnet
anchor deploy --provider.cluster devnet

# Run devnet tests
npm run test:devnet
```

## Program Instructions

### Administrative

- `initialize_vault`: Creates a new vault with specified parameters
- `initialize_protocol_adapter`: Adds support for a new DeFi protocol
- `update_vault_config`: Modifies vault parameters (fees, limits)
- `update_yield_data`: Updates APY and liquidity data for protocols
- `rebalance`: Redistributes funds across protocols for optimal yield
- `compound_rewards`: Reinvests earned rewards
- `emergency_withdraw`: Pauses vault and enables emergency procedures

### User Operations

- `deposit`: Deposits stablecoins and receives proportional vault shares
- `withdraw`: Burns vault shares to withdraw proportional stablecoins

## Testing

The test suite covers:

- Vault initialization and configuration
- Protocol adapter management
- User deposit and withdrawal flows
- Administrative functions and access control
- Error conditions and edge cases

Run tests with:
```bash
# Local tests
anchor test

# Devnet tests
npm run test:devnet
```

## Configuration

### Fees

- Management Fee: Maximum 2% annually (200 basis points)
- Performance Fee: Maximum 20% of profits (2000 basis points)

### Limits

- Minimum deposit amounts configurable per vault
- Maximum total vault capacity limits
- Per-protocol allocation maximums (40% default)
- Rebalancing cooldown period (1 hour default)

## Protocol Integration

The system uses a modular adapter pattern for protocol integration. Each protocol adapter stores:

- Current APY and available liquidity
- Protocol-specific configuration data
- Allocation limits and status

Currently supported protocol types:
- Kamino
- Drift
- Meteora
- Marinade
- Jito
- Sanctum

## Development

### Project Structure

```
programs/defi_yield_optimizer/src/
├── lib.rs                    # Program entry point
├── constants.rs              # Program constants
├── error.rs                  # Error definitions
├── state/
│   ├── vault.rs             # Vault account structure
│   ├── user_account.rs      # User position tracking
│   └── protocol_adapter.rs  # Protocol integration
└── instructions/
    ├── initialize_vault.rs
    ├── deposit.rs
    ├── withdraw.rs
    ├── rebalance.rs
    └── ...
```

### Adding New Protocols

1. Add protocol type to `ProtocolType` enum
2. Implement protocol-specific integration logic
3. Update rebalancing algorithms
4. Add comprehensive tests

## License

This project is licensed under the Apache License.

## Deployment

**Devnet**: `DGqtQj1izTNEooEmZVwjMXtbuwfWex3HmZVkHHXeyYPF`

View on Solana Explorer: https://explorer.solana.com/address/DGqtQj1izTNEooEmZVwjMXtbuwfWex3HmZVkHHXeyYPF?cluster=devnet