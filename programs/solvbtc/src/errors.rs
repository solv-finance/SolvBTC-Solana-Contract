use anchor_lang::prelude::*;

#[error_code]
pub enum SolvError {
    #[msg("SolvVault: Currency array full")]
    CurrencyArrayFull,
    #[msg("SolvVault: Currency already exists")]
    CurrencyAlreadyExists,
    #[msg("SolvVault: Currency not found")]
    CurrencyNotFound,
    #[msg("SolvVault: Invalid fee ratio")]
    InvalidFeeRatio,
    #[msg("SolvVault: InvalidHash")]
    InvalidHash,
    #[msg("SolvVault: Slippage exceeded")]
    SlippageExceeded,
    #[msg("SolvVault: Invalid address")]
    InvalidAddress,
    #[msg("SolvVault: Mint not whitelisted in vault")]
    MintNotWhitelisted,
    #[msg("SolvMinterManager: Minter array full")]
    MinterArrayFull,
    #[msg("SolvMinterManager: Minter already exists")]
    MinterAlreadyExists,
    #[msg("SolvMinterManager: Minter not found")]
    MinterNotFound,
    #[msg("SolvOracle: Invalid NAV value - must be >= 1 Bitcoin")]
    InvalidNAVValue,
    #[msg("SolvOracle: NAV exceeded")]
    NAVExceeded,
    #[msg("SolvOracle: Math overflow occurred")]
    MathOverflow,
    #[msg("SolvOracle: Invalid Max NAV Change - must be <=10,000")]
    InvalidMaxNavChange,
}
