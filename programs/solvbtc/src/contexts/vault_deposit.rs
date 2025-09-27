use crate::{errors::SolvError, events::DepositEvent, helpers::{mint_to_checked_1_of_n_multisig, MintToChecked1ofNMultisig}, state::Vault};
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{
        transfer_checked, Mint, TokenAccount, TokenInterface,
        TransferChecked,
    },
};

#[derive(Accounts)]
pub struct VaultDeposit<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        mut,
        token::authority = user,
        token::mint = mint_token
    )]
    pub user_token_ta: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        mut,
        token::authority = user,
        token::mint = mint_target
    )]
    pub user_target_ta: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        mut,
        associated_token::authority = vault.treasurer,
        associated_token::mint = mint_token
    )]
    pub treasurer_token_ta: Box<InterfaceAccount<'info, TokenAccount>>,
    /// The multisig account that serves as mint authority
    /// CHECK: This is validated as a multisig account by the token program
    pub multisig: AccountInfo<'info>,
    pub mint_token: Box<InterfaceAccount<'info, Mint>>,
    #[account(mut)]
    pub mint_target: Box<InterfaceAccount<'info, Mint>>,
    #[account(
        mut,
        seeds = [b"vault", mint_target.key().as_ref()],
        bump = vault.bump,
        constraint = vault.is_whitelisted(&mint_token.key()),
    )]
    pub vault: Account<'info, Vault>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> VaultDeposit<'info> {
    pub fn deposit_tokens(&mut self, amount: u64) -> Result<()> {
        let accounts = TransferChecked {
            from: self.user_token_ta.to_account_info(),
            mint: self.mint_token.to_account_info(),
            to: self.treasurer_token_ta.to_account_info(),
            authority: self.user.to_account_info(),
        };

        let ctx = CpiContext::new(self.token_program.to_account_info(), accounts);

        transfer_checked(ctx, amount, self.mint_token.decimals)
    }

    pub fn mint_target_tokens(&mut self, amount: u64, min_amount_out: u64) -> Result<()> {
        let (mint_amount, fee_amount) = Vault::calculate_fee(self.vault.shares_from_deposit(amount)?, self.vault.deposit_fee(&self.mint_token.key())?)?;

        // Slippage protection
        require_gte!(mint_amount, min_amount_out, SolvError::SlippageExceeded);

        // For a 1/2 multisig, we only need 1 signature (the PDA)
        let accounts = MintToChecked1ofNMultisig {
            mint: self.mint_target.to_account_info(),
            to: self.user_target_ta.to_account_info(),
            multisig: self.multisig.to_account_info(),
            signer: self.vault.to_account_info(),
        };
        
        // Create PDA signer seeds for the multisig operation
        let signer_seeds: [&[&[u8]];1] = [&[b"vault".as_ref(), self.mint_target.to_account_info().key.as_ref(), &[self.vault.bump]]];

        let ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(), 
            accounts,
            &signer_seeds
        );

        mint_to_checked_1_of_n_multisig(ctx, mint_amount, self.mint_target.decimals)?;

        emit!(DepositEvent {
            user: self.user.key(),
            vault: self.vault.key(),
            mint_token: self.mint_token.key(),
            mint_target: self.mint_target.key(),
            deposit_amount: amount,
            mint_amount,
            fee_amount,
        });

        Ok(())
    }
}
