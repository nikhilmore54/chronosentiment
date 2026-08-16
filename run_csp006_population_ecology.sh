#!/usr/bin/env bash
# CS-P-006-C.2-P Search #1 population ecology. Identity-gated replay. Not Search #2.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")" && pwd)"
cd "$ROOT"

SEARCH="${CSP006_SEARCH_DIR:-product_validation/CS-P-006/discovery/20260814T195327Z}"
CACHE="${CSP006_YAHOO_CACHE:-product_validation/CS-P-006/snapshot/20260814T183851Z_7instrument/yahoo_cache}"
OUT="${CSP006_ECOLOGY_OUT:-$SEARCH/ecology}"

if [[ "${DATABASE_URL:-}" == *"chrono_b3_test"* || "${DATABASE_URL:-}" == *"chrono_b4_test"* ]]; then
  echo "STOP: refusing certified database in DATABASE_URL" >&2
  exit 2
fi

echo "=== CS-P-006-C.2-P Search #1 population ecology (no Search #2) ==="
mkdir -p "$OUT"

cargo build --release -p chronosentiment_adapter --bin csp006_population_ecology
TARGET_DIR="$(cargo metadata --format-version 1 --no-deps | python3 -c 'import json,sys; print(json.load(sys.stdin)["target_directory"])')"
BIN="$TARGET_DIR/release/csp006_population_ecology"

"$BIN" \
  --search-dir "$ROOT/$SEARCH" \
  --yahoo-cache "$ROOT/$CACHE" \
  --output "$ROOT/$OUT"

(
  cd "$OUT"
  shasum -a 256 archive.json ecology.json ECOLOGY.md > SHA256SUMS
)

echo "Wrote $OUT"
echo "Search #1 evidence files were not modified."
