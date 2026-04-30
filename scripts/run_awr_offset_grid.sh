#!/usr/bin/env bash
# Deterministic AWR offset grid (non-overlapping: step >= LIMIT).
# Per `.cursor/rules`: same streamer/limit; only REAL_STREAM_OFFSET varies.
# Usage (from repo root):
#   ./scripts/run_awr_offset_grid.sh
# Optional env (override defaults):
#   ELITE_PATH, REAL_STREAM_GLOB, REAL_STREAM_SYMBOLS, REAL_STREAM_LIMIT,
#   REC_BLOCK_SYMBOLS, PAPER_EV_TABLE_PATH, GA_LEARN_PASS_MIN, etc.

set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

LIMIT="${REAL_STREAM_LIMIT:-3000}"

ENGINE="${ROOT}/target/release/examples/live_engine"
if [[ ! -x "$ENGINE" ]]; then
  echo "error: missing $ENGINE — run: cargo build --release -p chronosentiment_core --example live_engine" >&2
  exit 1
fi

OUT_DIR="${ROOT}/analysis/awr_grid"
mkdir -p "$OUT_DIR"

GLOB="${REAL_STREAM_GLOB:-analysis/coverage_buckets/largecap/*.csv}"
NSYM="${REAL_STREAM_SYMBOLS:-99}"

# Shortest data-row count among first NSYM CSVs (streamer stops when any file ends).
# Valid start offsets for full LIMIT-length replays: 0, LIMIT, 2*LIMIT, ... <= max_start.
export REAL_STREAM_GLOB_RESOLVE="$GLOB"
export REAL_STREAM_SYMBOLS_RESOLVE="$NSYM"
export REAL_STREAM_LIMIT_RESOLVE="$LIMIT"
read -r MIN_ROWS MAX_START <<<"$(python3 <<'PY'
import glob, os

pattern = os.environ["REAL_STREAM_GLOB_RESOLVE"]
nsym = int(os.environ["REAL_STREAM_SYMBOLS_RESOLVE"])
lim = int(os.environ["REAL_STREAM_LIMIT_RESOLVE"])
files = sorted(glob.glob(pattern))[:nsym]
if not files:
    print("0 0")
    raise SystemExit
mins = []
for path in files:
    with open(path, "rb") as f:
        rows = sum(1 for _ in f)
    mins.append(max(0, rows - 1))
short = min(mins)
max_start = max(0, short - lim)
print(short, max_start)
PY
)"
unset REAL_STREAM_GLOB_RESOLVE REAL_STREAM_SYMBOLS_RESOLVE REAL_STREAM_LIMIT_RESOLVE
echo "[AWR_GRID] shortest_csv_data_rows=${MIN_ROWS} max_start_offset=${MAX_START} step=${LIMIT} glob=${GLOB} symbols=${NSYM}" >&2

OFFSETS=()
if [[ "${MIN_ROWS}" -eq 0 ]]; then
  echo "error: no CSV rows matched glob=${GLOB}" >&2
  exit 1
fi
for ((O = 0; O <= MAX_START; O += LIMIT)); do
  OFFSETS+=("$O")
done
if [[ "${#OFFSETS[@]}" -eq 0 ]]; then
  OFFSETS=(0)
fi

for O in "${OFFSETS[@]}"; do
  echo "RUN offset=${O} limit=${LIMIT}" >&2
  LOG="${OUT_DIR}/awr_limit${LIMIT}_offset${O}.log"
  REAL_STREAM_GLOB="$GLOB" \
  REAL_STREAM_SYMBOLS="$NSYM" \
  REAL_STREAM_OFFSET="$O" \
  REAL_STREAM_LIMIT="$LIMIT" \
  python3 -u "${ROOT}/scripts/real_data_streamer.py" 2>/dev/null | \
  env \
    ELITE_PATH="${ELITE_PATH:-${ROOT}/core/elite/intraday_nse.json}" \
    REC_BLOCK_SYMBOLS="${REC_BLOCK_SYMBOLS:-}" \
    PAPER_EV_TABLE_PATH="${PAPER_EV_TABLE_PATH:-${ROOT}/analysis/ev_table_rich_survival.json}" \
    PAPER_EV_EXPIRE_THRESHOLD="${PAPER_EV_EXPIRE_THRESHOLD:-0.000000}" \
    GA_LEARN_FLOOR_ABS="${GA_LEARN_FLOOR_ABS:-0.000058}" \
    GA_LEARN_PASS_MIN="${GA_LEARN_PASS_MIN:-0.20}" \
    INTENT_MAX_AGE_BASE="${INTENT_MAX_AGE_BASE:-14}" \
    REC_CONFIRM_DELTA="${REC_CONFIRM_DELTA:-1}" \
    REC_CONFIRM_VOL_MULT="${REC_CONFIRM_VOL_MULT:-2.0}" \
    REC_CAND_VOTER_PCT="${REC_CAND_VOTER_PCT:-30}" \
    REC_CAND_CONF_PCT="${REC_CAND_CONF_PCT:-50}" \
    LIVE_GATE_RECO_STABILITY_MIN="${LIVE_GATE_RECO_STABILITY_MIN:-0.1}" \
    LIVE_GATE_RECO_AGREEMENT_GLOBAL_MIN="${LIVE_GATE_RECO_AGREEMENT_GLOBAL_MIN:-0.1}" \
    LIVE_GATE_RECO_FITNESS_MIN="${LIVE_GATE_RECO_FITNESS_MIN:-0.1}" \
    "$ENGINE" >/dev/null 2> "$LOG"

  awk 'index($0, "[AWR_SUMMARY]") { print "offset='"${O}"' | " $0 }' "$LOG" || true
done
