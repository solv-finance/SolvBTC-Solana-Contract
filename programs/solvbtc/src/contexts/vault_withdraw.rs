use crate::events::WithdrawEvent;
use crate::state::{Vault, WithdrawRequest};
use crate::errors::SolvError;
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked},
};
use crate::{constants::{MAX_FEE_BPS, MAX_NAV_DIFF_BPS}};

use solana_secp256k1_ecdsa::{SECP256K1_ECDSA_SIGNATURE_LENGTH, Secp256k1EcdsaSignature};

#[derive(Accounts)]
#[instruction(hash: [u8;32])]
pub struct VaultWithdraw<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        mut,
        owner = crate::ID,
        seeds = [
            b"withdraw_request", 
            vault.key().as_ref(),
            mint_withdraw.key().as_ref(),
            user.key().as_ref(),
            hash.as_ref(),
        ],
        bump
    )]
    /// CHECK: We manually deserialize this in the function body
    pub withdraw_request: AccountInfo<'info>,
    #[account(
        mut,
        token::authority = user,
        token::mint = mint_withdraw,
    )]
    pub user_withdraw_ta: Box<InterfaceAccount<'info, TokenAccount>>,
    pub mint_withdraw: Box<InterfaceAccount<'info, Mint>>,
    #[account(
        mut,
        seeds = [b"vault",vault.mint.key().as_ref()],
        bump = vault.bump,
    )]
    pub vault: Account<'info, Vault>,
    #[account(
        mut,
        associated_token::authority = vault,
        associated_token::mint = mint_withdraw
    )]
    pub vault_withdraw_ta: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        mut,
        associated_token::authority = vault.fee_receiver,
        associated_token::mint = mint_withdraw
    )]
    pub fee_receiver_ta: Box<InterfaceAccount<'info, TokenAccount>>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> VaultWithdraw<'info> {
    pub fn validate(&self) -> Result<()> {
        self.vault.is_whitelisted(&self.mint_withdraw.key())?;

        Ok(())
    }

    pub fn withdraw_tokens(&mut self, signature: [u8; SECP256K1_ECDSA_SIGNATURE_LENGTH]) -> Result<()> {
        // Get withdraw request
        let mut withdraw_request_data = &self.withdraw_request.data.borrow()[..];
        let withdraw_request = WithdrawRequest::try_deserialize(&mut withdraw_request_data)?;

        // Verify withdraw account address;
        if self.user_withdraw_ta.key().ne(&withdraw_request.withdraw_token_account) {
            return Err(SolvError::InvalidAddress)?;
        }

        // Verify signature
        withdraw_request.verify_signature(Secp256k1EcdsaSignature(signature), self.vault.verifier)?;

        // Check 1.01*nav >= nav of withdraw request
        let nav_diff: u64 = u64::try_from(u128::from(self.vault.nav)
            .checked_mul(MAX_NAV_DIFF_BPS as u128)
            .ok_or(ProgramError::ArithmeticOverflow)?
            .checked_div(MAX_FEE_BPS.into())
            .ok_or(ProgramError::ArithmeticOverflow)?)
            .map_err(|_| ProgramError::ArithmeticOverflow)?;
        let max_nav = self.vault.nav.checked_add(nav_diff).ok_or(ProgramError::ArithmeticOverflow)?;
        require_gte!(max_nav, withdraw_request.nav, SolvError::NAVExceeded);

        // Get withdraw amount and withdraw fee
        let (amount, fee) = Vault::calculate_fee(withdraw_request.withdraw_amount, self.vault.withdraw_fee)?;

        // Signer seeds
        let key = self.vault.mint.key();
        let bump = [self.vault.bump];
        let signer_seeds: [&[&[u8]]; 1] = [&[b"vault", key.as_ref(), bump.as_ref()]];
        // Withdraw fee
        let accounts = TransferChecked {
            from: self.vault_withdraw_ta.to_account_info(),
            to: self.fee_receiver_ta.to_account_info(),
            mint: self.mint_withdraw.to_account_info(),
            authority: self.vault.to_account_info(),
        };

        let ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            accounts,
            &signer_seeds,
        );

        transfer_checked(ctx, fee, self.mint_withdraw.decimals)?;

        // Withdraw amount
        let accounts = TransferChecked {
            from: self.vault_withdraw_ta.to_account_info(),
            to: self.user_withdraw_ta.to_account_info(),
            mint: self.mint_withdraw.to_account_info(),
            authority: self.vault.to_account_info(),
        };

        let ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            accounts,
            &signer_seeds,
        );

        transfer_checked(ctx, amount, self.mint_withdraw.decimals)?;

        emit!(WithdrawEvent {
            user: self.user.key(),
            withdraw_amount:amount, 
            withdraw_token: self.mint_withdraw.key(), 
            request_hash: withdraw_request.request_hash, 
            withdraw_fee: fee,
        });

        Ok(())
    }

    /// Resize the withdraw_request account to zero bytes and refund Rent to user
    pub fn close_request_account(&mut self) -> Result<()> {
        let rent = Rent::get()?;
        let min_balance_for_zero = rent.minimum_balance(0);
        let current_lamports = self.withdraw_request.lamports();

        // Resize to zero first
        self.withdraw_request.resize(0)?;

        // Calculate how much to refund (keeping minimum for 0-byte account)
        if current_lamports > min_balance_for_zero {
            let refund = current_lamports - min_balance_for_zero;
            self.withdraw_request.sub_lamports(refund)?;
            self.user.add_lamports(refund)?;
        }
        Ok(())
    }
}
