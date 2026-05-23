#!/bin/bash
set -e

echo "Starting Phase 2A Degradation Experiment: Native Tick vs 1m Kline"
cd core

# 1 Hour of ETF Approval Day (Jan 8 2024, 12:00 UTC to 13:00 UTC)
# Start: 1704715200000
# End: 1704718800000

echo "Fetching Tier 0 (Native Tick) Chronology..."
cargo run --release --bin historical_importer -- --symbol BTCUSDT --interval tick --start-time 1704715200000 --end-time 1704718800000 --name 2024_etf_approval_1h_tick

echo "Fetching Tier 1 (1m Kline) Chronology..."
cargo run --release --bin historical_importer -- --symbol BTCUSDT --interval 1m --start-time 1704715200000 --end-time 1704718800000 --name 2024_etf_approval_1h_1m

echo "Generating Trace Artifact for Tier 0..."
cargo run --release --bin trace_replay -- --substrate tier0_tick --substrate-file chronology/historical/2024_etf_approval_1h_tick/btcusdt_1704715200000.jsonl --topology osc_50_1.0 --cognition rolling_50

echo "Generating Trace Artifact for Tier 1..."
cargo run --release --bin trace_replay -- --substrate tier1_1m --substrate-file chronology/historical/2024_etf_approval_1h_1m/btcusdt_1704715200000.jsonl --topology osc_50_1.0 --cognition rolling_50

echo "Running Degradation Comparison..."
cd ..
python3 scripts/phase2a_degradation_study.py --tier0 core/artifacts/tier0_tick/osc_50_1.0/rolling_50/trace_v1.json --tier1 core/artifacts/tier1_1m/osc_50_1.0/rolling_50/trace_v1.json
