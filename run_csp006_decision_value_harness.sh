#!/usr/bin/env bash
# CS-P-006-N decision-value harness. Measurement only. Does not evolve.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")" && pwd)"
cd "$ROOT"

SEARCH="${CSP006_SEARCH_DIR:-product_validation/CS-P-006/discovery/20260814T195327Z}"
OUT="${CSP006_HARNESS_OUT:-$SEARCH/harness}"

if [[ "${DATABASE_URL:-}" == *"chrono_b3_test"* || "${DATABASE_URL:-}" == *"chrono_b4_test"* ]]; then
  echo "STOP: refusing certified database in DATABASE_URL" >&2
  exit 2
fi

echo "=== CS-P-006-N decision-value harness (no evolution, C.3 not authorized) ==="
mkdir -p "$OUT"

cargo build --release -p chronosentiment_adapter --bin csp006_decision_value_harness
TARGET_DIR="$(cargo metadata --format-version 1 --no-deps | python3 -c 'import json,sys; print(json.load(sys.stdin)["target_directory"])')"
BIN="$TARGET_DIR/release/csp006_decision_value_harness"

"$BIN" \
  --search-dir "$ROOT/$SEARCH" \
  --output "$ROOT/$OUT"

(
  cd "$OUT"
  shasum -a 256 harness.json HARNESS.md table_a_decision_distribution.json table_b_decision_value.json > SHA256SUMS
)

echo "Wrote $OUT"
echo "Search #1 evidence files were not modified."
echo "C.3 is not authorized."
