# Security

This document describes the SDK security boundaries. Use
[SECURITY.md](../SECURITY.md) for vulnerability reporting.

## Secrets

The SDK accepts caller-provided guest JWTs, activated API tokens, wallet
signatures, and Solana signers. It does not manage private keys, seed phrases,
or durable secret storage.

`GuestJwt`, `ApiToken`, and `AuthHeaders` redact their `Debug` output. Do not
log raw `HeaderMap` values, request bodies, private keys, seed phrases, detached
wallet signatures, or complete tokens.

## Activation

The SDK centralizes the activation message:

```text
${txSig}:${selectedLeagues.join(",")}:${jwt}
```

Empty league lists produce:

```text
${txSig}::${jwt}
```

The wallet signature must come from the wallet that submitted the Devnet
`subscribe` transaction.

## RPC Endpoints

`TxlineConfig::devnet().with_rpc_url(...)` keeps the TxLINE program ID and mints
fixed to Devnet. Validation rejects empty and obvious mainnet-looking RPC URLs,
but caller-provided custom RPC endpoints still need operator review.

Before using a custom provider, verify that it is connected to Solana Devnet and
that it is acceptable for the data, rate limits, and availability assumptions of
your application.

## Purchase Quotes

The SDK can request a Devnet purchase quote, decode the returned transaction
bytes, and check the financial shape. It does not yet perform a full decoded
transaction audit.

Before signing paid quote transactions, inspect:

- fee payer,
- signer set,
- backend or admin signature,
- invoked program IDs,
- account metas,
- decoded TxLINE instruction,
- requested TxL amount.

## Streams

SSE clients send both credentials, preserve `Last-Event-ID`, and renew the guest
JWT on HTTP 401. Heartbeat events are filtered before typed JSON
deserialization. HTTP 403 is treated as an entitlement, token, expiry, or network
mismatch condition.

## Live Credentials

Default tests must not require real Devnet credentials. Live examples should be
run only when the required environment variables are present, and results should
state whether live validation actually ran.
