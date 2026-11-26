# P4 Sprint 1 — Rank Results

**Date:** 2026-09-11
**Population:** 300 COMPLETE decisions

---

## Baseline

| Cohort | N | Mean return | Median | Win rate | Loss rate | Mean win | Mean loss | Stdev |
|---|---|---|---|---|---|---|---|---|
| All COMPLETE | 300 | +12.765% | +0.206% | 55.0% | 44.3% | +39.759% | -20.531% | +53.381% |
| Watch only | 114 | +16.018% | +0.315% | 59.6% | 39.5% | +42.052% | -22.967% | +55.735% |
| LONG (all) | 130 | -19.213% | +0.000% | 49.2% | 49.2% | +2.054% | -41.080% | +40.595% |
| SHORT (all) | 170 | +37.219% | +0.543% | 59.4% | 40.6% | +63.651% | -1.472% | +48.944% |
| LONG Watch | 53 | -18.287% | -0.096% | 47.2% | 50.9% | +1.827% | -37.588% | +39.828% |
| SHORT Watch | 61 | +45.823% | +1.376% | 70.5% | 29.5% | +65.438% | -1.034% | +50.328% |

---

## Evidence class breakdown (Watch decisions only)

| Evidence | N | Mean return | Median | Win rate | Loss rate |
|---|---|---|---|---|---|
| Favourable | 0 | — | — | — | — |
| Mixed | 114 | +16.018% | +0.315% | 59.6% | 39.5% |
| Insufficient | 0 | — | — | — | — |

---

## Ranking experiment — does rank_score separate outcomes?

Universe: Watch decisions only, ranked by `rank_score` descending.

| Slice | N | Mean return | Median | Win rate | Loss rate |
|---|---|---|---|---|---|
| Top 5% | 5 | +20.009% | +0.088% | 60.0% | 40.0% |
| Top 10% | 11 | +45.380% | +0.712% | 72.7% | 27.3% |
| Top 20% | 22 | +4.381% | -0.167% | 45.5% | 54.5% |
| Top 50% | 57 | +5.670% | +0.134% | 54.4% | 45.6% |
| Full universe | 114 | +16.018% | +0.315% | 59.6% | 39.5% |
| Bottom 20% | 22 | +45.490% | +1.333% | 77.3% | 22.7% |
| Bottom 10% | 11 | +17.935% | +0.091% | 54.5% | 45.5% |
| Bottom 5% | 5 | -0.637% | -0.288% | 20.0% | 80.0% |

---

## Ranking experiment — target_rate

Universe: Watch decisions only, ranked by `target_rate` descending.

| Slice | N | Mean return | Median | Win rate | Loss rate |
|---|---|---|---|---|---|
| Top 5% | 5 | -0.859% | -0.977% | 0.0% | 100.0% |
| Top 10% | 11 | -36.541% | -1.212% | 18.2% | 81.8% |
| Top 20% | 22 | -4.847% | -0.447% | 45.5% | 54.5% |
| Full universe | 114 | +16.018% | +0.315% | 59.6% | 39.5% |
| Bottom 20% | 22 | -9.305% | -0.395% | 40.9% | 59.1% |
| Bottom 10% | 11 | -54.554% | -100.000% | 18.2% | 81.8% |
| Bottom 5% | 5 | -100.000% | -100.000% | 0.0% | 100.0% |

---

## Ranking experiment — expected_return (EV)

Universe: Watch decisions with valid target/risk, ranked by `expected_return` descending.

| Slice | N | Mean return | Median | Win rate | Loss rate |
|---|---|---|---|---|---|
| Top 5% | 5 | -0.937% | -0.977% | 0.0% | 100.0% |
| Top 10% | 11 | +17.483% | -0.656% | 27.3% | 72.7% |
| Top 20% | 22 | +26.621% | -0.288% | 36.4% | 63.6% |
| Full universe | 114 | +16.018% | +0.315% | 59.6% | 39.5% |
| Bottom 20% | 22 | -25.876% | +0.072% | 54.5% | 40.9% |
| Bottom 10% | 11 | +2.342% | +0.369% | 72.7% | 18.2% |
| Bottom 5% | 5 | +1.242% | +0.071% | 60.0% | 20.0% |

---

## Ranking experiment — rr_ratio

Universe: Watch decisions with valid rr_ratio, ranked by `rr_ratio` descending.

| Slice | N | Mean return | Median | Win rate | Loss rate |
|---|---|---|---|---|---|
| Top 5% | 2 | -0.004% | -0.004% | 50.0% | 50.0% |
| Top 10% | 5 | +2.408% | +0.088% | 60.0% | 40.0% |
| Top 20% | 10 | +2.539% | +0.356% | 80.0% | 20.0% |
| Full universe | 53 | -18.287% | -0.096% | 47.2% | 50.9% |
| Bottom 20% | 10 | +0.057% | +0.035% | 50.0% | 40.0% |
| Bottom 10% | 5 | +0.189% | +0.071% | 60.0% | 20.0% |
| Bottom 5% | 2 | -0.167% | -0.167% | 50.0% | 50.0% |

---

## Selection filter experiment — LONG Watch only

| Filter | N | Mean return | Median | Win rate | Loss rate |
|---|---|---|---|---|---|
| LONG Watch (all) | 53 | -18.287% | -0.096% | 47.2% | 50.9% |
| LONG Watch Favourable | 0 | — | — | — | — |
| LONG Watch Fav+Mixed | 53 | -18.287% | -0.096% | 47.2% | 50.9% |
| LONG Watch Exact | 53 | -18.287% | -0.096% | 47.2% | 50.9% |
| LONG Watch sample>=50 | 49 | -19.715% | +0.071% | 51.0% | 46.9% |
| LONG Watch sample>=100 | 37 | -21.270% | +0.071% | 51.4% | 45.9% |
| LONG Watch target_rate>=0.35 | 19 | -20.754% | +0.134% | 57.9% | 42.1% |
| LONG Watch target_rate>=0.40 | 0 | — | — | — | — |
| LONG Watch rank_score>=0.5 | 2 | -0.004% | -0.004% | 50.0% | 50.0% |
| LONG Watch rank_score>=0.6 | 0 | — | — | — | — |
| LONG Watch rr>=1.0 | 31 | -18.580% | -0.518% | 45.2% | 54.8% |
| LONG Watch Exact+sample>=50 | 49 | -19.715% | +0.071% | 51.0% | 46.9% |
| LONG Watch Fav+sample>=50 | 0 | — | — | — | — |
| LONG Watch Fav+tr>=0.35 | 0 | — | — | — | — |

---

## Selection filter experiment — SHORT Watch only

| Filter | N | Mean return | Median | Win rate | Loss rate |
|---|---|---|---|---|---|
| SHORT Watch (all) | 61 | +45.823% | +1.376% | 70.5% | 29.5% |
| SHORT Watch Favourable | 0 | — | — | — | — |
| SHORT Watch Fav+Mixed | 61 | +45.823% | +1.376% | 70.5% | 29.5% |
| SHORT Watch Exact | 21 | +61.821% | +100.000% | 76.2% | 23.8% |
| SHORT Watch sample>=50 | 59 | +47.405% | +3.100% | 72.9% | 27.1% |
| SHORT Watch target_rate>=0.35 | 14 | +49.516% | +50.466% | 64.3% | 35.7% |
| SHORT Watch rank_score>=0.5 | 3 | +33.351% | +0.712% | 66.7% | 33.3% |

---

## Sample size distribution (Watch decisions)

| Sample size bucket | N decisions | Mean return |
|---|---|---|
| 0 (NoTrade/Insufficient) | 0 | — |
| 1–25 | 2 | -0.816% |
| 26–50 | 5 | -0.768% |
| 51–100 | 32 | +34.995% |
| 101–150 | 38 | +0.126% |
| 151–999 | 37 | +19.105% |

---

## Interpretation notes

- All policies evaluated on the same 300 COMPLETE decisions (historical development set).
- These results are in-sample observations, not walk-forward validated.
- Any policy showing material improvement becomes a finalist for walk-forward evaluation.
- A policy that fails to improve is an equally valid result — it eliminates a hypothesis.
- TIME-009 observations are used as outcome data only; they are not a tuning target.

**Next step:** Identify the strongest-performing selection/ranking combination,
then proceed to Sprint 2 (Measure) and Sprint 3 (Failure analysis).