#!/bin/bash
set -e

echo "Running CI Determinism Test..."

# Move to core
cd core || exit 1

# Topology and Cognition to test
TOP="plateau_low"
COG="event_reset"
SUBSTRATE="BTCUSDT"
FILE="chronology/live_capture_0001.jsonl"

echo "Pass 1: Generating baseline artifact..."
cargo run --quiet --bin trace_replay -- --substrate "$SUBSTRATE" --substrate-file "$FILE" --topology "$TOP" --cognition "$COG" > /dev/null

# Extract hashes using python for reliability
HASH1=$(python3 -c "import json; print(json.load(open('artifacts/$SUBSTRATE/$TOP/$COG/metadata.json'))['artifact_hash'])")

echo "Pass 2: Regenerating artifact..."
cargo run --quiet --bin trace_replay -- --substrate "$SUBSTRATE" --substrate-file "$FILE" --topology "$TOP" --cognition "$COG" > /dev/null

HASH2=$(python3 -c "import json; print(json.load(open('artifacts/$SUBSTRATE/$TOP/$COG/metadata.json'))['artifact_hash'])")

echo "---"
echo "Artifact Hash 1: $HASH1"
echo "Artifact Hash 2: $HASH2"

if [ "$HASH1" != "$HASH2" ]; then
    echo "❌ FAIL: Determinism violation detected! Artifact hashes diverged."
    exit 1
else
    echo "✅ PASS: Replay equivalence and determinism verified."
fi
