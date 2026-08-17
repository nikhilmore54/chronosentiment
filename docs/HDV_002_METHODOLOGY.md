# HDV-002 Risk-Boundary Research — Methodology & Freeze Gate

**Status:** FROZEN  
**Frozen:** 2026-08-17  
**Prerequisite:** HDV-001-F PASS (commit a7919ce06)

---

## 1. Research Boundary

HDV-001 established that Coralys direction contains measurable information
relative to random and inverse-direction baselines (+6.0 pp and +12.4 pp
respectively), satisfying the pre-specified criterion in 2 of 4 state regimes.

HDV-002 asks a different and narrower question:

> **Given that Coralys has demonstrated directional information, can an
> appropriate risk boundary extract more economic value from that information
> than the current C3-002 policy?**

HDV-002 is NOT:
- A re-examination of whether Coralys direction is valid
- An attempt to improve Coralys direction prediction
- An optimisation of the +0.8 pp momentum gap
- A retrospective selection of the best-performing state regime

HDV-002 IS:
- A structured investigation of risk-boundary policy
- Evaluated against pre-specified economic metrics
- Conducted on a population that keeps the HDV-001 development sample out
  of parameter selection
- Governed by a freeze gate before any parameter is examined

---

## 2. What Must Not Change

The following are frozen and must not be modified during HDV-002:

| Item | Status |
|------|--------|
| C3-002 decision logic | FROZEN |
| Coralys direction signal | FROZEN |
| HDV-001 development sample (728 COMPLETE decisions) | FROZEN — not used for parameter selection |
| Reference-risk boundary formula | FROZEN as baseline |
| Decision Intelligence v0.1 | FROZEN |

---

## 3. Research Question

What risk-boundary policy, applied to Coralys decisions, produces the best
risk-adjusted economic outcome on out-of-sample data, relative to the current
C3-002 reference-risk baseline?

---

## 4. Evaluation Population

**Eligible decisions:** All Coralys decisions from the HDV-001 validation
period (2026-08-18 onwards) and holdout period (2026-11-01 onwards), as
defined in docs/HDV_001_PERIODS.md.

**Excluded from parameter selection:** The 728 COMPLETE decisions from the
HDV-001 development period (2026-07-14 to 2026-08-13). These may be used
for descriptive reference only.

**Minimum sample for parameter selection:** To be determined once the
validation period has accumulated sufficient decisions (target: >= 200
COMPLETE decisions).

---

## 5. Out-of-Sample Split

| Period | Dates | Role |
|--------|-------|------|
| Development (HDV-001) | 2026-07-14 to 2026-08-13 | Reference only — not used for selection |
| Validation | 2026-08-18 to 2026-10-31 | Parameter selection |
| Holdout (test) | 2026-11-01 to 2026-12-31 | Final evaluation — unseen until policy frozen |

The holdout period must remain completely unseen until the risk-boundary
policy is frozen. No parameter may be adjusted after examining holdout results.

---

## 6. Candidate Boundary Families

The following families are pre-specified. No other families may be introduced
after this document is frozen.

**Family A — Symmetric percentage boundaries**
- Target distance = k × declared_stop_distance_pct
- Stop distance = declared_stop_distance_pct
- k ∈ {0.5, 1.0, 1.5, 2.0, 2.5, 3.0}
- k = 1.0 is the C3-002 baseline

**Family B — Volatility-scaled boundaries**
- Target and stop distances scaled by ATR(N) at decision time
- N ∈ {5, 10, 14, 20}
- Requires ATR enrichment of the decision dataset

**Family C — State-conditional boundaries**
- Different k values per Coralys state (Bullish/Bearish × Positive/Negative)
- Uses the 4-state segmentation from HDV-001-E
- Maximum 4 free parameters (one k per state)

**Family D — Asymmetric boundaries**
- Target distance ≠ stop distance
- Target multiplier t ∈ {1.0, 1.5, 2.0, 2.5}
- Stop multiplier s ∈ {0.5, 0.75, 1.0, 1.25}
- Grid search: 4 × 4 = 16 combinations

---

## 7. Objective Function

The primary objective is **risk-adjusted expectancy per decision**, defined as:

```
expectancy = (target_hit_rate × mean_favorable_excursion)
           - (stop_hit_rate  × mean_adverse_excursion)
```

Secondary metrics (reported but not used for selection):
- TARGET_BEFORE_RISK rate
- Median MFE at session 10
- Median MAE at session 10
- Maximum drawdown (sequence of consecutive RISK_BEFORE_TARGET outcomes)
- Profit factor (sum of favorable / sum of adverse excursions)

The primary objective must be defined and frozen before any parameter is
examined. It must not be changed after results are seen.

---

## 8. Multiple-Comparison Protection

**Maximum comparisons:** 50 parameter combinations across all families.
If a family requires more than 50 combinations, it must be reduced before
the search begins.

**Selection rule:** The policy with the highest primary objective on the
validation set is selected. No post-hoc adjustment is permitted.

**Final evaluation:** The selected policy is evaluated exactly once on the
holdout set. The holdout result is the official HDV-002 finding.

---

## 9. Constraints

A candidate policy is only eligible for selection if:

1. It does not increase mean adverse excursion by more than 20% relative
   to the C3-002 baseline on the validation set.
2. It does not produce a maximum drawdown (consecutive losses) more than
   50% worse than the C3-002 baseline.
3. It is implementable without modifying C3-002 decision logic.

---

## 10. Governance

A successful HDV-002 experiment may **recommend** a change to the
reference-risk boundary.

It must NOT directly modify C3-002 until:
1. The holdout evidence gate passes.
2. A separate implementation review is completed.
3. The change is committed as a versioned update to C3-002 with full
   provenance documentation.

---

## 11. HDV-002 Milestones

| Milestone | Description | Status |
|-----------|-------------|--------|
| HDV-002-A | Methodology freeze gate (this document) | FROZEN |
| HDV-002-B | Validation period price cache | Pending |
| HDV-002-C | ATR enrichment (for Family B) | Pending |
| HDV-002-D | Parameter search on validation set | Pending |
| HDV-002-E | Policy selection and freeze | Pending |
| HDV-002-F | Holdout evaluation | Pending |
| HDV-002-G | Official determination | Pending |

HDV-002-D must not begin until HDV-002-B and HDV-002-C are complete and
the validation set has accumulated >= 200 COMPLETE decisions.

---

## 12. Relationship to HDV-001

| Programme | Question | Evidence |
|-----------|----------|---------|
| HDV-001 | Does Coralys direction contain information? | PASS — +6.0 pp vs random, +12.4 pp vs inverse, 2/4 segments |
| HDV-002 | Can a better risk boundary extract more value? | TBD |

HDV-001 evidence is not re-examined in HDV-002. The directional signal is
taken as established. HDV-002 asks only about the risk boundary.

---

*This document is frozen. No parameter, family, objective function, or
evaluation population may be changed after this freeze.*