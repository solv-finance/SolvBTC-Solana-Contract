use anchor_lang::prelude::*;

#[event]
pub struct WithdrawRequestEvent {
    pub user: Pubkey,
    pub withdraw_token: Pubkey,
    pub withdraw_amount: u64,
    pub token: Pubkey,
    pub shares: u64,
    pub request_hash: [u8; 32],
    pub nav: u64,
}

#[event]
pub struct WithdrawEvent {
    pub user: Pubkey,
    pub withdraw_token: Pubkey,
    pub withdraw_amount: u64,
    pub request_hash: [u8; 32],
}

#[event]
pub struct DepositEvent {
    pub user: Pubkey,
    pub vault: Pubkey,
    pub mint_token: Pubkey,
    pub mint_target: Pubkey,
    pub deposit_amount: u64,
    pub mint_amount: u64,
    pub fee_amount: u64,
}