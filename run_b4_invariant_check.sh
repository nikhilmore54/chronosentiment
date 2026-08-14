#!/usr/bin/env bash
# B4 invariant checks. Does not read or write B3.
set -euo pipefail

DBNAME="${1:?usage: run_b4_invariant_check.sh <dbname>}"
FAIL=0

check_count() {
  local query=$1
  local expected=$2
  local name=$3
  local actual
  actual=$(psql -h "${PGHOST:-localhost}" -d "$DBNAME" -t -A -c "$query")
  if [ "$actual" != "$expected" ]; then
    echo "FAIL: $name count $actual != $expected"
    FAIL=1
  else
    echo "PASS: $name count $actual"
  fi
}

check_count "SELECT COUNT(*) FROM knowledge_assessments;" "195" "assessments"
check_count "SELECT COUNT(*) FROM knowledge_decisions;" "195" "decisions"
check_count "SELECT COUNT(*) FROM knowledge_strategies;" "110" "strategies"
check_count "SELECT COUNT(*) FROM knowledge_outcomes;" "440" "outcomes"

orphan_decisions=$(psql -h "${PGHOST:-localhost}" -d "$DBNAME" -t -A -c "SELECT COUNT(*) FROM knowledge_decisions WHERE assessment_id IS NULL;")
if [ "$orphan_decisions" != "0" ]; then
  echo "FAIL: orphan decisions $orphan_decisions != 0"
  FAIL=1
else
  echo "PASS: orphan decisions 0"
fi

orphan_strategies=$(psql -h "${PGHOST:-localhost}" -d "$DBNAME" -t -A -c "SELECT COUNT(*) FROM knowledge_strategies s LEFT JOIN knowledge_outcomes o ON o.strategy_id = s.id WHERE o.id IS NULL;")
if [ "$orphan_strategies" != "0" ]; then
  echo "FAIL: orphan strategies $orphan_strategies != 0"
  FAIL=1
else
  echo "PASS: orphan strategies 0"
fi

match_count=$(psql -h "${PGHOST:-localhost}" -d "$DBNAME" -t -A -c "SELECT COUNT(*) FROM knowledge_outcomes o JOIN knowledge_strategies s ON o.strategy_id = s.id;")
if [ "$match_count" != "440" ]; then
  echo "FAIL: strategy→outcome matches $match_count != 440"
  FAIL=1
else
  echo "PASS: strategy→outcome matches 440"
fi

four_per=$(psql -h "${PGHOST:-localhost}" -d "$DBNAME" -t -A -c "SELECT COUNT(*) FROM (SELECT strategy_id FROM knowledge_outcomes GROUP BY strategy_id HAVING COUNT(*) <> 4) sub;")
if [ "$four_per" != "0" ]; then
  echo "FAIL: strategies without exactly 4 outcomes: $four_per"
  FAIL=1
else
  echo "PASS: all strategies have exactly 4 outcomes"
fi

horizons=$(psql -h "${PGHOST:-localhost}" -d "$DBNAME" -t -A -c "SELECT STRING_AGG(horizon, ',' ORDER BY horizon) FROM (SELECT DISTINCT horizon FROM knowledge_outcomes) h;")
if [ "$horizons" != "10D,20D,5D,60D" ]; then
  echo "FAIL: horizons $horizons"
  FAIL=1
else
  echo "PASS: horizons 5D/10D/20D/60D"
fi

# The invariant E-GATE v2 omitted.
assess_after=$(psql -h "${PGHOST:-localhost}" -d "$DBNAME" -t -A -c "
SELECT COUNT(*)
FROM knowledge_decisions d
JOIN knowledge_assessments a ON a.id = d.assessment_id
WHERE a.evaluation_timestamp > d.evaluation_timestamp;
")
if [ "$assess_after" != "0" ]; then
  echo "FAIL: assessment.evaluation_timestamp > decision.evaluation_timestamp on $assess_after pairs"
  FAIL=1
else
  echo "PASS: assessment.evaluation_timestamp <= decision.evaluation_timestamp (all pairs)"
fi

outcome_before_decision=$(psql -h "${PGHOST:-localhost}" -d "$DBNAME" -t -A -c "
SELECT COUNT(*)
FROM knowledge_outcomes o
JOIN knowledge_strategies s ON o.strategy_id = s.id
JOIN knowledge_decisions d ON s.decision_id = d.id
WHERE o.evaluation_timestamp < d.evaluation_timestamp;
")
if [ "$outcome_before_decision" != "0" ]; then
  echo "FAIL: outcome.evaluation_timestamp < decision.evaluation_timestamp on $outcome_before_decision rows"
  FAIL=1
else
  echo "PASS: outcome timestamps not before decision timestamps"
fi

expiry_ok=$(psql -h "${PGHOST:-localhost}" -d "$DBNAME" -t -A -c "
SELECT COUNT(*) FROM knowledge_outcomes
WHERE horizon_expiry_timestamp <= evaluation_timestamp;
")
if [ "$expiry_ok" != "0" ]; then
  echo "FAIL: horizon_expiry_timestamp <= evaluation_timestamp on $expiry_ok outcomes"
  FAIL=1
else
  echo "PASS: outcome evaluation_timestamp < horizon_expiry_timestamp"
fi

if [ "$FAIL" -ne 0 ]; then
  echo "B4 invariant verification FAILED"
  exit 1
fi
echo "All B4 invariants PASS"
exit 0
