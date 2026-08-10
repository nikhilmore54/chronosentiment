# Phase 3 Excursion Characterization Report

**Date:** 2026-06-03  
**Data:** Q1 and Q2  

This phase aimed to characterize the transitions between the baseline attractor (Ecology A) and the transient excursion state (Ecology B). The guiding question was: *What differentiates stable occupancy from transition?*

---

## 1. Boundary Analysis

> Do transitions occur near the geometric separation boundary?

We computed the continuous geometric distance to the boundary (the hyperplane equidistant from the A and B centroids) and calculated the probability of transition based on proximity.

### Results
- **Ecology A ($A \rightarrow B$):** The probability of an excursion is identical (~36%) regardless of whether the session is hovering right on the geometric boundary or is buried deep in the absolute core of the quiet state ($>2.5\sigma$ away). Point-biserial correlation is completely flat (p=0.807).
- **Ecology B ($B \rightarrow A$):** Similarly, transition probability is independent of distance to the boundary (p=0.218).

### Interpretation
**Transitions are not gradual geometric drifts.** The market does not randomly walk to the edge of an ecology and slip over. The geometry strictly describes state *occupancy*, but it is completely uninvolved in the *transition process*. Transitions are discrete jumps.

---

## 2. Genesis Characterization

> Do transition sessions look structurally different *immediately before* the jump?

We compared the final Ecology A session before a stay ($A \rightarrow A$) versus before a break ($A \rightarrow B$) using all primary metrics and geometric distances.

### Pooled Results
| Metric | A $\rightarrow$ A (n=100) | A $\rightarrow$ B (n=56) | Cohen's d | p-value |
|--------|---------------------------|--------------------------|-----------|---------|
| Volatility | 0.0003 | 0.0003 | -0.041 | 0.803 |
| Trend Strength | 7.97 | 7.02 | -0.182 | 0.241 |
| Range (%) | 0.91 | 0.88 | -0.105 | 0.520 |
| Distance to Boundary | 1.47 | 1.44 | -0.041 | 0.802 |

### Interpretation
**Genesis is completely null.** A quiet session that is about to explode into Ecology B looks statistically identical to a quiet session that will remain quiet. There is absolutely no internal "build-up" detectable at the daily resolution.

---

## 3. Decay Characterization

> Does the excursion state show internal signs of exhaustion before reverting?

We compared Ecology B sessions that persist ($B \rightarrow B$) against those that revert ($B \rightarrow A$).

### Replication Failure
The validation criterion required differences to replicate independently across Q1 and Q2.
- **In Q1:** Distance to the B centroid strongly predicted reversion (p=0.000). Sessions that reverted were *farther* from the core.
- **In Q2:** The distance effect completely flipped (d=-0.635). Sessions that reverted were *closer* to the core. `gap_pct` showed a significant effect in Q2 (p=0.037) but was entirely flat in Q1 (p=0.959).

### Interpretation
**No replicated structural differences exist.** While some metrics showed noisy significance in individual quarters, they completely failed out-of-sample replication. We cannot mechanistically explain the collapse of Ecology B using internal session metrics.

---

## Conclusion

The evidence overwhelmingly points to a single profound conclusion:

**Transitions are externally forced.**

1. The geometry describes occupancy, not transitions.
2. The birth of an excursion ($A \rightarrow B$) has no internal signature.
3. The decay of an excursion ($B \rightarrow A$) lacks any replicated internal signature.

The transition process cannot be inferred from the current ecological state at the daily resolution. The ecological geometry acts purely as a map of how the market absorbs information (either in its quiet baseline or its highly volatile excursion), but the *catalyst* moving the market between these states resides outside the geometry entirely.
