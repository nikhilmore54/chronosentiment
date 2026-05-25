# SIGNAL INTERFACE CONTRACT v1

## 1. Interface Boundary
This document governs the interaction between Layer 2 (Admissibility) and Layer 3 (Alpha / Signal Generation). It enforces the structural isolation of predictive logic from environmental observation.

## 2. Core Prohibitions
1. **Signals cannot mutate chronology**: Signal generation must not backfill data, adapt barriers, or modify the historical substrate.
2. **Signals cannot mutate admissibility policy**: A strategy cannot override or negotiate with the Execution Governor. If `new_entries_allowed` is `false`, the intent must be blocked mechanically by the execution pipeline.
3. **Signals cannot access observability narratives**: The Alpha engine is mathematically prohibited from consuming the `regime_state` string (e.g., `TRANSITIONAL_RECOVERY`). It may only consume explicit boolean constraints (e.g., `new_entries_allowed`) and quantitative limits (e.g., `acceptance_ratio`).
4. **Signal validity is orthogonal to observability topology**: A signal may be perfectly valid, yet execution may be denied due to environmental blindness. The two realities must not be conflated.

## 3. Intersection Logic
The pipeline MUST strictly follow this operational order:
1. **Admissibility**: Observability topology computes `new_entries_allowed`.
2. **Alpha**: Signal engine computes directional intents purely based on price physics.
3. **Execution Pipeline**: `Admissibility ∩ Intent → Executable Action`.

## 4. Replay Axiom
Replay environments must reconstruct the identical admissibility constraints using the canonical ledger. The Alpha engine in a backtest must experience the exact same execution denials that the live engine experienced at the exact same chronological barrier.
