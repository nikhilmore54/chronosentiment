# STATE COHERENCE CONTRACT v1

## 1. Core Mandate
This document governs the scientific isolation between topological fragmentation and internal strategy state (memory). As strategies transition from Phase 1 (stateless) to Phase 2 (stateful), memory structures must remain deterministic, causally auditable, and replayable.

## 2. Allowed Memory Structures
Stateful strategies (e.g., rolling windows, persistence detectors) must employ fixed-length, immutable memory buffers. 
- Memory must be constructed strictly from canonical historical candles.
- Accumulation logic must be deterministic and mechanically defined.

## 3. Forbidden Adaptive Recovery Behaviors
- **State Interpolation**: Strategies may not "guess" or interpolate missing states during `DEGRADED_OBSERVABILITY`.
- **Adaptive Truncation**: Rolling windows may not shrink dynamically in response to fragmentation.
- **Topology-Coupled State**: Strategy memory MUST NOT read the `admissibility_reason` to decide whether to update its internal buffer. State accumulation must be blind to observability metadata.

## 4. Replay State Equivalence
During any replay traversal, the strategy's internal memory state at barrier `N` must reconstruct perfectly identically to the live run, provided the underlying chronological timeline is preserved. A strategy's memory may only mutate if the chronological substrate itself mutates, never due to the transport topology that governs admissibility.

## 5. Cognitive Divergence Detection
Statefulness introduces the possibility of legitimate **Cognitive Divergence**, where topology indirectly alters accumulated state (e.g., due to missing or delayed data in the true underlying timeline). 
We strictly enforce a causal separation in reporting:
- `execution_blocked`: The environment suppressed the intent (Physics Intervention).
- `intent_sequence_diverged`: The environment distorted the internal state accumulation (Cognitive Divergence). 
Cognitive divergence is a measurable ecological property, not necessarily an experimental failure.
