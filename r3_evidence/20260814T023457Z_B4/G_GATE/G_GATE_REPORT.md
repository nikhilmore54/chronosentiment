# G-GATE Report

**Protocol:** G-Extension Methodology v1.1  
**Dataset:** B4 (read-only)  
**Seed:** `20260813`  
**Leakage audit:** PASS  
**Classification:** `INCONCLUSIVE`

Scientific question: does decision-time `signature_hash` contain statistically demonstrable predictive information for `Y_h = 1[outcome_return > 0]` beyond the frozen training-prevalence baseline, on the held-out chronological test fold?

## Horizon metrics (test fold)

| Horizon | N | Positive | Negative | AUC | 95% CI | ΔAUC | ΔAUC 95% CI | Brier | Cal. intercept | Cal. slope | p-value | Holm-adjusted p | Classification contribution |
|---------|---|----------|----------|-----|--------|------|-------------|-------|----------------|------------|---------|-----------------|------------------------------|
| 5D | 28 | 12 | 16 | 0.5000000000 | — | 0.0000000000 | — | 0.2449586777 | 0.3800865801 | 0.1111111111 | — | — | defined=false ΔAUC>0=false CI_lb>0=false holm_p<0.05=false |
| 10D | 28 | 12 | 16 | 0.5000000000 | [0.5000000000, 0.5000000000] | 0.0000000000 | [0.0000000000, 0.0000000000] | 0.2468476978 | — | — | 1.0000000000 | — | defined=false ΔAUC>0=false CI_lb>0=false holm_p<0.05=false |
| 20D | 28 | 7 | 21 | 0.5000000000 | [0.5000000000, 0.5000000000] | 0.0000000000 | [0.0000000000, 0.0000000000] | 0.2293388430 | 0.2500000000 | 0.0000000000 | 1.0000000000 | — | defined=true ΔAUC>0=false CI_lb>0=false holm_p<0.05=false |
| 60D | 28 | 6 | 22 | 0.5000000000 | — | 0.0000000000 | — | 0.2028571429 | 0.3285714286 | -0.2857142857 | — | — | defined=false ΔAUC>0=false CI_lb>0=false holm_p<0.05=false |

Blank / `—` values are undefined, not zero.

## Secondary rates

| Horizon | p_baseline | Observed event rate | Predicted event rate | Brier baseline | ΔBrier | Undefined bootstrap AUCs |
|---------|------------|---------------------|----------------------|----------------|--------|--------------------------|
| 5D | 0.4363636364 | 0.4285714286 | 0.4363636364 | 0.2449586777 | 0.0000000000 | 2 |
| 10D | 0.4727272727 | 0.4285714286 | 0.4727272727 | 0.2468476978 | 0.0000000000 | 0 |
| 20D | 0.4545454545 | 0.2500000000 | 0.4545454545 | 0.2293388430 | 0.0000000000 | 0 |
| 60D | 0.4000000000 | 0.2142857143 | 0.4000000000 | 0.2028571429 | 0.0000000000 | 4 |

## Reliability tables

### 10D

```json
[
  {
    "mean_p": 0.4727272727,
    "mean_y": 0.4285714286,
    "n": 28
  }
]
```

### 20D

```json
[
  {
    "mean_p": 0.4545454545,
    "mean_y": 0.25,
    "n": 28
  }
]
```

### 5D

```json
[
  {
    "mean_p": 0.4363636364,
    "mean_y": 0.4285714286,
    "n": 28
  }
]
```

### 60D

```json
[
  {
    "mean_p": 0.4,
    "mean_y": 0.2142857143,
    "n": 28
  }
]
```

## Classification rule applied

`PREDICTIVE_VALUE_DETECTED` requires leakage PASS, all four horizons metrics-defined, every ΔAUC > 0, every ΔAUC CI lower bound > 0, and every Holm-adjusted p < 0.05.

**Final result:** `INCONCLUSIVE`
