#!/bin/bash
set -e

echo "Starting Phase 2E-C2 Matrix (Counter-Pressure Directional Rally)..."
cd core

# Replication Baseline A
TOPOLOGY_A="osc_50_1.0"
COGNITION_A="rolling_50"

# Replication Baseline B
TOPOLOGY_B="osc_50_1.0"
COGNITION_B="event_reset"

UNIVERSES=(
    "2026_tsla_upward_rally"
)

for UNIVERSE in "${UNIVERSES[@]}"; do
    echo "================================================="
    echo "Processing Universe: $UNIVERSE"
    echo "================================================="
    
    KLINE_FILE="chronology/historical/${UNIVERSE}_5m/*.jsonl"
    KLINE_PATH=$(ls $KLINE_FILE 2>/dev/null | head -n 1) || true
    
    if [[ -z "$KLINE_PATH" ]]; then
        echo "Missing data for $UNIVERSE. Skipping..."
        continue
    fi

    # --- Baseline A Execution ---
    echo "--- Baseline A: $TOPOLOGY_A + $COGNITION_A ---"
    TIER1_OUT_A="artifacts/phase2e_c2/${UNIVERSE}/tier1_5m/${COGNITION_A}"

    cargo run --release --bin trace_replay -- --substrate tier1_1m --substrate-file "$KLINE_PATH" --topology "$TOPOLOGY_A" --cognition "$COGNITION_A"
    
    mkdir -p "$TIER1_OUT_A"
    cp -r "artifacts/tier1_1m/$TOPOLOGY_A/$COGNITION_A"/* "$TIER1_OUT_A/"

    # --- Baseline B Execution ---
    echo "--- Baseline B: $TOPOLOGY_B + $COGNITION_B ---"
    TIER1_OUT_B="artifacts/phase2e_c2/${UNIVERSE}/tier1_5m/${COGNITION_B}"

    cargo run --release --bin trace_replay -- --substrate tier1_1m --substrate-file "$KLINE_PATH" --topology "$TOPOLOGY_B" --cognition "$COGNITION_B"
    
    mkdir -p "$TIER1_OUT_B"
    cp -r "artifacts/tier1_1m/$TOPOLOGY_B/$COGNITION_B"/* "$TIER1_OUT_B/"
        
    echo ""
done

echo "Phase 2E-C2 Matrix complete."
