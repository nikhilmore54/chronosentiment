# P4 Intraday — Phase 4: Prospective Paper Test

**Status:** ACTIVE — rules frozen, prospective observation running  
**Frozen:** 2026-09-12  
**No rule changes permitted during Phase 4.**

---

## Purpose

Determine whether the historically validated path-reassessment rules survive new cohorts not seen during discovery, robustness testing, or failure analysis.

Phase 4 is a **blind prospective paper test**. The rules are applied unchanged to new daily cohorts as they arrive. Results are recorded without modification.

---

## Frozen Candidate Rules

### Track B — SHORT Watch State Machine

```
T0: SHORT Watch decision identified

H15 (3 bars, ~15 min):
  FAV (>+0.2%)  → candidate ENTER
  FLAT (±0.2%)  → WAIT
  ADV (<-0.2%)  → candidate AVOID

H60 (12 bars, ~60 min):
  [H15 FAV] + H60 FAV  → ENTER
  [H15 FAV] + H60 ADV  → WAIT
  [H15 ADV] + H60 ADV  → AVOID (MFE_H60=0 override also → AVOID)
  [H15 ADV] + H60 FAV  → WAIT
  [H15 FLAT] + any     → WAIT

H120 (24 bars, ~120 min) — WAIT sub-classification:
  WAIT + H120 FAV       → ENTER-LATE
  WAIT + H120 FLAT      → WAIT-LATE
  WAIT + H120 ADV       → AVOID-LATE
  MFE_H120 = 0          → AVOID-LATE (override)

H180 (36 bars, ~180 min) — confirmation layer:
  ENTER + H180 FAV      → CONFIRMED (100% win rate historically)
  ENTER + H180 ADV      → DETERIORATING (warning)
  AVOID + H180 ADV      → CONFIRMED AVOID (6.2% win rate historically)
```

### Track C — LONG Watch State Machine

Identical structure, direction-adjusted:

```
T0: LONG Watch decision identified

H15: FAV/FLAT/ADV (same thresholds ±0.2%)
H60: same classification logic
H120: same WAIT sub-classification
H180: same confirmation layer
```

---

## Historical Performance Reference (frozen, immutable)

### Track B — SHORT Watch (N=187)

| Bucket | N | H300 median | H300 win |
|---|---|---|---|
| ENTER (H60) | 45 | +0.963% | 82.2% |
| ENTER-LATE (H120) | 47 | +0.779% | 89.4% |
| WAIT-LATE | 35 | +0.085% | 65.7% |
| AVOID (H60) | 44 | -0.379% | 36.4% |
| AVOID-LATE (H120) | 16 | -0.739% | 6.2% |
| **ENTER+ENTER-LATE** | **92** | **+0.892%** | **88.4%** |
| **AVOID+AVOID-LATE** | **63** | **-0.538%** | **31.7%** |
| Baseline (all SHORT Watch) | 187 | +0.266% | 63.6% |

H120 FAV → H180 FAV: N=35, win **100.0%**  
MFE_H120=0: N=31, win 22.6%, median -0.780%

Permutation p-values (1000 shuffles): all ≤ 0.001  
Leave-one-cohort-out ENTER win: 78.9–93.9%  
Threshold sensitivity: stable ±0.10% to ±0.50%

### Track C — LONG Watch (N=230)

| Bucket | N | H300 median | H300 win |
|---|---|---|---|
| ENTER (H60) | 16 | +0.610% | 81.2% |
| ENTER-LATE (H120) | 40 | +0.449% | 77.5% |
| WAIT-LATE | 43 | -0.120% | 46.5% |
| AVOID (H60) | 102 | -0.728% | 17.6% |
| AVOID-LATE (H120) | 29 | -0.412% | 10.3% |
| **ENTER+ENTER-LATE** | **54** | **+0.554%** | **79.6%** |
| **AVOID+AVOID-LATE** | **123** | **-0.668%** | **12.2%** |
| Baseline (all LONG Watch) | 230 | -0.285% | 37.0% |

H120 FAV → H180 FAV: N=30, win **90.0%**  
H120 ADV → H180 ADV: N=20, win **0.0%**  
MFE_H120=0: N=80, win 8.8%, median -0.841%

Permutation p-values (1000 shuffles): all ≤ 0.001  
Leave-one-cohort-out ENTER win: 76.9–85.7%  
Threshold sensitivity: stable ±0.10% to ±0.50%

---

## Phase 4 Observation Protocol

### For each new daily cohort (Sep 11+):

1. Apply frozen rules to all SHORT Watch and LONG Watch decisions.
2. Record classification at H15, H60, H120, H180.
3. Record H300 realized outcome.
4. Do NOT modify rules based on observed outcomes.
5. Record every ENTER failure and AVOID win for failure analysis.

### Pass criteria (prospective):

- SHORT ENTER+ENTER-LATE win rate ≥ 75% (vs historical 88.4%)
- LONG ENTER+ENTER-LATE win rate ≥ 65% (vs historical 79.6%)
- SHORT AVOID+AVOID-LATE win rate ≤ 45% (vs historical 31.7%)
- LONG AVOID+AVOID-LATE win rate ≤ 30% (vs historical 12.2%)
- Minimum N=20 per direction before declaring pass/fail

### Failure criteria:

- ENTER win rate collapses to baseline (≤ 65% SHORT, ≤ 45% LONG)
- AVOID win rate rises to baseline (≥ 60% either direction)
- Single cohort drives all separation (cohort stability check)

---

## What Phase 4 Must Answer

1. Does the separation survive new cohorts not seen during discovery?
2. Does the LONG/SHORT symmetry survive prospectively?
3. Does the reassessment mechanism (H120/H180) add value prospectively?
4. What breaks — record every failure without changing rules.

---

## What Comes After Phase 4

If Phase 4 passes:
- Phase 5: external-market stress test (crypto or other market)
- Phase 6: B+C unification decision
- Phase 7: Decision Cockpit integration

If Phase 4 fails:
- Diagnose failure mode from recorded failures
- Determine whether failure is regime-specific or structural
- Do NOT reopen historical tuning

---

## Governance

- Track A: frozen operating baseline (SHORT Watch + rank_score ≥ 0.35)
- Track B: SHORT path engine candidate — frozen at v0.3
- Track C: LONG path engine candidate — frozen at v0.2
- No rule changes during Phase 4
- LONG and SHORT remain logically separate during Phase 4
- Merge decision deferred until Phase 4 complete