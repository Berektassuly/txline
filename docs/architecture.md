# Architecture

This SDK is Devnet-only in the current phase.

## Layers

| Layer | Modules | Responsibility |
| --- | --- | --- |
| Configuration | `config` | Canonical Devnet hosts, mints, program ID, and RPC override. |
| Credentials | `auth`, `client` | Guest JWTs, activated API tokens, activation preimage construction, and redacted header helpers. |
| Data access | `http` | Fixtures, odds, scores, purchase quote, and proof endpoints from the hosted OpenAPI. |
| Streams | `stream` | SSE parsing, heartbeats, `Last-Event-ID`, reconnects, and JWT renewal on 401. |
| Solana | `solana` | Devnet PDAs, Token-2022 ATA derivation, and `subscribe` transaction helpers. |
| Validation | `validation` | Proof decoding, legacy stat validation DTOs, V2 payload conversion, and strategy builders. |

## Flow

1. Build `TxlineConfig::devnet()` and `TxlineClient::new(cfg)`.
2. Call `start_guest_session()` or set a caller-provided guest JWT.
3. Submit a Devnet `subscribe(service_level_id, weeks)` transaction.
4. Sign the SDK-built activation preimage with the subscribing wallet.
5. Call `activate_subscription(...)` and store the returned API token.
6. Use fixtures, odds, scores, streams, and validation endpoints with both credentials.

Mainnet is out of scope for this implementation phase.
