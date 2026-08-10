# Phase 2 Mechanism Validation — Test Results

**Date:** 2026-06-03  
**Tests executed:** 3  

---

## Test 1: Ecological Position → Next-Session Return

### Hypothesis

```
ecological_position(t) → net_return(t+1)
```

If the ecological axis carries predictive information, sessions with higher displacement (Ecology B direction) should have systematically different next-session returns.

### Results

| Quarter | Symbol | Pearson r | p-value | E[ret\|low eco] | E[ret\|high eco] | Δ |
|---------|--------|-----------|---------|----------------|-----------------|-----|
| Q1 | NIFTY | +0.085 | 0.523 | 0.531 | 0.523 | -0.008 |
| Q1 | BANKNIFTY | -0.048 | 0.717 | 0.614 | 0.560 | -0.054 |
| **Q1** | **POOLED** | **+0.001** | **0.989** | **0.570** | **0.544** | **-0.026** |
| Q2 | NIFTY | +0.018 | 0.890 | 0.574 | 0.622 | +0.048 |
| Q2 | BANKNIFTY | +0.101 | 0.445 | 0.533 | 0.683 | +0.150 |
| **Q2** | **POOLED** | **+0.056** | **0.550** | **0.528** | **0.679** | **+0.151** |

### Verdict: **No predictive signal**

> [!IMPORTANT]
> The simple `ecological_position(t) → net_return(t+1)` experiment produces **no statistically significant result** in either quarter. All p-values exceed 0.4. The ecological structure is real but does **not** carry linear predictive information for next-day absolute returns.

### What this means

The ecological axis separates sessions by **displacement magnitude** (how much the session moved). But knowing today's displacement magnitude does not predict tomorrow's displacement magnitude in a simple linear way.

This is actually consistent with efficient market behavior — if ecological position trivially predicted next-day returns, it would be an arbitrageable anomaly.

### What it does NOT rule out

- Prediction of **volatility** (not return direction/magnitude)
- Prediction conditioned on **persistence** (run-length, not single-step)
- Prediction of **transition probability** (not return itself)
- Non-linear relationships
- Prediction at different time horizons

---

## Test 2: Gap → Transition Probability

### Hypothesis

```
gap_pct(t) → P(ecology transition at t)
```

Even though gap_pct doesn't separate the ecologies, it might predict when the market *switches* between them (shock-entry information).

### Results

| Quarter | Symbol | gap→trans r | p-value | \|gap\|→trans r | p-value |
|---------|--------|------------|---------|-------------|---------|
| Q1 | NIFTY | +0.216 | 0.100 | -0.032 | 0.811 |
| Q1 | BANKNIFTY | -0.020 | 0.878 | -0.027 | 0.839 |
| **Q1** | **POOLED** | **+0.072** | **0.437** | **-0.001** | **0.988** |
| Q2 | NIFTY | +0.153 | 0.249 | -0.065 | 0.623 |
| Q2 | BANKNIFTY | +0.058 | 0.664 | +0.112 | 0.400 |
| **Q2** | **POOLED** | **+0.107** | **0.248** | **+0.019** | **0.840** |

| Quarter | Mean \|gap\| STAY | Mean \|gap\| TRANSITION |
|---------|------------------|------------------------|
| Q1 | 0.330 | 0.329 |
| Q2 | 0.432 | 0.459 |

### Verdict: **No signal**

> [!NOTE]
> Neither `gap_pct` nor `|gap_pct|` predicts whether an ecology transition occurs. All p-values exceed 0.2. The mean absolute gap is nearly identical for stay vs. transition events.

`gap_pct` is confirmed as a **non-contributor** to both ecological structure AND ecological dynamics. It is orthogonal to the phenomenon.

---

## Test 3: GMM (k=2) vs Ward — Separation Vector Comparison

### Hypothesis

If the ecological axis is a real geometric property of the data (not a Ward artifact), a different clustering method should find the same separation direction.

### Results

| Quarter | ARI (Ward vs GMM) | Separation cosine (Ward vs GMM) |
|---------|-------------------|---------------------------------|
| Q1 | 0.229 | **0.919** |
| Q2 | 0.808 | **0.990** |

| | Ward sizes | GMM sizes |
|--|-----------|-----------|
| Q1 | 90/30 | 61/59 |
| Q2 | 67/53 | 65/55 |

### Cross-quarter comparison

| Method | Q1↔Q2 Separation cosine |
|--------|------------------------|
| **Ward** | **0.969** |
| **GMM** | **0.984** |

### GMM covariance structure (confirms Ward's asymmetry bias)

| Quarter | GMM Ecology A variance | GMM Ecology B variance | Ratio (B/A) |
|---------|----------------------|----------------------|-------------|
| Q1 | 2.53 | 4.85 | **1.92** |
| Q2 | 1.61 | 5.72 | **3.56** |

### Verdict: **Strong validation**

> [!TIP]
> The GMM separation vector cosine vs Ward is **0.919–0.990**. The cross-quarter GMM cosine is **0.984** — even higher than Ward's 0.969. The geometric axis is **real and method-independent**.

Key findings:

1. **The separation direction is preserved** regardless of clustering method
2. **GMM confirms the variance asymmetry** you flagged — Ecology B has 2–3.5× the variance of Ecology A. Ward was indeed compressing B.
3. **GMM gives more balanced cluster sizes** (Q1: 61/59 vs Ward's 90/30), suggesting Ward's equal-variance assumption was assigning some borderline Ecology B sessions to A
4. **The Q1 ARI is low (0.229)** — Ward and GMM disagree substantially on cluster *membership* in Q1, but agree on the *direction* of the partition. This means the axis is stable even when the boundary placement shifts.

---

## Summary

| Test | Result | Implication |
|------|--------|-------------|
| **Projection → return** | ❌ No signal | Ecological position does not linearly predict next-day returns |
| **Gap → transition** | ❌ No signal | gap_pct is orthogonal to both structure and dynamics |
| **GMM vs Ward** | ✅ Strong match | Geometric axis is real, method-independent, cosine 0.984 |

---

## What this changes

### The good news

The **structure** is validated more strongly than before. A completely different clustering method (GMM, which allows unequal variances and elliptical clusters) finds the same axis with even higher cross-quarter stability (0.984 > 0.969).

### The bad news

The simplest prediction experiment failed. `eco_position(t) → return(t+1)` has no linear signal. This rules out the easiest path to Phase 3.

### What to try next

The failure of Test 1 does not mean the structure has no predictive utility. It means the predictive channel is not:

```
position → next return (linear)
```

More promising candidates:

1. **Position → next volatility** (does knowing you're in Ecology B predict higher next-day realized vol?)
2. **Run-length → return** (is the *persistence duration*, not the position itself, predictive?)
3. **Transition → return** (do returns differ on transition days vs persistence days?)
4. **Position → conditional distribution** (does ecological position predict the *shape* of the return distribution, not just the mean?)

The structure exists and is geometrically stable. The question is now which **channel** connects it to outcomes.
