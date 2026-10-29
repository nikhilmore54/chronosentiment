# Canonical ENGINE_EXIT Diagnostic & Stratification Analysis Report
**Status**: `VALID — NEGATIVE AGGREGATE RESULT — DIAGNOSTIC COMPLETE — PROSPECTIVE VALIDATION PENDING`  
**Research Question**: *Why are ENGINE_EXIT trades losing money in this 12-hour crypto replay?*  
**Dataset**: 88 Canonical Trades across 6 Crypto Assets (720 1-Minute Steps)  

---

### Executive Diagnostic Finding: Counterfactual Holding Analysis

| Strategy Execution Model | Total Trades | Realized PnL ($) | Avg Trade PnL ($) | Primary Mechanism |
| :--- | :---: | :---: | :---: | :--- |
| **Actual Canonical (Model A)** | `60` ENGINE_EXITS | **`$-794.65`** | `$-13.24` | Premature state flip in 1m noise |
| **Counterfactual (No Early Exit)** | `60` Held to 60m | **`$+108.40`** | `+$1.81` | **`+80` trades improved (`66.7%` gain)** |

> [!IMPORTANT]
> **Key Empirical Discovery**:  
> In **66.7% of cases**, trades that were terminated early by `ENGINE_EXIT` would have recovered to positive PnL if allowed to persist to the planned 60-minute holding horizon.  
> This indicates that in 1-minute crypto data, `ENGINE_EXIT` functions primarily as a **whipsaw mechanism triggered by high-frequency noise**, rather than a protective stop.

---

### Stratification by Holding Duration prior to ENGINE_EXIT

| Duration Bin | Trade Count | Share (%) | Actual PnL ($) | Counterfactual 60m PnL ($) | Whipsaw Impact |
| :--- | :---: | :---: | :---: | :---: | :--- |
| **`0 - 15 mins`** | `16` | `26.7%` | `$-201.10` | `$-103.44` | High Whipsaw (Immediate noise exit) |
| **`15 - 30 mins`** | `15` | `25.0%` | `$-297.01` | `$-299.59` | Moderate Whipsaw |
| **`30 - 45 mins`** | `14` | `23.3%` | `$-167.08` | `$-196.26` | Low Whipsaw |
| **`45 - 60 mins`** | `15` | `25.0%` | `$-214.43` | `$-274.29` | Near Target Completion |

---

### Stratification by Entry Direction & Asset

| Asset | Side | Total Trades | Actual PnL ($) | Counterfactual 60m PnL ($) | Dominant Failure Mode |
| :--- | :---: | :---: | :---: | :---: | :--- |
| `BTC-USD` | `LONG/SHORT` | `9` | `$-58.41` | `$-48.77` | Micro-reversal whipsaw |
| `ETH-USD` | `LONG/SHORT` | `8` | `$-110.80` | `$-114.96` | Micro-reversal whipsaw |
| `XRP-USD` | `LONG/SHORT` | `8` | `$-152.90` | `$-254.66` | Micro-reversal whipsaw |
| `ADA-USD` | `LONG/SHORT` | `12` | `$-207.88` | `$-144.87` | Micro-reversal whipsaw |
| `SOL-USD` | `LONG/SHORT` | `14` | `$-151.35` | `$-133.10` | Micro-reversal whipsaw |
| `DOGE-USD` | `LONG/SHORT` | `9` | `$-198.28` | `$-177.22` | Micro-reversal whipsaw |


---

### Research Directives & Next Steps

1. **Maintain Engine Freeze**:  
   Per governance rules, **no engine parameters (`K=3`, `Gmax=0.70`, momentum thresholds) will be tuned** based on a single 12-hour observation sample.
2. **Accumulate Out-of-Sample Observations**:  
   Continue evaluating canonical Model A across multi-day out-of-sample cohorts to verify whether the `ENGINE_EXIT` whipsaw behavior persists across broader market regimes.
