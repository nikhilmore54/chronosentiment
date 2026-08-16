#!/usr/bin/env bash
# CS-P-006-C.1 diagnosis of sealed Search #1. Does not evolve. Does not overwrite Search #1 files.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")" && pwd)"
cd "$ROOT"

SEARCH="${CSP006_SEARCH_DIR:-product_validation/CS-P-006/discovery/20260814T195327Z}"
CACHE="${CSP006_YAHOO_CACHE:-product_validation/CS-P-006/snapshot/20260814T183851Z_7instrument/yahoo_cache}"
OUT="${CSP006_DIAGNOSIS_OUT:-$SEARCH/diagnosis}"

if [[ "${DATABASE_URL:-}" == *"chrono_b3_test"* || "${DATABASE_URL:-}" == *"chrono_b4_test"* ]]; then
  echo "STOP: refusing certified database in DATABASE_URL" >&2
  exit 2
fi

echo "=== CS-P-006-C.1 Search #1 diagnosis (no evolution) ==="
mkdir -p "$OUT"

cargo build --release -p chronosentiment_adapter --bin csp006_search_diagnosis
TARGET_DIR="$(cargo metadata --format-version 1 --no-deps | python3 -c 'import json,sys; print(json.load(sys.stdin)["target_directory"])')"
BIN="$TARGET_DIR/release/csp006_search_diagnosis"

"$BIN" \
  --search-dir "$ROOT/$SEARCH" \
  --yahoo-cache "$ROOT/$CACHE" \
  --output "$ROOT/$OUT"

(
  cd "$OUT"
  shasum -a 256 diagnosis.json DIAGNOSIS.md > SHA256SUMS
)

echo "Wrote $OUT"
echo "Search #1 evidence files were not modified."
