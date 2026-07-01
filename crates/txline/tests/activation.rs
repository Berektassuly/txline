use txline::{ApiToken, GuestJwt, TxlineClient, TxlineConfig, activation_preimage};

#[test]
fn activation_preimage_preserves_empty_league_slot() {
    let jwt = GuestJwt::new("jwt-value").unwrap();
    assert_eq!(activation_preimage("txSig", &[], &jwt), "txSig::jwt-value");
}

#[test]
fn activation_preimage_joins_leagues_in_order() {
    let jwt = GuestJwt::new("jwt-value").unwrap();
    assert_eq!(
        activation_preimage("txSig", &[501, 804, 202], &jwt),
        "txSig:501,804,202:jwt-value"
    );
}

#[test]
fn auth_debug_redacts_secrets() {
    let client = TxlineClient::new(TxlineConfig::devnet()).unwrap();
    client.set_guest_jwt(GuestJwt::new("secret-jwt").unwrap());
    client.set_api_token(ApiToken::new("secret-api-token").unwrap());

    let headers = client.auth_headers(true).unwrap();
    let debug = format!("{headers:?}");

    assert!(!debug.contains("secret-jwt"));
    assert!(!debug.contains("secret-api-token"));
    assert!(debug.contains("<redacted>"));
    assert!(headers.has_api_token());
}
