//! Server-Sent Events parsing and reconnect helpers.

use std::time::Duration;

use futures_util::{StreamExt, stream::BoxStream};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::{Result, TxlineClient, TxlineError};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RawSseEvent {
    pub id: Option<String>,
    pub event: Option<String>,
    pub data: String,
    pub retry: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SseEvent<T> {
    pub id: Option<String>,
    pub event: Option<String>,
    pub data: T,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StreamOptions {
    pub fixture_id: Option<i64>,
    pub last_event_id: Option<String>,
    pub initial_backoff: Duration,
    pub max_backoff: Duration,
}

impl Default for StreamOptions {
    fn default() -> Self {
        Self {
            fixture_id: None,
            last_event_id: None,
            initial_backoff: Duration::from_secs(1),
            max_backoff: Duration::from_secs(30),
        }
    }
}

#[derive(Debug, Default)]
pub struct SseDecoder {
    buffer: String,
}

impl SseDecoder {
    pub fn push(&mut self, bytes: &[u8]) -> Result<Vec<RawSseEvent>> {
        let chunk = std::str::from_utf8(bytes)
            .map_err(|err| TxlineError::invalid_input(format!("SSE utf8 error: {err}")))?;
        self.buffer.push_str(chunk);
        let mut events = Vec::new();
        while let Some((block, remainder)) = split_sse_block(&self.buffer) {
            let parsed = parse_sse_block(&block);
            self.buffer = remainder;
            if let Some(event) = parsed {
                events.push(event);
            }
        }
        Ok(events)
    }

    pub fn finish(&mut self) -> Option<RawSseEvent> {
        if self.buffer.trim().is_empty() {
            self.buffer.clear();
            return None;
        }
        let event = parse_sse_block(&self.buffer);
        self.buffer.clear();
        event
    }
}

pub fn parse_sse_block(block: &str) -> Option<RawSseEvent> {
    let mut message = RawSseEvent::default();
    for raw_line in block.lines() {
        if raw_line.is_empty() || raw_line.starts_with(':') {
            continue;
        }
        let (field, value) = raw_line
            .split_once(':')
            .map(|(field, value)| (field, value.strip_prefix(' ').unwrap_or(value)))
            .unwrap_or((raw_line, ""));
        match field {
            "id" => message.id = Some(value.to_owned()),
            "event" => message.event = Some(value.to_owned()),
            "data" => {
                message.data.push_str(value);
                message.data.push('\n');
            }
            "retry" => {
                if let Ok(retry) = value.parse::<u64>() {
                    message.retry = Some(retry);
                }
            }
            _ => {}
        }
    }
    if message.data.ends_with('\n') {
        message.data.pop();
    }
    if message.id.is_some() || message.event.is_some() || !message.data.is_empty() {
        Some(message)
    } else {
        None
    }
}

pub(crate) fn typed_stream<T>(
    client: TxlineClient,
    path: &'static str,
    options: StreamOptions,
) -> BoxStream<'static, Result<SseEvent<T>>>
where
    T: DeserializeOwned + Send + 'static,
{
    Box::pin(async_stream::stream! {
        let mut last_event_id = options.last_event_id;
        let mut backoff = options.initial_backoff;
        loop {
            let mut query = Vec::new();
            if let Some(fixture_id) = options.fixture_id {
                query.push(("fixtureId", fixture_id.to_string()));
            }

            match client.sse_response(path, query, last_event_id.as_deref()).await {
                Ok(response) => {
                    backoff = options.initial_backoff;
                    let mut decoder = SseDecoder::default();
                    let mut chunks = response.bytes_stream();
                    while let Some(chunk) = chunks.next().await {
                        let chunk = match chunk {
                            Ok(chunk) => chunk,
                            Err(err) => {
                                yield Err(TxlineError::from(err));
                                break;
                            }
                        };
                        let events = match decoder.push(&chunk) {
                            Ok(events) => events,
                            Err(err) => {
                                yield Err(err);
                                break;
                            }
                        };
                        for raw_event in events {
                            if let Some(id) = &raw_event.id {
                                last_event_id = Some(id.clone());
                            }
                            if let Some(retry) = raw_event.retry {
                                backoff = Duration::from_millis(retry).min(options.max_backoff);
                            }
                            if raw_event.data.is_empty() {
                                continue;
                            }
                            let data = match serde_json::from_str::<T>(&raw_event.data) {
                                Ok(data) => data,
                                Err(err) => {
                                    yield Err(TxlineError::from(err));
                                    continue;
                                }
                            };
                            yield Ok(SseEvent {
                                id: raw_event.id,
                                event: raw_event.event,
                                data,
                            });
                        }
                    }
                    if let Some(raw_event) = decoder.finish() {
                        if let Some(id) = &raw_event.id {
                            last_event_id = Some(id.clone());
                        }
                        if !raw_event.data.is_empty() {
                            let data = match serde_json::from_str::<T>(&raw_event.data) {
                                Ok(data) => data,
                                Err(err) => {
                                    yield Err(TxlineError::from(err));
                                    tokio::time::sleep(backoff).await;
                                    backoff = (backoff * 2).min(options.max_backoff);
                                    continue;
                                }
                            };
                            yield Ok(SseEvent {
                                id: raw_event.id,
                                event: raw_event.event,
                                data,
                            });
                        }
                    }
                }
                Err(err) => {
                    yield Err(err);
                }
            }
            tokio::time::sleep(backoff).await;
            backoff = (backoff * 2).min(options.max_backoff);
        }
    })
}

fn split_sse_block(buffer: &str) -> Option<(String, String)> {
    let lf = buffer.find("\n\n");
    let crlf = buffer.find("\r\n\r\n");
    let (idx, sep_len) = match (lf, crlf) {
        (Some(a), Some(b)) if b < a => (b, 4),
        (Some(a), _) => (a, 2),
        (None, Some(b)) => (b, 4),
        (None, None) => return None,
    };
    Some((buffer[..idx].to_owned(), buffer[idx + sep_len..].to_owned()))
}
