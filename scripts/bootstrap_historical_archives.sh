#!/bin/bash
set -e

echo "Bootstrapping Canonical Historical Archives..."
cd core

# ETF Approval Week (Approx: Jan 8 2024 to Jan 15 2024)
echo "Fetching 2024_etf_approval..."
cargo run --release --bin historical_importer -- --symbol BTCUSDT --interval 1m --start-time 1704672000000 --end-time 1704758400000 --name 2024_etf_approval

# 2026 Q1 Volatility Expansion (Jan 2 2026 to Jan 3 2026 as sample)
echo "Fetching 2026_q1_volatility_expansion..."
cargo run --release --bin historical_importer -- --symbol BTCUSDT --interval 1m --start-time 1767225600000 --end-time 1767312000000 --name 2026_q1_volatility_expansion

# 2026 FOMC March (March 18 2026)
echo "Fetching 2026_fomc_march..."
cargo run --release --bin historical_importer -- --symbol BTCUSDT --interval 1m --start-time 1773792000000 --end-time 1773878400000 --name 2026_fomc_march

# 2026 Weekend Liquidity Gap (April 4 2026 to April 5 2026)
echo "Fetching 2026_weekend_liquidity_gap..."
cargo run --release --bin historical_importer -- --symbol BTCUSDT --interval 1m --start-time 1775260800000 --end-time 1775347200000 --name 2026_weekend_liquidity_gap

# --- FUTURE CURATED TARGETS ---
# Do NOT bulk-scrape. Adhere strictly to bounded, semantic universes.
#
# Target: 2024 CPI Release Shock (e.g., Feb 13 2024)
# Reason: Burst compression
# cargo run --release --bin historical_importer -- --symbol BTCUSDT --interval 1m --start-time 1707829200000 --end-time 1707915600000 --name 2024_cpi_release
#
# Target: Liquidation Cascade (e.g., Aug 17 2023 or FTX collapse)
# Reason: Rapid occupancy shock
# 
# Target: Low-Volatility Drift Week
# Reason: Continuity persistence
#
# Target: Exchange Outage Period
# Reason: Explicit discontinuity truth

echo "Done."
