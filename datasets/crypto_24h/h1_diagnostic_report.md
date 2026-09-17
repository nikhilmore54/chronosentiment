# Stage C.1: H1 Diagnostic Checks (Validation Only)

Hypothesis: `LOW_VOL_UP / R1 / 300m`

## 1. Quintile Ordering & 2. Score-bin Occupancy
| Quintile | N | Mean_Ret | Median_Ret |
|---|---|---|---|
| Q1 (Low R1) | 764 | -3.8bps | -0.5bps |
| Q2 | 763 | -15.2bps | -19.8bps |
| Q3 | 764 | -23.8bps | -17.2bps |
| Q4 | 763 | -8.5bps | -8.7bps |
| Q5 (High R1) | 764 | 27.1bps | 16.3bps |

## 3. Block-level Distribution
| Block_ID | N | Spearman |
|---|---|---|
| 41340.0000 | 295.0000 | -0.3505 |
| 41341.0000 | 337.0000 | 0.0816 |
| 41342.0000 | 285.0000 | -0.0650 |
| 41343.0000 | 237.0000 | 0.0950 |
| 41344.0000 | 463.0000 | 0.8089 |
| 41345.0000 | 328.0000 | 0.0382 |
| 41346.0000 | 390.0000 | 0.6548 |
| 41347.0000 | 321.0000 | 0.3099 |
| 41348.0000 | 380.0000 | 0.5168 |
| 41349.0000 | 312.0000 | 0.5932 |
| 41350.0000 | 389.0000 | -0.3598 |
| 41351.0000 | 81.0000 | 0.8476 |

## 4. Conditional Return & 5. Sign Symmetry
| Subset | N | Mean_Ret | Median_Ret |
|---|---|---|---|
| High R1 (> Median) | 1909 | 0.6bps | -1.1bps |
| Low R1 (<= Median) | 1909 | -10.3bps | -7.6bps |

## 6. Outlier/Influence Check
| Sample | N | Spearman |
|---|---|---|
| Full Validation | 3818 | 0.1833 |
| Trimmed (1% Tails Removed) | 3740 | 0.1661 |

