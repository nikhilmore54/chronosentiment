# LIVE CAPTURE ISOLATION CONTRACT v1

## 1. Axiomatic Purpose
The Live Capture Layer exists solely to function as a **scientific sensor**. It captures the physical truth of the market and converts it into a verifiable, mathematically bound chronology universe. 
**It is mechanically incapable of interpretation.**

## 2. Prohibited Behaviors (The Anti-Seduction Clause)
The Capture Daemon and any related ingestion architectures are strictly prohibited from implementing or executing:
- Topology inference, classification, or detection during capture.
- Morphology computation (Occupancy Traces, Transition Density) during capture.
- Continuity scoring or regime ranking.
- Adaptive throttling based on market conditions.
- Live replay mutation or backtesting callbacks.
- Inline observability generation (no streaming charts, no live dashboards).
- Machine Learning enrichment, prediction, or recommendation generation.

*Violating any of these prohibitions immediately downgrades the observatory from a scientific instrument to an ordinary adaptive trading system.*

## 3. Asymmetrical Truth Separation
The system recognizes two distinctly isolated domains of physical truth:
- **`capture_hash`**: Secures raw market chronology integrity.
- **`artifact_hash`**: Secures deterministic replay artifact integrity.

These hashes must never merge, overlap, or mutually influence one another. The `capture_hash` is computed on the raw sequence; the `artifact_hash` is computed on the topological deformation resulting from the replay.

## 4. Immutable Raw Preservation
Chronology substrates must be preserved sequentially without destruction of raw data:
1. **`raw_capture/`**: Unadulterated websocket payloads, original exchange sequencing, and raw timestamps.
2. **`normalized_capture/`**: Float-casted, normalized `NormalizedTick` schemas ready for deterministic replay ingestion.
3. **`replay_artifacts/`**: The downstream traces and morphological manifestations.

## 5. Canonical Capture Schema Freeze
To prevent historical corruption over longitudinal archives, the capture schema is frozen. Future modifications require formal migration protocols.
- **Symbol Normalization:** Lowercase, pair-continuous (e.g., `btcusdt`).
- **Timestamp Semantics:** Strict Unix Epoch milliseconds.
- **Precision Rules:** Unbounded standard floats for prices and volumes.
- **Serialization:** Strict JSONL (one event per line).
- **Ordering:** Strictly append-only in order of network arrival.

**Status:** IN EFFECT.
**Enforcement:** Mandatory for all core contributors.
