#!/usr/bin/env bash
# start_backend.sh — Start the Coralys Decision Server, run baseline enrichment,
# and run the LIVE-001→LIVE-005 ticker fetch pipeline so fresh OHLCV values
# are reflected on the frontend.
#
# Usage:
#   ./scripts/start_backend.sh
#
# What it does:
#   1. Starts coralys_decision_server on :3001 (background)
#   2. Waits until the server is accepting connections (up to 60s)
#   3. Runs LIVE-001 → LIVE-005 pipeline to fetch fresh OHLCV and emit live decisions
#   4. Runs csp006_p_enrich to populate the historical CDI baseline (202 decisions)
#
# The ticker fetch (step 3) uses live Yahoo data — LIVE dataset (runs first).
# The enrichment (step 4) uses the CS-P-006 historical snapshot — BASELINE dataset.
#
# Environment:
#   PORT (optional) — override server port, default 3001
#   SKIP_LIVE_PIPELINE (optional) — set to 1 to skip step 4 (baseline only)

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PORT="${PORT:-3001}"

LEDGER_PATH="product_validation/CS-P-006/observatory/prospective/ledger.json"
OUTCOMES_PATH="product_validation/CS-P-006/observatory/prospective/outcomes.json"
REC001H_DIR_PATH="product_validation/CS-P-006/observatory/prospective"
YAHOO_CACHE="product_validation/CS-P-006/snapshot/20260814T183851Z_100instrument/yahoo_cache"
ENRICH_NOW="2026-08-17T03:45:00Z"

# LIVE pipeline paths
LIVE_SNAPSHOT_DIR="live_capture/snapshots"
LIVE_EVAL_DIR="live_capture/evaluations"
LIVE_RECOMMEND_DIR="live_capture/recommendations"
LIVE_CERTIFY_DIR="live_capture/certifications"
LIVE_LEDGER_DIR="live_capture/ledger"
LIVE_YAHOO_CACHE="live_capture/yahoo_cache"
UNIVERSE="datasets/universes/coralys_102_v1.json"

cd "$REPO_ROOT"

# ── 1. Start the server ───────────────────────────────────────────────────────

echo "[start_backend] Building and starting coralys_decision_server on :${PORT}..."

REC001H_DIR="$REC001H_DIR_PATH" \
HDV001_OUTCOMES_PATH="$OUTCOMES_PATH" \
  cargo run -p coralys_decision_server &

SERVER_PID=$!
echo "[start_backend] Server PID: $SERVER_PID"

# ── 2. Wait for server to bind ────────────────────────────────────────────────

echo "[start_backend] Waiting for server to bind on :${PORT}..."
MAX_WAIT=60
WAITED=0

until curl -sf "http://localhost:${PORT}/decisions" > /dev/null 2>&1; do
  if [ $WAITED -ge $MAX_WAIT ]; then
    echo "[start_backend] ERROR: Server did not start within ${MAX_WAIT}s. Aborting."
    kill "$SERVER_PID" 2>/dev/null || true
    exit 1
  fi
  sleep 1
  WAITED=$((WAITED + 1))
  echo "[start_backend]   ... waiting (${WAITED}s)"
done

echo "[start_backend] Server is up after ${WAITED}s."

# ── 3. LIVE-001 → LIVE-005 ticker fetch pipeline ─────────────────────────────

if [ "${SKIP_LIVE_PIPELINE:-0}" = "1" ]; then
  echo "[start_backend] SKIP_LIVE_PIPELINE=1 — skipping ticker fetch."
else
  echo "[start_backend] Running LIVE-001 → LIVE-005 ticker fetch pipeline..."

  mkdir -p "$LIVE_SNAPSHOT_DIR" "$LIVE_EVAL_DIR" "$LIVE_RECOMMEND_DIR" \
           "$LIVE_CERTIFY_DIR" "$LIVE_LEDGER_DIR" "$LIVE_YAHOO_CACHE"

  # LIVE-001: fetch fresh OHLCV snapshot
  echo "[start_backend] LIVE-001: fetching fresh OHLCV snapshot..."
  CHRONO_YAHOO_CACHE_DIR="$LIVE_YAHOO_CACHE" \
    cargo run -p chronosentiment_adapter --bin live001_snapshot -- \
      --universe "$UNIVERSE" \
      --output "$LIVE_SNAPSHOT_DIR"
  echo "[start_backend] LIVE-001 complete."

  # LIVE-002: evaluate snapshot against frozen C3-002 policy
  echo "[start_backend] LIVE-002: evaluating snapshot..."
  cargo run -p chronosentiment_adapter --bin live002_evaluate -- \
    --snapshot "$LIVE_SNAPSHOT_DIR/latest.json" \
    --output "$LIVE_EVAL_DIR"
  echo "[start_backend] LIVE-002 complete."

  # LIVE-003: generate recommendations
  echo "[start_backend] LIVE-003: generating recommendations..."
  cargo run -p chronosentiment_adapter --bin live003_recommend -- \
    --state "$LIVE_EVAL_DIR/latest.json" \
    --output "$LIVE_RECOMMEND_DIR"
  echo "[start_backend] LIVE-003 complete."

  # LIVE-004: certify recommendations
  echo "[start_backend] LIVE-004: certifying recommendations..."
  cargo run -p chronosentiment_adapter --bin live004_certify -- \
    --snapshot "$LIVE_SNAPSHOT_DIR/latest.json" \
    --recommendations "$LIVE_RECOMMEND_DIR/latest.json" \
    --output "$LIVE_CERTIFY_DIR"
  echo "[start_backend] LIVE-004 complete."

  # LIVE-005: admit certified decisions to ledger
  echo "[start_backend] LIVE-005: admitting to ledger..."
  cargo run -p chronosentiment_adapter --bin live005_ledger -- \
    --certification "$LIVE_CERTIFY_DIR/latest.json" \
    --ledger-dir "$LIVE_LEDGER_DIR"
  echo "[start_backend] LIVE-005 complete."

  echo "[start_backend] Ticker fetch pipeline complete. Fresh OHLCV values now reflected."
fi

# ── 4. Historical CDI baseline enrichment ────────────────────────────────────

TOTAL=$(curl -sf "http://localhost:${PORT}/decisions" | python3 -c "import sys,json; print(json.load(sys.stdin)['total'])" 2>/dev/null || echo "0")

if [ "$TOTAL" -gt 0 ]; then
  echo "[start_backend] Ledger already has ${TOTAL} decisions — skipping enrichment."
else
  echo "[start_backend] Ledger is empty. Running csp006_p_enrich (baseline: CS-P-006)..."
  CHRONO_YAHOO_CACHE_DIR="$YAHOO_CACHE" \
    cargo run -p chronosentiment_adapter --bin csp006_p_enrich -- \
    --ledger "$LEDGER_PATH" \
    --emit-url "http://localhost:${PORT}" \
    --now "$ENRICH_NOW"

  TOTAL=$(curl -sf "http://localhost:${PORT}/decisions" | python3 -c "import sys,json; print(json.load(sys.stdin)['total'])" 2>/dev/null || echo "unknown")
  echo "[start_backend] Enrichment complete. Ledger now has ${TOTAL} decisions."
fi

echo "[start_backend] Backend ready. Server PID: $SERVER_PID"
echo "[start_backend] To stop: kill $SERVER_PID"

# Keep script alive so the server stays in foreground
wait "$SERVER_PID"