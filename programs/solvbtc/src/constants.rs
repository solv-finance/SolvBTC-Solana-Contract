use anchor_lang::prelude::*;

#[constant]
pub const ONE_BITCOIN: u64 = 100_000_000;
#[constant]
pub const ADMIN_WHITELIST: &[Pubkey] = &[
    Pubkey::from_str_const("BsF2mR9brTd7u7wGWrejksQzsdrGFNcddRSYeNpHZixM")
];
#[constant]
pub const MAX_FEE_BPS: u16 = 10_000;
#[constant]
pub const MAX_NAV_GROWTH_BPS: u16 = 5; // 0.05%
