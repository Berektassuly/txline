use txline::http::models::UpdateStats;
use txline::validation::legacy::{ScoreStat, ScoresBatchSummary};
use txline::validation::proof::Hash32;
use txline::validation::strategy::{
    BinaryExpression, Comparison, NDimensionalStrategy, TraderPredicate,
};
use txline::validation::v2::{ScoresStatValidationV2, ScoresStatValidationV2Response};
use txline::{ApiToken, GuestJwt, TxlineClient, TxlineConfig};

#[tokio::test]
async fn rejects_seq_zero_before_auth_or_network() {
    let client = TxlineClient::new(TxlineConfig::devnet()).unwrap();
    let err = client
        .scores()
        .stat_validation_legacy(17952170, 0, 1002, None)
        .await
        .unwrap_err();
    assert!(err.to_string().contains("seq must be greater than zero"));
}

#[test]
fn v2_preserves_requested_stat_key_order() {
    let validation =
        ScoresStatValidationV2::from_response(vec![1001, 1002], response_with(2)).unwrap();
    assert_eq!(validation.requested_stat_keys(), &[1001, 1002]);
    assert_eq!(validation.stats_to_prove()[0].key, 1);
    assert_eq!(validation.stats_to_prove()[1].key, 2);
    assert_eq!(validation.to_validation_input().stats.len(), 2);
}

#[test]
fn v2_rejects_length_mismatch() {
    let err = ScoresStatValidationV2::from_response(vec![1001, 1002, 1007], response_with(2))
        .unwrap_err();
    assert!(err.to_string().contains("statsToProve length"));
}

#[test]
fn strategy_builder_rejects_out_of_bounds_indices() {
    let predicate = TraderPredicate::new(0, Comparison::equal_to());
    let err = NDimensionalStrategy::builder(2)
        .binary(0, 2, BinaryExpression::subtract(), predicate)
        .unwrap_err();
    assert!(err.to_string().contains("out of bounds"));
}

#[test]
fn client_activation_preimage_uses_stored_jwt() {
    let client = TxlineClient::new(TxlineConfig::devnet()).unwrap();
    client.set_guest_jwt(GuestJwt::new("jwt").unwrap());
    client.set_api_token(ApiToken::new("api").unwrap());
    assert_eq!(
        client.activation_preimage("abc", &[1, 2]).unwrap(),
        "abc:1,2:jwt"
    );
}

fn response_with(count: usize) -> ScoresStatValidationV2Response {
    let hash = Hash32::from_bytes([9u8; 32]).unwrap();
    ScoresStatValidationV2Response {
        ts: 1,
        stats_to_prove: (0..count)
            .map(|idx| ScoreStat {
                key: (idx + 1) as u32,
                value: idx as i32,
                period: 0,
            })
            .collect(),
        event_stat_root: hash,
        summary: ScoresBatchSummary {
            fixture_id: 1,
            update_stats: UpdateStats {
                update_count: 1,
                min_timestamp: 1,
                max_timestamp: 1,
            },
            event_stats_sub_tree_root: hash,
        },
        stat_proofs: vec![Vec::new(); count],
        sub_tree_proof: Vec::new(),
        main_tree_proof: Vec::new(),
    }
}
