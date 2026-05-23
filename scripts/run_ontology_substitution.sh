#!/bin/bash
set -e

echo "Starting Ontology Substitution Replication Matrix..."
cd core

# Replication Baseline A
TOPOLOGY_A="osc_50_1.0"
COGNITION_A="rolling_50"

# Replication Baseline B
TOPOLOGY_B="osc_50_1.0"
COGNITION_B="event_reset"

UNIVERSES=(
    "2026_recent_crossfeed_1h_ontology"
    "2026_recent_discontinuity_1h_ontology"
)

for UNIVERSE in "${UNIVERSES[@]}"; do
    echo "================================================="
    echo "Processing Ontology Universe: $UNIVERSE"
    echo "================================================="
    
    TICK_FILE="chronology/historical/${UNIVERSE}_binance_tick/btc*.jsonl"
    KLINE_FILE="chronology/historical/${UNIVERSE}_yahoo_1m/btc*.jsonl"
    
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

echo "Ontology Substitution Matrix complete."
