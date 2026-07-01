# Validation

`/api/scores/stat-validation` is implemented in both published modes.

## Legacy

Legacy mode uses:

```text
fixtureId=...&seq=...&statKey=...
fixtureId=...&seq=...&statKey=...&statKey2=...
```

The SDK returns `ScoresStatValidation` with `statToProve`, `statProof`, and
optional second-stat fields.

## V2

V2 mode uses:

```text
fixtureId=...&seq=...&statKeys=1001,1002,1007,2007
```

Requested stat key order is preserved in `ScoresStatValidationV2`. Strategy
indices refer to that preserved order. The SDK checks:

- `seq > 0`,
- every proof hash decodes to exactly 32 bytes,
- `statsToProve.len() == requested_stat_keys.len()`,
- `statProofs.len() == statsToProve.len()`.

## Strategy Builder

`NDimensionalStrategy::builder(stat_count)` supports:

- single-stat predicates by index,
- binary predicates using add/subtract,
- geometric targets,
- distance predicates.

The builder rejects out-of-bounds indices before a malformed strategy can be
submitted.

## Sequence Source

`seq` must come from a real score record from snapshot, updates, historical
scores, or the scores stream. Do not use `seq=0`, fixture IDs, array positions,
or synthetic sequence numbers.
