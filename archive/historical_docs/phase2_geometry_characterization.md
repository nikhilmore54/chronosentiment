# Phase 2 Geometry Characterization

**Date:** 2026-06-03  
**Data:** Q1 (120 sessions) and Q2 (120 sessions), k=2 Ward partition  
**Labels aligned:** Q2 labels matched to Q1 by centroid proximity  

---

## 1. Metric Contribution Analysis

> Which metrics actually separate the two ecologies?

### Effect sizes (Cohen's d) per metric

| Metric | Q1 Cohen's d | Q2 Cohen's d | Direction |
|--------|-------------|-------------|-----------|
| **net_return_pct** | **-2.862** | **-2.820** | SAME ✅ |
| **session_range_pct** | **-2.537** | **-1.591** | SAME ✅ |
| **trend_strength** | **-1.945** | **-2.554** | SAME ✅ |
| realized_volatility | -0.964 | -0.606 | SAME ✅ |
| gap_pct | -0.435 | 0.135 | FLIPPED ⚠️ |

### Interpretation

**Three metrics drive the partition consistently:**

```
net_return_pct        d ≈ -2.8 (both quarters)  — strongest separator
trend_strength        d ≈ -2.0 to -2.6          — strong separator
session_range_pct     d ≈ -1.6 to -2.5          — strong separator
```

**One metric is moderate but consistent:**

```
realized_volatility   d ≈ -0.6 to -1.0          — moderate separator
```

**One metric contributes almost nothing:**

```
gap_pct               d ≈ -0.4 (Q1) / +0.1 (Q2) — noise, direction flipped
```

> [!IMPORTANT]
> `gap_pct` is the only metric whose separation direction flipped between Q1 and Q2. Its effect size is near zero in Q2 (d = 0.135). This metric does not contribute to the stable ecological geometry and may be a candidate for exclusion in future work.

### What the ecologies are (geometrically, not named)

```
Ecology A (negative d):
  Lower net_return, lower range, lower trend, lower volatility
  → "Quiet" sessions

Ecology B (positive d):
  Higher net_return, higher range, higher trend, higher volatility
  → "Active" sessions
```

---

## 2. Centroid Stability

> Are the ecologies in the same location across quarters?

### Centroid coordinates (standardized)

| Metric | Q1 Eco-A | Q2 Eco-A | Q1 Eco-B | Q2 Eco-B |
|--------|----------|----------|----------|----------|
| realized_volatility | -0.224 | -0.258 | 0.672 | 0.327 |
| trend_strength | -0.374 | -0.701 | 1.121 | 0.886 |
| gap_pct | -0.108 | 0.060 | 0.324 | -0.076 |
| session_range_pct | -0.429 | -0.554 | 1.286 | 0.701 |
| net_return_pct | -0.451 | -0.726 | 1.352 | 0.918 |

### Similarity measures

| Property | Value |
|----------|-------|
| Ecology A cosine similarity (Q1 ↔ Q2) | **0.969** |
| Ecology B cosine similarity (Q1 ↔ Q2) | **0.969** |
| Separation vector cosine similarity | **0.969** |
| Ecology A Euclidean drift | 0.477 |
| Ecology B Euclidean drift | 0.930 |

> [!TIP]
> A cosine similarity of **0.969** for the separation vector means the partition "cuts the space" in nearly the same orientation in both quarters. The ecologies are not just statistically significant — they are geometrically the same partition.

### Separation vectors (A − B)

```
Q1: [-0.90, -1.50, -0.43, -1.71, -1.80]
Q2: [-0.59, -1.59,  0.14, -1.26, -1.64]
```

The vectors are nearly parallel (cosine 0.969). The Q2 separation is slightly smaller in magnitude (centroids closer together), but the direction is preserved.

---

## 3. Variance Structure

> Is the partition explaining the same proportion of variance?

### Variance ratio (R²) per metric

| Metric | Q1 R² | Q2 R² |
|--------|-------|-------|
| net_return_pct | **0.610** | **0.666** |
| session_range_pct | **0.551** | 0.388 |
| trend_strength | 0.419 | **0.621** |
| realized_volatility | 0.150 | 0.084 |
| gap_pct | 0.035 | 0.005 |

| | Q1 | Q2 |
|--|-----|-----|
| **Overall R²** | **0.353** | **0.353** |

> [!IMPORTANT]
> The overall R² is **exactly 0.353 in both quarters**. The k=2 partition explains the same total proportion of variance in Q1 and Q2. This is a striking invariant.

### Per-ecology variance (std of standardized features)

| | Ecology A | Ecology B |
|--|-----------|-----------|
| **Q1** | [0.88, 0.73, 1.02, 0.66, 0.54] | [1.04, 0.85, 0.85, 0.71, 0.83] |
| **Q2** | [0.73, 0.39, 0.66, 0.57, 0.29] | [1.19, 0.82, 1.30, 0.98, 0.81] |

**Finding:** Ecology A is consistently **tighter** (lower variance). Ecology B is consistently **noisier**. This asymmetry is preserved across quarters.

---

## 4. Transition Behavior

> Do ecologies persist or alternate?

### Transition matrices

**Q1** (n=118 transitions)

|  | → A | → B |
|--|-----|-----|
| A → | **0.559** | 0.186 |
| B → | 0.203 | 0.051 |

**Q2** (n=118 transitions)

|  | → A | → B |
|--|-----|-----|
| A → | **0.288** | 0.271 |
| B → | 0.280 | 0.161 |

### Persistence probabilities

| | P(A→A) | P(B→B) |
|--|--------|--------|
| Q1 | **0.750** | 0.200 |
| Q2 | **0.515** | 0.365 |

### Interpretation

- **Ecology A is persistent** — once in a "quiet" regime, sessions tend to stay quiet. This is stronger in Q1 (75%) than Q2 (52%), but still above 50% in both.
- **Ecology B is transient** — "active" sessions rarely persist. In Q1, P(B→B) = 0.20. In Q2 it rises to 0.37, but is still below majority.
- **Q2 has more frequent ecology switching** — transitions are more evenly distributed. This may reflect a more volatile market regime in Q2 (April crash, recovery).

> [!WARNING]
> Transition dynamics are the **least stable** property across quarters. The persistence pattern exists in both, but its strength varies. This may limit the predictive value of simple "Ecology(t) → Outcome(t+1)" models.

---

## 5. Run-Length Analysis

> How long do ecological states persist?

| | Ecology A runs | Ecology B runs |
|--|----------------|----------------|
| **Q1** | 24 runs, mean=3.8, median=2.0, max=13 | 24 runs, mean=1.2, median=1.0, max=3 |
| **Q2** | 33 runs, mean=2.0, median=2.0, max=6 | 34 runs, mean=1.6, median=1.0, max=4 |

### Interpretation

- Ecology A shows **multi-session persistence** (runs of 2–13 in Q1, 2–6 in Q2).
- Ecology B is mostly **single-session spikes** (median run length = 1 in both quarters).
- This is consistent with the "quiet baseline / active disruption" interpretation: the market spends most of its time in a quiet state, with occasional volatile sessions that don't persist.

---

## 6. Outlier Relationship

> Where do extreme sessions sit?

### Q1: No sessions exceed 3σ from centroid

### Q2: 4 sessions exceed 3σ

| Date | Symbol | Ecology | Distance from centroid |
|------|--------|---------|----------------------|
| **2025-04-07** | **NIFTY** | **B** | **7.96σ** |
| **2025-04-07** | **BANKNIFTY** | **B** | **6.90σ** |
| 2025-04-15 | NIFTY | A | 2.88σ |
| 2025-04-15 | BANKNIFTY | A | 3.02σ |

### Interpretation

- The April 7 crash sessions are **extreme members of Ecology B** (the "active" ecology), not a separate third ecology. They sit at 7–8σ from the Ecology B centroid — very far, but on the same side of the partition.
- The April 15 sessions (recovery rally) are **borderline outliers of Ecology A** (~3σ). These are "quiet" sessions that were slightly unusual.
- No Q1 outliers were detected at the 3σ threshold, consistent with Q1 being a calmer market period.

---

## 7. Stable Geometric Findings

These properties were **invariant** across Q1 and Q2:

| Property | Q1 | Q2 | Verdict |
|----------|-----|-----|---------|
| Separation direction | [-0.90, -1.50, -0.43, -1.71, -1.80] | [-0.59, -1.59, 0.14, -1.26, -1.64] | **Cosine 0.969** |
| Variance explained (R²) | 0.353 | 0.353 | **Identical** |
| Dominant separators | net_return, range, trend | net_return, trend, range | **Same 3 metrics** |
| Ecology A tighter than B | ✅ | ✅ | **Preserved** |
| Ecology A more persistent | ✅ (P=0.75) | ✅ (P=0.52) | **Preserved (weaker)** |
| Ecology B transient | ✅ (P=0.20) | ✅ (P=0.37) | **Preserved** |
| gap_pct contribution | Negligible | Negligible | **Stable non-contributor** |

These properties **varied** across quarters:

| Property | Q1 | Q2 | Note |
|----------|-----|-----|------|
| Cluster sizes | 90/30 | 67/53 | Q2 more balanced |
| Persistence strength | Strong A | Moderate A | Weaker in Q2 |
| Transition frequency | Low | High | More switching in Q2 |
| Outlier presence | None | 4 sessions | Market-event driven |

---

## 8. Open Questions

1. **Is gap_pct useful at all?** It contributes R² ≈ 0.005–0.035 and its direction flipped. Removing it would make the ecological coordinates a 4-metric system. Worth testing.

2. **Why is Ecology B transient?** Single-session "active" spikes that revert to "quiet" the next session. Is this a fundamental property of the market, or an artifact of the daily resolution?

3. **What drives the Q1→Q2 persistence difference?** Is it the market regime (Q2 was more volatile due to the April crash), or a genuine structural change?

4. **Are the stable coordinates sufficient for prediction?** The separation vector is preserved (cosine 0.969), so ecological position could serve as a conditioning variable for next-session outcome prediction — even without knowing which quarter you're in.

5. **What happens at intra-day resolution?** The current analysis uses daily sessions. Ecological coordinates might be even more informative at 5-minute or hourly granularity.

---

## Implication for Prediction

The characterization reveals that the simplest predictive experiment would be:

```
ecological_position(t)  →  net_return(t+1)
```

where `ecological_position` is the projection onto the stable separation vector:

```
w = [-0.90, -1.50, -0.43, -1.71, -1.80]  (Q1)
    [-0.59, -1.59,  0.14, -1.26, -1.64]  (Q2)
```

This single scalar captures most of the ecological information. If it carries incremental predictive power for next-session returns, the phenomenon has practical utility beyond existence.
