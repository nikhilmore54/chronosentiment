#!/usr/bin/env bash
# One-command live UI bootstrap (deterministic prep; .cursor/rules/chronosentiment-core.mdc).
# Research multi-engine / telemetry daemons were removed — use the minimal live path.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$ROOT"

PORT="${PORT:-8501}"
STRICT_PORT="${STRICT_PORT:-0}"

echo "ChronoSentiment Live"

if [ -d ".venv" ]; then
  # shellcheck source=/dev/null
  source .venv/bin/activate
fi

command -v streamlit >/dev/null 2>&1 || {
  echo "streamlit not found in PATH" >&2
  exit 1
}

PY="python3"
if [ -x ".venv/bin/python" ]; then
  PY=".venv/bin/python"
elif [ -x ".venv/bin/python3" ]; then
  PY=".venv/bin/python3"
fi

mkdir -p logs analysis/real_live

if command -v lsof >/dev/null 2>&1; then
  if lsof -i ":${PORT}" >/dev/null 2>&1; then
    if [[ "${STRICT_PORT}" == "1" ]]; then
      echo "Port ${PORT} already in use (set STRICT_PORT=0 to warn only, or free the port)." >&2
      exit 1
    else
      echo "Port ${PORT} already in use — Streamlit may fail to bind." >&2
    fi
  fi
fi

if [[ ! -x target/release/cs-ingest ]]; then
  echo "Building cs-ingest (release)..."
  cargo build -p cs-ingest --release
fi

echo "Live path reminder:"
echo "  1) python3 scripts/freeze_cohort_candles.py --batch-id N"
echo "  2) ./target/release/cs-ingest cohort --batch-id N --run-label …"
echo "  3) ./target/release/cs-ingest governor --archive-dir … [--once]"
echo "  4) python3 scripts/run_live_session.py --batch-id N …"
echo "  5) ./target/release/cs-ingest verify --batch-id N --run-label …"
echo ""

if command -v open >/dev/null 2>&1; then
  ( sleep 3 && open "http://127.0.0.1:${PORT}" ) &
fi

echo "Starting Streamlit UI (port ${PORT})..."
streamlit run app.py --server.port "${PORT}"
