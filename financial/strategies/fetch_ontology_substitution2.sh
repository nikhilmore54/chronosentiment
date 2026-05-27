#!/bin/bash
set -e
cd core

echo "Fetching 2026_recent_discontinuity_1h_ontology (Binance BTCUSD Tick)..."
cargo run --release --bin historical_importer -- --symbol BTCUSD --interval tick --start-time 1779332400000 --end-time 1779336000000 --name 2026_recent_discontinuity_1h_ontology_binance_tick

mkdir -p chronology/historical/2026_recent_discontinuity_1h_ontology_yahoo_1m
cp chronology/historical/2026_recent_discontinuity_1h_yahoo_1m/*.jsonl chronology/historical/2026_recent_discontinuity_1h_ontology_yahoo_1m/
