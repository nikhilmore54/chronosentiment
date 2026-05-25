# EXECUTION ECOLOGY SPECIFICATION v1

## 1. Core Mandate
This specification governs the classification of Alpha strategies under adversarial observability conditions. ChronoSentiment does not measure "good" vs "bad" Alpha strictly by PnL. It measures **Ecological Survivability**: the degree to which a strategy's intents remain causally stable and mechanically executable across fractured topologies.

## 2. Ecological Metrics (Strategy Fingerprinting)
Every Alpha strategy must be mapped against these five invariants across the 5 synthetic topologies (Uniform, Bimodal, Wave, Anticipatory, Collapse):
1. **Synchronization Sensitivity**: The rate at which the strategy's intents are suppressed when strict simultaneity (`strict_ratio >= 0.9`) decays.
2. **Fragmentation Tolerance**: The percentage of generated intents that survive execution gating under `FRAGMENTED_BUT_USABLE` states. 
3. **Recovery Survivability**: Measurement of intent stability during `TRANSITIONAL_RECOVERY`. Does the strategy over-fire when staggered waves re-synchronize?
4. **Collapse Resilience**: The strategy's ability to maintain a mathematically invariant intent sequence when the environment hits `DEGRADED_OBSERVABILITY`.
5. **Replay Stability**: The ability for the Strategy + Environment intersection to produce identical **Replay Artifact Hashes** across equivalent topologies.

## 3. Topology Stress Taxonomies
All strategies must be tested against the formal perturbations defined in the Topology Injector:
*   `uniform_delay`: Stresses base chronological staleness.
*   `bimodal`: Stresses the variance absorption of the admissibility governor.
*   `rolling_wave`: Stresses synchronization half-life.
*   `anticipatory`: Stresses negative lag handling.
*   `collapse`: Stresses the execution suppression mechanics.

## 4. Prohibited Semantic Reinterpretations (Ecological Mythology)
*   **"Surviving fragmentation means the Alpha is superior."** FALSE. It merely indicates the Alpha's execution timeframe is wider than the fragmentation variance. It is a property of scale, not superiority.
*   **"The strategy is predicting the transport wave."** FALSE. The strategy is blind to transport. The execution layer simply survived the wave.
*   **"We should tune the strategy to fire only during synchronized states."** FALSE. That couples Alpha to Observability. If execution is only safe during synchronization, the Admissibility Governor will block it; the Alpha must not self-censor based on environmental metadata.

## 5. Survivability Scoring Boundaries
A strategy is considered "Ecologically Viable" if and only if:
1. Intent generation remains strictly identical across all 5 synthetic topologies.
2. The Replay Artifact Hash fractures precisely where admissibility blockades are inserted.
3. No hidden queueing or deferred execution occurs after a `DEGRADED_OBSERVABILITY` block is lifted.
