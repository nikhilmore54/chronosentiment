#!/usr/bin/env bash
# CS-P-006-C.3-D live-rule ecology. Does not evolve. Does not overwrite Search #2.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")" && pwd)"
cd "$ROOT"

TWO="${CSP006_SEARCH_TWO:-product_validation/CS-P-006/discovery/20260815T051900Z_c3}"
OUT="${CSP006_C3D_OUT:-$TWO/rule_ecology}"

if [[ "${DATABASE_URL:-}" == *"chrono_b3_test"* || "${DATABASE_URL:-}" == *"chrono_b4_test"* ]]; then
  echo "STOP: refusing certified database in DATABASE_URL" >&2
  exit 2
fi

echo "=== CS-P-006-C.3-D live-rule ecology (no Search #3) ==="
mkdir -p "$OUT"

cargo build --release -p chronosentiment_adapter --bin csp006_c3_rule_ecology
TARGET_DIR="$(cargo metadata --format-version 1 --no-deps | python3 -c 'import json,sys; print(json.load(sys.stdin)["target_directory"])')"
BIN="$TARGET_DIR/release/csp006_c3_rule_ecology"

"$BIN" \
  --search-two-dir "$ROOT/$TWO" \
  --output "$ROOT/$OUT"

(
  cd "$OUT"
  shasum -a 256 ecology.json ECOLOGY.md > SHA256SUMS
)

echo "Wrote $OUT"
echo "Search #2 selected_policy.json was not modified."
echo "Search #3 is not authorized."
