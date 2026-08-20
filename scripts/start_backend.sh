#!/usr/bin/env bash
# start_backend.sh — Manage the Coralys Decision Server lifecycle.
#
# Usage:
#   ./scripts/start_backend.sh [start|restart|stop]
#
#   start   (default) — start server if not already running, then run pipelines
#   restart           — kill any running server, then start fresh
#   stop              — kill any running server and exit
#
# What it does on start/restart:
#   1. Starts coralys_decision_server on :PORT (background)
#   2. Waits until the server is accepting connections (up to 60s)
#   3. Runs LIVE-001 → LIVE-005 pipeline (fresh OHLCV → live decisions)
#   4. Runs csp006_p_enrich (historical CDI baseline, skipped if ledger populated)
#   5. wait $SERVER_PID (keeps script alive)
#
# Environment:
#   PORT               (optional) — server port, default 3001
#   SKIP_LIVE_PIPELINE (optional) — set to 1 to skip step 3

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PORT="${PORT:-3001}"
CMD="${1:-start}"

# ── Paths ─────────────────────────────────────────────────────────────────────

LEDGER_PATH="product_validation/CS-P-006/observatory/prospective/ledger.json"
OUTCOMES_PATH="datasets/hdv001/hdv001_outcomes_v1.json"
REC001H_DIR_PATH="datasets/recommendation/historical"
YAHOO_CACHE="product_validation/CS-P-006/snapshot/20260814T183851Z_100instrument/yahoo_cache"
ENRICH_NOW="2026-08-17T03:45:00Z"

LIVE_SNAPSHOT_DIR="live_capture/snapshots"
LIVE_EVAL_DIR="live_capture/evaluations"
LIVE_RECOMMEND_DIR="live_capture/recommendations"
LIVE_CERTIFY_DIR="live_capture/certifications"
LIVE_LEDGER_DIR="live_capture/ledger"
LIVE_YAHOO_CACHE="live_capture/yahoo_cache"
UNIVERSE="datasets/universes/coralys_102_v1.json"

cd "$REPO_ROOT"

# ── Helper: kill any running server ──────────────────────────────────────────

stop_server() {
  echo "[backend] Stopping any running coralys_decision_server..."
  pkill -f coralys_decision_server 2>/dev/null || true
  # Also kill any cargo run process holding the server
  pkill -f "cargo run -p coralys_decision_server" 2>/dev/null || true
  sleep 1
  # Verify port is free
  if curl -sf "http://localhost:${PORT}/decisions" > /dev/null 2>&1; then
    echo "[backend] WARNING: port ${PORT} still in use after stop attempt."
  else
    echo "[backend] Server stopped."
  fi
}

# ── Handle stop command ───────────────────────────────────────────────────────

if [ "$CMD" = "stop" ]; then
  stop_server
  exit 0
fi

# ── Handle restart: kill existing server first ────────────────────────────────

if [ "$CMD" = "restart" ]; then
  stop_server
fi

# ── Verify not already running (for start) ────────────────────────────────────

if [ "$CMD" = "start" ]; then
  if curl -sf "http://localhost:${PORT}/decisions" > /dev/null 2>&1; then
    echo "[backend] Server already running on :${PORT}. Use 'restart' to restart."
    exit 0
  fi
fi

# ── 1. Start the server ───────────────────────────────────────────────────────

echo "[backend] Building and starting coralys_decision_server on :${PORT}..."

REC001H_DIR="$REC001H_DIR_PATH" \
HDV001_OUTCOMES_PATH="$OUTCOMES_PATH" \
  cargo run -p coralys_decision_server &

SERVER_PID=$!
echo "[backend] Server PID: $SERVER_PID"

# ── 2. Wait for server to bind ────────────────────────────────────────────────

echo "[backend] Waiting for server to bind on :${PORT}..."
MAX_WAIT=60
WAITED=0

until curl -sf "http://localhost:${PORT}/decisions" > /dev/null 2>&1; do
  if [ $WAITED -ge $MAX_WAIT ]; then
    echo "[backend] ERROR: Server did not start within ${MAX_WAIT}s. Aborting."
    kill "$SERVER_PID" 2>/dev/null || true
    exit 1
  fi
  sleep 1
  WAITED=$((WAITED + 1))
  echo "[backend]   ... waiting (${WAITED}s)"
done

echo "[backend] Server is up after ${WAITED}s."

# ── 3. LIVE-001 → LIVE-005 ticker fetch pipeline ─────────────────────────────

if [ "${SKIP_LIVE_PIPELINE:-0}" = "1" ]; then
  echo "[backend] SKIP_LIVE_PIPELINE=1 — skipping ticker fetch."
else
  echo "[backend] Running LIVE-001 → LIVE-005 ticker fetch pipeline..."

  mkdir -p "$LIVE_SNAPSHOT_DIR" "$LIVE_EVAL_DIR" "$LIVE_RECOMMEND_DIR" \
           "$LIVE_CERTIFY_DIR" "$LIVE_LEDGER_DIR" "$LIVE_YAHOO_CACHE"

  echo "[backend] LIVE-001: fetching fresh OHLCV snapshot..."
  CHRONO_YAHOO_CACHE_DIR="$LIVE_YAHOO_CACHE" \
    cargo run -p chronosentiment_adapter --bin live001_snapshot -- \
      --universe "$UNIVERSE" \
      --output "$LIVE_SNAPSHOT_DIR"
  echo "[backend] LIVE-001 complete."

  echo "[backend] LIVE-002: evaluating snapshot..."
  cargo run -p chronosentiment_adapter --bin live002_evaluate -- \
    --snapshot "$LIVE_SNAPSHOT_DIR/latest.json" \
    --output "$LIVE_EVAL_DIR"
  echo "[backend] LIVE-002 complete."

  echo "[backend] LIVE-003: generating recommendations..."
  cargo run -p chronosentiment_adapter --bin live003_recommend -- \
    --state "$LIVE_EVAL_DIR/latest.json" \
    --output "$LIVE_RECOMMEND_DIR"
  echo "[backend] LIVE-003 complete."

  echo "[backend] LIVE-004: certifying recommendations..."
  cargo run -p chronosentiment_adapter --bin live004_certify -- \
    --snapshot "$LIVE_SNAPSHOT_DIR/latest.json" \
    --state    "$LIVE_EVAL_DIR/latest.json" \
    --recommend "$LIVE_RECOMMEND_DIR/latest.json" \
    --output   "$LIVE_CERTIFY_DIR"
  echo "[backend] LIVE-004 complete."

  echo "[backend] LIVE-005: admitting to ledger and emitting to Decision Server..."
  cargo run -p chronosentiment_adapter --bin live005_ledger -- \
    --certification "$LIVE_CERTIFY_DIR/latest.json" \
    --recommend     "$LIVE_RECOMMEND_DIR/latest.json" \
    --ledger        "$LIVE_LEDGER_DIR" \
    --audit         "$LIVE_LEDGER_DIR/audit" \
    --emit-url      "http://localhost:${PORT}"
  echo "[backend] LIVE-005 complete."

  echo "[backend] Ticker fetch pipeline complete. Fresh OHLCV values now reflected."

  echo "[backend] LIVE-006 (TIME-009): running prospective observation tick..."
  cargo run -p chronosentiment_adapter --bin time009_observe -- \
    --ledger  "$LIVE_LEDGER_DIR" \
    --output  "time_machine/analysis/TIME009/observations" \
    --cache   "$LIVE_YAHOO_CACHE"
  echo "[backend] LIVE-006 (TIME-009) complete."
fi

# ── 4. Historical CDI baseline enrichment ────────────────────────────────────

TOTAL=$(curl -sf "http://localhost:${PORT}/decisions" | python3 -c "import sys,json; print(json.load(sys.stdin)['total'])" 2>/dev/null || echo "0")

if [ "$TOTAL" -gt 0 ]; then
  echo "[backend] Ledger already has ${TOTAL} decisions — skipping enrichment."
else
  echo "[backend] Ledger is empty. Running csp006_p_enrich (baseline: CS-P-006)..."
  CHRONO_YAHOO_CACHE_DIR="$YAHOO_CACHE" \
    cargo run -p chronosentiment_adapter --bin csp006_p_enrich -- \
    --ledger "$LEDGER_PATH" \
    --emit-url "http://localhost:${PORT}" \
    --now "$ENRICH_NOW"

  TOTAL=$(curl -sf "http://localhost:${PORT}/decisions" | python3 -c "import sys,json; print(json.load(sys.stdin)['total'])" 2>/dev/null || echo "unknown")
  echo "[backend] Enrichment complete. Ledger now has ${TOTAL} decisions."
fi

echo "[backend] Backend ready. Server PID: $SERVER_PID"
echo "[backend] To stop:    kill $SERVER_PID  OR  ./scripts/start_backend.sh stop"
echo "[backend] To restart: ./scripts/start_backend.sh restart"

# ── 5. Keep script alive ──────────────────────────────────────────────────────

wait "$SERVER_PID"