# Prospective Live Shadow Validation Analysis Report
**Status**: `PROSPECTIVE LIVE SHADOW VALIDATION — N=1 SESSION`  
**Execution Mode**: Genuinely Prospective Live Shadow Run ($T_0 = \text{NOW}$ per minute tick)  
**Universe**: BTC-USD, ETH-USD, XRP-USD, ADA-USD, SOL-USD, DOGE-USD  
**Engine & Contract**: Frozen Coralys v0.1.0 + Config A (Model A Temporal Exit Policy)  
**Session Progress**: 720 / 720 Live Ticks (100.0% Complete)  
**Start UTC**: `2026-09-27T02:49:18Z` | **Data Gaps**: `0` | **Daemon Interruptions**: `0`  

---

### Comparative Evaluation: Historical 12h Replay vs Live 12h Shadow Run

| Performance Metric | Historical 12h Replay (Model A) | Prospective Live 12h Shadow (Model A) | Observed Variance / Shift |
| :--- | :---: | :---: | :--- |
| **Total Canonical Trades** | `88` Trades | **`74` Trades** | Live market opportunity frequency |
| **Win Rate** | `37.50%` | **`35.14%`** | Win/loss ratio shift |
| **Gross Profit** | `$+860.33` | **`$+1,032.77`** | Positive exit payoff |
| **Gross Loss** | `$-1,161.58` | **`$1,206.23`** | Negative exit drag |
| **Net Realized PnL** | **`$-301.25`** | **`$-173.46`** | Net prospective yield |
| **Profit Factor** | **`0.74`** | **`0.86`** | Gross PnL efficiency ratio |
| **`ENGINE_EXIT` Trades** | `60` (`$-794.65`) | **`47` (`$-836.28`)** | Early state flip behavior |
| **`TARGET_EXIT` Trades** | `22` (`$+469.29`) | **`27` (`$+662.82`)** | 60m planned horizon completion |
| **`MAXIMUM_TIME_EXIT`** | `0` (`$+0.00`) | **`0` (`$+0.00`)** | 180m hard boundary hit |

---

### Per-Asset Prospective Live Results

| Instrument | Shadow Trades | Win Rate (%) | Net Realized PnL ($) | Active Positions |
| :--- | :---: | :---: | :---: | :---: |
| `BTC-USD` | `14` | `21.4%` | `$-37.24` | `OPEN` |
| `ETH-USD` | `10` | `40.0%` | `$-57.32` | `OPEN` |
| `XRP-USD` | `12` | `33.3%` | `$+29.26` | `OPEN` |
| `ADA-USD` | `13` | `46.2%` | `$-67.41` | `OPEN` |
| `SOL-USD` | `11` | `36.4%` | `$+222.85` | `OPEN` |
| `DOGE-USD` | `14` | `35.7%` | `$-263.60` | `OPEN` |


---

### Live Shadow Trade Journal (Latest Entries)

| Trade ID | Symbol | Side | Entry Price | Exit Price | Holding Mins | Exit Reason | Realized PnL ($) |
| :--- | :--- | :---: | :---: | :---: | :---: | :---: | :---: |
| `SHADOW-0060` | `ADA-USD` | `LONG` | `$0.26` | `$0.26` | `60m` | `TARGET_EXIT` | `$+54.46` |
| `SHADOW-0061` | `BTC-USD` | `LONG` | `$84,913.36` | `$84,894.76` | `56m` | `ENGINE_EXIT` | `$-1.86` |
| `SHADOW-0062` | `XRP-USD` | `LONG` | `$1.54` | `$1.54` | `60m` | `ENGINE_EXIT` | `$-13.65` |
| `SHADOW-0063` | `DOGE-USD` | `LONG` | `$0.10` | `$0.10` | `28m` | `ENGINE_EXIT` | `$-122.40` |
| `SHADOW-0064` | `BTC-USD` | `SHORT` | `$84,639.81` | `$84,906.38` | `14m` | `ENGINE_EXIT` | `$-26.66` |
| `SHADOW-0065` | `ETH-USD` | `SHORT` | `$2,711.50` | `$2,710.27` | `36m` | `ENGINE_EXIT` | `$+4.92` |
| `SHADOW-0066` | `ADA-USD` | `SHORT` | `$0.26` | `$0.26` | `32m` | `ENGINE_EXIT` | `$-46.92` |
| `SHADOW-0067` | `DOGE-USD` | `SHORT` | `$0.10` | `$0.10` | `11m` | `ENGINE_EXIT` | `$-61.80` |
| `SHADOW-0068` | `XRP-USD` | `SHORT` | `$1.52` | `$1.53` | `21m` | `ENGINE_EXIT` | `$-12.54` |
| `SHADOW-0069` | `SOL-USD` | `SHORT` | `$123.93` | `$123.02` | `55m` | `ENGINE_EXIT` | `$+72.80` |
| `SHADOW-0070` | `BTC-USD` | `LONG` | `$84,944.63` | `$84,908.85` | `5m` | `ENGINE_EXIT` | `$-3.58` |
| `SHADOW-0071` | `ADA-USD` | `SHORT` | `$0.26` | `$0.26` | `3m` | `ENGINE_EXIT` | `$-50.96` |
| `SHADOW-0072` | `DOGE-USD` | `LONG` | `$0.10` | `$0.10` | `6m` | `ENGINE_EXIT` | `$-10.20` |
| `SHADOW-0073` | `DOGE-USD` | `LONG` | `$0.10` | `$0.10` | `6m` | `ENGINE_EXIT` | `$-51.00` |
| `SHADOW-0074` | `BTC-USD` | `LONG` | `$84,993.53` | `$84,827.60` | `18m` | `ENGINE_EXIT` | `$-16.59` |


---

### Audit Logging Integrity
- **DecisionCard Per-Tick History**: Written atomically to `live_capture/shadow_validation/decision_cards_log.jsonl`
- **Shadow Trade Journal**: Written atomically to `live_capture/shadow_validation/shadow_trade_journal.jsonl`
- **Zero Forward Bias Guarantee**: Each minute tick fetches live prices at $T_0 = \text{NOW}$ via continuous daemon polling.
