use crate::state::MinterManager;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct MinterManagerUpdateMinter<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    /// Separate authority from payer to support multisig and PDA signers
    pub admin: Signer<'info>,
    #[account(
        mut,
        has_one = admin
    )]
    pub minter_manager: Account<'info, MinterManager>,
}

impl<'info> MinterManagerUpdateMinter<'info> {
    pub fn add_minter(&mut self, minter: Pubkey) -> Result<()> {
        self.minter_manager.add_minter(minter)
    }

    pub fn remove_minter(&mut self, minter: Pubkey) -> Result<()> {
        self.minter_manager.remove_minter(minter)
    }
}
