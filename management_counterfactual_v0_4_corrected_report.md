# Management Behaviour Census v0.4 — E4-PP-050 Counterfactual (Validation Correction)

> Dataset: LONG ASOF Sep 9 + Sep 15
> Policy: E4-PP-050 (+0.05% entry-protection stop after +0.50% MFE)

## 1. Trigger Status
- Total positions evaluated: 87
- TRIGGERED (+0.50% MFE reached): 78 (89.7%)
- NOT_TRIGGERED: 9 (10.3%)

### ALL POSITIONS (n=87)
**Outcome & IMV**
- Baseline Realized: -0.23%
- Managed Realized: 0.11%
- Incremental Management Value (IMV): **Mean: +0.34% | Median: +0.24% | Sum: +29.87%**

**MFE Preservation (Exit-Censored)**
- Baseline MFE (until exit): 1.33%
- Managed MFE (until exit): 1.32%
- Profit Capture (Managed Realized / Trigger Return): 11.51%

**Round-Trip Reduction**
- Baseline Round Trips (>0.5% then <0): 62
- Managed Round Trips: 4
- Δ Round Trips: -58

**Tail-Loss Distribution (Managed vs Baseline)**
- Mean MAE: -0.02% (vs -0.41% base)
- Median MAE: 0.00% (vs -0.30% base)
- P10 Realized: -0.04% (vs -0.95% base)
- P25 Realized: 0.05% (vs -0.54% base)
- Median Realized: 0.05% (vs -0.22% base)
- Worst Trade: -1.24% (vs -2.13% base)

### TRIGGERED POSITIONS ONLY (n=78)
**Outcome & IMV**
- Baseline Realized: -0.25%
- Managed Realized: 0.13%
- Incremental Management Value (IMV): **Mean: +0.38% | Median: +0.27% | Sum: +29.87%**

**MFE Preservation (Exit-Censored)**
- Baseline MFE (until exit): 1.42%
- Managed MFE (until exit): 1.41%
- Profit Capture (Managed Realized / Trigger Return): 11.51%

**Round-Trip Reduction**
- Baseline Round Trips (>0.5% then <0): 61
- Managed Round Trips: 3
- Δ Round Trips: -58

**Tail-Loss Distribution (Managed vs Baseline)**
- Mean MAE: 0.02% (vs -0.42% base)
- Median MAE: 0.01% (vs -0.29% base)
- P10 Realized: 0.04% (vs -0.91% base)
- P25 Realized: 0.05% (vs -0.55% base)
- Median Realized: 0.05% (vs -0.22% base)
- Worst Trade: -0.05% (vs -2.13% base)

