# Canonical Repository Index

**Document ID:** GOV-IDX-001
**Version:** 1.57
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
| EV-GOV-003 | `G_GATE_v1.1_STATISTICAL_CLOSURE.md` | G-GATE v1.1 closed: B4 INCONCLUSIVE; predictive value not established; Decision A |

### 9. Product & Strategy

| ID | Path | Title |
|----|------|-------|
| BLUEPRINT-001 | `docs/ChronoSentiment_Product_Blueprint_v1.md` | ChronoSentiment Product Blueprint |
| STRATEGY-001 | `docs/CORALYS_PLATFORM_STRATEGY.md` | Coralys Platform Strategy |
| STRATEGY-002 | `docs/ChronoSentiment_Product_Strategy_v1.md` | ChronoSentiment Product Strategy |
| PRD-001 | `docs/PRD_v3_3.md` | Product Requirements Document v3.3 (superseded; historical) |
| PRD-002 | `docs/CHRONOSENTIMENT_PRD_V1.md` | ChronoSentiment Product Definition v1.0 (authoritative commercial PRD) |
| CS-P-001 | `docs/CS-P-001_DECISION_SUPPORT_PRODUCT_MODE.md` | Product mode after G-GATE v1.1 close; co-pilot / paper trading; v1.2 not opened |
| CS-P-002 | `docs/CS-P-002_DECISION_VALIDATION_PLATFORM.md` | Decision Validation Platform v1: one engine, replay/live adapters, backtest + forward paper |
| CS-P-002-R1 | `product_validation/B4_unfrozen_dev/HISTORICAL_PERFORMANCE_REPORT.md` | B4 historical product validation baseline (`unfrozen-dev`; not G-GATE; v1.0 not frozen) |
| CS-P-003 | `docs/CS-P-003_FORWARD_PAPER_VALIDATION.md` | Forward/Paper Validation v0.1: daily tick continues (confirmation, not discovery) |
| CS-P-004 | `docs/CS-P-004_HISTORICAL_RESEARCH_LABORATORY.md` | Historical Research & Robustness Laboratory: B4 reconstruction, no engine change |
| CS-P-004-R1 | `product_validation/CS-P-004_unfrozen_dev/HISTORICAL_RESEARCH_SUMMARY.md` | First B4 laboratory reports (`unfrozen-dev`; not a candidate policy; not G-GATE) |
| CS-P-004-A1 | `product_validation/CS-P-004_adapter_v0.1/HISTORICAL_RESEARCH_SUMMARY.md` | Adapter v0.1 lab re-run: unavailable confidence + preserved evidence; not a candidate policy |
| CS-P-004-E1 | `docs/CS-P-004_HISTORICAL_RESEARCH_LABORATORY.md` | Assessment Enrichment v0.1: factors at T; code + tests |
| CS-P-004-E1-S1 | `product_validation/assessment_enrichment_v0.1/` | Information-fidelity snapshot + factor availability (not B5; not a strategy experiment) |
| CS-P-TEST-001 | `docs/CS-P-TEST-001_DECISION_INTELLIGENCE_VERIFICATION_MATRIX.md` | Decision intelligence verification matrix (vision-level tests; not a snapshot) |
| CS-P-005 | `docs/CS-P-005_FACTOR_ECOLOGY_ANALYSIS.md` | Factor Ecology Analysis v0.1: states at T; no candidate policy |
| CS-P-AUDIT-001 | `docs/CS-P-AUDIT-001_ADAPTER_DISCIPLINE.md` | Read-only ChronoSentiment adapter discipline inventory (no cleanup) |
| CS-P-CLEAN-001 | `docs/CS-P-CLEAN-001_ADAPTER_QUARANTINE.md` | PR-1: research/legacy quarantine; B3/B4 generators preserved; no policy change |
| CS-P-CLEAN-002 | `docs/CS-P-CLEAN-002_EXPLICIT_POLICY_CONTRACT.md` | PR-2: explicit DecisionPolicy required; baseline fixture is not the ChronoSentiment strategy |
| CS-P-006 | `docs/CS-P-006_CORALYS_POLICY_DISCOVERY.md` | Research frozen at C.3-F; P.3–P.6 observatory slice; no Search #3 |
| CS-P-006-A | `docs/CS-P-006-A_POLICY_DISCOVERY_CONTRACT.md` | PolicyArtifact consumption contract (`csp006a.policy_artifact.1`); no search |
| CS-P-006-B | `docs/CS-P-006-B_RESEARCH_PROTOCOL.md` | Discovery protocol; S1 certified; B.1 partition frozen |
| CS-P-006-B.1 | `docs/CS-P-006-B.1_CHRONOLOGICAL_PARTITION.md` | Frozen development/selection/evaluation partition (protocol TRAIN/VAL/TEST) |
| CS-P-006-S1 | `docs/CS-P-006-S1_SEVEN_INSTRUMENT_SNAPSHOT.md` | Disposable 7-instrument research snapshot (not B4/B5); certified PASS |
| CS-P-006-V | `docs/CS-P-006-V_DECISION_VALUE.md` | Vision: prediction ≠ decision; future certified state families; not this genome |
| CS-P-006-C | `docs/CS-P-006-C_POLICY_DISCOVERY.md` | Search #1 immutable: reproducible discovery, failed generalization |
| CS-P-006-C.1 | `docs/CS-P-006-C.1_SEARCH_DIAGNOSIS.md` | Post-search diagnosis; no Search #2; TMV insufficiency not concluded |
| CS-P-006-C.2 | `docs/CS-P-006-C.2_RESEARCH_GAP_REVIEW.md` | Instrumentation & information-gap review; Search #2 not authorized |
| CS-P-006-C.2-O | `docs/CS-P-006-C.2-O_SEARCH_OBSERVABILITY.md` | Search observability; same artifact on/off; no Search #2 |
| CS-P-006-C.2-P | `docs/CS-P-006-C.2-P_POPULATION_ECOLOGY.md` | Search #1 population ecology; no Search #2 |
| CS-P-006-C.2-R | `docs/CS-P-006-C.2-R_RECOMMENDATION_OUTCOME.md` | Search #1 recommendation-vs-outcome matrix; no Search #2 |
| CS-P-006-C.2-S | `docs/CS-P-006-C.2-S_SELECTION_DECISION_VALUE.md` | Selection bottleneck + decision-value review; no Search #2 |
| CS-P-006-C.2-D | `docs/CS-P-006-C.2-D_DECISION_VALUE_LANDSCAPE.md` | Decision-value landscape of Search #1 recommendations; no Search #2 |
| CS-P-006-M | `docs/CS-P-006-M_DECISION_VALUE_MODEL.md` | Decision-value model (protocol questions); not Coralys fitness; no Search #2 |
| CS-P-006-M.1 | `docs/CS-P-006-M.1_DECISION_VALUE_SPECIFICATION.md` | Decision-value specification; continuous V; regret not fitness; no Search #2 |
| CS-P-006-N | `docs/CS-P-006-N_DECISION_VALUE_RESEARCH_HARNESS.md` | Decision-value harness; symbol matrices required; C.3 protocol opened separately |
| CS-P-006-C.3 | `docs/CS-P-006-C.3_PROTOCOL.md` | C.3 protocol authorization; Search #2 not started; same TMV |
| CS-P-006-C.3-I | `docs/CS-P-006-C.3-I_IMPLEMENTATION.md` | C.3-I implementation PASS; identity gate; Search #2 not run |
| CS-P-006-C.3-R | `docs/CS-P-006-C.3-R_SEARCH.md` | One authorized Search #2 run; Search #1 immutable; no iteration |
| CS-P-006-C.3-C | `docs/CS-P-006-C.3-C_COMPARATIVE_REVIEW.md` | Search #1 vs #2 sealed-artifact review; no Search #3 |
| CS-P-006-C.3-D | `docs/CS-P-006-C.3-D_RULE_ECOLOGY.md` | Search #2 live-rule ecology; candidate not promoted; no Search #3 |
| CS-P-006-C.3-E | `docs/CS-P-006-C.3-E_RULE_PERSISTENCE.md` | Search #2 discovered-rule persistence; no pass threshold; no Search #3 |
| CS-P-006-C.3-F | `docs/CS-P-006-C.3-F_STATE_ACTION_LANDSCAPE.md` | Certified TMV state x action value landscape; frozen conclusion; no Search #3 |
| CS-P-006-C.3-G | `docs/CS-P-006-C.3-G_REGIME_PERSISTENCE_QUESTION.md` | Regime-persistence question only; research loop stopped; no Search #3 |
| CS-P-006-P | `docs/CS-P-006-P_DECISION_OBSERVATORY.md` | Decision Observatory; evidence dashboard; no early peek; C3-002 paper-only |
| CS-P-006-P.H | `docs/CS-P-006-P.H_HISTORICAL_REPLAY.md` | Historical Observatory Replay; same engine/policy; no lookahead; not C.3-G |
| CS-P-006-P.H.1 | `docs/CS-P-006-P.H.1_DECISION_EVIDENCE_ENGINE.md` | Decision Evidence Engine; Replay v0 20 calendar days archived; not a statistical backtest |
| CS-P-006-P.H.2 | `docs/CS-P-006-P.H.2_MARKET_SESSION_HORIZON.md` | Observatory 20 market sessions; Replay v1; v0 not reinterpreted; not C.3-G |
| CS-P-006-P.H.3 | `docs/CS-P-006-P.H.3_DECISION_EVIDENCE_DASHBOARD.md` | Decision Evidence Dashboard; replay integrity ≠ strategy validation; no C.3-G |
| CS-P-006-P.E | `docs/CS-P-006-P.E_TARGETED_DECISION_EXECUTION.md` | Targeted execution contract; target sealed at T; OHLC first-exit; not C.3-G |
| CS-P-006-P.E.1 | `docs/CS-P-006-P.E.1_EXECUTION_EVIDENCE_SURFACE.md` | Decision / Execution / Evidence layers; Execution Contract v0 owns target_pct; frozen; not C.3-G |
| CS-P-006-P.E.2 | `docs/CS-P-006-P.E.2_LIVE_EXECUTION_OBSERVATION.md` | Frozen prospective lifecycle with fixed Execution Contract v0; not a 5% quality test; not C.3-G |
| CS-P-006-P.E.2.H | `docs/CS-P-006-P.E.2.H_HISTORICAL_LIFECYCLE_VALIDATION.md` | Historical P.E.2 lifecycle validation — PASS; live P.E.2 remains AWAITING_NEXT_SESSION; not a statistical backtest |
| CS-P-006-P.E.B | `docs/CS-P-006-P.E.B_CORALYS_TARGET_FROM_STATE.md` | Pointer — Coralys target discovery is CS-P-006-P.E.3; no Search #3 |
| CS-P-006-P.E.3 | `docs/CS-P-006-P.E.3_CORALYS_TARGET_DISCOVERY.md` | Coralys target from state at T; waits for CS-P-007; P.E.2 is the control; not started; no Search #3 |
| CS-P-006-P.E.3.A | `docs/CS-P-006-P.E.3.A_CORALYS_TARGET_ARTIFACT.md` | Coralys Target Artifact contract; no generator, ATR map, or search; waits for CS-P-007; no Search #3 |
| CS-P-007 | `docs/CS-P-007_STATISTICAL_STRATEGY_VALIDATION.md` | Statistical validation of frozen C3-002 + Execution Contract v0; specified not run; P.E.3 waits; no Search #3 |
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
| 1.57 | 2026-08-15 | CS-P-007 Statistical Strategy Validation specified (not run; P.E.3 waits; no Search #3) |
| 1.56 | 2026-08-15 | CS-P-006-P.E.3.A Coralys Target Artifact contract (P.E.2.H PASS; no target algorithm; no Search #3) |
| 1.55 | 2026-08-15 | CS-P-006-P.E.2.H historical P.E.2 lifecycle validation (15 Jul clock; live P.E.2 untouched; no Search #3) |
| 1.54 | 2026-08-15 | CS-P-006-P.E.3 Coralys Target Discovery specified (P.E.2 frozen as control; no Search #3) |
| 1.53 | 2026-08-15 | CS-P-006-P.E.B Coralys target from certified state at T (P.E.1 is the +5% control; no Search #3) |
| 1.52 | 2026-08-15 | CS-P-006-P.E.2 Live Execution Observation (next cohort with Execution Contract v0; no Search #3) |
| 1.51 | 2026-08-15 | CS-P-006-P.E.1 Execution Evidence Surface (Decision ≠ Execution ≠ Evidence; no Search #3) |
| 1.50 | 2026-08-15 | CS-P-006-P.E Targeted Decision Execution (target sealed at T; OHLC first-exit; no Search #3) |
| 1.49 | 2026-08-15 | CS-P-006-P.H.3 Decision Evidence Dashboard (replay integrity ≠ strategy validation; no Search #3) |
| 1.48 | 2026-08-15 | CS-P-006-P.H.2 20 market-session Observatory horizon (Replay v1; v0 archived; no Search #3) |
| 1.47 | 2026-08-15 | CS-P-006-P.H.1 Decision Evidence Engine (20 calendar days; session rule; not a statistical backtest) |
| 1.46 | 2026-08-15 | CS-P-006-P.H Historical Observatory Replay (same engine; no lookahead; not C.3-G) |
| 1.45 | 2026-08-15 | CS-P-006-P Observatory wording (outcome not yet observed; no early peek; no Search #3) |
| 1.44 | 2026-08-15 | CS-P-006-P Observatory maturity path (countdown / OUTCOME DUE; no early peek; no Search #3) |
| 1.43 | 2026-08-15 | CS-P-006-P prospective C3-002 paper clock (OBSERVING; no outcomes at seal; no Search #3) |
| 1.42 | 2026-08-15 | CS-P-006-P.7 Observatory product screens (Decision object; OBSERVED status; no Search #3) |
| 1.41 | 2026-08-15 | CS-P-006-P.3–P.6 observatory vertical slice (immutable record; append-only outcome; no Search #3) |
| 1.40 | 2026-08-15 | CS-P-006-P Decision Observatory protocol (C3-002 paper-only; research loop stopped; no Search #3) |
| 1.39 | 2026-08-15 | CS-P-006-C.3-G regime-persistence question (experiment not authorized; no Search #3) |
| 1.38 | 2026-08-15 | CS-P-006-C.3-F certified TMV state x action landscape (no Search #3; not a strategy) |
| 1.37 | 2026-08-15 | CS-P-006-C.3-E Search #2 discovered-rule persistence (no pass threshold; no Search #3) |
| 1.36 | 2026-08-15 | CS-P-006-C.3-D Search #2 live-rule ecology (no Search #3; not promoted) |
| 1.35 | 2026-08-15 | CS-P-006-C.3-C Search #1 vs #2 comparative review (no Search #3) |
| 1.34 | 2026-08-15 | CS-P-006-C.3-R one authorized Search #2 run (Search #1 immutable; no iteration) |
| 1.33 | 2026-08-15 | CS-P-006-C.3-I implementation and identity gate (Search #2 not run) |
| 1.32 | 2026-08-15 | CS-P-006-C.3 protocol authorized (Search #2 not started; same TMV; M.1 V) |
| 1.31 | 2026-08-15 | CS-P-006-N harness implemented (symbol matrices required; C.3 not authorized) |
| 1.30 | 2026-08-15 | CS-P-006-M.1 decision-value specification + CS-P-006-N harness spec (no Search #2) |
| 1.29 | 2026-08-15 | CS-P-006-M decision-value model (protocol only; no Search #2; advantage is not fitness) |
| 1.28 | 2026-08-15 | CS-P-006-C.2-D decision-value landscape (no Search #2; no invented bands) |
| 1.27 | 2026-08-15 | CS-P-006-C.2-S selection and decision-value review (no Search #2) |
| 1.26 | 2026-08-15 | CS-P-006-C.2-R Search #1 recommendation-vs-outcome matrix (no Search #2) |
| 1.25 | 2026-08-15 | CS-P-006-C.2-P Search #1 population ecology (no Search #2; C.3 not authorized) |
| 1.24 | 2026-08-15 | CS-P-006-C.2-O search observability (no Search #2; artifact identity unchanged) |
| 1.23 | 2026-08-15 | CS-P-006-C.2 research-gap review (observability + volatility presence; no Search #2) |
| 1.22 | 2026-08-15 | CS-P-006-C.1 Search #1 diagnosis (no second search; no promotion) |
| 1.21 | 2026-08-14 | CS-P-006-C first Coralys TMV discovery (sealed PolicyArtifact; evaluation not fed back) |
| 1.20 | 2026-08-14 | CS-P-006-V decision-value vision (future certified families; not 006-C genome) |
| 1.19 | 2026-08-14 | CS-P-006-B.1 chronological partition freeze (39 timestamps; Coralys authorized) |
| 1.18 | 2026-08-14 | CS-P-006-S1 certified PASS (7-name research snapshot; dates still not frozen) |
| 1.17 | 2026-08-14 | Index CS-P-006-S1 7-instrument research snapshot (not B4/B5; no date freeze) |
| 1.16 | 2026-08-14 | Index CS-P-006-B research protocol (7-name universe; split dates not frozen) |
| 1.15 | 2026-08-14 | Index CS-P-006 / CS-P-006-A Policy Discovery Contract (no optimizer; no split dates) |
| 1.14 | 2026-08-14 | Index CS-P-CLEAN-002 explicit DecisionPolicy contract (PR-2; no CS-P-006) |
| 1.13 | 2026-08-14 | Index CS-P-CLEAN-001 adapter quarantine (PR-1; no B4 regeneration) |
| 1.12 | 2026-08-14 | Index CS-P-AUDIT-001 adapter discipline audit (read-only; no cleanup) |
| 1.11 | 2026-08-14 | Index CS-P-TEST-001 verification matrix and CS-P-005 Factor Ecology (no candidate policy) |
| 1.10 | 2026-08-14 | Index CS-P-004-E1-S1 information-fidelity snapshot (not B5; factor availability only) |
| 1.9 | 2026-08-14 | Index CS-P-004-E1 Assessment Enrichment v0.1 (code/tests only; no new dataset) |
| 1.8 | 2026-08-14 | Index CS-P-004-A1 adapter information-fidelity enhancement |
| 1.7 | 2026-08-14 | Index CS-P-004-R1 B4 historical laboratory reports |
| 1.6 | 2026-08-14 | Index CS-P-004 Historical Research Laboratory; CS-P-003 remains confirmation clock |
| 1.5 | 2026-08-14 | Index CS-P-003 Forward/Paper Validation v0.1 |
| 1.4 | 2026-08-14 | Index CS-P-002-R1 B4 historical product validation (`unfrozen-dev`) |
| 1.3 | 2026-08-14 | Index CS-P-002 Decision Validation Platform |
| 1.2 | 2026-08-14 | Index CS-P-001 product mode; PRD-002 |
| 1.1 | 2026-08-14 | Index EV-GOV-003 G-GATE v1.1 statistical closure |
| 1.0 | 2026-08-01 | Initial creation — Repository v2 Governance Artifact 2 |