#!/usr/bin/env bash
# Bounded B4 repair populate. Never writes to B3 dump, chrono_b3_test, or G-GATE v1.1 artifacts.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")" && pwd)"
cd "$ROOT"

B3_DUMP="r3_evidence/20260813T035739Z_B3/db/full_dump.dump"
B3_EXPECT="af11d318b03fb171207f96348fcf210e1b9149b1ab6e699c06c363faec518788"
DBNAME="${B4_DBNAME:-chrono_b4_test}"
PGHOST="${PGHOST:-localhost}"
STAMP="$(date -u +%Y%m%dT%H%M%SZ)"
EVIDENCE="r3_evidence/${STAMP}_B4"

if [[ "$DBNAME" == "chrono_b3_test" ]]; then
  echo "STOP: refusing to populate chrono_b3_test" >&2
  exit 2
fi

B3_GOT="$(shasum -a 256 "$B3_DUMP" | awk '{print $1}')"
if [[ "$B3_GOT" != "$B3_EXPECT" ]]; then
  echo "STOP: B3 dump hash changed; aborting B4 so B3 evidence stays frozen" >&2
  exit 2
fi

echo "=== Build repaired populator ==="
cargo build --release -p chronosentiment_adapter --bin m4_populate_knowledge_lake
TARGET_DIR="$(cargo metadata --format-version 1 --no-deps | python3 -c 'import json,sys; print(json.load(sys.stdin)["target_directory"])')"
BIN="$TARGET_DIR/release/m4_populate_knowledge_lake"
BIN_SHA="$(shasum -a 256 "$BIN" | awk '{print $1}')"

echo "=== Create B4 database ${DBNAME} ==="
dropdb --if-exists -h "$PGHOST" "$DBNAME"
createdb -h "$PGHOST" "$DBNAME"

echo "=== Populate B4 (Yahoo fetch; same semantics as B3 except assessment dt stamp) ==="
export DATABASE_URL="postgresql://${PGUSER:-$(whoami)}@${PGHOST}:5432/${DBNAME}"
"$BIN"

mkdir -p "$EVIDENCE/db" "$EVIDENCE/provenance" "$EVIDENCE/binary"
pg_dump -Fc --no-owner --no-acl -h "$PGHOST" -d "$DBNAME" -f "$EVIDENCE/db/full_dump.dump"
pg_dump --schema-only -h "$PGHOST" -d "$DBNAME" > "$EVIDENCE/db/schema_after_populate.sql"
DUMP_SHA="$(shasum -a 256 "$EVIDENCE/db/full_dump.dump" | awk '{print $1}')"
echo "$DUMP_SHA  $ROOT/$EVIDENCE/db/full_dump.dump" > "$EVIDENCE/provenance/dataset_snapshot.sha256"
echo "$BIN_SHA  $BIN" > "$EVIDENCE/provenance/m4_populate_knowledge_lake.sha256"
cp "$BIN" "$EVIDENCE/binary/m4_populate_knowledge_lake" || true

echo "=== B4 invariants + E-GATE v3 ==="
set +e
"$ROOT/run_e_gate_v3.sh" "$DBNAME" "$EVIDENCE"
EGATE=$?
set -e
if [[ "$EGATE" -ne 0 ]]; then
  echo "E-GATE v3 FAIL — not running G-GATE"
  exit "$EGATE"
fi

echo "B4_EVIDENCE=$EVIDENCE"
echo "B4_DUMP_SHA256=$DUMP_SHA"
echo "B4_BINARY_SHA256=$BIN_SHA"
echo "E-GATE v3 PASS — G-GATE v1.1 may be run against this B4 snapshot only"
