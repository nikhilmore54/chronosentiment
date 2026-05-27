#!/bin/bash
set -e
cd core
echo "Fetching 2021_may_19_cascade_4h (Tick)..."
cargo run --release --bin historical_importer -- --symbol BTCUSDT --interval tick --start-time 1621425600000 --end-time 1621440000000 --name 2021_may_19_cascade_4h_tick
echo "Fetching 2021_may_19_cascade_4h (1m)..."
cargo run --release --bin historical_importer -- --symbol BTCUSDT --interval 1m --start-time 1621425600000 --end-time 1621440000000 --name 2021_may_19_cascade_4h_1m
echo "Fetching 2021_oct_17_quiet_sunday_4h (Tick)..."
cargo run --release --bin historical_importer -- --symbol BTCUSDT --interval tick --start-time 1634428800000 --end-time 1634443200000 --name 2021_oct_17_quiet_sunday_4h_tick
echo "Fetching 2021_oct_17_quiet_sunday_4h (1m)..."
cargo run --release --bin historical_importer -- --symbol BTCUSDT --interval 1m --start-time 1634428800000 --end-time 1634443200000 --name 2021_oct_17_quiet_sunday_4h_1m
echo "Fetching 2022_sep_13_cpi_shock_2h (Tick)..."
cargo run --release --bin historical_importer -- --symbol BTCUSDT --interval tick --start-time 1663070400000 --end-time 1663077600000 --name 2022_sep_13_cpi_shock_2h_tick
echo "Fetching 2022_sep_13_cpi_shock_2h (1m)..."
cargo run --release --bin historical_importer -- --symbol BTCUSDT --interval 1m --start-time 1663070400000 --end-time 1663077600000 --name 2022_sep_13_cpi_shock_2h_1m
