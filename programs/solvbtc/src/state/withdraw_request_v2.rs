use anchor_lang::prelude::*;
use solana_secp256k1::UncompressedPoint;
use solana_secp256k1_ecdsa::{Secp256k1EcdsaSignature, hash::keccak::Keccak};
use const_crypto::bs58::{encode_pubkey};

use crate::events::WithdrawRequestEventV2;

#[account(discriminator = [4])]
#[derive(InitSpace)]
pub struct WithdrawRequestV2 {
    pub user: Pubkey,
    pub withdraw_token_account: Pubkey,
    pub withdraw_token: Pubkey,
    pub withdraw_amount: u64,
    pub token: Pubkey,
    pub shares: u64,
    pub request_hash: [u8; 32],
    pub nav: u64,
    pub mint: Pubkey
}

impl WithdrawRequestV2 {
    #[allow(clippy::too_many_arguments)]
    pub fn initialize(
        &mut self,
        user: Pubkey,
        withdraw_token_account: Pubkey,
        withdraw_token: Pubkey,
        withdraw_amount: u64,
        token: Pubkey,
        shares: u64,
        request_hash: [u8; 32],
        nav: u64,
        mint: Pubkey
    ) -> Result<()> {

        *self = WithdrawRequestV2 {
            user,
            withdraw_token_account,
            withdraw_token,
            withdraw_amount,
            token,
            shares,
            request_hash,
            nav,
            mint,
        };

        // Emit initialize event
        emit!(WithdrawRequestEventV2 {
            user,
            withdraw_token,
            withdraw_amount,
            token,
            shares,
            request_hash,
            nav,
            mint
        });

        Ok(())
    }

    pub fn verify_eip191(&self, signature: Secp256k1EcdsaSignature, verifier: [u8;64]) -> Result<()>{
        Ok(signature
            .normalize_s()
            .verify::<Keccak, UncompressedPoint>(&self.eip191_msg(), UncompressedPoint(verifier))
            .map_err(|_| ProgramError::MissingRequiredSignature)?)
    }

    pub fn concat(&self) -> String {
        let data = self.user.to_string() + "\n" +
        self.withdraw_token.to_string().as_str() + "\n" +
        encode_pubkey(&self.request_hash).str() + "\n" +
        &self.shares.to_string() + "\n" +
        &self.nav.to_string() + "\n" +
        &self.mint.to_string();
        data
    }

    pub fn eip191_msg(&self) -> Vec<u8>{
        let message_prefix = "\x19Ethereum Signed Message:\n";
        let data = self.concat();
        let length= data.len().to_string();
        let full_msg = message_prefix.to_string() + &length + &data;
        full_msg.as_bytes().to_vec()
    }
}
