#!/usr/bin/env bash
# Restore certified B4 into a disposable DB and run replay ledger + Outcome Engine v0.1
# + Performance Engine v0.1 + Replay Adapter SQL integration tests.
# Does not modify the B4 dump, chrono_b4_test, chrono_b3_test, or G-GATE artifacts.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")" && pwd)"
cd "$ROOT"

B4_DUMP="${B4_DUMP:-r3_evidence/20260814T023457Z_B4/db/full_dump.dump}"
B4_EXPECT="${B4_EXPECT:-f74e576e8e98b24058cc913b14a567d9ff4b3eabc75662ab9a96901b102f8cd6}"
DBNAME="${REPLAY_B4_DBNAME:-chrono_replay_b4_validate}"
PGHOST="${PGHOST:-localhost}"
PGUSER="${PGUSER:-$(whoami)}"

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
export REPLAY_REQUIRE_B4=1

echo "=== 3. Schedule-driven DecisionLedger (clean B4 restore) ==="
cargo test -p chronosentiment_adapter --test replay_backtest_sqlx_tests -- --nocapture

echo "=== 4. Performance Engine v0.1 on ledger + outcomes (clean B4 restore) ==="
cargo test -p chronosentiment_adapter --test performance_engine_sqlx_tests -- --nocapture

echo "=== 5. Outcome Engine v0.1 on ledger + B4 ==="
cargo test -p chronosentiment_adapter --test outcome_engine_sqlx_tests -- --nocapture

echo "=== 6. Replay Adapter future-exclusion test ==="
cargo test -p chronosentiment_adapter --test replay_adapter_sqlx_tests -- --nocapture

echo "=== 7. Drop disposable restore ==="
dropdb -h "$PGHOST" "$DBNAME"
echo "B4 dump untouched. Decision Engine v1.0 still unfrozen. G-GATE closed."
