#!/usr/bin/env bash
# CS-P-006-P.E Targeted Decision Execution. Does not retune C3-002.
# Does not mutate Replay v0/v1 or the 14 August prospective cohort.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")" && pwd)"
cd "$ROOT"

TWO="${CSP006_SEARCH_TWO:-product_validation/CS-P-006/discovery/20260815T051900Z_c3}"
CACHE="${CSP006_YAHOO_CACHE:-product_validation/CS-P-006/snapshot/20260814T183851Z_7instrument/yahoo_cache}"
OUT="${CSP006_P_EXECUTE_OUT:-product_validation/CS-P-006/observatory/targeted_execution_v0}"
NOW="${CSP006_P_EXECUTE_NOW:-2026-08-15T06:30:00Z}"

if [[ "${DATABASE_URL:-}" == *"chrono_b3_test"* || "${DATABASE_URL:-}" == *"chrono_b4_test"* ]]; then
  echo "STOP: refusing certified database in DATABASE_URL" >&2
  exit 2
fi
if [[ "$OUT" == *"observatory/prospective"* || "$OUT" == *"historical_replay_v0"* || "$OUT" == *"historical_replay_v1"* ]]; then
  echo "STOP: refusing to overwrite protected Observatory ledgers" >&2
  exit 2
fi
if [[ "$OUT" == *"targeted_execution_v0"* ]]; then
  echo "STOP: P.E.1 sidecar targeted_execution_v0 is frozen" >&2
  exit 2
fi

echo "=== CS-P-006-P.E Targeted Decision Execution (not C.3-G) ==="
mkdir -p "$OUT"

cargo build --release -p chronosentiment_adapter --bin csp006_p_execute
TARGET_DIR="$(cargo metadata --format-version 1 --no-deps | python3 -c 'import json,sys; print(json.load(sys.stdin)["target_directory"])')"
BIN="$TARGET_DIR/release/csp006_p_execute"

"$BIN" \
  --search-two-dir "$ROOT/$TWO" \
  --yahoo-cache "$ROOT/$CACHE" \
  --output "$ROOT/$OUT" \
  --now "$NOW"

(
  cd "$OUT"
  shasum -a 256 intents.json report.json REPORT.md evidence.html CONTRACT.txt > SHA256SUMS
)

echo "Wrote $OUT"
echo "Search #2 selected_policy.json was not modified."
echo "Replay v0/v1 and the 14 August prospective cohort were not written."
echo "C.3-G was not started."
echo "Search #3 is not authorized."
