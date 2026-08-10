# Observatory Claims Repository

This repository holds the scientific observations of the ChronoSentiment Observatory. It is governed by a strict epistemological structure designed to prevent mythology, unbounded generalization, and narrative-driven conclusions.

**Proof is never "I believe this pattern exists."**
Proof is: *"This exact replay artifact repeatedly emitted this bounded behavior under these explicit conditions."*

## The Epistemic Standard
Every claim within this repository MUST be formatted exactly as follows:

| Field | Requirement |
| --- | --- |
| **Claim** | What specific geometric or behavioral phenomenon is being asserted? |
| **Evidence** | Which exact chronological universes (`capture_hash`) and artifacts (`artifact_hash`) demonstrate this behavior? |
| **Boundary** | Under what exact conditions (substrate, resolution, topology version, cognition geometry) does this claim apply? |
| **Falsification** | What specific future replay result, timeline degradation, or structural observation would instantly invalidate this claim? |

## Claim Status Lifecycle
Claims move through the following directories based on their reproducibility:

- **`provisional/`**: Initial observations isolated to a small number of universes. High risk of representation bias.
- **`replicated/`**: Claims that have survived tests across multiple distinct chronology universes (e.g., across volatility expansions and weekend liquidity gaps) while holding their boundaries intact.
- **`unstable/`**: Claims that exhibit phase-dependence; they hold true in some universes but invert or collapse in others.
- **`falsified/`**: Claims that were formally broken by a new chronological universe or by a degradation test (e.g., failed to survive 1m aggregation).

Humility keeps the architecture clean. All claims, even those in `replicated/`, remain provisional, replay-bound, substrate-bound, and representation-bound.
