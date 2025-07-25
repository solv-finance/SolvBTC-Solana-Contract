use crate::state::Vault;
use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;

#[derive(Accounts)]
pub struct VaultUpdate<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    /// Separate authority from payer to support multisig and PDA signers
    pub admin: Signer<'info>,
    pub mint: InterfaceAccount<'info, Mint>,
    #[account(
        mut,
        has_one = admin,
        seeds = [b"vault", mint.key().as_ref()],
        bump = vault.bump
    )]
    pub vault: Account<'info, Vault>,
    pub system_program: Program<'info, System>,
}

impl<'info> VaultUpdate<'info> {
    pub fn transfer_admin(&mut self, admin: Pubkey) -> Result<()> {
        self.vault.transfer_admin(admin)
    }

    pub fn set_deposit_fee(&mut self, deposit_fee: u16) -> Result<()> {
        self.vault.set_deposit_fee(deposit_fee)
    }

    pub fn set_withdraw_fee(&mut self, withdraw_fee: u16) -> Result<()> {
        self.vault.set_withdraw_fee(withdraw_fee)
    }

    pub fn set_fee_receiver(&mut self, receiver: Pubkey) -> Result<()> {
        self.vault.set_fee_receiver(receiver)
    }

    pub fn set_verifier(&mut self, verifier: [u8; 64]) -> Result<()> {
        self.vault.set_verifier(verifier)
    }

    pub fn set_treasurer(&mut self, treasurer: Pubkey) -> Result<()> {
        self.vault.set_treasurer(treasurer)
    }

    pub fn add_currency(&mut self, currency: Pubkey) -> Result<()> {
        self.vault.add_currency(currency)
    }

    pub fn remove_currency(&mut self, currency: Pubkey) -> Result<()> {
        self.vault.remove_currency(currency)
    }
}
