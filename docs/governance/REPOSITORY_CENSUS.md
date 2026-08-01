# Repository Census

**Document ID:** GOV-RR1-001
**Version:** 2.0
**Status:** Active
**Created:** 2026-08-01
**Programme:** Repository Rationalization Programme (RRP v1.0)

---

## Purpose

This document is the engineering source of truth for the Repository Rationalization Programme. It answers exactly one question:

> **What exists?**

Disposition decisions (keep, archive, delete) belong to RR4–RR7. This document records facts only.

**Rule:** No file may be archived or deleted under RRP without a corresponding entry in this census confirming its classification.

---

## Schema

Each crate entry records:

| Field | Values |
|-------|--------|
| **Workspace** | `coralys-workspace` |
| **Package** | Cargo package name |
| **Workspace Member** | Yes / No (Orphan) |
| **Lifecycle** | Active / Research / Deprecated / Frozen / Stub / Unknown |
| **Compile Reachability** | Workspace / Dependency / Binary / Example / Test / Bench / None |
| **Canonical Owner** | Which crate/module owns this capability |
| **Governed By** | CLN-xxx or RR-xxx decision reference, if applicable |
| **RR Decision** | Pending (all entries start as Pending) |

---

## Baseline Metrics (2026-08-01)

| Metric | Value |
|--------|-------|
| Total Rust source files | 477 |
| Total LOC (confirmed by wc -l) | 114,544 |
| Workspace members | 26 |
| Non-workspace crates | 2 (coralys-policy orphan; research_experiments) |
| Files in deprecated/ subdirectories | 7 (adapters/ultracrew/src/bin/deprecated/) |
| Stub crates (≤3 files, minimal content) | 8 |
| Version candidate groups | 6 (see GOV-RR3-001) |

---

## Crate Inventory

### 1. `adapters/airline` — `coralys-airline`

| Field | Value |
|-------|-------|
| **Package** | coralys-airline |
| **Workspace Member** | Yes |
| **Files / LOC** | 50 files / 12,760 LOC |
| **Lifecycle** | Active (lib) + Frozen (tests) |
| **Compile Reachability** | Workspace (lib); Test (tests/); Dependency (coralys-gerad depends on this) |
| **Canonical Owner** | Self — canonical airline crew pairing domain model |
| **Governed By** | ADAPTER-001 (GOV-KS-001); CLN-012 (gerad→airline one-directional confirmed) |
| **RR Decision** | Pending |

**Source files by compile target:**

| Path | Target | Lifecycle |
|------|--------|-----------|
| `src/lib.rs` | lib | Active |
| `src/domain/*.rs` (8 files) | lib | Active |
| `src/legality/*.rs` (8 files) | lib | Active |
| `src/optimization/*.rs` (4 files) | lib | Active |
| `src/optimization/neighborhood/*.rs` (3 files) | lib | Active |
| `src/optimization/search/*.rs` (3 files) | lib | Active |
| `src/planner/*.rs` (4 files) | lib | Active |
| `src/resilience/*.rs` (4 files) | lib | Active |
| `tests/gerad_coralys.rs` | test | Frozen (2026-08-01) |
| `tests/gerad_e2e.rs` | test | Active |
| `tests/harness/mod.rs` | test | Frozen (2026-08-01) |
| `tests/harness/schema.rs` | test | Frozen (2026-08-01) |
| `tests/harness/logging.rs` | test | Frozen (2026-08-01) |
| `tests/harness/persistence.rs` | test | Frozen (2026-08-01) |
| `tests/harness/reproducibility.rs` | test | Frozen (2026-08-01) |
| `tests/harness/report.rs` | test | Frozen (2026-08-01) |

---

### 2. `adapters/gerad` — `coralys-gerad`

| Field | Value |
|-------|-------|
| **Package** | coralys-gerad |
| **Workspace Member** | Yes |
| **Files / LOC** | 8 files / 1,462 LOC |
| **Lifecycle** | Active |
| **Compile Reachability** | Workspace (lib) |
| **Canonical Owner** | Self — GERAD G-2014-22 benchmark parser |
| **Governed By** | CLN-012 (one-directional gerad→airline dependency confirmed) |
| **RR Decision** | Pending |

---

### 3. `adapters/ultracrew` — `coralys-ultracrew`

| Field | Value |
|-------|-------|
| **Package** | coralys-ultracrew |
| **Workspace Member** | Yes |
| **Files / LOC** | 99 files / 19,638 LOC |
| **Lifecycle** | Mixed (see below) |
| **Compile Reachability** | Workspace (lib); Binary (src/bin/*.rs) |
| **Canonical Owner** | Self — UltraCrew INRC scheduling adapter |
| **Governed By** | CLN-010 (telemetry.rs canonical for service logging) |
| **RR Decision** | Pending |

**Source files by lifecycle:**

| Path | Target | Lifecycle |
|------|--------|-----------|
| `src/lib.rs` + `src/**/*.rs` (lib files) | lib | Active |
| `src/telemetry.rs` | lib | Active — canonical service logging |
| `src/bin/ultracrew-cli.rs` | Binary | Active |
| `src/bin/config_sweep.rs` | Binary | Research |
| `src/bin/inrc_ecology_ablation_matrix.rs` | Binary | Research |
| `src/bin/inrc_ecology_cost_curve.rs` | Binary | Research |
| `src/bin/inrc_ecology_memory_depth.rs` | Binary | Research |
| `src/bin/inrc_ecology_multi_week_ablation.rs` | Binary | Research |
| `src/bin/inrc_ecology_response_curve.rs` | Binary | Research |
| `src/bin/inrc_m22_ancestry.rs` | Binary | Research |
| `src/bin/inrc_m22_benchmark.rs` | Binary | Research |
| `src/bin/inrc_natural_history_pilot.rs` | Binary | Research |
| `src/bin/m23a_synthetic.rs` | Binary | Research |
| `src/bin/m30_0b_passive_telemetry.rs` | Binary | Research |
| `src/bin/m30_0d_active_pilot.rs` | Binary | Research |
| `src/bin/m31_2a_engagement_audit.rs` | Binary | Research |
| `src/bin/m31_benchmarks.rs` | Binary | Research |
| `src/bin/story1.rs` | Binary | Research |
| `src/bin/deprecated/inrc_ecology_ablation.rs` | Binary | Deprecated |
| `src/bin/deprecated/inrc_ecology_history_test.rs` | Binary | Deprecated |
| `src/bin/deprecated/inrc_ecology_horizon_test.rs` | Binary | Deprecated |
| `src/bin/deprecated/inrc_ecology_mechanism_audit.rs` | Binary | Deprecated |
| `src/bin/deprecated/inrc_ecology_multi_week_ablation.rs` | Binary | Deprecated |
| `src/bin/deprecated/ultracrew_atlas.rs` | Binary | Deprecated |
| `src/bin/deprecated/ultracrew_repair_atlas.rs` | Binary | Deprecated |

---

### 4. `adapters/cvrp` — `coralys-cvrp`

| Field | Value |
|-------|-------|
| **Package** | coralys-cvrp |
| **Workspace Member** | Yes |
| **Files / LOC** | 25 files / 9,662 LOC |
| **Lifecycle** | Mixed (Active + Research) |
| **Compile Reachability** | Workspace (lib); Binary (src/bin/*.rs) |
| **Canonical Owner** | Self — CVRP adapter |
| **Governed By** | — |
| **RR Decision** | Pending |

**Binaries by lifecycle:**

| File | Lifecycle |
|------|-----------|
| `campaign.rs` | Active |
| `bdd_baseline.rs` through `bdd_validation.rs` (8 files) | Active |
| `compare.rs`, `cvrp_sanity.rs`, `cvrplib_registry.rs`, `search_config.rs` | Active |
| `m30_2_active_pilot.rs`, `m30_2a_1_ecology_audit.rs`, `m30_2a_2_shadow_advisory.rs` | Research |

---

### 5. `adapters/roadef` — `coralys-roadef`

| Field | Value |
|-------|-------|
| **Package** | coralys-roadef |
| **Workspace Member** | Yes |
| **Files / LOC** | 27 files / 7,128 LOC |
| **Lifecycle** | Mixed (Active + Research) |
| **Compile Reachability** | Workspace (lib); Binary (src/bin/*.rs) |
| **Canonical Owner** | Self — ROADEF 2026 network optimization adapter |
| **Governed By** | — |
| **RR Decision** | Pending |

**Binaries by lifecycle:**

| File | Lifecycle | Note |
|------|-----------|------|
| `campaign.rs` | Research | Version candidate — see GOV-RR3-001 |
| `campaign_engine.rs` | Research | Version candidate — see GOV-RR3-001 |
| `e001_dual_path.rs` | Active | Validation harness |
| `eval_profiler.rs` | Research | — |
| `tiny_solver.rs` | Research | — |
| `m25_benchmark.rs` | Research | — |
| `m25_8_bridge.rs` | Research | Version candidate — see GOV-RR3-001 |
| `m25_8b_ecology.rs` | Research | Version candidate — see GOV-RR3-001 |
| `m25_final.rs` | Research | — |
| `m26_1_observation_audit.rs` through `m26_1e_survival_curves.rs` (4 files) | Research | — |
| `m26_3_passive_learner.rs` | Research | — |
| `m26_4a_shadow_advisory.rs`, `m26_4b_active_pilot.rs` | Research | — |
| `m27_1_passive_operator_telemetry.rs` | Research | — |

---

### 6. `adapters/chronosentiment` — `coralys-chronosentiment`

| Field | Value |
|-------|-------|
| **Package** | coralys-chronosentiment |
| **Workspace Member** | Yes |
| **Files / LOC** | 6 files / 1,597 LOC |
| **Lifecycle** | Active |
| **Compile Reachability** | Workspace (lib) |
| **Canonical Owner** | Self — ChronoSentiment domain adapter |
| **Governed By** | — |
| **RR Decision** | Pending |

---

### 7. `adapters/cvd001` — `coralys-cvd001`

| Field | Value |
|-------|-------|
| **Package** | coralys-cvd001 |
| **Workspace Member** | Yes |
| **Files / LOC** | 8 files / TBD LOC |
| **Lifecycle** | Unknown |
| **Compile Reachability** | Workspace (lib) |
| **Canonical Owner** | Unknown — purpose to be determined in RR2 |
| **Governed By** | — |
| **RR Decision** | Pending |

---

### 8. `coralys-core`

| Field | Value |
|-------|-------|
| **Package** | coralys-core |
| **Workspace Member** | Yes |
| **Files / LOC** | 11 files / 437 LOC |
| **Lifecycle** | Active |
| **Compile Reachability** | Dependency (depended on by multiple crates) |
| **Canonical Owner** | Self — core Coralys platform types |
| **Governed By** | — |
| **RR Decision** | Pending |

---

### 9. `coralys-ecology`

| Field | Value |
|-------|-------|
| **Package** | coralys-ecology |
| **Workspace Member** | Yes |
| **Files / LOC** | 6 files / 1,553 LOC |
| **Lifecycle** | Active |
| **Compile Reachability** | Dependency |
| **Canonical Owner** | Self — ecology system |
| **Governed By** | CLN-011 (ObjectiveVector remains in this crate; coralys-eval owns objective value semantics) |
| **RR Decision** | Pending |

---

### 10. `coralys-eval`

| Field | Value |
|-------|-------|
| **Package** | coralys-eval |
| **Workspace Member** | Yes |
| **Files / LOC** | 5 files / 956 LOC |
| **Lifecycle** | Active |
| **Compile Reachability** | Dependency |
| **Canonical Owner** | Self — canonical owner of objective value semantics |
| **Governed By** | CLN-011 |
| **RR Decision** | Pending |

---

### 11. `coralys-moga`

| Field | Value |
|-------|-------|
| **Package** | coralys-moga |
| **Workspace Member** | Yes |
| **Files / LOC** | 33 files / 4,500 LOC |
| **Lifecycle** | Active |
| **Compile Reachability** | Dependency + Workspace |
| **Canonical Owner** | Self — multi-objective genetic algorithm engine |
| **Governed By** | — |
| **RR Decision** | Pending |

---

### 12–18. Stub Crates

| Crate | Package | Workspace Member | Files | LOC | Lifecycle | Compile Reachability |
|-------|---------|-----------------|-------|-----|-----------|---------------------|
| `coralys-infrastructure` | coralys-infrastructure | Yes | 1 | 14 | Stub | Workspace |
| `coralys-matching` | coralys-matching | Yes | 1 | 96 | Stub | Workspace |
| `coralys-planning` | coralys-planning | Yes | 1 | 103 | Stub | Workspace |
| `coralys-recommendation` | coralys-recommendation | Yes | 3 | 293 | Stub | Workspace |
| `coralys-simulation` | coralys-simulation | Yes | 2 | 91 | Stub | Workspace |
| `coralys-v2` | coralys-v2 | Yes | 1 | 96 | Stub | Workspace |
| `coralys-decision` | coralys-decision | Yes | 2 | 96 | Stub | Workspace |

**Note:** All 7 stub crates are workspace members but have minimal content (1–3 files). Whether any active crate depends on them is to be determined in RR2. Governed By: RR2 audit pending.

---

### 19. `coralys-policy`

| Field | Value |
|-------|-------|
| **Package** | coralys-policy |
| **Workspace Member** | **No — ORPHAN** |
| **Files / LOC** | 1 file / 136 LOC |
| **Lifecycle** | Unknown |
| **Compile Reachability** | None (not in workspace; cannot be built by `cargo build --workspace`) |
| **Canonical Owner** | Unknown |
| **Governed By** | — |
| **RR Decision** | Pending |

---

### 20. `infrastructure/core` — `chronosentiment_core`

| Field | Value |
|-------|-------|
| **Package** | chronosentiment_core |
| **Workspace Member** | Yes |
| **Files / LOC** | 23 files / 3,472 LOC |
| **Lifecycle** | Active |
| **Compile Reachability** | Workspace (lib + binaries) |
| **Canonical Owner** | Self — core infrastructure (capture daemon, importers) |
| **Governed By** | CLN-013 (deprecated_examples/ deleted 2026-08-01) |
| **RR Decision** | Pending |

---

### 21. `infrastructure/optimization`

| Field | Value |
|-------|-------|
| **Package** | TBD |
| **Workspace Member** | Yes |
| **Files / LOC** | 2 files / TBD LOC |
| **Lifecycle** | Unknown |
| **Compile Reachability** | Workspace (lib) |
| **Canonical Owner** | Unknown — relationship to coralys-moga to be determined in RR2 |
| **Governed By** | — |
| **RR Decision** | Pending |

---

### 22. `infrastructure/observatory/api`

| Field | Value |
|-------|-------|
| **Package** | TBD |
| **Workspace Member** | Yes |
| **Files / LOC** | 15+ files / TBD LOC |
| **Lifecycle** | Active |
| **Compile Reachability** | Workspace (lib + binary: main.rs) |
| **Canonical Owner** | Self — Observatory API server |
| **Governed By** | — |
| **RR Decision** | Pending |

---

### 23. `services/ultracrew_server`

| Field | Value |
|-------|-------|
| **Package** | TBD |
| **Workspace Member** | Yes |
| **Files / LOC** | 21 files / 7,981 LOC |
| **Lifecycle** | Mixed (Active + Research) |
| **Compile Reachability** | Workspace (lib + binaries) |
| **Canonical Owner** | Self — UltraCrew HTTP server |
| **Governed By** | — |
| **RR Decision** | Pending |

**Binaries by lifecycle:**

| File | Lifecycle |
|------|-----------|
| `acceptance_test.rs`, `benchmark.rs`, `cs_governance_validation.rs`, `ecology_validation.rs` | Active |
| `inrc_archive_forensics.rs`, `m8g_cs_validation.rs`, `m8g_ultracrew_validation.rs`, `m9a_search_observatory.rs`, `policy_seed_runner.rs`, `validation_pass.rs` | Research |

---

### 24. `services/cvrp_server`

| Field | Value |
|-------|-------|
| **Package** | TBD |
| **Workspace Member** | Yes |
| **Files / LOC** | 31 files / 9,550 LOC |
| **Lifecycle** | Research |
| **Compile Reachability** | Workspace (binaries only — research-only server) |
| **Canonical Owner** | Self — CVRP research experiment server |
| **Governed By** | — |
| **RR Decision** | Pending |

**All 31 binaries are research experiment binaries (m11–m22 series + supporting tools).**

---

### 25–27. Financial Programme

| Crate | Package | Workspace Member | Files | Lifecycle | Compile Reachability |
|-------|---------|-----------------|-------|-----------|---------------------|
| `financial/core` | TBD | Yes | 10 | Research | Dependency |
| `financial/ese` | TBD | Yes | 14 | Research | Workspace (lib + binary) |
| `financial/strategies` | TBD | Yes | 45 | Research | Workspace (lib + binaries) |

**Note:** Financial programme is a separate research programme from Coralys/UltraCrew. `original_engine.rs` (root-level monolith, ~13,000+ LOC, not a workspace member) is used by financial strategies.

---

### 28. `research_experiments`

| Field | Value |
|-------|-------|
| **Package** | None |
| **Workspace Member** | No |
| **Files / LOC** | 4 files / 259 LOC |
| **Lifecycle** | Research |
| **Compile Reachability** | None (not in workspace) |
| **Canonical Owner** | Unknown |
| **Governed By** | — |
| **RR Decision** | Pending |

---

## Non-Workspace Files

| File | Lifecycle | Compile Reachability | Note |
|------|-----------|---------------------|------|
| `original_engine.rs` | Unknown | None (not in workspace) | Root-level monolith, ~13,000+ LOC. Used by financial strategies via path include or direct compilation. |
| `coralys-policy/src/lib.rs` | Unknown | None | Orphan — not in workspace Cargo.toml |

---

## Version Candidates

Version candidate groups have been moved to GOV-RR3-001. See `docs/governance/RR3_VERSION_ANALYSIS.md` (to be created).

The six groups identified during RR1 are:

1. `adapters/roadef/src/bin/campaign.rs` vs `campaign_engine.rs`
2. `adapters/roadef/src/bin/m25_8_bridge.rs` vs `m25_8b_ecology.rs`
3. `adapters/ultracrew/src/bin/deprecated/inrc_ecology_ablation.rs` vs active `inrc_ecology_ablation_matrix.rs`
4. `adapters/ultracrew/src/bin/deprecated/inrc_ecology_multi_week_ablation.rs` vs active version
5. `adapters/ultracrew/src/bin/deprecated/ultracrew_atlas.rs` + `ultracrew_repair_atlas.rs`
6. Stub crates (7 crates) vs `coralys-core` / `coralys-moga`

---

## Governance Rule

> **Any workspace member discovered during RR1 that is absent from GOV-KS-001 must be added to the Knowledge Survey before RR2 commences.**

Missing from GOV-KS-001 (to be added): `financial/core`, `financial/ese`, `financial/strategies`, `infrastructure/optimization`, `infrastructure/observatory/api`, `adapters/cvd001`.