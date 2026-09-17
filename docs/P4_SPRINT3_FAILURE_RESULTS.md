# P4 Sprint 3 — Failure Analysis Results

**Date:** 2026-09-11
**Population:** 300 COMPLETE decisions
**SHORT Watch decisions:** 61

**Governing question:** Can we find a decision-time characteristic that predicts
genuine positive returns rather than merely predicting gap-through outcomes?

**Gap-through threshold:** |return| > 50%

**Three outcome buckets:**
- **A — Genuine winners:** positive return, |return| ≤ 50%
- **B — Losers:** negative return
- **C — Gap-through winners:** return > 50% (extreme positive, typically target hit)

---

## 1. Outcome bucket overview

| Cohort | N | Genuine winners (A) | Losers (B) | Gap-through winners (C) | Flat |
|---|---|---|---|---|---|
| All Watch | 114 | 40 (35.1%) | 45 (39.5%) | 28 (24.6%) | 1 (0.9%) |
| LONG Watch | 53 | 25 (47.2%) | 27 (50.9%) | 0 (0.0%) | 1 (1.9%) |
| SHORT Watch | 61 | 15 (24.6%) | 18 (29.5%) | 28 (45.9%) | 0 (0.0%) |

---

## 2. SHORT Watch — gap-through excluded (decisive experiment)

If the 51–150 sample size band survives gap-through removal, it is a genuine signal.
If it collapses, the 'sweet spot' was an artifact of the gap-through mechanism.

| Cohort | N | Mean | Median | Win% | Loss% | Stdev |
|---|---|---|---|---|---|---|
| SHORT Watch (all, incl gaps) | 61 | +45.8% | +1.4% | 70.5% | 29.5% | +50.3% |
| SHORT Watch (gap-through excluded) | 33 | -0.1% | -0.1% | 45.5% | 54.5% | +1.4% |
| SHORT Watch genuine winners only | 15 | +0.9% | +0.7% | 100.0% | 0.0% | +1.0% |
| SHORT Watch losers only | 18 | -1.0% | -0.8% | 0.0% | 100.0% | +0.9% |
| SHORT Watch gap-through winners only | 28 | +100.0% | +100.0% | 100.0% | 0.0% | +0.0% |

---

## 3. Sample size bands — with and without gap-throughs

| Cohort | N | Mean | Median | Win% | Loss% | Stdev |
|---|---|---|---|---|---|---|
| SHORT Watch sample 1–50 (all) | 3 | -0.8% | -0.7% | 0.0% | 100.0% | +0.2% |
| SHORT Watch sample 1–50 (no gap) | 3 | -0.8% | -0.7% | 0.0% | 100.0% | +0.2% |
| SHORT Watch sample 51–100 (all) | 20 | +64.9% | +100.0% | 80.0% | 20.0% | +49.0% |
| SHORT Watch sample 51–100 (no gap) | 7 | -0.2% | -0.2% | 42.9% | 57.1% | +0.9% |
| SHORT Watch sample 101–150 (all) | 9 | +88.9% | +100.0% | 100.0% | 0.0% | +33.2% |
| SHORT Watch sample 101–150 (no gap) | 1 | +0.4% | +0.4% | 100.0% | 0.0% | +0.0% |
| SHORT Watch sample 151+ (all) | 29 | +24.1% | +0.3% | 62.1% | 37.9% | +43.6% |
| SHORT Watch sample 151+ (no gap) | 22 | -0.1% | +0.0% | 50.0% | 50.0% | +1.6% |

---

## 4. rank_score quartiles — with and without gap-throughs

| Cohort | N | Mean | Median | Win% | Loss% | Stdev |
|---|---|---|---|---|---|---|
| SHORT Watch rank_score Q1 (all) | 15 | +20.1% | +0.2% | 66.7% | 33.3% | +41.4% |
| SHORT Watch rank_score Q1 (no gap) | 12 | +0.1% | +0.1% | 58.3% | 41.7% | +0.9% |
| SHORT Watch rank_score Q2 (all) | 15 | +46.6% | +3.1% | 66.7% | 33.3% | +51.7% |
| SHORT Watch rank_score Q2 (no gap) | 8 | -0.1% | -0.2% | 37.5% | 62.5% | +2.2% |
| SHORT Watch rank_score Q3 (all) | 15 | +59.8% | +100.0% | 80.0% | 20.0% | +50.9% |
| SHORT Watch rank_score Q3 (no gap) | 6 | -0.4% | -0.1% | 50.0% | 50.0% | +1.5% |
| SHORT Watch rank_score Q4 (all) | 16 | +56.1% | +100.0% | 68.8% | 31.2% | +51.4% |
| SHORT Watch rank_score Q4 (no gap) | 7 | -0.4% | -0.4% | 28.6% | 71.4% | +0.7% |

---

## 5. target_rate quartiles — with and without gap-throughs

| Cohort | N | Mean | Median | Win% | Loss% | Stdev |
|---|---|---|---|---|---|---|
| SHORT Watch target_rate Q1 (all) | 15 | +26.3% | -0.2% | 46.7% | 53.3% | +46.0% |
| SHORT Watch target_rate Q1 (no gap) | 11 | -0.5% | -0.3% | 27.3% | 72.7% | +0.8% |
| SHORT Watch target_rate Q2 (all) | 15 | +46.9% | +1.4% | 86.7% | 13.3% | +51.4% |
| SHORT Watch target_rate Q2 (no gap) | 8 | +0.5% | +0.2% | 75.0% | 25.0% | +0.6% |
| SHORT Watch target_rate Q3 (all) | 15 | +66.5% | +100.0% | 80.0% | 20.0% | +49.1% |
| SHORT Watch target_rate Q3 (no gap) | 5 | -0.6% | -0.4% | 40.0% | 60.0% | +1.2% |
| SHORT Watch target_rate Q4 (all) | 16 | +43.7% | +3.1% | 68.8% | 31.2% | +51.3% |
| SHORT Watch target_rate Q4 (no gap) | 9 | -0.1% | -0.7% | 44.4% | 55.6% | +2.2% |

---

## 6. Characteristics of SHORT Watch losers (Bucket B)

N losers: 18

### 6a. Sample size distribution of losers

| Sample size band | N losers | N total SHORT Watch | Loser rate |
|---|---|---|---|
| 1–50 | 3 | 3 | 100.0% |
| 51–100 | 4 | 20 | 20.0% |
| 101–150 | 0 | 9 | 0.0% |
| 151+ | 11 | 29 | 37.9% |

### 6b. rank_score distribution of losers vs winners

| Bucket | N | Mean rank_score | Median rank_score |
|---|---|---|---|
| Genuine winners (A) | 15 | 0.3548 | 0.3317 |
| Losers (B) | 18 | 0.3689 | 0.3349 |
| Gap-through winners (C) | 28 | 0.4008 | 0.3579 |

### 6c. target_rate distribution of losers vs winners

| Bucket | N | Mean target_rate | Median target_rate |
|---|---|---|---|
| Genuine winners (A) | 15 | 0.3329 | 0.3254 |
| Losers (B) | 18 | 0.3394 | 0.3253 |
| Gap-through winners (C) | 28 | 0.3345 | 0.3333 |

### 6d. sample_size distribution of losers vs winners

| Bucket | N | Mean sample_size | Median sample_size |
|---|---|---|---|
| Genuine winners (A) | 15 | 142.8 | 158.0 |
| Losers (B) | 18 | 126.0 | 157.0 |
| Gap-through winners (C) | 28 | 108.7 | 102.0 |

### 6e. rr_ratio distribution of losers vs winners

| Bucket | N | Mean rr_ratio | Median rr_ratio |
|---|---|---|---|
| Genuine winners (A) | 15 | 0.0000 | 0.0000 |
| Losers (B) | 18 | 0.0000 | 0.0000 |
| Gap-through winners (C) | 28 | 0.0000 | 0.0000 |

---

## 7. SHORT Watch — gap-through excluded, full decomposition

Repeating Sprint 2 splits on gap-through-excluded SHORT Watch decisions.
This is the decisive test of whether any signal survives gap-through removal.

| Cohort | N | Mean | Median | Win% | Loss% | Stdev |
|---|---|---|---|---|---|---|
| SHORT Watch Exact (no gap) | 8 | -0.2% | -0.3% | 37.5% | 62.5% | +0.8% |
| SHORT Watch Approximate (no gap) | 0 | — | — | — | — | — |
| SHORT Watch sample 51–100 (no gap) | 7 | -0.2% | -0.2% | 42.9% | 57.1% | +0.9% |
| SHORT Watch sample 101–150 (no gap) | 1 | +0.4% | +0.4% | 100.0% | 0.0% | +0.0% |
| SHORT Watch sample 151+ (no gap) | 22 | -0.1% | +0.0% | 50.0% | 50.0% | +1.6% |
| SHORT Watch (no gap) rank_score Q1 (lowest) | 8 | -0.4% | -0.1% | 37.5% | 62.5% | +0.7% |
| SHORT Watch (no gap) rank_score Q2 | 8 | +0.8% | +0.6% | 75.0% | 25.0% | +1.1% |
| SHORT Watch (no gap) rank_score Q3 | 8 | -0.9% | -1.4% | 25.0% | 75.0% | +2.0% |
| SHORT Watch (no gap) rank_score Q4 (highest) | 9 | -0.2% | -0.2% | 44.4% | 55.6% | +0.8% |
| SHORT Watch (no gap) target_rate Q1 (lowest) | 8 | -0.6% | -0.8% | 25.0% | 75.0% | +0.8% |
| SHORT Watch (no gap) target_rate Q2 | 8 | +0.1% | +0.1% | 62.5% | 37.5% | +0.4% |
| SHORT Watch (no gap) target_rate Q3 | 8 | -0.1% | +0.1% | 50.0% | 50.0% | +1.2% |
| SHORT Watch (no gap) target_rate Q4 (highest) | 9 | -0.1% | -0.7% | 44.4% | 55.6% | +2.2% |

---

## 8. LONG Watch Q3 rank_score pocket — detailed inspection

Sprint 2 found: LONG Watch rank_score Q3 — N=13, +2.5% mean, +1.3% median, 76.9% win, 2.9% stdev.
Suspiciously low stdev suggests a cluster of similar decisions. Inspecting.

N in LONG Watch rank_score Q3: 13

| decision_id | sample_size | target_rate | rank_score | realized_return |
|---|---|---|---|---|
| LIVE-005-20260821-0654-VBL_NS | 70 | 0.3286 | 0.4682 | -0.8% |
| LIVE-005-20260821-1027-IDEA_NS | 58 | 0.3103 | 0.4695 | +6.3% |
| LIVE-005-20260820-1327-IDEA_NS | 58 | 0.3103 | 0.4695 | +6.3% |
| LIVE-005-20260821-0654-IDEA_NS | 58 | 0.3103 | 0.4695 | +6.3% |
| LIVE-005-20260820-1218-IDEA_NS | 58 | 0.3103 | 0.4695 | +6.3% |
| LIVE-005-20260821-0654-POWERGRID_NS | 104 | 0.3462 | 0.4772 | -0.7% |
| LIVE-005-20260820-1218-POWERGRID_NS | 104 | 0.3462 | 0.4772 | +2.9% |
| LIVE-005-20260821-1027-POWERGRID_NS | 104 | 0.3462 | 0.4772 | -1.1% |
| LIVE-005-20260820-1327-POWERGRID_NS | 104 | 0.3462 | 0.4772 | +2.9% |
| LIVE-005-20260820-1218-HINDUNILVR_NS | 167 | 0.3533 | 0.4774 | +0.7% |
| LIVE-005-20260821-1027-HINDUNILVR_NS | 167 | 0.3533 | 0.4774 | +1.3% |
| LIVE-005-20260820-1327-HINDUNILVR_NS | 167 | 0.3533 | 0.4774 | +0.7% |
| LIVE-005-20260821-0654-HINDUNILVR_NS | 167 | 0.3533 | 0.4774 | +1.1% |

**Ticker concentration in LONG Watch Q3:**

| Ticker | Count |
|---|---|
| IDEA_NS | 4 |
| POWERGRID_NS | 4 |
| HINDUNILVR_NS | 4 |
| VBL_NS | 1 |

---

## 9. Failure taxonomy — emerging categories

Based on the three-bucket analysis, the following failure patterns are observable:

**Genuine winners (A):** N=15, mean sample_size=143, mean target_rate=0.333
**Losers (B):** N=18, mean sample_size=126, mean target_rate=0.339
**Gap-through winners (C):** N=28, mean sample_size=109, mean target_rate=0.335

Failure taxonomy (to be refined as more data arrives):

| Failure class | Observable signal | Hypothesis |
|---|---|---|
| Insufficient analogues | sample_size < 51 | Too few historical comparisons to establish reliable direction |
| Analogue saturation | sample_size > 150 | Large analogue pools may include dissimilar situations, diluting signal |
| Gap-through dependency | |return| > 50% | Apparent edge may require target hit; non-gap-through outcomes are flat |
| Direction mismatch | LONG direction | LONG Watch decisions are structurally negative in current sample |
| Score inflation | High rank_score on LONG | rank_score assigns higher values to LONG, which underperforms |

---

## 10. Walk-forward finalist candidates — revised after gap-through analysis

Criteria: N >= 15, positive median (including gap-through excluded), win rate > 60%

| Candidate | N (all) | Median (all) | N (no gap) | Median (no gap) | Win% (no gap) | Status |
|---|---|---|---|---|---|---|
| SHORT Watch Exact | 21 | +100.0% | 8 | -0.3% | 37.5% | ✗ ELIMINATED |
| SHORT Watch sample 51–100 | 20 | +100.0% | 7 | -0.2% | 42.9% | ✗ ELIMINATED |
| SHORT Watch sample 101–150 | 9 | +100.0% | 1 | +0.4% | 100.0% | ✗ ELIMINATED |
| SHORT Watch sample 51–150 | 29 | +100.0% | 8 | -0.0% | 50.0% | ✗ ELIMINATED |
| SHORT Watch target_rate Q2 | 15 | +1.4% | 8 | +0.2% | 75.0% | ✓ FINALIST |
| SHORT Watch target_rate Q3 | 15 | +100.0% | 5 | -0.4% | 40.0% | ✗ ELIMINATED |

---

## Interpretation notes

- Gap-through events (|return| > 50%) are the primary driver of SHORT Watch's apparent edge.
- The decisive question is whether any decision-time variable predicts genuine returns
  (positive return without gap-through) rather than merely predicting gap-through frequency.
- Any finalist surviving gap-through removal is a candidate for walk-forward evaluation.
- Finalists with gap-through-dependent performance require a different policy framework:
  they may still be valuable if gap-through events are predictable, but that requires
  a separate gap-through prediction experiment, not a return-prediction experiment.
- TIME-009 observations used as outcome data only — not a tuning target.

**Next step:** Walk-forward evaluation of FINALIST candidates.