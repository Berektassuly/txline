//! Fixture endpoints.

use crate::TxlineClient;
use crate::http::models::{Fixture, FixtureBatchValidation, FixtureValidation};
use crate::{Result, TxlineError};

#[derive(Debug, Clone, Copy)]
pub struct FixturesClient<'a> {
    client: &'a TxlineClient,
}

impl<'a> FixturesClient<'a> {
    pub(crate) fn new(client: &'a TxlineClient) -> Self {
        Self { client }
    }

    pub async fn snapshot(
        &self,
        start_epoch_day: Option<u32>,
        competition_id: Option<i32>,
    ) -> Result<Vec<Fixture>> {
        let mut query = Vec::new();
        if let Some(start_epoch_day) = start_epoch_day {
            query.push(("startEpochDay", start_epoch_day.to_string()));
        }
        if let Some(competition_id) = competition_id {
            query.push(("competitionId", competition_id.to_string()));
        }
        self.client
            .get_json("/fixtures/snapshot", query, true)
            .await
    }

    pub async fn updates(&self, epoch_day: u32, hour_of_day: u8) -> Result<Vec<Fixture>> {
        validate_hour(hour_of_day)?;
        self.client
            .get_json(
                &format!("/fixtures/updates/{epoch_day}/{hour_of_day}"),
                Vec::new(),
                true,
            )
            .await
    }

    pub async fn validation(
        &self,
        fixture_id: i64,
        timestamp: Option<i64>,
    ) -> Result<FixtureValidation> {
        let mut query = vec![("fixtureId", fixture_id.to_string())];
        if let Some(timestamp) = timestamp {
            query.push(("timestamp", timestamp.to_string()));
        }
        self.client
            .get_json("/fixtures/validation", query, true)
            .await
    }

    pub async fn batch_validation(
        &self,
        epoch_day: u32,
        hour_of_day: u8,
    ) -> Result<FixtureBatchValidation> {
        validate_hour(hour_of_day)?;
        self.client
            .get_json(
                "/fixtures/batch-validation",
                vec![
                    ("epochDay", epoch_day.to_string()),
                    ("hourOfDay", hour_of_day.to_string()),
                ],
                true,
            )
            .await
    }
}

pub(crate) fn validate_hour(hour_of_day: u8) -> Result<()> {
    if hour_of_day > 23 {
        return Err(TxlineError::invalid_input("hour_of_day must be 0..=23"));
    }
    Ok(())
}

pub(crate) fn validate_interval(interval: u8) -> Result<()> {
    if interval > 11 {
        return Err(TxlineError::invalid_input(
            "interval must be the 0-indexed 5-minute bucket 0..=11",
        ));
    }
    Ok(())
}
