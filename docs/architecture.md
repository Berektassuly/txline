# Architecture

`txline-rs` is a Devnet-only SDK. The code is organized to keep hosted API
access, Solana transaction construction, and validation payload preparation
separate and reviewable.

## Design Principles

- Keep Devnet constants fixed in one configuration path.
- Prefer typed DTOs and validation helpers over caller-side JSON handling.
- Reject malformed local inputs before sending network requests.
- Keep secret material caller-owned and redacted in debug output.
- Keep transaction helpers conservative until each paid flow is audited end to
  end.

## Layers

| Layer | Modules | Responsibility |
| --- | --- | --- |
| Configuration | `config` | Canonical Devnet hosts, mints, program ID, and RPC override validation. |
| Credentials | `auth`, `client` | Guest JWTs, API tokens, activation preimages, and redacted headers. |
| Data access | `http` | Fixtures, odds, scores, purchase quotes, and validation endpoints. |
| Streams | `stream` | SSE parsing, heartbeat filtering, reconnects, and `Last-Event-ID`. |
| Solana | `solana` | Devnet PDAs, Token-2022 ATA derivation, and subscription helpers. |
| Validation | `validation` | Proof decoding, stat-validation DTOs, payload conversion, and strategies. |

## Runtime Flows

### Guest and API Credentials

1. Build `TxlineConfig::devnet()`.
2. Construct `TxlineClient::new(cfg)`.
3. Call `start_guest_session()` or set a caller-provided `GuestJwt`.
4. Submit a Devnet `subscribe(service_level_id, weeks)` transaction.
5. Sign the SDK-built activation preimage with the subscribing wallet.
6. Call `activate_subscription(...)` and store the returned `ApiToken`.

### REST Access

REST clients are exposed from `TxlineClient`:

- `fixtures()`
- `odds()`
- `scores()`

Authenticated requests automatically retry once with a fresh guest JWT on HTTP
401. HTTP status errors preserve the status code and response body.

### Streams

Odds and scores streams use Server-Sent Events. The typed stream wrapper:

- preserves `Last-Event-ID`,
- applies server-provided `retry` backoff hints,
- filters `event: heartbeat` before JSON deserialization,
- yields JSON errors for malformed data events.

### Validation

Validation helpers prepare payloads that match the hosted proof responses. V2
payloads preserve requested stat key order and verify returned stat keys by
position before exposing validation input.

## Public Surface

The crate exports a small top-level API:

- `TxlineClient`
- `TxlineConfig`
- `Network`
- `GuestJwt`
- `ApiToken`
- `AuthHeaders`
- `GuestSession`
- `activation_preimage`
- `Result`
- `TxlineError`

Internal modules stay public for the current SDK review phase, but new public
APIs should remain narrow and covered by tests.

## Out of Scope

- Mainnet constants or feature flags.
- Mainnet RPC support.
- Secret storage or wallet key management.
- Signing paid purchase quotes without caller review.
- Live Devnet tests as part of the default test suite.
