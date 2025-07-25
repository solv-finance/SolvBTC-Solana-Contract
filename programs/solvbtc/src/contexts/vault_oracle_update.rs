use crate::state::Vault;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct VaultOracleUpdate<'info> {
    pub oracle_manager: Signer<'info>,
    #[account(
        mut,
        has_one = oracle_manager
    )]
    pub vault: Account<'info, Vault>,
}

impl<'info> VaultOracleUpdate<'info> {
    pub fn set_nav(&mut self, nav: u64) -> Result<()> {
        self.vault.set_nav(nav)
    }

    pub fn set_manager(&mut self, manager: Pubkey) -> Result<()> {
        self.vault.set_oracle_manager(manager)
    }
}
