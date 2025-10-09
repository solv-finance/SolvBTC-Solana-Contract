use crate::{errors::SolvError, state::Vault};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct VaultOracleUpdate<'info> {
    pub oracle_manager: Signer<'info>,
    #[account(mut)]
    pub vault: Account<'info, Vault>,
}

impl<'info> VaultOracleUpdate<'info> {
    pub fn set_nav(&mut self, nav: u64) -> Result<()> {
        require_keys_eq!(self.vault.oracle_manager, self.oracle_manager.key(), SolvError::InvalidAddress);
        self.vault.set_nav(nav)
    }
    
    pub fn set_manager(&mut self, manager: Pubkey) -> Result<()> {
        require_keys_eq!(self.vault.admin, self.oracle_manager.key(), SolvError::InvalidAddress);
        self.vault.set_oracle_manager(manager)
    }
}
