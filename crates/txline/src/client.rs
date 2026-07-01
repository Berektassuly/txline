//! Top-level TxLINE client.

use std::sync::{Arc, RwLock};

use reqwest::{Method, Response, StatusCode, Url};
use serde::{Serialize, de::DeserializeOwned};

use crate::auth::{
    ActivationPayload, ApiToken, AuthHeaders, GuestJwt, GuestSession, TokenResponse,
    activation_preimage,
};
use crate::config::TxlineConfig;
use crate::http::{fixtures::FixturesClient, odds::OddsClient, scores::ScoresClient};
use crate::solana::SolanaClient;
use crate::stream::{odds::OddsStreamClient, scores::ScoresStreamClient};
use crate::{Result, TxlineError};

#[derive(Debug, Default, Clone)]
struct TokenState {
    guest_jwt: Option<GuestJwt>,
    api_token: Option<ApiToken>,
}

/// Entry point for TxLINE Devnet operations.
#[derive(Debug, Clone)]
pub struct TxlineClient {
    config: TxlineConfig,
    http: reqwest::Client,
    tokens: Arc<RwLock<TokenState>>,
}

impl TxlineClient {
    /// Create a Devnet client from configuration.
    pub fn new(config: TxlineConfig) -> Result<Self> {
        config.validate()?;
        let http = reqwest::Client::builder()
            .user_agent(format!("txline-rs/{}", env!("CARGO_PKG_VERSION")))
            .build()?;
        Ok(Self {
            config,
            http,
            tokens: Arc::new(RwLock::new(TokenState::default())),
        })
    }

    pub fn config(&self) -> &TxlineConfig {
        &self.config
    }

    pub fn fixtures(&self) -> FixturesClient<'_> {
        FixturesClient::new(self)
    }

    pub fn odds(&self) -> OddsClient<'_> {
        OddsClient::new(self)
    }

    pub fn scores(&self) -> ScoresClient<'_> {
        ScoresClient::new(self)
    }

    pub fn odds_stream(&self) -> OddsStreamClient {
        OddsStreamClient::new(self.clone())
    }

    pub fn scores_stream(&self) -> ScoresStreamClient {
        ScoresStreamClient::new(self.clone())
    }

    pub fn solana(&self) -> SolanaClient<'_> {
        SolanaClient::new(&self.config)
    }

    pub async fn purchase_quote(
        &self,
        buyer_pubkey: impl Into<String>,
        txline_amount: u64,
    ) -> Result<crate::http::models::PurchaseQuoteResponse> {
        crate::solana::purchase::purchase_quote(self, buyer_pubkey, txline_amount).await
    }

    /// Acquire and store a fresh Devnet guest JWT.
    pub async fn start_guest_session(&self) -> Result<GuestSession> {
        let response = self.http.post(&self.config.guest_auth_url).send().await?;
        let token = Self::decode_response::<TokenResponse>(response)
            .await?
            .token;
        let token = GuestJwt::new(token)?;
        self.set_guest_jwt(token.clone());
        Ok(GuestSession { token })
    }

    pub fn set_guest_jwt(&self, jwt: GuestJwt) {
        let mut tokens = self.tokens.write().expect("token lock poisoned");
        tokens.guest_jwt = Some(jwt);
    }

    pub fn set_api_token(&self, token: ApiToken) {
        let mut tokens = self.tokens.write().expect("token lock poisoned");
        tokens.api_token = Some(token);
    }

    pub fn guest_jwt(&self) -> Option<GuestJwt> {
        self.tokens
            .read()
            .expect("token lock poisoned")
            .guest_jwt
            .clone()
    }

    pub fn api_token(&self) -> Option<ApiToken> {
        self.tokens
            .read()
            .expect("token lock poisoned")
            .api_token
            .clone()
    }

    pub fn auth_headers(&self, require_api_token: bool) -> Result<AuthHeaders> {
        let tokens = self.tokens.read().expect("token lock poisoned");
        let jwt = tokens
            .guest_jwt
            .clone()
            .ok_or(TxlineError::MissingGuestJwt)?;
        let api_token = if require_api_token {
            Some(
                tokens
                    .api_token
                    .clone()
                    .ok_or(TxlineError::MissingApiToken)?,
            )
        } else {
            tokens.api_token.clone()
        };
        Ok(AuthHeaders::new(jwt, api_token))
    }

    /// Activate an API token after a confirmed Devnet `subscribe` transaction.
    ///
    /// The caller signs [`activation_preimage`] and passes the base64 detached
    /// wallet signature. The SDK sends the stored guest JWT and persists the
    /// returned API token.
    pub async fn activate_subscription(
        &self,
        tx_sig: impl AsRef<str>,
        selected_leagues: &[i32],
        wallet_signature_base64: impl AsRef<str>,
    ) -> Result<ApiToken> {
        let jwt = self.guest_jwt().ok_or(TxlineError::MissingGuestJwt)?;
        let tx_sig = tx_sig.as_ref();
        let wallet_signature_base64 = wallet_signature_base64.as_ref();
        if tx_sig.trim().is_empty() {
            return Err(TxlineError::invalid_input(
                "subscription transaction signature must not be empty",
            ));
        }
        if wallet_signature_base64.trim().is_empty() {
            return Err(TxlineError::invalid_input(
                "wallet activation signature must not be empty",
            ));
        }

        let payload = ActivationPayload {
            tx_sig,
            wallet_signature: wallet_signature_base64,
            leagues: selected_leagues,
        };
        let response = self
            .http
            .post(self.api_url("/token/activate")?)
            .headers(AuthHeaders::new(jwt, None).to_header_map()?)
            .json(&payload)
            .send()
            .await?;
        let token_text = Self::decode_text_response(response).await?;
        let token = if token_text.trim_start().starts_with('{') {
            serde_json::from_str::<TokenResponse>(&token_text)?.token
        } else {
            token_text
        };
        let token = ApiToken::new(token)?;
        self.set_api_token(token.clone());
        Ok(token)
    }

    pub fn activation_preimage(
        &self,
        tx_sig: impl AsRef<str>,
        selected_leagues: &[i32],
    ) -> Result<String> {
        let jwt = self.guest_jwt().ok_or(TxlineError::MissingGuestJwt)?;
        Ok(activation_preimage(tx_sig, selected_leagues, &jwt))
    }

    pub(crate) async fn get_json<T>(
        &self,
        path: &str,
        query: Vec<(&'static str, String)>,
        require_api_token: bool,
    ) -> Result<T>
    where
        T: DeserializeOwned,
    {
        self.request_json(
            Method::GET,
            path,
            query,
            Option::<&()>::None,
            require_api_token,
        )
        .await
    }

    pub(crate) async fn post_json<B, T>(
        &self,
        path: &str,
        body: &B,
        require_api_token: bool,
    ) -> Result<T>
    where
        B: Serialize + ?Sized,
        T: DeserializeOwned,
    {
        self.request_json(
            Method::POST,
            path,
            Vec::new(),
            Some(body),
            require_api_token,
        )
        .await
    }

    async fn request_json<B, T>(
        &self,
        method: Method,
        path: &str,
        query: Vec<(&'static str, String)>,
        body: Option<&B>,
        require_api_token: bool,
    ) -> Result<T>
    where
        B: Serialize + ?Sized,
        T: DeserializeOwned,
    {
        let mut response = self
            .send_request(method.clone(), path, &query, body, require_api_token)
            .await?;
        if response.status() == StatusCode::UNAUTHORIZED {
            self.start_guest_session().await?;
            response = self
                .send_request(method, path, &query, body, require_api_token)
                .await?;
        }
        Self::decode_response(response).await
    }

    async fn send_request<B>(
        &self,
        method: Method,
        path: &str,
        query: &[(&'static str, String)],
        body: Option<&B>,
        require_api_token: bool,
    ) -> Result<Response>
    where
        B: Serialize + ?Sized,
    {
        let mut request = self
            .http
            .request(method, self.api_url(path)?)
            .headers(self.auth_headers(require_api_token)?.to_header_map()?);
        if !query.is_empty() {
            request = request.query(query);
        }
        if let Some(body) = body {
            request = request.json(body);
        }
        Ok(request.send().await?)
    }

    pub(crate) async fn sse_response(
        &self,
        path: &str,
        query: Vec<(&'static str, String)>,
        last_event_id: Option<&str>,
    ) -> Result<Response> {
        let mut response = self.send_sse_request(path, &query, last_event_id).await?;
        if response.status() == StatusCode::UNAUTHORIZED {
            self.start_guest_session().await?;
            response = self.send_sse_request(path, &query, last_event_id).await?;
        }
        if !response.status().is_success() {
            return Err(Self::status_error(response).await);
        }
        Ok(response)
    }

    async fn send_sse_request(
        &self,
        path: &str,
        query: &[(&'static str, String)],
        last_event_id: Option<&str>,
    ) -> Result<Response> {
        let mut request = self
            .http
            .get(self.api_url(path)?)
            .headers(self.auth_headers(true)?.to_header_map()?)
            .header(reqwest::header::ACCEPT, "text/event-stream")
            .header(reqwest::header::CACHE_CONTROL, "no-cache");
        if let Some(last_event_id) = last_event_id {
            request = request.header("Last-Event-ID", last_event_id);
        }
        if !query.is_empty() {
            request = request.query(query);
        }
        Ok(request.send().await?)
    }

    fn api_url(&self, path: &str) -> Result<Url> {
        let path = path.strip_prefix('/').unwrap_or(path);
        Ok(Url::parse(&format!("{}/{}", self.config.api_base, path))?)
    }

    async fn decode_response<T>(response: Response) -> Result<T>
    where
        T: DeserializeOwned,
    {
        if !response.status().is_success() {
            return Err(Self::status_error(response).await);
        }
        Ok(response.json::<T>().await?)
    }

    async fn decode_text_response(response: Response) -> Result<String> {
        if !response.status().is_success() {
            return Err(Self::status_error(response).await);
        }
        Ok(response.text().await?)
    }

    async fn status_error(response: Response) -> TxlineError {
        let status = response.status().as_u16();
        let body = response.text().await.unwrap_or_default();
        TxlineError::HttpStatus { status, body }
    }
}
