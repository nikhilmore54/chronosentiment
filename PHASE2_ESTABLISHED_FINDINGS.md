# Phase 2 Established Findings

**Status:** FROZEN  
**Date:** 2026-06-03  

This document serves as the formal closure of Phase 2. It demarcates the mechanisms that survived rigorous replication and control testing from those that failed. Future phases must treat the "Established" findings as axiomatic and must not quietly re-litigate the "Not Established" findings.

---

## ✅ Established (Replicated & Validated)

1. **Stable Two-Ecology Partition**
   The daily sessions occupy a stable low-dimensional geometry consisting of exactly two ecological states.

2. **Geometric Invariance across Quarters**
   The geometry replicated identically across Q1 and Q2. The separation vector between the two ecologies maintained a cosine similarity of ~0.97. The partition explained identical total variance (R² = 0.353) in both independent periods.

3. **Methodological Robustness**
   The geometry is not an artifact of Ward clustering. Gaussian Mixture Modeling (GMM) recovered the same separation axis with even higher cross-quarter similarity (cosine 0.984).

4. **Dominant Metrics**
   The partition is driven entirely by displacement and intensity metrics: `net_return_pct`, `trend_strength`, and `session_range_pct`.

5. **Asymmetric Persistence**
   The two states exhibit fundamentally different survival curves:
   - **Ecology A (Baseline Attractor):** Persistent, long-lived, and characterized by a fat-tailed survival distribution.
   - **Ecology B (Transient Excursion):** Short-lived, fragile, with a hard duration ceiling. It is structurally unstable.

6. **Variance Asymmetry**
   Ecology B exhibits significantly higher intrinsic variance (2-3.5×) than Ecology A. Ward clustering artificially compressed this asymmetry, which GMM explicitly confirmed.

---

## ❌ Not Established (Falsified or Selection Artifact)

1. **Return Prediction**
   Ecological position does not provide straightforward linear predictive power for next-session directional returns.

2. **Intensity Prediction**
   Categorical ecological membership does not reliably determine next-session intensity (volatility or range). While a signal existed in Q1, it failed to replicate in Q2.

3. **Independent Run-Length Memory**
   While run-duration within Ecology A appeared to predict future volatility decay, a control regression revealed this was primarily a selection artifact. The duration effect loses significance when controlling for current-session volatility.

4. **Gap Significance**
   `gap_pct` does not structurally define the ecologies and does not predict transitions between them. It is orthogonal to both the geometry and its dynamics.

---

## 🔍 Open (The Domain of Phase 3)

The stable geometry discovered in Phase 2 successfully categorizes market intensity states and their distinct survival profiles, but lacks predictive mechanics. The open questions concern transition dynamics:

1. **Genesis of Ecology B**
   What measurable conditions immediately precede an $A \rightarrow B$ boundary crossing?

2. **Decay of Ecology B**
   Why is Ecology B structurally unstable, and what specific conditions trigger the $B \rightarrow A$ reversion?

3. **Boundary-Crossing Mechanics**
   How does the market behave dynamically as it approaches and crosses the geometric separation vector?

4. **Event Association**
   Are B-state entries exclusively associated with exogenous shocks (e.g., the April 7 crash), or do endogenous market dynamics generate them?
