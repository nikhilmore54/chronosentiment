# Architecture Glossary

> **Status**: v1.2 — Updated 2026-07-21 (added Solution Adapter; D-8, D-9)
> **Date**: 2026-07-19  
> **Purpose**: Define terms that recur across [`CODEBASE_ASSESSMENT.md`](CODEBASE_ASSESSMENT.md), [`ARCHITECTURE_EVOLUTION.md`](ARCHITECTURE_EVOLUTION.md), and [`PRODUCT_PORTFOLIO.md`](../PRODUCT_PORTFOLIO.md). A shared vocabulary prevents two contributors from using the same word to mean different things.  
> **Policy**: Add terms when they first appear in a document and their meaning is not immediately obvious. Do not add terms whose meaning is unambiguous in context.

---

## Core Architectural Terms

### Platform

The collection of reusable, domain-agnostic crates that products are built on. Platform crates never depend on product crates. The dependency direction is always: Product → Platform.

In this codebase: `coralys-core`, `coralys-moga`, and the planned generic planning capability crate are Platform crates.

---

### Domain Library

A crate that implements the types, rules, and semantics of a specific problem domain. Domain libraries are reusable assets — they are not temporary scaffolding. They depend on the Platform but not on Products or on other Domain Libraries.

In this codebase: `adapters/ultracrew` (INRC2 nurse rostering domain), `coralys-scheduling` (airline crew scheduling domain), `adapters/cvrp` (vehicle routing domain).

Formerly called "Domain Implementation" in earlier versions of this document. Renamed to "Domain Library" to reflect that these are permanent, reusable assets.

---

### Solution Adapter

The preferred term for a Domain Library when emphasizing its role as the integration boundary between an Application and the Coralys Platform. The two terms refer to the same crate; "Solution Adapter" is used in dependency rules and architectural invariants to make the boundary role explicit.

A Solution Adapter:
- is the exclusive integration layer between an Application and the Coralys Platform;
- owns all domain models, domain-specific algorithms, configuration, policies, and orchestration logic;
- translates between platform abstractions (e.g. `Genome`, `FitnessVector`, `GaResult`) and application concepts (e.g. `ScheduleSolution`, `ConstraintReport`);
- presents a stable, domain-meaningful public API to the Application layer;
- must not expose platform implementation details in its public API surface.

In this codebase: `adapters/ultracrew` is the Solution Adapter for UltraCrew. `adapters/chronosentiment` is the Solution Adapter for ChronoSentiment.

The dependency hierarchy is: Application → Solution Adapter → Platform. See Rules 5 and 6 in [`PLATFORM_CRATE_RESPONSIBILITIES.md`](PLATFORM_CRATE_RESPONSIBILITIES.md).

---

### Product

A marketable, independently deployable software product built on top of Platform crates and Domain Libraries. Products own workflows and user experience. They do not own optimization algorithms (those belong to the Platform) or domain semantics (those belong to Domain Libraries).

In this codebase: UltraCrew, AirlineOps, ChronoSentiment.

---

### Application

A deployment artifact — a REST server, CLI tool, desktop application, or cloud service — that exposes a Product's capabilities to end users. An Application is not the same as the Product it deploys. The Product is the business logic and domain model; the Application is how it is packaged and delivered.

In this codebase: `services/ultracrew_server` is an Application that deploys UltraCrew capabilities. It is not UltraCrew itself.

---

### Capability

A reusable architectural service provided by the Platform through a stable interface. Products consume Capabilities; they do not depend on the concrete implementation behind the interface. Capabilities are bigger than functions — they can encompass engines, workflows, or coordinated behavior.

The three primary Capabilities in the Coralys architecture are:

- **Optimization** — finding solutions in a search space (provided by `coralys-moga`)
- **Planning** — assigning work to workers over a planning horizon (provided by the planned generic planning capability crate)
- **Decision Intelligence** — explaining, comparing, and recommending decisions (provided by `coralys-decision` and related crates)

These three Capabilities are peer concerns. They are composed, not collapsed into a single abstraction.

---

### Optimization

The Capability of finding high-quality solutions in a large search space, typically using a population-based or local-search algorithm. In this codebase, Optimization is provided by `coralys-moga` (multi-objective genetic algorithm engine).

Optimization is a Platform concern. Products do not import optimization algorithms directly; they consume the Optimization Capability through an interface.

---

### Planning

The Capability of assigning Resources to Atomic Planning Units over a Planning Horizon, subject to Constraints and Objectives. Planning is the core Capability of UltraCrew and the crew assignment stage of AirlineOps.

Planning answers: *who or what performs the work?* Scheduling answers: *when does the work happen?* Planning consumes the work schedule produced by Scheduling and allocates Resources to it (D-7).

Planning is distinct from Optimization: Optimization finds solutions in a search space; Planning defines what a valid solution means in a specific domain.

Note: Planning is a capability of the **planning layer**, not the full Coralys platform. ChronoSentiment operates at the decision layer and may not involve resource allocation at all (see OQ-6 in [`ARCHITECTURE_EVOLUTION.md`](ARCHITECTURE_EVOLUTION.md)).

---

### Decision Intelligence

The Capability of explaining why a decision was made, comparing alternative decisions, and recommending actions. Decision Intelligence is the core Capability of ChronoSentiment and the Explanation Engine (Stream B module B3).

Decision Intelligence is distinct from Planning: Planning produces a roster; Decision Intelligence explains it.

---

### Atomic Planning Unit

The smallest schedulable unit that is directly assigned to a Resource within a planning problem. Domain-specific structures may exist below this level, but they are not exposed through the generic planning interface.

Examples:
- INRC2: a `Shift` (a named, time-bounded work period)
- Airline crew rostering: a `Pairing` (a base-to-base crew trip consisting of one or more Duties)
- Manufacturing: a task or operation
- Retail: a shift

The `FlightLeg` and `Duty` in the airline domain are sub-components of a Pairing. They are not Atomic Planning Units because they are not directly assigned to crew members — Pairings are.

Cross-domain abstractions are compared at the Atomic Planning Unit level, not at the level of the full domain hierarchy (see Principle 7 in [`ARCHITECTURE_EVOLUTION.md`](ARCHITECTURE_EVOLUTION.md)).

**Important**: Atomic Planning Unit is a **comparison lens** for Phase 1 analysis. It is not necessarily the name of the final interface type. The correct interface type will be determined by Phase 1 evidence.

---

### Resource

The generic term for any constrained entity that can be allocated to a Planning Unit. Resources are the fundamental concept of the planning layer. Domain-specific specializations include: Nurse (INRC2), Crew Member (airline), Aircraft (fleet planning), Vehicle (routing), Machine (manufacturing), Room (meeting planning).

Resource is the correct platform-level vocabulary for the planning capability (D-5). It is broader than Worker: Worker is a specialization of Resource for workforce planning problems.

Note: Resource is fundamental to the **planning layer**, not to the full Coralys platform. ChronoSentiment, for example, may not have resources at all — it operates at the decision layer (see OQ-6).

---

### Worker

A specialization of Resource for workforce planning problems. A Worker is a human resource that can be assigned to shifts, pairings, or other Planning Units.

Domain-specific names: Nurse (INRC2), Crew Member (airline), Staff Member (retail), Operator (manufacturing).

Worker is the correct term within workforce planning domain libraries (e.g. `adapters/ultracrew`). The platform planning capability uses Resource, not Worker, to remain applicable to non-human resources.

---

### Benchmark

A validated, reproducible test of system performance against a known problem instance. Benchmarks are permanent (Architectural Invariant AI-2). They serve as regression suites, validation datasets, and research evidence.

In this codebase: the INRC2 benchmark suite (n030–n120 instances) in `adapters/ultracrew/tests/data/` is the primary benchmark. The CVRP benchmark suite in `services/cvrp_server/` is a secondary benchmark.

---

### Scenario

A complete description of a planning problem instance: the Workers, the Atomic Planning Units to be assigned, the constraints, and the objectives. In `coralys-core`, `Scenario` is a marker trait. Domain-specific scenarios (e.g. `InrcScenario`) implement this trait.

---

### Objective

A measurable quantity that the optimizer attempts to minimize or maximize. Objectives may be in conflict (e.g. minimize cost while maximizing worker preference satisfaction), which is why multi-objective optimization is used.

---

### Constraint

A rule that a valid solution must satisfy. Hard constraints must not be violated (e.g. a worker cannot be assigned to two overlapping shifts). Soft constraints may be violated at a penalty cost (e.g. a worker's shift preference is not met).

---

### Roster

The output of a workforce planning problem: a complete assignment of Workers to Atomic Planning Units over a planning horizon. In the INRC2 domain, a Roster is a `HashMap<NurseId, Vec<ShiftType>>`. In the airline domain, a Roster is a `Rotation` (a sequence of Pairings assigned to a Crew Member).

Roster is a domain-specific specialization of Utilization Plan.

---

### Utilization Plan

A domain-agnostic allocation of Resources to work over a Planning Horizon that satisfies mandatory Constraints and optimizes one or more Objectives within an operational time budget. The Utilization Plan is the generic output of the planning capability.

Domain-specific specializations:
- Nurse rostering: Roster (nurses assigned to shifts)
- Airline crew: Crew Allocation (crew members assigned to pairings)
- Vehicle routing: Route Plan (vehicles assigned to customer visits)
- Manufacturing: Production Schedule (machines and operators assigned to operations)

The Utilization Plan is the output of the **planning layer**. Products in the decision layer (e.g. ChronoSentiment) produce Decisions, not Utilization Plans.

---

### Workflow

A sequence of steps that a Product orchestrates to deliver a business outcome. Workflows are a Product concern. The Platform provides Capabilities that individual steps in a Workflow consume.

Example: the AirlineOps crew scheduling workflow is: import flight schedule → optimize pairings → assign crew (via Planning Capability) → publish roster → monitor for disruptions → recover.

---

### Planning Horizon

The time span over which a planning problem is solved. The Planning Horizon is a fundamental cross-domain concept: every scheduling domain has one, but its length and granularity vary significantly.

| Domain | Typical Planning Horizon |
|---|---|
| INRC2 (nurse rostering) | Multi-week (4–8 weeks) |
| Airline crew rostering | Schedule period (months) |
| Manufacturing | Shift cycle (days to weeks) |
| Retail | Roster cycle (weeks) |
| Contact Centre | Intraday to weekly |

The Planning Horizon affects how constraints are evaluated (e.g. maximum working weekends per month), how objectives are measured (e.g. total cost over the period), and how history is carried forward between planning cycles.

In the generic planning interface, the Planning Horizon is expected to be a first-class parameter of the planning scenario, not an implicit assumption baked into constraint logic.

---

## Status Terms

### Mature

A crate or module that has a complete implementation, test coverage, and benchmark validation. Changes to mature code require regression testing.

### Stub

A crate or module that exists in the workspace with a defined interface but no functional implementation. Stubs are placeholders for future work.

### Working Hypothesis

A name, design, or assumption that is being used provisionally during analysis. A working hypothesis is not a decision. It may be changed without a formal decision record. Working hypotheses are explicitly marked as such in [`ARCHITECTURE_EVOLUTION.md`](ARCHITECTURE_EVOLUTION.md).

### Proposal

A concrete architectural suggestion that has been written down and is pending review. A Proposal becomes a Decision when it is accepted and recorded in the Decisions Recorded table in [`ARCHITECTURE_EVOLUTION.md`](ARCHITECTURE_EVOLUTION.md).

### Decision

A recorded architectural choice. Decisions are listed in the Decisions Recorded table in [`ARCHITECTURE_EVOLUTION.md`](ARCHITECTURE_EVOLUTION.md) with a date and rationale. Decisions may be revisited, but only with a new recorded decision that supersedes the previous one.

---

*Add new terms here as they are introduced in the architecture documents. Terms should be defined at the level of precision needed to prevent misunderstanding — not more.*