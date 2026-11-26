# Regime and Distribution Analysis: Train vs Validation

## 1. Feature Distribution Shift

| Feature | Train Mean | Train Std | Val Mean | Val Std | Mean Shift (Z) |
|---|---|---|---|---|---|
| volatility_1m_std_24h | 0.0004 | 0.0001 | 0.0003 | 0.0001 | -0.9165 |
| upside_excursion_24h | 0.0112 | 0.0073 | 0.0086 | 0.0048 | -0.3619 |
| downside_excursion_24h | -0.0124 | 0.0099 | -0.0051 | 0.0036 | 0.7398 |
| trend_dir | 0.1172 | 0.9931 | 0.4498 | 0.8930 | 0.3349 |


## 2. Regime-Conditional Correlations (Spearman at 300m)

We evaluate the raw correlation of the strongest Train variables against forward returns, conditioned on the trailing regime.

### A. Downside Excursion

| Split | Target | High Vol Corr | Low Vol Corr | Up Trend Corr | Down Trend Corr | N |
|---|---|---|---|---|---|---|
| Train | downside_excursion_24h -> ret_300m | -0.2106 | -0.1101 | -0.1839 | -0.1036 | 24876 |
| Validation | downside_excursion_24h -> ret_300m | 0.2605 | -0.0286 | 0.0665 | 0.1331 | 8292 |


### B. Upside Excursion

| Split | Target | High Vol Corr | Low Vol Corr | Up Trend Corr | Down Trend Corr | N |
|---|---|---|---|---|---|---|
| Train | upside_excursion_24h -> ret_300m | -0.0293 | -0.2777 | -0.1536 | 0.0199 | 24876 |
| Validation | upside_excursion_24h -> ret_300m | 0.1828 | -0.0266 | 0.2696 | -0.3152 | 8292 |


