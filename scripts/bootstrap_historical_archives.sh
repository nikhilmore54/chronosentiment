#!/bin/bash
set -e

echo "Bootstrapping Canonical Historical Archives..."
cd core

# ETF Approval Week (Approx: Jan 8 2024 to Jan 15 2024)
# 1704672000000 to 1705276800000
echo "Fetching 2024_etf_approval..."
cargo run --release --bin historical_importer -- --symbol BTCUSDT --start-time 1704672000000 --end-time 1704758400000 --name 2024_etf_approval

# Add more ranges here as needed

echo "Done."
