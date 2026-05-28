# Capability: Optimization

**Owner**: `infrastructure/optimization`
**Consumers**: `financial/core` (for types), `financial/strategies` (for evolution invocation)

## Description
Mechanically evolves mathematical candidates using a pure Genetic Algorithm.

## Invariants
- Zero financial awareness.
- Absolute structural determinism (identical seed = identical population and crossover outcomes).
- Mathematical purity in fitness ranking and selection.

## Forbidden Dependencies
- `chronosentiment_strategies`
- `chronosentiment_financial_core`
- Any domain vocabulary (`PnL`, `Regime`, `Trade`, `Bull`, `Bear`).