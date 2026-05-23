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

# 2023 Aug 17 Liquidation Cascade (Aug 17 2023, 21:00 UTC to 22:00 UTC)
echo "Fetching 2023_liquidation_cascade (Tick)..."
cargo run --release --bin historical_importer -- --symbol BTCUSDT --interval tick --start-time 1692306000000 --end-time 1692309600000 --name 2023_liquidation_cascade_1h_tick
echo "Fetching 2023_liquidation_cascade (1m)..."
cargo run --release --bin historical_importer -- --symbol BTCUSDT --interval 1m --start-time 1692306000000 --end-time 1692309600000 --name 2023_liquidation_cascade_1h_1m

# 2023 Binance Outage / Discontinuity (March 24 2023, 11:30 UTC to 12:30 UTC)
echo "Fetching 2023_binance_outage (Tick)..."
cargo run --release --bin historical_importer -- --symbol BTCUSDT --interval tick --start-time 1679657400000 --end-time 1679661000000 --name 2023_binance_outage_1h_tick
echo "Fetching 2023_binance_outage (1m)..."
cargo run --release --bin historical_importer -- --symbol BTCUSDT --interval 1m --start-time 1679657400000 --end-time 1679661000000 --name 2023_binance_outage_1h_1m

# 2023 Christmas Low-Volatility Drift (Dec 24 2023, 12:00 UTC to 13:00 UTC)
echo "Fetching 2023_christmas_drift (Tick)..."
cargo run --release --bin historical_importer -- --symbol BTCUSDT --interval tick --start-time 1703419200000 --end-time 1703422800000 --name 2023_christmas_drift_1h_tick
echo "Fetching 2023_christmas_drift (1m)..."
cargo run --release --bin historical_importer -- --symbol BTCUSDT --interval 1m --start-time 1703419200000 --end-time 1703422800000 --name 2023_christmas_drift_1h_1m

echo "Done."

# --- FUTURE CURATED TARGETS (PHASE 2B SCENARIO EXPANSION) ---
# Do NOT bulk-scrape. Adhere strictly to the "Wind Tunnel" philosophy:
# Small, curated, phenomenologically distinct universes.
#
# Target 1: Multi-Hour Liquidation Cascade (e.g., 24h FTX Collapse window)
# Reason: Tests persistence carryover and reset fragmentation propagation.
#
# Target 2: Intermittent Discontinuity / Partial Outage
# Reason: Tests degradation under bursty continuity corruption, not total absence.
#
# Target 3: Regime Transition Window
# Reason: Tests chronology sensitivity onset from drift -> expansion -> compression.
#
# Target 4: Fidelity Ladder Curvature (5m, 15m, OHLCV)
# Reason: Tests degradation curvature, not just binary tick vs 1m.
