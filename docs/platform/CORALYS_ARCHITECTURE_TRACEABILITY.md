# Coralys Platform — Architecture Traceability

**Document type:** Architecture Traceability
**Version:** 1.0
**Status:** Baseline
**Date:** 2026-07-26
**Owner:** Platform / Engineering

---

## Document Governance

| Field | Value |
|-------|-------|
| Document Status | Baseline v1.0 |
| Review Trigger | New crate added; primitive implementation status changes; platform architecture revision |

**Relationship to other documents:**
- Informed by: `CORALYS_PLATFORM_ARCHITECTURE.md` (platform architecture — primitives, lifecycle, Continuous Learning Engine)
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

## Platform Primitive Traceability

### Workspace

**Architecture definition:** The transaction boundary for a single lifecycle instance. Everything in a lifecycle happens inside exactly one Workspace. Unit of provenance, archival, and access control.

| Aspect | Detail |
|--------|--------|
| Implementation status | **Partial** |
| Primary crate | `coralys-core` |
| Primary type | `Scenario` trait (`coralys-core/src/lib.rs:1`) |
| Notes | `Scenario` is the domain-neutral trait that corresponds to Workspace at the platform level. Domain adapters implement `Scenario` to define their Workspace type. The full Workspace lifecycle (Active → Completed → Archived) is not yet enforced at the platform level — it is managed within each adapter. |

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
| Notes | `DecisionProposal` carries the intent of a decision step but is not a full Intent primitive. A standalone `Intent` trait at the platform level is not yet implemented. Intent is currently implicit in the `Scenario` configuration passed to each adapter. |

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
| Implementation status | **Partial** |
| Primary crate | `coralys-core` |
| Primary type | `EvaluationResult` (`coralys-core/src/models/evaluation_result.rs`) |
| Notes | `EvaluationResult` captures the result of evaluating a candidate solution — this is the closest current implementation to the Evidence primitive. True Evidence immutability (the platform invariant that Evidence cannot be mutated once recorded) is not yet enforced at the platform level. Evidence is managed within each adapter. |

---

### Hypothesis

**Architecture definition:** Versioned statement of what the Actor believes and why. Hypotheses evolve through versioning.

| Aspect | Detail |
|--------|--------|
| Implementation status | **Partial** |
| Primary crate | `coralys-core` |
| Primary type | `DecisionProposal` (`coralys-core/src/models/decision_proposal.rs`) |
| Notes | `DecisionProposal` is the closest current implementation to the Hypothesis primitive — it represents a proposed decision with associated reasoning. Hypothesis versioning (v1 → v2 → v3) is not yet implemented at the platform level. In the UltraCrew adapter, roster versions serve as hypothesis versions. |

---

### Review

**Architecture definition:** Structured periodic comparison of Hypothesis against Evidence.

| Aspect | Detail |
|--------|--------|
| Implementation status | **Partial** |
| Primary crate | `coralys-core` |
| Primary type | `DecisionPlugin::evaluate()` (`coralys-core/src/lib.rs:38`) |
| Notes | `DecisionPlugin::evaluate()` performs the review step — comparing the current state against the proposal. A structured Review primitive with documented outcomes, attendees, and conditions is not yet implemented at the platform level. |

---

### Timeline

**Architecture definition:** Complete record of how the Hypothesis evolved. Every state transition is recorded.

| Aspect | Detail |
|--------|--------|
| Implementation status | **Implemented** |
| Primary crate | `coralys-core` |
| Primary type | `DecisionLineage` (`coralys-core/src/models/decision_lineage.rs:16`) |
| Notes | `DecisionLineage` is a full implementation of the Timeline primitive. It maintains a tree of `LineageNode` entries, each with a parent reference, a `DecisionProposal`, and an `EvaluationResult`. The `root_id` and `current_id` fields allow traversal of the full decision history. This is the most complete primitive implementation in the platform. |

---

### Outcome

**Architecture definition:** What actually happened; raw material for Learning. Every Outcome belongs to exactly one Workspace.

| Aspect | Detail |
|--------|--------|
| Implementation status | **Implemented** |
| Primary crate | `coralys-core` |
| Primary type | `Outcome` trait (`coralys-core/src/lib.rs:5`) |
| Notes | `Outcome` is fully implemented as a platform trait. It provides `objectives()` (multi-objective fitness values), `primary_objective()`, `is_valid()`, and `solution()`. The platform invariant (every Outcome belongs to exactly one Workspace) is not yet enforced at the platform level — it is managed within each adapter. |

---

### Learning

**Architecture definition:** Analyses Outcomes; extracts Patterns; does not store. Learning computes — it never mutates historical Evidence.

| Aspect | Detail |
|--------|--------|
| Implementation status | **Implemented** |
| Primary crate | `coralys-core` |
| Primary type | `InnovationTracker` (`coralys-core/src/memory.rs:14`) |
| Notes | `InnovationTracker` is a full implementation of the Learning primitive. It observes solution signatures across generations, tracks novel discoveries, persistence, rediscovery, and extinction. It produces `InnovationTelemetry` — a structured learning output. It does not store solutions — it stores signatures (hashes). The platform invariant (Learning never mutates historical Evidence) is satisfied by design. |

---

### Pattern

**Architecture definition:** Cross-Workspace knowledge extracted by Learning. Patterns are generalised from Outcomes.

| Aspect | Detail |
|--------|--------|
| Implementation status | **Partial** |
| Primary crate | `coralys-ecology` |
| Primary type | `MemoryModel` trait (`coralys-ecology/src/traits.rs:7`) |
| Notes | `MemoryModel` provides the observe/state interface for pattern accumulation. `TopologyModel` provides the transformation interface for pattern extraction. The full Pattern primitive (with maturity levels: Candidate → Observed → Repeated → Validated → Institutionalised) is not yet implemented. Patterns are currently implicit in the `InnovationTracker` signature memory. |

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
| Coralys primitives realised | Workspace (Scheduling Workspace), Actor (Scheduler), Intent (Scheduling objective), Subject (Scheduling period / crew base), Evidence (Operational data), Hypothesis (Roster Strategy), Timeline (Scheduling Timeline), Outcome (Operational KPIs), Learning (Workforce Operations Learning Loop), Pattern (Workforce Behaviour Pattern) |
| Key modules | `constraint_engine`, `decision_intelligence`, `ecology`, `optimization`, `pipeline`, `recommendation`, `schedule_solution`, `public_contracts` |
| INRC2 implementation | `inrc/evaluator`, `inrc/schedule_optimizer`, `inrc/validator`, `inrc/parser`, `inrc/history`, `inrc/audit`, `inrc/baseline`, `inrc/bipartite_matching` |
| Benchmark evidence | Ablation matrices (30-seed), survival curves, extinction curves, horizon tests, alpha sweeps |

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
| Implementation status | **Stub** |
| Coralys primitives realised | None — adapter is an empty stub |
| Notes | The ChronoSentiment adapter is registered in the codebase but contains no implementation. All ChronoSentiment product capabilities (Decision Workspace, Investment Thesis, Committee Review, etc.) are documented in the blueprints but not yet implemented in the adapter. |

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

---

## Platform Invariant Implementation Status

The following platform invariants are documented in Architecture Observation 9. This table records their current enforcement status.

| Invariant | Status | Notes |
|-----------|--------|-------|
| Every Workspace has exactly one Intent | **Not enforced** | Intent is implicit in Scenario configuration; no platform-level enforcement |
| Evidence is immutable once recorded | **Not enforced** | Immutability is a design principle; not yet enforced by the platform |
| Every Outcome belongs to exactly one Workspace | **Not enforced** | Ownership is managed within each adapter; no platform-level enforcement |
| Learning never mutates historical Evidence | **Satisfied by design** | `InnovationTracker` stores signatures, not Evidence; cannot mutate Evidence |
| Knowledge Graph stores but does not infer without traceability | **Partial** | `MemoryModel` stores state; traceability of inference is not yet implemented |

---

## Summary

| Primitive | Status | Primary crate | Primary type |
|-----------|--------|--------------|-------------|
| Workspace | Partial | `coralys-core` | `Scenario` |
| Actor | Partial | `coralys-planning` | `Worker` |
| Intent | Stub | `coralys-core` | `DecisionProposal` (partial) |
| Subject | Partial | `coralys-planning` | `PlanningScenario` |
| Context | Planned | — | — |
| Evidence | Partial | `coralys-core` | `EvaluationResult` |
| Hypothesis | Partial | `coralys-core` | `DecisionProposal` |
| Review | Partial | `coralys-core` | `DecisionPlugin::evaluate()` |
| Timeline | **Implemented** | `coralys-core` | `DecisionLineage` |
| Outcome | **Implemented** | `coralys-core` | `Outcome` trait |
| Learning | **Implemented** | `coralys-core` | `InnovationTracker` |
| Pattern | Partial | `coralys-ecology` | `MemoryModel` |
| Knowledge Graph | Partial | `coralys-ecology` | `MemoryModel` + `TopologyModel` |
| Continuous Learning Engine | **Implemented** | `coralys-moga` | `MogaReasoningEngine` |

---

*Coralys Platform Architecture Traceability v1.0 | July 2026 | Status: Baseline*
*Maps every Coralys platform primitive to its Rust implementation.*
*Review trigger: New crate added; primitive implementation status changes; platform architecture revision.*