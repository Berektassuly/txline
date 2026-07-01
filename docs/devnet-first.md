# Devnet First

The crate supports TxLINE Devnet only.

## Values

| Value | Devnet |
| --- | --- |
| Solana RPC | `https://api.devnet.solana.com` |
| API base | `https://txline-dev.txodds.com/api` |
| Guest JWT | `https://txline-dev.txodds.com/auth/guest/start` |
| Program ID | `6pW64gN1s2uqjHkn1unFeEjAwJkPGHoppGvS715wyP2J` |
| TxL mint | `4Zao8ocPhmMgq7PdsYWyxvqySMGx7xb9cMftPMkEokRG` |
| USDT mint | `ELWTKspHKCnCfCiCiqYw1EDH77k8VCP74dK9qytG2Ujh` |

## Guardrails

- `Network` has only `Devnet`.
- There is no `mainnet` feature.
- Config validation rejects mixed non-RPC values.
- Examples require Devnet env vars and do not contain secrets.
- `seq` is rejected when it is zero or negative before a validation request is sent.

Free tiers do not require TxL payment, but still require SOL for Solana fees and possible rent.
