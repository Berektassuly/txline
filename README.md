# txline-rs

Devnet-only Rust SDK for TxLINE.

This implementation intentionally supports **TxLINE Devnet only**. Mainnet
constants, feature flags, examples, and transaction flows are out of scope for
this phase.

## Devnet Values

| Value | Devnet |
| --- | --- |
| API host | `https://txline-dev.txodds.com` |
| API base | `https://txline-dev.txodds.com/api` |
| Guest auth URL | `https://txline-dev.txodds.com/auth/guest/start` |
| Program ID | `6pW64gN1s2uqjHkn1unFeEjAwJkPGHoppGvS715wyP2J` |
| TxL mint | `4Zao8ocPhmMgq7PdsYWyxvqySMGx7xb9cMftPMkEokRG` |
| USDT mint | `ELWTKspHKCnCfCiCiqYw1EDH77k8VCP74dK9qytG2Ujh` |
| Default RPC | `https://api.devnet.solana.com` |

## Quick Start

```rust,no_run
use txline::{ApiToken, GuestJwt, TxlineClient, TxlineConfig};

# async fn run() -> txline::Result<()> {
let cfg = TxlineConfig::devnet();
let client = TxlineClient::new(cfg)?;

let guest = client.start_guest_session().await?;

// After a confirmed Devnet subscribe transaction, sign:
let message = client.activation_preimage("SUBSCRIBE_TX_SIGNATURE", &[])?;
// Then call activate_subscription with the base64 detached wallet signature.

client.set_guest_jwt(GuestJwt::new(guest.token.as_str())?);
client.set_api_token(ApiToken::new("activated-api-token")?);

let fixtures = client.fixtures().snapshot(None, None).await?;
println!("fixtures: {}", fixtures.len());
# Ok(())
# }
```

Activation signs this exact preimage:

```text
${txSig}:${selectedLeagues.join(",")}:${jwt}
```

For the standard bundle with no selected leagues:

```text
${txSig}::${jwt}
```

## Implemented

- Devnet config and client construction.
- Guest JWT acquisition and storage.
- API-token activation request after Devnet `subscribe`.
- Authenticated REST access for fixtures, odds, scores, validation, and purchase quotes.
- Legacy and V2 scores stat-validation DTOs and conversion helpers.
- 32-byte proof hash decoding from base64, hex, and byte arrays.
- V2 strategy builder for single, binary, geometric, and distance predicates.
- Devnet PDA helpers, including Token-2022 ATA derivation.
- Anchor-compatible `subscribe(service_level_id, weeks)` instruction and sign/send helpers.
- SSE parsing and reconnecting odds/scores stream wrappers.
- Tests for Devnet constants, activation preimages, redaction, proof decoding, PDA derivation, V2 ordering, and strategy bounds.

## Still Intentional Future Work

- Mainnet support.
- Full Anchor client bindings for on-chain validation transactions.
- Full purchase-quote transaction audit before signing. The SDK decodes quote bytes and checks financial shape, but callers must still inspect fee payer, signers, invoked programs, account metas, and decoded instruction before signing paid quote transactions.
- Live integration tests with real Devnet credentials.

## Examples

Examples require caller-provided Devnet credentials. They do not contain real
tokens or private keys.

```bash
cargo run -p txline --example devnet_free_tier
cargo run -p txline --example devnet_scores_stream
cargo run -p txline --example devnet_validate_stat
cargo run -p txline --example devnet_validate_stat_v2
```

Common env vars:

```bash
TXLINE_DEVNET_JWT=...
TXLINE_DEVNET_API_TOKEN=...
TXLINE_FIXTURE_ID=17952170
TXLINE_SCORE_SEQ=941
TXLINE_STAT_KEY=1002
TXLINE_STAT_KEYS=1,2,3001,3002
```

`TXLINE_SCORE_SEQ` must come from a real score record observed through snapshot,
updates, historical scores, or the scores stream.

## Verification

```bash
cargo fmt
cargo check --all-features
cargo test
```

Normal tests do not require live Devnet credentials.

## Sources

- TxLINE docs: <https://txline.txodds.com/documentation/quickstart>
- OpenAPI: <https://txline.txodds.com/docs/docs.yaml>
- Program addresses: <https://txline.txodds.com/documentation/programs/addresses>
- Streaming docs: <https://txline.txodds.com/documentation/examples/streaming-data>
- On-chain validation docs: <https://txline.txodds.com/documentation/examples/onchain-validation>
- Devnet examples branch: <https://github.com/txodds/tx-on-chain/tree/nojira-re-adding-examples>
- PR #3: <https://github.com/txodds/tx-on-chain/pull/3>
- PR #4: <https://github.com/txodds/tx-on-chain/pull/4>
