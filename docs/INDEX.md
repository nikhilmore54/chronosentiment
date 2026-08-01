# Canonical Repository Index

**Document ID:** GOV-IDX-001
**Version:** 1.0
**Status:** Active
**Created:** 2026-08-01

---

## Purpose

This is the single authoritative entry point for the repository. Every canonical artifact has exactly one entry here. If a document, crate, experiment, or evidence asset is not listed in this index, it is not canonical.

**Rule:** Before creating any new document, add it here first. If it cannot be added here, it should not be created.

---

## Current Status

| Area | Status | Last Updated |
|------|--------|-------------|
| Section 2 — Pairing Topology Mutation Evaluation | **FROZEN** 2026-08-01 | 2026-08-01 |
| Section 3 — Coralys Native Scheduler | Active — Experiment 0 spec complete | 2026-08-01 |
| Experiment Harness | Stable — all 5 modules complete | 2026-08-01 |
| GERAD Coralys v1.0 Baseline | **FROZEN** 2026-08-01 | 2026-08-01 |
| Repository Governance | Active — GOV-001 complete, GOV-002 this document | 2026-08-01 |

---

## Frozen Documents

Documents in this section must not be modified except for typographical corrections approved by the reviewer.

| ID | Path | Frozen Date | Modification Policy |
|----|------|-------------|---------------------|
| UC-R-001 §2 | `docs/research/UltraCrew_Pairing_Topology_Mutation_Evaluation.md` (Section 2) | 2026-08-01 | Numerical corrections and typographical fixes only; requires reviewer approval + version increment |
| BENCH-001 | `adapters/airline/tests/gerad_coralys.rs` | 2026-08-01 | No modifications; improvements go in `gerad_coralys_v2.rs` |

---

## Canonical Documents

### 1. Governance

| ID | Path | Title | Role |
|----|------|-------|------|
| GOV-KS-001 | `docs/governance/KNOWLEDGE_SURVEY.md` | Repository Knowledge Survey | Full typed asset inventory — authoritative |
| GOV-IDX-001 | `docs/INDEX.md` | Canonical Repository Index | This document — entry point for all canonical assets |
| GOV-DEP-001 | `docs/governance/DEPENDENCY_GRAPH.md` | Repository Dependency Graph | Typed dependency graph across all knowledge systems |
| GOV-CLN-001 | `docs/governance/CLEANUP_REGISTER.md` | Cleanup Register | Tracks every consolidation, merge, archival, deletion |

### 2. Research Programme

| ID | Path | Title | Status |
|----|------|-------|--------|
| UC-R-001 | `docs/research/UltraCrew_Pairing_Topology_Mutation_Evaluation.md` | Pairing Topology Mutation Evaluation | Section 2 FROZEN; Section 3 active |
| UC-R-002 | `docs/research/UltraCrew_Coralys_Native_Scheduler_Section3.md` | Coralys Native Scheduler — Section 3 | Active — Experiment 0 spec complete |
| UC-R-003 | `docs/research/UltraCrew_GENCOL_Pipeline_Divergence_Analysis.md` | GENCOL Pipeline Divergence Analysis | Active |
| UC-R-004 | `docs/research/UltraCrew_Layover_Threshold_Experiment.md` | Layover Threshold Experiment | Active |
| UC-R-005 | `docs/research/UltraCrew_Objective_Function_Alignment.md` | Objective Function Alignment | Active |

### 3. Experiment Infrastructure

| ID | Path | Title | Status |
|----|------|-------|--------|
| HARNESS-001 | `adapters/airline/tests/harness/schema.rs` | Experiment Schema | Stable |
| HARNESS-002 | `adapters/airline/tests/harness/logging.rs` | Event Logger | Stable |
| HARNESS-003 | `adapters/airline/tests/harness/persistence.rs` | Result Persistence | Stable |
| HARNESS-004 | `adapters/airline/tests/harness/reproducibility.rs` | Reproducibility Info | Stable |
| HARNESS-005 | `adapters/airline/tests/harness/report.rs` | Report Generator | Stable |
| HARNESS-006 | `adapters/airline/tests/harness/mod.rs` | Harness Root | Stable |

### 4. Benchmarks

| ID | Path | Title | Status |
|----|------|-------|--------|
| BENCH-001 | `adapters/airline/tests/gerad_coralys.rs` | GERAD Coralys v1.0 Baseline | **FROZEN** 2026-08-01 |
| BENCH-002 | `adapters/airline/tests/gerad_e2e.rs` | GERAD End-to-End | Active |
| BENCH-003 | `adapters/airline/tests/benchmark.rs` | General Benchmark Suite | Active |

### 5. Platform Crates

| ID | Path | Role |
|----|------|------|
| CRATE-001 | `coralys-core/` | Core EA engine |
| CRATE-002 | `coralys-decision/` | Decision layer |
| CRATE-003 | `coralys-ecology/` | Ecology / population dynamics |
| CRATE-004 | `coralys-eval/` | Fitness evaluation |
| CRATE-005 | `coralys-infrastructure/` | Shared infrastructure |
| CRATE-006 | `coralys-matching/` | Matching algorithms |
| CRATE-007 | `coralys-moga/` | Multi-objective GA |
| CRATE-008 | `coralys-planning/` | Planning layer |
| CRATE-009 | `coralys-policy/` | Policy layer |
| CRATE-010 | `coralys-recommendation/` | Recommendation engine |
| CRATE-011 | `coralys-simulation/` | Simulation layer |
| CRATE-012 | `coralys-v2/` | Coralys v2 |

### 6. Adapters

| ID | Path | Domain |
|----|------|--------|
| ADAPTER-001 | `adapters/airline/` | Airline crew pairing — canonical research adapter |
| ADAPTER-002 | `adapters/chronosentiment/` | ChronoSentiment financial signals |
| ADAPTER-003 | `adapters/roadef/` | ROADEF 2026 challenge |

### 7. Architecture

| ID | Path | Title |
|----|------|-------|
| ARCH-001 | `docs/CORALYS_PLATFORM_ARCHITECTURE.md` | Coralys Platform Architecture |
| ARCH-002 | `docs/PLATFORM_CRATE_RESPONSIBILITIES.md` | Platform Crate Responsibilities |
| ARCH-003 | `docs/Service_Boundary_Definition.md` | Service Boundary Definition |
| ARCH-004 | `docs/Event_Flow_Specification.md` | Event Flow Specification |
| ARCH-005 | `docs/ARCHITECTURE_GLOSSARY.md` | Architecture Glossary |

### 8. Evidence Governance

| ID | Path | Title |
|----|------|-------|
| STANDARD-001 | `docs/CORALYS_EVIDENCE_GOVERNANCE_STANDARD.md` | Coralys Evidence Governance Standard |
| STANDARD-002 | `docs/BENCHMARK-GOVERNANCE.md` | Benchmark Governance |
| STANDARD-003 | `docs/BENCHMARK-REFERENCE-SPECIFICATION-v1.0.md` | Benchmark Reference Specification |
| EV-GOV-001 | `docs/CHRONOSENTIMENT_EVIDENCE_PROGRAMME.md` | ChronoSentiment Evidence Programme |
| EV-GOV-002 | `docs/ULTRACREW_WORKFORCE_EVIDENCE.md` | UltraCrew Workforce Evidence |

### 9. Product & Strategy

| ID | Path | Title |
|----|------|-------|
| BLUEPRINT-001 | `docs/ChronoSentiment_Product_Blueprint_v1.md` | ChronoSentiment Product Blueprint |
| STRATEGY-001 | `docs/CORALYS_PLATFORM_STRATEGY.md` | Coralys Platform Strategy |
| STRATEGY-002 | `docs/ChronoSentiment_Product_Strategy_v1.md` | ChronoSentiment Product Strategy |
| PRD-001 | `docs/PRD_v3_3.md` | Product Requirements Document v3.3 |
| ROADMAP-001 | `docs/EP-002_ROADMAP.md` | Engineering Programme Roadmap |

### 10. Visualisation

| ID | Path | Title | Status |
|----|------|-------|--------|
| VIZ-001 | `docs/code_map.html` | Code Map | Needs update — Rust-only; does not cover research or governance nodes |

---

## Evidence Directories (runtime)

These directories are populated by experiments and are not individually indexed. They are governed by the harness persistence module (`HARNESS-003`).

| Path | Contents |
|------|----------|
| `results/` | Experiment results — CSV, JSON, per-run directories |
| `logs/` | Runtime logs |
| `pilot_sessions/` | Pilot session recordings |
| `snapshots/` | System snapshots |
| `reports/` | Generated markdown reports |

---

## Non-Canonical / Pending Review

The following documents exist in the repository but have not been confirmed as canonical. They require review before being added to this index or archived.

| Path | Issue |
|------|-------|
| `docs/REPOSITORY_SURVEY.md` | Superseded by `docs/governance/KNOWLEDGE_SURVEY.md` — archive candidate |
| `docs/ChronoSentiment_Personal_Blueprint_v1.md` | Scope overlap with `ChronoSentiment_Product_Blueprint_v1.md` — review needed |
| `docs/CODEBASE_ASSESSMENT.md` | Possible duplicate of `CODEBASE_ARCHITECTURE_ASSESSMENT.md` — review needed |
| `docs/EP-001_MILESTONE.md` | Possible duplicate of `docs/P001_MILESTONE.md` — review needed |
| `docs/research/RESEARCH_LINEAGE.md` | Possible duplicate of `docs/RESEARCH_LINEAGE.md` — review needed |
| `docs/research/CS-R-009_AI_Adoption_Investment_Management.md` | Stub — content not yet authored |
| `docs/research/CS-R-015_Investment_Thesis.md` | Stub — content not yet authored |
| `docs/research/CS-R-015A_Executive_Investment_Summary.md` | Stub — content not yet authored |
| `docs/ChronoSentiment_Product_Blueprint_v1.md` | Stub — content not yet authored |
| `docs/ui/uiux.md` | Stub — content not yet authored |

---

## Changelog

| Version | Date | Change |
|---------|------|--------|
| 1.0 | 2026-08-01 | Initial creation — Repository v2 Governance Artifact 2 |