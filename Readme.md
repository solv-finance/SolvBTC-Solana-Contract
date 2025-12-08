# SolvBTC Program

The SolvBTC Program is a Solana-based implementation that combines vault management, minter management, and oracle functionality into a single program. It enables users to deposit tokens, mint yield-bearing derivatives, and withdraw funds with cryptographic verification.

## Program Architecture

The program consists of three main components:

### 1. Vault Management
Handles liquidity deposits, withdrawals, and token minting/burning with NAV-based pricing.

### 2. Minter Manager  
Controls mint authority and whitelist management for authorized minters.

### 3. Oracle Integration
Provides NAV (Net Asset Value) updates with signature verification for accurate pricing.

## Instructions

### Vault Instructions

#### `vault_initialize`
Initialize a new vault with admin, fee settings, and oracle configuration.
- **Parameters**: `admin`, `fee_receiver`, `treasurer`, `verifier` (64-byte ECDSA public key), `oracle_manager`, `nav`, `fee` (basis points)

#### `vault_deposit`  
Deposit tokens to mint target tokens based on current NAV.
- **Parameters**: `amount`, `min_amount_out` (slippage protection)

#### `vault_withdraw_request`
Create a withdrawal request that burns target tokens and records withdrawal intent.
- **Parameters**: `request_hash` (32-byte unique identifier), `amount` (shares to burn)

#### `vault_withdraw`
Process a withdrawal request with cryptographic signature verification.
- **Parameters**: `hash` (request hash), `signature` (64-byte ECDSA signature)

### Administrative Instructions

#### `vault_transfer_admin`
Transfer vault admin privileges to a new address.

#### `vault_add_currency` / `vault_remove_currency`
Manage supported deposit currencies for the vault.

#### `vault_set_withdraw_fee`
Set withdrawal fee in basis points (e.g., 50 = 0.5%).

#### `vault_set_fee_receiver`
Update the address that receives withdrawal fees.

#### `vault_set_treasurer` 
Update the treasurer address for deposits.

#### `vault_set_verifier`
Update the ECDSA public key used for withdrawal signature verification.

### Oracle Instructions

#### `vault_set_nav`
Update the NAV (Net Asset Value) of the vault. Must be >= 1 Bitcoin (100,000,000 base units).

#### `vault_set_nav_manager`
Transfer oracle management privileges to a new address.

### Minter Manager Instructions

#### `minter_manager_initialize`
Initialize a minter manager for a specific vault.

#### `minter_manager_add_minter_by_admin` / `minter_manager_remove_minter_by_admin`
Add or remove addresses from the authorized minters list (max 10 minters).

#### `minter_manager_mint_to`
Mint tokens to a specified address (requires minter authorization).

#### `minter_manager_transfer_admin`
Transfer minter manager admin privileges.

## Key Features

### ECDSA Signature Verification
Withdrawals require cryptographic signatures from authorized verifiers, ensuring secure fund management.

### NAV-Based Pricing
All deposits and withdrawals are calculated based on the current Net Asset Value, providing fair pricing.

### Multi-Currency Support
Vaults can accept deposits in multiple supported currencies.

### Slippage Protection
Deposits include minimum output amount protection against price movements.

### Fee Management
Configurable withdrawal fees with dedicated fee receiver addresses.

### Account Rent Optimization
Withdrawal request accounts are resized and rent-refunded upon completion.

## Security Considerations

- All admin functions require proper authorization
- Withdrawal signatures are cryptographically verified
- NAV updates have minimum value validation (>= 1 Bitcoin)
- Account seeds prevent unauthorized access to user funds
- Slippage protection prevents sandwich attacks on deposits

## Testing

The test suite covers:
- Complete vault lifecycle (initialize → deposit → withdraw request → withdraw)
- Admin function authorization and transfers
- Error conditions and edge cases
- Multi-vault scenarios
- Fee calculations and transfers
- Oracle NAV management

Setup:

1) Install [Surfpool](https://surfpool.run/)
2) Install [Bun](https://bun.sh/)
3) Generate a new keypair and save it at `./whitelisted-admin-wallet.json`, no funding required
4) In `programs/solvbtc/src/constants.rs`, replace the address in `ADMIN_WHITELIST` with the address of `authority-keypair-wallet.json`
5) Sync program ID: `anchor keys sync`
6) Build program: `anchor build`

To run the tests:

1) In another terminal, start Surfpool: `bun run surfpool:start`
2) Run all tests: `bun test`
