# HDV-001-D Decision Path Metrics Report

**Generated:** 2026-08-17
**Source:** `datasets/hdv001/hdv001_price_paths_v1.json`
**Output:** `datasets/hdv001/hdv001_decision_metrics_v1.json`

## Direction Normalization

Positive MFE/MAE = price moved in Coralys predicted direction.
Negative MFE/MAE = price moved against Coralys predicted direction.

LONG:  favorable_return = (close - reference_price) / reference_price
SHORT: favorable_return = (reference_price - close) / reference_price

## Statistics

| Metric | Value |
|--------|-------|
| Total decisions | 1144 |
| COMPLETE | 728 |
| MATURING | 416 |
| No sessions | 0 |

## Aggregate Summary (COMPLETE decisions only)

N = 728

| Session | Median MFE | Median MAE | % MFE > 0 |
|---------|-----------|-----------|-----------|
| 1 | +0.138% | +0.138% | 55.2% |
| 2 | +0.658% | -0.189% | 70.5% |
| 3 | +1.037% | -0.434% | 77.2% |
| 5 | +1.723% | -0.774% | 82.5% |
| 10 | +2.803% | -1.270% | 88.5% |

## Target and Stop Hit Rates (COMPLETE decisions)

| Metric | Count | Rate |
|--------|-------|------|
| Target hit within 10 sessions | 306 | 42.0% |
| Stop hit within 10 sessions | 313 | 43.0% |

## Notes

Primary analysis uses COMPLETE decisions (>= 10 sessions observed).
MATURING decisions are included in the output file but excluded from
aggregate summary statistics.

Do not modify C3-002 based on these findings.
HDV-001-G freeze gate must be passed before any implementation changes.