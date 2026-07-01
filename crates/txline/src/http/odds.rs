//! Odds endpoints.

use crate::Result;
use crate::TxlineClient;
use crate::http::fixtures::{validate_hour, validate_interval};
use crate::http::models::{OddsPayload, OddsValidation};

#[derive(Debug, Clone, Copy)]
pub struct OddsClient<'a> {
    client: &'a TxlineClient,
}

impl<'a> OddsClient<'a> {
    pub(crate) fn new(client: &'a TxlineClient) -> Self {
        Self { client }
    }

    pub async fn snapshot(&self, fixture_id: i64, as_of: Option<i64>) -> Result<Vec<OddsPayload>> {
        let mut query = Vec::new();
        if let Some(as_of) = as_of {
            query.push(("asOf", as_of.to_string()));
        }
        self.client
            .get_json(&format!("/odds/snapshot/{fixture_id}"), query, true)
            .await
    }

    pub async fn live_updates_by_fixture(&self, fixture_id: i64) -> Result<Vec<OddsPayload>> {
        self.client
            .get_json(&format!("/odds/updates/{fixture_id}"), Vec::new(), true)
            .await
    }

    pub async fn historical_updates(
        &self,
        epoch_day: u32,
        hour_of_day: u8,
        interval: u8,
        fixture_id: Option<i64>,
    ) -> Result<Vec<OddsPayload>> {
        validate_hour(hour_of_day)?;
        validate_interval(interval)?;
        let mut query = Vec::new();
        if let Some(fixture_id) = fixture_id {
            query.push(("fixtureId", fixture_id.to_string()));
        }
        self.client
            .get_json(
                &format!("/odds/updates/{epoch_day}/{hour_of_day}/{interval}"),
                query,
                true,
            )
            .await
    }

    pub async fn validation(&self, message_id: &str, ts: i64) -> Result<OddsValidation> {
        self.client
            .get_json(
                "/odds/validation",
                vec![("messageId", message_id.to_owned()), ("ts", ts.to_string())],
                true,
            )
            .await
    }
}
