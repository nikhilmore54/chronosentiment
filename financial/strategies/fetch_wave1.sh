
# ---------------------------------------------------------
# PHASE 2C: WAVE 1 ACQUISITION
# ---------------------------------------------------------

# 2022 FTX Collapse 4h (Nov 8 14:00 to Nov 8 18:00)
# Justification: archive_justifications/2022_ftx_collapse_4h.md
echo "Fetching 2022_ftx_collapse_4h (Tick)..."
cargo run --release --bin historical_importer -- --symbol BTCUSDT --interval tick --start-time 1667916000000 --end-time 1667930400000 --name 2022_ftx_collapse_4h_tick
echo "Fetching 2022_ftx_collapse_4h (1m)..."
cargo run --release --bin historical_importer -- --symbol BTCUSDT --interval 1m --start-time 1667916000000 --end-time 1667930400000 --name 2022_ftx_collapse_4h_1m

# 2024 CPI Shock 1h (Feb 13 13:00 to 14:00)
# Justification: archive_justifications/2024_cpi_shock_1h.md
echo "Fetching 2024_cpi_shock_1h (Tick)..."
cargo run --release --bin historical_importer -- --symbol BTCUSDT --interval tick --start-time 1707829200000 --end-time 1707832800000 --name 2024_cpi_shock_1h_tick
echo "Fetching 2024_cpi_shock_1h (1m)..."
cargo run --release --bin historical_importer -- --symbol BTCUSDT --interval 1m --start-time 1707829200000 --end-time 1707832800000 --name 2024_cpi_shock_1h_1m

# 2024 Quiet Sunday 4h (Feb 18 00:00 to 04:00)
# Justification: archive_justifications/2024_quiet_sunday_4h.md
echo "Fetching 2024_quiet_sunday_4h (Tick)..."
cargo run --release --bin historical_importer -- --symbol BTCUSDT --interval tick --start-time 1708214400000 --end-time 1708228800000 --name 2024_quiet_sunday_4h_tick
echo "Fetching 2024_quiet_sunday_4h (1m)..."
cargo run --release --bin historical_importer -- --symbol BTCUSDT --interval 1m --start-time 1708214400000 --end-time 1708228800000 --name 2024_quiet_sunday_4h_1m

echo "Done."

# --- PHASE 2C: CURATED CHRONOLOGY PRESSURE ACQUISITION ---
# The observatory must stay in Phase 2C to build deep recurrence.
