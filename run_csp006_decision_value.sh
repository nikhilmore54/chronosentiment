#!/usr/bin/env bash
# CS-P-006-C.2-D decision-value landscape of sealed Search #1. Does not evolve.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")" && pwd)"
cd "$ROOT"

SEARCH="${CSP006_SEARCH_DIR:-product_validation/CS-P-006/discovery/20260814T195327Z}"
OUT="${CSP006_DECISION_VALUE_OUT:-$SEARCH/decision_value}"

if [[ "${DATABASE_URL:-}" == *"chrono_b3_test"* || "${DATABASE_URL:-}" == *"chrono_b4_test"* ]]; then
  echo "STOP: refusing certified database in DATABASE_URL" >&2
  exit 2
fi

echo "=== CS-P-006-C.2-D Search #1 decision-value landscape (no Search #2) ==="
mkdir -p "$OUT"

cargo build --release -p chronosentiment_adapter --bin csp006_decision_value
TARGET_DIR="$(cargo metadata --format-version 1 --no-deps | python3 -c 'import json,sys; print(json.load(sys.stdin)["target_directory"])')"
BIN="$TARGET_DIR/release/csp006_decision_value"

"$BIN" \
  --search-dir "$ROOT/$SEARCH" \
  --output "$ROOT/$OUT"

(
  cd "$OUT"
  shasum -a 256 rows.json landscape.json LANDSCAPE.md > SHA256SUMS
)

echo "Wrote $OUT"
echo "Search #1 evidence files were not modified."
