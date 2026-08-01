# Repository Knowledge Survey

**Document ID:** GOV-KS-001
**Version:** 1.0
**Status:** Active
**Created:** 2026-08-01
**Scope:** Full repository asset inventory — all five knowledge systems

---

## Purpose

This survey is the authoritative, typed inventory of every canonical asset in the repository. It supersedes `docs/REPOSITORY_SURVEY.md` (which covered documents only). Every asset — document, code module, test, experiment, harness, benchmark, dataset, evidence file, or governance artifact — is registered here.

**Rule:** If an asset is not in this survey, it is not canonical.

---

## Asset Type Taxonomy

| Code | Type | Description |
|------|------|-------------|
| **CONST** | Constitution | Foundational rules and axioms governing the research programme |
| **CONTRACT** | Contract | Formal interface or behaviour contracts between components |
| **STANDARD** | Standard | Methodology or quality standards |
| **RESEARCH** | Research Paper | Formal research documents (CS-R-xxx series) |
| **EXPERIMENT** | Experiment Spec | Experiment specifications and plans |
| **HARNESS** | Harness | Experimental infrastructure (schema, logging, persistence, report, reproducibility) |
| **BENCHMARK** | Benchmark | Benchmark adapter and runner |
| **EVIDENCE** | Evidence | Runtime logs, CSV, JSON, reports produced by experiments |
| **CRATE** | Crate | Rust library crate |
| **ADAPTER** | Adapter | Domain adapter (airline, roadef, chronosentiment) |
| **SERVICE** | Service | Deployed service |
| **APP** | Application | End-user application |
| **TEST** | Test | Test file (unit, integration, e2e) |
| **GOV** | Governance | Governance artifacts (this survey, index, dependency graph, cleanup register) |
| **VIZ** | Visualisation | HTML maps, dashboards |
| **BLUEPRINT** | Blueprint | Product or architecture blueprints |
| **STRATEGY** | Strategy | Commercial and product strategy documents |

---

## Knowledge System 1 — Documentation Governance

### Constitutions

| ID | Path | Title | Status |
|----|------|-------|--------|
| CONST-001 | `docs/research/CHRONOLOGY_AXIOMS_CONTRACT_v1.md` | Chronology Axioms | Active |
| CONST-002 | `docs/research/ARCHIVE_CURATION_CONTRACT_v1.md` | Archive Curation | Active |
| CONST-003 | `docs/research/GENERALIZATION_BOUNDARY_CONTRACT_v1.md` | Generalisation Boundary | Active |

### Contracts

| ID | Path | Title | Status |
|----|------|-------|--------|
| CONTRACT-001 | `docs/research/CRYPTO_SUBSTRATE_CONTRACT_v1.md` | Crypto Substrate | Active |
| CONTRACT-002 | `docs/research/ECOLOGICAL_SURVIVABILITY_SURFACE_SPEC_v1.md` | Ecological Survivability Surface | Active |
| CONTRACT-003 | `docs/research/ECOLOGY_COMPARISON_PROTOCOL_v1.md` | Ecology Comparison Protocol | Active |
| CONTRACT-004 | `docs/research/ECONOMIC_SEMANTICS_CONTRACT_v1.md` | Economic Semantics | Active |
| CONTRACT-005 | `docs/research/EXECUTION_ECOLOGY_SPEC_v1.md` | Execution Ecology | Active |
| CONTRACT-006 | `docs/research/LIVE_CAPTURE_ISOLATION_CONTRACT_v1.md` | Live Capture Isolation | Active |
| CONTRACT-007 | `docs/research/METROLOGY_LAYER_CONTRACT_v1.md` | Metrology Layer | Active |
| CONTRACT-008 | `docs/research/MORPHOLOGY_RESPONSE_ISOLATION_CONTRACT_v1.md` | Morphology Response Isolation | Active |
| CONTRACT-009 | `docs/research/OBSERVABILITY_SEMANTICS_CONTRACT_v1.md` | Observability Semantics | Active |
| CONTRACT-010 | `docs/research/OSCILLATORY_TOPOLOGY_DEFINITION_CONTRACT_v1.md` | Oscillatory Topology Definition | Active |
| CONTRACT-011 | `docs/research/PERSISTENCE_SEMANTICS_CONTRACT_v1.md` | Persistence Semantics | Active |
| CONTRACT-012 | `docs/research/PNL_ATTRIBUTION_CONTRACT_v1.md` | P&L Attribution | Active |
| CONTRACT-013 | `docs/research/REPAIR_LABORATORY_ISOLATION_PROTOCOL_v1.md` | Repair Laboratory Isolation | Active |
| CONTRACT-014 | `docs/research/REPAIR_SEMANTICS_CONTRACT_v1.md` | Repair Semantics | Active |
| CONTRACT-015 | `docs/research/REPLAY_EQUIVALENCE_CONTRACT_v1.md` | Replay Equivalence | Active |
| CONTRACT-016 | `docs/research/REPLAY_MANIFEST_SPECIFICATION_v1.md` | Replay Manifest | Active |
| CONTRACT-017 | `docs/research/RUST_PORT_CONTRACT_v1.md` | Rust Port | Active |
| CONTRACT-018 | `docs/research/SIGNAL_INTERFACE_CONTRACT_v1.md` | Signal Interface | Active |
| CONTRACT-019 | `docs/research/STATE_COHERENCE_CONTRACT_v1.md` | State Coherence | Active |
| CONTRACT-020 | `docs/research/SURFACE_HASH_CONTRACT_v1.md` | Surface Hash | Active |
| CONTRACT-021 | `docs/research/SURFACE_INTERPRETATION_CONTRACT_v1.md` | Surface Interpretation | Active |
| CONTRACT-022 | `docs/research/TOPOLOGY_NEUTRALITY_CONTRACT_v1.md` | Topology Neutrality | Active |
| CONTRACT-023 | `docs/research/TOPOLOGY_PERTURBATION_CONTRACT_v1.md` | Topology Perturbation | Active |

### Standards

| ID | Path | Title | Status |
|----|------|-------|--------|
| STANDARD-001 | `docs/CORALYS_EVIDENCE_GOVERNANCE_STANDARD.md` | Coralys Evidence Governance | Active |
| STANDARD-002 | `docs/BENCHMARK-GOVERNANCE.md` | Benchmark Governance | Active |
| STANDARD-003 | `docs/BENCHMARK-REFERENCE-SPECIFICATION-v1.0.md` | Benchmark Reference Specification | Active |

---

## Knowledge System 2 — Platform Architecture

### Coralys Crates

| ID | Path | Role | Status |
|----|------|------|--------|
| CRATE-001 | `coralys-core/` | Core EA engine | Active |
| CRATE-002 | `coralys-decision/` | Decision layer | Active |
| CRATE-003 | `coralys-ecology/` | Ecology / population dynamics | Active |
| CRATE-004 | `coralys-eval/` | Fitness evaluation | Active |
| CRATE-005 | `coralys-infrastructure/` | Shared infrastructure | Active |
| CRATE-006 | `coralys-matching/` | Matching algorithms | Active |
| CRATE-007 | `coralys-moga/` | Multi-objective GA | Active |
| CRATE-008 | `coralys-planning/` | Planning layer | Active |
| CRATE-009 | `coralys-policy/` | Policy layer | Active |
| CRATE-010 | `coralys-recommendation/` | Recommendation engine | Active |
| CRATE-011 | `coralys-simulation/` | Simulation layer | Active |
| CRATE-012 | `coralys-v2/` | Coralys v2 (next generation) | Active |

### Adapters

| ID | Path | Domain | Status |
|----|------|--------|--------|
| ADAPTER-001 | `adapters/airline/` | Airline crew pairing (UltraCrew / GERAD) | Active — canonical research adapter |
| ADAPTER-002 | `adapters/chronosentiment/` | ChronoSentiment financial signals | Active |
| ADAPTER-003 | `adapters/roadef/` | ROADEF 2026 challenge | Active |

### Services

| ID | Path | Role | Status |
|----|------|------|--------|
| SERVICE-001 | `services/ultracrew_server/` | UltraCrew HTTP server | Active |

### Applications

| ID | Path | Role | Status |
|----|------|------|--------|
| APP-001 | `apps/ultracrew-pilot-portal/` | UltraCrew pilot portal (React) | Active |

### Architecture Documents

| ID | Path | Title | Status |
|----|------|-------|--------|
| ARCH-001 | `docs/CORALYS_PLATFORM_ARCHITECTURE.md` | Coralys Platform Architecture | Active |
| ARCH-002 | `docs/CORALYS_PLATFORM_STRATEGY.md` | Coralys Platform Strategy | Active |
| ARCH-003 | `docs/PLATFORM_CRATE_RESPONSIBILITIES.md` | Platform Crate Responsibilities | Active |
| ARCH-004 | `docs/ARCHITECTURE_EVOLUTION.md` | Architecture Evolution | Active |
| ARCH-005 | `docs/ARCHITECTURE_GLOSSARY.md` | Architecture Glossary | Active |
| ARCH-006 | `docs/CODEBASE_ARCHITECTURE_ASSESSMENT.md` | Codebase Architecture Assessment | Active |
| ARCH-007 | `docs/Backend_Architecture_Blueprint.md` | Backend Architecture Blueprint | Active |
| ARCH-008 | `docs/Service_Boundary_Definition.md` | Service Boundary Definition | Active |
| ARCH-009 | `docs/Event_Flow_Specification.md` | Event Flow Specification | Active |

---

## Knowledge System 3 — Research Programme

### Research Papers (CS-R series)

| ID | Path | Title | Status | Frozen? |
|----|------|-------|--------|---------|
| RESEARCH-000 | `docs/research/CS-R-000_Research_Evidence_Sufficiency_Matrix.md` | Research Evidence Sufficiency Matrix | Active | No |
| RESEARCH-001 | `docs/research/CS-R-001_Market_Landscape.md` | Market Landscape | Active | No |
| RESEARCH-002 | `docs/research/CS-R-002_Competitive_Landscape.md` | Competitive Landscape | Active | No |
| RESEARCH-003 | `docs/research/CS-R-003_Customer_Problem_Evidence.md` | Customer Problem Evidence | Active | No |
| RESEARCH-004 | `docs/research/CS-R-004_Regulatory_Landscape.md` | Regulatory Landscape | Active | No |
| RESEARCH-005 | `docs/research/CS-R-005_Pricing_Analysis.md` | Pricing Analysis | Active | No |
| RESEARCH-006 | `docs/research/CS-R-006_Data_Landscape.md` | Data Landscape | Active | No |
| RESEARCH-007 | `docs/research/CS-R-007_Explainability_Research.md` | Explainability Research | Active | No |
| RESEARCH-008 | `docs/research/CS-R-008_Point_In_Time_Architecture_Review.md` | Point-in-Time Architecture Review | Active | No |
| RESEARCH-009 | `docs/research/CS-R-009_AI_Adoption_Investment_Management.md` | AI Adoption & Investment Management | Active | No |
| RESEARCH-010 | `docs/research/CS-R-010_Investment_Workflow_Evolution.md` | Investment Workflow Evolution | Active | No |
| RESEARCH-011 | `docs/research/CS-R-011_Decision_Governance_Research.md` | Decision Governance Research | Active | No |
| RESEARCH-012 | `docs/research/CS-R-012_Build_vs_Buy_Analysis.md` | Build vs Buy Analysis | Active | No |
| RESEARCH-013 | `docs/research/CS-R-013_Technology_Readiness_Assessment.md` | Technology Readiness Assessment | Active | No |
| RESEARCH-014 | `docs/research/CS-R-014_Product_Category_Creation_Study.md` | Product Category Creation Study | Active | No |
| RESEARCH-015 | `docs/research/CS-R-015_Investment_Thesis.md` | Investment Thesis | Active | No |
| RESEARCH-015A | `docs/research/CS-R-015A_Executive_Investment_Summary.md` | Executive Investment Summary | Active | No |
| RESEARCH-016 | `docs/research/CS-R-016_Credit_Framework_Architecture.md` | Credit Framework Architecture | Active | No |

### UltraCrew Research Papers

| ID | Path | Title | Status | Frozen? |
|----|------|-------|--------|---------|
| UC-R-001 | `docs/research/UltraCrew_Pairing_Topology_Mutation_Evaluation.md` | Pairing Topology Mutation Evaluation | **Section 2 FROZEN 2026-08-01** | Section 2 yes |
| UC-R-002 | `docs/research/UltraCrew_Coralys_Native_Scheduler_Section3.md` | Coralys Native Scheduler — Section 3 | Active | No |
| UC-R-003 | `docs/research/UltraCrew_GENCOL_Pipeline_Divergence_Analysis.md` | GENCOL Pipeline Divergence Analysis | Active | No |
| UC-R-004 | `docs/research/UltraCrew_Layover_Threshold_Experiment.md` | Layover Threshold Experiment | Active | No |
| UC-R-005 | `docs/research/UltraCrew_Objective_Function_Alignment.md` | Objective Function Alignment | Active | No |

### Research Infrastructure

| ID | Path | Title | Status |
|----|------|-------|--------|
| RI-001 | `docs/research/RESEARCH_LOG.md` | Research Log | Active |
| RI-002 | `docs/research/RESEARCH_LINEAGE.md` | Research Lineage | Active (also at `docs/RESEARCH_LINEAGE.md`) |
| RI-003 | `docs/research/DISCREPANCY_REPORT.md` | Discrepancy Report | Active |
| RI-004 | `docs/research/HANDOFF.md` | Handoff Document | Active |
| RI-005 | `docs/research/README.md` | Research README | Active |

---

## Knowledge System 4 — Implementation

### Experiment Harness (ADAPTER-001 / airline)

| ID | Path | Role | Key Exports | Status |
|----|------|------|-------------|--------|
| HARNESS-001 | `adapters/airline/tests/harness/schema.rs` | Data types | `ExperimentConfig`, `GenerationRecord`, `RunSummary`, `MultiRunAggregate`, `ExperimentResult` | Stable |
| HARNESS-002 | `adapters/airline/tests/harness/logging.rs` | Structured logging | `EventLogger`, `ExperimentEvent` | Stable |
| HARNESS-003 | `adapters/airline/tests/harness/persistence.rs` | Output writers | `ResultPersistence` | Stable |
| HARNESS-004 | `adapters/airline/tests/harness/reproducibility.rs` | Reproducibility snapshot | `ReproducibilityInfo`, `capture()` | Stable |
| HARNESS-005 | `adapters/airline/tests/harness/report.rs` | Markdown report generation | `ReportGenerator` | Stable |
| HARNESS-006 | `adapters/airline/tests/harness/mod.rs` | Harness root | re-exports all harness types | Stable |

### Benchmarks

| ID | Path | Benchmark | Baseline | Status |
|----|------|-----------|----------|--------|
| BENCH-001 | `adapters/airline/tests/gerad_coralys.rs` | GERAD Coralys | v1.0 frozen 2026-08-01 | **Baseline Frozen** |
| BENCH-002 | `adapters/airline/tests/gerad_e2e.rs` | GERAD end-to-end | — | Active |
| BENCH-003 | `adapters/airline/tests/benchmark.rs` | General benchmark suite | — | Active |

### Tests

| ID | Path | Type | Covers | Status |
|----|------|------|--------|--------|
| TEST-001 | `adapters/airline/tests/robustness.rs` | Robustness | Airline adapter robustness | Active |
| TEST-002 | `adapters/airline/tests/scalability.rs` | Scalability | Airline adapter scalability | Active |
| TEST-003 | `adapters/airline/tests/scenario_validation.rs` | Scenario | Scenario validation | Active |
| TEST-004 | `adapters/airline/tests/solution_quality.rs` | Quality | Solution quality metrics | Active |

---

## Knowledge System 5 — Evidence

### Evidence Governance

| ID | Path | Title | Status |
|----|------|-------|--------|
| EV-GOV-001 | `docs/CHRONOSENTIMENT_EVIDENCE_PROGRAMME.md` | ChronoSentiment Evidence Programme | Active |
| EV-GOV-002 | `docs/INRC_PRODUCT_EVIDENCE_PROGRAMME.md` | INRC Product Evidence Programme | Active |
| EV-GOV-003 | `docs/ULTRACREW_WORKFORCE_EVIDENCE.md` | UltraCrew Workforce Evidence | Active |
| EV-GOV-004 | `docs/EL-001_Phase1B_Evidence_Ledger.md` | Phase 1B Evidence Ledger | Active |
| EV-GOV-005 | `docs/PX-001_Product_Evidence_Baseline.md` | Product Evidence Baseline | Active |

### Certification Artifacts

| ID | Path | Title | Status |
|----|------|-------|--------|
| CERT-001 | `docs/certification/PHASE6_EXPLAINABILITY_CERTIFICATION.json` | Phase 6 Explainability Certification | Certified |
| CERT-002 | `docs/certification/PHASE6_EXPLAINABILITY_TRACES.md` | Phase 6 Explainability Traces | Certified |
| CERT-003 | `docs/certification/PHASE7_FRAGILITY_MAPS.md` | Phase 7 Fragility Maps | Certified |
| CERT-004 | `docs/certification/PHASE8_PORTFOLIO_INVARIANCE_SURFACE.md` | Phase 8 Portfolio Invariance Surface | Certified |
| CERT-005 | `docs/certification/replay_certification_log.md` | Replay Certification Log | Active |
| CERT-006 | `docs/certification/sweep_projection_certification.md` | Sweep Projection Certification | Active |
| CERT-007 | `docs/certification/orchestration_execution_order_certification.md` | Orchestration Execution Order Certification | Active |
| CERT-008 | `docs/certification/PHASE6_5_CONSEQUENCE_REPORT.md` | Phase 6.5 Consequence Report | Active |

### Runtime Evidence (live)

| ID | Path | Type | Status |
|----|------|------|--------|
| EV-001 | `results/` | Experiment results (CSV, JSON) | Active — populated by harness |
| EV-002 | `logs/` | Runtime logs | Active |
| EV-003 | `pilot_sessions/` | Pilot session recordings | Active |
| EV-004 | `snapshots/` | System snapshots | Active |
| EV-005 | `reports/` | Generated reports | Active |

---

## Governance Assets

| ID | Path | Title | Status |
|----|------|-------|--------|
| GOV-KS-001 | `docs/governance/KNOWLEDGE_SURVEY.md` | Repository Knowledge Survey | Active (this document) |
| GOV-IDX-001 | `docs/INDEX.md` | Canonical Repository Index | Pending (Artifact 2) |
| GOV-DEP-001 | `docs/governance/DEPENDENCY_GRAPH.md` | Repository Dependency Graph | Pending (Artifact 3) |
| GOV-CLN-001 | `docs/governance/CLEANUP_REGISTER.md` | Cleanup Register | Pending (Artifact 4) |
| GOV-OLD-001 | `docs/REPOSITORY_SURVEY.md` | Repository Survey (superseded) | Superseded by GOV-KS-001 |
| GOV-VIZ-001 | `docs/code_map.html` | Code Map (Rust-only) | Needs update — does not cover research or governance nodes |

---

## Duplicate / Overlap Register

The following pairs of assets have overlapping scope and require consolidation (tracked in GOV-CLN-001):

| Item | Asset A | Asset B | Overlap | Recommended Action |
|------|---------|---------|---------|-------------------|
| DUP-001 | `docs/REPOSITORY_SURVEY.md` | `docs/governance/KNOWLEDGE_SURVEY.md` (this file) | Document-only vs full asset survey | Archive A, canonical = B |
| DUP-002 | `docs/RESEARCH_LINEAGE.md` | `docs/research/RESEARCH_LINEAGE.md` | Identical title, likely duplicate | Verify content, keep one |
| DUP-003 | `docs/ChronoSentiment_Product_Blueprint_v1.md` | `docs/ChronoSentiment_Personal_Blueprint_v1.md` | Blueprint variants | Verify scope difference, consolidate or rename |
| DUP-004 | `docs/CODEBASE_ARCHITECTURE_ASSESSMENT.md` | `docs/CODEBASE_ASSESSMENT.md` | Architecture assessment variants | Verify content, keep one |
| DUP-005 | `docs/EP-001_MILESTONE.md` | `docs/P001_MILESTONE.md` | Milestone documents | Verify if same milestone, consolidate |
| DUP-006 | Multiple `*_CONTRACT_v1.md` files | — | 23 contracts in `docs/research/` — may overlap with `docs/contracts/` | Survey `docs/contracts/` and cross-reference |

---

## Summary Statistics

| Type | Count | Notes |
|------|-------|-------|
| CONST | 3 | Constitutions |
| CONTRACT | 23 | All in `docs/research/` |
| STANDARD | 3 | Governance standards |
| RESEARCH | 18 | CS-R series (000–016) + 015A |
| UC-RESEARCH | 5 | UltraCrew research papers |
| HARNESS | 6 | All stable, airline adapter |
| BENCHMARK | 3 | GERAD Coralys frozen |
| TEST | 4 | Airline adapter tests |
| CRATE | 12 | Coralys platform crates |
| ADAPTER | 3 | airline, chronosentiment, roadef |
| SERVICE | 1 | ultracrew_server |
| APP | 1 | ultracrew-pilot-portal |
| ARCH | 9 | Architecture documents |
| EV-GOV | 5 | Evidence governance |
| CERT | 8 | Certification artifacts |
| EV-LIVE | 5 | Runtime evidence directories |
| GOV | 4 | Governance artifacts (1 active, 3 pending) |
| **Total** | **~113** | Excludes `docs/archive/` and subdirectory contents not yet surveyed |

---

## Unsurveyed Areas

The following directories exist but have not been fully inventoried in this survey. They should be surveyed in a future pass:

- `docs/archive/` — archived documents
- `docs/blueprints/` — blueprint variants
- `docs/capabilities/` — capability documents
- `docs/commercial/` — commercial documents
- `docs/constitution/` — constitution variants (may overlap with `docs/research/` contracts)
- `docs/contracts/` — contract variants (may overlap with `docs/research/` contracts)
- `docs/deployment/` — deployment documents
- `docs/ga/` — GA-specific documents
- `docs/governance/` — governance documents (partially surveyed)
- `docs/migrations/` — migration documents
- `docs/platform/` — platform documents
- `docs/releases/` — release documents
- `docs/risk/` — risk documents
- `docs/simulation/` — simulation documents
- `docs/strategy/` — strategy documents
- `docs/submission/` — submission documents
- `docs/testing/` — testing documents
- `research_experiments/` — research experiment scripts
- `benchmarks/` — benchmark data
- `data/` — data assets
- `validation/` — validation artifacts
- `claims/` — claims registry (provisional, replicated, falsified)