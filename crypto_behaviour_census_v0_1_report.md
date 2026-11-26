# Crypto Behaviour Census v0.1: Results

## Data Audit
| Metric | Count |
|---|---|
| Raw observations | 4321 |
| Eligible T0 (after 1440m warmup) | 2881 |
| Complete 60-bar forward window | 2821 |
| Complete 300-bar forward window | 2581 (Census N) |
| UP trend (240m) | 2582 |
| DOWN trend (240m) | 0 |
| ZERO trend (240m) | 0 |

## Stage A — Future Behaviour Census (Unconditional)
Raw behavioral distributions across all valid T0.

### Terminal & Excursion Returns (60-bar constraints except ret)
| 15m Ret | 30m Ret | 60m Ret | 120m Ret | 300m Ret | MFE | MAE | Time-to-MFE | Time-to-MAE |
|---|---|---|---|---|---|---|---|---|
| 0.0006 | 0.0013 | 0.0025 | 0.0039 | 0.0127 | 0.0053 | -0.0025 | 37.0 | 15.0 |

### Path & Exit Behaviours
| Max Recovery (after MAE) | Max Giveback (after MFE) | Crossed +1% | Crossed -1% | Crossed +2% | Crossed -2% | Target (+5%) | Stop (-5%) | TimeStop |
|---|---|---|---|---|---|---|---|---|
| 0.0071 | 0.0057 | 20.72% | 5.54% | 6.93% | 0.12% | 0.00% | 0.00% | 100.00% |

## Stage B — State Conditioning (60-bar)
Partitioned by predefined state thresholds:
- VA_HIGH > 1.4638
- LOW_PERSISTENCE <= 0.1129

| VA State | Persistence State | N | Median 60m Ret | Hit Rate | Median MFE | Median MAE | % crossed +1% | % crossed -1% |
|---|---|---|---|---|---|---|---|---|
| HIGH | LOW | 309 | 0.0056 | 73.79% | 0.0077 | -0.0020 | 30.42% | 2.59% |
| HIGH | HIGH | 201 | 0.0046 | 78.11% | 0.0103 | -0.0027 | 52.24% | 7.46% |
| NORMAL | LOW | 1323 | 0.0028 | 71.66% | 0.0048 | -0.0020 | 14.21% | 2.87% |
| NORMAL | HIGH | 749 | 0.0003 | 52.20% | 0.0048 | -0.0039 | 19.76% | 10.95% |

## Stage C — Directional Decomposition
Evaluating the 60-bar outcome across trailing trend components.

| Trend | VA State | Persistence State | N | Median 60m Ret | Hit Rate | Median MFE | Median MAE |
|---|---|---|---|---|---|---|---|
| UP | HIGH | LOW | 309 | 0.0056 | 73.79% | 0.0077 | -0.0020 |
| UP | HIGH | HIGH | 201 | 0.0046 | 78.11% | 0.0103 | -0.0027 |
| UP | NORMAL | LOW | 1323 | 0.0028 | 71.66% | 0.0048 | -0.0020 |
| UP | NORMAL | HIGH | 749 | 0.0003 | 52.20% | 0.0048 | -0.0039 |

*Note: The current dataset contains exclusively UP trend observations (240m trailing).* 

## Stage D — Temporal Decomposition (300-bar)
Evolution of terminal returns across forward horizons (15m, 30m, 60m, 120m, 300m).

| VA State | Persistence State | Trend | 15m Med | 30m Med | 60m Med | 120m Med | 300m Med |
|---|---|---|---|---|---|---|---|
| HIGH | LOW | UP | 0.0006 | 0.0022 | 0.0056 | 0.0097 | 0.0196 |
| HIGH | HIGH | UP | 0.0013 | 0.0032 | 0.0046 | 0.0011 | 0.0064 |
| NORMAL | LOW | UP | 0.0006 | 0.0013 | 0.0028 | 0.0047 | 0.0156 |
| NORMAL | HIGH | UP | 0.0004 | 0.0005 | 0.0003 | 0.0017 | 0.0094 |

## Exit Behaviour Distribution
| VA State | Persistence State | Target (5%) | Stop (-5%) | TimeStop (60m) |
|---|---|---|---|---|
| HIGH | LOW | 0.00% | 0.00% | 100.00% |
| HIGH | HIGH | 0.00% | 0.00% | 100.00% |
| NORMAL | LOW | 0.00% | 0.00% | 100.00% |
| NORMAL | HIGH | 0.00% | 0.00% | 100.00% |
