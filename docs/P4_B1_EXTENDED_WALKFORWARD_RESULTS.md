# P4 Track B.1 — Extended Walk-Forward Results

**Date:** 2026-09-12  
**Status:** RESOLVED — RETIRE  
**Produced by:** manual evaluation, walk-forward cohort Aug 24 – Sep 9

---

## Policy Under Evaluation

| Field | Value |
|---|---|
| Track | B.1 |
| Direction | SHORT |
| Action | Watch |
| target_rate window | [0.327778, 0.345900) |
| Horizon | H60 (daily realized_return proxy — see note) |
| Locked from | Aug 20-21 dev set boundary |

---

## Walk-Forward Cohort

- **Cohort:** Aug 24 – Sep 9 2026 (14 cohort dates)
- **Excluded:** Aug 20-21 dev set (policy was derived from this set — not eligible for walk-forward)
- **Total ledger entries (Aug 24+):** 870
- **SHORT Watch COMPLETE (baseline):** N=126
- **Track B.1 COMPLETE:** N=7

---

## B.1 Individual Results

| Decision ID | target_rate | realized_return | Outcome |
|---|---|---|---|
| LIVE-005-20260824-1000-BHARTIARTL_NS | 0.341176 | +0.72% | WIN |
| LIVE-005-20260825-1000-BHARTIARTL_NS | 0.341176 | +1.46% | WIN |
| LIVE-005-20260825-1000-BRITANNIA_NS | 0.338462 | +1.35% | WIN |
| LIVE-005-20260826-1000-BRITANNIA_NS | 0.338462 | +0.03% | WIN |
| LIVE-005-20260828-1000-BRITANNIA_NS | 0.338462 | +3.69% | WIN |
| LIVE-005-20260902-1000-SIEMENS_NS | 0.333333 | +2.29% | WIN |
| LIVE-005-20260907-1000-SIEMENS_NS | 0.333333 | -1.35% | LOSS |

---

## B.1 Walk-Forward Statistics

| Metric | B.1 (N=7) | SHORT Watch baseline (N=126) |
|---|---|---|
| Mean return | +1.169% | +0.884% (median; mean inflated by sentinels) |
| Median return | +1.347% | +0.884% |
| Win rate | 85.7% | 66.7% |
| Min | -1.355% | — |
| Max | +3.690% | — |

**Baseline note:** SHORT Watch baseline mean (+7.654%) is inflated by `realized_return = +1.0000` sentinel values (target-reached encoding). The median (+0.884%) is the correct comparison point.

---

## Sep 1/8/9 Cohort Contribution

The Sep 1/8/9 cohorts were recomputed and admitted (214 entries). Of the 157 COMPLETE Watch observations in those cohorts:

- SHORT Watch COMPLETE: 54
- Qualifying for B.1 locked window [0.327778, 0.345900): **2**
  - LIVE-005-20260902-1000-SIEMENS_NS (already counted above — Sep 2 is an Aug 24+ cohort date)
  - LIVE-005-20260907-1000-SIEMENS_NS (already counted above — Sep 7 is an Aug 24+ cohort date)

The Sep 1, Sep 8, Sep 9 cohorts proper contributed **0 new B.1 candidates** from their COMPLETE observations.

**Structural sparsity:** The locked B.1 boundary [0.327778, 0.345900) sits in the lower quartile of the Sep 1/8/9 SHORT Watch target_rate distribution (Q1=0.326923, Q2=0.351648, Q3=0.387097). The prospective cohort's target_rate distribution has shifted upward relative to the dev set boundary.

---

## Verdict

**Track B.1: RETIRE**

### Reasoning

1. **N=7 after full extended walk-forward.** The Sep 1/8/9 cohorts added 0 new qualifying decisions beyond what was already counted. The locked boundary is structurally sparse in the prospective cohort.

2. **Returns are positive but not meaningfully separated from baseline.** B.1 median +1.347% vs SHORT Watch baseline median +0.884%. The difference (+0.463%) is within noise at N=7.

3. **Win rate 85.7% is encouraging but unreliable at N=7.** 6 of 7 wins; a single additional loss would drop win rate to 71.4%, indistinguishable from baseline 66.7%.

4. **The locked boundary does not capture the prospective cohort.** The target_rate window was derived from the Aug 20-21 dev set. In the Aug 24+ prospective cohort, the same window captures only 7 decisions across 14 cohort dates — fewer than 1 per cohort on average. This is not a viable operating policy.

5. **No path to resolution.** Waiting for more Sep 1/8/9 observations to complete will not change the structural sparsity problem. The boundary itself is the issue, not the sample size.

### Decision

B.1 is **retired**. The locked boundary [0.327778, 0.345900) does not generalize to the prospective cohort distribution. No Sprint 4 effort should be directed at B.1.

---

## Sprint 4 Authorization Decision

With B.1 resolved (RETIRE), the full walk-forward verdict is:

| Track | N | Verdict | Sprint 4 action |
|---|---|---|---|
| Track A (daily) | 25 | **PASS** | **Carry forward** |
| Track B.1 (H60) | 7 | **RETIRE** | Do not pursue |
| Track B.2 (H300) | 62 | PASS, no incremental edge | Do not pursue |
| Track B.3 (H15) | 63 | PASS, no incremental edge | Do not pursue |

**Sprint 4 is now authorized on Track A only.**

Track A criterion: SHORT Watch, target_rate ≥ Q2 boundary (locked from Aug 20-21 dev set), daily horizon.  
Sprint 4 scope: Change exactly ONE thing about Track A. Identify the single most suspected improvement. Historical development test → locked walk-forward test → compare against Track A control.

**No new policy search. No threshold changes outside the Sprint 4 single-change protocol.**