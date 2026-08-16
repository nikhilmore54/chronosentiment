#!/usr/bin/env bash
# CS-P-006-C.3-F certified TMV state x action landscape. Does not evolve.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")" && pwd)"
cd "$ROOT"

ONE="${CSP006_SEARCH_ONE:-product_validation/CS-P-006/discovery/20260814T195327Z}"
TWO="${CSP006_SEARCH_TWO:-product_validation/CS-P-006/discovery/20260815T051900Z_c3}"
OUT="${CSP006_C3F_OUT:-$TWO/state_landscape}"

if [[ "${DATABASE_URL:-}" == *"chrono_b3_test"* || "${DATABASE_URL:-}" == *"chrono_b4_test"* ]]; then
  echo "STOP: refusing certified database in DATABASE_URL" >&2
  exit 2
fi

echo "=== CS-P-006-C.3-F state x action landscape (no Search #3) ==="
mkdir -p "$OUT"

cargo build --release -p chronosentiment_adapter --bin csp006_c3_state_landscape
TARGET_DIR="$(cargo metadata --format-version 1 --no-deps | python3 -c 'import json,sys; print(json.load(sys.stdin)["target_directory"])')"
BIN="$TARGET_DIR/release/csp006_c3_state_landscape"

"$BIN" \
  --search-one-dir "$ROOT/$ONE" \
  --search-two-dir "$ROOT/$TWO" \
  --output "$ROOT/$OUT"

(
  cd "$OUT"
  shasum -a 256 landscape.json LANDSCAPE.md > SHA256SUMS
)

echo "Wrote $OUT"
echo "Neither selected_policy.json was modified."
echo "No product claim is authorized."
echo "Search #3 is not authorized."
