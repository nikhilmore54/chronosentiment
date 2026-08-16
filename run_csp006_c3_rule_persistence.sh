#!/usr/bin/env bash
# CS-P-006-C.3-E discovered-rule persistence. Does not evolve. Does not overwrite Search #2.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")" && pwd)"
cd "$ROOT"

TWO="${CSP006_SEARCH_TWO:-product_validation/CS-P-006/discovery/20260815T051900Z_c3}"
OUT="${CSP006_C3E_OUT:-$TWO/rule_persistence}"

if [[ "${DATABASE_URL:-}" == *"chrono_b3_test"* || "${DATABASE_URL:-}" == *"chrono_b4_test"* ]]; then
  echo "STOP: refusing certified database in DATABASE_URL" >&2
  exit 2
fi

echo "=== CS-P-006-C.3-E discovered-rule persistence (no Search #3) ==="
mkdir -p "$OUT"

cargo build --release -p chronosentiment_adapter --bin csp006_c3_rule_persistence
TARGET_DIR="$(cargo metadata --format-version 1 --no-deps | python3 -c 'import json,sys; print(json.load(sys.stdin)["target_directory"])')"
BIN="$TARGET_DIR/release/csp006_c3_rule_persistence"

"$BIN" \
  --search-two-dir "$ROOT/$TWO" \
  --output "$ROOT/$OUT"

(
  cd "$OUT"
  shasum -a 256 persistence.json PERSISTENCE.md > SHA256SUMS
)

echo "Wrote $OUT"
echo "Search #2 selected_policy.json was not modified."
echo "No pass/fail threshold was introduced."
echo "Search #3 is not authorized."
