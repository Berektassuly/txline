//! Purchase quote helpers for paid Devnet flows.

use base64::Engine;
use base64::engine::general_purpose::STANDARD;

use crate::http::models::{PurchaseQuoteRequest, PurchaseQuoteResponse};
use crate::{Result, TxlineClient, TxlineError};

pub const MAX_QUOTE_TXLINE_AMOUNT: u64 = 100_000_000;

pub async fn purchase_quote(
    client: &TxlineClient,
    buyer_pubkey: impl Into<String>,
    txline_amount: u64,
) -> Result<PurchaseQuoteResponse> {
    validate_quote_amount(txline_amount)?;
    let request = PurchaseQuoteRequest {
        buyer_pubkey: buyer_pubkey.into(),
        txline_amount,
    };
    client
        .post_json("/guest/purchase/quote", &request, false)
        .await
}

pub fn validate_quote_amount(txline_amount: u64) -> Result<()> {
    if txline_amount == 0 || txline_amount > MAX_QUOTE_TXLINE_AMOUNT {
        return Err(TxlineError::invalid_input(format!(
            "txline_amount must be 1..={MAX_QUOTE_TXLINE_AMOUNT}"
        )));
    }
    Ok(())
}

impl PurchaseQuoteResponse {
    pub fn transaction_bytes(&self) -> Result<Vec<u8>> {
        let bytes = STANDARD.decode(&self.transaction_base64)?;
        if bytes.is_empty() {
            return Err(TxlineError::solana(
                "purchase quote transaction decoded to an empty byte buffer",
            ));
        }
        Ok(bytes)
    }

    pub fn validate_financial_shape(&self) -> Result<()> {
        if self.base_usdt_cost < 0.0 || self.fee_usdt_amount < 0.0 || self.total_usdt_charged < 0.0
        {
            return Err(TxlineError::solana(
                "purchase quote contains negative USDT amounts",
            ));
        }
        let expected = self.base_usdt_cost + self.fee_usdt_amount;
        if (expected - self.total_usdt_charged).abs() > 0.000_001 {
            return Err(TxlineError::solana(
                "purchase quote total does not equal base cost plus fee",
            ));
        }
        Ok(())
    }
}
