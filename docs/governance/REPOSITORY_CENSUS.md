# Repository Census

**Document ID:** GOV-RR1-001
**Version:** 1.0
**Status:** Active
**Created:** 2026-08-01
**Programme:** Repository Rationalization Programme (RRP v1.0)

---

## Purpose

This document is the source of truth for the Repository Rationalization Programme. It records every Rust source file by crate, compile target, and purpose. It is the input to RR2 (Reachability), RR3 (Version Analysis), and all subsequent rationalization workstreams.

**Rule:** No file may be archived or deleted under RRP without a corresponding entry in this census confirming its classification.

---

## Baseline Metrics (2026-08-01)

| Metric | Value |
|--------|-------|
| Total Rust source files | 477 |
| Total LOC (confirmed) | 114,544 |
| Crates / top-level directories | 23 |
| Files with explicit `deprecated/` path | 7 (ultracrew/bin/deprecated) |
| Files in `original_engine.rs` (monolith) | 1 (~13,000+ LOC) |

---

## Per-Crate Inventory

### 1. `adapters/airline` — `coralys-airline` (50 files, 12,760 LOC)

**Role:** Canonical airline crew pairing domain model. Layer 1 of the scheduling stack.

**Compile targets:**

| Path | Target | Purpose |
|------|--------|---------|
| `src/lib.rs` | lib | Crate root |
| `src/domain/*.rs` (8 files) | lib | Domain types: flight, duty, crew, pairing, rotation, roster, cost, credit |
| `src/legality/*.rs` (8 files) | lib | Legality checks: FDP, rest, duty time, coverage, qualification |
| `src/optimization/*.rs` (4 files) | lib | Objective functions, metrics, cost |
| `src/optimization/neighborhood/*.rs` (3 files) | lib | Relocate, swap operators |
| `src/optimization/search/*.rs` (3 files) | lib | Greedy, local search |
| `src/planner/*.rs` (4 files) | lib | Incremental planner, what-if, summary |
| `src/resilience/*.rs` (4 files) | lib | Disruption, reserve, robustness |
| `tests/gerad_coralys.rs` | test | **FROZEN** — GERAD Coralys v1.0 baseline (2026-08-01) |
| `tests/gerad_e2e.rs` | test | GERAD end-to-end integration test |
| `tests/harness/*.rs` (6 files) | test | **FROZEN** — Experiment harness API (2026-08-01) |

**Classification:** KEEP (canonical domain model + frozen research baseline)

---

### 2. `adapters/gerad` — `coralys-gerad` (8 files, 1,462 LOC)

**Role:** GERAD G-2014-22 benchmark parser. Translates benchmark format into `coralys-airline` domain model. Downstream consumer of `coralys-airline`.

**Compile targets:**

| Path | Target | Purpose |
|------|--------|---------|
| `src/lib.rs` | lib | Crate root |
| `src/parser.rs` | lib | GERAD file format parser |
| `src/importer.rs` | lib | Domain model importer |
| `src/*.rs` (remaining) | lib | Supporting types |

**Classification:** KEEP (active benchmark adapter, one-directional dependency confirmed)

---

### 3. `adapters/ultracrew` — `coralys-ultracrew` (99 files, 19,638 LOC)

**Role:** UltraCrew INRC scheduling adapter. Largest crate by file count. Contains active binaries, deprecated binaries, and research experiments.

**Compile targets — library:**

| Path | Target | Purpose |
|------|--------|---------|
| `src/lib.rs` | lib | Crate root |
| `src/inrc/*.rs` | lib | INRC domain model |
| `src/telemetry.rs` | lib | **CANONICAL** — tracing-based logging for service code |
| `src/strict_validator.rs` | lib | Constraint validation |
| `src/decision_intelligence.rs` | lib | Cycle review report generation |

**Compile targets — active binaries (`src/bin/`):**

| File | Purpose | Status |
|------|---------|--------|
| `ultracrew-cli.rs` | Production CLI entry point | KEEP |
| `config_sweep.rs` | Parameter sweep experiment | Research |
| `inrc_ecology_ablation_matrix.rs` | Ecology ablation experiment | Research |
| `inrc_ecology_cost_curve.rs` | Cost curve experiment | Research |
| `inrc_ecology_memory_depth.rs` | Memory depth experiment | Research |
| `inrc_ecology_multi_week_ablation.rs` | Multi-week ablation | Research |
| `inrc_ecology_response_curve.rs` | Response curve experiment | Research |
| `inrc_m22_ancestry.rs` | M22 ancestry analysis | Research |
| `inrc_m22_benchmark.rs` | M22 benchmark | Research |
| `inrc_natural_history_pilot.rs` | Natural history pilot | Research |
| `m23a_synthetic.rs` | M23a synthetic experiment | Research |
| `m30_0b_passive_telemetry.rs` | M30 passive telemetry | Research |
| `m30_0d_active_pilot.rs` | M30 active pilot | Research |
| `m31_2a_engagement_audit.rs` | M31 engagement audit | Research |
| `m31_benchmarks.rs` | M31 benchmarks | Research |
| `story1.rs` | Story 1 narrative experiment | Research |

**Compile targets — deprecated binaries (`src/bin/deprecated/`):**

| File | Purpose | Status |
|------|---------|--------|
| `inrc_ecology_ablation.rs` | Superseded by `inrc_ecology_ablation_matrix.rs` | **DEPRECATED** |
| `inrc_ecology_history_test.rs` | Superseded ecology history test | **DEPRECATED** |
| `inrc_ecology_horizon_test.rs` | Superseded ecology horizon test | **DEPRECATED** |
| `inrc_ecology_mechanism_audit.rs` | Superseded mechanism audit | **DEPRECATED** |
| `inrc_ecology_multi_week_ablation.rs` | Duplicate of active version | **DEPRECATED** |
| `ultracrew_atlas.rs` | Superseded atlas experiment | **DEPRECATED** |
| `ultracrew_repair_atlas.rs` | Superseded repair atlas | **DEPRECATED** |

**Classification:** KEEP (lib + active binaries); DEPRECATED directory is RR3/RR7 candidate

---

### 4. `adapters/cvrp` — `coralys-cvrp` (25 files, 9,662 LOC)

**Role:** CVRP (Capacitated Vehicle Routing Problem) adapter. Contains campaign runner, BDD tests, and research experiment binaries.

**Compile targets — active binaries:**

| File | Purpose | Status |
|------|---------|--------|
| `campaign.rs` | CVRP campaign runner | KEEP |
| `bdd_baseline.rs` | BDD baseline | KEEP |
| `bdd_benchmark.rs` | BDD benchmark | KEEP |
| `bdd_campaign.rs` | BDD campaign | KEEP |
| `bdd_campaign_compare.rs` | BDD campaign comparison | KEEP |
| `bdd_check_negative.rs` | BDD negative check | KEEP |
| `bdd_p55_multi.rs` | BDD P55 multi | KEEP |
| `bdd_telemetry.rs` | BDD telemetry | KEEP |
| `bdd_validation.rs` | BDD validation | KEEP |
| `compare.rs` | Result comparison | KEEP |
| `cvrp_sanity.rs` | Sanity check | KEEP |
| `cvrplib_registry.rs` | CVRPLIB instance registry | KEEP |
| `search_config.rs` | Search configuration | KEEP |
| `m30_2_active_pilot.rs` | M30.2 active pilot | Research |
| `m30_2a_1_ecology_audit.rs` | M30.2a ecology audit | Research |
| `m30_2a_2_shadow_advisory.rs` | M30.2a shadow advisory | Research |

**Classification:** KEEP (active campaign + BDD infrastructure); research binaries are RR3 candidates

---

### 5. `adapters/roadef` — `coralys-roadef` (27 files, 7,128 LOC)

**Role:** ROADEF 2026 network optimization adapter. Contains campaign runners and research experiment binaries (m25–m27 series).

**Compile targets — active:**

| File | Purpose | Status |
|------|---------|--------|
| `campaign.rs` | ROADEF campaign runner | KEEP |
| `campaign_engine.rs` | Alternative campaign engine | Research — version of campaign.rs |
| `e001_dual_path.rs` | E-001 dual-path validation | KEEP |
| `eval_profiler.rs` | Evaluator performance profiler | Research |
| `tiny_solver.rs` | Minimal solver for testing | Research |
| `m25_benchmark.rs` | M25 benchmark | Research |
| `m25_8_bridge.rs` | M25.8 bridge experiment | Research |
| `m25_8b_ecology.rs` | M25.8b ecology variant | Research — version of m25_8_bridge.rs |
| `m25_final.rs` | M25 final experiment | Research |
| `m26_1_observation_audit.rs` | M26.1 observation audit | Research |
| `m26_1c_discriminative_audit.rs` | M26.1c discriminative audit | Research |
| `m26_1d_failure_density.rs` | M26.1d failure density | Research |
| `m26_1e_survival_curves.rs` | M26.1e survival curves | Research |
| `m26_3_passive_learner.rs` | M26.3 passive learner | Research |
| `m26_4a_shadow_advisory.rs` | M26.4a shadow advisory | Research |
| `m26_4b_active_pilot.rs` | M26.4b active pilot | Research |
| `m27_1_passive_operator_telemetry.rs` | M27.1 operator telemetry | Research |

**Note:** `campaign.rs` and `campaign_engine.rs` are version candidates (RR3). `m25_8_bridge.rs` and `m25_8b_ecology.rs` are version candidates.

**Classification:** KEEP (campaign + validation); research binaries are RR3 candidates

---

### 6. `adapters/chronosentiment` — `coralys-chronosentiment` (6 files, 1,597 LOC)

**Role:** ChronoSentiment domain adapter (evidence, hypothesis, learning, timeline, workspace).

**Classification:** KEEP (active domain adapter)

---

### 7. `coralys-core` (11 files, 437 LOC)

**Role:** Core Coralys types: decision lineage, decision proposal, evaluation result, matching result, state reference, violation, analysis, memory, telemetry.

**Classification:** KEEP (core platform types)

---

### 8. `coralys-ecology` (6 files, 1,553 LOC)

**Role:** Ecology system: diagnostics, models, progress, state, traits.

**Classification:** KEEP (active platform component)

---

### 9. `coralys-eval` (5 files, 956 LOC)

**Role:** Evaluation types. **Canonical owner of objective value semantics** (per CLN-011 decision).

**Classification:** KEEP (canonical evaluation types)

---

### 10. `coralys-moga` (33 files, 4,500 LOC)

**Role:** Multi-objective genetic algorithm engine. Core optimization platform.

**Classification:** KEEP (active optimization engine)

---

### 11. Stub crates (7 crates, ~830 LOC total)

| Crate | Files | LOC | Status |
|-------|-------|-----|--------|
| `coralys-infrastructure` | 1 | 14 | Stub — single `lib.rs` |
| `coralys-matching` | 1 | 96 | Stub |
| `coralys-planning` | 1 | 103 | Stub |
| `coralys-policy` | 1 | 136 | Stub |
| `coralys-recommendation` | 3 | 293 | Stub |
| `coralys-simulation` | 2 | 91 | Stub |
| `coralys-v2` | 1 | 96 | Stub |
| `coralys-decision` | 2 | 96 | Stub |

**Classification:** RR2/RR6 candidates — stub crates with minimal content may exist only to satisfy dependency declarations. Verify whether any active crate depends on them.

---

### 12. `infrastructure/core` — `chronosentiment_core` (23 files, 3,472 LOC)

**Role:** Core infrastructure: capture daemon, historical importer, Yahoo importer, and supporting modules.

**Note:** `deprecated_examples/` directory deleted (CLN-013, 2026-08-01). Remaining files are active infrastructure.

**Classification:** KEEP (active infrastructure)

---

### 13. `services/ultracrew_server` (21 files, 7,981 LOC)

**Role:** UltraCrew HTTP server. Contains main server, persistence, and validation/benchmark binaries.

**Compile targets — binaries:**

| File | Purpose | Status |
|------|---------|--------|
| `acceptance_test.rs` | Acceptance benchmark | KEEP |
| `benchmark.rs` | Performance benchmark | KEEP |
| `cs_governance_validation.rs` | Governance validation | KEEP |
| `ecology_validation.rs` | Ecology validation | KEEP |
| `inrc_archive_forensics.rs` | INRC archive forensics | Research |
| `m8g_cs_validation.rs` | M8g CS validation | Research |
| `m8g_ultracrew_validation.rs` | M8g UltraCrew validation | Research |
| `m9a_search_observatory.rs` | M9a search observatory | Research |
| `policy_seed_runner.rs` | Policy seed runner | Research |
| `validation_pass.rs` | Validation pass | Research |

**Classification:** KEEP (active server); research binaries are RR3 candidates

---

### 14. `services/cvrp_server` (31 files, 9,550 LOC)

**Role:** CVRP research server. Contains m11–m22 series experiment binaries (landscape analysis, repair atlas, backbone causality, structural invariants).

**Note:** This is a research-only server — all binaries are experiment binaries in the m11–m22 series. No production service functionality.

**Compile targets — all research binaries:**

m11 through m22 series (20 files), plus: `basin_characterization.rs`, `check_demand.rs`, `elite_manifold_probe.rs`, `elite_partition_probe.rs`, `find_797.rs`, `frozen_partition_probe.rs`, `initial_basin_distribution.rs`, `seed_ecology_study.rs`, `verify_758.rs`, `verify_bks.rs`

**Classification:** Research only — RR3/RR7 candidates. Determine which experiments are complete vs ongoing.

---

### 15. `financial` (69 files, 9,130 LOC)

**Role:** Financial strategy research: ESE (Execution Strategy Engine), strategies, infrastructure.

**Classification:** Research only — separate research programme from Coralys/UltraCrew. RR2 candidate.

---

### 16. `research_experiments` (4 files, 259 LOC)

**Role:** Python bridge and Yahoo adapter for research experiments.

**Classification:** Research only — RR2 candidate.

---

### 17. `original_engine.rs` (1 file, ~13,000+ LOC)

**Role:** Monolithic original engine. Contains `#[allow(dead_code)]` annotations for multiple planned but unwired features (Phase 2 items). Used by `financial` strategies.

**Classification:** RR2/RR3 candidate — largest single file in repository. Determine whether it is the canonical financial engine or a superseded implementation.

---

## Version Candidates (RR3 Priority List)

The following file pairs/groups are version candidates identified during RR1:

| Group | Files | Relationship |
|-------|-------|-------------|
| Campaign runners (ROADEF) | `campaign.rs`, `campaign_engine.rs` | Two versions of the same campaign runner |
| M25.8 experiments (ROADEF) | `m25_8_bridge.rs`, `m25_8b_ecology.rs` | Ecology variant of bridge experiment |
| Ecology ablation (ultracrew) | `inrc_ecology_ablation.rs` (deprecated), `inrc_ecology_ablation_matrix.rs` (active) | Deprecated superseded by active |
| Multi-week ablation (ultracrew) | `inrc_ecology_multi_week_ablation.rs` (deprecated), `inrc_ecology_multi_week_ablation.rs` (active) | Exact filename duplicate across deprecated/ |
| Atlas experiments (ultracrew) | `ultracrew_atlas.rs` (deprecated), `ultracrew_repair_atlas.rs` (deprecated) | Superseded atlas experiments |
| Stub crates | 8 crates with 1–3 files each | May be superseded by `coralys-core` or `coralys-moga` |

---

### 18. `infrastructure/optimization` (2 files, ~LOC TBD)

**Role:** Workspace member. Contains `evolution_engine.rs` and `lib.rs`. Likely an optimization infrastructure layer.

**Classification:** RR2 candidate — determine relationship to `coralys-moga`.

---

### 19. `infrastructure/observatory/api` (15+ files)

**Role:** Workspace member. Observatory API server with handlers, routes, DTOs, market adapter, replay, simulation, timeline, certify, signatures, events, errors, inspector.

**Classification:** KEEP (active API server)

---

### 20. `financial/core` (10 files), `financial/ese` (14 files), `financial/strategies` (45 files)

**Role:** Financial research programme. Three workspace members covering core types, ESE (Execution Strategy Engine), and trading strategies.

**Classification:** Research only — separate programme from Coralys/UltraCrew.

---

### 21. `adapters/cvd001` (8 files)

**Role:** CVD001 adapter. Workspace member.

**Classification:** RR2 candidate — purpose and active consumers to be determined.

---

### 22. `coralys-policy` (1 file — `lib.rs`)

**Role:** Single-file stub. **NOT a workspace member** (absent from root `Cargo.toml`). Cannot be compiled by `cargo build --workspace`.

**Classification:** **ORPHAN** — not in workspace, not reachable. RR7 candidate for deletion.

---

## Workspace Membership Summary

| Crate | In Workspace | Files | LOC |
|-------|-------------|-------|-----|
| `infrastructure/core` | ✓ | 23 | 3,472 |
| `infrastructure/optimization` | ✓ | 2 | TBD |
| `infrastructure/observatory/api` | ✓ | 15+ | TBD |
| `financial/ese` | ✓ | 14 | — |
| `financial/strategies` | ✓ | 45 | — |
| `financial/core` | ✓ | 10 | — |
| `coralys-moga` | ✓ | 33 | 4,500 |
| `coralys-simulation` | ✓ | 2 | 91 |
| `coralys-ecology` | ✓ | 6 | 1,553 |
| `coralys-decision` | ✓ | 2 | 96 |
| `coralys-recommendation` | ✓ | 3 | 293 |
| `coralys-infrastructure` | ✓ | 1 | 14 |
| `adapters/ultracrew` | ✓ | 99 | 19,638 |
| `adapters/chronosentiment` | ✓ | 6 | 1,597 |
| `adapters/cvrp` | ✓ | 25 | 9,662 |
| `adapters/cvd001` | ✓ | 8 | TBD |
| `coralys-v2` | ✓ | 1 | 96 |
| `coralys-core` | ✓ | 11 | 437 |
| `coralys-eval` | ✓ | 5 | 956 |
| `coralys-matching` | ✓ | 1 | 96 |
| `adapters/roadef` | ✓ | 27 | 7,128 |
| `adapters/airline` | ✓ | 50 | 12,760 |
| `adapters/gerad` | ✓ | 8 | 1,462 |
| `coralys-planning` | ✓ | 1 | 103 |
| `services/ultracrew_server` | ✓ | 21 | 7,981 |
| `services/cvrp_server` | ✓ | 31 | 9,550 |
| `coralys-policy` | **✗ ORPHAN** | 1 | 136 |
| `research_experiments` | **✗** | 4 | 259 |
| `financial` (root) | **✗** | — | — |

---

## Files Not Yet Classified

The following require RR2 investigation:

- `original_engine.rs` (root-level monolith, ~13,000+ LOC — not a workspace member)
- `tests/` (workspace-level integration tests)
- `benchmarks/` directory
- Any `.rs` files in `scripts/`, `scratch/`, or other non-crate directories
- `coralys-policy/src/lib.rs` (orphan — not in workspace)

---

## Next Steps

1. **RR2** — Reachability: for each crate, determine whether it is compiled by any active `Cargo.toml` workspace member.
2. **RR3** — Version Analysis: resolve the six version candidate groups above.
3. **RR6** — Dependency Reduction: audit the 8 stub crates; determine if any can be removed.