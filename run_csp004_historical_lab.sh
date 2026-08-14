#!/usr/bin/env bash
# CS-P-004 Historical Research Laboratory on a disposable B4 restore.
# Does not modify the B4 dump, chrono_b4_test, chrono_b3_test, or the Decision Engine.
# CS-P-003 forward clock is independent and may keep running.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")" && pwd)"
cd "$ROOT"

B4_DUMP="${B4_DUMP:-r3_evidence/20260814T023457Z_B4/db/full_dump.dump}"
B4_EXPECT="${B4_EXPECT:-f74e576e8e98b24058cc913b14a567d9ff4b3eabc75662ab9a96901b102f8cd6}"
DBNAME="${CSP004_DBNAME:-chrono_replay_b4_csp004}"
PGHOST="${PGHOST:-localhost}"
PGUSER="${PGUSER:-$(whoami)}"
OUT="${CSP004_OUT:-product_validation/CS-P-004_adapter_v0.1}"

hash_of() { shasum -a 256 "$1" | awk '{print $1}'; }

if [[ "$DBNAME" == "chrono_b3_test" || "$DBNAME" == "chrono_b4_test" ]]; then
  echo "STOP: refusing certified database name $DBNAME" >&2
  exit 2
fi

echo "=== 1. Verify B4 dump identity ==="
B4_GOT="$(hash_of "$B4_DUMP")"
if [[ "$B4_GOT" != "$B4_EXPECT" ]]; then
  echo "STOP: B4 dump hash mismatch" >&2
  echo " expected $B4_EXPECT" >&2
  echo " got      $B4_GOT" >&2
  exit 2
fi
echo "B4 dump hash OK"

echo "=== 2. Restore into disposable $DBNAME ==="
dropdb --if-exists -h "$PGHOST" "$DBNAME"
createdb -h "$PGHOST" "$DBNAME"
pg_restore --no-owner --no-acl -h "$PGHOST" --dbname="$DBNAME" "$B4_DUMP"

export DATABASE_URL="postgresql://${PGUSER}@${PGHOST}:5432/${DBNAME}"

echo "=== 3. Laboratory run ==="
cargo run -p chronosentiment_adapter --bin csp004_historical_lab -- --output "$OUT"
(
  cd "$OUT"
  shasum -a 256 laboratory.json *.md > SHA256SUMS
)

echo "=== 4. Drop disposable restore ==="
dropdb -h "$PGHOST" "$DBNAME"
echo "B4 dump untouched. Decision Engine v1.0 still unfrozen. CS-P-003 clock unchanged."
echo "Reports: $OUT/"
