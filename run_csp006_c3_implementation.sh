#!/usr/bin/env bash
# CS-P-006-C.3-I implementation verification. Does not evolve Search #2.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")" && pwd)"
cd "$ROOT"

SEARCH="${CSP006_SEARCH_DIR:-product_validation/CS-P-006/discovery/20260814T195327Z}"
OUT="${CSP006_C3I_OUT:-$SEARCH/c3i}"

if [[ "${DATABASE_URL:-}" == *"chrono_b3_test"* || "${DATABASE_URL:-}" == *"chrono_b4_test"* ]]; then
  echo "STOP: refusing certified database in DATABASE_URL" >&2
  exit 2
fi

echo "=== CS-P-006-C.3-I implementation verification (Search #2 not run) ==="
mkdir -p "$OUT"

cargo build --release -p chronosentiment_adapter --bin csp006_c3_implementation
TARGET_DIR="$(cargo metadata --format-version 1 --no-deps | python3 -c 'import json,sys; print(json.load(sys.stdin)["target_directory"])')"
BIN="$TARGET_DIR/release/csp006_c3_implementation"

"$BIN" \
  --search-dir "$ROOT/$SEARCH" \
  --output "$ROOT/$OUT"

(
  cd "$OUT"
  shasum -a 256 verification.json IMPLEMENTATION.md > SHA256SUMS
)

echo "Wrote $OUT"
echo "Search #1 evidence files were not modified."
echo "Search #2 was not run."
