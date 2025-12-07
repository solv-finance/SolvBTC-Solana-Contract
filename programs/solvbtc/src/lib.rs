#![allow(deprecated)]
use anchor_lang::prelude::*;

pub mod constants;
pub mod contexts;
pub mod errors;
pub mod events;
pub mod state;
pub mod helpers;

use contexts::*;

declare_id!("soLv1S6GsAEVEnXmVY3oz6GtrNJteQ28iTyRQrHXvkz");

#[program]
pub mod solvbtc {
    use super::*;

    // SolvBTC Vault Instructions
    //
    // These instructions enable the creation of Solv vaults by the contract admin.
    // These vaults are used to accept user deposits and handle withdrawals.
    #[instruction(discriminator = 0)]
    #[doc = "# Deposit\nEnable user to deposit accepted deposit tokens to a Solv vault and mint target token in return based upon pro-rata share of NAV."]
    pub fn vault_deposit(
        ctx: Context<VaultDeposit>,
        amount: u64,
        min_amount_out: u64,
    ) -> Result<()> {
        ctx.accounts.deposit_tokens(amount)?;
        ctx.accounts.mint_target_tokens(amount, min_amount_out)
    }

    // Check that request hash has not been requested or withdrawn. Apply a second hash as the key instead of using the request hash directly, for security purposes.
    // The second hash includes (user, withdraw token address, request hash, shares, NAV)
    // Record the second hash (hash, status)
    // User transfer target token to vault, vault burn the token
    // emit event  WithdrawRequest(user, withdraw token, shares  token address, shares, request hash, current NAV)
    #[instruction(discriminator = 1)]
    #[doc = "# Withdraw Request\nEnable user to request a withdrawal in a certain currency and record it onchain."]
    pub fn vault_withdraw_request(
        ctx: Context<VaultRequestWithdraw>,
        request_hash: [u8; 32],
        amount: u64,
    ) -> Result<()> {
        ctx.accounts.burn_tokens(amount)?;
        ctx.accounts
            .open_request_account(request_hash, amount)
    }

    #[instruction(discriminator = 2)]
    #[doc = "# Withdraw\nEnable user to process a withdrawal with a signed withdraw request."]
    pub fn vault_withdraw(
        ctx: Context<VaultWithdraw>,
        _hash: [u8; 32],
        signature: [u8; 64],
    ) -> Result<()> {
        ctx.accounts.withdraw_tokens(signature)?;
        ctx.accounts.close_request_account()
    }

    #[instruction(discriminator = 3)]
    #[doc = "# Initialize Solv Vault\nEnable admin to initialize a Solv vault."]
    pub fn vault_initialize(
        ctx: Context<VaultInitialize>,
        admin: Pubkey,
        fee_receiver: Pubkey,
        treasurer: Pubkey,
        verifier: [u8; 64],
        oracle_manager: Pubkey,
        nav: u64,
        withdraw_fee: u16,
    ) -> Result<()> {
        ctx.accounts.initialize(
            admin,
            fee_receiver,
            treasurer,
            verifier,
            oracle_manager,
            nav,
            withdraw_fee,
            ctx.bumps.vault,
        )
    }

    #[instruction(discriminator = 4)]
    #[doc = "# Transfer Vault Admin\nEnable admin to transfer vault admin role to a new address."]
    pub fn vault_transfer_admin(ctx: Context<VaultUpdate>, admin: Pubkey) -> Result<()> {
        ctx.accounts.transfer_admin(admin)
    }

    #[instruction(discriminator = 5)]
    #[doc = "# Add Vault Currency\nEnable admin to add deposit currency to vault."]
    pub fn vault_add_currency(ctx: Context<VaultUpdate>, currency: Pubkey, deposit_fee: u16) -> Result<()> {
        ctx.accounts.add_currency(currency, deposit_fee)
    }

    #[instruction(discriminator = 6)]
    #[doc = "# Remove Vault Currency\nEnable admin to remove deposit currency from vault."]
    pub fn vault_remove_currency(ctx: Context<VaultUpdate>, currency: Pubkey) -> Result<()> {
        ctx.accounts.remove_currency(currency)
    }

    #[instruction(discriminator = 7)]
    #[doc = "# Set Vault Deposit Fee\nEnable admin to set vault deposit fee in basis points."]
    pub fn vault_set_deposit_fee(ctx: Context<VaultUpdate>, currency: Pubkey, deposit_fee: u16) -> Result<()> {
        ctx.accounts.set_deposit_fee(currency, deposit_fee)
    }

    #[instruction(discriminator = 8)]
    #[doc = "# Set Vault Withdraw Fee\nEnable admin to set vault withdraw fee in basis points."]
    pub fn vault_set_withdraw_fee(ctx: Context<VaultUpdate>, withdraw_fee: u16) -> Result<()> {
        ctx.accounts.set_withdraw_fee(withdraw_fee)
    }

    #[instruction(discriminator = 9)]
    #[doc = "# Set Vault Fee Receiver\nEnable admin to set vault withdraw fee receiver."]
    pub fn vault_set_fee_receiver(ctx: Context<VaultUpdate>, receiver: Pubkey) -> Result<()> {
        ctx.accounts.set_fee_receiver(receiver)
    }

    #[instruction(discriminator = 10)]
    #[doc = "# Set Vault Verifier\nEnable admin to set vault withdrawal verifier."]
    pub fn vault_set_verifier(ctx: Context<VaultUpdate>, verifier: [u8; 64]) -> Result<()> {
        ctx.accounts.set_verifier(verifier)
    }

    #[instruction(discriminator = 11)]
    #[doc = "# Set Vault Treasurer\nEnable admin to set vault treasurer."]
    pub fn vault_set_treasurer(ctx: Context<VaultUpdate>, treasurer: Pubkey) -> Result<()> {
        ctx.accounts.set_treasurer(treasurer)
    }

    #[instruction(discriminator = 12)]
    #[doc = "# Set Oracle NAV\nEnable admin to update the NAV of a whitelisted token in the contract."]
    pub fn vault_set_nav(ctx: Context<VaultOracleUpdate>, nav: u64) -> Result<()> {
        ctx.accounts.set_nav(nav)
    }

    #[instruction(discriminator = 13)]
    #[doc = "# Set Oracle NAV\nEnable admin to update the manager of an oracle."]
    pub fn vault_set_nav_manager(ctx: Context<VaultOracleUpdate>, manager: Pubkey) -> Result<()> {
        ctx.accounts.set_manager(manager)
    }

    // SolvBTC Minter Manager Instructions
    //
    // These instructions enable the admin to mint whitelisted assets like SolvBTC, xSolvBTC, etc.
    // as well as control the whitelist of addresses that have permission to mint.
    #[instruction(discriminator = 14)]
    #[doc = "# Initialize Mint Manager\nEnable minter to mint a whitelisted token from the contract."]
    pub fn minter_manager_initialize(
        ctx: Context<MinterManagerInitialize>,
        admin: Pubkey,
    ) -> Result<()> {
        ctx.accounts.initialize(admin, ctx.bumps.minter_manager)
    }

    #[instruction(discriminator = 15)]
    #[doc = "# Add Minter By Admin\nEnable admin to add an address to the mint authority list."]
    pub fn minter_manager_add_minter_by_admin(
        ctx: Context<MinterManagerUpdateMinter>,
        minter: Pubkey,
    ) -> Result<()> {
        ctx.accounts.add_minter(minter)
    }

    #[instruction(discriminator = 16)]
    #[doc = "# Remove Minter By Admin\nEnable admin to remove an address from the mint authority list."]
    pub fn minter_manager_remove_minter_by_admin(
        ctx: Context<MinterManagerUpdateMinter>,
        minter: Pubkey,
    ) -> Result<()> {
        ctx.accounts.remove_minter(minter)
    }

    #[instruction(discriminator = 17)]
    #[doc = "# Mint\nEnable minter to mint a whitelisted token from the contract."]
    pub fn minter_manager_mint_to(ctx: Context<MinterManagerMint>, amount: u64) -> Result<()> {
        ctx.accounts.mint(amount, [ctx.bumps.vault])
    }

    #[instruction(discriminator = 18)]
    #[doc = "# Transfer Minter Manager Admin\nEnable admin to transfer admin priveleges to a new address."]
    pub fn minter_manager_transfer_admin(
        ctx: Context<MinterManagerTransferAdmin>,
        admin: Pubkey,
    ) -> Result<()> {
        ctx.accounts.transfer_admin(admin)
    }
}
