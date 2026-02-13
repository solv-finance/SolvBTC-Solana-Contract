use crate::state::MinterManager;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct MinterManagerTransferAdmin<'info> {

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
