# HDV-001-F Official Determination

**Date:** 2026-08-17
**Determination:** PASS
**Frozen criterion source:** datasets/hdv001/HDV_001_G_FREEZE_GATE.md (Gate 6)

---

## Governance Note

The v2 baseline runner (`hdv001_run_baselines.py`) incorrectly added Baseline C
(Momentum) to the pass/fail gate and declared "Overall FAIL". This was a
governance error. Baseline C is a contextual baseline only and is not part of
the frozen HDV-001-G success criterion.

This document contains the official determination against the frozen criterion.

---

## Frozen Success Criterion (HDV-001-G Gate 6)

1. Coralys TARGET_BEFORE_RISK rate > Baseline A (Random) by >= 5 pp
2. Coralys TARGET_BEFORE_RISK rate > Baseline B (Inverse) by >= 5 pp
3. The difference is consistent across at least 2 of the 4 Coralys state segments

Baseline C (Momentum) is reported for context but is NOT part of the criterion.

---

## Aggregate Results (N=728 COMPLETE decisions)

| Model | TARGET_HIT | Rate | Margin vs Coralys | Criterion |
|-------|-----------|------|-------------------|-----------|
| **Coralys** | 254 | **34.9%** | — | — |
| Baseline A — Random | 210 | 28.8% | +6.0 pp | PASS (>= 5 pp) |
| Baseline B — Inverse | 164 | 22.5% | +12.4 pp | PASS (>= 5 pp) |
| Baseline C — Momentum (contextual) | 248 | 34.1% | +0.8 pp | not in criterion |

---

## State-Segment Evaluation

| State | N | Coralys | Random | Inverse | Beats both by 5pp? |
|-------|---|---------|--------|---------|-------------------|
| Bullish_Positive | 299 | 40.8% | 29.1% | 17.7% | YES |
| Bullish_Negative | 113 | 25.7% | 34.5% | 39.8% | no |
| Bearish_Positive | 95 | 37.9% | 36.8% | 20.0% | no |
| Bearish_Negative | 221 | 30.3% | 22.2% | 21.3% | YES |

Segments beating both baselines by >= 5 pp: **2 / 4**

---

## Same-Bar Ambiguity

Same-bar target+stop cases (Coralys evaluation): **6**
Resolution rule: TARGET takes precedence (per HDV-001-G Gate 4).

---

## Official Criterion Evaluation

| Gate | Check | Result | Status |
|------|-------|--------|--------|
| 1 | Coralys > Random by >= 5 pp | +6.0 pp | PASS |
| 2 | Coralys > Inverse by >= 5 pp | +12.4 pp | PASS |
| 3 | >= 2 of 4 segments beat both | 2/4 | PASS |
| **OVERALL** | | | **PASS** |

---

## Governance Constraints

Criterion PASS: risk-boundary research (HDV-002) may proceed.

Do not modify C3-002 or reference-risk boundaries based on these findings.
