#!/usr/bin/env bash
set -euo pipefail

# Temporary database name based on timestamp
DBNAME="b3_verify_$(date +%s)"

# Create the database (assumes current user has rights)
createdb $DBNAME

# Restore the dump into the temp DB
pg_restore -d $DBNAME "/Users/nikhil/ChronoSentiment_MEGA_FINAL/r3_evidence/20260813T035739Z_B3/db/full_dump.dump"

FAIL=0

# Helper to compare count
check_count() {
  local query=$1
  local expected=$2
  local name=$3
  local actual=$(psql -d $DBNAME -t -c "$query" | tr -d '[:space:]')
  if [ "$actual" != "$expected" ]; then
    echo "FAIL: $name count $actual != $expected"
    FAIL=1
  else
    echo "PASS: $name count $actual"
  fi
}

# 1. Assessments = 195
check_count "SELECT COUNT(*) FROM knowledge_assessments;" "195" "assessments"
# 2. Decisions = 195
check_count "SELECT COUNT(*) FROM knowledge_decisions;" "195" "decisions"
# 3. Strategies = 110
check_count "SELECT COUNT(*) FROM knowledge_strategies;" "110" "strategies"
# 4. Outcomes = 440
check_count "SELECT COUNT(*) FROM knowledge_outcomes;" "440" "outcomes"

# Orphan decisions (no assessment_id) – should be 0
orphan_decisions=$(psql -d $DBNAME -t -c "SELECT COUNT(*) FROM knowledge_decisions WHERE assessment_id IS NULL;" | tr -d '[:space:]')
if [ "$orphan_decisions" != "0" ]; then
  echo "FAIL: orphan decisions $orphan_decisions != 0"
  FAIL=1
else
  echo "PASS: orphan decisions $orphan_decisions"
fi

# Orphan strategies (no linked outcomes) – should be 0
orphan_strategies=$(psql -d $DBNAME -t -c "SELECT COUNT(*) FROM knowledge_strategies s LEFT JOIN knowledge_outcomes o ON o.strategy_id = s.id WHERE o.id IS NULL;" | tr -d '[:space:]')
if [ "$orphan_strategies" -eq 0 ]; then
  echo "PASS: orphan strategies $orphan_strategies"
else
  echo "FAIL: orphan strategies $orphan_strategies != 0"
  FAIL=1
fi

# 440 outcome->strategy matches (foreign key ensures, just count matches)
match_count=$(psql -d $DBNAME -t -c "SELECT COUNT(*) FROM knowledge_outcomes WHERE strategy_id IS NOT NULL;" | tr -d '[:space:]')
if [ "$match_count" != "440" ]; then
  echo "FAIL: outcome->strategy matches $match_count != 440"
  FAIL=1
else
  echo "PASS: outcome->strategy matches $match_count"
fi

# Exactly 4 outcomes per strategy
four_per=$(psql -d $DBNAME -t -c "SELECT COUNT(*) FROM (SELECT strategy_id, COUNT(*) cnt FROM knowledge_outcomes GROUP BY strategy_id HAVING COUNT(*) != 4) sub;" | tr -d '[:space:]')
if [ "$four_per" != "0" ]; then
  echo "FAIL: strategies with not exactly 4 outcomes count $four_per != 0"
  FAIL=1
else
  echo "PASS: all strategies have exactly 4 outcomes"
fi

# Horizons exactly {5D,10D,20D,60D}
valid_horizons=$(psql -d $DBNAME -t -c "SELECT COUNT(DISTINCT horizon) FROM knowledge_outcomes;" | tr -d '[:space:]')
if [ "$valid_horizons" != "4" ]; then
  echo "FAIL: horizon distinct count $valid_horizons != 4"
  FAIL=1
else
  horizons=$(psql -d $DBNAME -t -c "SELECT STRING_AGG(DISTINCT horizon, ',') FROM knowledge_outcomes;" | tr -d '[:space:]')
  expected="5D,10D,20D,60D"
  sorted_expected=$(echo $expected | tr ',' '\n' | sort | tr '\n' ',' | sed 's/,$//')
  sorted_horizons=$(echo $horizons | tr ',' '\n' | sort | tr '\n' ',' | sed 's/,$//')
  if [ "$sorted_horizons" != "$sorted_expected" ]; then
    echo "FAIL: horizons set $sorted_horizons does not match expected $sorted_expected"
    FAIL=1
  else
    echo "PASS: horizons set matches expected"
  fi
fi

# Foreign key existence is already enforced by schema; we assume it's valid if no errors.

if [ $FAIL -ne 0 ]; then
  echo "Invariant verification FAILED"
  exit 1
else
  echo "All invariants PASS"
  exit 0
fi
