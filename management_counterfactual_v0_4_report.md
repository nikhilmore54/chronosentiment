# Management Behaviour Census v0.4 — 0.50% Break-Even Stop Counterfactual

> Dataset: LONG ASOF Sep 9 + Sep 15

## 1. Trigger Status
- Total positions evaluated: 87
- TRIGGERED (+0.50% MFE reached): 78 (89.7%)
- NOT_TRIGGERED: 9 (10.3%)

### ALL POSITIONS (n=87)
**Outcome & IMV**
- Baseline Realized: -0.23%
- Managed Realized: 0.11%
- Incremental Management Value (IMV): **+0.34%**

**MFE Preservation & Capture**
- Baseline MFE: 1.33%
- Managed MFE: 1.32%
- MFE Capture (Managed Realized / Baseline MFE): 8.25%

**Round-Trip Reduction**
- Baseline Round Trips (>0.5% then <0): 62
- Managed Round Trips: 4
- Δ Round Trips: -58

**Tail-Loss Distribution (Managed vs Baseline)**
- Mean MAE: -0.02% (vs -0.40% base)
- P10 Realized: -0.04% (vs -0.95% base)
- P25 Realized: 0.05% (vs -0.54% base)
- Median Realized: 0.05% (vs -0.22% base)
- Worst Trade: -1.24% (vs -2.13% base)

### TRIGGERED POSITIONS ONLY (n=78)
**Outcome & IMV**
- Baseline Realized: -0.25%
- Managed Realized: 0.13%
- Incremental Management Value (IMV): **+0.38%**

**MFE Preservation & Capture**
- Baseline MFE: 1.42%
- Managed MFE: 1.41%
- MFE Capture (Managed Realized / Baseline MFE): 9.12%

**Round-Trip Reduction**
- Baseline Round Trips (>0.5% then <0): 61
- Managed Round Trips: 3
- Δ Round Trips: -58

**Tail-Loss Distribution (Managed vs Baseline)**
- Mean MAE: 0.02% (vs -0.40% base)
- P10 Realized: 0.04% (vs -0.91% base)
- P25 Realized: 0.05% (vs -0.55% base)
- Median Realized: 0.05% (vs -0.22% base)
- Worst Trade: -0.05% (vs -2.13% base)

