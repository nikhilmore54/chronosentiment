# Capability: Runtime Replay

**Owner**: `financial/core`
**Consumers**: `financial/strategies`

## Description
Maintains market chronology, processes raw event streams, and provides the fundamental simulation substrate.

## Invariants
- Strictly monotonic timestamps.
- Deterministic execution causality (same events = same fill/latency/state).
- No orchestration logic or strategy semantic interpretations.

## Forbidden Dependencies
- `chronosentiment_strategies`