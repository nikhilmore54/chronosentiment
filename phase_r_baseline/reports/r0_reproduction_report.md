# R0 — Test/Population Reproduction Report

**Date:** 2026-08-11  
**Fresh database:** `chronosentiment_phase_r`  
**Legacy database:** `postgres` (196/195/110/440 lake)  
**Code changes:** NONE — all commands run against unmodified source  

---

## 1. Database Setup

### Command
```bash
psql postgres://nikhil@localhost:5432/postgres -c "CREATE DATABASE chronosentiment_phase_r;"
```

### Result
```
CREATE DATABASE
```

### Verification
```sql
SELECT tablename FROM pg_tables WHERE schemaname = 'public';
-- (0 rows) — confirmed empty
```

---

## 2. Test Suite Execution

### Command
```bash
DATABASE_URL=postgres://nikhil@localhost:5432/chronosentiment_phase_r \
  cargo test 2>&1
```

### Result: **COMPILATION FAILURE**

The test suite **did not compile**. The blocking errors are in [week2_tests.rs](file:///Users/nikhil/ChronoSentiment_MEGA_FINAL/adapters/chronosentiment/tests/week2_tests.rs):

| Error | Location | Cause |
|-------|----------|-------|
| `no method named 'len' found for struct 'AssessmentProfile'` | L21 | Test treats `AssessmentProfile` as a `Vec` — stale API |
| `no method named 'iter' found for struct 'AssessmentProfile'` | L24, L28 | Same — expects iterable, but `AssessmentProfile` is now a struct with `.assessments` field |
| `no method named 'len' found for struct 'EvidenceSet'` | L46 | Test treats `EvidenceSet` as a `Vec` |
| `cannot index into a value of type 'EvidenceSet'` | L47–49 | Same — `EvidenceSet` is a struct, not indexable |

**Conclusion:** The test suite is stale. It references an older API where `AssessmentEngine.assess()` returned a `Vec<DomainAssessment>` rather than the current `AssessmentProfile` struct. These tests have not been updated since the struct refactoring.

> [!WARNING]
> **No tests ran.** We cannot determine from this run which tests populate the Knowledge Lake and which operate in memory only.

---

## 3. Population Binary Execution

### Command
```bash
DATABASE_URL=postgres://nikhil@localhost:5432/chronosentiment_phase_r \
  cargo run --bin m4_populate_knowledge_lake 2>&1
```

### Result: **RUNTIME FAILURE**

The binary compiled successfully (with 24 warnings) but **failed at runtime** on the first `INSERT INTO knowledge_decisions`:

```
Error: Database(PgDatabaseError {
    severity: Error,
    code: "23502",
    message: "null value in column \"assessment_id\" of relation \"knowledge_decisions\"
              violates not-null constraint",
    table: Some("knowledge_decisions"),
    column: Some("assessment_id"),
})
```

### Root Cause

The fresh database ran **both** migrations:

```sql
-- Migration 1: 20260811000000_schema.sql (base schema)
-- Migration 2: 20260811000001_add_assessment_fk.sql (adds assessment_id NOT NULL)
```

```sql
-- From migration 2:
ALTER TABLE knowledge_decisions
    ADD COLUMN assessment_id UUID NOT NULL;

ALTER TABLE knowledge_decisions
    ADD CONSTRAINT fk_decision_assessment
    FOREIGN KEY (assessment_id)
    REFERENCES knowledge_assessments(id)
    ON DELETE RESTRICT;
```

But the `store()` implementation for `Decision` in [postgres_knowledge.rs L155-162](file:///Users/nikhil/ChronoSentiment_MEGA_FINAL/adapters/chronosentiment/src/repository/postgres_knowledge.rs#L155-L162) only INSERTs 6 columns:

```sql
INSERT INTO knowledge_decisions (
    id, instrument_id, evaluation_timestamp,
    opportunity, metadata_json, decision_json
)
VALUES ($1, $2, $3, $4, $5, $6)
```

It does **not** include `assessment_id`. The column is `NOT NULL` with no default, so the INSERT fails.

### Why the Legacy Lake Worked

The legacy database applied **only migration 1**:

| Database | Migrations applied | `assessment_id` column |
|----------|-------------------|----------------------|
| Legacy (`postgres`) | `20260811000000_schema` only | **Does not exist** |
| Phase R (`chronosentiment_phase_r`) | Both migrations | **Exists, NOT NULL** |

```sql
-- Legacy DB migration history:
SELECT version, description FROM _sqlx_migrations ORDER BY version;
    version     | description
----------------+-------------
 20260811000000 | schema
(1 row)

-- Phase R DB migration history:
SELECT version, description FROM _sqlx_migrations ORDER BY version;
    version     |    description
----------------+-------------------
 20260811000000 | schema
 20260811000001 | add assessment fk
(2 rows)
```

The legacy lake was populated **before** migration 2 was created. Migration 2 was added to the `migrations/` directory by a previous agent session but:
- Was **never applied** to the legacy database
- Was **never committed** to git (`git log` returns empty)
- The corresponding **code changes were never made** (no `assessment_id` in the `Decision` struct or `store()` SQL)

---

## 4. Phase R Database State After Failure

```sql
SELECT 'assessments' AS tbl, COUNT(*) FROM knowledge_assessments
UNION ALL SELECT 'decisions', COUNT(*) FROM knowledge_decisions
UNION ALL SELECT 'strategies', COUNT(*) FROM knowledge_strategies
UNION ALL SELECT 'outcomes', COUNT(*) FROM knowledge_outcomes
UNION ALL SELECT 'instruments', COUNT(*) FROM instruments;
```

```
     tbl     | count
-------------+-------
 assessments |     1
 decisions   |     0
 strategies  |     0
 outcomes    |     0
 instruments |     5
```

The pipeline stored 5 instruments and 1 assessment before failing on the first decision INSERT.

---

## 5. Legacy vs Phase R Schema Comparison

```
Legacy knowledge_decisions:
  id                   | uuid       | not null
  instrument_id        | uuid       |
  evaluation_timestamp | timestamptz| not null
  opportunity          | varchar(50)| not null
  metadata_json        | jsonb      | not null
  decision_json        | jsonb      | not null
  recorded_at          | timestamptz| not null | default now()

Phase R knowledge_decisions:
  id                   | uuid       | not null
  instrument_id        | uuid       |
  evaluation_timestamp | timestamptz| not null
  opportunity          | varchar(50)| not null
  metadata_json        | jsonb      | not null
  decision_json        | jsonb      | not null
  recorded_at          | timestamptz| not null | default now()
  assessment_id        | uuid       | not null          ← ADDED BY MIGRATION 2
```

---

## 6. Summary of R0 Findings

| Check | Result | Details |
|-------|--------|---------|
| **Test suite compilation** | **FAIL** | `week2_tests.rs` references stale API (pre-refactoring) |
| **Test suite execution** | **BLOCKED** | Cannot run due to compilation failure |
| **Population binary compilation** | **PASS** | Compiles with 24 warnings |
| **Population binary execution** | **FAIL** | `assessment_id NOT NULL` constraint violation on first decision INSERT |
| **Fresh DB migration** | **DIVERGES** | Applies migration 2 (`add_assessment_fk`) which legacy DB never received |
| **Schema/code consistency** | **FAIL** | Migration 2 requires `assessment_id` but code doesn't supply it |
| **Git tracking of migration 2** | **UNTRACKED** | File exists on disk but has never been committed |
| **Legacy lake reproducibility** | **IMPOSSIBLE** | The current codebase + current migrations cannot reproduce the legacy lake |

---

## 7. Implications for Phase R

> [!CAUTION]
> **The current codebase cannot populate any database.** The schema/code divergence caused by the untracked migration makes the population binary non-functional against any fresh database.

Before proceeding to R1, the following must be resolved (in order):

1. **Decide the fate of migration `20260811000001_add_assessment_fk.sql`:**
   - **Option A:** Remove it (revert to legacy schema) so the existing code can populate a fresh database, then add it back properly after the code is corrected.
   - **Option B:** Keep it and update the code to populate `assessment_id` — but this means the code changes happen before we can reproduce the baseline.

2. **Fix or quarantine `week2_tests.rs`** so the test suite compiles and can be used for verification.

Either way, **the legacy 196/195/110/440 lake was produced by a different effective codebase** (same source but without migration 2). The current codebase, run from a clean state, produces a different outcome (failure).

---

## 8. Commands Executed (Exact)

```bash
# 1. Create fresh database
psql postgres://nikhil@localhost:5432/postgres \
  -c "CREATE DATABASE chronosentiment_phase_r;"

# 2. Verify empty
psql postgres://nikhil@localhost:5432/chronosentiment_phase_r \
  -c "SELECT tablename FROM pg_tables WHERE schemaname = 'public';"

# 3. Run test suite
DATABASE_URL=postgres://nikhil@localhost:5432/chronosentiment_phase_r \
  cargo test 2>&1

# 4. Run population binary
DATABASE_URL=postgres://nikhil@localhost:5432/chronosentiment_phase_r \
  cargo run --bin m4_populate_knowledge_lake 2>&1

# 5. Inspect Phase R DB state
psql postgres://nikhil@localhost:5432/chronosentiment_phase_r \
  -c "SELECT ... FROM knowledge_*;"

# 6. Compare schemas
psql postgres://nikhil@localhost:5432/postgres -c "\\d knowledge_decisions"
psql postgres://nikhil@localhost:5432/chronosentiment_phase_r -c "\\d knowledge_decisions"

# 7. Compare migration histories
psql postgres://nikhil@localhost:5432/postgres \
  -c "SELECT version, description FROM _sqlx_migrations;"
psql postgres://nikhil@localhost:5432/chronosentiment_phase_r \
  -c "SELECT version, description FROM _sqlx_migrations;"

# 8. Check git tracking
git log --all --oneline -- \
  adapters/chronosentiment/migrations/20260811000001_add_assessment_fk.sql
# Result: empty — file is untracked
```

> **No source files, migrations, schemas, tests, or CI configurations were modified during R0.**
