use anchor_lang::prelude::*;

use crate::errors::SolvError;

#[account(discriminator = [2])]
#[derive(InitSpace)]
pub struct MinterManager {
    pub admin: Pubkey,
    pub minters: [Pubkey; 10],
    pub updated: i64,
    pub bump: u8,
    _padding0: [u8; 7],
}

impl MinterManager {
    pub fn initialize(&mut self, admin: Pubkey, bump: u8) -> Result<()> {
        self.admin = admin;
        self.bump = bump;
        self.update()
    }

    #[inline(always)]
    fn update(&mut self) -> Result<()> {
        self.updated = Clock::get()?.unix_timestamp;
        Ok(())
    }

    pub fn transfer_admin(&mut self, admin: Pubkey) -> Result<()> {
        self.admin = admin;
        self.update()
    }

    pub fn add_minter(&mut self, minter: Pubkey) -> Result<()> {
        // Ensure we are not trying to add a null address
        if minter.eq(&Pubkey::default()) {
            return Err(SolvError::InvalidAddress.into());
        }
        // Find the first empty slot (Pubkey::default())
        if let Some(empty_index) = self
            .minters
            .iter()
            .position(|&pubkey| pubkey == Pubkey::default())
        {
            // Check if the minter already exists in the occupied slots (0..empty_index)
            if self.minters[0..empty_index].contains(&minter) {
                return Err(SolvError::MinterAlreadyExists.into());
            }

            // Add the minter to the first empty slot
            self.minters[empty_index] = minter;
            self.update()
        } else {
            // No empty slots available
            Err(SolvError::MinterArrayFull.into())
        }
    }

    pub fn remove_minter(&mut self, minter: Pubkey) -> Result<()> {
        // Ensure we are not trying to add a null address
        if minter.eq(&Pubkey::default()) {
            return Err(SolvError::InvalidAddress.into());
        }

        // Find the first instance of the minter
        if let Some(index) = self.minters.iter().position(|&pubkey| pubkey == minter) {
            // Shift all elements after the found index up by one position
            for i in index..self.minters.len() - 1 {
                self.minters[i] = self.minters[i + 1];
            }
            // Set the last element to default (empty)
            self.minters[self.minters.len() - 1] = Pubkey::default();
            self.update()
        } else {
            // Minter not found
            Err(SolvError::MinterNotFound.into())
        }
    }
}
