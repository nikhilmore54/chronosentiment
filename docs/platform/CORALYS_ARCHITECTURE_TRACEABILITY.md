# Coralys Platform — Architecture Traceability

**Document type:** Architecture Traceability
**Version:** 2.0
**Status:** Updated — EP-001 resolutions applied
**Date:** 2026-07-27
**Owner:** Platform / Engineering

---

## Document Governance

| Field | Value |
|-------|-------|
| Document Status | v2.0 — EP-001 post-sprint update |
| Previous Version | v1.0 Baseline (2026-07-26) |
| Review Trigger | New crate added; primitive implementation status changes; platform architecture revision |

**Relationship to other documents:**
- Informed by: `CORALYS_PLATFORM_ARCHITECTURE.md` (platform architecture — primitives, lifecycle, Continuous Learning Engine)
- Governed by: `../strategy/CS-S-001_Product_First_Governance_Principle.md` (platform promotion criterion — authoritative)
- Informs: `CORALYS_PRODUCT_TRACEABILITY.md` (product capability → crate mapping)
- Informs: `CORALYS_GAP_REGISTER.md` (implementation gap register)

---

## Purpose

This document maps every Coralys platform primitive defined in `CORALYS_PLATFORM_ARCHITECTURE.md` to its Rust implementation in the codebase. It records implementation status for each primitive and identifies the crate and type responsible for each.

This is a living engineering reference. It should be updated whenever a primitive moves from Planned to Partial to Implemented.

---

## Implementation Status Legend

| Status | Meaning |
|--------|---------|
| **Implemented** | Fully implemented in the codebase; production-ready |
| **Partial** | Partially implemented; core structure exists but capabilities are incomplete |
| **Stub** | Type or trait defined but not implemented |
| **Planned** | Documented in architecture; not yet implemented |

---

## EP-001 Status Changes

| Primitive / Adapter | v1.0 Status | v2.0 Status | Change |
|--------------------|-------------|-------------|--------|
| Evidence | Partial | Implemented (adapter level) | ChronoSentiment `evidence.rs` — immutability enforced |
| Hypothesis | Partial | Implemented (adapter level) | ChronoSentiment `hypothesis.rs` — versioning implemented |
| Pattern | Partial | Partial (updated notes) | `PatternMaturity` lifecycle now in both adapters |
| ChronoSentiment Adapter | Stub | Implemented | 5 modules, 22 tests |
| UltraCrew Adapter | Implemented | Implemented (extended) | disruption_recovery + decision_intelligence added |

---

## Platform Primitive Traceability

> **Promotion criterion (CS-S-001, 2026-07-27):** "Implemented (adapter level)" does not automatically qualify a primitive for promotion to `coralys-core`. Promotion requires at least two independent products to have demonstrated the same semantics. This table tracks implementation state; promotion decisions are governed by [`CS-S-001`](../strategy/CS-S-001_Product_First_Governance_Principle.md) and triggered by evidence in the Evidence Ledger.

### Workspace

**Architecture definition:** The transaction boundary for a single lifecycle instance. Everything in a lifecycle happens inside exactly one Workspace. Unit of provenance, archival, and access control.

| Aspect | Detail |
|--------|--------|
| Implementation status | **Partial** |
| Primary crate | `coralys-core` |
| Primary type | `Scenario` trait (`coralys-core/src/lib.rs:1`) |
| Notes | `Scenario` is the domain-neutral trait that corresponds to Workspace at the platform level. Domain adapters implement `Scenario` to define their Workspace type. The full Workspace lifecycle (Active → Completed → Archived) is not yet enforced at the platform level — it is managed within each adapter. EP-001: `InvestmentWorkspace` in `adapters/chronosentiment/src/workspace.rs` enforces the lifecycle and the single-Outcome invariant at the adapter level. |

---

### Actor

**Architecture definition:** The entity that performs actions within a Workspace — a person, team, or system.

| Aspect | Detail |
|--------|--------|
| Implementation status | **Partial** |
| Primary crate | `coralys-planning` |
| Primary type | `Worker` trait (`coralys-planning/src/lib.rs:28`) |
| Notes | `Worker` is the planning-domain realisation of Actor. The platform-level Actor primitive is not yet defined as a standalone trait in `coralys-core`. Actor identity is carried implicitly through `Worker::id()`. |

---

### Intent

**Architecture definition:** The purpose of a Workspace — what the Actor is trying to achieve.

| Aspect | Detail |
|--------|--------|
| Implementation status | **Stub** |
| Primary crate | `coralys-core` |
| Primary type | `DecisionProposal` (`coralys-core/src/models/decision_proposal.rs`) |
| Notes | `DecisionProposal` carries the intent of a decision step but is not a full Intent primitive. A standalone `Intent` trait at the platform level is not yet implemented. Intent is currently implicit in the `Scenario` configuration passed to each adapter. EP-001: `InvestmentWorkspace.research_objective` carries the Intent for the ChronoSentiment adapter — the invariant "one Intent per Workspace" is enforced at the adapter level. Platform-level promotion pending. |

---

### Subject

**Architecture definition:** The entity being reasoned about within a Workspace — the company, crew base, or scheduling period.

| Aspect | Detail |
|--------|--------|
| Implementation status | **Partial** |
| Primary crate | `coralys-planning` |
| Primary type | `PlanningScenario` trait (`coralys-planning/src/lib.rs:87`) |
| Notes | `PlanningScenario` encapsulates the Subject (the scheduling period, crew base, or operational unit) along with its workers, planning units, and coverage demands. A standalone `Subject` primitive at the platform level is not yet defined. |

---

### Context

**Architecture definition:** The environment in which the Subject exists — the fund mandate, operational environment, or regulatory context.

| Aspect | Detail |
|--------|--------|
| Implementation status | **Planned** |
| Primary crate | None |
| Primary type | None |
| Notes | Context is not yet implemented as a platform primitive. In the UltraCrew adapter, operational context (airport, network, constraints) is embedded in the `Scenario` configuration. A standalone `Context` primitive would allow context to be tracked, versioned, and queried independently. |

---

### Evidence

**Architecture definition:** Immutable records that inform the Hypothesis. Evidence is immutable once recorded — no adapter may mutate historical Evidence.

| Aspect | Detail |
|--------|--------|
| Implementation status | **Implemented (adapter level)** |
| Primary crate | `adapters/chronosentiment` (adapter); `coralys-core` (platform foundation) |
| Primary type | `EvidenceItem` (`adapters/chronosentiment/src/evidence.rs`); `EvaluationResult` (`coralys-core/src/models/evaluation_result.rs`) |
| Notes | EP-001: `EvidenceItem` in `adapters/chronosentiment/src/evidence.rs` is a full implementation of the Evidence primitive with immutability enforced — no mutation methods exist; `add_evidence` is append-only; superseded items are preserved with a forward reference. The platform invariant (Evidence cannot be mutated once recorded) is enforced at the adapter level. Platform-level promotion (a `coralys-core` Evidence trait that enforces immutability across all adapters) is pending. |

---

### Hypothesis

**Architecture definition:** Versioned statement of what the Actor believes and why. Hypotheses evolve through versioning.

| Aspect | Detail |
|--------|--------|
| Implementation status | **Implemented (adapter level)** |
| Primary crate | `adapters/chronosentiment` (adapter); `coralys-core` (platform foundation) |
| Primary type | `InvestmentThesis` (`adapters/chronosentiment/src/hypothesis.rs`); `DecisionProposal` (`coralys-core/src/models/decision_proposal.rs`) |
| Notes | EP-001: `InvestmentThesis` in `adapters/chronosentiment/src/hypothesis.rs` is a full implementation of the Hypothesis primitive with versioning — `add_thesis_version` creates a new version; previous versions are never modified; all versions are preserved. The platform invariant (versions immutable once created) is enforced at the adapter level. Platform-level promotion (a `coralys-core` Hypothesis trait with versioning) is pending. |

---

### Review

**Architecture definition:** Structured periodic comparison of Hypothesis against Evidence.

| Aspect | Detail |
|--------|--------|
| Implementation status | **Partial** |
| Primary crate | `coralys-core` |
| Primary type | `DecisionPlugin::evaluate()` (`coralys-core/src/lib.rs:38`) |
| Notes | `DecisionPlugin::evaluate()` performs the review step — comparing the current state against the proposal. A structured Review primitive with documented outcomes, attendees, and conditions is not yet implemented at the platform level. EP-001: `ThesisReview` in `adapters/chronosentiment/src/hypothesis.rs` is a structured review record with verdict, reviewer, and notes — an adapter-level realisation of the Review primitive. |

---

### Timeline

**Architecture definition:** Complete record of how the Hypothesis evolved. Every state transition is recorded.

| Aspect | Detail |
|--------|--------|
| Implementation status | **Implemented** |
| Primary crate | `coralys-core` |
| Primary type | `DecisionLineage` (`coralys-core/src/models/decision_lineage.rs:16`) |
| Notes | `DecisionLineage` is a full implementation of the Timeline primitive. It maintains a tree of `LineageNode` entries, each with a parent reference, a `DecisionProposal`, and an `EvaluationResult`. The `root_id` and `current_id` fields allow traversal of the full decision history. This is the most complete primitive implementation in the platform. EP-001: `TimelineEvent` and `TimelineView` in `adapters/chronosentiment/src/timeline.rs` provide a domain-specific Timeline with 15 event kinds, filtering, and narrative generation. |

---

### Outcome

**Architecture definition:** What actually happened; raw material for Learning. Every Outcome belongs to exactly one Workspace.

| Aspect | Detail |
|--------|--------|
| Implementation status | **Implemented** |
| Primary crate | `coralys-core` |
| Primary type | `Outcome` trait (`coralys-core/src/lib.rs:5`) |
| Notes | `Outcome` is fully implemented as a platform trait. It provides `objectives()` (multi-objective fitness values), `primary_objective()`, `is_valid()`, and `solution()`. The platform invariant (every Outcome belongs to exactly one Workspace) is not yet enforced at the platform level. EP-001: `InvestmentOutcome` in `adapters/chronosentiment/src/workspace.rs` enforces the single-Outcome invariant at the adapter level — `record_outcome` returns `Err` if called twice. |

---

### Learning

**Architecture definition:** Analyses Outcomes; extracts Patterns; does not store. Learning computes — it never mutates historical Evidence.

| Aspect | Detail |
|--------|--------|
| Implementation status | **Implemented** |
| Primary crate | `coralys-core` |
| Primary type | `InnovationTracker` (`coralys-core/src/memory.rs:14`) |
| Notes | `InnovationTracker` is a full implementation of the Learning primitive. It observes solution signatures across generations, tracks novel discoveries, persistence, rediscovery, and extinction. It produces `InnovationTelemetry` — a structured learning output. It does not store solutions — it stores signatures (hashes). The platform invariant (Learning never mutates historical Evidence) is satisfied by design. EP-001: `OperationalLearningLoop` (UltraCrew) and `PersonalInvestmentLearningLoop` (ChronoSentiment) are adapter-level Learning implementations with full PatternMaturity lifecycle. |

---

### Pattern

**Architecture definition:** Cross-Workspace knowledge extracted by Learning. Patterns are generalised from Outcomes.

| Aspect | Detail |
|--------|--------|
| Implementation status | **Partial** |
| Primary crate | `coralys-ecology` |
| Primary type | `MemoryModel` trait (`coralys-ecology/src/traits.rs:7`) |
| Notes | `MemoryModel` provides the observe/state interface for pattern accumulation. `TopologyModel` provides the transformation interface for pattern extraction. EP-001: `PatternMaturity` lifecycle (Candidate → Observed → Repeated → Validated) is now implemented in both `adapters/ultracrew/src/decision_intelligence.rs` (`WorkforcePattern`) and `adapters/chronosentiment/src/learning.rs` (`InvestmentPattern`). Platform-level Pattern primitive (with persistence and cross-Workspace accumulation) is pending. |

---

### Knowledge Graph

**Architecture definition:** Stores Patterns and links; queryable by future Workspaces. The Knowledge Graph stores but does not infer without traceability.

| Aspect | Detail |
|--------|--------|
| Implementation status | **Partial** |
| Primary crate | `coralys-ecology` |
| Primary type | `MemoryModel` + `TopologyModel` (`coralys-ecology/src/traits.rs`) |
| Notes | The ecology crate provides the foundational traits for the Knowledge Graph — `MemoryModel` (observe/state) and `TopologyModel` (transform). A full Knowledge Graph with persistence, traversal, semantic retrieval, and contextual enrichment (as described in Architecture Observation 7) is not yet implemented. The current implementation is a trait-level foundation. |

---

## Continuous Learning Engine Traceability

**Architecture definition:** The core computational subsystem of the Coralys platform. Orchestrates the lifecycle. Drives improvement. Does not own primitives — it operates on them.

| Aspect | Detail |
|--------|--------|
| Implementation status | **Implemented** |
| Primary crate | `coralys-moga` |
| Primary type | `MogaReasoningEngine` / `EvolutionEngineBuilder` (`coralys-moga/src/lib.rs`) |
| Notes | The MOGA engine is a full implementation of the Continuous Learning Engine for the planning domain. It provides: `EvolutionEngineBuilder` (configurable pipeline), `MogaReasoningEngine` (the engine itself), `FitnessEvaluator` (evaluation), `MutationOperator` + `CrossoverOperator` (evolution), `SelectionStrategy` (selection), `ImprovementOperator` (local search), `EliteArchive` (elite preservation), `TerminationPolicy` (termination), `PipelineObserver` + `ProcessingMetricsCollector` (observability), `ConstraintChecker` + `RepairHeuristic` (constraint enforcement). The engine is domain-neutral — it operates on any `Genome` type. |

---

## Domain Adapter Traceability

### UltraCrew Adapter (`adapters/ultracrew`)

| Aspect | Detail |
|--------|--------|
| Implementation status | **Implemented** |
| Coralys primitives realised | Workspace (Scheduling Workspace), Actor (Scheduler), Intent (Scheduling objective), Subject (Scheduling period / crew base), Evidence (Operational data, disruption records), Hypothesis (Roster Strategy), Timeline (Scheduling Timeline), Outcome (Operational KPIs), Learning (Workforce Operations Learning Loop), Pattern (Workforce Behaviour Pattern with PatternMaturity lifecycle) |
| Key modules | `constraint_engine`, `decision_intelligence`, `disruption_recovery`, `ecology`, `optimization`, `pipeline`, `recommendation`, `schedule_solution`, `public_contracts` |
| INRC2 implementation | `inrc/evaluator`, `inrc/schedule_optimizer`, `inrc/validator`, `inrc/parser`, `inrc/history`, `inrc/audit`, `inrc/baseline`, `inrc/bipartite_matching` |
| Benchmark evidence | Ablation matrices (30-seed), survival curves, extinction curves, horizon tests, alpha sweeps |
| EP-001 additions | `disruption_recovery.rs` — 5-step disruption workflow, 4 tests; `decision_intelligence.rs` — OperationalLearningLoop, PatternMaturity lifecycle, CycleReviewReport, 4 tests |

---

### Airline Adapter (`adapters/airline`)

| Aspect | Detail |
|--------|--------|
| Implementation status | **Implemented** |
| Coralys primitives realised | Workspace (Crew scheduling cycle), Actor (Crew scheduler), Subject (Route / crew base), Evidence (Flight data, crew availability), Hypothesis (Crew roster), Outcome (Coverage and compliance KPIs) |
| Key modules | `domain` (crew, duty, flight, pairing, roster, rotation), `legality` (FDP, duty time, minimum rest, base return, qualification, coverage), `resilience` (disruption, reserve, robustness) |
| Benchmark evidence | Scalability tests, solution quality benchmarks, robustness tests |

---

### ChronoSentiment Adapter (`adapters/chronosentiment`)

| Aspect | Detail |
|--------|--------|
| Implementation status | **Implemented** |
| Coralys primitives realised | Workspace (InvestmentWorkspace — transaction boundary, lifecycle, single-Outcome invariant), Evidence (EvidenceItem — immutable, append-only, EvidenceDossier), Hypothesis (InvestmentThesis — versioned, ThesisReview), Timeline (TimelineEvent, 15 event kinds, TimelineView with filtering and narrative), Outcome (InvestmentOutcome — immutable once recorded), Learning (PersonalInvestmentLearningLoop — PatternMaturity lifecycle, QuarterlyReviewReport), Pattern (InvestmentPattern — 6 types, PatternMaturity) |
| Key modules | `evidence`, `hypothesis`, `timeline`, `workspace`, `learning` |
| Test coverage | 22 tests (4 per module + 5 in workspace) |
| EP-001 note | Adapter moved from empty stub to full shared foundation for both ChronoSentiment Personal and Enterprise products. Enterprise-specific wiring (committee review, organisational learning loop, institutional KG) is pending. |

---

## Future Primitives Traceability

### Question *(future)*

| Aspect | Detail |
|--------|--------|
| Implementation status | **Planned** |
| Notes | Documented in Architecture as a candidate future primitive. Not yet implemented. |

### Pattern Extraction Engine *(future)*

| Aspect | Detail |
|--------|--------|
| Implementation status | **Planned** |
| Notes | Documented in Architecture as a candidate future engine. The `coralys-ecology` crate provides the foundational traits (`TopologyModel`, `MemoryModel`) that would underpin this engine. Full implementation (clustering, embeddings, similarity search, graph mining) is planned for v2. |