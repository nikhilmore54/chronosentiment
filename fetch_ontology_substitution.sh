#!/bin/bash
set -e
cd core

# 2026 Recent Crossfeed (Ontology Aligned)
echo "Fetching 2026_recent_crossfeed_1h_ontology (Binance BTCUSD Tick)..."
cargo run --release --bin historical_importer -- --symbol BTCUSD --interval tick --start-time 1779285600000 --end-time 1779289200000 --name 2026_recent_crossfeed_1h_ontology_binance_tick

# Note: Yahoo BTC-USD 1m is already fetched in 2026_recent_crossfeed_1h_yahoo_1m, we will just copy it
mkdir -p chronology/historical/2026_recent_crossfeed_1h_ontology_yahoo_1m
cp chronology/historical/2026_recent_crossfeed_1h_yahoo_1m/*.jsonl chronology/historical/2026_recent_crossfeed_1h_ontology_yahoo_1m/

# 2024 CPI Shock (Ontology Aligned)
echo "Fetching 2024_cpi_shock_1h_ontology (Binance BTCUSD Tick)..."
cargo run --release --bin historical_importer -- --symbol BTCUSD --interval tick --start-time 1707829200000 --end-time 1707832800000 --name 2024_cpi_shock_1h_ontology_tick
echo "Fetching 2024_cpi_shock_1h_ontology (Binance BTCUSD 1m)..."
cargo run --release --bin historical_importer -- --symbol BTCUSD --interval 1m --start-time 1707829200000 --end-time 1707832800000 --name 2024_cpi_shock_1h_ontology_1m
