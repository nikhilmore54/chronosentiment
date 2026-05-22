#!/usr/bin/env bash
# One-command live UI + multi-lane pipelines (deterministic prep; .cursor/rules/chronosentiment-core.mdc).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$ROOT"

PORT="${PORT:-8501}"
STRICT_PORT="${STRICT_PORT:-0}"
LOG_DIR="analysis/live_multi"

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

"${PY}" -c "import yfinance" 2>/dev/null || {
  echo "yfinance not importable in ${PY} — install project deps." >&2
  exit 1
}

mkdir -p "${LOG_DIR}" logs

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

PIPE_PID=""
DAEMON_PID=""
PG_KILL=false
cleanup() {
  echo "Shutting down pipelines..."
  if [[ -n "${DAEMON_PID:-}" ]] && kill -0 "${DAEMON_PID}" 2>/dev/null; then
    kill "${DAEMON_PID}" 2>/dev/null || true
    wait "${DAEMON_PID}" 2>/dev/null || true
  fi
  if [[ -n "${PIPE_PID:-}" ]] && kill -0 "${PIPE_PID}" 2>/dev/null; then
    if [[ "${PG_KILL}" == "true" ]]; then
      kill -TERM -"${PIPE_PID}" 2>/dev/null || true
      sleep 1
      kill -KILL -"${PIPE_PID}" 2>/dev/null || true
    else
      kill "${PIPE_PID}" 2>/dev/null || true
    fi
    wait "${PIPE_PID}" 2>/dev/null || true
  fi
  echo "Clean shutdown"
}

trap cleanup EXIT INT TERM HUP

echo "Starting multi-engine pipelines..."
if command -v setsid >/dev/null 2>&1; then
  setsid "${PY}" scripts/run_multi_engine.py > logs/pipeline.out 2>&1 &
  PG_KILL=true
else
  "${PY}" scripts/run_multi_engine.py > logs/pipeline.out 2>&1 &
fi
PIPE_PID=$!

echo "Starting Telemetry Archiver Daemon..."
"${PY}" scripts/telemetry_archive_daemon.py > logs/archiver.out 2>&1 &
DAEMON_PID=$!

sleep 2

if ! kill -0 "${PIPE_PID}" 2>/dev/null; then
  echo "Pipeline failed to start — last 40 lines of logs/pipeline.out:" >&2
  tail -n 40 logs/pipeline.out 2>/dev/null || true
  exit 1
fi

ok=false
for _ in 1 2 3 4 5 6 7 8; do
  if compgen -G "${LOG_DIR}/live_*.log*" >/dev/null 2>&1; then
    if find "${LOG_DIR}" -type f -name 'live_*.log*' -mmin -0.15 2>/dev/null | grep -q .; then
      ok=true
      break
    fi
  fi
  sleep 1
done

if [[ "${ok}" != "true" ]]; then
  echo "Pipelines are running but no fresh log updates detected yet (warming up or stalled — see logs/pipeline.out)." >&2
fi

if command -v open >/dev/null 2>&1; then
  ( sleep 3 && open "http://127.0.0.1:${PORT}" ) &
fi

echo "Starting Streamlit UI (port ${PORT})..."
streamlit run app.py --server.port "${PORT}"
