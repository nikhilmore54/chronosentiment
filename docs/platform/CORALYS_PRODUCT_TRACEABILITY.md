# Coralys Platform — Product Traceability

**Document type:** Product Traceability
**Version:** 1.0
**Status:** Baseline
**Date:** 2026-07-26
**Owner:** Platform / Engineering

---

## Document Governance

| Field | Value |
|-------|-------|
| Document Status | Baseline v1.0 |
| Review Trigger | Blueprint capability status changes; new crate or module added; product architecture revision |

**Relationship to other documents:**
- Informed by: `CORALYS_ARCHITECTURE_TRACEABILITY.md` (primitive → crate mapping)
- Informed by: `UC-B-001_UltraCrew_Blueprint_v1.0.md` (UltraCrew blueprint)
- Informed by: `CS-E-B-001_ChronoSentiment_Enterprise_Blueprint_v1.0.md` (Enterprise blueprint)
- Informed by: `CS-P-B-001_ChronoSentiment_Personal_Blueprint_v1.0.md` (Personal blueprint)
- Informs: `CORALYS_GAP_REGISTER.md` (implementation gap register)

---

## Purpose

This document maps every blueprint capability for each product to its implementing crate and module. It turns the product blueprints into living engineering references — showing exactly where each capability is implemented, partially implemented, or not yet started.

---

## Implementation Status Legend

| Status | Meaning |
|--------|---------|
| **Implemented** | Fully implemented; production-ready |
| **Partial** | Core structure exists; capabilities incomplete |
| **Stub** | Type or module defined but not implemented |
| **Planned** | Documented in blueprint; not yet started |

---

## UltraCrew — Workforce Decision Engine

### Scheduling Workspace

| Capability | Status | Crate | Module / Type |
|------------|--------|-------|--------------|
| Workspace creation and lifecycle | **Partial** | `coralys-core` | `Scenario` trait |
| Evidence capture (operational data) | **Implemented** | `adapters/ultracrew` | `generic_import`, `inrc/parser` |
| Roster Strategy management | **Implemented** | `adapters/ultracrew` | `optimization`, `inrc/schedule_optimizer` |
| Workspace status transitions | **Planned** | — | — |
| Workspace archival | **Planned** | — | — |

---

### Roster Strategy (Hypothesis)

| Capability | Status | Crate | Module / Type |
|------------|--------|-------|--------------|
| Strategy creation | **Implemented** | `adapters/ultracrew` | `optimization`, `inrc/optimization` |
| Optimisation profile configuration | **Implemented** | `adapters/ultracrew` | `config/optimization_profiles`, `config/optimizer_config` |
| Roster version generation | **Implemented** | `adapters/ultracrew` | `inrc/schedule_optimizer`, `schedule_solution` |
| Strategy status tracking | **Partial** | `adapters/ultracrew` | `models` |
| Strategy outcome linkage | **Partial** | `adapters/ultracrew` | `decision_intelligence` |

---

### Optimisation Engine (Continuous Learning Engine)

| Capability | Status | Crate | Module / Type |
|------------|--------|-------|--------------|
| Multi-objective genetic algorithm | **Implemented** | `coralys-moga` | `engine::MogaReasoningEngine` |
| Elite archive | **Implemented** | `coralys-moga` | `state::EliteArchive` |
| Configurable termination | **Implemented** | `coralys-moga` | `termination::TerminationPolicy` |
| Evolution metrics | **Implemented** | `coralys-moga` | `metrics::EvolutionMetrics` |
| Processor metrics | **Implemented** | `coralys-moga` | `metrics::ProcessorMetrics` |
| Pluggable improvement operators | **Implemented** | `coralys-moga` | `traits::ImprovementOperator` |
| Local search | **Implemented** | `coralys-moga` | `traits::LocalSearchOperator` |
| Constraint repair | **Implemented** | `coralys-moga` | `repair::FeasibilityRepairFramework` |
| Pipeline observability | **Implemented** | `coralys-moga` | `observatory::PipelineObserver` |

---

### Constraint Engine

| Capability | Status | Crate | Module / Type |
|------------|--------|-------|--------------|
| Hard constraint enforcement | **Implemented** | `adapters/ultracrew` | `constraint_engine` |
| Constraint violation detection | **Implemented** | `coralys-moga` | `repair::ConstraintChecker` |
| Constraint repair heuristics | **Implemented** | `coralys-moga` | `repair::RepairHeuristic` |
| INRC2 constraint evaluation | **Implemented** | `adapters/ultracrew` | `inrc/evaluator`, `inrc/validator` |
| Airline legality rules (FDP) | **Implemented** | `adapters/airline` | `legality/fdp` |
| Airline legality rules (duty time) | **Implemented** | `adapters/airline` | `legality/duty_time` |
| Airline legality rules (minimum rest) | **Implemented** | `adapters/airline` | `legality/minimum_rest` |
| Airline legality rules (base return) | **Implemented** | `adapters/airline` | `legality/base_return` |
| Airline legality rules (qualification) | **Implemented** | `adapters/airline` | `legality/qualification` |
| Airline legality rules (coverage) | **Implemented** | `adapters/airline` | `legality/coverage` |

---

### Disruption Recovery

| Capability | Status | Crate | Module / Type |
|------------|--------|-------|--------------|
| Disruption modelling | **Implemented** | `adapters/airline` | `resilience/disruption` |
| Reserve crew management | **Implemented** | `adapters/airline` | `resilience/reserve` |
| Robustness scoring | **Implemented** | `adapters/airline` | `resilience/robustness` |
| Real-time re-optimisation workflow | **Planned** | — | — |
| Disruption evidence recording | **Planned** | — | — |
| Recovery option ranking | **Planned** | — | — |

---

### Scheduling Timeline

| Capability | Status | Crate | Module / Type |
|------------|--------|-------|--------------|
| Decision lineage tracking | **Implemented** | `coralys-core` | `models::DecisionLineage` |
| Timeline item types | **Partial** | `adapters/ultracrew` | `decision_intelligence` |
| Timeline filtering | **Planned** | — | — |
| Timeline audit export | **Partial** | `adapters/ultracrew` | `inrc/audit`, `generic_export` |

---

### Workforce Operations Learning Loop

| Capability | Status | Crate | Module / Type |
|------------|--------|-------|--------------|
| Innovation tracking | **Implemented** | `coralys-core` | `memory::InnovationTracker` |
| Ecology-aware optimisation | **Implemented** | `adapters/ultracrew` | `ecology` |
| Pattern accumulation | **Partial** | `coralys-ecology` | `traits::MemoryModel` |
| Learning loop workflow | **Planned** | — | — |
| Cycle review report | **Planned** | — | — |

---

### Operational Knowledge Graph

| Capability | Status | Crate | Module / Type |
|------------|--------|-------|--------------|
| Memory model (observe/state) | **Partial** | `coralys-ecology` | `traits::MemoryModel` |
| Topology model (transform) | **Partial** | `coralys-ecology` | `traits::TopologyModel` |
| Persistence | **Planned** | — | — |
| Traversal | **Planned** | — | — |
| Semantic retrieval | **Planned** | — | — |
| Contextual enrichment | **Planned** | — | — |

---

### CLI and Pipeline

| Capability | Status | Crate | Module / Type |
|------------|--------|-------|--------------|
| CLI interface | **Implemented** | `adapters/ultracrew` | `bin/ultracrew-cli` |
| Optimisation pipeline | **Implemented** | `adapters/ultracrew` | `pipeline` |
| Passive telemetry | **Implemented** | `adapters/ultracrew` | `bin/m30_0b_passive_telemetry` |
| Engagement audit | **Implemented** | `adapters/ultracrew` | `bin/m31_2a_engagement_audit` |
| Benchmark framework | **Implemented** | `adapters/ultracrew` | `bin/m31_benchmarks`, `inrc/baseline` |
| Health monitoring | **Implemented** | `adapters/ultracrew` | `health` |
| Public contracts | **Implemented** | `adapters/ultracrew` | `public_contracts` |

---

## ChronoSentiment Enterprise — Financial Decision Intelligence Platform

**Overall adapter status:** Stub (`adapters/chronosentiment/src/lib.rs` is empty)

All ChronoSentiment Enterprise capabilities are documented in the blueprint but not yet implemented in the adapter. The Coralys platform provides the lifecycle infrastructure; the domain adapter is the missing layer.

| Capability | Status | Crate | Module / Type |
|------------|--------|-------|--------------|
| Decision Workspace | **Planned** | `adapters/chronosentiment` | — |
| Investment Thesis with versioning | **Planned** | `adapters/chronosentiment` | — |
| Evidence management | **Planned** | `adapters/chronosentiment` | — |
| Committee Review workflow | **Planned** | `adapters/chronosentiment` | — |
| Decision Timeline | **Planned** | `coralys-core` (foundation) | `models::DecisionLineage` (reusable) |
| Decision Outcome recording | **Planned** | `adapters/chronosentiment` | — |
| Organisational Decision Learning Loop | **Planned** | `adapters/chronosentiment` | — |
| Institutional Decision Knowledge Graph | **Planned** | `coralys-ecology` (foundation) | `traits::MemoryModel` (reusable) |
| AI conversation documentation | **Planned** | `adapters/chronosentiment` | — |

**Platform foundations reusable for ChronoSentiment Enterprise:**
- `coralys-core::DecisionLineage` → Decision Timeline
- `coralys-core::InnovationTracker` → Organisational Decision Learning Loop (foundation)
- `coralys-ecology::MemoryModel` → Institutional Decision Knowledge Graph (foundation)
- `coralys-decision::CandidateEvaluator`, `DecisionMaker`, `DecisionPolicy` → Decision evaluation stubs

---

## ChronoSentiment Personal — Personal Investment Knowledge Platform

**Overall adapter status:** Stub (shares `adapters/chronosentiment` with Enterprise)

All ChronoSentiment Personal capabilities are documented in the blueprint but not yet implemented. The Personal product shares the same adapter stub as Enterprise.

| Capability | Status | Crate | Module / Type |
|------------|--------|-------|--------------|
| Research Workspace | **Planned** | `adapters/chronosentiment` | — |
| Research Dossier | **Planned** | `adapters/chronosentiment` | — |
| Investment Thesis with versioning | **Planned** | `adapters/chronosentiment` | — |
| Research Timeline | **Planned** | `coralys-core` (foundation) | `models::DecisionLineage` (reusable) |
| Quarterly Research Review | **Planned** | `adapters/chronosentiment` | — |
| Investment Outcome recording | **Planned** | `adapters/chronosentiment` | — |
| Personal Investment Learning Loop | **Planned** | `adapters/chronosentiment` | — |
| Personal Investment Knowledge Graph | **Planned** | `coralys-ecology` (foundation) | `traits::MemoryModel` (reusable) |
| AI conversation documentation | **Planned** | `adapters/chronosentiment` | — |

---

## Cross-Product Platform Reuse Summary

The following platform capabilities are implemented once and reused across all products:

| Platform capability | Crate | Type | Products |
|--------------------|-------|------|---------|
| Multi-objective optimisation | `coralys-moga` | `MogaReasoningEngine` | UltraCrew (implemented); Enterprise/Personal (planned) |
| Decision lineage / Timeline | `coralys-core` | `DecisionLineage` | UltraCrew (partial); Enterprise/Personal (planned) |
| Innovation tracking / Learning | `coralys-core` | `InnovationTracker` | UltraCrew (implemented); Enterprise/Personal (planned) |
| Memory model / Knowledge Graph | `coralys-ecology` | `MemoryModel` | UltraCrew (partial); Enterprise/Personal (planned) |
| Constraint enforcement | `coralys-moga` | `ConstraintChecker` | UltraCrew (implemented); Enterprise/Personal (planned) |
| Outcome evaluation | `coralys-core` | `Outcome` trait | UltraCrew (implemented); Enterprise/Personal (planned) |

---

*Coralys Platform Product Traceability v1.0 | July 2026 | Status: Baseline*
*Maps every blueprint capability to its implementing crate and module.*
*Review trigger: Blueprint capability status changes; new crate or module added; product architecture revision.*