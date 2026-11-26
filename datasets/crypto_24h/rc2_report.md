# RC-2: LOW_VOL_DOWN / R2 Diagnostic Stability Test

We isolate the `LOW_VOL_DOWN` regime and the `R2` formulation (`-z_upside_excursion`) to analyze whether its positive rank association is broadly distributed or concentrated in a few anomalous chronological blocks.

## Horizon: 120m

### 1. Conditional Quintile Curves (R2 vs Mean Return)

| Split | Q1 (Low R2/High Excursion) | Q2 | Q3 | Q4 | Q5 (High R2/Low Excursion) |
|---|---|---|---|---|---|
| Train | 0.029% | -0.175% | -0.152% | 0.109% | 0.084% |
| Validation | 0.005% | -0.108% | -0.148% | 0.130% | 0.191% |

### 2. Validation Block Breakdown

**Validation Positive Blocks:** 3 / 9 (33.3%)

| Block_ID | N | Spearman | Upside_Mean | Ret_Mean |
|---|---|---|---|---|
| 41343.0000 | 418.0000 | -0.1264 | -0.9995 | -0.0006 |
| 41344.0000 | 563.0000 | 0.7255 | -0.9439 | 0.0018 |
| 41345.0000 | 27.0000 | -0.9371 | -0.1030 | 0.0038 |
| 41346.0000 | 40.0000 | -0.5188 | -0.8272 | 0.0006 |
| 41347.0000 | 140.0000 | -0.2739 | -1.2076 | 0.0011 |
| 41348.0000 | 606.0000 | 0.3737 | -1.0944 | 0.0004 |
| 41349.0000 | 102.0000 | 0.1380 | -0.8823 | 0.0035 |
| 41350.0000 | 29.0000 | -0.6777 | -0.4834 | 0.0027 |
| 41351.0000 | 349.0000 | -0.0136 | -0.9391 | -0.0042 |

## Horizon: 300m

### 1. Conditional Quintile Curves (R2 vs Mean Return)

| Split | Q1 (Low R2/High Excursion) | Q2 | Q3 | Q4 | Q5 (High R2/Low Excursion) |
|---|---|---|---|---|---|
| Train | 0.012% | 0.017% | -0.114% | 0.123% | -0.024% |
| Validation | -0.037% | -0.091% | -0.055% | 0.370% | 0.433% |

### 2. Validation Block Breakdown

**Validation Positive Blocks:** 5 / 9 (55.6%)

| Block_ID | N | Spearman | Upside_Mean | Ret_Mean |
|---|---|---|---|---|
| 41343.0000 | 418.0000 | -0.1955 | -0.9995 | -0.0015 |
| 41344.0000 | 563.0000 | 0.6640 | -0.9439 | 0.0057 |
| 41345.0000 | 27.0000 | -0.7766 | -0.1030 | 0.0040 |
| 41346.0000 | 40.0000 | -0.0801 | -0.8272 | 0.0015 |
| 41347.0000 | 140.0000 | -0.5460 | -1.2076 | 0.0009 |
| 41348.0000 | 606.0000 | 0.3826 | -1.0944 | 0.0024 |
| 41349.0000 | 102.0000 | 0.4661 | -0.8823 | 0.0034 |
| 41350.0000 | 29.0000 | 0.4954 | -0.4834 | 0.0026 |
| 41351.0000 | 349.0000 | 0.4662 | -0.9391 | -0.0056 |

