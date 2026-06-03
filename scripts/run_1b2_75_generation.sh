#!/bin/bash
set -e

echo "Generating strictly paired executions (1m vs tick) for Phase 1B-2.75..."

EVENTS=(
    "2021_may_19_cascade_4h"
    "2021_oct_17_quiet_sunday_4h"
    "2022_ftx_collapse_4h"
    "2022_sep_13_cpi_shock_2h"
    "2023_binance_outage_1h"
    "2023_christmas_drift_1h"
    "2023_liquidation_cascade_1h"
    "2024_asia_open_impulse_2h"
    "2024_cpi_shock_1h"
    "2024_etf_approval_1h"
)

COGNITIONS=("rolling_50" "event_reset")
FREQUENCIES=("1m" "tick")

OUT_DIR="phase1b2_75_artifacts"
rm -rf "$OUT_DIR"
mkdir -p "$OUT_DIR"

for EVENT in "${EVENTS[@]}"; do
    for FREQ in "${FREQUENCIES[@]}"; do
        DATASET_NAME="${EVENT}_${FREQ}"
        FILE_PATH=$(ls infrastructure/core/chronology/historical/${DATASET_NAME}/*.jsonl 2>/dev/null | head -n 1) || true
        
        if [[ -z "$FILE_PATH" ]]; then
            echo "Missing data for $DATASET_NAME. Skipping..."
            continue
        fi
        
        for COG in "${COGNITIONS[@]}"; do
            echo "Running: $EVENT | $FREQ | $COG"
            cargo run --manifest-path financial/strategies/Cargo.toml --release --bin financial_replay -- --substrate "$FREQ" --substrate-file "$FILE_PATH" --topology "osc_50_1.0" --cognition "$COG" > /dev/null 2>&1
            
            mkdir -p "$OUT_DIR/$EVENT/$FREQ/$COG"
            cp "artifacts/$FREQ/osc_50_1.0/$COG/trace_v1.json" "$OUT_DIR/$EVENT/$FREQ/$COG/" 2>/dev/null || true
        done
    done
done

echo "Generation complete."
