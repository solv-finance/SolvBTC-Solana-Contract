use anchor_lang::prelude::*;

pub const ONE_BITCOIN: u64 = 100_000_000;
pub const FREEZE_PERIOD: i64 = 86400;
pub const DELTA: u128 = 5;
pub const ADMIN_WHITELIST: &[Pubkey] = &[
    Pubkey::from_str_const("BsF2mR9brTd7u7wGWrejksQzsdrGFNcddRSYeNpHZixM")
];

pub const MAX_FEE: u16 = 10_000;


