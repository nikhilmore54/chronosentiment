#!/usr/bin/env bash
# Capture a small, paste-friendly sample of [DIAG] / [RECOMMENDATION] / [ADAPTIVE_INTENT] only.
# Relaxed live-scale gates (see .cursor/rules / docs). Deterministic for fixed stream + env.
#
# Usage (repo root = cwd or any cwd):
#   bash scripts/capture_live_diag_sample.sh
#   bash scripts/capture_live_diag_sample.sh diag.log    # also writes max lines to diag.log
#
# Env overrides:
#   REAL_STREAM_SYMBOLS=10 STREAM_HEAD_LINES=3000 MAX_MATCHES=40 ELITE_PATH=core/elite/intraday_nse.json
#   LIVE_GATE_RECO_* , LIVE_GATE_CONF_MIN , RECO_DEBUG , POOL_DEBUG
#   DIAG_SAMPLE_LOG=diag.log  → tee to file (same as optional first arg)
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

export LIVE_GATE_RECO_STABILITY_MIN="${LIVE_GATE_RECO_STABILITY_MIN:-0.4}"
export LIVE_GATE_RECO_AGREEMENT_GLOBAL_MIN="${LIVE_GATE_RECO_AGREEMENT_GLOBAL_MIN:-0.25}"
export LIVE_GATE_RECO_FITNESS_MIN="${LIVE_GATE_RECO_FITNESS_MIN:-0.2}"
export LIVE_GATE_CONF_MIN="${LIVE_GATE_CONF_MIN:-0.5}"
export RECO_DEBUG="${RECO_DEBUG:-1}"
export POOL_DEBUG="${POOL_DEBUG:-1}"
export REAL_STREAM_SYMBOLS="${REAL_STREAM_SYMBOLS:-10}"

STREAM_HEAD_LINES="${STREAM_HEAD_LINES:-3000}"
MAX_MATCHES="${MAX_MATCHES:-40}"
LOG_OUT="${1:-${DIAG_SAMPLE_LOG:-}}"

if [[ ! -f "$ROOT/scripts/real_data_streamer.py" ]]; then
  echo "error: missing scripts/real_data_streamer.py" >&2
  exit 1
fi

run_filtered() {
  REAL_STREAM_SYMBOLS="$REAL_STREAM_SYMBOLS" python3 "$ROOT/scripts/real_data_streamer.py" 2>/dev/null |
    head -n "$STREAM_HEAD_LINES" |
    (cd "$ROOT/core" && cargo run --release --example live_engine 2>&1) |
    grep -E '^\[DIAG\]|^\[RECOMMENDATION\]|^\[ADAPTIVE_INTENT\]'
}

# head may SIGPIPE upstream (cargo); treat as normal stop once we have enough lines.
set +o pipefail
if [[ -n "$LOG_OUT" ]]; then
  run_filtered | head -n "$MAX_MATCHES" | tee "$LOG_OUT"
  echo "" >&2
  echo "Wrote up to $MAX_MATCHES lines to $LOG_OUT" >&2
else
  run_filtered | head -n "$MAX_MATCHES"
fi
set -o pipefail
exit 0
