#!/usr/bin/env bash
# CS-P-006-C.3-R — one complete Search #2 run. Does not overwrite Search #1.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")" && pwd)"
cd "$ROOT"

STAMP="${CSP006_C3_STAMP:-$(date -u +%Y%m%dT%H%M%SZ)}"
OUT="${CSP006_C3_OUT:-product_validation/CS-P-006/discovery/${STAMP}_c3}"
CACHE="${CSP006_YAHOO_CACHE:-product_validation/CS-P-006/snapshot/20260814T183851Z_7instrument/yahoo_cache}"
SEARCH_ONE="${CSP006_SEARCH_ONE:-product_validation/CS-P-006/discovery/20260814T195327Z}"

if [[ "${DATABASE_URL:-}" == *"chrono_b3_test"* || "${DATABASE_URL:-}" == *"chrono_b4_test"* ]]; then
  echo "STOP: refusing certified database in DATABASE_URL" >&2
  exit 2
fi

if [[ "$OUT" == "$SEARCH_ONE" || "$OUT" == *"/20260814T195327Z" ]]; then
  echo "STOP: refusing to write Search #2 into Search #1" >&2
  exit 2
fi

echo "=== CS-P-006-C.3-R Search #2 (one complete run; Search #1 immutable) ==="
mkdir -p "$OUT"

export CHRONO_YAHOO_CACHE_DIR="$ROOT/$CACHE"
cargo build --release -p chronosentiment_adapter --bin csp006_c3_search
TARGET_DIR="$(cargo metadata --format-version 1 --no-deps | python3 -c 'import json,sys; print(json.load(sys.stdin)["target_directory"])')"
BIN="$TARGET_DIR/release/csp006_c3_search"

"$BIN" \
  --output "$ROOT/$OUT" \
  --yahoo-cache "$ROOT/$CACHE" \
  --search-one-dir "$ROOT/$SEARCH_ONE"

(
  cd "$OUT"
  shasum -a 256 \
    search_evidence.json SEARCH.md \
    selected_policy.json SELECTED.md \
    archive.json COMPARISON.md PROVENANCE.md \
    > SHA256SUMS
  (cd harness && shasum -a 256 harness.json HARNESS.md table_a_decision_distribution.json table_b_decision_value.json > SHA256SUMS)
  (cd ecology && shasum -a 256 ecology.json ECOLOGY.md > SHA256SUMS)
  (cd recommendations && shasum -a 256 recommendations.json > SHA256SUMS)
)

echo "Wrote $OUT"
echo "Search #1 evidence files were not modified."
echo "This was one complete experiment. Do not iterate from these numbers."
