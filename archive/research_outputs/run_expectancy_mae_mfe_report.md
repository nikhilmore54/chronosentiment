# Expectancy and MAE/MFE Report

Aligned with `.cursor/rules/chronosentiment-core.mdc` and `docs/SRS_v1_6.md`.

- Source log: `run_data_driven_metrics.log`
- REC_OUTCOME samples: `45`
- TRADE_PATH samples: `45`

## True Expectancy

- WinRate: `44.44%`
- AvgWin: `0.023531`
- AvgLoss: `-0.015891`
- Expectancy(mean pnl): `0.001630`
- Expectancy check (WinRate*AvgWin - LossRate*|AvgLoss|): `0.001630`

## MFE / MAE Distribution Percentiles

- MFE P50/P60/P70/P80: `0.008191`, `0.011823`, `0.015332`, `0.028469`
- MAE P20/P30/P40: `-0.020892`, `-0.015915`, `-0.011325`

## Data-Derived Cutoff Suggestions

- Suggested TP (MFE P60): `0.011823`
- Suggested SL (MAE P30): `-0.015915`
- Approx theoretical expectancy with suggested TP/SL: `-0.001769`
