//! Odds SSE stream.

use futures_util::stream::BoxStream;

use crate::http::models::OddsPayload;
use crate::stream::sse::{SseEvent, StreamOptions, typed_stream};
use crate::{Result, TxlineClient};

#[derive(Debug, Clone)]
pub struct OddsStreamClient {
    client: TxlineClient,
}

impl OddsStreamClient {
    pub(crate) fn new(client: TxlineClient) -> Self {
        Self { client }
    }

    pub fn stream(
        &self,
        options: StreamOptions,
    ) -> BoxStream<'static, Result<SseEvent<OddsPayload>>> {
        typed_stream(self.client.clone(), "/odds/stream", options)
    }

    pub fn stream_all(&self) -> BoxStream<'static, Result<SseEvent<OddsPayload>>> {
        self.stream(StreamOptions::default())
    }

    pub fn stream_fixture(
        &self,
        fixture_id: i64,
    ) -> BoxStream<'static, Result<SseEvent<OddsPayload>>> {
        self.stream(StreamOptions {
            fixture_id: Some(fixture_id),
            ..StreamOptions::default()
        })
    }
}
