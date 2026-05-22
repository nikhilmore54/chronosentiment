# REPAIR LABORATORY ISOLATION PROTOCOL v1

## 1. Constitutional Authority
This protocol governs the introduction, execution, and boundary maintenance of **Adaptive Memory Repair** within the ChronoSentiment Execution Laboratory. Adaptive repair introduces synthetic chronology into stateful memory, posing the highest drift risk to the architecture. Therefore, it is strictly quarantined.

## 2. The Two Universes
The system now formally recognizes two distinct execution universes that must never bleed into one another:

### Universe A: Canonical Ecology (The Baseline)
* **Rule**: NO REPAIR EVER.
* **Function**: To measure raw, unmitigated topology deformation against pristine stateful memory architectures.
* **Requirement**: Chronological discontinuity must cleanly propagate into cognition, causing deterministic Memory Coherence Index (MCI) fracture.

### Universe B: Synthetic Repair Ecology (The Laboratory)
* **Rule**: Controlled, opt-in experimental repair only.
* **Function**: To test the efficacy of continuity restoration policies against known topological scars.
* **Requirement**: Repair mechanisms operate exclusively within a parallel isolation layer, never mutating the underlying timeline.

## 3. Strict Boundary Rules

### Rule 1: Immutable Chronological Truth
Repair algorithms may NOT overwrite raw market chronology, synthesize false ticks in the ledger, or permanently alter the primary dataset. The base ledger remains mathematically sacred.

### Rule 2: Parallel Reconstruction Layer
When adaptive repair is engaged, it must occur inside a `Synthetic Reconstruction Window`. The canonical window (`window_state_live`) and the natively fragmented window (`window_state_fragmented`) must remain untouched, allowing us to compute a three-way divergence:
1. `Canonical` vs `Fragmented` (Baseline MCI)
2. `Canonical` vs `Repaired` (Restoration MCI)

### Rule 3: Separate Replay Hash Spaces
Any strategy implementing adaptive repair must be placed in a distinct namespace (e.g., `rolling_window_momentum_v3_adaptive`). It must generate its own Replay Artifact Hash. We will never evaluate an adaptive strategy without comparing it against its canonical counterpart to measure the precise efficacy of the repair.

### Rule 4: Repair is Not "Truth"
A repaired intent is NOT a more "accurate" intent. It is simply a synthetic intent derived from hallucinated interpolation. All repairs must log their exact interpolation methodology (e.g., "last-value-carry-forward", "linear-interpolation", "confidence-weighted-decay") in the state divergence trace.

## 4. Required Observability
Before any repair strategy is promoted to an Execution Ecology Fingerprint, the laboratory must expose:
1. **Repair Efficacy Score**: How much of the Canonical MCI was restored?
2. **False Hallucination Rate**: How often did the repair logic hallucinate a false state that diverged *further* from the Canonical truth than the raw Fragmented state?
3. **Repair Destabilization Factor**: Did the interpolation cause oscillatory intent generation that would not have occurred otherwise?

## 5. Violations
Any code change that allows adaptive repair logic to silently patch the primary data stream, mutate the raw execution tape without explicit flagging, or obscure the fundamental topology severity index is considered a critical architectural violation and will be rejected.
