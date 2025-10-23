use anchor_lang::prelude::*;

pub const ONE_BITCOIN: u64 = 100_000_000;
pub const ADMIN_WHITELIST: &[Pubkey] = &[
    Pubkey::from_str_const("3Aq5Zw5BjY3YfdFBLD1CKDbB5HgtXajJhhPBspXokbdS")
];

pub const MAX_FEE: u16 = 10_000;

/* Mints */
// pub const SOLV_MINT: ([u8; 32], u8) =
//     const_crypto::ed25519::derive_program_address(&[b"solv"], crate::ID_CONST.as_array());
// pub const SOLV_MINT_AUTH: Pubkey = Pubkey::new_from_array(SOLV_MINT.0);
// pub const SOLV_MINT_AUTH_BUMP: [u8; 1] = [SOLV_MINT.1];

// pub const POOL_SIGNER_SEED: &[u8] = b"ccip_tokenpool_signer";
// pub const TOKENPOOL_PROGRAM_ID: Pubkey = pubkey!("ECvqYduigrFHeAU1kFCkehiiQz9eaeddUz6gH7BfD7AL");

// pub const SOLV_BTC: Pubkey = pubkey!("SoLvHDFVstC74Jr9eNLTDoG4goSUsn1RENmjNtFKZvW");
// pub const X_SOLV_BTC: Pubkey = pubkey!("SoLvAiHLF7LGEaiTN5KGZt1bNnraoWTi5mjcvRoDAX4");
// pub const SOLV_BTC_JUP: Pubkey = pubkey!("SoLvzL3ZVjofmNB5LYFrf94QtNhMUSea4DawFhnAau8");

// HEoPcCuuHzvxBNjfG5i8jjCDqf8RsPyhuWQtE5oRSFYn
// pub const SOLV_BTC_POOL: ([u8; 32], u8) = const_crypto::ed25519::derive_program_address(
//     &[POOL_SIGNER_SEED, SOLV_BTC.as_array()],
//     TOKENPOOL_PROGRAM_ID.as_array(),
// );
// pub const SOLV_BTC_POOL_AUTH: Pubkey = Pubkey::new_from_array(SOLV_BTC_POOL.0);
// pub const SOLV_BTC_POOL_AUTH_BUMP: [u8; 1] = [SOLV_BTC_POOL.1];

// DcUZY1UdSc5mFT97wvfcDu7PB7ax5XJN62efdb4F6eW6
// pub const X_SOLV_BTC_POOL: ([u8; 32], u8) = const_crypto::ed25519::derive_program_address(
//     &[POOL_SIGNER_SEED, X_SOLV_BTC.as_array()],
//     TOKENPOOL_PROGRAM_ID.as_array(),
// );
// pub const X_SOLV_BTC_POOL_AUTH: Pubkey = Pubkey::new_from_array(X_SOLV_BTC_POOL.0);
// pub const X_SOLV_BTC_POOL_AUTH_BUMP: [u8; 1] = [X_SOLV_BTC_POOL.1];

// DbM3ixNXZ3BjvKNZANzXKjYh2ZywjyUoaB4uL7QmRkd7
// pub const SOLV_BTC_JUP_POOL: ([u8; 32], u8) = const_crypto::ed25519::derive_program_address(
//     &[POOL_SIGNER_SEED, SOLV_BTC_JUP.as_array()],
//     TOKENPOOL_PROGRAM_ID.as_array(),
// );
// pub const SOLV_BTC_JUP_POOL_AUTH: Pubkey = Pubkey::new_from_array(SOLV_BTC_JUP_POOL.0);
// pub const SOLV_BTC_JUP_POOL_AUTH_BUMP: [u8; 1] = [SOLV_BTC_JUP_POOL.1];

// #[test]
// fn get_key() {
//     println!("{}", SOLV_MINT_AUTH.to_string());
//     println!("{}", SOLV_BTC_POOL_AUTH.to_string());
//     println!("{}", X_SOLV_BTC_POOL_AUTH.to_string());
//     println!("{}", SOLV_BTC_JUP_POOL_AUTH.to_string());
// }
