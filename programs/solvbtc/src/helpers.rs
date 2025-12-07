use anchor_lang::{prelude::*, solana_program::program::invoke_signed};
use anchor_spl::token::spl_token::instruction::mint_to_checked;

use crate::{
    constants::{MAX_FEE_BPS, ONE_BITCOIN},
    errors::SolvError,
};

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
    ctx: CpiContext<'_, '_, '_, 'info, MintToChecked1ofNMultisig<'info>>,
    amount: u64,
    decimals: u8,
) -> Result<()> {
    let ix = mint_to_checked(
        ctx.program.key,
        ctx.accounts.mint.key,
        ctx.accounts.to.key,
        ctx.accounts.multisig.key,
        &[ctx.accounts.signer.key],
        amount,
        decimals,
    )?;
    invoke_signed(
        &ix,
        &[
            ctx.accounts.to,
            ctx.accounts.mint,
            ctx.accounts.multisig,
            ctx.accounts.signer,
        ],
        ctx.signer_seeds,
    )
    .map_err(Into::into)
}

pub fn validate_nav(nav: u64) -> Result<()> {
    require_gte!(nav, ONE_BITCOIN, SolvError::InvalidNAVValue);

    Ok(())
}

pub fn validate_pubkey(address: &Pubkey) -> Result<()> {
    if address.eq(&Pubkey::default()) {
        return Err(SolvError::InvalidAddress.into());
    }
    Ok(())
}

pub fn validate_fee(value: u16) -> Result<()> {
    if value > MAX_FEE_BPS {
        return Err(SolvError::InvalidFeeRatio.into());
    }
    Ok(())
}
