#!/usr/bin/env bash
set -euo pipefail

# ------------------------------------------------------------
# R3.1-B2 evidence capture script
# ------------------------------------------------------------
# Generates a unique disposable PostgreSQL database, applies the two
# migrations, runs the population binary, captures counts, relationship
# metrics, Yahoo provider data (as available), and a full pg_dump.
# All artefacts are stored under a timestamped immutable directory.
# ------------------------------------------------------------

# 1. Generate timestamp and DB name (UTC)
TIMESTAMP=$(date -u +"%Y%m%dT%H%M%SZ")
EVIDENCE_ROOT="${PWD}/r3_evidence/${TIMESTAMP}_B2"
mkdir -p "${EVIDENCE_ROOT}" \
  "${EVIDENCE_ROOT}/git" \
  "${EVIDENCE_ROOT}/db" \
  "${EVIDENCE_ROOT}/binary" \
  "${EVIDENCE_ROOT}/source" \
  "${EVIDENCE_ROOT}/migrations"
DB_NAME="chronosentiment_r3_b2_${TIMESTAMP}"

# Helper to log commands with timestamps
log_cmd() {
  echo "$(date -u +"%Y-%m-%dT%H:%M:%SZ") COMMAND: $*" >> "${EVIDENCE_ROOT}/commands.log"
  "$@" 2>&1 | tee -a "${EVIDENCE_ROOT}/commands.log"
}

# 2. Record Git state
log_cmd git rev-parse HEAD > "${EVIDENCE_ROOT}/git/head.txt"
log_cmd git status --porcelain > "${EVIDENCE_ROOT}/git/status.txt"
log_cmd git diff > "${EVIDENCE_ROOT}/git/diff.patch"

# 3. Create disposable DB
log_cmd createdb "${DB_NAME}"
log_cmd psql -d "${DB_NAME}" -c "SELECT current_database(), inet_server_addr(), inet_server_port();" > "${EVIDENCE_ROOT}/db/db_identity.txt"

# 5. Capture post‑migration schema
log_cmd pg_dump -s -d "${DB_NAME}" > "${EVIDENCE_ROOT}/db/schema_before_populate.sql"

# 6. Run the population binary
export DATABASE_URL="postgres://localhost/${DB_NAME}"
BIN_PATH="${PWD}/target/debug/m4_populate_knowledge_lake"
log_cmd "${BIN_PATH}" > "${EVIDENCE_ROOT}/binary/stdout.log" 2> "${EVIDENCE_ROOT}/binary/stderr.log"
echo "$?" > "${EVIDENCE_ROOT}/binary/exit_code.txt"

# 7. Yahoo provider capture (raw responses if cache files exist)
YAHOO_CACHE_DIR="${PWD}/cache/yahoo_responses"
if [ -d "${YAHOO_CACHE_DIR}" ]; then
  mkdir -p "${EVIDENCE_ROOT}/yahoo_responses"
  find "${YAHOO_CACHE_DIR}" -type f -exec cp {} "${EVIDENCE_ROOT}/yahoo_responses/" \;
  for f in "${EVIDENCE_ROOT}/yahoo_responses"/*; do
    sha=$(shasum -a 256 "$f" | awk '{print $1}')
    echo "${sha}  $(basename $f)" >> "${EVIDENCE_ROOT}/yahoo_responses/sha256.txt"
  done
else
  # fallback: extract request URLs from binary logs
  grep -E "GET|POST" "${EVIDENCE_ROOT}/binary/stdout.log" > "${EVIDENCE_ROOT}/yahoo_capture.log" || true
fi

# 8. Aggregate counts
log_cmd psql -d "${DB_NAME}" -Atc "SELECT COUNT(*) FROM knowledge_assessments;" > "${EVIDENCE_ROOT}/db/counts_assessments.txt"
log_cmd psql -d "${DB_NAME}" -Atc "SELECT COUNT(*) FROM knowledge_decisions;" > "${EVIDENCE_ROOT}/db/counts_decisions.txt"
log_cmd psql -d "${DB_NAME}" -Atc "SELECT COUNT(*) FROM knowledge_strategies;" > "${EVIDENCE_ROOT}/db/counts_strategies.txt"
log_cmd psql -d "${DB_NAME}" -Atc "SELECT COUNT(*) FROM knowledge_outcomes;" > "${EVIDENCE_ROOT}/db/counts_outcomes.txt"

# 9. Relationship checks (saved as a markdown file)
REL_MD="${EVIDENCE_ROOT}/relationship_checks.md"
{
  echo "## Assessment → Decision"
  echo "- Total assessments: $(psql -d "${DB_NAME}" -Atc "SELECT COUNT(*) FROM knowledge_assessments;")"
  echo "- Total decisions: $(psql -d "${DB_NAME}" -Atc "SELECT COUNT(*) FROM knowledge_decisions;")"
  echo "- Assessments without decisions: $(psql -d "${DB_NAME}" -Atc "SELECT COUNT(*) FROM knowledge_assessments a LEFT JOIN knowledge_decisions d ON d.assessment_id = a.id WHERE d.id IS NULL;")"
  echo "- Decisions without assessment FK: $(psql -d "${DB_NAME}" -Atc "SELECT COUNT(*) FROM knowledge_decisions d LEFT JOIN knowledge_assessments a ON d.assessment_id = a.id WHERE a.id IS NULL;")"
  echo "- Decisions per assessment distribution:"
  psql -d "${DB_NAME}" -A -F "|" -c "SELECT assessment_id, COUNT(*) FROM knowledge_decisions GROUP BY assessment_id ORDER BY assessment_id;"

  echo "\n## Decision → Strategy"
  echo "- Total strategies: $(psql -d "${DB_NAME}" -Atc "SELECT COUNT(*) FROM knowledge_strategies;")"
  echo "- Positive decisions: $(psql -d "${DB_NAME}" -Atc "SELECT COUNT(*) FROM knowledge_decisions WHERE opportunity = 'Positive';")"
  echo "- Non‑positive decisions: $(psql -d "${DB_NAME}" -Atc "SELECT COUNT(*) FROM knowledge_decisions WHERE opportunity <> 'Positive';")"
  echo "- Decisions without strategy: $(psql -d "${DB_NAME}" -Atc "SELECT COUNT(*) FROM knowledge_decisions d LEFT JOIN knowledge_strategies s ON s.decision_id = d.id WHERE s.id IS NULL;")"
  echo "- Strategies without decision FK: $(psql -d "${DB_NAME}" -Atc "SELECT COUNT(*) FROM knowledge_strategies s LEFT JOIN knowledge_decisions d ON s.decision_id = d.id WHERE d.id IS NULL;")"

  echo "\n## Strategy → Outcome"
  echo "- Total outcomes: $(psql -d "${DB_NAME}" -Atc "SELECT COUNT(*) FROM knowledge_outcomes;")"
  echo "- Outcomes per horizon:"
  psql -d "${DB_NAME}" -A -F "|" -c "SELECT horizon, COUNT(*) FROM knowledge_outcomes GROUP BY horizon ORDER BY horizon;"
  echo "- Strategies missing outcomes: $(psql -d "${DB_NAME}" -Atc "SELECT COUNT(*) FROM knowledge_strategies s LEFT JOIN knowledge_outcomes o ON o.strategy_id = s.id WHERE o.id IS NULL;")"
  echo "- Orphan outcomes (no strategy): $(psql -d "${DB_NAME}" -Atc "SELECT COUNT(*) FROM knowledge_outcomes o LEFT JOIN knowledge_strategies s ON o.strategy_id = s.id WHERE s.id IS NULL;")"
} > "${REL_MD}"

# 10. Full database dump (custom format)
log_cmd pg_dump -Fc -d "${DB_NAME}" -f "${EVIDENCE_ROOT}/db/full_dump.dump"
shasum -a 256 "${EVIDENCE_ROOT}/db/full_dump.dump" | awk '{print $1}' > "${EVIDENCE_ROOT}/db/full_dump.sha256"

# 11. Capture final schema
log_cmd pg_dump -s -d "${DB_NAME}" > "${EVIDENCE_ROOT}/db/schema_after_populate.sql"

# 12. Source file checksums (the core files used in the run)
SRC_FILES=(
  "${PWD}/adapters/chronosentiment/src/bin/m4_populate_knowledge_lake.rs"
  "${PWD}/adapters/chronosentiment/src/reasoning/decision.rs"
  "${PWD}/adapters/chronosentiment/src/reasoning/strategy.rs"
  "${PWD}/adapters/chronosentiment/src/validation/outcome.rs"
  "${PWD}/adapters/chronosentiment/migrations/20260811000000_schema.sql"
  "${PWD}/adapters/chronosentiment/migrations/20260811000001_add_assessment_fk.sql"
)
mkdir -p "${EVIDENCE_ROOT}/source"
for f in "${SRC_FILES[@]}"; do
  sha=$(shasum -a 256 "$f" | awk '{print $1}')
  echo "${sha}  $(basename $f)" >> "${EVIDENCE_ROOT}/source/checksums.sha256"
done

# 13. Binary hash (already built)
shasum -a 256 "${BIN_PATH}" | awk '{print $1}' > "${EVIDENCE_ROOT}/binary/m4_populate_knowledge_lake.sha256"

# 14. Preserve original mapping report (if it exists) and create runtime results file
if [ -f "${PWD}/r3_evidence/20260812T054359Z_B/source/mapping_report.md" ]; then
  cp "${PWD}/r3_evidence/20260812T054359Z_B/source/mapping_report.md" "${EVIDENCE_ROOT}/source/mapping_report_pre.md"
fi
cat > "${EVIDENCE_ROOT}/source/mapping_report_runtime.md" <<'EOF'
# Runtime Verification Results

## Expected vs Observed

| Artifact | Expected | Observed |
|----------|----------|----------|
| Assessments | 196 | TBD |
| Decisions   | 195 | TBD |
| Strategies  | 110 | TBD |
| Outcomes    | 440 | TBD |

## Result Classification (to be filled after run)

- PASS / DISCREPANCY / FAIL / BLOCKED / INCONCLUSIVE

EOF

# 15. Artifact trace – preserve any stdout lines that already contain per‑instrument details
# (No modification of source; just keep stdout as collected.)

# 16. Cleanup: drop the disposable DB after evidence is fully written
log_cmd dropdb "${DB_NAME}"

echo "Evidence collection complete. Directory: ${EVIDENCE_ROOT}" | tee -a "${EVIDENCE_ROOT}/commands.log"

