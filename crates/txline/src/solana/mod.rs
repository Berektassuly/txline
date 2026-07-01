//! Devnet Solana helpers.

pub mod pda;
pub mod purchase;
pub mod subscription;
pub mod transaction_safety;

use solana_sdk::hash::Hash;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{Signature, Signer};
use solana_sdk::transaction::Transaction;

use crate::Result;
use crate::config::TxlineConfig;
use crate::solana::pda::{DevnetPdas, parse_pubkey};
use crate::solana::subscription::{
    build_subscribe_transaction, send_subscribe_transaction, sign_subscribe_transaction,
};

#[derive(Debug, Clone, Copy)]
pub struct SolanaClient<'a> {
    config: &'a TxlineConfig,
}

impl<'a> SolanaClient<'a> {
    pub(crate) fn new(config: &'a TxlineConfig) -> Self {
        Self { config }
    }

    pub fn program_id(&self) -> Result<Pubkey> {
        parse_pubkey(&self.config.program_id)
    }

    pub fn pdas(&self) -> Result<DevnetPdas> {
        DevnetPdas::new()
    }

    pub fn build_subscribe_transaction(
        &self,
        user: Pubkey,
        service_level_id: u16,
        weeks: u8,
        recent_blockhash: Hash,
    ) -> Result<Transaction> {
        build_subscribe_transaction(
            self.program_id()?,
            user,
            service_level_id,
            weeks,
            recent_blockhash,
        )
    }

    pub fn sign_subscribe_transaction<S: Signer>(
        &self,
        signer: &S,
        service_level_id: u16,
        weeks: u8,
        recent_blockhash: Hash,
    ) -> Result<Transaction> {
        sign_subscribe_transaction(
            self.config,
            signer,
            service_level_id,
            weeks,
            recent_blockhash,
        )
    }

    pub fn send_subscribe_transaction<S: Signer>(
        &self,
        signer: &S,
        service_level_id: u16,
        weeks: u8,
    ) -> Result<Signature> {
        send_subscribe_transaction(self.config, signer, service_level_id, weeks)
    }
}
