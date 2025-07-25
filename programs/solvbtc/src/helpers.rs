use anchor_lang::prelude::*;
use anchor_spl::token::spl_token;

#[derive(Accounts)]
pub struct MintToChecked1ofNMultisig<'info> {
    /// CHECK: This is safe
    pub mint: AccountInfo<'info>,
    /// CHECK: This is safe
    pub to: AccountInfo<'info>,
    /// CHECK: This is safe
    pub multisig: AccountInfo<'info>,
    /// CHECK: This is safe
    pub signer: AccountInfo<'info>,
}

pub fn mint_to_checked_1_of_n_multisig<'info>(
    ctx: anchor_lang::prelude::CpiContext<'_, '_, '_, 'info, MintToChecked1ofNMultisig<'info>>,
    amount: u64,
    decimals: u8,
) -> Result<()> {
    let ix = spl_token::instruction::mint_to_checked(
        ctx.program.key,
        ctx.accounts.mint.key,
        ctx.accounts.to.key,
        ctx.accounts.multisig.key,
        &[ctx.accounts.signer.key],
        amount,
        decimals,
    )?;
    anchor_lang::solana_program::program::invoke_signed(
        &ix,
        &[ctx.accounts.to, ctx.accounts.mint, ctx.accounts.multisig, ctx.accounts.signer],
        ctx.signer_seeds,
    )
    .map_err(Into::into)
}