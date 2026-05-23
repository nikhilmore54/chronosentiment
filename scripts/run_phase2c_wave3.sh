#!/bin/bash
set -e

echo "Starting Phase 2B Cross-Universe Replication Matrix..."
cd core

# Replication Baseline A
TOPOLOGY_A="osc_50_1.0"
COGNITION_A="rolling_50"

# Replication Baseline B
TOPOLOGY_B="osc_50_1.0"
COGNITION_B="event_reset"

UNIVERSES=(
    "2021_may_19_cascade_4h"
    "2021_oct_17_quiet_sunday_4h"
    "2022_sep_13_cpi_shock_2h"
)

for UNIVERSE in "${UNIVERSES[@]}"; do
    echo "================================================="
    echo "Processing Universe: $UNIVERSE"
    echo "================================================="
    
    TICK_FILE="chronology/historical/${UNIVERSE}_tick/btc*.jsonl"
    KLINE_FILE="chronology/historical/${UNIVERSE}_1m/btc*.jsonl"
    
    TICK_PATH=$(ls $TICK_FILE 2>/dev/null | head -n 1) || true
    KLINE_PATH=$(ls $KLINE_FILE 2>/dev/null | head -n 1) || true
    
    if [[ -z "$TICK_PATH" || -z "$KLINE_PATH" ]]; then
        echo "Missing data for $UNIVERSE. Skipping..."
        continue
    fi

    # --- Baseline A Execution ---
    echo "--- Baseline A: $TOPOLOGY_A + $COGNITION_A ---"
    TIER0_OUT_A="artifacts/phase2c/${UNIVERSE}/tier0_tick/${COGNITION_A}"
    TIER1_OUT_A="artifacts/phase2c/${UNIVERSE}/tier1_1m/${COGNITION_A}"

    cargo run --release --bin trace_replay -- --substrate tier0_tick --substrate-file "$TICK_PATH" --topology "$TOPOLOGY_A" --cognition "$COGNITION_A"
    cargo run --release --bin trace_replay -- --substrate tier1_1m --substrate-file "$KLINE_PATH" --topology "$TOPOLOGY_A" --cognition "$COGNITION_A"
    
    mkdir -p "$TIER0_OUT_A" "$TIER1_OUT_A"
    cp -r "artifacts/tier0_tick/$TOPOLOGY_A/$COGNITION_A"/* "$TIER0_OUT_A/"
    cp -r "artifacts/tier1_1m/$TOPOLOGY_A/$COGNITION_A"/* "$TIER1_OUT_A/"

    # --- Baseline B Execution ---
    echo "--- Baseline B: $TOPOLOGY_B + $COGNITION_B ---"
    TIER0_OUT_B="artifacts/phase2c/${UNIVERSE}/tier0_tick/${COGNITION_B}"
    TIER1_OUT_B="artifacts/phase2c/${UNIVERSE}/tier1_1m/${COGNITION_B}"

    cargo run --release --bin trace_replay -- --substrate tier0_tick --substrate-file "$TICK_PATH" --topology "$TOPOLOGY_B" --cognition "$COGNITION_B"
    cargo run --release --bin trace_replay -- --substrate tier1_1m --substrate-file "$KLINE_PATH" --topology "$TOPOLOGY_B" --cognition "$COGNITION_B"
    
    mkdir -p "$TIER0_OUT_B" "$TIER1_OUT_B"
    cp -r "artifacts/tier0_tick/$TOPOLOGY_B/$COGNITION_B"/* "$TIER0_OUT_B/"
    cp -r "artifacts/tier1_1m/$TOPOLOGY_B/$COGNITION_B"/* "$TIER1_OUT_B/"
        
    echo ""
done

echo "Matrix replication complete."
