use crate::{
    errors::SolvError, state::{Vault, WithdrawRequest}
};
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{burn_checked, BurnChecked, Mint, TokenAccount, TokenInterface},
};

#[derive(Accounts)]
#[instruction(hash: [u8;32])]
pub struct VaultRequestWithdraw<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        mut,
        token::authority = user,
        token::mint = mint_target
    )]
    pub user_target_ta: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        mut,
        token::authority = user,
        token::mint = mint_withdraw
    )]
    pub user_withdraw_ta: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(mut)]
    pub mint_target: Box<InterfaceAccount<'info, Mint>>,
    pub mint_withdraw: Box<InterfaceAccount<'info, Mint>>,
    #[account(
        init,
        payer = user,
        space = WithdrawRequest::DISCRIMINATOR.len() + WithdrawRequest::INIT_SPACE,
        seeds = [
            b"withdraw_request", 
            vault.key().as_ref(),
            mint_withdraw.key().as_ref(),
            user.key().as_ref(),
            hash.as_ref()
        ],
        bump,
    )]
    pub withdraw_request: Account<'info, WithdrawRequest>,
    #[account(
        mut,
        seeds = [b"vault", mint_target.key().as_ref()],
        bump = vault.bump,
        constraint = vault.is_whitelisted(&mint_withdraw.key())
    )]
    pub vault: Account<'info, Vault>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> VaultRequestWithdraw<'info> {
    pub fn burn_tokens(&mut self, amount: u64) -> Result<()> {
        // Ensure no zero values are withdrawn
        if amount.eq(&0) {
            return Err(SolvError::MathOverflow)?;
        }

        let accounts = BurnChecked {
            mint: self.mint_target.to_account_info(),
            from: self.user_target_ta.to_account_info(),
            authority: self.user.to_account_info(),
        };

        let ctx = CpiContext::new(self.token_program.to_account_info(), accounts);

        burn_checked(ctx, amount, self.mint_target.decimals)
    }

    pub fn open_request_account(
        &mut self,
        request_hash: [u8; 32],
        shares: u64,
    ) -> Result<()> {
        let withdraw_amount = self.vault.withdrawal_from_shares(shares)?;

        self.withdraw_request.initialize(
            self.user.key(),
            self.user_withdraw_ta.key(),
            self.mint_withdraw.key(),
            withdraw_amount,
            self.mint_target.key(),
            shares,
            request_hash,
            self.vault.nav,
        )
    }
}
