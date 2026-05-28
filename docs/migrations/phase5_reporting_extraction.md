# Phase 5 Migration: `reporting.rs` Extraction

## Authority Justification
Extracting semantic formatting and reporting projections from `pipeline/old.rs`. This zone transforms already-certified semantic truth into representational form. It performs zero causal decision-making, infers no new semantics, and mutates no replay truth.

## Extracted Symbols
- `MetricAggregation`
- `AssetResult`
- `StrategyEvaluationDto`
- `SignalMeta`
- `SignalsSnapshot<T>`
- `EdgeLossReason`
- `EdgeTransfer`
- `EdgeLossBreakdown`
- `ReasonLossShare`
- `ThresholdSweepRow`

## Known Unchanged Invariants
- `pipeline/old.rs` continues to handle all orchestration, aggregation, sweeps, and routing.
- Replay equivalence hashes remain perfectly intact.

## Certification Hashes
- **Pre-Migration Replay Hash**: Verified by `golden_replay` CI suite (Baseline Tag)
- **Post-Migration Replay Hash**: Verified by `golden_replay` CI suite (Post-Extraction)
- **Equivalence Status**: IDENTICAL (CI passed `cargo test replay --release -- --test-threads=1`)
