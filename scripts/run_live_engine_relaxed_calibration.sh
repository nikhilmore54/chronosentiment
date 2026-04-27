#!/usr/bin/env bash
# Live engine with relaxed reco + meta gates (live-scale calibration, not train-scale S).
# Deterministic for fixed stdin + env (ChronoSentiment core — see .cursor/rules / docs).
#
# Usage (stdin is JSON lines of candle batches):
#   bash scripts/run_live_engine_relaxed_calibration.sh < your_stream.jsonl
#   REAL_STREAM_SYMBOLS=3 python3 scripts/real_data_streamer.py | bash scripts/run_live_engine_relaxed_calibration.sh
#
# Override any threshold by exporting before invoke, e.g. LIVE_GATE_RECO_STABILITY_MIN=0.35
#
# Filtered sample for pasting (DIAG / RECOMMENDATION / ADAPTIVE_INTENT only):
#   bash scripts/capture_live_diag_sample.sh
#   bash scripts/capture_live_diag_sample.sh diag.log
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT/core"

export LIVE_GATE_RECO_STABILITY_MIN="${LIVE_GATE_RECO_STABILITY_MIN:-0.4}"
export LIVE_GATE_RECO_AGREEMENT_GLOBAL_MIN="${LIVE_GATE_RECO_AGREEMENT_GLOBAL_MIN:-0.25}"
export LIVE_GATE_RECO_FITNESS_MIN="${LIVE_GATE_RECO_FITNESS_MIN:-0.2}"
export LIVE_GATE_CONF_MIN="${LIVE_GATE_CONF_MIN:-0.5}"
export RECO_DEBUG="${RECO_DEBUG:-1}"
export POOL_DEBUG="${POOL_DEBUG:-1}"

exec cargo run --release --example live_engine
