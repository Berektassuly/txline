//! Server-Sent Events support for Devnet odds and scores streams.

pub mod odds;
pub mod scores;
pub mod sse;

pub use sse::{RawSseEvent, SseDecoder, SseEvent, StreamOptions, parse_sse_block};
