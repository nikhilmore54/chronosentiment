#!/usr/bin/env bash
# start_backend.sh — Start the Coralys Decision Server and run baseline enrichment.
#
# Usage:
#   ./scripts/start_backend.sh
#
# What it does:
#   1. Starts coralys_decision_server on :3001 (background)
#   2. Waits until the server is accepting connections (up to 60s)
#   3. Runs csp006_p_enrich to populate the historical CDI baseline (202 decisions)
#
# The enrichment uses the CS-P-006 historical snapshot — this is the BASELINE dataset.
# It is NOT the LIVE-001 path. Live decisions will be a separate pipeline.
#
# Environment:
#   PORT (optional) — override server port, default 3001

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PORT="${PORT:-3001}"

LEDGER_PATH="product_validation/CS-P-006/observatory/prospective/ledger.json"
OUTCOMES_PATH="product_validation/CS-P-006/observatory/prospective/outcomes.json"
REC001H_DIR_PATH="product_validation/CS-P-006/observatory/prospective"
YAHOO_CACHE="product_validation/CS-P-006/snapshot/20260814T183851Z_100instrument/yahoo_cache"
ENRICH_NOW="2026-08-17T03:45:00Z"

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

# ── 3. Check if ledger already has decisions ──────────────────────────────────

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