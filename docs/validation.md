# Validation

`/api/scores/stat-validation` is represented in both published modes. The SDK
keeps network calls, response decoding, and on-chain payload preparation
separate so settlement-oriented code can review each step.

## Legacy

Legacy mode uses one or two stat keys:

```text
fixtureId=...&seq=...&statKey=...
fixtureId=...&seq=...&statKey=...&statKey2=...
```

The SDK returns `ScoresStatValidation` with `statToProve`, `statProof`, and
optional second-stat fields.

## V2

V2 mode accepts an ordered list of stat keys:

```text
fixtureId=...&seq=...&statKeys=1001,1002,1007,2007
```

Requested stat key order is preserved in `ScoresStatValidationV2`.
Settlement-oriented strategies refer to indices in that preserved order.

The SDK checks:

- `seq > 0`,
- every proof hash decodes to exactly 32 bytes,
- `statsToProve.len() == requested_stat_keys.len()`,
- `statsToProve[i].key == requested_stat_keys[i]`,
- `statProofs.len() == statsToProve.len()`.

## Strategy Builder

`NDimensionalStrategy::builder(stat_count)` supports:

- single-stat predicates by index,
- binary predicates using add or subtract,
- geometric targets,
- distance predicates.

The builder rejects out-of-bounds indices before malformed strategy data can be
submitted.

## Sequence Source

`seq` must come from a real score record from snapshot, updates, historical
scores, or the scores stream. Do not use `seq=0`, fixture IDs, array positions,
or synthetic sequence numbers.

## Testing

Regression tests cover:

- proof hash decoding,
- positive `seq` validation,
- V2 stat/proof length checks,
- V2 stat key order checks,
- strategy index bounds.
