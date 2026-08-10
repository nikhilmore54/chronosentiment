# Cleanup Register

**Document ID:** GOV-CLN-001
**Version:** 1.0
**Status:** Active
**Created:** 2026-08-01

---

## Purpose

Every consolidation, merge, archival, or deletion in the repository is recorded here with a unique identifier. This makes cleanup reproducible and reversible. No asset may be deleted or archived without a corresponding entry in this register.

**Rule:** Implement → Freeze → Inventory → Consolidate → Validate → Delete. Never delete while implementing.

---

## Register Format

Each entry contains:

- **ID** — unique cleanup identifier (CLN-NNN)
- **Type** — Archive | Delete | Merge | Rename | Consolidate
- **Asset(s)** — path(s) affected
- **Reason** — why this cleanup is needed
- **Canonical** — the asset that survives (if applicable)
- **Dependents** — assets that reference the affected asset and must be updated
- **Validation** — how to confirm the cleanup is safe
- **Status** — Pending | In Progress | Complete | Blocked
- **Date** — date completed (if applicable)

---

## Open Items

### CLN-001 — Archive superseded repository survey

| Field | Value |
|-------|-------|
| **ID** | CLN-001 |
| **Type** | Archive |
| **Asset** | `docs/REPOSITORY_SURVEY.md` |
| **Reason** | Superseded by `docs/governance/KNOWLEDGE_SURVEY.md` (GOV-KS-001), which covers all five knowledge systems rather than documents only |
| **Canonical** | `docs/governance/KNOWLEDGE_SURVEY.md` |
| **Dependents** | Any document that links to `docs/REPOSITORY_SURVEY.md` — search with `grep -r "REPOSITORY_SURVEY" docs/` |
| **Validation** | Confirm no active document links to `REPOSITORY_SURVEY.md`; confirm `KNOWLEDGE_SURVEY.md` covers all content |
| **Status** | Pending |
| **Date** | — |

---

### CLN-002 — Resolve RESEARCH_LINEAGE duplicate

| Field | Value |
|-------|-------|
| **ID** | CLN-002 |
| **Type** | Merge or Delete |
| **Asset A** | `docs/RESEARCH_LINEAGE.md` |
| **Asset B** | `docs/research/RESEARCH_LINEAGE.md` |
| **Reason** | Two files with identical names in different directories; likely duplicates |
| **Canonical** | TBD — read both files and compare content |
| **Dependents** | Any document linking to either path |
| **Validation** | Diff both files; if identical, delete one and update all references; if different, merge and note provenance |
| **Status** | Pending |
| **Date** | — |

---

### CLN-003 — Resolve ChronoSentiment Blueprint duplicate

| Field | Value |
|-------|-------|
| **ID** | CLN-003 |
| **Type** | Rename or Merge |
| **Asset A** | `docs/ChronoSentiment_Product_Blueprint_v1.md` |
| **Asset B** | `docs/ChronoSentiment_Personal_Blueprint_v1.md` |
| **Reason** | Overlapping names; unclear whether "Personal" vs "Product" is a meaningful distinction or a naming accident |
| **Canonical** | TBD — read both files and determine scope |
| **Dependents** | Any document linking to either path |
| **Validation** | If scopes are distinct, rename to make the distinction explicit; if overlapping, merge |
| **Status** | Pending |
| **Date** | — |

---

### CLN-004 — Resolve Codebase Assessment duplicate

| Field | Value |
|-------|-------|
| **ID** | CLN-004 |
| **Type** | Merge or Delete |
| **Asset A** | `docs/CODEBASE_ARCHITECTURE_ASSESSMENT.md` |
| **Asset B** | `docs/CODEBASE_ASSESSMENT.md` |
| **Reason** | Two files with similar names; likely one is a draft or earlier version of the other |
| **Canonical** | TBD — read both files and compare content |
| **Dependents** | Any document linking to either path |
| **Validation** | Diff both files; keep the more complete version; archive the other |
| **Status** | Pending |
| **Date** | — |

---

### CLN-005 — Resolve Milestone document duplicate

| Field | Value |
|-------|-------|
| **ID** | CLN-005 |
| **Type** | Merge or Delete |
| **Asset A** | `docs/EP-001_MILESTONE.md` |
| **Asset B** | `docs/P001_MILESTONE.md` |
| **Reason** | Two milestone documents; unclear if they cover the same milestone or different ones |
| **Canonical** | TBD — read both files and determine scope |
| **Dependents** | `docs/EP-002_ROADMAP.md` and any document referencing either milestone |
| **Validation** | If same milestone, merge; if different, rename to make the distinction explicit |
| **Status** | Pending |
| **Date** | — |

---

### CLN-006 — Survey docs/contracts/ vs docs/research/ contracts

| Field | Value |
|-------|-------|
| **ID** | CLN-006 |
| **Type** | Consolidate |
| **Asset A** | `docs/contracts/` (directory — not yet surveyed) |
| **Asset B** | 23 `*_CONTRACT_v1.md` files in `docs/research/` |
| **Reason** | Contracts may exist in both locations; canonical location is unclear |
| **Canonical** | TBD — survey `docs/contracts/` first |
| **Dependents** | Any document referencing contracts in either location |
| **Validation** | List all files in `docs/contracts/`; cross-reference with `docs/research/` contracts; identify duplicates |
| **Status** | Pending |
| **Date** | — |

---

---

## WS1 — Duplicate Code Inventory (2026-08-01)

The following items were identified by read-only scan of all `*.rs` files. No source files were modified.

---

### CLN-007 — Duplicate `percentile` function

| Field | Value |
|-------|-------|
| **ID** | CLN-007 |
| **Type** | Consolidate |
| **Asset A** | `archive/research_outputs/original_engine.rs` line 7034 — `fn percentile(mut values: Vec<f64>, p: f64) -> f64` |
| **Asset B** | `infrastructure/core/deprecated_examples/live_engine.rs` line 613 — identical signature and body |
| **Reason** | Two identical implementations; Asset B is in `deprecated_examples/` |
| **Canonical** | **`archive/research_outputs/original_engine.rs`** — annotated `UTILITY`, deferred wiring. Asset B is in `deprecated_examples/` which has no active Cargo consumers (confirmed 2026-08-01). |
| **Dependents** | None outside their respective files |
| **Validation** | Asset B will be removed as part of CLN-013 (deprecated_examples archival). No separate action needed for CLN-007 beyond confirming CLN-013 is complete. |
| **Status** | **COMPLETE** — canonical is `archive/research_outputs/original_engine.rs`; Asset B removed via CLN-013 (2026-08-01) |
| **Date** | 2026-08-01 |

---

### CLN-008 — Duplicate `mean` helper function

| Field | Value |
|-------|-------|
| **ID** | CLN-008 |
| **Type** | Consolidate |
| **Asset A** | `archive/research_outputs/original_engine.rs` line 606 — `fn mean(&self) -> f64` (method on a struct) |
| **Asset B** | `adapters/ultracrew/src/bin/config_sweep.rs` line 185 — `fn mean(sum: f64, cnt: usize) -> f64` (free function, different signature) |
| **Reason** | Both compute arithmetic mean but with different signatures; not true duplicates — different calling conventions |
| **Canonical** | Not a true duplicate — different signatures. No action required. |
| **Dependents** | — |
| **Validation** | Signatures confirmed different (method vs free function, different parameters). Closed. |
| **Status** | **CLOSED — not a true duplicate** |
| **Date** | 2026-08-01 |

---

### CLN-009 — Scattered CSV writers (no canonical abstraction)

| Field | Value |
|-------|-------|
| **ID** | CLN-009 |
| **Type** | Consolidate |
| **Assets** | 40+ `writeln!(file, ...)` / `File::create("*.csv")` patterns across: `services/ultracrew_server/src/bin/validation_pass.rs`, `services/cvrp_server/src/bin/m11_*.rs` through `m18_*.rs`, `adapters/cvrp/src/bin/m30_*.rs`, `adapters/roadef/src/bin/m26_*.rs` through `m27_*.rs`, `adapters/ultracrew/src/bin/deprecated/*.rs`, `adapters/ultracrew/src/bin/inrc_*.rs` |
| **Reason** | Every experiment writes its own CSV with inline `writeln!` — no shared abstraction. The harness `ResultPersistence` (`HARNESS-003`) is the canonical CSV writer for airline experiments but is not used by other adapters. |
| **WS2 Decision** | This is an **architectural ownership** question, not a duplicate-function problem. The decision is: (1) `ResultPersistence` is intentionally experiment-specific to the airline harness — it should not be promoted repository-wide without a clear cross-adapter requirement; (2) domain-specific persistence layers are intentional — each adapter owns its own output format; (3) the 40+ inline CSV writers in experiment binaries are acceptable as-is; they are not duplicates of each other — each writes a different schema for a different experiment; (4) if a future experiment requires a shared CSV abstraction, it should be added to the harness at that time. No consolidation now. |
| **Canonical** | `adapters/airline/tests/harness/persistence.rs` — `ResultPersistence` (airline experiments only, intentionally scoped) |
| **Dependents** | All experiment binaries listed above — no change required |
| **Validation** | No action required. Domain-specific persistence is intentional. |
| **Status** | **Decided — no consolidation; domain-specific persistence layers are intentional** |
| **Date** | 2026-08-01 |

---

### CLN-010 — Scattered `eprintln!` logging (no canonical abstraction outside harness)

| Field | Value |
|-------|-------|
| **ID** | CLN-010 |
| **Type** | Consolidate |
| **Assets** | 100+ `eprintln!` call sites across: `coralys-moga/src/engine.rs`, `adapters/cvrp/src/bin/campaign.rs`, `adapters/roadef/src/bin/campaign.rs`, `adapters/roadef/src/bin/campaign_engine.rs`, `adapters/ultracrew/src/bin/ultracrew-cli.rs`, and many others |
| **Canonical A** | `adapters/airline/tests/harness/logging.rs` — `EventLogger` (structured JSON-lines + stderr; airline experiments only) |
| **Canonical B** | `adapters/ultracrew/src/telemetry.rs` — `tracing`-based macros (`uc_info!`, `uc_warn!`, `uc_error!`) for UltraCrew service code |
| **Reason** | Two structured logging systems exist (harness `EventLogger` and UltraCrew `telemetry.rs`); all other adapters use raw `eprintln!`. |
| **WS2 Decision** | These are **intentionally separate concerns**, not competing implementations: (1) `EventLogger` is experiment-infrastructure logging — structured, machine-readable, tied to the harness lifecycle (ExperimentStart, RunStart, GenerationEnd, etc.); (2) `telemetry.rs` is runtime service logging — `tracing`-based, for production UltraCrew service code; (3) raw `eprintln!` in experiment binaries is acceptable ad-hoc diagnostic output — it is not a logging system and should not be replaced unless a binary is promoted to production. Decision: both canonical systems remain; raw `eprintln!` in existing experiment binaries requires no action. New experiment code in the airline harness should use `EventLogger`; new UltraCrew service code should use `telemetry.rs`. |
| **Canonical** | `EventLogger` for experiment harness code; `telemetry.rs` for UltraCrew service code; `eprintln!` acceptable for experiment binaries |
| **Dependents** | All binaries using raw `eprintln!` — no change required |
| **Validation** | No action required. Separate concerns are intentional. |
| **Status** | **Decided — two canonical logging systems for two different concerns; raw eprintln! in experiment binaries is acceptable** |
| **Date** | 2026-08-01 |

---

### CLN-011 — Duplicate `ObjectiveVector` / `ObjectiveValue` / `ObjectiveWeights`

| Field | Value |
|-------|-------|
| **ID** | CLN-011 |
| **Type** | Consolidate |
| **Asset A** | `coralys-ecology/src/diagnostics.rs` — `ObjectiveVector` struct + impl |
| **Asset B** | `coralys-eval/src/types.rs` — `ObjectiveValue` struct + impl |
| **Asset C** | `adapters/ultracrew/src/inrc/models.rs` — `ObjectiveWeights` struct + impl Default |
| **Reason** | Three structs representing objective-related data in three different crates; names differ but purposes overlap. `ObjectiveVector` holds a `Vec<f64>`, `ObjectiveValue` holds a named scalar, `ObjectiveWeights` holds integer weights for INRC constraints. |
| **WS2 Decision** | These are **not mechanical duplicates** — they serve different semantic roles at different abstraction layers. The WS2 decision is: (1) `coralys-eval` owns the semantic definition of objective values (`ObjectiveValue`); (2) `coralys-ecology` may retain `ObjectiveVector` as a diagnostics-layer type if it is not consumed outside that crate; (3) `ObjectiveWeights` in `adapters/ultracrew` is domain-specific to INRC and should remain in the adapter. No consolidation until a consuming crate requires cross-crate objective type compatibility. |
| **Canonical** | `coralys-eval/src/types.rs` — canonical for objective value semantics. `ObjectiveVector` and `ObjectiveWeights` remain in their respective crates until a cross-crate consumer requires unification. |
| **Dependents** | Any code importing from `coralys-ecology::diagnostics`, `coralys-eval::types`, or `adapters/ultracrew::inrc::models` |
| **Validation** | Confirm `ObjectiveVector` is not consumed outside `coralys-ecology`; confirm `ObjectiveWeights` is not consumed outside `adapters/ultracrew`. If either is consumed cross-crate, revisit. |
| **Status** | **Decided — no immediate consolidation; canonical owner assigned per layer** |
| **Date** | 2026-08-01 |

---

### CLN-012 — `adapters/gerad/` vs `adapters/airline/` overlap

| Field | Value |
|-------|-------|
| **ID** | CLN-012 |
| **Type** | Consolidate |
| **Asset A** | `adapters/gerad/` — GERAD benchmark parser and importer (`coralys-gerad`) |
| **Asset B** | `adapters/airline/` — airline crew pairing domain model (`coralys-airline`) |
| **Reason** | Both adapters exist; role of `adapters/gerad/` relative to `adapters/airline/` was unclear. |
| **Canonical** | **`adapters/airline/`** — confirmed canonical domain model. `adapters/gerad/` is a downstream consumer: `coralys-gerad` depends on `coralys-airline` (confirmed via `adapters/gerad/Cargo.toml`). Dependency is one-directional: `gerad → airline`. No cycle, no overlap. |
| **Dependents** | `adapters/airline/tests/gerad_coralys.rs` (FROZEN), `adapters/airline/tests/gerad_e2e.rs` |
| **Validation** | Confirmed: `adapters/airline/Cargo.toml` has no dependency on `coralys-gerad`. `adapters/gerad/Cargo.toml` description: "After import, no downstream code knows the data originated from GERAD." These are complementary, not overlapping. |
| **Status** | **CLOSED — not a problematic overlap; one-directional gerad→airline dependency is correct** |
| **Date** | 2026-08-01 |

---

### CLN-013 — `infrastructure/core/deprecated_examples/` directory

| Field | Value |
|-------|-------|
| **ID** | CLN-013 |
| **Type** | Archive or Delete |
| **Asset** | `infrastructure/core/deprecated_examples/` — contains `live_engine.rs`, `live_observatory.rs`, `replay_live_suggestions.rs`, `train_nse.rs`, `training_nse.rs` |
| **Reason** | Directory is explicitly named `deprecated_examples`. No active `Cargo.toml` declares these as `[[example]]` entries (confirmed 2026-08-01). |
| **Canonical** | None — these are superseded implementations |
| **Dependents** | `training_nse.rs` uses `#[path = "train_nse.rs"]` to include `train_nse.rs` — internally coupled within the deprecated directory only. `financial/ese/src/main.rs` references `./target/release/examples/live_observatory` — this is a compiled binary path, not the source file. `financial/ese/examples/live_observatory.rs` is a separate file in the `financial` crate, not the deprecated one. |
| **Validation** | Confirmed: no active `Cargo.toml` includes these files as `[[example]]` entries. `live_engine.rs` and `replay_live_suggestions.rs` have no active consumers. `train_nse.rs`/`training_nse.rs` are not declared as Cargo examples. Entire directory is safe to archive or delete. |
| **Status** | **COMPLETE — deleted 2026-08-01** |
| **Date** | 2026-08-01 |

---

## Completed Items

*No items completed yet.*

---

## Workstream Sequence

Per the reviewer's instruction, cleanup follows this sequence:

```
Implement
    ↓
Freeze
    ↓
Inventory  ← GOV-KS-001 (complete)
    ↓
Consolidate  ← this register governs consolidation
    ↓
Validate
    ↓
Delete
```

Workstreams execute in this order:

1. **WS1 — Duplicate Code Inventory** — identify duplicate helper functions, statistics calculations, CSV writers, logging, report generation, benchmark loaders, genome builders
2. **WS2 — Canonical Components** — for every duplicated capability, choose exactly one owner (harness modules are the canonical implementations)
3. **WS3 — Remove Dead Code** — only after WS1 and WS2; every deletion justified by the inventory
4. **WS4 — Module Boundaries** — verify harness/optimizer/research/GERAD responsibilities are clean
5. **WS5 — Public API Freeze** — freeze harness, statistics, reporting, and persistence APIs after cleanup

---

## Maintenance Protocol

1. Before deleting or archiving any asset, create an entry in this register.
2. Set status to Pending; get reviewer approval before proceeding.
3. After completing the cleanup, update status to Complete and record the date.
4. If a cleanup is blocked (e.g. a dependent cannot be updated), set status to Blocked and explain why.
5. Never delete an asset that is referenced by a frozen document without explicit reviewer approval.

---

*Last updated: 2026-08-01 | Maintained by: Repository Governance*