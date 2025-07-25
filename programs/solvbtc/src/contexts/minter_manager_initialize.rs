use crate::{constants::ADMIN_WHITELIST, state::MinterManager};
use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;

#[derive(Accounts)]
pub struct MinterManagerInitialize<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    /// Separate authority from payer to support multisig and PDA signers
    #[account(constraint = ADMIN_WHITELIST.contains(&authority.key()))]
    pub authority: Signer<'info>,
    pub mint: InterfaceAccount<'info, Mint>,
    #[account(
        seeds = [b"vault", mint.key().as_ref()],
        bump
    )]
    /// CHECK: This PDA is one of the signers in the multisig
    pub vault: AccountInfo<'info>,
    #[account(
        init,
        payer = payer,
        space = MinterManager::DISCRIMINATOR.len() + MinterManager::INIT_SPACE,
        seeds = [b"minter_manager", vault.key().as_ref()],
        bump
    )]
    pub minter_manager: Account<'info, MinterManager>,
    pub system_program: Program<'info, System>,
}

impl<'info> MinterManagerInitialize<'info> {
    pub fn initialize(&mut self, admin: Pubkey, bump: u8) -> Result<()> {
        self.minter_manager.initialize(admin, bump)
    }
}
