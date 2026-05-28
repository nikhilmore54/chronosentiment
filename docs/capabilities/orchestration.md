# Capability: Orchestration

**Owner**: `financial/strategies/src/pipeline/orchestration.rs`
**Consumers**: `infrastructure/observatory/api`

## Description
The outer application loop. Coordinates asset loops, sweeps, and aggregates metrics.

## Invariants
- Never modifies underlying market chronological truth.
- Output snapshots must be identical under the same configuration.

## Forbidden Operations
- Re-implementing the GA engine or tick-level execution semantics.