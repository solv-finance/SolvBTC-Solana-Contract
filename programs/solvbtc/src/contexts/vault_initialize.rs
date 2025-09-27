use crate::{constants::ADMIN_WHITELIST, state::Vault};
use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;

#[derive(Accounts)]
pub struct VaultInitialize<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    /// Separate authority from payer to support multisig and PDA signers
    #[account(constraint = ADMIN_WHITELIST.contains(&authority.key()))]
    pub authority: Signer<'info>,
    pub mint: InterfaceAccount<'info, Mint>,
    #[account(
        init,
        payer = payer,
        space = Vault::DISCRIMINATOR.len() + Vault::INIT_SPACE,
        seeds = [b"vault", mint.key().as_ref()],
        bump
    )]
    pub vault: Account<'info, Vault>,
    pub system_program: Program<'info, System>,
}

impl<'info> VaultInitialize<'info> {
    #[allow(clippy::too_many_arguments)]
    pub fn initialize(
        &mut self,
        admin: Pubkey, 
        fee_receiver: Pubkey, 
        treasurer: Pubkey, 
        verifier: [u8; 64], 
        oracle_manager: Pubkey, 
        nav: u64,
        withdraw_fee: u16, 
        bump: u8
    ) -> Result<()> {
        self.vault.initialize(
            admin,
            self.mint.key(),
            fee_receiver,
            treasurer,
            verifier,
            oracle_manager,
            nav,
            withdraw_fee,
            bump,
        )
    }
}
