# P4 Sprint 2 — Measure Results

**Date:** 2026-09-11
**Population:** 300 COMPLETE decisions
**Watch decisions:** 114 (LONG: 53, SHORT: 61)

**Sprint 2 question:** Within SHORT decisions, what observable characteristics
systematically separate stronger from weaker outcomes?

Column key: N | Mean | Median | P25 | P75 | Win% | Loss% | Stdev | Target hit% | Risk hit%

---

## 1. Direction × action breakdown (full distributional view)

| Cohort | N | Mean | Median | P25 | P75 | Win% | Loss% | Stdev | Target hit% | Risk hit% |
|---|---|---|---|---|---|---|---|---|---|---|
| All COMPLETE | 300 | +12.8% | +0.2% | -1.0% | +5.2% | 55.0% | 44.3% | +53.4% | 15.3% | 5.3% |
| Watch (all) | 114 | +16.0% | +0.3% | -0.8% | +6.3% | 59.6% | 39.5% | +55.7% | 31.6% | 8.8% |
| LONG Watch | 53 | -18.3% | -0.1% | -1.2% | +0.8% | 47.2% | 50.9% | +39.8% | 11.3% | 18.9% |
| SHORT Watch | 61 | +45.8% | +1.4% | -0.2% | +100.0% | 70.5% | 29.5% | +50.3% | 49.2% | 0.0% |
| NoTrade (all) | 186 | +10.8% | +0.1% | -1.1% | +3.5% | 52.2% | 47.3% | +51.9% | 5.4% | 3.2% |

---

## 2. SHORT Watch decomposition — degradation level

| Cohort | N | Mean | Median | Win% | Loss% | Stdev |
|---|---|---|---|---|---|---|
| SHORT Watch Exact | 21 | +61.8% | +100.0% | 76.2% | 23.8% | +49.9% |
| SHORT Watch Approximate | 0 | — | — | — | — | — |
| SHORT Watch Insufficient | 0 | — | — | — | — | — |

---

## 3. SHORT Watch decomposition — sample size bands

| Cohort | N | Mean | Median | Win% | Loss% | Stdev |
|---|---|---|---|---|---|---|
| SHORT Watch sample 1–25 | 2 | -0.8% | -0.8% | 0.0% | 100.0% | +0.2% |
| SHORT Watch sample 26–50 | 1 | -0.7% | -0.7% | 0.0% | 100.0% | +0.0% |
| SHORT Watch sample 51–100 | 20 | +64.9% | +100.0% | 80.0% | 20.0% | +49.0% |
| SHORT Watch sample 101–150 | 9 | +88.9% | +100.0% | 100.0% | 0.0% | +33.2% |
| SHORT Watch sample 151+ | 29 | +24.1% | +0.3% | 62.1% | 37.9% | +43.6% |

---

## 4. SHORT Watch decomposition — target_rate quartiles

| Cohort | N | Mean | Median | Win% | Loss% | Stdev |
|---|---|---|---|---|---|---|
| SHORT Watch target_rate Q1 (lowest) | 15 | +26.3% | -0.2% | 46.7% | 53.3% | +46.0% |
| SHORT Watch target_rate Q2 | 15 | +46.9% | +1.4% | 86.7% | 13.3% | +51.4% |
| SHORT Watch target_rate Q3 | 15 | +66.5% | +100.0% | 80.0% | 20.0% | +49.1% |
| SHORT Watch target_rate Q4 (highest) | 16 | +43.7% | +3.1% | 68.8% | 31.2% | +51.3% |

---

## 5. SHORT Watch decomposition — rank_score quartiles

| Cohort | N | Mean | Median | Win% | Loss% | Stdev |
|---|---|---|---|---|---|---|
| SHORT Watch rank_score Q1 (lowest) | 15 | +20.1% | +0.2% | 66.7% | 33.3% | +41.4% |
| SHORT Watch rank_score Q2 | 15 | +46.6% | +3.1% | 66.7% | 33.3% | +51.7% |
| SHORT Watch rank_score Q3 | 15 | +59.8% | +100.0% | 80.0% | 20.0% | +50.9% |
| SHORT Watch rank_score Q4 (highest) | 16 | +56.1% | +100.0% | 68.8% | 31.2% | +51.4% |

---

## 6. SHORT Watch decomposition — vol_regime

| Cohort | N | Mean | Median | Win% | Loss% | Stdev |
|---|---|---|---|---|---|---|
| SHORT Watch vol=present | 61 | +45.8% | +1.4% | 70.5% | 29.5% | +50.3% |
| SHORT Watch vol=absent | 0 | — | — | — | — | — |

---

## 7. SHORT Watch decomposition — volume_regime

| Cohort | N | Mean | Median | Win% | Loss% | Stdev |
|---|---|---|---|---|---|---|
| SHORT Watch volume=Normal | 61 | +45.8% | +1.4% | 70.5% | 29.5% | +50.3% |
| SHORT Watch volume=High | 0 | — | — | — | — | — |
| SHORT Watch volume=Low | 0 | — | — | — | — | — |

---

## 8. SHORT Watch decomposition — certification_status

| Cohort | N | Mean | Median | Win% | Loss% | Stdev |
|---|---|---|---|---|---|---|
| SHORT Watch cert=CERTIFIED | 0 | — | — | — | — | — |
| SHORT Watch cert=DEGRADED | 61 | +45.8% | +1.4% | 70.5% | 29.5% | +50.3% |

---

## 9. SHORT Watch — cross-tabulation: degradation × sample_size

| Cohort | N | Mean | Median | Win% | Loss% | Stdev |
|---|---|---|---|---|---|---|
| SHORT Watch Exact sample 1–50 | 1 | -0.7% | -0.7% | 0.0% | 100.0% | +0.0% |
| SHORT Watch Exact sample 51–150 | 20 | +64.9% | +100.0% | 80.0% | 20.0% | +49.0% |
| SHORT Watch Exact sample 151+ | 0 | — | — | — | — | — |
| SHORT Watch Approximate sample 1–50 | 0 | — | — | — | — | — |
| SHORT Watch Approximate sample 51–150 | 0 | — | — | — | — | — |
| SHORT Watch Approximate sample 151+ | 0 | — | — | — | — | — |

---

## 10. rank_score anomaly investigation — direction interaction

Sprint 1 found: bottom 20% by rank_score (+45.5%) beats top 20% (+4.4%).
Hypothesis: the anomaly is driven by direction composition, not rank_score itself.

| Cohort | N | Mean | Median | Win% | Loss% | Stdev |
|---|---|---|---|---|---|---|

**LONG Watch by rank_score quartile:**

| Cohort | N | Mean | Median | Win% | Loss% | Stdev |
|---|---|---|---|---|---|---|
| LONG Watch rank_score Q1 (lowest) | 13 | -38.1% | -0.2% | 38.5% | 53.8% | +50.9% |
| LONG Watch rank_score Q2 | 13 | -8.1% | -0.9% | 38.5% | 61.5% | +27.6% |
| LONG Watch rank_score Q3 | 13 | +2.5% | +1.3% | 76.9% | 23.1% | +2.9% |
| LONG Watch rank_score Q4 (highest) | 14 | -28.6% | -0.4% | 35.7% | 64.3% | +46.9% |

**SHORT Watch by rank_score quartile:**

| Cohort | N | Mean | Median | Win% | Loss% | Stdev |
|---|---|---|---|---|---|---|
| SHORT Watch rank_score Q1 (lowest) | 15 | +20.1% | +0.2% | 66.7% | 33.3% | +41.4% |
| SHORT Watch rank_score Q2 | 15 | +46.6% | +3.1% | 66.7% | 33.3% | +51.7% |
| SHORT Watch rank_score Q3 | 15 | +59.8% | +100.0% | 80.0% | 20.0% | +50.9% |
| SHORT Watch rank_score Q4 (highest) | 16 | +56.1% | +100.0% | 68.8% | 31.2% | +51.4% |

**All Watch by rank_score quartile (mixed direction):**

| Cohort | N | Mean | Median | Win% | Loss% | Stdev |
|---|---|---|---|---|---|---|
| Watch rank_score Q1 (lowest) | 28 | +35.9% | +1.1% | 71.4% | 28.6% | +48.7% L:0/S:28 |
| Watch rank_score Q2 | 29 | +17.2% | +0.4% | 58.6% | 37.9% | +65.9% L:12/S:17 |
| Watch rank_score Q3 | 28 | +7.8% | -0.0% | 50.0% | 50.0% | +46.5% L:22/S:6 |
| Watch rank_score Q4 (highest) | 29 | +3.6% | +0.1% | 58.6% | 41.4% | +56.6% L:19/S:10 |

---

## 11. LONG Watch decomposition — where does the loss come from?

| Cohort | N | Mean | Median | Win% | Loss% | Stdev |
|---|---|---|---|---|---|---|
| LONG Watch Exact | 53 | -18.3% | -0.1% | 47.2% | 50.9% | +39.8% |
| LONG Watch Approximate | 0 | — | — | — | — | — |
| LONG Watch Insufficient | 0 | — | — | — | — | — |

| LONG Watch sample 1–50 | 4 | -0.8% | -0.9% | 0.0% | 100.0% | +0.5% |
| LONG Watch sample 51–100 | 12 | -14.9% | -0.0% | 50.0% | 50.0% | +39.9% |
| LONG Watch sample 101–150 | 29 | -27.4% | -0.4% | 41.4% | 55.2% | +45.6% |
| LONG Watch sample 151+ | 8 | +1.1% | +1.0% | 87.5% | 12.5% | +0.8% |

| LONG Watch vol=present | 53 | -18.3% | -0.1% | 47.2% | 50.9% | +39.8% |
| LONG Watch vol=absent | 0 | — | — | — | — | — |

---

## 12. Outcome distribution — gap-through analysis

Extreme ±100% values distort means. Identifying gap-through frequency.

| Cohort | N | N with |return|>50% | Gap-through% | Median (excl gaps) | Mean (excl gaps) |
|---|---|---|---|---|---|
| All Watch | 114 | 38 | 33.3% | +0.1% | +0.3% |
| LONG Watch | 53 | 10 | 18.9% | +0.1% | +0.7% |
| SHORT Watch | 61 | 28 | 45.9% | -0.1% | -0.1% |

---

## 13. SHORT Watch — best observable combination candidates

Combinations of decision-time variables showing strongest separation.
These are finalist candidates for walk-forward evaluation.

| Combination | N | Mean | Median | Win% | Loss% | Stdev |
|---|---|---|---|---|---|---|
| SHORT Watch Exact | 21 | +61.8% | +100.0% | 76.2% | 23.8% | +49.9% |
| SHORT Watch Exact sample>=50 | 21 | +61.8% | +100.0% | 76.2% | 23.8% | +49.9% |
| SHORT Watch Exact sample>=100 | 0 | — | — | — | — | — |
| SHORT Watch sample>=50 | 59 | +47.4% | +3.1% | 72.9% | 27.1% | +50.4% |
| SHORT Watch sample>=100 | 38 | +39.4% | +1.1% | 71.1% | 28.9% | +49.6% |
| SHORT Watch vol=present | 61 | +45.8% | +1.4% | 70.5% | 29.5% | +50.3% |
| SHORT Watch Exact+vol=present | 21 | +61.8% | +100.0% | 76.2% | 23.8% | +49.9% |
| SHORT Watch Exact+sample>=50+vol=present | 21 | +61.8% | +100.0% | 76.2% | 23.8% | +49.9% |
| SHORT Watch target_rate>=0.30 | 61 | +45.8% | +1.4% | 70.5% | 29.5% | +50.3% |
| SHORT Watch target_rate>=0.35 | 14 | +49.5% | +50.5% | 64.3% | 35.7% | +52.4% |
| SHORT Watch Exact+target_rate>=0.30 | 21 | +61.8% | +100.0% | 76.2% | 23.8% | +49.9% |

---

## Interpretation notes

- All results are in-sample on the 300 COMPLETE historical development set.
- Median is the primary reliability indicator given gap-through outliers.
- N < 15 should be treated as directional signals only, not stable estimates.
- Any combination with N >= 15, positive median, and win rate > 60% is a walk-forward candidate.
- TIME-009 observations used as outcome data only — not a tuning target.

**Next step:** Sprint 3 — Failure analysis.
For every losing SHORT Watch decision: what observable characteristics were present?
Let the failure taxonomy emerge from the data.