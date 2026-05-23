#!/bin/bash
set -e
cd core
echo "Fetching 2024_asia_open_impulse_2h (Tick)..."
cargo run --release --bin historical_importer -- --symbol BTCUSDT --interval tick --start-time 1709596800000 --end-time 1709604000000 --name 2024_asia_open_impulse_2h_tick
echo "Fetching 2024_asia_open_impulse_2h (1m)..."
cargo run --release --bin historical_importer -- --symbol BTCUSDT --interval 1m --start-time 1709596800000 --end-time 1709604000000 --name 2024_asia_open_impulse_2h_1m
echo "Fetching 2026_recent_crossfeed_1h (Binance Tick)..."
cargo run --release --bin historical_importer -- --symbol BTCUSDT --interval tick --start-time 1779285600000 --end-time 1779289200000 --name 2026_recent_crossfeed_1h_binance_tick
echo "Fetching 2026_recent_crossfeed_1h (Yahoo 1m)..."
cargo run --release --bin yahoo_importer -- --symbol BTC-USD --interval 1m --name 2026_recent_crossfeed_1h_yahoo_1m
echo "Fetching 2026_recent_discontinuity_1h (Binance Tick)..."
cargo run --release --bin historical_importer -- --symbol BTCUSDT --interval tick --start-time 1779332400000 --end-time 1779336000000 --name 2026_recent_discontinuity_1h_binance_tick
echo "Fetching 2026_recent_discontinuity_1h (Yahoo 1m)..."
cargo run --release --bin yahoo_importer -- --symbol BTC-USD --interval 1m --name 2026_recent_discontinuity_1h_yahoo_1m
