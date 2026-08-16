#!/usr/bin/env bash
# CS-P-006-P.H Historical Observatory Replay. Does not evolve. Does not start C.3-G.
# Does not mutate the 14 August prospective cohort.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")" && pwd)"
cd "$ROOT"

TWO="${CSP006_SEARCH_TWO:-product_validation/CS-P-006/discovery/20260815T051900Z_c3}"
CACHE="${CSP006_YAHOO_CACHE:-product_validation/CS-P-006/snapshot/20260814T183851Z_7instrument/yahoo_cache}"
OUT="${CSP006_P_REPLAY_OUT:-product_validation/CS-P-006/observatory/historical_replay_v1}"
NOW="${CSP006_P_REPLAY_NOW:-2026-08-15T06:30:00Z}"

if [[ "${DATABASE_URL:-}" == *"chrono_b3_test"* || "${DATABASE_URL:-}" == *"chrono_b4_test"* ]]; then
  echo "STOP: refusing certified database in DATABASE_URL" >&2
  exit 2
fi
if [[ "$OUT" == *"observatory/prospective"* ]]; then
  echo "STOP: refusing to write the prospective cohort" >&2
  exit 2
fi
if [[ "$OUT" == *"historical_replay_v0"* ]]; then
  echo "STOP: refusing to overwrite Replay v0" >&2
  exit 2
fi

echo "=== CS-P-006-P.H Historical Observatory Replay (not C.3-G) ==="
mkdir -p "$OUT"

cargo build --release -p chronosentiment_adapter --bin csp006_p_replay
TARGET_DIR="$(cargo metadata --format-version 1 --no-deps | python3 -c 'import json,sys; print(json.load(sys.stdin)["target_directory"])')"
BIN="$TARGET_DIR/release/csp006_p_replay"

"$BIN" \
  --search-two-dir "$ROOT/$TWO" \
  --yahoo-cache "$ROOT/$CACHE" \
  --output "$ROOT/$OUT" \
  --now "$NOW"

(
  cd "$OUT"
  shasum -a 256 ledger.json REPORT.md observatory.html report.json > SHA256SUMS
)

echo "Wrote $OUT"
echo "Search #2 selected_policy.json was not modified."
echo "The 14 August prospective cohort was not written."
echo "C.3-G was not started."
echo "Search #3 is not authorized."
