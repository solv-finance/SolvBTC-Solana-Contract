use anchor_lang::prelude::*;

use crate::{
    constants::{MAX_FEE_BPS, MAX_NAV_GROWTH_BPS, ONE_BITCOIN},
    errors::SolvError,
    helpers::{validate_nav, validate_pubkey},
};

#[account(discriminator = [1])]
#[derive(InitSpace)]
pub struct Vault {
    pub admin: Pubkey,
    pub mint: Pubkey,
    pub fee_receiver: Pubkey,
    pub treasurer: Pubkey,
    pub deposit_currencies: [WhitelistedToken; 10],
    pub verifier: [u8; 64],
    pub oracle_updated: i64,
    pub oracle_manager: Pubkey,
    pub nav: u64,
    pub withdraw_fee: u16,
    pub bump: u8,
    _padding0: [u8; 1],
}

#[derive(Default, Clone, Copy, InitSpace, AnchorSerialize, AnchorDeserialize)]
pub struct WhitelistedToken {
    mint: Pubkey,
    deposit_fee: u16,
}

impl Vault {
    #[allow(clippy::too_many_arguments)]
    pub fn initialize(
        &mut self,
        admin: Pubkey,
        mint: Pubkey,
        fee_receiver: Pubkey,
        treasurer: Pubkey,
        verifier: [u8; 64],
        oracle_manager: Pubkey,
        nav: u64,
        withdraw_fee: u16,
        bump: u8,
    ) -> Result<()> {
        validate_nav(nav)?;
        require_gte!(MAX_FEE_BPS, withdraw_fee, SolvError::InvalidFeeRatio);
        *self = Vault {
            admin,
            mint,
            fee_receiver,
            treasurer,
            verifier,
            deposit_currencies: [WhitelistedToken::default(); 10],
            oracle_updated: Clock::get()?.unix_timestamp,
            oracle_manager,
            nav,
            withdraw_fee,
            bump,
            _padding0: [0; 1],
        };
        Ok(())
    }

    pub fn is_whitelisted(&self, mint: &Pubkey) -> Result<()> {
        let is_whitelisted = self.deposit_currencies
            .iter()
            .find(|token| token.mint.eq(mint))
            .is_some();

        require!(is_whitelisted, SolvError::MintNotWhitelisted);

        Ok(())
    }

    pub fn update(&mut self) -> Result<()> {
        self.oracle_updated = Clock::get()?.unix_timestamp;
        Ok(())
    }

    pub fn transfer_admin(&mut self, admin: Pubkey) -> Result<()> {
        self.admin = admin;
        self.update()
    }

    pub fn set_deposit_fee(&mut self, currency: Pubkey, deposit_fee: u16) -> Result<()> {
        require_gte!(MAX_FEE_BPS, deposit_fee, SolvError::InvalidFeeRatio);
        let index = self
            .deposit_currencies
            .iter()
            .position(|token| token.mint.eq(&currency))
            .ok_or(SolvError::CurrencyNotFound)?;
        self.deposit_currencies[index].deposit_fee = deposit_fee;
        self.update()
    }

    pub fn set_withdraw_fee(&mut self, withdraw_fee: u16) -> Result<()> {
        require_gte!(MAX_FEE_BPS, withdraw_fee, SolvError::InvalidFeeRatio);
        self.withdraw_fee = withdraw_fee;
        self.update()
    }

    pub fn set_fee_receiver(&mut self, fee_receiver: Pubkey) -> Result<()> {
        self.fee_receiver = fee_receiver;
        self.update()
    }

    pub fn set_verifier(&mut self, verifier: [u8; 64]) -> Result<()> {
        self.verifier = verifier;
        self.update()
    }

    pub fn set_treasurer(&mut self, treasurer: Pubkey) -> Result<()> {
        self.treasurer = treasurer;
        self.update()
    }

    pub fn add_currency(&mut self, mint: Pubkey, deposit_fee: u16) -> Result<()> {
        // Ensure we are not trying to add a null address
        validate_pubkey(&mint)?;
        // Find the first empty slot (Pubkey::default())
        if let Some(empty_index) = self
            .deposit_currencies
            .iter()
            .position(|&token| token.mint.eq(&Pubkey::default()))
        {
            // Check if the currency already exists in the occupied slots (0..empty_index)
            if self.deposit_currencies[0..empty_index]
                .iter()
                .find(|token| token.mint.eq(&mint))
                .is_some()
            {
                return Err(SolvError::CurrencyAlreadyExists.into());
            }

            // Add the currency to the first empty slot
            self.deposit_currencies[empty_index] = WhitelistedToken { mint, deposit_fee };

            self.update()
        } else {
            // No empty slots available
            Err(SolvError::CurrencyArrayFull.into())
        }
    }

    pub fn remove_currency(&mut self, currency: Pubkey) -> Result<()> {
        // Ensure we are not trying to add a null address
        validate_pubkey(&currency)?;
        // Find the first instance of the currency
        if let Some(index) = self
            .deposit_currencies
            .iter()
            .position(|&token| token.mint.eq(&currency))
        {
            // Shift all elements after the found index up by one position
            for i in index..self.deposit_currencies.len() - 1 {
                self.deposit_currencies[i] = self.deposit_currencies[i + 1];
            }
            // Set the last element to default (empty)
            self.deposit_currencies[self.deposit_currencies.len() - 1] =
                WhitelistedToken::default();

            self.update()
        } else {
            // Currency not found
            Err(SolvError::CurrencyNotFound.into())
        }
    }

    pub fn set_nav(&mut self, nav: u64) -> Result<()> {
        // Check nav growth/ does not exceed 0.05%
        let nav_diff: u64 = u64::try_from(
            u128::from(self.nav)
                .checked_mul(MAX_NAV_GROWTH_BPS as u128)
                .ok_or(ProgramError::ArithmeticOverflow)?
                .checked_div(MAX_FEE_BPS.into())
                .ok_or(ProgramError::ArithmeticOverflow)?,
        )
        .map_err(|_| ProgramError::ArithmeticOverflow)?;

        let max_nav = self
            .nav
            .checked_add(nav_diff)
            .ok_or(ProgramError::ArithmeticOverflow)?;
        let min_nav = self
            .nav
            .checked_sub(nav_diff)
            .ok_or(ProgramError::ArithmeticOverflow)?;
        require_gte!(max_nav, nav, SolvError::InvalidNAVValue);
        require_gte!(nav, min_nav, SolvError::InvalidNAVValue);
        validate_nav(nav)?;
        self.nav = nav;
        self.update()
    }

    pub fn set_oracle_manager(&mut self, manager: Pubkey) -> Result<()> {
        self.oracle_manager = manager;
        self.update()
    }

    pub fn calculate_fee(amount: u64, fee: u16) -> Result<(u64, u64)> {
        let fee: u64 = u128::from(amount)
            .checked_mul(fee as u128)
            .ok_or(ProgramError::ArithmeticOverflow)?
            .checked_div(MAX_FEE_BPS.into())
            .ok_or(ProgramError::ArithmeticOverflow)?
            .try_into()
            .map_err(|_| ProgramError::ArithmeticOverflow)?;
        let amount = amount
            .checked_sub(fee)
            .ok_or(ProgramError::ArithmeticOverflow)?;
        Ok((amount, fee))
    }

    pub fn deposit_fee(&self, currency: &Pubkey) -> Result<u16> {
        let index = self
            .deposit_currencies
            .iter()
            .position(|token| token.mint.eq(currency))
            .ok_or(SolvError::CurrencyNotFound)?;
        Ok(self.deposit_currencies[index].deposit_fee)
    }

    /// Calculate shares to mint from a deposit amount
    /// deposit_amount * ONE_BITCOIN / nav = shares
    pub fn shares_from_deposit(&self, deposit_amount: u64) -> Result<u64> {
        u128::from(deposit_amount)
            .checked_mul(ONE_BITCOIN.into())
            .ok_or(ProgramError::ArithmeticOverflow)?
            .checked_div(self.nav.into())
            .ok_or(ProgramError::ArithmeticOverflow)?
            .try_into()
            .map_err(|_| ProgramError::ArithmeticOverflow.into())
    }

    /// Calculate withdrawal amount from shares to burn
    /// shares * nav / ONE_BITCOIN = withdrawal_amount
    pub fn withdrawal_from_shares(&self, shares: u64) -> Result<u64> {
        validate_nav(self.nav)?;
        u128::from(shares)
            .checked_mul(self.nav.into())
            .ok_or(ProgramError::ArithmeticOverflow)?
            .checked_div(ONE_BITCOIN.into())
            .ok_or(ProgramError::ArithmeticOverflow)?
            .try_into()
            .map_err(|_| ProgramError::ArithmeticOverflow.into())
    }
}
