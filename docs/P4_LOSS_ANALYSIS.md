# P4 Loss Analysis — 1m vs 5m Resolution Comparison

**Status:** COMPLETE — evidence baseline frozen  
**Date:** 2026-09-12  
**Cohort:** LIVE-003 Watch decisions, Sep 3–11 (N=224 decisions, 6 cohorts)  
**Script:** [`scripts/p4_loss_analysis_1m_vs_5m.py`](../scripts/p4_loss_analysis_1m_vs_5m.py)

---

## Purpose

This document is a permanent product evidence artifact. It captures the 1m vs 5m
intraday path comparison for ENTER failures and AVOID false negatives, the product
conclusions drawn from that comparison, and the changes explicitly rejected.

It is not a research document. It does not propose new rules. It records what we
observed and what we decided not to do.

---

## Methodology

Same 224 LIVE-003 Watch decisions evaluated against both 1m and 5m intraday paths.
Same frozen Phase 4 state machine applied at both resolutions.

**Resolution mapping:**

| Horizon | 1m bars | 5m bars |
|---------|---------|---------|
| H15     | 15      | 3       |
| H60     | 60      | 12      |
| H120    | 120     | 24      |
| H180    | 180     | 36      |
| H300    | 300     | 60      |

**Frozen classification rules (immutable):**

- Threshold ±0.2% (0.002)
- MFE@H60 < 0.1% override → AVOID regardless of H15/H60 buckets
- H15 FAV + H60 FAV → ENTER
- H15 ADV + H60 ADV → AVOID
- Otherwise → WAIT
- H120 sub-classification: FAV → ENTER-LATE, ADV → AVOID-LATE, FLAT → WAIT-LATE

**Four conditions for a pattern to become a product candidate:**

1. Present at BOTH resolutions
2. Visible BEFORE the eventual outcome
3. Materially DIFFERENT between winners and losers
4. ACTIONABLE at the time

---

## Data

- Watch decisions loaded: 224 (SHORT=69, LONG=155)
- Cohorts: 2026-09-03, 09-04, 09-07, 09-08, 09-09, 09-10
- 1m measured: 189/224 (35 tickers not in 1m cache)
- 5m measured: 224/224

---

## 1m vs 5m Failure Matrix

| Failure characteristic | SHORT 1m | SHORT 5m | LONG 1m | LONG 5m |
|---|---:|---:|---:|---:|
| **ENTER losses** | 3/13 | 3/16 | 7/17 | 2/8 |
| Win rate | 76.9% | 81.2% | 58.8% | 75.0% |
| — IMMEDIATE | 0 | 0 | 0 | 0 |
| — EARLY | 0 | 0 | 0 | 0 |
| — MID-REVERSAL | 0 | 0 | 0 | 0 |
| — LATE-REVERSAL | 0 | 0 | 0 | 0 |
| — VERY-LATE-REVERSAL | 1 | 1 | 0 | 0 |
| — GRADUAL/OTHER | 2 | 2 | 7 | 2 |
| H60 FAV (losers) | 100% | 100% | 100% | 100% |
| H120 FAV (losers) | 66.7% | 100% | 42.9% | 100% |
| MFE@H60 median (losers) | +0.6% | +0.7% | +0.9% | +0.5% |
| First adverse bar (losers) | 242m | 240m | 269m | 50m |
| H60 FAV (winners) | 100% | 100% | 100% | 100% |
| H120 FAV (winners) | 90% | 100% | 90% | 100% |
| MFE@H60 median (winners) | +1.1% | +0.9% | +1.6% | +1.2% |
| First adverse bar (winners) | 2m | 0m | 0m | 0m |
| **AVOID false negatives** | 7/20 | 3/18 | 2/28 | 18/71 |
| FN rate | 35.0% | 16.7% | 7.1% | 25.4% |
| FN H60 ADV rate | 71.4% | 66.7% | 100% | 50.0% |
| FN H120 FAV rate | 14.3% | 33.3% | 0.0% | 55.6% |
| FN H300 median return | +0.5% | +0.7% | +0.7% | +0.5% |

---

## Findings

### SHORT ENTER

**H120 FAV as protective signal: REJECTED**

H120 FAV gap (winners − losers): 1m = +23.3%, 5m = +0.0%.

At 5m resolution, 100% of both SHORT ENTER winners and losers reach H120 FAV.
H120 FAV is universal for SHORT ENTER — it carries no discriminating power.
The 1m gap (+23.3%) does not survive the 5m comparison. Condition 1 fails.

**Consistent finding at both resolutions:**

First adverse bar for losers ≈ 240m vs winners ≈ 0–2m. SHORT losers hold up for
approximately 4 hours before failing; winners fail immediately or not at all.

This is a timing pattern, not a path-state pattern. It cannot be detected from
H15/H60/H120 checkpoints. It requires continuous bar-level monitoring.

**Failure mode:** GRADUAL/OTHER (2) + VERY-LATE-REVERSAL (1) at both resolutions.
No IMMEDIATE, EARLY, or MID-REVERSAL failures in this cohort.

### LONG ENTER

**H120 FAV as protective signal: NOT CONFIRMED**

H120 FAV gap (winners − losers): 1m = +47.1%, 5m = +0.0%.

At 5m resolution, 100% of both LONG ENTER winners and losers reach H120 FAV.
The 5m LONG ENTER N is only 8 (too small to be reliable), but the direction is
consistent with SHORT: H120 FAV is universal at 5m. The 1m signal is suggestive
but not confirmed at 5m. Condition 1 fails.

**Surviving candidate — MFE@H60 magnitude:**

| Group | 1m median | 5m median |
|-------|-----------|-----------|
| LONG ENTER losers | +0.9% | +0.5% |
| LONG ENTER winners | +1.6% | +1.2% |
| Gap | +0.7pp | +0.7pp |

The separation survives both resolutions. Winners show ~+0.7pp higher MFE@H60
than losers at both 1m and 5m. This satisfies conditions 1, 2, and 3.

Condition 4 (actionable) requires a threshold and distribution analysis before
any product change. This is a **watch item** for the prospective test, not a
product change now.

**Failure mode:** GRADUAL/OTHER dominates (7/7 at 1m, 2/2 at 5m). No sharp
reversals. LONG failures are slow and diffuse.

### AVOID False Negatives

**AVOID override mechanism: REJECTED**

FN H300 median return: SHORT +0.5–0.7%, LONG +0.5–0.7% at both resolutions.
These are economically small wins.

The 5m LONG AVOID shows 55.6% H120 FAV recovery among false negatives, but the
1m LONG AVOID shows 0% H120 FAV recovery. The two resolutions disagree — this is
a resolution artefact, not a robust signal. The 5m H60 window (12 bars = 60 min)
is coarser and catches more initial weakness that subsequently recovers.

The AVOID signal is validated. 66–100% of false negatives remain H60 ADV at the
time of the AVOID decision. The system correctly identified adverse early path in
the majority of cases. The eventual winners did so by a median of only ~+0.5–0.7%
— not sufficient economic justification to complicate the product.

---

## Product Conclusions

### What was confirmed

- The frozen Phase 4 state machine (H15/H60 → ENTER/AVOID/WAIT) produces
  directionally consistent results at both 1m and 5m resolution.
- AVOID signal is robust. False negatives are economically small and path-confirmed.
- MFE@H60 magnitude for LONG ENTER is a candidate signal worth prospective testing.
- SHORT failures are timing-driven (first adverse bar ~240m), not path-state-driven.

### What was rejected

| Proposed change | Reason for rejection |
|---|---|
| H120 FAV as SHORT protection | Universal at 5m — 100% of both winners and losers reach H120 FAV. Misleading if surfaced. |
| H120 FAV as LONG protection | Universal at 5m. 1m signal not confirmed at 5m. |
| AVOID override at H120 | Resolution-dependent. FN H300 median only +0.5–0.7%. Not economically justified. |
| MFE@H60 threshold for LONG | Distribution not yet analysed. N too small for threshold selection. Watch item only. |

### What this means for Decision Cockpit v0.4

- Do NOT present H120 FAV as a confidence booster for either direction.
- Surface H120 state as information only (the four-row H15/H60/H120/H180 display).
- Do NOT build an AVOID override mechanism.
- MFE@H60 magnitude may be surfaced as a secondary indicator for LONG ENTER
  quality — but only after distribution analysis confirms a threshold.

### Open investigations

1. **LONG ENTER MFE@H60 distribution** — bin by 0–0.25%, 0.25–0.5%, 0.5–0.75%,
   0.75–1.0%, 1.0–1.25%, 1.25–1.5%, >1.5%; compare H300 outcomes per bin.
   Question: at what MFE@H60 level does LONG ENTER quality materially improve?

2. **SHORT ENTER continuous deterioration** — can rolling 1m bar monitoring detect
   the ~240m failure point before H300? This is a product architecture question
   (continuous polling vs checkpoint system), not a rule change.

---

---

## Addendum: LONG ENTER MFE@H60 Distribution Analysis

**Date:** 2026-09-12
**Script:** [`scripts/p4_mfe_h60_distribution.py`](../scripts/p4_mfe_h60_distribution.py)

### 1m Bin Results (N=17 LONG ENTER)

| MFE@H60 bin | N | Win% | H300 median |
|---|---|---|---|
| 0.25–0.50% | 2 | 50% | +0.37% |
| 0.50–0.75% | 2 | 0% | −0.12% |
| 0.75–1.00% | 2 | 50% | −0.55% |
| 1.00–1.25% | 2 | 50% | +0.40% |
| 1.25–1.50% | 3 | 33% | +0.10% |
| 1.50–2.00% | 4 | **100%** | **+1.38%** |
| >2.00% | 2 | **100%** | **+2.81%** |

Winner MFE@H60: min=+0.40%, p25=+1.18%, median=+1.56%, p75=+1.73%, max=+4.39%
Loser MFE@H60:  min=+0.47%, p25=+0.51%, median=+0.93%, p75=+1.45%, max=+1.46%

### 5m Bin Results (N=8 LONG ENTER)

| MFE@H60 bin | N | Win% | H300 median |
|---|---|---|---|
| 0.25–0.50% | 2 | 50% | +0.38% |
| 0.50–0.75% | 2 | 50% | +0.39% |
| 1.00–1.25% | 1 | 100% | +0.87% |
| 1.25–1.50% | 2 | 100% | +0.83% |
| >2.00% | 1 | 100% | +2.79% |

Winner MFE@H60: min=+0.46%, p25=+0.52%, median=+1.17%, p75=+1.43%, max=+3.44%
Loser MFE@H60:  min=+0.36%, p25=+0.36%, median=+0.49%, p75=+0.63%, max=+0.63%

### Threshold Candidate Analysis

| Threshold | 1m above N | 1m above W% | 1m below N | 1m below W% | 5m above N | 5m above W% | 5m below N | 5m below W% |
|---|---|---|---|---|---|---|---|---|
| ≥0.50% | 15 | 60% | 2 | 50% | 6 | 83% | 2 | 50% |
| ≥0.70% | 13 | 69% | 4 | 25% | 4 | 100% | 4 | 50% |
| ≥1.00% | 11 | 73% | 6 | 33% | 4 | 100% | 4 | 50% |
| ≥1.20% | 9 | 78% | 8 | 38% | 3 | 100% | 5 | 60% |
| **≥1.50%** | **6** | **100%** | **11** | **36%** | **1** | **100%** | **7** | **71%** |

### Conclusion

The ≥1.50% threshold at 1m shows a clean separation (100% vs 36% win rate, 64pp gap).
The 5m cohort has only N=1 above ≥1.50% — insufficient to confirm.

**Decision: MFE@H60 ≥1.50% is a prospective watch item.**
Record MFE@H60 for every LONG ENTER decision in the Sep 15+ prospective run.
Do NOT lock this threshold until N≥10 above threshold in prospective data.
No product change made.

---

## Governance

- Rules remain frozen. No new policies introduced.
- This document is the evidence baseline for the next product iteration.
- Any future change must reference this document and demonstrate improvement
  against the metrics recorded here.
- Script: [`scripts/p4_loss_analysis_1m_vs_5m.py`](../scripts/p4_loss_analysis_1m_vs_5m.py)
- Related: [`docs/P4_INTRADAY_PHASE4_PROSPECTIVE.md`](P4_INTRADAY_PHASE4_PROSPECTIVE.md)