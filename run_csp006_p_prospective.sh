#!/usr/bin/env bash
# CS-P-006-P prospective C3-002 paper clock. Does not evolve. Does not start C.3-G.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")" && pwd)"
cd "$ROOT"

TWO="${CSP006_SEARCH_TWO:-product_validation/CS-P-006/discovery/20260815T051900Z_c3}"
HIST="${CSP006_P_HIST:-product_validation/CS-P-006/observatory}"
OUT="${CSP006_P_PROSPECTIVE_OUT:-product_validation/CS-P-006/observatory/prospective}"

if [[ "${DATABASE_URL:-}" == *"chrono_b3_test"* || "${DATABASE_URL:-}" == *"chrono_b4_test"* ]]; then
  echo "STOP: refusing certified database in DATABASE_URL" >&2
  exit 2
fi

echo "=== CS-P-006-P prospective C3-002 (no Search #3) ==="
mkdir -p "$OUT"

cargo build --release -p chronosentiment_adapter --bin csp006_p_prospective
TARGET_DIR="$(cargo metadata --format-version 1 --no-deps | python3 -c 'import json,sys; print(json.load(sys.stdin)["target_directory"])')"
BIN="$TARGET_DIR/release/csp006_p_prospective"

"$BIN" \
  --search-two-dir "$ROOT/$TWO" \
  --historical-dir "$ROOT/$HIST" \
  --output "$ROOT/$OUT"

(
  cd "$OUT"
  shasum -a 256 ledger.json observatory.html > SHA256SUMS
)

echo "Wrote $OUT"
echo "Search #2 selected_policy.json was not modified."
echo "No outcomes were attached."
echo "C.3-G was not started."
echo "Search #3 is not authorized."
echo "This is not CS-P-003 validation."
