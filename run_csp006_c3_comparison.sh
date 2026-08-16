#!/usr/bin/env bash
# CS-P-006-C.3-C sealed-artifact comparison. Does not evolve. Does not overwrite policies.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")" && pwd)"
cd "$ROOT"

ONE="${CSP006_SEARCH_ONE:-product_validation/CS-P-006/discovery/20260814T195327Z}"
TWO="${CSP006_SEARCH_TWO:-product_validation/CS-P-006/discovery/20260815T051900Z_c3}"
OUT="${CSP006_C3C_OUT:-$TWO/review}"

if [[ "${DATABASE_URL:-}" == *"chrono_b3_test"* || "${DATABASE_URL:-}" == *"chrono_b4_test"* ]]; then
  echo "STOP: refusing certified database in DATABASE_URL" >&2
  exit 2
fi

echo "=== CS-P-006-C.3-C comparative review (no Search #3) ==="
mkdir -p "$OUT"

cargo build --release -p chronosentiment_adapter --bin csp006_c3_comparison
TARGET_DIR="$(cargo metadata --format-version 1 --no-deps | python3 -c 'import json,sys; print(json.load(sys.stdin)["target_directory"])')"
BIN="$TARGET_DIR/release/csp006_c3_comparison"

"$BIN" \
  --search-one-dir "$ROOT/$ONE" \
  --search-two-dir "$ROOT/$TWO" \
  --output "$ROOT/$OUT"

(
  cd "$OUT"
  shasum -a 256 comparison.json REVIEW.md pairwise_rows.json conversion_rows.json action_matrix.json > SHA256SUMS
)

echo "Wrote $OUT"
echo "Search #1 and Search #2 selected_policy.json were not modified."
echo "Search #3 is not authorized."
