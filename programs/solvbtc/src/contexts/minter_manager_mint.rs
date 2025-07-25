use anchor_lang::prelude::*;
use crate::{helpers::{mint_to_checked_1_of_n_multisig, MintToChecked1ofNMultisig}, state::MinterManager};
use anchor_spl::token_interface::{
    Mint, TokenAccount, TokenInterface,
};

#[derive(Accounts)]
pub struct MinterManagerMint<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    /// Separate authority from payer to support multisig and PDA signers
    pub authority: Signer<'info>,
    #[account(mut)]
    pub mint: InterfaceAccount<'info, Mint>,
    /// The multisig account that serves as mint authority
    /// CHECK: This is validated as a multisig account by the token program
    pub multisig: AccountInfo<'info>,
    /// The PDA signer that is one of the multisig signers
    #[account(
        seeds = [b"vault", mint.key().as_ref()],
        bump
    )]
    /// CHECK: This PDA is one of the signers in the multisig
    pub vault: AccountInfo<'info>,
    #[account(
        mut,
        token::mint = mint
    )]
    pub to: InterfaceAccount<'info, TokenAccount>,
    // MinterManager
    #[account(
        constraint = minter_manager.minters.contains(&authority.key()),
        seeds = [b"minter_manager", vault.key().as_ref()],
        bump = minter_manager.bump
    )]
    pub minter_manager: Account<'info, MinterManager>,
    pub token_program: Interface<'info, TokenInterface>,
}

impl<'info> MinterManagerMint<'info> {
    pub fn mint(&mut self, amount: u64, pda_bump: [u8; 1]) -> Result<()> {
        // Create PDA signer seeds for the multisig operation
        let signer_seeds: [&[&[u8]];1] = [&[b"vault".as_ref(), self.mint.to_account_info().key.as_ref(), &pda_bump]];

        // For a 1/2 multisig, we only need 1 signature (the PDA)
        let accounts = MintToChecked1ofNMultisig {
            mint: self.mint.to_account_info(),
            to: self.to.to_account_info(),
            multisig: self.multisig.to_account_info(),
            signer: self.vault.to_account_info(),
        };

        let ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(), 
            accounts,
            &signer_seeds
        );

        mint_to_checked_1_of_n_multisig(ctx, amount, self.mint.decimals)
    }
}
