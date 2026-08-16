# R1 — Repository State Reconciliation Report

**Date:** 2026-08-11  
**Branch:** `governance-hardening` (up to date with `origin/governance-hardening`)  
**HEAD commit:** `9297dbc72` — "Phase B complete: tests run against PostgreSQL successfully"  
**Code changes:** NONE — all evidence from read-only Git commands

---

## R1-A — Git State

### Branch and HEAD

```
* governance-hardening    (HEAD, up to date with origin/governance-hardening)
  local-pre-filter-backup-branch
  main
  master
  phase4-simulation-amputation
  post-qfth/cert-restore-and-gulf-validation
```

### Relevant Commit History (chronological)

| Commit | Date | Description | Key files |
|--------|------|-------------|-----------|
| `50a07ea73` | 2026-07-26 | EP-1 execution phase baseline | Initial staging |
| `f0f04206c` | 2026-08-10 | Phase 1B: Replay Engine & Context | Validation platform |
| `5ac6e8c81` | 2026-08-11 10:49 | **Phase 4**: Decision/Strategy/Outcome engines | 50 files, +3190 lines |
| `b46fcba7f` | 2026-08-11 11:34 | **Phase B**: Artifact Foundation + DB test | `knowledge.rs`, `postgres_knowledge.rs`, `hash.rs` |
| `9297dbc72` | 2026-08-11 11:59 | **Phase B complete**: Tests run against PostgreSQL | Schema migration, `knowledge_tests.rs` |

### Key Finding

> [!CAUTION]
> **No commit corresponds to the legacy 196/195/110/440 lake.** The population binary (`m4_populate_knowledge_lake.rs`) has never been committed to Git. The legacy lake was produced by uncommitted code running against the legacy database.

---

## R1-B — File Classification

### Modified tracked files (uncommitted local changes)

| File | Committed in | Local changes |
|------|-------------|---------------|
| [20260811000000_schema.sql](file:///Users/nikhil/ChronoSentiment_MEGA_FINAL/adapters/chronosentiment/migrations/20260811000000_schema.sql) | `9297dbc72` | +58 lines: added `knowledge_outcomes`, `knowledge_decisions`, `knowledge_strategies` tables |
| [decision.rs](file:///Users/nikhil/ChronoSentiment_MEGA_FINAL/adapters/chronosentiment/src/reasoning/decision.rs) | `5ac6e8c81` | +20 lines: added `metadata` field, `ArtifactMetadata::mock()`, `KnowledgeArtifact` impl |
| [strategy.rs](file:///Users/nikhil/ChronoSentiment_MEGA_FINAL/adapters/chronosentiment/src/reasoning/strategy.rs) | `5ac6e8c81` | +27 lines: added `metadata` field, `KnowledgeArtifact` impl, lineage population |
| [outcome.rs](file:///Users/nikhil/ChronoSentiment_MEGA_FINAL/adapters/chronosentiment/src/validation/outcome.rs) | `5ac6e8c81` | Refactored: added `metadata`, `strategy_id`, Serialize/Deserialize, content hash, new `measure_outcome` signature |
| [postgres_knowledge.rs](file:///Users/nikhil/ChronoSentiment_MEGA_FINAL/adapters/chronosentiment/src/repository/postgres_knowledge.rs) | `b46fcba7f` | +172 lines: added `store`/`get` for `OutcomeRecord`, `Decision`, `OpportunityStrategy` |
| [knowledge.rs](file:///Users/nikhil/ChronoSentiment_MEGA_FINAL/adapters/chronosentiment/src/repository/knowledge.rs) | `b46fcba7f` | +1 line: added `Strategy` variant to `ArtifactType` |
| [lib.rs](file:///Users/nikhil/ChronoSentiment_MEGA_FINAL/adapters/chronosentiment/src/lib.rs) | `5ac6e8c81` | +1 line: `pub mod research;` |
| [historical_reasoning.rs](file:///Users/nikhil/ChronoSentiment_MEGA_FINAL/adapters/chronosentiment/src/reasoning/historical_reasoning.rs) | `5ac6e8c81` | +65 lines |
| [calibration.rs](file:///Users/nikhil/ChronoSentiment_MEGA_FINAL/adapters/chronosentiment/src/validation/calibration.rs) | `5ac6e8c81` | +2 lines |
| [m4_time_machine_demo.rs](file:///Users/nikhil/ChronoSentiment_MEGA_FINAL/adapters/chronosentiment/src/bin/m4_time_machine_demo.rs) | `5ac6e8c81` | +7 lines |
| [m4_validation_gate.rs](file:///Users/nikhil/ChronoSentiment_MEGA_FINAL/adapters/chronosentiment/src/bin/m4_validation_gate.rs) | `5ac6e8c81` | +2 lines |
| [knowledge_tests.rs](file:///Users/nikhil/ChronoSentiment_MEGA_FINAL/adapters/chronosentiment/tests/knowledge_tests.rs) | `9297dbc72` | +10 lines |

### Untracked files (never committed)

| File | Category | Role in Knowledge Lake |
|------|----------|----------------------|
| `migrations/20260811000001_add_assessment_fk.sql` | **Migration** | Adds `assessment_id NOT NULL` to `knowledge_decisions` |
| `src/bin/m4_populate_knowledge_lake.rs` | **Population** | The binary that produced the legacy lake |
| `src/bin/m6_phase_g_experiment.rs` | **Research** | Phase G experiment runner |
| `src/research/dataset.rs` | **Research** | Research dataset module |
| `src/research/experiment.rs` | **Research** | Experiment module |
| `src/research/laboratory.rs` | **Research** | Laboratory module |
| `src/research/mod.rs` | **Research** | Research module root |
| `src/research/predictive_value.rs` | **Research** | Predictive value analysis |
| `scripts/phase_c_gate.sh` | **CI/scripts** | Phase C gate script |
| `tests/knowledge_outcome_tests.rs` | **Test** | Outcome tests |
| `tests/outcome_determinism_tests.rs` | **Test** | Determinism tests |
| `tests/phase_g_predictive_value_tests.rs` | **Test** | Phase G tests |
| `tests/reproducibility_tests.rs` | **Test** | Reproducibility tests |
| `tests/research_dataset_tests.rs` | **Test** | Dataset tests |
| `tests/research_laboratory_tests.rs` | **Test** | Laboratory tests |
| `tests/fixtures/phase_c_replay/` | **Test fixtures** | Replay test data |

---

## R1-B (cont.) — Migration State

| Migration | Git tracked? | Legacy DB | Fresh DB | Application compatible? |
|-----------|:-----------:|:---------:|:--------:|:----------------------:|
| `20260811000000_schema.sql` (committed version) | ✓ | ✓ (only this) | ✓ | ✓ (Assessment only) |
| `20260811000000_schema.sql` (local modifications) | **Modified** | ✗ | ✓ | ✓ |
| `20260811000001_add_assessment_fk.sql` | **Untracked** | ✗ | ✓ | **✗** |

> [!WARNING]
> The committed migration creates only `instruments` and `knowledge_assessments`. The `knowledge_decisions`, `knowledge_strategies`, and `knowledge_outcomes` tables exist only in the **uncommitted local modification** of the base migration.

---

## R1-C — Test State Reconciliation

### `week2_tests.rs` — Stale since commit

The tests reference `AssessmentValue::Bullish` and treat `AssessmentProfile` as a `Vec`. However, `AssessmentProfile` was already a struct (containing `assessments: Vec<DomainAssessment>`) in commit `5ac6e8c81` (Phase 4), which is **earlier** than the test file's introduction.

The committed `week2_tests.rs` (from `5ac6e8c81`) was **already incompatible with its own commit's API**. The `assess()` method returned `AssessmentProfile` but the tests call `.len()` and `.iter()` on it directly.

### Other tests

| Test file | Git status | Interacts with PostgreSQL? |
|-----------|-----------|---------------------------|
| `week1_tests.rs` | Committed | No (in-memory metrics only) |
| `week2_tests.rs` | Committed | No (in-memory, but stale/broken) |
| `knowledge_tests.rs` | Committed (modified) | Yes (PostgreSQL, Assessment only) |
| `knowledge_outcome_tests.rs` | **Untracked** | Yes (PostgreSQL) |
| `outcome_determinism_tests.rs` | **Untracked** | Unknown |
| `phase_g_predictive_value_tests.rs` | **Untracked** | Yes (PostgreSQL) |
| `reproducibility_tests.rs` | **Untracked** | Unknown |
| `research_dataset_tests.rs` | **Untracked** | Unknown |
| `research_laboratory_tests.rs` | **Untracked** | Unknown |

---

## R1 Summary — Can We Identify a Coherent Historical State?

### The Last Committed Coherent State

**Commit `9297dbc72`** ("Phase B complete") is the last committed state. It contains:

- `instruments` table ✓
- `knowledge_assessments` table ✓
- `AssessmentProfile` struct ✓
- `PostgresKnowledgeRepository::store(&AssessmentProfile)` ✓
- `knowledge_tests.rs` testing Assessment persistence ✓

It does **NOT** contain:

- ✗ `knowledge_decisions` table
- ✗ `knowledge_strategies` table
- ✗ `knowledge_outcomes` table
- ✗ `Decision` persistence
- ✗ `Strategy` persistence
- ✗ `Outcome` persistence
- ✗ Population binary
- ✗ Phase G experiment
- ✗ Research module

### Historical State That Produced the Legacy Lake

**Does not exist as a Git commit.** The legacy lake was produced by:

```
Committed state (9297dbc72)
    +
Uncommitted local modifications to 12 tracked files
    +
19 untracked files (including the population binary itself)
    -
Migration 20260811000001 (which was added AFTER the lake was populated)
```

This means the exact executable state that produced the 196/195/110/440 lake is:

```
Current working tree
    MINUS
    20260811000001_add_assessment_fk.sql
```

But this state was **never captured as a reproducible checkpoint**.

---

## R1 Verdict

| Check | Result |
|-------|--------|
| **R1-A Git state** | Working tree has extensive uncommitted changes |
| **R1-B Migration consistency** | Divergent — untracked migration incompatible with code |
| **R1-C Test reconciliation** | Tests are stale; committed tests already incompatible with their own commit |
| **Coherent historical commit** | **DOES NOT EXIST** |
| **Legacy lake reproducibility** | **Cannot be reproduced from any committed state** |

> [!CAUTION]
> **The repository has never had a committed state that could produce the Knowledge Lake.** The entire Decision/Strategy/Outcome pipeline, including the population binary, persistence layer, schema tables, and research module, exists exclusively as uncommitted local modifications. There is no tag, branch, or commit corresponding to the legacy 196/195/110/440 lake.

---

## Implications for Phase R

1. **Before any corrective work**, the current working tree should be committed as a **known baseline** (possibly on a dedicated branch like `phase-r-baseline`). This preserves the exact state that produced the legacy lake (minus migration 000001).

2. **Migration `20260811000001_add_assessment_fk.sql` should be quarantined** — it was a premature schema change that broke the pipeline. It should not be applied until the code supports it.

3. **R2 (Historical Population Reconstruction) requires** removing or bypassing migration 000001 to run the population binary against a fresh database using the current (uncommitted) code.

4. The test failures in `week2_tests.rs` predate the Knowledge Lake work and are orthogonal to Phase R, but they prevent the test suite from compiling.

> **No source files, migrations, tests, schemas, or CI configurations were modified during R1.**
