# HDV-001-F Independent Baseline Comparison Report

**Generated:** 2026-08-17
**Source:** `datasets/hdv001/hdv001_outcomes_v1.json`
**Output:** `datasets/hdv001/hdv001_baseline_results_v1.json`
**N (COMPLETE decisions):** 728

## Frozen Success Criterion (from HDV-001-G Gate 6)

Coralys must beat both Random and Inverse by >= 5 percentage points in
TARGET_BEFORE_RISK rate, with the advantage appearing in >= 2 of 4 state segments.

## Aggregate Outcome Rates

| Model | TARGET | RISK | HORIZON |
|-------|--------|------|---------|
| Coralys | 35.7% | 41.5% | 22.8% |
| Random_A | 65.2% | 21.8% | 12.9% |
| Inverse_B | 96.7% | 3.3% | 0.0% |
| Momentum_C | 62.9% | 21.7% | 15.4% |

## Excursion Statistics

| Model | Median MFE | Median MAE | Mean MFE | Mean MAE |
|-------|-----------|-----------|---------|---------|
| Coralys | +2.803% | -1.270% | +3.715% | -1.657% |
| Random_A | +2.056% | -1.849% | +2.814% | -2.557% |
| Inverse_B | +1.270% | -2.803% | +1.657% | -3.715% |
| Momentum_C | +2.136% | -1.762% | +2.744% | -2.627% |

## State Segmentation (TARGET_BEFORE_RISK rate)

| State | Coralys | Random | Inverse | Momentum | Beats both by 5pp? |
|-------|---------|--------|---------|----------|-------------------|
| Bullish_Positive | 33.1% | 63.5% | 98.7% | 53.5% | no |
| Bullish_Negative | 22.1% | 54.0% | 97.4% | 57.5% | no |
| Bearish_Positive | 44.2% | 79.0% | 96.8% | 70.5% | no |
| Bearish_Negative | 42.5% | 67.4% | 93.7% | 75.1% | no |

## Random Sensitivity Analysis

| Metric | Value |
|--------|-------|
| Seeds | 1000 |
| Mean random TARGET rate | 66.3% |
| 95% interval | 63.2% -- 69.1% |
| Coralys rate | 35.7% |
| Coralys percentile | 0.0th |

## Frozen Success Criterion Result

| Check | Result | Status |
|-------|--------|--------|
| Coralys vs Random (>= +5pp) | -29.5pp | FAIL |
| Coralys vs Inverse (>= +5pp) | -61.0pp | FAIL |
| Segments beating both (>= 2) | 0/4 | FAIL |
| **OVERALL** | | **FAIL** |

## Governance Note

This criterion was frozen in HDV-001-G Gate 6 before baselines were run.
Do not modify C3-002 or reference-risk boundaries based on these findings.
If criterion PASS: proceed to risk-boundary research (HDV-002).
If criterion FAIL: do not resume stop-loss research.