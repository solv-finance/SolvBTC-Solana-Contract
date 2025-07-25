use crate::state::MinterManager;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct MinterManagerTransferAdmin<'info> {
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

impl<'info> MinterManagerTransferAdmin<'info> {
    pub fn transfer_admin(&mut self, admin: Pubkey) -> Result<()> {
        self.minter_manager.transfer_admin(admin)
    }
}
