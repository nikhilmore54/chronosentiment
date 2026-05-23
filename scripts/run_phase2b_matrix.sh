#!/bin/bash
set -e

echo "Starting Phase 2B Cross-Universe Replication Matrix..."
cd core

# Define frozen machinery
TOPOLOGY="osc_50_1.0"
COGNITION="rolling_50"

# Define target universes (assumes data is already bootstrapped via bootstrap_historical_archives.sh)
UNIVERSES=(
    "2024_etf_approval_1h"
    "2023_liquidation_cascade_1h"
    "2023_binance_outage_1h"
    "2023_christmas_drift_1h"
)

# NOTE: For a real matrix run, you need the tick and 1m JSONL files fetched for all target universes.
# This script orchestrates the replay mechanism purely for tracing and bounded comparison.

for UNIVERSE in "${UNIVERSES[@]}"; do
    echo "================================================="
    echo "Processing Universe: $UNIVERSE"
    echo "================================================="
    
    # Paths
    TICK_FILE="chronology/historical/${UNIVERSE}_tick/btcusdt_*.jsonl"
    KLINE_FILE="chronology/historical/${UNIVERSE}_1m/btcusdt_*.jsonl"
    
    # We expand the glob if it exists
    TICK_PATH=$(ls $TICK_FILE 2>/dev/null | head -n 1) || true
    KLINE_PATH=$(ls $KLINE_FILE 2>/dev/null | head -n 1) || true
    
    if [[ -z "$TICK_PATH" || -z "$KLINE_PATH" ]]; then
        echo "Missing data for $UNIVERSE. Skipping..."
        continue
    fi

    # Output Structure: artifacts/phase2b/<universe>/<fidelity>/<topology>/<cognition>/
    TIER0_OUT="artifacts/phase2b/${UNIVERSE}/tier0_tick"
    TIER1_OUT="artifacts/phase2b/${UNIVERSE}/tier1_1m"

    echo "Replaying Tier 0 (Native Tick)..."
    cargo run --release --bin trace_replay -- --substrate tier0_tick --substrate-file "$TICK_PATH" --topology "$TOPOLOGY" --cognition "$COGNITION"

    echo "Replaying Tier 1 (1m Kline)..."
    cargo run --release --bin trace_replay -- --substrate tier1_1m --substrate-file "$KLINE_PATH" --topology "$TOPOLOGY" --cognition "$COGNITION"
    
    # We move the outputs to the formal phase2b structure
    # Note: trace_replay automatically puts it in artifacts/<substrate>/<topology>/<cognition>
    mkdir -p "$TIER0_OUT" "$TIER1_OUT"
    cp -r "artifacts/tier0_tick/$TOPOLOGY/$COGNITION" "$TIER0_OUT/"
    cp -r "artifacts/tier1_1m/$TOPOLOGY/$COGNITION" "$TIER1_OUT/"

    echo "Running Bounded Comparison..."
    python3 ../scripts/phase2a_degradation_study.py \
        --tier0 "$TIER0_OUT/$COGNITION/trace_v1.json" \
        --tier1 "$TIER1_OUT/$COGNITION/trace_v1.json"
        
    echo ""
done

echo "Matrix replication complete. Update claims/replication_log.md with results."
