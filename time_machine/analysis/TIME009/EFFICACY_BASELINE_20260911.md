# TIME-009 Efficacy Baseline — 2026-09-11

**Status:** FROZEN INTERIM BASELINE  
**Generated:** 2026-09-11  
**Decision:** HOLD / OBSERVE — do not modify decision engine, thresholds, evidence classification, or horizon.

---

## Pipeline State

| Component | Status |
|---|---|
| TIME-009 observer | Operational (all 5 ACs passing) |
| LIVE-005 ledger | 1393 entries |
| TIME-009 COMPLETE | 404 (Aug 20–21 cohorts, horizon=3–4) |
| TIME-009 PENDING | 1093 (Aug 24–Sep 10 cohorts, horizon=20) |
| INTRADAY-001 COMPLETE | 4014 artifacts (669 decisions × 6 horizons) |
| INTRADAY-001 INVALID_DATA_GAP | 4344 artifacts (724 decisions, Aug 20–28, permanent) |
| Scheduler | Running (PID 82044, fires 15:45 IST each NSE trading day) |

---

## TIME-009 Daily Efficacy (n=208 eligible, Aug 20–21 cohorts)

### Exit reason breakdown (all 404 COMPLETE)

| Exit reason | Count | Rate |
|---|---|---|
| NO_TRADE | 196 | 48.5% |
| HORIZON | 146 | 36.1% |
| TARGET_GAP_THROUGH | 32 | 7.9% |
| TARGET | 14 | 3.5% |
| RISK_GAP_THROUGH | 12 | 3.0% |
| RISK | 4 | 1.0% |

**Note:** Gap-through exits (n=44) produce ±100% artefact returns in the observer output
(`realized_return = (open - ref) / ref` hitting ±1.0). These are excluded from all return statistics.

### Clean exits only (n=164, excl. gap-through)

| Metric | All (n=164) | LONG (n=86) | SHORT (n=78) |
|---|---|---|---|
| Target rate | 8.5% | 14.0% | 2.6% |
| Risk rate | 2.4% | — | — |
| Horizon exit | 89.0% | — | — |
| Mean return | +0.204% | +0.715% | -0.358% |
| Median return | +0.072% | — | — |
| Win rate | 50.6% | 55.8% | 44.9% |
| Stdev | 1.952% | — | — |

### By evidence class (clean exits)

| Evidence class | n | Target rate | Mean return | Win rate |
|---|---|---|---|---|
| Favourable | 42 | 9.5% | -0.004% | 50.0% |
| Mixed | 122 | 8.2% | +0.276% | 50.8% |

**Observation:** No demonstrated advantage for the Favourable class in this sample.

---

## Intraday 5m Efficacy (n=160 eligible, all Aug 20–Sep 10 cohorts)

All 16 cohort dates are within the 5m cache range (Jun 22–Sep 11). Analysis uses the same
target/risk levels as TIME-009. 0% risk hit rate at every horizon.

| Horizon | n | TARGET | RISK | HORIZON | Mean return | Win rate | Stdev |
|---|---|---|---|---|---|---|---|
| H15 (15 min) | 160 | 1.2% | 0.0% | 98.8% | +0.049% | 47.5% | 0.690% |
| H30 (30 min) | 160 | 1.9% | 0.0% | 98.1% | +0.042% | 50.6% | 0.782% |
| H120 (2 hr) | 160 | 3.1% | 0.0% | 96.9% | +0.042% | 46.9% | 0.922% |
| H180 (3 hr) | 160 | 3.1% | 0.0% | 96.9% | -0.027% | 45.6% | 0.972% |
| H240 (4 hr) | 160 | 3.8% | 0.0% | 96.2% | +0.002% | 46.2% | 1.178% |
| H300 (5 hr) | 160 | 3.8% | 0.0% | 96.2% | -0.026% | 45.6% | 1.156% |

**Key finding:** 0% intraday risk hit rate across all horizons. The risk boundary is not being
engaged intraday, suggesting it is set wide relative to intraday volatility.

---

## HDV-001 Historical Context (NOT merged into TIME-009 statistics)

HDV-001 (728 COMPLETE decisions, generated 2026-08-17, 10-session horizon) used a materially
different decision engine (coralys_trend/momentum/volatility schema) and different exit mechanics.

| Metric | HDV-001 | TIME-009 |
|---|---|---|
| Target rate | 35.7% | 8.5% |
| Risk rate | 41.5% | 2.4% |
| Horizon exit | 22.8% | 89.0% |
| Horizon length | 10 sessions | 3–4 sessions (Aug 20–21 cohorts) |

**The 35.7% vs 8.5% target rate comparison cannot be interpreted as performance deterioration**
without controlling for the different decision schema, horizon, eligibility, and exit mechanics.
HDV-001 is context, not evidence of degradation.

---

## Backfill Assessment

| Cohort range | 5m cache available | Backfill feasible |
|---|---|---|
| Aug 20–Sep 10 (all 16 cohorts) | Yes (Jun 22–Sep 11) | Yes, for 5m intraday analysis |
| Pre-Aug 20 | Not in current 5m cache | No |
| 1093 PENDING horizon-20 | Must mature naturally | No — protocol frozen |

---

## Consolidated Conclusion

**The pipeline is operational, but the current evidence does not demonstrate meaningful decision efficacy.**

- No meaningful intraday edge across H15–H300 (mean returns +0.025% to +0.049%)
- Daily edge is marginal (+0.204% mean, 50.6% win rate) with 89% horizon exits
- LONG shows better daily performance than SHORT (+0.715% vs -0.358%) — observation only, not actionable at n=86/78
- Favourable evidence class shows no advantage over Mixed in this sample

**The principal unresolved question is whether the 1,093 pending horizon=20 decisions
(Aug 24–Sep 10 cohorts, completing Sep 16–Oct 9) will demonstrate a persistent edge.**

**Decision: HOLD / OBSERVE.** Let TIME-009 mature to its predefined stopping condition.
Do not change the decision engine, thresholds, evidence classification, or horizon based on
this interim result.

---

## Horizon Completion Schedule

| Cohort | Elapsed | Completes |
|---|---|---|
| Aug 20 | 17/20 | Sep 16 |
| Aug 21 | 16/20 | Sep 17 |
| Aug 24 | 15/20 | Sep 18 |
| Aug 25 | 14/20 | Sep 19 |
| Aug 26 | 13/20 | Sep 22 |
| Aug 27 | 12/20 | Sep 23 |
| Aug 28 | 11/20 | Sep 24 |
| Aug 31 | 10/20 | Sep 25 |
| Sep 1 | 9/20 | Sep 26 |
| Sep 2 | 8/20 | Sep 29 |
| Sep 3 | 7/20 | Sep 30 |
| Sep 4 | 6/20 | Oct 1 |
| Sep 7 | 5/20 | Oct 6 |
| Sep 8 | 4/20 | Oct 7 |
| Sep 9 | 3/20 | Oct 8 |
| Sep 10 | 2/20 | Oct 9 |