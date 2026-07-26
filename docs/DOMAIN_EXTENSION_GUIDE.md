# Domain Extension Guide

> **Status**: v1.1 — Architecture Baseline v1.0
> **Date**: 2026-07-19
> **Purpose**: Define what a Domain Library is, what an Integration Adapter is, how a Product uses them, the required responsibilities of each, and the naming conventions for future domains. This document is the contract for adding a new domain to Coralys.
> **Relationship**: Consistent with [`ARCHITECTURE_EVOLUTION.md`](ARCHITECTURE_EVOLUTION.md) (principles and invariants), [`PRODUCT_PORTFOLIO.md`](../PRODUCT_PORTFOLIO.md) (product definitions), and [`ARCHITECTURE_GLOSSARY.md`](ARCHITECTURE_GLOSSARY.md) (vocabulary).

---

## 0. The Guiding Principle

**A Domain Library should model its domain faithfully. Implementing the Coralys execution contract must not require simplifying, renaming, or restructuring domain concepts purely to satisfy the platform. If a domain cannot express itself naturally through the execution contract, the contract — not the domain — should be questioned.**

Coralys does not ask: *"How do we fit this domain into Coralys?"*

It asks: *"Is Coralys generic enough that this domain can express itself without compromise?"*

The platform owns only the minimal, stable contract that lets diverse domains coexist: `Scenario`, `Solution`, `Outcome`, the execution lifecycle, the optimization interface, and the decision orchestration. It never owns what a `Pairing` is, what a `Shift` is, what a `Market Event` is, or what a `Vehicle Route` is. Those remain entirely within the Domain Library.

This principle is the reason Principle 4 in [`ARCHITECTURE_EVOLUTION.md`](ARCHITECTURE_EVOLUTION.md) requires two independent implementations before promoting an abstraction to the platform: you are extracting only what is demonstrably common, not inventing a universal domain model.

---

## 1. Three Concepts That Are Not the Same

The word "adapter" currently appears in the codebase with multiple meanings. This document establishes the correct vocabulary.

### Domain Library

A crate that models a specific problem domain. It defines the entities, constraints, objectives, and evaluation semantics of that domain. It is a permanent, reusable asset.

**Responsibilities:**
- Define domain entities (e.g. `Nurse`, `Shift`, `FlightLeg`, `Pairing`)
- Define domain constraints (e.g. INRC2 contract rules, FDP limits)
- Define domain objectives (e.g. coverage, fairness, cost)
- Implement platform capability interfaces for the domain
- Provide scenario construction utilities
- Contain benchmark instances and validation data

**Does not own:**
- User experience or workflow orchestration (Product concern)
- External system integration (Integration Adapter concern)
- Generic platform algorithms (Platform concern)

**Dependency direction:** Domain Library depends on Platform. Never on Products or on other Domain Libraries (AI-6).

---

### Integration Adapter

A crate or module that bridges Coralys to an external system. It translates external data formats into Coralys domain models and translates Coralys outputs back to external formats.

**Responsibilities:**
- Parse external data (CSV, REST, SQL, Kafka, SAP, Oracle, Workday, etc.)
- Construct domain scenarios from external data
- Serialize Coralys outputs to external formats
- Handle authentication, rate limiting, and protocol concerns

**Does not own:**
- Domain semantics (Domain Library concern)
- Business logic or optimization (Platform or Product concern)

**Examples (future):**
- `adapters/sap` — SAP workforce data to INRC2 scenario
- `adapters/workday` — Workday schedule to UltraCrew scenario
- `adapters/ods` — Airline ODS flight schedule to AirlineOps scenario

---

### Product

A marketable, independently deployable application built on top of Platform crates and Domain Libraries. Products own workflows and user experience.

**Responsibilities:**
- Define the decision workflow (which capabilities, in what order)
- Own the user-facing application (UI, API, CLI)
- Compose Platform capabilities through the common execution contract
- Inject Domain Libraries at startup (not at compile time)

**Does not own:**
- Domain semantics (Domain Library concern)
- Optimization algorithms (Platform concern)
- External system integration (Integration Adapter concern)

---

## 2. The Correct Dependency Graph

```
External Systems
      |
      v
Integration Adapters
      |
      v
Domain Libraries
      |
      v
Platform (coralys-core, coralys-moga, coralys-simulation, etc.)
      ^
      |
Products (UltraCrew, AirlineOps, ChronoSentiment)
      ^
      |
Applications (ultracrew_server, airlineops_server, etc.)
```

Domain Libraries and Products both depend on the Platform. Domain Libraries never depend on Products. Products never depend on Domain Libraries at compile time — they inject them at runtime through the platform's execution contract.

---

## 3. Current Codebase Mapping

The current `adapters/` directory contains crates that are Domain Libraries, not Integration Adapters in the traditional sense. This is a naming inconsistency noted in [`CODEBASE_ASSESSMENT.md`](CODEBASE_ASSESSMENT.md). The correct roles are:

| Crate | Correct Role | Notes |
|---|---|---|
| `adapters/ultracrew` | Domain Library (INRC2 nurse rostering) | Contains benchmark data, scenario construction, constraint evaluation |
| `adapters/cvrp` | Domain Library (vehicle routing) | Contains CVRP benchmark instances |
| `coralys-scheduling` | Domain Library (airline crew scheduling) | Contains airline domain model; long-term scope under architectural review (OQ-1) |

No Integration Adapters (external system bridges) currently exist in the codebase. They will be created when products need to connect to external data sources.

---

## 4. Domain Library Specification Template

Every Domain Library must answer the following ten questions before implementation begins. This prevents domain libraries from being created without a clear scope.

**1. What decision problem does this domain model?**
State the problem in one sentence. Example: "Assign nurses to shifts over a multi-week planning horizon while satisfying contract rules and coverage requirements."

**2. What are the primary Resources?**
List the entities that receive assignments. Example: Nurses, Crew Members, Vehicles, Machines.

**3. What are the Atomic Planning Units?**
List the units of work that are assigned to Resources. Example: Shifts, Pairings, Routes, Operations.

**4. What are the hard Constraints?**
List the rules that must not be violated. Example: No nurse works two overlapping shifts. No crew member exceeds FDP limits.

**5. What are the soft Constraints / Objectives?**
List the rules that may be violated at a cost, and the quantities to optimize. Example: Minimize uncovered shifts. Maximize preference satisfaction. Minimize total cost.

**6. What is the Planning Horizon?**
State the typical time span. Example: 4-week roster cycle. Monthly schedule period.

**7. What Scenarios does it support?**
List the scenario types. Example: Initial roster construction. Disruption recovery. What-if analysis.

**8. What is the output (Utilization Plan)?**
Describe the output. Example: A roster mapping each nurse to a sequence of shifts. A crew allocation mapping each crew member to a sequence of pairings.

**9. Which Platform capabilities does it consume?**
List the platform capabilities used. Example: Optimization (coralys-moga). Evaluation (coralys-eval). Simulation (coralys-simulation).

**10. What benchmark instances exist or are planned?**
List the benchmark datasets. Example: INRC2 instances n030, n060, n100, n120. CVRP benchmark set A.

---

## 5. Domain Inventory

Current and planned Domain Libraries with their specification summaries.

### INRC2 — Nurse Rostering (`adapters/ultracrew`)

| Field | Value |
|---|---|
| Decision problem | Assign nurses to shifts over a multi-week planning horizon |
| Resources | Nurses |
| Atomic Planning Units | Shifts |
| Hard constraints | Contract rules, forbidden successors, minimum rest |
| Objectives | Coverage, fairness, preference satisfaction |
| Planning Horizon | 4-8 weeks |
| Output | Roster (nurse → shift sequence) |
| Platform capabilities | Optimization (coralys-moga) |
| Benchmarks | INRC2 instances n030, n060, n100, n120 |
| Status | Mature — benchmark-validated |

---

### Airline Crew (`coralys-scheduling`)

| Field | Value |
|---|---|
| Decision problem | Assign crew members to pairings over a schedule period |
| Resources | Crew members (captains, first officers, cabin crew) |
| Atomic Planning Units | Pairings (working assumption — to be validated in Phase 1) |
| Hard constraints | FDP limits, rest rules, aircraft qualification, base constraints |
| Objectives | Cost minimization, robustness, legality |
| Planning Horizon | Schedule period (months) |
| Output | Crew allocation (crew member → pairing sequence) |
| Platform capabilities | Optimization (coralys-moga) — planning capability TBD (Phase 1) |
| Benchmarks | None yet — to be established in Phase 1 |
| Status | Stub — domain model exists, no benchmark validation |

---

### CVRP — Vehicle Routing (`adapters/cvrp`)

| Field | Value |
|---|---|
| Decision problem | Assign vehicles to customer visit sequences minimizing total distance |
| Resources | Vehicles |
| Atomic Planning Units | Routes (sequences of customer visits) |
| Hard constraints | Vehicle capacity, time windows |
| Objectives | Minimize total distance / cost |
| Planning Horizon | Single planning period (daily or weekly) |
| Output | Route plan (vehicle → customer visit sequence) |
| Platform capabilities | Optimization (coralys-moga) |
| Benchmarks | Standard CVRP benchmark set |
| Status | Benchmark domain — used for platform validation |

---

### ChronoSentiment — Financial Decision Intelligence

| Field | Value |
|---|---|
| Decision problem | Produce actionable trading or investment decisions from market evidence |
| Resources | Possibly none — operates at the decision layer, not the planning layer |
| Atomic Planning Units | N/A — decisions are not assignments to resources |
| Hard constraints | Risk limits, position limits, regulatory constraints |
| Objectives | Risk-adjusted return, decision confidence, explanation quality |
| Planning Horizon | Intraday to multi-session |
| Output | Decision recommendation with explanation and confidence |
| Platform capabilities | Decision Intelligence (coralys-decision), Simulation (coralys-simulation), Recommendation (coralys-recommendation) |
| Benchmarks | Historical market scenarios (see `chronology/`) |
| Status | Active development — financial domain library |

---

## 6. Relationship to Architecture Principles

This guide implements the following principles and invariants from [`ARCHITECTURE_EVOLUTION.md`](ARCHITECTURE_EVOLUTION.md):

- **Principle 9** (Scale across dimensions): New domains are added through Domain Libraries without changing the platform.
- **AI-3** (Two implementations before promotion): A generic abstraction must have at least two Domain Library implementations before it is promoted to the platform.
- **AI-5** (Domain libraries own semantics): What a `Pairing` or `Shift` means is defined in the Domain Library, not in the Product.
- **AI-6** (No lateral dependencies): Domain Libraries never depend on one another.

---

*Add new Domain Library entries to Section 5 when a new domain is introduced. Complete the ten-question specification template in Section 4 before writing any code.*

---

## 7. Two Categories of Domain Library

The ten-question template in Section 4 is designed around planning problems. Not all domains are planning domains. Coralys supports two categories of Domain Library, and the correct template questions differ between them.

### Planning Domain Library

A domain where the primary decision problem is allocating Resources to work over a Planning Horizon.

Defining characteristics:
- Has Resources (nurses, crew members, vehicles, machines)
- Has Atomic Planning Units (shifts, pairings, routes, operations)
- Produces a Utilization Plan as output
- Consumes the Planning capability

Examples: INRC2 (nurse rostering), Airline Crew, CVRP (vehicle routing), Manufacturing, Retail.

All ten questions in Section 4 apply.

---

### Decision Domain Library

A domain where the primary decision problem is producing a recommendation or decision from evidence, without necessarily allocating resources.

Defining characteristics:
- Has Evidence (market data, sensor readings, observations)
- Has Models (hypotheses, scenarios, predictions)
- Produces a Decision Recommendation as output
- Consumes the Decision Intelligence capability
- May not have Resources or Planning Units at all

Examples: ChronoSentiment (financial decision intelligence).

For Decision Domain Libraries, questions 2 (Resources), 3 (Atomic Planning Units), 6 (Planning Horizon), and 8 (Utilization Plan) in Section 4 should be answered as "N/A — Decision Domain" rather than left blank.

---

## 8. Domain Libraries Must Not Expose Platform Implementation

A Domain Library answers: **"What does this domain know?"**

It must not answer: **"How is this optimized?"**

This rule is absolute. A Domain Library must never expose or depend on:
- Genetic algorithm types or parameters
- Mutation operators or crossover strategies
- NSGA-II, simulated annealing, or any other search algorithm
- Population sizes, generation counts, or convergence criteria
- Any type from `coralys-moga` or other optimization crates

**Why**: If a Domain Library imports optimization types, it becomes coupled to a specific algorithm. Replacing the optimizer (e.g. switching from MOGA to a constraint solver) would require changing the domain library — which is wrong. The domain library should be completely indifferent to how its scenarios are solved.

**Correct pattern**: The Domain Library defines the `Scenario`, `Constraint`, and `Objective` types. The Platform selects and executes the optimizer. The Domain Library never sees the optimizer.

**Example violation** (do not do this):
```rust
// WRONG — domain library importing platform implementation
use coralys_moga::Population;
use coralys_moga::MutationOperator;

impl InrcScenario {
    pub fn mutate(&self, pop: &Population) -> Population { ... }
}
```

**Correct pattern**:
```rust
// CORRECT — domain library defines semantics only
impl Scenario for InrcScenario {
    fn evaluate(&self, solution: &Solution) -> Fitness { ... }
    fn constraints(&self) -> Vec<Constraint> { ... }
    fn objectives(&self) -> Vec<Objective> { ... }
}
```

The optimizer calls `evaluate`. The domain library never calls the optimizer.