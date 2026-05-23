# Claim 019: Extraction Window Robustness (Replay Stability Retest)

**Status:** Provisional (Phase 2E-M Evidence)
**Date:** 2026-05-23
**Claim Type:** Epistemic Method Validation / Robustness

## Rule 12 Replay Metadata Snapshot
- **Session Type:** Bounded (Continuous intraday session)
- **Replay Class:** Replay Stability Retest (Perturbed Extraction Windows)
- **Authority Type:** Composite (Yahoo Presentation)
- **Chronology Density:** 5m Presentation

## Core Assertion
Under a systematic temporal perturbation of the extraction window (-10m, +10m, +20m) for the previously tracked Lateral Exhaustion Divergence pair (Claim 017), the observed persistence geometry remained locally stable. NVDA consistently preserved continuity, while AMD consistently fractured. This proves that the observed divergence is robust against minor chronological bounding shifts and is not an artifact of a brittle, overfit 60-tick extraction cutoff.

## Evidence Base
1. **Lateral Exhaustion Divergence Perturbations:**
   - **Original Window:** NVDA (38), AMD (25).
   - **-10m Shift (-2 ticks):** NVDA (38), AMD (25).
   - **+10m Shift (+2 ticks):** NVDA (38), AMD (28).
   - **+20m Shift (+4 ticks):** NVDA (39), AMD (29).
   - **Interpretation:** NVDA's preservation geometry (38-39) and AMD's fracture geometry (25-29) did not invert or collapse across a 30-minute sliding chronological offset. The core contradiction pairing remained intact.

## Scientific Conclusion
The empirical correspondence pairs documented within the bounded composite 5m semiconductor family demonstrate window-local robustness. The geometry is sensitive to internal structural perturbations (like volume collapses or reversals) but stable against minor temporal offsets of the extraction boundary itself. This validates the 60-tick bounding methodology as a stable framework for isolating continuity.
