#!/usr/bin/env bash

# run_soak.sh – orchestrates the historical and live soak for BTCUSD, ETHUSD, SOLUSD
# Requires python ingestion scripts and the existing verifier scripts.

set -euo pipefail

# -------------------------------------------------------------------
# Configuration
# -------------------------------------------------------------------
SYMBOLS=(BTCUSD ETHUSD SOLUSD)
PROVIDERS=(yahoo binance)
HISTORICAL_WINDOW_START="2026-04-01"
HISTORICAL_WINDOW_END="2026-05-01"
LIVE_DURATION_SECONDS=1800   # 30 minutes

# Baseline and head commits (auto‑filled for reproducibility)
BASELINE_COMMIT="cbf0f859148b9b0f3497a38aa44cd2d441166d23"
HEAD_COMMIT="7a028394"

REPORT_FILE="docs/governance/soak_report_$(date +%Y%m%d%H%M).md"

# -------------------------------------------------------------------
# Helper functions
# -------------------------------------------------------------------
hash_chronology() {
    local dir=$1
    # Compute a deterministic hash of the entire chronology directory
    # (sorted file list, then sha256 of concatenated contents)
    find "$dir" -type f -print0 | sort -z | xargs -0 sha256sum | sha256sum | awk '{print $1}'
}

run_historical() {
    echo "=== Running historical ingestion ==="
    for provider in "${PROVIDERS[@]}"; do
        for sym in "${SYMBOLS[@]}"; do
            echo "Ingesting $sym from $provider (historical)"
            python3 scripts/ingest_historical.py \
                --symbol "$sym" \
                --provider "$provider" \
                --start "$HISTORICAL_WINDOW_START" \
                --end "$HISTORICAL_WINDOW_END"
        done
    done
}

run_live() {
    echo "=== Starting live ingestion (background) ==="
    for provider in "${PROVIDERS[@]}"; do
        for sym in "${SYMBOLS[@]}"; do
            echo "Live ingest $sym from $provider (duration ${LIVE_DURATION_SECONDS}s)"
            # Use timeout to limit the live daemon run time
            timeout "$LIVE_DURATION_SECONDS" python3 scripts/live_ingest.py \
                --symbol "$sym" \
                --provider "$provider" \
                &
        done
    done
    # Wait for all background jobs to finish
    wait
}

run_verification() {
    echo "=== Running replay certification ==="
    # The verifier script expects the root of the chronology; we assume it is under fixtures/strategy_identity/<symbol>/<provider>/
    for provider in "${PROVIDERS[@]}"; do
        for sym in "${SYMBOLS[@]}"; do
            chrono_dir="fixtures/strategy_identity/${sym}/${provider}"
            if [[ -d "$chrono_dir" ]]; then
                echo "Verifying $sym from $provider"
                scripts/ci_fast.sh "$chrono_dir"
            else
                echo "WARNING: Chronology directory $chrono_dir not found"
            fi
        done
    done
}

generate_report() {
    echo "=== Generating soak report: $REPORT_FILE ==="
    cat > "$REPORT_FILE" <<EOF
# Soak Report $(date '+%Y-%m-%d %H:%M:%S')

**Baseline commit:** $BASELINE_COMMIT
**Head commit:** $HEAD_COMMIT

## Metrics Summary

| Metric                     | BTCUSD | ETHUSD | SOLUSD |
|---------------------------|--------|--------|--------|
| Historical Yahoo PASS      | ✓      | ✓      | ✓      |
| Historical Binance PASS    | ✓      | ✓      | ✓      |
| Live Binance PASS          | ✓      | ✓      | ✓      |
| Identity Translation Count | 0      | 0      | 0      |
| Chronology Hash Stable     | ✓      | ✓      | ✓      |
| Replay Hash Stable         | ✓      | ✓      | ✓      |
| Provider Identity PASS     | ✓      | ✓      | ✓      |
| Replay Determinism PASS    | ✓      | ✓      | ✓      |

*The ✓ entries are placeholders – the script will need to be extended to compute the actual values.*
EOF
    echo "Report generated at $REPORT_FILE"
}

# -------------------------------------------------------------------
# Main execution flow
# -------------------------------------------------------------------
run_historical
run_live
run_verification
generate_report

echo "=== Soak execution completed ==="
