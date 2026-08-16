#!/usr/bin/env bash
# CS-P-006 disposable 7-instrument research snapshot.
# Not B4. Not B5. Never writes chrono_b3_test / chrono_b4_test.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")" && pwd)"
cd "$ROOT"

STAMP="${CSP006_SNAPSHOT_STAMP:-$(date -u +%Y%m%dT%H%M%SZ)}"
OUT="${CSP006_SNAPSHOT_OUT:-product_validation/CS-P-006/snapshot/${STAMP}_7instrument}"
CACHE="$OUT/yahoo_cache"
FIVE_CACHE="${CSP006_FIVE_CACHE:-product_validation/assessment_enrichment_v0.1/yahoo_cache}"
FIVE_IDENTITY="${CSP006_FIVE_IDENTITY:-product_validation/assessment_enrichment_v0.1/provenance/identity_run1.txt}"

if [[ "${DATABASE_URL:-}" == *"chrono_b3_test"* || "${DATABASE_URL:-}" == *"chrono_b4_test"* ]]; then
  echo "STOP: refusing certified database in DATABASE_URL" >&2
  exit 2
fi

echo "=== CS-P-006 7-instrument snapshot (not B4, not B5) ==="
mkdir -p "$CACHE" "$OUT/provenance"

echo "=== Copy certified five-instrument Yahoo cache ==="
for ticker in HDFCBANK.NS ICICIBANK.NS INFY.NS RELIANCE.NS TCS.NS; do
  cp "$FIVE_CACHE/${ticker}.json" "$CACHE/${ticker}.json"
done

echo "=== Fetch IDEA.NS and MAHABANK.NS if missing ==="
export CHRONO_YAHOO_CACHE_DIR="$ROOT/$CACHE"
cargo build --release -p chronosentiment_adapter --bin csp006_research_snapshot
TARGET_DIR="$(cargo metadata --format-version 1 --no-deps | python3 -c 'import json,sys; print(json.load(sys.stdin)["target_directory"])')"
BIN="$TARGET_DIR/release/csp006_research_snapshot"

"$BIN" \
  --output "$ROOT/$OUT" \
  --yahoo-cache "$ROOT/$CACHE" \
  --five-instrument-identity "$ROOT/$FIVE_IDENTITY"

(
  cd "$OUT"
  shasum -a 256 \
    snapshot.json identity.txt CERTIFICATION.md certification.json PROVENANCE.md \
    yahoo_cache/*.json \
    > SHA256SUMS
)

echo "Wrote $OUT"
echo "B3/B4 dumps untouched. Not B5. 006-B.1 date freeze is not this step."
