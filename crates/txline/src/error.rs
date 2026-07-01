//! SDK error types.

use thiserror::Error;

/// SDK result type.
pub type Result<T> = std::result::Result<T, TxlineError>;

#[derive(Debug, Error)]
pub enum TxlineError {
    #[error("configuration error: {0}")]
    Config(String),

    #[error("missing guest JWT; call start_guest_session or set_guest_jwt first")]
    MissingGuestJwt,

    #[error("missing API token; activate a subscription or call set_api_token first")]
    MissingApiToken,

    #[error("invalid input: {0}")]
    InvalidInput(String),

    #[error("proof decode error: {0}")]
    ProofDecode(String),

    #[error("validation payload error: {0}")]
    Validation(String),

    #[error("HTTP {status}: {body}")]
    HttpStatus { status: u16, body: String },

    #[error("HTTP client error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("URL parse error: {0}")]
    Url(#[from] url::ParseError),

    #[error("invalid HTTP header value: {0}")]
    Header(#[from] reqwest::header::InvalidHeaderValue),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("base64 decode error: {0}")]
    Base64(#[from] base64::DecodeError),

    #[error("Solana error: {0}")]
    Solana(String),

    #[error("Solana RPC error: {0}")]
    SolanaRpc(#[from] solana_client::client_error::ClientError),
}

impl TxlineError {
    pub(crate) fn config(message: impl Into<String>) -> Self {
        Self::Config(message.into())
    }

    pub(crate) fn invalid_input(message: impl Into<String>) -> Self {
        Self::InvalidInput(message.into())
    }

    pub(crate) fn proof_decode(message: impl Into<String>) -> Self {
        Self::ProofDecode(message.into())
    }

    pub(crate) fn validation(message: impl Into<String>) -> Self {
        Self::Validation(message.into())
    }

    pub(crate) fn solana(message: impl Into<String>) -> Self {
        Self::Solana(message.into())
    }
}
