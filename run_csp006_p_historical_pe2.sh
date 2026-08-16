#!/usr/bin/env bash
# CS-P-006-P.E.2.H historical time-machine of the frozen P.E.2 control.
# Does not modify the P.E.2 specification. Does not mutate 14 August, P.E.1,
# Replay v0/v1, or live prospective_execution_v0.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")" && pwd)"
cd "$ROOT"

TWO="${CSP006_SEARCH_TWO:-product_validation/CS-P-006/discovery/20260815T051900Z_c3}"
CACHE="${CSP006_YAHOO_CACHE:-product_validation/CS-P-006/snapshot/20260814T183851Z_7instrument/yahoo_cache}"
OUT="${CSP006_P_HISTORICAL_PE2_OUT:-product_validation/CS-P-006/observatory/historical_pe2_replay}"

if [[ "${DATABASE_URL:-}" == *"chrono_b3_test"* || "${DATABASE_URL:-}" == *"chrono_b4_test"* ]]; then
  echo "STOP: refusing certified database in DATABASE_URL" >&2
  exit 2
fi
if [[ "$OUT" == *"observatory/prospective"* && "$OUT" != *"historical_pe2"* ]]; then
  echo "STOP: refusing to overwrite the 14 August prospective ledger" >&2
  exit 2
fi
if [[ "$OUT" == *"prospective_execution_v0"* || "$OUT" == *"targeted_execution_v0"* || "$OUT" == *"historical_replay_v0"* || "$OUT" == *"historical_replay_v1"* ]]; then
  echo "STOP: refusing to overwrite protected Observatory ledgers" >&2
  exit 2
fi

echo "=== CS-P-006-P.E.2.H Historical P.E.2 Lifecycle Validation (not C.3-G) ==="
mkdir -p "$OUT"

cargo build --release -p chronosentiment_adapter --bin csp006_p_historical_pe2
TARGET_DIR="$(cargo metadata --format-version 1 --no-deps | python3 -c 'import json,sys; print(json.load(sys.stdin)["target_directory"])')"
BIN="$TARGET_DIR/release/csp006_p_historical_pe2"

"$BIN" \
  --search-two-dir "$ROOT/$TWO" \
  --yahoo-cache "$ROOT/$CACHE" \
  --output "$ROOT/$OUT"

(
  cd "$OUT"
  shasum -a 256 ledger.json REPORT.md evidence.html CONTRACT.txt > SHA256SUMS
)

echo "Wrote $OUT"
echo "Search #2 selected_policy.json was not modified."
echo "The 14 August prospective cohort was not written."
echo "Live prospective_execution_v0 was not written."
echo "P.E.1 targeted_execution_v0 was not written."
echo "Replay v0/v1 were not written."
echo "C.3-G was not started."
echo "Search #3 is not authorized."
echo "P.E.3 was not started."
