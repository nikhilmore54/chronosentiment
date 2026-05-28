# Capability: Signal Generation

**Owner**: `financial/strategies/src/signals`
**Consumers**: `financial/strategies/src/pipeline`

## Description
Generates, scores, rejects, and normalizes actionable `TradeSignal` outputs from candidate evaluations.

## Invariants
- Must map edge loss deterministically to semantic rejection reasons (e.g. `LowConfidence`, `SidewaysMarket`).
- Generates deterministic `SignalMeta` properties without corrupting pure replay traces.