#!/usr/bin/env bash
# First complete B4 historical product validation (CS-P-002).
# Replay → ledger → outcomes → performance → report.
# Does not modify the B4 dump, chrono_b4_test, chrono_b3_test, or G-GATE artifacts.
# Does not freeze Decision Engine v1.0. Engine version remains unfrozen-dev.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")" && pwd)"
cd "$ROOT"

B4_DUMP="${B4_DUMP:-r3_evidence/20260814T023457Z_B4/db/full_dump.dump}"
B4_EXPECT="${B4_EXPECT:-f74e576e8e98b24058cc913b14a567d9ff4b3eabc75662ab9a96901b102f8cd6}"
DBNAME="${HISTORICAL_B4_DBNAME:-chrono_replay_b4_historical}"
PGHOST="${PGHOST:-localhost}"
PGUSER="${PGUSER:-$(whoami)}"
OUT="${HISTORICAL_OUT:-product_validation/B4_unfrozen_dev}"

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
export B4_DUMP_SHA256="$B4_EXPECT"
export GIT_HEAD="$(git rev-parse HEAD)"
if [[ -n "$(git status --porcelain)" ]]; then
  export GIT_DIRTY=1
else
  export GIT_DIRTY=0
fi

RUN1="$(mktemp -d /tmp/csp002_b4_hist1.XXXXXX)"
RUN2="$(mktemp -d /tmp/csp002_b4_hist2.XXXXXX)"

echo "=== 3. Historical run 1 ==="
cargo run -p chronosentiment_adapter --bin csp002_b4_historical_run -- --output "$RUN1"

echo "=== 4. Historical run 2 (determinism) ==="
cargo run -p chronosentiment_adapter --bin csp002_b4_historical_run -- --output "$RUN2"

HASH1="$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["content_hash"])' "$RUN1/performance.json")"
HASH2="$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["content_hash"])' "$RUN2/performance.json")"
if [[ "$HASH1" != "$HASH2" ]]; then
  echo "STOP: performance content_hash mismatch between runs" >&2
  echo " run1 $HASH1" >&2
  echo " run2 $HASH2" >&2
  exit 2
fi
echo "Deterministic performance hash: $HASH1"

echo "=== 5. Write canonical artifact ==="
rm -rf "$OUT"
mkdir -p "$OUT"
cp "$RUN1/performance.json" "$RUN1/lineage.json" "$RUN1/provenance.json" \
  "$RUN1/HISTORICAL_PERFORMANCE_REPORT.md" "$OUT/"
(
  cd "$OUT"
  shasum -a 256 performance.json lineage.json provenance.json HISTORICAL_PERFORMANCE_REPORT.md \
    > SHA256SUMS
)

echo "=== 6. Drop disposable restore ==="
dropdb -h "$PGHOST" "$DBNAME"
rm -rf "$RUN1" "$RUN2"
echo "B4 dump untouched. Decision Engine v1.0 still unfrozen. G-GATE closed."
echo "Report: $OUT/HISTORICAL_PERFORMANCE_REPORT.md"
