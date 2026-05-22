# REPLAY EQUIVALENCE CONTRACT v1

## 1. The Core Invariant
The Replay Artifact Hash is the constitutional law of the ChronoSentiment execution substrate. Any backtest, simulation, or historical experiment traversing a frozen chronological ledger must produce an identical cryptographic hash to the live session tape.

## 2. Experimental Corruption (Hash Fractures)
A fractured hash signifies a failure of causal isolation. The following conditions constitute experimental corruption:
- **Semantic Leakage**: An Alpha strategy alters its intent generation based on observability metadata (`regime_state`, `acceptance_ratio`). Intent generation must be strictly topology-invariant.
- **Hidden Queuing**: An admissibility blockade defers or buffers an intent rather than mechanically destroying it.
- **Chronology Mutation**: The ingestion sequence, barrier deduplication, or timeline fingerprint diverges between live capture and frozen replay.
- **Admissibility Drift**: The execution layer applies a different threshold boundary in replay than was applied in live execution.

## 3. Admissibility Replay Guarantees
Execution blockades are purely environmental. During any replay event:
- The execution layer must reproduce the exact `admissibility_reason` experienced by the live engine at that specific tick, mathematically derived from the raw step telemetry.
- The admissibility constraint must be strictly evaluated against the Alpha intent, blocking execution mechanically without alerting the Alpha logic to the environmental failure.

## 4. Valid Topology Perturbations
Controlled experimentation may inject synthetic topology (e.g., bimodal lag, uniform fragmentation) into the replay environment to observe execution survival. During such controlled perturbations:
- The final Replay Artifact Hash **will and must change** (because execution blockades will differ).
- The Alpha Intent sequence **must remain mathematically identical** (proving strategy cognition is environmentally neutral).
