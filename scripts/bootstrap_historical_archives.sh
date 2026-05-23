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

# 2026 Intraday Impulse Shock (May 23 2026, 07:30 UTC to 08:00 UTC)
# Justification: archive_justifications/2026_intraday_impulse_shock.md
echo "Fetching 2026_intraday_impulse_shock (Tick)..."
cargo run --release --bin historical_importer -- --symbol BTCUSDT --interval tick --start-time 1779521400000 --end-time 1779523200000 --name 2026_intraday_impulse_shock_0730_0800_utc_tick
echo "Fetching 2026_intraday_impulse_shock (1m)..."
cargo run --release --bin historical_importer -- --symbol BTCUSDT --interval 1m --start-time 1779521400000 --end-time 1779523200000 --name 2026_intraday_impulse_shock_0730_0800_utc_1m

# 2026 Multi-Stage Cascade Transition (May 22 2026, 20:00 UTC to May 23 2026, 04:00 UTC)
# Justification: archive_justifications/2026_multi_stage_cascade_transition.md
echo "Fetching 2026_multi_stage_cascade_transition (Tick)..."
cargo run --release --bin historical_importer -- --symbol BTCUSDT --interval tick --start-time 1779480000000 --end-time 1779508800000 --name 2026_multi_stage_cascade_transition_tick
echo "Fetching 2026_multi_stage_cascade_transition (1m)..."
cargo run --release --bin historical_importer -- --symbol BTCUSDT --interval 1m --start-time 1779480000000 --end-time 1779508800000 --name 2026_multi_stage_cascade_transition_1m

# 2026 Cross-Feed State Disagreement (May 23 2026, 04:20 UTC to 07:06 UTC)
# Justification: archive_justifications/2026_crossfeed_state_disagreement.md
echo "Fetching 2026_crossfeed_state_disagreement (Binance Tick)..."
cargo run --release --bin historical_importer -- --symbol BTCUSDT --interval tick --start-time 1779510000000 --end-time 1779520000000 --name 2026_crossfeed_state_disagreement_binance_tick
echo "Fetching 2026_crossfeed_state_disagreement (Yahoo 1m)..."
cargo run --release --bin yahoo_importer -- --symbol BTC-USD --interval 1m --name 2026_crossfeed_state_disagreement_yahoo_1m

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
