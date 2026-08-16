#!/usr/bin/env bash
# CS-P-006-C first Coralys TMV discovery run.
# Not B4. Not B5. Never writes chrono_b3_test / chrono_b4_test.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")" && pwd)"
cd "$ROOT"

STAMP="${CSP006_DISCOVERY_STAMP:-$(date -u +%Y%m%dT%H%M%SZ)}"
OUT="${CSP006_DISCOVERY_OUT:-product_validation/CS-P-006/discovery/${STAMP}}"
CACHE="${CSP006_YAHOO_CACHE:-product_validation/CS-P-006/snapshot/20260814T183851Z_7instrument/yahoo_cache}"

if [[ "${DATABASE_URL:-}" == *"chrono_b3_test"* || "${DATABASE_URL:-}" == *"chrono_b4_test"* ]]; then
  echo "STOP: refusing certified database in DATABASE_URL" >&2
  exit 2
fi

echo "=== CS-P-006-C Coralys TMV discovery (not B4, not B5) ==="
mkdir -p "$OUT"

export CHRONO_YAHOO_CACHE_DIR="$ROOT/$CACHE"
cargo build --release -p chronosentiment_adapter --bin csp006_policy_discovery
TARGET_DIR="$(cargo metadata --format-version 1 --no-deps | python3 -c 'import json,sys; print(json.load(sys.stdin)["target_directory"])')"
BIN="$TARGET_DIR/release/csp006_policy_discovery"

"$BIN" \
  --output "$ROOT/$OUT" \
  --yahoo-cache "$ROOT/$CACHE"

(
  cd "$OUT"
  shasum -a 256 \
    search_evidence.json SEARCH.md \
    selected_policy.json SELECTED.md \
    evaluation_handoff.json EVALUATION.md \
    PROVENANCE.md \
    > SHA256SUMS
)

echo "Wrote $OUT"
echo "B3/B4 dumps untouched. Not B5. Evaluation was not fed back to Coralys."
