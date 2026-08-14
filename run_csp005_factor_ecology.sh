#!/usr/bin/env bash
# CS-P-005 Factor Ecology Analysis v0.1
# Read-only restore of the certified enrichment snapshot. No candidate policy. No backtest.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")" && pwd)"
cd "$ROOT"

DUMP="${ENRICHMENT_DUMP:-product_validation/assessment_enrichment_v0.1/db/full_dump.dump}"
EXPECT="${ENRICHMENT_EXPECT:-e7685d936bdfaf53d7055ca683a87b4ca85149dd0eb89402dfaa93facfd8616f}"
CACHE="${ENRICHMENT_CACHE:-product_validation/assessment_enrichment_v0.1/yahoo_cache}"
DBNAME="${CSP005_DBNAME:-chrono_enrichment_ecology}"
PGHOST="${PGHOST:-localhost}"
PGUSER="${PGUSER:-$(whoami)}"
OUT="${CSP005_OUT:-product_validation/CS-P-005_factor_ecology_v0.1}"

if [[ "$DBNAME" == "chrono_b3_test" || "$DBNAME" == "chrono_b4_test" ]]; then
  echo "STOP: refusing certified database name $DBNAME" >&2
  exit 2
fi

GOT="$(shasum -a 256 "$DUMP" | awk '{print $1}')"
if [[ "$GOT" != "$EXPECT" ]]; then
  echo "STOP: enrichment dump hash mismatch" >&2
  echo " expected $EXPECT" >&2
  echo " got      $GOT" >&2
  exit 2
fi

dropdb --if-exists -h "$PGHOST" "$DBNAME"
createdb -h "$PGHOST" "$DBNAME"
pg_restore --no-owner --no-acl -h "$PGHOST" --dbname="$DBNAME" "$DUMP"

export DATABASE_URL="postgresql://${PGUSER}@${PGHOST}:5432/${DBNAME}"
cargo run -p chronosentiment_adapter --bin csp005_factor_ecology -- \
  --output "$OUT" --yahoo-cache "$ROOT/$CACHE"

(
  cd "$OUT"
  shasum -a 256 FACTOR_ECOLOGY.md DESIGN_CONSTRAINTS.md ecology.json rows.json > SHA256SUMS
)

dropdb --if-exists -h "$PGHOST" "$DBNAME"
echo "B3/B4 untouched. Not B5. No candidate policy. Reports: $OUT/"
