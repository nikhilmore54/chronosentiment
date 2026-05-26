# V-001 Ratification Decision

## Status

**Decision:** ACCEPTED - bounded reconstructed pair satisfies V-001 ratification
**Date:** 2026-05-26
**Baseline anchor:** `replay-governance-baseline-v1`
**Canonical authority:** `core/src/strategy_id.rs`
**Ratification substrate:** batch `003`, `live_manifest_7dac` vs `replay_equiv_manifest_7dac`
**Declared window:** `1778643900` to `1779181800`
**Comparator report:** `state_archive/batches/batch_003/runs/live_manifest_7dac/metadata/replay_equivalence_report.json`

## Decision

The bounded reconstructed batch `003` pair is accepted as the lawful replay
ratification substrate for V-001.

`core/src/strategy_id.rs` may move from `CRITICAL domain / TRANSITIONAL impl` to
`CRITICAL / STABLE` for strategy genome parsing authority.

This stability was achieved through governed normalization and replay-certified
authority consolidation, not through preservation of legacy parser plurality.

## Scope Accepted

The accepted certification scope is limited to:

- Batch `003`.
- Declared frozen-substrate window `1778643900` to `1779181800`.
- Reconstructed bounded labels:
  - `live_manifest_7dac`
  - `replay_equiv_manifest_7dac`
- Strategy identity authority migration under V-001:
  - edge-decay parser routing through `core/src/strategy_id.rs`
  - API parser routing through the compatibility adapter
  - historical underscore ID compatibility translation

The original non-overlapping `live` archive is excluded from ratification scope.
It remains historical evidence of why restoration was required, not the
certification substrate.

## Evidence Chain

V-001 ratification is supported by the following governed evidence:

- Parser evidence corpus: `fixtures/strategy_identity/`
- Current differential report: `fixtures/strategy_identity/differential_report.json`
- Edge-decay routing delta: `docs/governance/V001_EDGE_DECAY_ROUTING_DELTA.md`
- API admissibility doctrine: `docs/governance/V001_API_ADMISSIBILITY_CONTRACT.md`
- API routing delta: `docs/governance/V001_API_ROUTING_DELTA.md`
- Replay restoration contract: `docs/governance/REPLAY_ARCHIVE_RESTORATION_CONTRACT.md`
- Replay restoration plan: `docs/governance/REPLAY_ARCHIVE_RESTORATION_PLAN.md`
- Batch `003` window declaration: `docs/governance/REPLAY_WINDOW_DECLARATION_BATCH_003.md`
- Bounded comparator report:
  `state_archive/batches/batch_003/runs/live_manifest_7dac/metadata/replay_equivalence_report.json`

Comparator result:

```text
barriers_compared = 366
ts_missing_live = 0
ts_missing_replay = 0
ts_tick_mismatch = 0
ts_corridor_mismatch = 0
ts_symbol_mismatch = 0
replay_equivalence = true
substrate_hash_certified = true
live_substrate_hash = 5f463caa0583acc354eb75b6897fd1658a9df97586a0f1b987199adc63fcb434
replay_substrate_hash = 5f463caa0583acc354eb75b6897fd1658a9df97586a0f1b987199adc63fcb434
```

## Interpretation

Within the declared certified window, V-001 shows:

```text
bounded replay substrate restored
deterministic equivalence preserved
no undeclared parser-induced replay drift observed
```

The compatibility normalization for historical underscore IDs remains
replay-addressable without making underscore syntax canonical. Historical
visibility does not imply future canonical serialization authority.

## Non-Claims

This decision does not claim:

- The original non-overlapping `live` archive is replay-equivalent.
- The full current 516-interval candle root is the sealed certification
  substrate.
- Any replay surface outside the declared batch `003` window is certified by
  this decision.
- API error unification, price-scale consolidation, manifest schema redesign, or
  replay engine redesign is authorized.

## Authority Outcome

V-001 is ratified as:

```text
replay-certified authority consolidation
CRITICAL / STABLE
```

Future strategy identity parsing changes remain `CRITICAL`: they require replay
scope declaration and evidence under the same governance rules.
