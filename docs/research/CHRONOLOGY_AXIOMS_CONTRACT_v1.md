# CHRONOLOGY AXIOMS v1

## The Immutable Principles of the Observatory

This contract defines the architectural constitutional law of the ChronoSentiment execution observatory. These axioms are designed to prevent semantic leakage, interpretive recursion, and un-falsifiable application drift. They must be defended against all downstream pressures.

### 1. Replay is Authoritative
The engine is a chronological state machine, not a prediction oracle. The deterministic sequence of ticks, state evaluations, and orders defines the absolute truth of the universe. If a behavior cannot be proven in a deterministic replay trace, it does not exist.

### 2. Chronology is Immutable
Topology modulates observation, but it never mutates the underlying execution history. The flow of time and the existence of price boundaries cannot bend to suit the interpretation.

### 3. Observability is Downstream-Only
The core generates traces; it does not read them. Observability must strictly follow:
`Replay → Topology → Cognition → Morphology Trace → Interpretation`.
Data never flows backward.

### 4. Artifacts are Canonical
The `OccupancyTrace` and `TraceArtifact` are cryptographically bound chronological witnesses. They act as the physical substrate for all further analysis. If analysis requires recomputing state overlap independently of the trace artifact, the architecture is broken.

### 5. Semantics Cannot Mutate Replay
Interpretive abstractions (e.g., "resilience", "fragile basins", "stable continuity") are downstream labels used by the observer. They must NEVER be used as control flow conditions inside the replay engine or topology modulators.

### 6. Determinism Outranks Optimization
Replay equivalence and state coherence are the highest priorities. If an adaptive layer or strategy enhancement breaks determinism, it must be removed. We do not optimize profitability at the cost of epistemic stability.

### 7. Interpretation is Non-Authoritative
Phase portraits, entropy graphs, and topology maps are projections. They are useful for understanding the deformation geometry, but they do not discover new physical ontology. Never embed visualization geometry back into the core mechanics.

### 8. Discontinuity is Truth
**Missing chronology is itself chronology.** Disconnects, packet losses, and outages are physical facts of the market timeline. The architecture must never silently interpolate, smooth, or rewrite continuity gaps. Discontinuity must be explicitly recorded and exposed to the downstream trace artifacts.

## Drift Enforcement Gates
Before any new logic is merged into the Rust core or Python analytical layer, it must pass the four enforcement gates:
- Gate 1: Does it alter chronology? (If yes, requires extreme scrutiny)
- Gate 2: Does it create backward semantic influence? (If yes, REJECT)
- Gate 3: Is it observability or interpretation? (Prefer observability; isolate interpretation)
- Gate 4: Can it be replay-falsified? (If no, it does NOT belong in the laboratory)
