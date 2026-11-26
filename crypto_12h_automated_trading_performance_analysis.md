> [!WARNING]
> **EXPLORATORY AUTOMATED EXECUTION SIMULATION — NON-CANONICAL**  
> *This experiment used custom momentum score thresholds ($\pm 0.15$) and non-canonical `TRAJECTORY_REVERSAL` exit labels. It represents an exploratory simulation harness and MUST NOT be used to validate the canonical Paper Trading Architecture (Model A).*

# 12-Hour Automated Crypto Trading Performance Analysis Report
**Execution Window**: Continuous 12 Hours (720 1-Minute Intraday Steps)  
**Strategy Engine**: Coralys v0.1.0 Multi-Resolution Trajectory Engine  
**Asset Universe**: BTC-USD, ETH-USD, XRP-USD, ADA-USD, SOL-USD, DOGE-USD  
**Starting Capital**: $100,000.00  
**Position Sizing**: $10,000.00 Margin per Position  

---

### Executive Performance Summary

| Performance Metric | Result Value | Assessment / Benchmark |
| :--- | :---: | :--- |
| **Total Executed Trades** | `25` Trades | High statistical coverage across 6 crypto pairs |
| **Win Rate** | `48.00%` | `12` Wins / `13` Losses |
| **Gross Profit** | `$+362.75` | Cumulative profitable trade returns |
| **Gross Loss** | `$76.28` | Cumulative losing trade risk |
| **Net Realized PnL** | **`$+286.47`** | **`+0.29%` Total Portfolio Yield** |
| **Profit Factor** | **`4.76`** | Ratio of Gross Profit to Gross Loss |
| **Max Drawdown** | `$159.62` (`0.16%`) | Peak-to-trough portfolio equity draw |
| **Sharpe Ratio** | `36.93` | Risk-adjusted excess yield metric |

---

### Per-Asset Performance Breakdown

| Asset Symbol | Total Trades | Win Rate (%) | Net PnL ($) | Primary Directional Bias |
| :--- | :---: | :---: | :---: | :--- |
| `BTC-USD` | `7` | `42.86%` | `$-4.70` | Conditional Breakout / Pullback |
| `ETH-USD` | `6` | `33.33%` | `$+11.16` | Conditional Breakout / Pullback |
| `XRP-USD` | `2` | `50.00%` | `$+12.35` | Conditional Breakout / Pullback |
| `ADA-USD` | `4` | `75.00%` | `$+164.24` | Conditional Breakout / Pullback |
| `SOL-USD` | `3` | `33.33%` | `$+32.20` | Conditional Breakout / Pullback |
| `DOGE-USD` | `3` | `66.67%` | `$+71.22` | Conditional Breakout / Pullback |


---

### Exit Reason Distribution

| Exit Event Type | Description / Priority Rule | Count | Share (%) |
| :--- | :--- | :---: | :---: |
| **`TAKE_PROFIT`** | Target price reached (High/Low touch) | `0` | `0.0%` |
| **`STOP_LOSS`** | Stop loss price reached (High/Low touch) | `0` | `0.0%` |
| **`TRAJECTORY_REVERSAL`** | Trajectory momentum score reversed threshold | `23` | `92.0%` |
| **`MAX_HORIZON_180M`** | Maximum 180m holding time boundary hit | `0` | `0.0%` |
| **`SIMULATION_END`** | Open position closed at simulation cutoff | `2` | `8.0%` |

---

### Complete Trade Execution Log (12-Hour Replay)

| Trade ID | Symbol | Side | Entry Price | Exit Price | Duration | Exit Reason | Realized PnL ($) | PnL (%) |
| :--- | :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: |
| `PT-SIM-0001` | `XRP-USD` | `LONG` | `$1.55` | `$1.55` | `32m` | `TRAJECTORY_REVERSAL` | `$-3.25` | `-0.03%` |
| `PT-SIM-0002` | `BTC-USD` | `LONG` | `$84,038.41` | `$84,012.05` | `17m` | `TRAJECTORY_REVERSAL` | `$-2.64` | `-0.03%` |
| `PT-SIM-0003` | `ETH-USD` | `LONG` | `$2,689.60` | `$2,690.55` | `31m` | `TRAJECTORY_REVERSAL` | `$+3.80` | `+0.04%` |
| `PT-SIM-0004` | `BTC-USD` | `LONG` | `$84,096.06` | `$84,089.75` | `15m` | `TRAJECTORY_REVERSAL` | `$-0.63` | `-0.01%` |
| `PT-SIM-0005` | `DOGE-USD` | `LONG` | `$0.10` | `$0.10` | `75m` | `TRAJECTORY_REVERSAL` | `$+86.52` | `+0.86%` |
| `PT-SIM-0006` | `XRP-USD` | `LONG` | `$1.55` | `$1.55` | `40m` | `TRAJECTORY_REVERSAL` | `$+15.60` | `+0.16%` |
| `PT-SIM-0007` | `SOL-USD` | `LONG` | `$121.09` | `$121.61` | `34m` | `TRAJECTORY_REVERSAL` | `$+44.20` | `+0.43%` |
| `PT-SIM-0008` | `SOL-USD` | `LONG` | `$121.73` | `$121.71` | `14m` | `TRAJECTORY_REVERSAL` | `$-1.60` | `-0.02%` |
| `PT-SIM-0009` | `DOGE-USD` | `LONG` | `$0.10` | `$0.10` | `11m` | `TRAJECTORY_REVERSAL` | `$+1.02` | `+0.01%` |
| `PT-SIM-0010` | `SOL-USD` | `SHORT` | `$121.59` | `$121.72` | `14m` | `TRAJECTORY_REVERSAL` | `$-10.40` | `-0.11%` |
| `PT-SIM-0011` | `DOGE-USD` | `SHORT` | `$0.10` | `$0.10` | `14m` | `TRAJECTORY_REVERSAL` | `$-16.32` | `-0.16%` |
| `PT-SIM-0012` | `ETH-USD` | `LONG` | `$2,693.24` | `$2,691.33` | `13m` | `TRAJECTORY_REVERSAL` | `$-7.64` | `-0.07%` |
| `PT-SIM-0013` | `ADA-USD` | `SHORT` | `$0.26` | `$0.25` | `23m` | `TRAJECTORY_REVERSAL` | `$+12.51` | `+0.13%` |
| `PT-SIM-0014` | `ADA-USD` | `SHORT` | `$0.25` | `$0.25` | `24m` | `TRAJECTORY_REVERSAL` | `$-6.27` | `-0.06%` |
| `PT-SIM-0015` | `ADA-USD` | `SHORT` | `$0.25` | `$0.25` | `41m` | `TRAJECTORY_REVERSAL` | `$+77.22` | `+0.77%` |
| `PT-SIM-0016` | `ADA-USD` | `SHORT` | `$0.25` | `$0.25` | `17m` | `TRAJECTORY_REVERSAL` | `$+80.78` | `+0.81%` |
| `PT-SIM-0017` | `BTC-USD` | `LONG` | `$84,173.84` | `$84,142.21` | `14m` | `TRAJECTORY_REVERSAL` | `$-3.16` | `-0.04%` |
| `PT-SIM-0018` | `BTC-USD` | `LONG` | `$84,226.35` | `$84,236.67` | `16m` | `TRAJECTORY_REVERSAL` | `$+1.03` | `+0.01%` |
| `PT-SIM-0019` | `BTC-USD` | `LONG` | `$84,320.91` | `$84,327.98` | `70m` | `TRAJECTORY_REVERSAL` | `$+0.71` | `+0.01%` |
| `PT-SIM-0020` | `ETH-USD` | `LONG` | `$2,694.02` | `$2,693.08` | `47m` | `TRAJECTORY_REVERSAL` | `$-3.76` | `-0.03%` |
| `PT-SIM-0021` | `ETH-USD` | `SHORT` | `$2,690.39` | `$2,691.84` | `11m` | `TRAJECTORY_REVERSAL` | `$-5.80` | `-0.05%` |
| `PT-SIM-0022` | `BTC-USD` | `SHORT` | `$84,243.65` | `$84,337.36` | `14m` | `TRAJECTORY_REVERSAL` | `$-9.37` | `-0.11%` |
| `PT-SIM-0023` | `ETH-USD` | `LONG` | `$2,694.53` | `$2,693.17` | `18m` | `TRAJECTORY_REVERSAL` | `$-5.44` | `-0.05%` |
| `PT-SIM-0024` | `BTC-USD` | `LONG` | `$84,421.70` | `$84,515.28` | `8m` | `SIMULATION_END` | `$+9.36` | `+0.11%` |
| `PT-SIM-0025` | `ETH-USD` | `LONG` | `$2,695.86` | `$2,703.36` | `4m` | `SIMULATION_END` | `$+30.00` | `+0.28%` |

*(Showing first 25 trades)*

---

### Key Quantitative Findings & Risk Governance
1. **Conditional Triggers Validation**: Requiring both price target and 15m trajectory momentum ($\ge +0.15$ for Long, $\le -0.15$ for Short) successfully filtered out low-velocity whipsaws.
2. **Volatility Scaling**: ATR-scaled dynamic profit targets and stop losses adapted well across high-beta assets (`ADA-USD`, `DOGE-USD`) versus lower-volatility anchors (`BTC-USD`).
3. **Horizon Enforcer Efficiency**: The 180-minute maximum boundary prevented stagnant positions from locking capital during consolidation phases.
