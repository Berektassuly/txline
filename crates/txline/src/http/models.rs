//! Shared DTOs for the published Devnet OpenAPI endpoints.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::validation::proof::ProofNode;

pub type ExtraFields = Map<String, Value>;

pub const FIXTURE_GAME_STATE_SCHEDULED: i32 = 1;
pub const FIXTURE_GAME_STATE_CANCELLED: i32 = 6;
pub const SCORE_ACTION_GAME_FINALISED: &str = "game_finalised";
pub const FINAL_SETTLEMENT_STATUS_ID: i32 = 100;
pub const FINAL_SETTLEMENT_PERIOD: i32 = 100;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Fixture {
    #[serde(rename = "Ts")]
    pub ts: i64,
    #[serde(rename = "StartTime")]
    pub start_time: i64,
    #[serde(rename = "Competition")]
    pub competition: String,
    #[serde(rename = "CompetitionId")]
    pub competition_id: i32,
    #[serde(rename = "FixtureGroupId")]
    pub fixture_group_id: i32,
    #[serde(rename = "Participant1Id")]
    pub participant1_id: i32,
    #[serde(rename = "Participant1")]
    pub participant1: String,
    #[serde(rename = "Participant2Id")]
    pub participant2_id: i32,
    #[serde(rename = "Participant2")]
    pub participant2: String,
    #[serde(rename = "FixtureId")]
    pub fixture_id: i64,
    #[serde(rename = "Participant1IsHome")]
    pub participant1_is_home: bool,
    #[serde(default, rename = "GameState")]
    pub game_state: Option<i32>,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

impl Fixture {
    pub fn is_scheduled(&self) -> bool {
        self.game_state == Some(FIXTURE_GAME_STATE_SCHEDULED)
    }

    pub fn is_cancelled(&self) -> bool {
        self.game_state == Some(FIXTURE_GAME_STATE_CANCELLED)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OddsPayload {
    #[serde(rename = "FixtureId")]
    pub fixture_id: i64,
    #[serde(rename = "MessageId")]
    pub message_id: String,
    #[serde(rename = "Ts")]
    pub ts: i64,
    #[serde(rename = "Bookmaker")]
    pub bookmaker: String,
    #[serde(rename = "BookmakerId")]
    pub bookmaker_id: i32,
    #[serde(rename = "SuperOddsType")]
    pub super_odds_type: String,
    #[serde(default, rename = "GameState")]
    pub game_state: Option<String>,
    #[serde(rename = "InRunning")]
    pub in_running: bool,
    #[serde(default, rename = "MarketParameters")]
    pub market_parameters: Option<String>,
    #[serde(default, rename = "MarketPeriod")]
    pub market_period: Option<String>,
    #[serde(default, rename = "PriceNames")]
    pub price_names: Vec<String>,
    #[serde(default, rename = "Prices")]
    pub prices: Vec<i32>,
    #[serde(default, rename = "Pct")]
    pub pct: Vec<String>,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Scores {
    pub fixture_id: i64,
    pub game_state: String,
    pub start_time: i64,
    pub is_team: bool,
    pub fixture_group_id: i32,
    pub competition_id: i32,
    pub country_id: i32,
    pub sport_id: i32,
    pub participant1_is_home: bool,
    pub participant2_id: i32,
    pub participant1_id: i32,
    pub action: String,
    pub id: i32,
    pub ts: i64,
    pub connection_id: i64,
    pub seq: i32,
    #[serde(default)]
    pub status_id: Option<i32>,
    #[serde(default)]
    pub period: Option<i32>,
    #[serde(default)]
    pub coverage_secondary_data: Option<bool>,
    #[serde(default)]
    pub coverage_type: Option<String>,
    #[serde(default)]
    pub confirmed: Option<bool>,
    #[serde(default)]
    pub participant: Option<i32>,
    #[serde(default)]
    pub possession: Option<i32>,
    #[serde(default)]
    pub stats: Option<BTreeMap<String, i32>>,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

impl Scores {
    pub fn is_final_outcome_record(&self) -> bool {
        self.action == SCORE_ACTION_GAME_FINALISED
            && self.status_id == Some(FINAL_SETTLEMENT_STATUS_ID)
            && self.period == Some(FINAL_SETTLEMENT_PERIOD)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateStats {
    pub update_count: i32,
    pub min_timestamp: i64,
    pub max_timestamp: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchMetadata {
    pub total_update_count: i32,
    pub num_unique_fixtures: i32,
    pub overall_batch_start_ts: i64,
    pub overall_batch_end_ts: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FixtureBatchSummary {
    pub fixture_id: i64,
    pub competition_id: i32,
    pub competition: String,
    pub update_stats: UpdateStats,
    pub update_sub_tree_root: crate::validation::proof::Hash32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FixtureValidation {
    pub snapshot: Fixture,
    pub summary: FixtureBatchSummary,
    #[serde(default)]
    pub sub_tree_proof: Vec<ProofNode>,
    #[serde(default)]
    pub main_tree_proof: Vec<ProofNode>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FixtureBatchValidation {
    pub metadata: BatchMetadata,
    #[serde(default)]
    pub proof: Vec<ProofNode>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OddsBatchSummary {
    pub fixture_id: i64,
    pub update_stats: UpdateStats,
    pub odds_sub_tree_root: crate::validation::proof::Hash32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OddsValidation {
    pub odds: OddsPayload,
    pub summary: OddsBatchSummary,
    #[serde(default)]
    pub sub_tree_proof: Vec<ProofNode>,
    #[serde(default)]
    pub main_tree_proof: Vec<ProofNode>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PurchaseQuoteRequest {
    pub buyer_pubkey: String,
    pub txline_amount: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PurchaseQuoteResponse {
    pub transaction_base64: String,
    pub base_usdt_cost: f64,
    pub fee_usdt_amount: f64,
    pub total_usdt_charged: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixture_deserializes_game_state() {
        let fixture = serde_json::from_str::<Fixture>(
            r#"{
                "Ts": 1781123456789,
                "StartTime": 1781129999999,
                "Competition": "Cup",
                "CompetitionId": 10,
                "FixtureGroupId": 20,
                "Participant1Id": 30,
                "Participant1": "Home",
                "Participant2Id": 40,
                "Participant2": "Away",
                "FixtureId": 17952170,
                "Participant1IsHome": true,
                "GameState": 6
            }"#,
        )
        .unwrap();

        assert_eq!(fixture.game_state, Some(FIXTURE_GAME_STATE_CANCELLED));
        assert!(fixture.is_cancelled());
        assert!(!fixture.is_scheduled());
    }

    #[test]
    fn scores_deserializes_final_settlement_markers() {
        let score = serde_json::from_str::<Scores>(
            r#"{
                "fixtureId": 17952170,
                "gameState": "final",
                "startTime": 1781129999999,
                "isTeam": true,
                "fixtureGroupId": 20,
                "competitionId": 10,
                "countryId": 1,
                "sportId": 1,
                "participant1IsHome": true,
                "participant2Id": 40,
                "participant1Id": 30,
                "action": "game_finalised",
                "id": 99,
                "ts": 1781130000000,
                "connectionId": 77,
                "seq": 941,
                "statusId": 100,
                "period": 100
            }"#,
        )
        .unwrap();

        assert_eq!(score.status_id, Some(FINAL_SETTLEMENT_STATUS_ID));
        assert_eq!(score.period, Some(FINAL_SETTLEMENT_PERIOD));
        assert!(score.is_final_outcome_record());
    }
}
