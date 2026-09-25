#!/usr/bin/env bash
# pre_open_job.sh - Deterministic pre-open LIVE-005 pipeline

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

if [[ $# -ne 2 || "$1" != "--as-of-date" ]]; then
    echo "Usage: $0 --as-of-date YYYY-MM-DD"
    exit 1
fi
SESSION_DATE="$2"

LIVE_SNAPSHOT_DIR="live_capture/snapshots"
LIVE_EVAL_DIR="live_capture/evaluations"
LIVE_RECOMMEND_DIR="live_capture/recommendations"
LIVE_CERTIFY_DIR="live_capture/certifications"
LIVE_LEDGER_DIR="live_capture/ledger"
LIVE_YAHOO_CACHE="live_capture/yahoo_cache"
UNIVERSE="datasets/universes/coralys_102_v1.json"
PORT="3001"

echo "=== PRE-OPEN JOB for ${SESSION_DATE} ==="

mkdir -p "$LIVE_SNAPSHOT_DIR" "$LIVE_EVAL_DIR" "$LIVE_RECOMMEND_DIR" \
         "$LIVE_CERTIFY_DIR" "$LIVE_LEDGER_DIR" "$LIVE_YAHOO_CACHE"

echo "[pre-open] Checking if today's certified book already exists..."
if python3 -c '
import json, sys, os
dataset_path = "datasets/today_live_dataset.json"
cert_path = "'"$LIVE_CERTIFY_DIR/latest.json"'"
if not os.path.exists(dataset_path) or not os.path.exists(cert_path):
    sys.exit(1)
with open(dataset_path) as f:
    try:
        d = json.load(f)
    except:
        sys.exit(1)
if not d:
    sys.exit(1)
for r in d:
    if r.get("cohort_date") != "'"$SESSION_DATE"'":
        sys.exit(1)
with open(cert_path) as f:
    try:
        c = json.load(f)
    except:
        sys.exit(1)
if c.get("certification_status") == "FAILED":
    sys.exit(1)
sys.exit(0)
'; then
    echo "[pre-open] ALREADY_READY: Verified certified LIVE-005 book for ${SESSION_DATE} already exists."
    exit 0
fi

echo "[pre-open] No valid certified book found for ${SESSION_DATE}. Proceeding with LIVE-001 -> LIVE-005 pipeline."

echo "[pre-open] LIVE-001: fetching fresh OHLCV snapshot..."
CHRONO_YAHOO_CACHE_DIR="$LIVE_YAHOO_CACHE" \
cargo run -p chronosentiment_adapter --bin live001_snapshot -- \
    --universe "$UNIVERSE" \
    --output "$LIVE_SNAPSHOT_DIR"

echo "[pre-open] LIVE-002: evaluating snapshot..."
cargo run -p chronosentiment_adapter --bin live002_evaluate -- \
    --snapshot "$LIVE_SNAPSHOT_DIR/latest.json" \
    --output "$LIVE_EVAL_DIR"

echo "[pre-open] LIVE-003: generating recommendations..."
cargo run -p chronosentiment_adapter --bin live003_recommend -- \
    --state "$LIVE_EVAL_DIR/latest.json" \
    --output "$LIVE_RECOMMEND_DIR"

echo "[pre-open] LIVE-004: certifying recommendations..."
cargo run -p chronosentiment_adapter --bin live004_certify -- \
    --snapshot "$LIVE_SNAPSHOT_DIR/latest.json" \
    --state    "$LIVE_EVAL_DIR/latest.json" \
    --recommend "$LIVE_RECOMMEND_DIR/latest.json" \
    --output   "$LIVE_CERTIFY_DIR"

echo "[pre-open] Verifying LIVE-004 certification..."
if ! python3 -c '
import json, sys
with open("'"$LIVE_CERTIFY_DIR/latest.json"'") as f:
    d = json.load(f)
if d.get("certification_status") == "FAILED":
    print("Certification FAILED")
    sys.exit(1)
'; then
    echo "ERROR: LIVE-004 certification failed. Failing closed."
    exit 1
fi

echo "[pre-open] LIVE-005: admitting to ledger..."
cargo run -p chronosentiment_adapter --bin live005_ledger -- \
    --certification "$LIVE_CERTIFY_DIR/latest.json" \
    --recommend     "$LIVE_RECOMMEND_DIR/latest.json" \
    --ledger        "$LIVE_LEDGER_DIR" \
    --audit         "$LIVE_LEDGER_DIR/audit" \
    --emit-url      "http://localhost:${PORT}" || echo "Warning: emission to backend failed, continuing..."

echo "[pre-open] Running bridge_live005.py for date ${SESSION_DATE}..."
if ! python3 scripts/bridge_live005.py --as-of-date "$SESSION_DATE"; then
    echo "ERROR: Bridge failed or missing cohort. Failing closed."
    exit 1
fi

echo "[pre-open] Verifying today_live_dataset.json..."
if ! python3 -c '
import json, sys
with open("datasets/today_live_dataset.json") as f:
    d = json.load(f)
if not d:
    print("Dataset empty")
    sys.exit(1)
for r in d:
    if r.get("cohort_date") != "'"$SESSION_DATE"'":
        print("Cohort date mismatch:", r.get("cohort_date"))
        sys.exit(1)
'; then
    echo "ERROR: Verification of today_live_dataset.json failed."
    exit 1
fi

echo "[pre-open] PRE-OPEN JOB COMPLETED SUCCESSFULLY for ${SESSION_DATE}."
