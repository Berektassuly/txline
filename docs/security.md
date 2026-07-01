# Security

## Secrets

The SDK accepts caller-provided guest JWTs, activated API tokens, wallet
signatures, and Solana signers. It does not manage private keys or persist
secrets.

`GuestJwt`, `ApiToken`, and `AuthHeaders` redact their `Debug` output. Do not log
raw `HeaderMap` values, request bodies, private keys, seed phrases, or detached
wallet signatures.

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

## Purchase Quotes

The SDK can request a Devnet purchase quote, decode the returned transaction
bytes, and check the financial shape. It does not yet perform a full decoded
transaction audit. Before signing paid quote transactions, inspect:

- fee payer,
- signer set,
- backend/admin signature,
- invoked program IDs,
- account metas,
- decoded TxLINE instruction,
- requested TxL amount.

## Streams

SSE clients send both credentials, preserve `Last-Event-ID`, and renew the guest
JWT on HTTP 401. HTTP 403 is treated as an entitlement, token, expiry, or network
mismatch condition.
