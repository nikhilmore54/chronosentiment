# Canonical Paper Trading Replay Performance Analysis (Model A)
**Status**: `VALID — NEGATIVE AGGREGATE RESULT — DIAGNOSTIC COMPLETE — PROSPECTIVE VALIDATION PENDING`  
**Execution Window**: 12-Hour Historical Intraday Replay (720 1-Minute Steps)  
**Architecture Contract**: Model A — Frozen Paper Trading Temporal-Exit Architecture  
**Decision Engine**: Frozen Coralys v0.1.0 Multi-Resolution Trajectory Engine (`T0` Firewall)  
**Asset Universe**: BTC-USD, ETH-USD, XRP-USD, ADA-USD, SOL-USD, DOGE-USD  
**Starting Capital**: $100,000.00  
**Position Allocation**: $10,000.00 per Trade  

---

### Executive Performance Summary

| Performance Metric | Model A (Canonical Result) | Contract Specification |
| :--- | :---: | :--- |
| **Total Canonical Trades** | `88` Trades | Generated strictly by DecisionState transitions |
| **Win Rate** | `37.50%` | `33` Wins / `53` Losses |
| **Gross Profit** | `$+860.33` | Sum of positive trade exits |
| **Gross Loss** | `$1,161.58` | Sum of negative trade exits |
| **Net Realized PnL** | **`$-301.25`** | **`-0.30%` Portfolio Yield** |
| **Profit Factor** | **`0.74`** | Ratio of Gross Profit to Gross Loss |

---

### Four Canonical Temporal Exit Reasons Analysis

| Exit Reason | Description | Trade Count | Net PnL ($) | Avg Duration |
| :--- | :--- | :---: | :---: | :---: |
| **`ENGINE_EXIT`** | Fired when Coralys DecisionState flipped to `EXIT` / opposite signal | `60` | `$-794.65` | `30.0m` |
| **`TARGET_EXIT`** | Planned holding horizon boundary reached (60m) | `22` | `$+469.29` | `60.0m` |
| **`MAXIMUM_TIME_EXIT`** | Hard maximum exit boundary reached (180m) | `0` | `$+0.00` | `180.0m` |
| **`USER_EXIT`** | Manual intervention (N/A for automated backtest) | `0` | `$0.00` | `0.0m` |
| **`SIMULATION_END`** | Open position closed at 12h simulation boundary | `6` | `$+24.11` | `14.0m` |

---

### Per-Asset Breakdown (Model A)

| Symbol | Trades | Win Rate (%) | Net PnL ($) | Primary Exit Trigger |
| :--- | :---: | :---: | :---: | :--- |
| `BTC-USD` | `14` | `35.71%` | `$-24.42` | Canonical State Transition |
| `ETH-USD` | `13` | `38.46%` | `$-75.20` | Canonical State Transition |
| `XRP-USD` | `14` | `57.14%` | `$+57.21` | Canonical State Transition |
| `ADA-USD` | `16` | `37.50%` | `$-46.28` | Canonical State Transition |
| `SOL-USD` | `16` | `18.75%` | `$-142.05` | Canonical State Transition |
| `DOGE-USD` | `15` | `40.00%` | `$-70.51` | Canonical State Transition |


---

### Canonical Trade Journal Log (Model A Replay)

| Trade ID | Symbol | Side | Entry Price | Exit Price | Duration | Exit Reason | Entry State | Exit State | Net PnL ($) |
| :--- | :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: |
| `PT-CANON-0001` | `ETH-USD` | `LONG` | `$2,689.45` | `$2,690.78` | `49m` | `ENGINE_EXIT` | `BUY` | `EXIT` | `$+5.32` |
| `PT-CANON-0002` | `SOL-USD` | `LONG` | `$121.03` | `$121.75` | `52m` | `ENGINE_EXIT` | `BUY` | `EXIT` | `$+61.20` |
| `PT-CANON-0003` | `ADA-USD` | `LONG` | `$0.26` | `$0.26` | `50m` | `ENGINE_EXIT` | `BUY` | `EXIT` | `$+61.30` |
| `PT-CANON-0004` | `XRP-USD` | `LONG` | `$1.55` | `$1.55` | `54m` | `ENGINE_EXIT` | `BUY` | `EXIT` | `$+18.85` |
| `PT-CANON-0005` | `DOGE-USD` | `LONG` | `$0.10` | `$0.10` | `60m` | `TARGET_EXIT` | `BUY` | `PULLBACK` | `$+57.12` |
| `PT-CANON-0006` | `BTC-USD` | `LONG` | `$84,047.68` | `$84,095.59` | `60m` | `TARGET_EXIT` | `BUY` | `PULLBACK` | `$+4.79` |
| `PT-CANON-0007` | `XRP-USD` | `SHORT` | `$1.54` | `$1.55` | `4m` | `ENGINE_EXIT` | `SHORT` | `EXIT` | `$-20.80` |
| `PT-CANON-0008` | `ETH-USD` | `LONG` | `$2,692.69` | `$2,690.76` | `31m` | `ENGINE_EXIT` | `BUY` | `EXIT` | `$-7.72` |
| `PT-CANON-0009` | `XRP-USD` | `LONG` | `$1.55` | `$1.55` | `22m` | `ENGINE_EXIT` | `BUY` | `SHORT` | `$-22.10` |
| `PT-CANON-0010` | `SOL-USD` | `LONG` | `$121.62` | `$121.51` | `3m` | `ENGINE_EXIT` | `BUY` | `EXIT` | `$-8.80` |
| `PT-CANON-0011` | `BTC-USD` | `LONG` | `$84,149.98` | `$84,069.60` | `46m` | `ENGINE_EXIT` | `BUY` | `EXIT` | `$-8.04` |
| `PT-CANON-0012` | `ETH-USD` | `SHORT` | `$2,687.30` | `$2,688.41` | `19m` | `ENGINE_EXIT` | `SHORT` | `EXIT` | `$-4.44` |
| `PT-CANON-0013` | `ADA-USD` | `SHORT` | `$0.26` | `$0.26` | `14m` | `ENGINE_EXIT` | `SHORT` | `EXIT` | `$-13.65` |
| `PT-CANON-0014` | `ETH-USD` | `LONG` | `$2,688.46` | `$2,688.43` | `14m` | `ENGINE_EXIT` | `BUY` | `EXIT` | `$-0.12` |
| `PT-CANON-0015` | `XRP-USD` | `SHORT` | `$1.55` | `$1.54` | `60m` | `TARGET_EXIT` | `SHORT` | `WATCH` | `$+60.45` |
| `PT-CANON-0016` | `ADA-USD` | `SHORT` | `$0.26` | `$0.26` | `22m` | `ENGINE_EXIT` | `SHORT` | `BUY` | `$-21.84` |
| `PT-CANON-0017` | `DOGE-USD` | `SHORT` | `$0.10` | `$0.10` | `5m` | `ENGINE_EXIT` | `SHORT` | `BUY` | `$-13.26` |
| `PT-CANON-0018` | `DOGE-USD` | `LONG` | `$0.10` | `$0.10` | `5m` | `ENGINE_EXIT` | `BUY` | `SHORT` | `$-26.52` |
| `PT-CANON-0019` | `ETH-USD` | `LONG` | `$2,690.38` | `$2,687.46` | `18m` | `ENGINE_EXIT` | `BUY` | `SHORT` | `$-11.68` |
| `PT-CANON-0020` | `ADA-USD` | `LONG` | `$0.26` | `$0.26` | `19m` | `ENGINE_EXIT` | `BUY` | `SHORT` | `$-30.34` |
| `PT-CANON-0021` | `BTC-USD` | `LONG` | `$84,078.71` | `$84,049.25` | `56m` | `ENGINE_EXIT` | `BUY` | `SHORT` | `$-2.95` |
| `PT-CANON-0022` | `SOL-USD` | `LONG` | `$121.37` | `$121.20` | `51m` | `ENGINE_EXIT` | `BUY` | `SHORT` | `$-13.60` |
| `PT-CANON-0023` | `DOGE-USD` | `SHORT` | `$0.10` | `$0.10` | `60m` | `TARGET_EXIT` | `SHORT` | `PULLBACK` | `$+30.60` |
| `PT-CANON-0024` | `ETH-USD` | `SHORT` | `$2,687.46` | `$2,685.36` | `60m` | `TARGET_EXIT` | `SHORT` | `PULLBACK` | `$+8.40` |
| `PT-CANON-0025` | `SOL-USD` | `SHORT` | `$121.20` | `$121.22` | `40m` | `ENGINE_EXIT` | `SHORT` | `EXIT` | `$-1.70` |
| `PT-CANON-0026` | `BTC-USD` | `SHORT` | `$84,049.25` | `$84,018.07` | `41m` | `ENGINE_EXIT` | `SHORT` | `EXIT` | `$+3.12` |
| `PT-CANON-0027` | `XRP-USD` | `SHORT` | `$1.54` | `$1.53` | `60m` | `TARGET_EXIT` | `SHORT` | `PULLBACK` | `$+63.05` |
| `PT-CANON-0028` | `ADA-USD` | `SHORT` | `$0.26` | `$0.26` | `57m` | `ENGINE_EXIT` | `SHORT` | `EXIT` | `$+32.45` |
| `PT-CANON-0029` | `ETH-USD` | `SHORT` | `$2,683.29` | `$2,681.89` | `60m` | `TARGET_EXIT` | `SHORT` | `SHORT` | `$+5.60` |
| `PT-CANON-0030` | `BTC-USD` | `SHORT` | `$83,996.75` | `$84,024.35` | `51m` | `ENGINE_EXIT` | `SHORT` | `EXIT` | `$-2.76` |
| `PT-CANON-0031` | `ADA-USD` | `SHORT` | `$0.25` | `$0.25` | `60m` | `TARGET_EXIT` | `SHORT` | `SHORT` | `$+113.97` |
| `PT-CANON-0032` | `SOL-USD` | `SHORT` | `$121.14` | `$121.31` | `57m` | `ENGINE_EXIT` | `SHORT` | `BUY` | `$-14.45` |
| `PT-CANON-0033` | `DOGE-USD` | `SHORT` | `$0.10` | `$0.10` | `60m` | `TARGET_EXIT` | `SHORT` | `PULLBACK` | `$+52.53` |
| `PT-CANON-0034` | `XRP-USD` | `SHORT` | `$1.52` | `$1.52` | `60m` | `TARGET_EXIT` | `SHORT` | `PULLBACK` | `$+13.20` |
| `PT-CANON-0035` | `SOL-USD` | `LONG` | `$121.31` | `$121.13` | `11m` | `ENGINE_EXIT` | `BUY` | `EXIT` | `$-14.40` |
| `PT-CANON-0036` | `BTC-USD` | `LONG` | `$84,062.31` | `$83,922.78` | `15m` | `ENGINE_EXIT` | `BUY` | `SHORT` | `$-13.95` |
| `PT-CANON-0037` | `BTC-USD` | `SHORT` | `$83,922.78` | `$84,099.19` | `33m` | `ENGINE_EXIT` | `SHORT` | `BUY` | `$-17.64` |
| `PT-CANON-0038` | `SOL-USD` | `SHORT` | `$120.48` | `$121.39` | `47m` | `ENGINE_EXIT` | `SHORT` | `BUY` | `$-77.35` |
| `PT-CANON-0039` | `ETH-USD` | `SHORT` | `$2,674.62` | `$2,690.64` | `55m` | `ENGINE_EXIT` | `SHORT` | `BUY` | `$-64.08` |
| `PT-CANON-0040` | `XRP-USD` | `SHORT` | `$1.51` | `$1.53` | `58m` | `ENGINE_EXIT` | `SHORT` | `BUY` | `$-74.58` |
| `PT-CANON-0041` | `ADA-USD` | `SHORT` | `$0.25` | `$0.25` | `60m` | `TARGET_EXIT` | `SHORT` | `PULLBACK` | `$-61.29` |
| `PT-CANON-0042` | `DOGE-USD` | `SHORT` | `$0.10` | `$0.10` | `60m` | `TARGET_EXIT` | `SHORT` | `PULLBACK` | `$-33.28` |
| `PT-CANON-0043` | `SOL-USD` | `LONG` | `$121.39` | `$121.30` | `32m` | `ENGINE_EXIT` | `BUY` | `EXIT` | `$-7.20` |
| `PT-CANON-0044` | `ETH-USD` | `LONG` | `$2,690.64` | `$2,685.95` | `29m` | `ENGINE_EXIT` | `BUY` | `EXIT` | `$-18.76` |
| `PT-CANON-0045` | `BTC-USD` | `LONG` | `$84,099.19` | `$84,320.01` | `60m` | `TARGET_EXIT` | `BUY` | `BUY` | `$+22.08` |
| `PT-CANON-0046` | `ADA-USD` | `SHORT` | `$0.25` | `$0.25` | `22m` | `ENGINE_EXIT` | `SHORT` | `EXIT` | `$-47.24` |
| `PT-CANON-0047` | `XRP-USD` | `LONG` | `$1.53` | `$1.52` | `41m` | `ENGINE_EXIT` | `BUY` | `EXIT` | `$-15.18` |
| `PT-CANON-0048` | `DOGE-USD` | `SHORT` | `$0.10` | `$0.10` | `32m` | `ENGINE_EXIT` | `SHORT` | `EXIT` | `$-31.20` |
| `PT-CANON-0049` | `SOL-USD` | `LONG` | `$121.63` | `$121.07` | `38m` | `ENGINE_EXIT` | `BUY` | `SHORT` | `$-44.80` |
| `PT-CANON-0050` | `SOL-USD` | `SHORT` | `$121.07` | `$121.12` | `14m` | `ENGINE_EXIT` | `SHORT` | `EXIT` | `$-4.25` |
| `PT-CANON-0051` | `BTC-USD` | `LONG` | `$84,290.47` | `$84,266.36` | `9m` | `ENGINE_EXIT` | `BUY` | `EXIT` | `$-2.41` |
| `PT-CANON-0052` | `XRP-USD` | `SHORT` | `$1.52` | `$1.52` | `36m` | `ENGINE_EXIT` | `SHORT` | `BUY` | `$-13.86` |
| `PT-CANON-0053` | `ETH-USD` | `LONG` | `$2,688.34` | `$2,691.84` | `60m` | `TARGET_EXIT` | `BUY` | `BUY` | `$+14.00` |
| `PT-CANON-0054` | `ADA-USD` | `SHORT` | `$0.25` | `$0.25` | `41m` | `ENGINE_EXIT` | `SHORT` | `BUY` | `$-17.82` |
| `PT-CANON-0055` | `DOGE-USD` | `SHORT` | `$0.10` | `$0.10` | `38m` | `ENGINE_EXIT` | `SHORT` | `BUY` | `$-24.96` |
| `PT-CANON-0056` | `DOGE-USD` | `LONG` | `$0.10` | `$0.10` | `14m` | `ENGINE_EXIT` | `BUY` | `EXIT` | `$-11.33` |
| `PT-CANON-0057` | `ADA-USD` | `LONG` | `$0.25` | `$0.25` | `19m` | `ENGINE_EXIT` | `BUY` | `EXIT` | `$+8.69` |
| `PT-CANON-0058` | `SOL-USD` | `LONG` | `$121.42` | `$121.42` | `48m` | `ENGINE_EXIT` | `BUY` | `EXIT` | `$+0.00` |
| `PT-CANON-0059` | `BTC-USD` | `LONG` | `$84,326.86` | `$84,352.83` | `60m` | `TARGET_EXIT` | `BUY` | `PULLBACK` | `$+2.60` |
| `PT-CANON-0060` | `XRP-USD` | `LONG` | `$1.52` | `$1.53` | `60m` | `TARGET_EXIT` | `BUY` | `PULLBACK` | `$+9.90` |
| `PT-CANON-0061` | `DOGE-USD` | `LONG` | `$0.10` | `$0.10` | `42m` | `ENGINE_EXIT` | `BUY` | `SHORT` | `$-35.02` |
| `PT-CANON-0062` | `ADA-USD` | `LONG` | `$0.25` | `$0.25` | `50m` | `ENGINE_EXIT` | `BUY` | `SHORT` | `$-54.37` |
| `PT-CANON-0063` | `ETH-USD` | `LONG` | `$2,693.78` | `$2,691.78` | `58m` | `ENGINE_EXIT` | `BUY` | `EXIT` | `$-8.00` |
| `PT-CANON-0064` | `BTC-USD` | `LONG` | `$84,372.66` | `$84,261.28` | `30m` | `ENGINE_EXIT` | `BUY` | `SHORT` | `$-11.14` |
| `PT-CANON-0065` | `BTC-USD` | `SHORT` | `$84,261.28` | `$84,287.70` | `2m` | `ENGINE_EXIT` | `SHORT` | `EXIT` | `$-2.64` |
| `PT-CANON-0066` | `SOL-USD` | `SHORT` | `$120.88` | `$120.88` | `60m` | `TARGET_EXIT` | `SHORT` | `PULLBACK` | `$+0.00` |
| `PT-CANON-0067` | `DOGE-USD` | `SHORT` | `$0.10` | `$0.10` | `60m` | `TARGET_EXIT` | `SHORT` | `PULLBACK` | `$+18.72` |
| `PT-CANON-0068` | `XRP-USD` | `SHORT` | `$1.52` | `$1.52` | `60m` | `TARGET_EXIT` | `SHORT` | `WATCH` | `$+26.40` |
| `PT-CANON-0069` | `ADA-USD` | `SHORT` | `$0.25` | `$0.25` | `60m` | `TARGET_EXIT` | `SHORT` | `SHORT` | `$+64.94` |
| `PT-CANON-0070` | `ETH-USD` | `LONG` | `$2,694.42` | `$2,691.04` | `60m` | `TARGET_EXIT` | `BUY` | `PULLBACK` | `$-13.52` |
| `PT-CANON-0071` | `XRP-USD` | `SHORT` | `$1.52` | `$1.52` | `15m` | `ENGINE_EXIT` | `SHORT` | `BUY` | `$-34.98` |
| `PT-CANON-0072` | `DOGE-USD` | `SHORT` | `$0.10` | `$0.10` | `22m` | `ENGINE_EXIT` | `SHORT` | `BUY` | `$-43.68` |
| `PT-CANON-0073` | `ADA-USD` | `SHORT` | `$0.25` | `$0.25` | `18m` | `ENGINE_EXIT` | `SHORT` | `BUY` | `$-66.47` |
| `PT-CANON-0074` | `DOGE-USD` | `LONG` | `$0.10` | `$0.10` | `22m` | `ENGINE_EXIT` | `BUY` | `EXIT` | `$+5.20` |
| `PT-CANON-0075` | `SOL-USD` | `LONG` | `$121.11` | `$121.29` | `20m` | `ENGINE_EXIT` | `BUY` | `EXIT` | `$+15.30` |
| `PT-CANON-0076` | `ADA-USD` | `LONG` | `$0.25` | `$0.25` | `14m` | `ENGINE_EXIT` | `BUY` | `EXIT` | `$-1.19` |
| `PT-CANON-0077` | `BTC-USD` | `LONG` | `$84,368.55` | `$84,458.89` | `60m` | `TARGET_EXIT` | `BUY` | `PULLBACK` | `$+9.03` |
| `PT-CANON-0078` | `SOL-USD` | `LONG` | `$121.60` | `$121.41` | `3m` | `ENGINE_EXIT` | `BUY` | `EXIT` | `$-15.20` |
| `PT-CANON-0079` | `XRP-USD` | `LONG` | `$1.52` | `$1.53` | `41m` | `ENGINE_EXIT` | `BUY` | `EXIT` | `$+42.90` |
| `PT-CANON-0080` | `ADA-USD` | `LONG` | `$0.25` | `$0.25` | `18m` | `ENGINE_EXIT` | `BUY` | `EXIT` | `$-15.40` |
| `PT-CANON-0081` | `DOGE-USD` | `LONG` | `$0.10` | `$0.10` | `20m` | `ENGINE_EXIT` | `BUY` | `SHORT` | `$-17.51` |
| `PT-CANON-0082` | `SOL-USD` | `LONG` | `$121.60` | `$121.38` | `12m` | `ENGINE_EXIT` | `BUY` | `SHORT` | `$-17.60` |
| `PT-CANON-0083` | `ETH-USD` | `LONG` | `$2,694.95` | `$2,699.90` | `56m` | `SIMULATION_END` | `BUY` | `SIMULATION_END` | `$+19.80` |
| `PT-CANON-0084` | `BTC-USD` | `LONG` | `$84,503.30` | `$84,458.22` | `14m` | `SIMULATION_END` | `BUY` | `SIMULATION_END` | `$-4.51` |
| `PT-CANON-0085` | `XRP-USD` | `SHORT` | `$1.52` | `$1.52` | `5m` | `SIMULATION_END` | `SHORT` | `SIMULATION_END` | `$+3.96` |
| `PT-CANON-0086` | `ADA-USD` | `SHORT` | `$0.25` | `$0.25` | `3m` | `SIMULATION_END` | `SHORT` | `SIMULATION_END` | `$+1.98` |
| `PT-CANON-0087` | `SOL-USD` | `SHORT` | `$121.38` | `$121.37` | `3m` | `SIMULATION_END` | `SHORT` | `SIMULATION_END` | `$+0.80` |
| `PT-CANON-0088` | `DOGE-USD` | `SHORT` | `$0.10` | `$0.10` | `3m` | `SIMULATION_END` | `SHORT` | `SIMULATION_END` | `$+2.08` |


---

### Comparative Evaluation: Model A vs Model B

```text
MODEL A (Canonical Contract)              MODEL B (Exploratory Harness)
DecisionCard (Coralys v0.1.0)             Price Target + ±0.15 Momentum
       ↓                                         ↓
State Transition Entry                    Automated Price Touch Entry
       ↓                                         ↓
4 Temporal Exits                          Custom Trajectory Reversal Exits
 (ENGINE, TARGET, MAXIMUM, USER)           (Non-Canonical Exits)
```

1. **Strict Protocol Alignment**: Model A aligns 100% with the frozen `DecisionCard` and exit policy architecture contract.
2. **Traceable Decision Pipeline**: Every trade entry and exit maps directly to a state transition in the Coralys decision engine without custom momentum offsets or non-canonical exit labels.
