#!/usr/bin/env bash
# Assessment Enrichment v0.1 — information-fidelity snapshot.
# Generate → temporal/lineage certify → factor-availability report. Stop.
#
# Not B5. Not G-GATE. Not a trading-strategy experiment. Not Decision Engine v1.0.
# Never writes chrono_b3_test / chrono_b4_test. Does not mutate B3/B4 dumps.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")" && pwd)"
cd "$ROOT"

DBNAME="${ENRICHMENT_DBNAME:-chrono_enrichment_v01}"
REPEAT_DB="${ENRICHMENT_REPEAT_DBNAME:-chrono_enrichment_v01_repeat}"
PGHOST="${PGHOST:-localhost}"
PGUSER="${PGUSER:-$(whoami)}"
OUT="${ENRICHMENT_OUT:-product_validation/assessment_enrichment_v0.1}"
CACHE="$OUT/yahoo_cache"
TEMPORAL="$OUT/temporal_admissibility.jsonl"

refuse_certified() {
  local name=$1
  if [[ "$name" == "chrono_b3_test" || "$name" == "chrono_b4_test" ]]; then
    echo "STOP: refusing certified database name $name" >&2
    exit 2
  fi
}

refuse_certified "$DBNAME"
refuse_certified "$REPEAT_DB"

echo "=== Assessment Enrichment v0.1 snapshot (not B5) ==="
mkdir -p "$OUT/db" "$OUT/provenance" "$CACHE"

echo "=== Build populator + certifier ==="
cargo build --release -p chronosentiment_adapter \
  --bin m4_populate_knowledge_lake \
  --bin csp004_enrichment_certify
TARGET_DIR="$(cargo metadata --format-version 1 --no-deps | python3 -c 'import json,sys; print(json.load(sys.stdin)["target_directory"])')"
BIN="$TARGET_DIR/release/m4_populate_knowledge_lake"
CERT="$TARGET_DIR/release/csp004_enrichment_certify"
BIN_SHA="$(shasum -a 256 "$BIN" | awk '{print $1}')"

populate_one() {
  local db=$1
  local write_temporal=${2:-0}
  refuse_certified "$db"
  dropdb --if-exists -h "$PGHOST" "$db"
  createdb -h "$PGHOST" "$db"
  export DATABASE_URL="postgresql://${PGUSER}@${PGHOST}:5432/${db}"
  export CHRONO_YAHOO_CACHE_DIR="$ROOT/$CACHE"
  if [[ "$write_temporal" == "1" ]]; then
    export CHRONO_TEMPORAL_LOG="$ROOT/$TEMPORAL"
  else
    unset CHRONO_TEMPORAL_LOG || true
  fi
  "$BIN"
}

echo "=== Populate ${DBNAME} (Yahoo fetch caches into ${CACHE}) ==="
rm -f "$TEMPORAL"
populate_one "$DBNAME" 1

echo "=== Dump snapshot ==="
pg_dump -Fc --no-owner --no-acl -h "$PGHOST" -d "$DBNAME" -f "$OUT/db/full_dump.dump"
pg_dump --schema-only -h "$PGHOST" -d "$DBNAME" > "$OUT/db/schema_after_populate.sql"
DUMP_SHA="$(shasum -a 256 "$OUT/db/full_dump.dump" | awk '{print $1}')"
echo "$DUMP_SHA  $ROOT/$OUT/db/full_dump.dump" > "$OUT/provenance/dataset_snapshot.sha256"
echo "$BIN_SHA  $BIN" > "$OUT/provenance/m4_populate_knowledge_lake.sha256"

identity_sql() {
  local db=$1
  psql -h "$PGHOST" -d "$db" -t -A -c "
    SELECT i.display_symbol || E'\t' || a.evaluation_timestamp || E'\t' || a.signature_hash
    FROM knowledge_assessments a
    JOIN instruments i ON i.id = a.instrument_id
    ORDER BY i.display_symbol, a.evaluation_timestamp, a.signature_hash;
  "
}

echo "=== Repeat populate from cache into ${REPEAT_DB} ==="
populate_one "$REPEAT_DB"
identity_sql "$DBNAME" > "$OUT/provenance/identity_run1.txt"
identity_sql "$REPEAT_DB" > "$OUT/provenance/identity_run2.txt"
if ! diff -q "$OUT/provenance/identity_run1.txt" "$OUT/provenance/identity_run2.txt" >/dev/null; then
  echo "FAIL: repeated snapshot signature hashes differ" >&2
  diff -u "$OUT/provenance/identity_run1.txt" "$OUT/provenance/identity_run2.txt" | head -n 80 >&2 || true
  dropdb --if-exists -h "$PGHOST" "$REPEAT_DB"
  exit 1
fi
echo "PASS: repeated snapshot signature hashes identical (symbol, T, signature_hash)"
dropdb --if-exists -h "$PGHOST" "$REPEAT_DB"

echo "=== Temporal + lineage certification + factor availability ==="
export DATABASE_URL="postgresql://${PGUSER}@${PGHOST}:5432/${DBNAME}"
set +e
"$CERT" --output "$OUT" --yahoo-cache "$ROOT/$CACHE"
CERT_STATUS=$?
set -e

(
  cd "$OUT"
  shasum -a 256 \
    FACTOR_AVAILABILITY.md CERTIFICATION.md \
    factor_availability.json certification.json \
    db/full_dump.dump provenance/identity_run1.txt \
    > SHA256SUMS
)

dropdb --if-exists -h "$PGHOST" "$DBNAME"

echo "DUMP_SHA256=$DUMP_SHA"
echo "BINARY_SHA256=$BIN_SHA"
echo "Reports: $OUT/"
echo "B3/B4 dumps untouched. Not B5. G-GATE closed. Engine remains unfrozen-dev."
exit "$CERT_STATUS"
