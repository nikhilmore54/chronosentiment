#!/usr/bin/env bash
# Verify: GA training writes elites → live_engine loads ELITE_PATH → recommendations → paper summary.
# Run from anywhere. ChronoSentiment: deterministic pipeline (see .cursor/rules / docs).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
# All Rust examples assume cwd = core/ (elite path relative to here).
cd "$ROOT/core"

export DATA_FOLDER="${DATA_FOLDER:-$ROOT/data/nse/5m}"
# Written by training_nse and read by live_engine when set.
export ELITE_PATH="${ELITE_PATH:-elite/intraday_nse.json}"
export GA_POPULATION_SIZE="${GA_POPULATION_SIZE:-5}"
export GA_GENERATIONS="${GA_GENERATIONS:-2}"
export NSE_TRAIN_MAX_ASSETS="${NSE_TRAIN_MAX_ASSETS:-3}"
export NSE_DETERMINISTIC="${NSE_DETERMINISTIC:-1}"

echo "== 1) GA training (training_nse) → elites → ${ELITE_PATH} =="
cargo run --release --example training_nse -- \
  --pop "${GA_POPULATION_SIZE}" \
  --gens "${GA_GENERATIONS}" \
  --slice "${NSE_TRAIN_MAX_ASSETS}" \
  --deterministic

echo ""
echo "== 2) Live recommendations + paper (stream cap; GA_BOOTSTRAP eases short replay) =="
export GA_BOOTSTRAP="${GA_BOOTSTRAP:-1}"
export REAL_STREAM_SYMBOLS="${REAL_STREAM_SYMBOLS:-3}"
# Live-scale reco gates (proxy pool weaker than train dashboard); override in env if needed.
export LIVE_GATE_RECO_STABILITY_MIN="${LIVE_GATE_RECO_STABILITY_MIN:-0.4}"
export LIVE_GATE_RECO_AGREEMENT_GLOBAL_MIN="${LIVE_GATE_RECO_AGREEMENT_GLOBAL_MIN:-0.25}"
export LIVE_GATE_RECO_FITNESS_MIN="${LIVE_GATE_RECO_FITNESS_MIN:-0.2}"
export LIVE_GATE_CONF_MIN="${LIVE_GATE_CONF_MIN:-0.5}"
(
  cd "$ROOT" && REAL_STREAM_SYMBOLS="${REAL_STREAM_SYMBOLS}" python3 scripts/real_data_streamer.py 2>/dev/null |
    head -n 1200
) | ELITE_PATH="$ELITE_PATH" GA_BOOTSTRAP="${GA_BOOTSTRAP}" \
  EDGE_PROBE="${EDGE_PROBE:-}" \
  EMIT_PROBE="${EMIT_PROBE:-}" \
  EXIT_PROBE="${EXIT_PROBE:-}" \
  LIVE_GATE_RECO_STABILITY_MIN="${LIVE_GATE_RECO_STABILITY_MIN}" \
  LIVE_GATE_RECO_AGREEMENT_GLOBAL_MIN="${LIVE_GATE_RECO_AGREEMENT_GLOBAL_MIN}" \
  LIVE_GATE_RECO_FITNESS_MIN="${LIVE_GATE_RECO_FITNESS_MIN}" \
  LIVE_GATE_CONF_MIN="${LIVE_GATE_CONF_MIN}" \
  cargo run --release --example live_engine 2>&1 |
  grep -E 'loaded [0-9]+ trained|\[RECOMMENDATION\]|\[ADAPTIVE_INTENT\]|\[FINALIZE\]|FINAL PAPER STATS|Trades |Win Rate|Equity Final|Avg PnL|\[EDGE_PIPE\]|\[EDGE_COMPONENTS\]|\[EMIT_TRACE\]|\[POOL_SIZE\]|\[EXIT_TRACE\]|\[TRADE_PATH\]|\[EXIT\]' || true

echo ""
echo "Done. Full live_engine log omitted; re-run without grep to see heartbeats and paper details."
