# UltraCrew — Product Strategy

**Document type:** Product Strategy
**Version:** 1.0
**Status:** Baseline
**Date:** 2026-07-26
**Owner:** Strategy / Product

---

## Document Governance

| Field | Value |
|-------|-------|
| Document Status | Baseline v1.0 |
| Review Trigger | Material change in product positioning, target market, or platform identity |

**Relationship to other documents:**
- Informed by: `CORALYS_PLATFORM_ARCHITECTURE.md` (platform architecture — primitives, lifecycle, Continuous Learning Engine)
- Informed by: `CORALYS_PLATFORM_STRATEGY.md` (platform portfolio positioning)
- Informed by: `CS-S-001_ChronoSentiment_Product_Strategy_v1.3.md` (sibling product strategy)
- Informs: UltraCrew Product Blueprint (to be written)

---

## Purpose

This document defines the product strategy for UltraCrew — what it is, who it is for, what problem it solves, and how it is positioned commercially. It is a strategy document grounded in the implemented product. The architecture is defined in `CORALYS_PLATFORM_ARCHITECTURE.md`.

---

## What UltraCrew Is

UltraCrew is a **Workforce Decision Engine**. It produces optimised workforce schedules, recovers from disruptions in real time, and accumulates operational knowledge across scheduling cycles — so that every scheduling run benefits from the experience of every previous one.

UltraCrew is built on the Coralys Knowledge Evolution Platform. It is the primary commercial realisation of the platform in the workforce scheduling domain.

The customer does not need to know this. The customer sees better schedules, faster disruption recovery, and operational decisions that improve over time.

---

## The Problem UltraCrew Solves

Workforce scheduling is one of the most complex operational problems in any workforce-intensive organisation. The quality of scheduling decisions directly affects operational performance, staff satisfaction, regulatory compliance, and cost.

Most organisations have a scheduling problem that compounds over time:

- Schedules are produced in isolation — each cycle starts from scratch.
- Disruption recovery is reactive and manual — the same disruptions recur without institutional learning.
- Scheduling decisions are not tracked against outcomes — there is no feedback loop.
- Operational knowledge is held by individuals — it leaves when they do.
- Compliance is checked after the fact — violations are discovered too late to fix cheaply.

UltraCrew solves this by providing a structured environment for workforce decision intelligence — from schedule generation through disruption recovery to outcome recording and operational knowledge accumulation.

---

## What UltraCrew Has Built

The following capabilities are implemented in the current codebase. This is the ground truth for the product strategy.

### Optimisation Engine

UltraCrew uses a multi-objective genetic algorithm (MOGA) engine to generate workforce schedules. The engine:

- Handles multiple competing objectives simultaneously (coverage, cost, fairness, compliance, robustness)
- Supports configurable optimisation profiles for different operational contexts
- Includes an elite archive that preserves the best solutions across generations
- Provides full observability — evolution metrics, processor metrics, convergence tracking
- Supports pluggable improvement operators, local search, and constraint repair

The MOGA engine is domain-neutral. UltraCrew configures it for workforce scheduling. The same engine is available for other domains.

### INRC2 Scheduling (Nurse Rostering)

UltraCrew implements the full INRC2 (International Nurse Rostering Competition 2) problem specification:

- Nurse and shift type modelling
- Hard and soft constraint evaluation
- Bipartite matching for assignment optimisation
- Schedule validation and audit
- History-aware scheduling (multi-week continuity)
- Baseline comparison and benchmark reporting

Benchmark data confirms performance across ablation matrices, survival curves, extinction curves, and horizon tests.

### Airline Crew Scheduling

UltraCrew implements airline crew scheduling with full legality rule enforcement:

- Crew member and qualification modelling
- Duty, pairing, rotation, and roster construction
- Flight leg coverage requirements
- Legality rules: FDP (Flight Duty Period), duty time limits, minimum rest, base return, qualification matching, duty connectivity
- Resilience: disruption modelling, reserve crew management, robustness scoring
- Scalability tests and solution quality benchmarks

### Constraint Engine

A dedicated constraint engine enforces hard constraints during schedule generation and repair. Constraints are domain-configurable — the engine does not know what a "nurse" or a "crew member" is; it knows what a constraint violation is.

### Decision Intelligence

A decision intelligence layer provides:

- Recommendation generation — surfacing scheduling options with explanations
- Ecology-aware optimisation — the platform's ecological coherence engine maintains solution quality across the population
- Operational knowledge accumulation — every scheduling cycle contributes to the operational knowledge graph

### Pipeline and Observability

A full optimisation pipeline with:

- Configurable pipeline stages
- Passive telemetry collection
- Engagement audit
- Benchmark framework with campaign comparison
- CLI interface (`ultracrew-cli`)

---

## Who UltraCrew Is For

Workforce-intensive industries where scheduling quality directly affects operational performance, compliance, and cost:

| Industry | Primary use case |
|----------|-----------------|
| Aviation | Crew rostering, pairing construction, disruption recovery |
| Healthcare | Nurse rostering, shift scheduling, on-call management |
| Logistics | Driver scheduling, route crew assignment |
| Retail | Staff scheduling, shift optimisation |
| Manufacturing | Shift planning, skills-based assignment |
| Utilities | Field crew scheduling, maintenance rostering |

Within each organisation, the primary buyers are:

- **Operations directors** — accountable for scheduling quality and operational performance
- **Workforce planning managers** — responsible for producing schedules
- **Compliance officers** — responsible for regulatory adherence
- **IT / platform teams** — responsible for integration with HR and operations systems

---

## Commercial Positioning

**Workforce Decision Engine.**

The customer buys better operational outcomes:

- Better schedules — higher coverage, lower cost, fairer distribution
- Faster disruption recovery — real-time re-optimisation when plans change
- Regulatory compliance — hard constraint enforcement built in, not bolted on
- Operational memory — every scheduling cycle makes the next one better
- Explainable decisions — every schedule recommendation has a traceable reasoning chain

The Coralys platform is not front and centre. The decision quality is.

---

## Why UltraCrew Is Different

Most workforce management tools are scheduling tools. They produce a schedule. UltraCrew is a decision engine. It produces a schedule, explains it, recovers from disruptions, and improves over time.

| Competitor | What they do | Why UltraCrew is different |
|------------|-------------|---------------------------|
| Kronos / UKG | Workforce management | Scheduling tool, not decision engine; no compounding knowledge |
| Workday | HR and workforce management | HR tool, not scheduling optimisation |
| Verint | Workforce optimisation | Contact centre focus, not multi-domain |
| NICE | Workforce management | Contact centre focus, not multi-domain |
| Quinyx | Workforce management | Scheduling tool, not decision engine |
| Custom solvers | Domain-specific optimisation | Single-domain, no knowledge accumulation, no platform |

The MOGA engine handles multiple competing objectives simultaneously — coverage, cost, fairness, compliance, robustness — in a single optimisation run. Most tools optimise for one objective and treat the others as constraints. UltraCrew treats them all as objectives and finds the best trade-off.

---

## Coralys Platform Realisation

UltraCrew is a realisation of the Coralys Knowledge Evolution Platform in the workforce scheduling domain. The platform provides:

- **Lifecycle governance** — every scheduling cycle is a Workspace with a structured lifecycle
- **Continuous Learning Engine** — every completed cycle contributes to operational knowledge
- **Knowledge Graph** — disruption patterns, crew behaviour patterns, and scheduling outcomes accumulate across cycles
- **Domain Adapter Model** — UltraCrew configures the platform with workforce vocabulary; the platform provides the lifecycle

The Coralys adapter vocabulary for UltraCrew:

| Coralys Primitive | UltraCrew |
|------------------|-----------|
| Workspace | Scheduling Workspace |
| Actor | Scheduler / Operations manager |
| Intent | Scheduling objective (e.g. "Generate crew roster — SunAir BOM base, July 2026") |
| Subject | Scheduling period / route / crew base |
| Context | Operational environment (airport, network, constraints) |
| Evidence | Operational data (disruptions, KPIs, crew availability, regulations) |
| Hypothesis | Roster Strategy |
| Hypothesis version | Roster version |
| Review | Schedule Review |
| Timeline | Scheduling Timeline |
| Outcome | Operational KPIs |
| Pattern | Workforce Behaviour Pattern |
| Learning | Workforce Operations Learning Loop |
| Knowledge Graph | Operational Knowledge Graph |

**Continuous Learning Engine realisation:** Workforce Operations Learning Loop

---

## Go-to-Market

**Primary channel:** Direct sales (enterprise)
**Secondary channel:** Industry partnerships (aviation, healthcare, logistics)
**Pricing model:** Enterprise licence (annual)
**Target price point:** $100,000–$1,000,000/year (enterprise)

**Initial target industries:** Aviation (crew scheduling) and Healthcare (nurse rostering) — both have implemented domain adapters with full legality rule enforcement.

---

## Roadmap Principles

1. **Implementation first** — the product strategy follows the implementation. Features are documented when they are built, not when they are imagined.
2. **Domain depth before breadth** — go deep in aviation and healthcare before expanding to logistics, retail, and manufacturing.
3. **Compounding value** — features that make the operational knowledge graph richer over time are more valuable than features that provide one-time utility.
4. **Governance by default** — compliance and audit capabilities are built in from the start, not retrofitted.
5. **Explainability as a feature** — every scheduling recommendation should be explainable to the operations manager who acts on it.

---

## Roadmap

| Phase | Features | Status |
|-------|----------|--------|
| v1.0 | MOGA scheduling engine, INRC2 nurse rostering, airline crew scheduling, constraint engine, decision intelligence, pipeline observability, CLI | **Implemented** |
| v1.1 | Disruption recovery workflow, real-time re-optimisation, operational knowledge accumulation, pattern recognition | In progress |
| v2.0 | Cross-domain optimisation, AI-assisted scheduling, predictive disruption, Knowledge Graph Services | Planned |

---

## Appendix A — Implementation Audit Summary

The following capabilities are confirmed implemented in the codebase as of July 2026:

**Core engine:** `coralys-moga` — MOGA engine with `EvolutionEngineBuilder`, `MogaReasoningEngine`, `FitnessEvaluator`, `MutationOperator`, `CrossoverOperator`, `SelectionStrategy`, `ImprovementOperator`, `EliteArchive`, `EvolutionState`, `TerminationPolicy`, `PipelineObserver`, `ProcessingMetricsCollector`, `ConstraintChecker`, `RepairHeuristic`

**Planning layer:** `coralys-planning` — domain-neutral planning traits (`Worker`, `PlanningUnit`, `CoverageDemand`, `PlanningSolution`, `PlanningScenario`) with INRC2 and airline as concrete implementations

**UltraCrew adapter:** `adapters/ultracrew` — `constraint_engine`, `decision_intelligence`, `ecology`, `optimization`, `pipeline`, `recommendation`, `schedule_solution`, `public_contracts`, full INRC2 implementation, CLI

**Airline adapter:** `adapters/airline` — full domain model (crew, duty, flight, pairing, roster, rotation), legality rules (FDP, duty time, minimum rest, base return, qualification, coverage), resilience (disruption, reserve, robustness), benchmark and scalability tests

**Benchmark evidence:** Ablation matrices (30-seed), survival curves, extinction curves, horizon tests, alpha sweeps, ancestry analysis — confirming optimisation engine performance across multiple problem configurations

---

*UltraCrew Product Strategy v1.0 | July 2026 | Status: Baseline*
*Defines product strategy for UltraCrew — Workforce Decision Engine.*
*Grounded in implementation audit of `adapters/ultracrew`, `adapters/airline`, `coralys-moga`, and `coralys-planning` as of July 2026.*
*Review trigger: Material change in product positioning, target market, or platform identity.*