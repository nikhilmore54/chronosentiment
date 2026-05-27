# ChronoSentiment — Propagation Ecology Observatory

A topology-aware propagation observatory for execution ecology research across crypto ecologies (BTC/ETH/SOL).

## Quick Start

```bash
# 1. Export replay data to JSON
python3 scripts/export_observatory_data.py

# 2. Serve the observatory
cd observatory && python3 -m http.server 8888

# 3. Open http://localhost:8888
```

## Architecture

```
archive/replay_1m_gen11.log          # Raw replay log (Rust engine output)
        │
scripts/export_observatory_data.py   # Parses TELEMETRY + AUDIT_TRADE → JSON
        │
observatory/data.json                # Structured trade + telemetry data
        │
observatory/                         # Frontend (HTML/CSS/JS, zero dependencies)
├── index.html                       # Page structure — 4 tab panels
├── style.css                        # Dark-mode design system
└── app.js                           # Canvas scatter plots, bar charts, tables
```

## Tabs

### 1. Execution Ecology
Summary dashboard: trade count, win rate, expectancy, Elastic Recovery Ratio, exit ecology distribution, per-asset performance, and PnL distribution scatter.

### 2. Smoothness Trap
Visualizes the core discovery: **high Directional Efficiency correlates with terminal exhaustion, not survivability**. Shows efficiency-vs-PnL scatter (color = exit type) and the monotonic topology inversion table.

### 3. Edge Genesis
Pre-entry microstructure analysis: Compression Release Ratio vs PnL, Pre-Entry Directional Bias vs PnL, and per-exit-type genesis conditions table. Demonstrates that winners enter from **compressed, directionless** pre-entry environments.

### 4. Toxicity Atlas
Temporal toxicity mapping: Elasticity Age vs PnL scatter, the logistic Freshness Decay curve, and age-binned toxicity clusters. Shows that 100% of false-elasticity mortalities occur at >10 bars since reload.

## Data Pipeline

To re-export from a different replay log:
```bash
python3 scripts/export_observatory_data.py archive/replay_1m_gen11.log observatory/data.json
```

The export script parses three log line types:
- `[TELEMETRY]` — atlas metrics (efficiency, density, resilience, age, genesis)
- `[REC_STATUS]` — links telemetry to trade rec_id
- `[AUDIT_TRADE]` — realized PnL, exit type, duration

## Key Metrics

| Metric | Definition |
|--------|-----------|
| Directional Efficiency | `net_move / cumulative_abs_move` |
| Compression Ratio | `exec_window_vol / pre_window_vol` |
| Elasticity Age | Bars since last liquidity reload |
| Elastic Recovery Ratio | `(TrailingStop + TakeProfit) / (StopLoss + Mortality)` |
| Freshness Decay | `1 / (1 + exp((age - 10) / 2.5))` |
