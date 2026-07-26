# Architecture Evolution — Constitutional Governance Document

> **Date**: 2026-07-19 (frozen 2026-07-22)
> **Version**: 2.0 — Constitutional Governance Baseline
> **Purpose**: Record architectural principles, decisions, implementation evidence, and the formal closure of the architecture program.
> **Relationship**: All factual claims in this document are grounded in [`CODEBASE_ASSESSMENT.md`](CODEBASE_ASSESSMENT.md). When evidence and proposal conflict, the assessment wins.
> **Standard**: This document is frozen. It is updated only when genuine architectural evidence justifies changing the constitutional baseline. Active engineering records use P-series milestones.

---

## Architectural Principles

These principles are stated before any proposals. They constrain every recommendation.

**Principle 1 — Platform crates never depend upward.**
Dependency direction is always toward the platform:

```
Application
    ↓
Solution Adapter (Domain Library)
    ↓
Platform
```

Platform crates must never import Solution Adapters, Products, or Applications. A platform crate importing any type from `adapters/`, `services/`, or any product crate is a violation. The full three-layer hierarchy is enforced by Rules 1, 5, and 6 in [`PLATFORM_CRATE_RESPONSIBILITIES.md`](PLATFORM_CRATE_RESPONSIBILITIES.md).

**Principle 2 — Solution adapters depend on platform capabilities expressed through interfaces, never on algorithms or concrete implementations.**
A solution adapter such as `adapters/ultracrew` depends on platform capability interfaces, not on specific optimization algorithms or concrete platform implementations. The MOGA engine is injected, not imported directly. Applications interact only with the solution adapter; the solution adapter composes platform capabilities through stable interfaces. Platform capabilities remain replaceable without affecting application code.

**Principle 3 — Benchmarks are permanent.**  
Research implementations are never discarded. The INRC2 benchmark suite is a regression suite, a validation dataset, a performance benchmark, and research evidence. It is preserved in full regardless of any refactoring.

**Principle 4 — Generalize after two concrete implementations, not one.**  
A shared abstraction extracted from a single implementation is shaped almost entirely by that implementation. The correct sequence is: implement concretely twice, compare at the right abstraction level, then extract the shared interface. This principle directly governs the timing of any generic scheduling or workforce abstraction.

**Principle 5 — Distinguish domain libraries from products.**
A domain library (e.g. the airline scheduling domain in `coralys-scheduling`) is not the same as a product (e.g. AirlineOps). The domain library is a reusable asset. The product is the application layer built on top of it. These are separate concerns and should be separate crates.

**Principle 6 — Name things for their long-term role, not their current contents.**
A crate should not be renamed based solely on what it currently contains. The rename decision requires clarity about the crate's intended long-term role. Current contents are evidence; they are not the decision.

**Principle 7 — Cross-domain abstractions are compared at the level of the Atomic Planning Unit.**

> **Atomic Planning Unit**: the smallest schedulable unit that is directly assigned to a worker within a planning problem. Domain-specific structures may exist below this level, but they are not exposed through the generic planning interface.

When comparing two scheduling domains to derive a shared abstraction, the comparison must be made at the Atomic Planning Unit level — not at the level of the full domain hierarchy. Comparing `Shift` (INRC2) with `Pairing` (airline) directly conflates different abstraction levels. The correct comparison is: what is the Atomic Planning Unit in each domain, and what do those units have in common?

**Principle 8 — Preserve the separation between Optimization, Planning, and Decision Intelligence.**
These are three distinct capabilities. Optimization finds solutions in a search space. Planning allocates Resources to work over a horizon. Decision Intelligence explains, compares, and recommends. Collapsing them into a single universal abstraction sacrifices the composability that gives the platform its long-term strength.

**Principle 9 — Scale across all dimensions without architectural redesign.**
The platform shall scale across problem size, domain breadth, computational resources, architectural composition, and decision complexity. New domains are added through Domain Libraries without changing the platform. New capabilities are composed without changing existing ones. New computational targets (multi-core, distributed, cloud, accelerators) are supported through the platform's execution infrastructure, not through product-level changes.

| Dimension | Meaning |
|---|---|
| Problem | From small to enterprise-scale optimization problems |
| Domain | New industries added through Domain Libraries, not platform changes |
| Computational | Multi-core, distributed, cloud, accelerators |
| Architectural | New capabilities composed without changing existing ones |
| Decision | From optimization-only to complete decision workflows |

**Principle 10 — Domains are modeled faithfully.**
The platform exists to provide the minimal stable execution contract shared across domains. Domain Libraries must model their domains faithfully. If implementing the platform contract requires simplifying, renaming, or restructuring domain concepts solely to satisfy the platform, the contract — not the domain — should be reconsidered. Coralys does not ask "How do we fit this domain into Coralys?" It asks "Is Coralys generic enough that this domain can express itself without compromise?"

---

## Architectural Invariants

Invariants are stronger than principles. They are enforced in code review and must not be violated without an explicit architectural decision recorded in this document.

**AI-1 — No platform crate imports a product crate.**
Violation: any `use` statement in a `coralys-*` crate that references a type from `adapters/ultracrew`, `services/ultracrew_server`, or any other product crate.

**AI-2 — No benchmark implementation is deleted.**
The INRC2 benchmark suite, CVRP benchmark suite, and any future benchmark implementations are permanent. Refactoring may move them; it may not remove them.

**AI-3 — Every generic abstraction must have at least two independent implementations before it is promoted to the platform.**
A trait or interface with only one implementation is not yet proven generic. It lives in the domain library until a second implementation validates the abstraction boundary.

**AI-4 — Solution adapters define decision workflows by composing platform capabilities through the common execution contract. Platform owns capability execution.**
The three-layer responsibility model is:
- Applications orchestrate user interactions (HTTP, CLI, auth, persistence, DTO mapping).
- Solution adapters orchestrate business workflows (domain logic, constraint evaluation, optimization configuration, simulation, recommendation).
- Platform executes capabilities (MOGA engine, simulation engine, decision intelligence, recommendation engine).

Solution adapters define which platform capabilities are needed and in what order. The platform executes those capabilities through the common execution contract (`coralys-core`). The optimization algorithm, simulation engine, or recommendation logic that executes within a step is a platform concern. Solution adapters must not implement capability execution themselves. Applications must not bypass the solution adapter to invoke platform capabilities directly (see Rule 5 in [`PLATFORM_CRATE_RESPONSIBILITIES.md`](PLATFORM_CRATE_RESPONSIBILITIES.md)).

**AI-5 — Domain libraries own semantics. Products own user experience.**
What a `Pairing` or a `Shift` means is defined in the domain library. How a planner interacts with pairings or shifts is defined in the product. These must not be mixed.

**AI-6 — Domain libraries do not depend on one another.**
`adapters/ultracrew` (INRC2) must not import types from `coralys-scheduling` (airline), and vice versa. Both depend downward toward the platform. Never sideways toward each other. Lateral coupling between domain libraries creates hidden constraints that prevent independent evolution of each domain.

**AI-7 — Platform interfaces evolve compatibly.**
Breaking changes to a platform interface require an explicit architectural decision recorded in this document. Benchmark adapters must continue to compile after any platform interface change. Domain libraries must not be required to rewrite large portions of code due to arbitrary platform API churn. Stability of the platform interface is a first-class concern.

---

## Layer Model

The intended dependency graph, from bottom to top:

```
┌─────────────────────────────────────────────────────────┐
│  Platform                                               │
│  coralys-core  coralys-moga  coralys-planning            │
└───────────────────────┬─────────────────────────────────┘
                        │
┌───────────────────────▼─────────────────────────────────┐
│  Solution Adapters (Domain Libraries)                   │
│  adapters/ultracrew (INRC2)   adapters/airline          │
│  adapters/cvrp                adapters/chronosentiment  │
└───────────────────────┬─────────────────────────────────┘
                        │
┌───────────────────────▼─────────────────────────────────┐
│  Products                                               │
│  UltraCrew            AirlineOps      ChronoSentiment   │
└───────────────────────┬─────────────────────────────────┘
                        │
┌───────────────────────▼─────────────────────────────────┐
│  Applications / Deployments                             │
│  REST servers   CLI tools   Desktop   Cloud             │
└─────────────────────────────────────────────────────────┘
```

Products and Applications are separate layers. `services/ultracrew_server` is an Application (a deployment artifact), not the UltraCrew product itself. The product is the domain library and business logic; the application is how it is deployed.

---

## Current State vs Target State

### UltraCrew

| Dimension | Current State | Target State |
|---|---|---|
| Domain | INRC2 nurse rostering | Workforce planning product built on the generic planning capability |
| Generic layer | `src/workforce/` stub (3 files, not functional) | Functional generic layer implementing a platform interface |
| Server coupling | `ultracrew_server` imports `InrcScenario` directly | Server depends on a generic scheduling interface; INRC2 injected at startup |
| Benchmark suite | INRC2 instances n030–n120, mature | Preserved in full; becomes regression suite for generic layer |

### AirlineOps

| Dimension | Current State | Target State |
|---|---|---|
| Domain model | Exists in `coralys-scheduling` (FlightLeg, Duty, Pairing, Rotation, FDP) | Same crate, possibly renamed |
| Product layer | Does not exist | Pairing optimizer, crew assignment, recovery, crew control |
| Crew assignment | Not implemented | Uses generic planning capability (not UltraCrew the product) |

### `adapters/airline`

| Dimension | Current State | Target State |
|---|---|---|
| Contents | Airline domain implementation | Airline domain library — permanently airline-specific (OQ-1 resolved) |
| Name | `coralys-airline` at `adapters/airline` | Complete — renamed from `coralys-scheduling` (Phase A, 2026-07-22) |

### Platform

| Dimension | Current State | Target State |
|---|---|---|
| Generic planning capability | Does not exist | `coralys-planning` — new platform crate (OQ-2, OQ-4 resolved) |
| `coralys-moga` | Mature, generic | Unchanged |
| `coralys-core` | Mature, generic | Unchanged |
| Stub crates | 7 stubs | Progressively implemented as products need them |

---

## Open Questions

These questions must be answered before the corresponding proposals can become decisions.

### OQ-1: What is the long-term role of `coralys-scheduling`?

**Evidence** (from [`CODEBASE_ASSESSMENT.md`](CODEBASE_ASSESSMENT.md) §4):
- The crate currently exports only airline types.
- Its own documentation says "Airline crew scheduling domain model."
- It has no dependency on INRC or generic workforce types.

**Two possible futures**:

Option A — Permanently airline-specific: rename to `coralys-airline` or `adapters/airline`. Correct if the crate is intended to remain the airline domain model forever.

Option B — Generic scheduling framework: preserve the name `coralys-scheduling`. The airline implementation came first; the crate will eventually contain generic scheduling abstractions with airline as one concrete implementation.

**Decision**: **Option A — Permanently airline-specific.** Resolved in Phase 1 (2026-07-21). The airline domain model in `coralys-scheduling` (`FlightLeg`, `Duty`, `Pairing`, `BaseReturnRule`, `FDP`, etc.) is not a generic scheduling framework — it is an airline domain library. The crate should be renamed to `adapters/airline` (or `coralys-airline`) to accurately reflect its role, parallel to `adapters/ultracrew` as the INRC2 domain library. The generic planning capability belongs in a new platform crate (`coralys-planning` — see OQ-2, OQ-4). Evidence and full rationale: [`docs/PHASE1_DOMAIN_COMPARISON.md`](PHASE1_DOMAIN_COMPARISON.md) §5 OQ-1.

---

### OQ-2: What is the correct home for the generic assignment abstraction?

**Evidence**: `coralys-workforce` does not exist. `adapters/ultracrew/src/workforce/` is a 3-file stub.

**Candidate architectures**:

Candidate A — New platform crate (e.g. `coralys-workforce` or `coralys-assignment`):
```
Manufacturing adapter  →  coralys-workforce  →  coralys-core
INRC2 adapter          →  coralys-workforce
AirlineOps             →  coralys-workforce
```
Advantage: clean dependency direction. No product becomes a platform.

Candidate B — Inside `adapters/ultracrew/src/workforce/`:
```
Manufacturing adapter  →  adapters/ultracrew  →  coralys-core
AirlineOps             →  adapters/ultracrew
```
Problem: UltraCrew becomes a de facto platform. Violates Principle 1.

**Proposal**: Candidate A. The generic abstraction belongs in the platform layer, not inside a product namespace. The crate name should be determined after Phase 1 identifies the correct abstraction boundary.

**Decision**: **Candidate A — New platform crate named `coralys-planning`.** Resolved in Phase 1 (2026-07-21). The shared interface (`PlanningScenario`, `Worker`, `PlanningUnit`, `CoverageDemand`, `PlanningSolution`) belongs in a new platform crate. Dependency direction: `adapters/ultracrew → coralys-planning → coralys-core` and `adapters/airline → coralys-planning`. This satisfies AI-3 (two independent implementations: INRC2 and airline). Evidence: [`docs/PHASE1_DOMAIN_COMPARISON.md`](PHASE1_DOMAIN_COMPARISON.md) §5 OQ-2.

---

### OQ-3: What is the correct abstraction level for comparing INRC2 and airline domains?

**Evidence** (from [`CODEBASE_ASSESSMENT.md`](CODEBASE_ASSESSMENT.md) §3 and §4):

INRC2 domain hierarchy:
```
Nurse → Shift (single work unit)
```

Airline domain hierarchy:
```
Crew Member → Flight Leg → Duty → Pairing → Rotation
```

These are not at the same abstraction level. A direct comparison of `Shift` (INRC2) with `Pairing` (airline) would conflate different concepts.

**Proposal**: Frame the comparison around the **assignable work unit** in each domain — the atomic thing that gets assigned to a worker. In INRC2 that is a `Shift`. In the airline domain, the assignable work unit for crew rostering is a `Pairing` (a base-to-base trip). The `FlightLeg` and `Duty` are sub-components of a pairing, not the unit of assignment.

This framing gives a cleaner conceptual bridge:

| Domain | Worker | Atomic Planning Unit | Sub-components | Planning Horizon |
|---|---|---|---|---|
| INRC2 | Nurse | Shift | (none — shift is atomic) | Multi-week (4–8 weeks typical) |
| Airline crew rostering | Crew member | Pairing | Duty → Flight Leg | Schedule period (months) |
| Manufacturing | Worker | Shift / task | (domain-specific) | Shift cycle |
| Retail | Staff member | Shift | (none) | Roster cycle (weeks) |

The generic abstraction should be defined at the `(Worker, AssignableWorkUnit)` level. Domain-specific sub-structure lives below that interface.

Note: the optimizer is not merely assigning — it is optimizing legality, sequencing, workload, fairness, preferences, coverage, and recovery. "Assignment" is one aspect of the problem. The correct name for the abstraction may turn out to be `PlanningScenario`, `AllocationScenario`, or something else. This framing identifies the right comparison level; it does not commit to a name.

**Decision**: **Resolved in Phase 1 (2026-07-21).** INRC2 Atomic Planning Unit: shift assignment `(nurse_id, shift_type_id, day_index)` — atomic, no sub-structure. Airline Atomic Planning Unit: `Pairing` — base-to-base trip; `FlightLeg` and `Duty` are sub-components, not the unit of assignment. The comparison is correctly framed at the `(Worker, PlanningUnit)` level. Evidence: [`docs/PHASE1_DOMAIN_COMPARISON.md`](PHASE1_DOMAIN_COMPARISON.md) §5 OQ-3.

---

### OQ-4: What should the generic abstraction be named?

**Working hypothesis**: Use a neutral name during Phase 1 — `AssignmentScenario` or `SchedulingScenario` — rather than `WorkforceScenario`. The name `WorkforceScenario` commits to a particular domain boundary before the comparison is complete.

This is a working hypothesis, not a proposal. The name should emerge from Phase 1 analysis, not precede it. Once Phase 1 identifies the correct abstraction boundary, the name follows from the boundary.

**Decision**: **`coralys-planning`** — resolved in Phase 1 (2026-07-21). The name is neutral (not workforce-specific, not airline-specific), consistent with D-7 (planning answers *who/what performs the work?*), and does not conflict with `coralys-scheduling` (which will be renamed to `adapters/airline`). Core trait: `PlanningScenario`. Evidence: [`docs/PHASE1_DOMAIN_COMPARISON.md`](PHASE1_DOMAIN_COMPARISON.md) §5 OQ-4.

---

> **Historical Record**
>
> The migration phases below are retained as the historical implementation plan that produced Architecture Baseline v1.0. They are complete and are preserved for traceability. Active engineering work is now tracked exclusively through the P-series milestones.
## Migration Phases

### Phase 0 — Documentation Baseline

1. Freeze [`CODEBASE_ASSESSMENT.md`](CODEBASE_ASSESSMENT.md) — immutable record of current state.
2. Maintain this document as the living evolution plan.
3. Update [`PRODUCT_PORTFOLIO.md`](../PRODUCT_PORTFOLIO.md) to distinguish current state from target state.
4. No code changes in Phase 0.

**Exit criterion**: Both documents exist and are consistent with each other.

---

### Phase 1 — Validate common abstractions (1–2 weeks)

**Goal**: Answer OQ-1, OQ-2, OQ-3, OQ-4 with evidence from the code.

**Work**:
1. Read `adapters/ultracrew/src/inrc/models.rs` in full.
2. Read `coralys-scheduling/src/domain/` in full.
3. Identify the assignable work unit in each domain (per OQ-3 framing).
4. Map shared concepts: worker/crew, assignable unit, constraint, objective, planning horizon.
5. Map domain-specific concepts: FDP, pairing, duty period (airline) vs. INRC contract types, forbidden successors, multi-week history (INRC).
6. Draft the candidate planning interface based on the intersection (name TBD — see OQ-4).
7. Decide on `coralys-scheduling` scope (OQ-1).
8. Decide on the home and name of the generic abstraction (OQ-2, OQ-4).

**Deliverable**: A Phase 1 comparison document recording the findings. This document feeds the Phase 2 implementation.

**No code changes in Phase 1.** Analysis only.

**Exit evidence** (all must be present before Phase 2 begins):
- [x] Domain comparison matrix completed (INRC2 vs airline, at Atomic Planning Unit level)
- [x] OQ-1 resolved: `coralys-scheduling` long-term scope decided — Option A (→ `adapters/airline`)
- [x] OQ-2 resolved: home of the generic planning capability decided — `coralys-planning`
- [x] OQ-3 resolved: Atomic Planning Unit identified in each domain — shift assignment (INRC2), Pairing (airline)
- [x] OQ-4 resolved: candidate name for the generic interface agreed — `coralys-planning`, trait `PlanningScenario`
- [x] Candidate interface sketch reviewed and accepted — see [`docs/PHASE1_DOMAIN_COMPARISON.md`](PHASE1_DOMAIN_COMPARISON.md) §6

**Phase 1 complete (2026-07-21).**

---

### Phase 2 — Create the generic planning capability (1 week)

**Precondition**: Phase 1 complete. OQ-1 through OQ-4 decided.

**Work**:
1. Create the platform crate identified in Phase 1 (name and abstraction boundary determined by Phase 1 analysis).
2. Add to workspace manifest.
3. Define the Phase 1 planning interface (`PlanningScenario`, `PlanningUnit`, `Worker`, `CoverageDemand`, `PlanningSolution`).
4. Implement the trait for `InrcScenario` — all existing INRC2 benchmark parity tests must pass (Principle 3, AI-2).
5. If `coralys-scheduling` is renamed (OQ-1 Option A): update `Cargo.toml`, package name, dependency declarations, use paths, documentation, CI, examples.

**Exit evidence** (all must be present before Phase 3 begins):
- [x] Generic planning capability crate exists in workspace manifest — `coralys-planning` (M-002, 2026-07-22)
- [x] Core types and traits defined and documented — `Worker`, `PlanningUnit`, `CoverageDemand`, `PlanningSolution`, `PlanningScenario` (M-002, 2026-07-22)
- [x] `InrcScenario` implements the planning capability trait — `InrcPlanningScenario` in `adapters/ultracrew/src/inrc/planning.rs` (M-002, 2026-07-22)
- [x] All INRC2 benchmark parity tests pass unchanged (AI-2) — zero regression confirmed (M-002, 2026-07-22)
- [x] Public API documented — traits documented in `coralys-planning/src/lib.rs` (M-002, 2026-07-22)
- [x] `coralys-scheduling` rename completed — renamed to `adapters/airline` (`coralys-airline`) (Phase A, 2026-07-22)

**Constraint**: The Phase 2 API is a minimal execution contract, not a comprehensive planning framework. Additional concepts are promoted to the platform only when multiple implementations demonstrate they belong there (Principle 4). Preferences, histories, recovery policies, temporal windows, geographic constraints, qualification matrices, and optimization strategy are not part of the Phase 2 API. They remain in the domain libraries until a second implementation validates that they are genuinely shared.

**Reminders**:
- Reminder 9: Application layer reduced to orchestration
- Reminder 10: Platform layer owns reusable execution capabilities
- Reminder 11: Implementation conforms to architectural dependency direction

---

### Phase 3 — Decouple `ultracrew_server` from `InrcScenario` (1 week)

**Precondition**: Phase 2 complete.

**Work**:
1. Replace `use ultracrew::inrc::models::InrcScenario` in [`services/ultracrew_server/src/optimizer.rs`](../services/ultracrew_server/src/optimizer.rs) with the generic planning capability interface from Phase 2.
2. Make the fitness function generic over constraint weights (pass via configuration, not hardcoded INRC names).
3. Inject the INRC2 adapter at server startup.
4. All existing server tests must pass.

**Exit evidence** (all must be present before Phase 4 begins):
- [x] `ultracrew_server` has no direct import of any INRC2 type — production binary imports only `adapters/ultracrew` public API (M-004, 2026-07-22)
- [x] Fitness function is generic over constraint weights — INRC-specific names are encapsulated inside the adapter; the server sees only `InrcParetoSolution` fields (M-004, 2026-07-22)
- [x] INRC2 adapter integrated at application startup — adapter composition performed through the UltraCrew Solution Adapter (M-004, 2026-07-22). Note: configuration-driven adapter selection is an engineering enhancement tracked under P-001 Stream 2 and is not required for architectural completion.
- [x] All existing server tests pass — 5 passed, 0 failed (M-004, 2026-07-22)

---

### Phase 4 — Stream B application modules (ongoing)

**Precondition**: Phase 3 complete.

**Goal**: Build the four UltraCrew application modules against the generic planning capability interface produced in Phase 2.

| Module | Description | Generic framing |
|---|---|---|
| B1 — Planner Workspace | Interactive roster construction and editing | Any industry |
| B2 — Disruption Console | Response to worker unavailability | Any industry |
| B3 — Explanation Engine | Assignment rationale and constraint audit | Any industry |
| B4 — Operational Readiness | Worker qualification, certification, availability | Any industry |

The INRC2 benchmark suite validates correctness throughout. No module should import INRC2 types directly.

---

### Phase 5 — AirlineOps product layer (future)

**Precondition**: Phase 1 complete (domain comparison done). Phase 2 complete (generic abstraction exists).

**Goal**: Build the AirlineOps product layer on top of the airline domain model.

**Work**:
1. Pairing optimizer (builds legal pairings from a flight schedule).
2. Crew assignment — uses the generic `PlanningScenario` capability, treating pairings as assignable work units. Does not depend on UltraCrew the product.
3. Crew recovery (disruption re-optimization).
4. Crew control (real-time replacement decisions).

**Note on AirlineOps and UltraCrew**: AirlineOps uses the **generic planning capability** (the platform interface from Phase 2), not UltraCrew the product. Whether UltraCrew's server infrastructure is reused as a deployment vehicle is an operational decision, not an architectural one. Architecturally, AirlineOps depends on a platform interface.

---

## Proposals — Resolved by Phase 1

| ID | Proposal | Status | Blocking |
|---|---|---|---|
| P-1 | Create a platform-level planning capability crate using Resource not Worker | **Resolved** — `coralys-planning` approved (D-7, OQ-2). Phase 1 (2026-07-21). | Phase 1 |
| P-2 | Use neutral working name during Phase 1 (not `WorkforceScenario` or `AssignmentScenario`) | **Resolved** — name settled as `coralys-planning` / `PlanningScenario` (OQ-4). Phase 1 (2026-07-21). | Phase 1 |
| P-3 | `coralys-scheduling` long-term scope: becomes generic (name preserved) | **Resolved — rejected.** Option A chosen: permanently airline-specific, rename to `adapters/airline` (OQ-1). Phase 1 (2026-07-21). | Phase 1 |
| P-4 | Decouple `ultracrew_server` from `InrcScenario` | Accepted in principle | Phase 2 |
| P-5 | AirlineOps uses platform interface, not UltraCrew product | Accepted in principle | Phase 5 |

---

## Decisions Recorded

| ID | Decision | Date | Rationale |
|---|---|---|---|
| D-1 | Do not rename `coralys-scheduling` before Phase 1 | 2026-07-19 | Principle 6: name for long-term role, not current contents. Intent unclear at time of decision. |
| D-2 | Generic abstraction belongs in platform layer, not in `adapters/ultracrew` | 2026-07-19 | Principle 1: platform crates never depend on products. |
| D-3 | Generalize after comparing INRC2 and airline domains, not from INRC2 alone | 2026-07-19 | Principle 4: generalize after two concrete implementations. |
| D-4 | Compare domains at the assignable work unit level, not at the full hierarchy level | 2026-07-19 | INRC2 `Shift` and airline `Pairing` are at different abstraction levels; direct comparison would conflate them. |
| D-5 | The planning capability is not workforce-specific — it is about Resources broadly | 2026-07-19 | `Worker` is a specialization of `Resource`. The platform must support planning problems involving non-human resources (aircraft, vehicles, equipment, rooms). Platform planning vocabulary uses Resource, not Worker. This applies to the planning layer only — not to the full platform (see OQ-5, OQ-6). |
| D-7 | Working model: Scheduling answers *when does work happen?* Planning answers *who/what performs the work?* | 2026-07-19 | `coralys-scheduling` models the work to be performed (temporal). The planning capability allocates Resources to that work (resource allocation). This separation makes `coralys-scheduling` an excellent name for the work-modeling crate. |
| D-8 | Application crates must not depend directly on `coralys-*` platform crates — Rule 5 added to `PLATFORM_CRATE_RESPONSIBILITIES.md` | 2026-07-21 | The dependency hierarchy must pass through the solution adapter: Application → Solution Adapter → `coralys-*`. Direct application-to-platform dependencies bypass the domain encapsulation boundary, couple application concerns to optimization internals, and prevent business logic reuse across multiple applications. This rule applies platform-wide to all Coralys-based applications (UltraCrew, ChronoSentiment, future products). Triggered by Phase C-B boundary assessment of `ultracrew_server`, which currently imports `coralys_moga` directly in `optimizer.rs`. |
| D-9 | Solution adapters must not expose platform implementation details to the application layer — Rule 6 added to `PLATFORM_CRATE_RESPONSIBILITIES.md`; "Solution Adapter" added to `ARCHITECTURE_GLOSSARY.md` v1.2 | 2026-07-21 | Rule 6 is the symmetric complement to Rule 5. Rule 5 prevents applications from bypassing the adapter (dependency direction). Rule 6 prevents the adapter from leaking platform internals upward (encapsulation direction). Together they define a complete, bidirectional boundary. "Solution Adapter" is introduced as the canonical term for a Domain Library when emphasizing its role as the integration boundary — the two terms refer to the same crate. This terminology applies uniformly to all Coralys-based products. |

---

## Open Question: OQ-5 — Common Execution Contract vs Pipeline vs Ecosystem

The existing platform crates suggest a pipeline shape, but a fixed pipeline forces all products into the same workflow. A pure ecosystem of independent services loses platform value.

**Candidate model**: Composable capabilities with a common execution contract. Each capability consumes a `Scenario` and produces an `Outcome` (the contract already suggested by `coralys-core` traits: `Scenario`, `Solution`, `Outcome`, `DecisionPlugin`, `ReasoningEngine`). Products define which capabilities they need and in what order. The platform provides the execution infrastructure but not a fixed pipeline.

**Implication**: `coralys-core` is not merely a traits library — it is the execution contract that makes all capabilities interoperable.

**Decision**: Pending. Does not block Phase 1.

---

## Open Question: OQ-6 — Coralys Platform Identity and Two-Layer Architecture

The architecture has evolved beyond "Decision Optimization Platform." Two stable abstraction levels are emerging:

**Decision layer** (universal — applies to all products):
```
Scenario → Decision Workflow → Outcome
```
ChronoSentiment and UltraCrew both live here.

**Planning layer** (planning products only):
```
Resources → Planning Units → Utilization Plan
```
UltraCrew and AirlineOps live here. ChronoSentiment does not.

**Candidate platform identity**: Coralys is an **Adaptive Decision Systems Platform** — a platform for building scalable decision systems that combines optimization, planning, simulation, evaluation, recommendation, and decision intelligence to produce high-quality decisions under real-world constraints within operational time limits.

**What this is not**: an AI platform (AI is a technique), an optimization platform (optimization is one capability), a scheduling platform (scheduling is one domain).

**Decision**: Pending. Affects `PRODUCT_PORTFOLIO.md` branding. Does not block Phase 1.

---

## Resolved: OQ-7 — Repository Taxonomy Migration

The repository currently uses `adapters/` and `services/` as top-level directory names. These names no longer reflect the architectural vocabulary established in this baseline.

**Target taxonomy** (post-Phase-1 migration):

```
platform/       coralys-core, coralys-moga, coralys-simulation, coralys-decision, ...
domains/        inrc, airline, cvrp, chronosentiment
products/       ultracrew, airlineops, chronosentiment
applications/   ultracrew-server, airlineops-server
```

**Current mapping:**
- `adapters/ultracrew` → `domains/inrc` (INRC2 Domain Library)
- `adapters/cvrp` → `domains/cvrp` (CVRP Domain Library)
- `coralys-scheduling` → `adapters/airline` (Airline Domain Library) — **complete (Phase A, 2026-07-22)**
- `services/ultracrew_server` → `applications/ultracrew-server`
- `coralys-*` crates → `platform/`

**Decision**: Architectural decision complete (2026-07-21, OQ-1). `coralys-scheduling` was permanently airline-specific and has been renamed to `adapters/airline` (package name `coralys-airline`). Repository migration complete (Phase A, 2026-07-22): `git mv coralys-scheduling adapters/airline`, package name updated, all test `use` paths updated, full test suite passing (182 tests, 0 failures). Remaining taxonomy migration (`adapters/` → `domains/`, `services/` → `applications/`) is deferred — not on the critical path for P-001.

---

## Open Question: OQ-8 — "Product" terminology ambiguity (backlog)

The document currently has four concepts: Platform, Solution Adapter, Product, and Application. The relationship between Product and Solution Adapter is slightly ambiguous. Principle 5 says the product is built on top of the domain library. The Layer Model shows Platform → Solution Adapter → Product → Application. AI-4 describes responsibilities only for Application, Solution Adapter, and Platform — Product does not appear.

This suggests Product is currently a conceptual/business layer rather than a code artifact, which is reasonable but should be made explicit.

**Proposed future revision**: Add to the glossary and Principle 5:

> A Product is the business capability delivered to customers. It may be realized by one or more Solution Adapters and one or more Applications. Product is a conceptual/business layer, not a code artifact.

This would clarify that "Product" describes what is sold and experienced, while "Solution Adapter" and "Application" describe how it is implemented and deployed.

**Decision**: Deferred. Does not affect any current enforcement rules. Address in a future revision when the Product/Application distinction becomes relevant to a concrete engineering decision.

---

## Open Question: OQ-9 — Phase 3 objective should be rephrased post-D-8 (backlog)

Phase 3 currently states: "Decouple `ultracrew_server` from `InrcScenario`."

After D-8 and D-9, the stronger architectural objective is: "Ensure `ultracrew_server` depends only on the UltraCrew Solution Adapter." Whether the adapter internally uses `InrcScenario` is an implementation detail of the adapter, not a concern of the application layer.

**Proposed future revision**: Rephrase Phase 3 objective to:

> Remove all direct platform dependencies from `ultracrew_server`. Remove all direct domain implementation dependencies. Ensure `ultracrew_server` interacts exclusively through the UltraCrew Solution Adapter public API.

This aligns the migration goal with Rule 5 and D-8.

**Decision**: Resolved (2026-07-22, M-004). Phase 3 was implemented under the stronger objective: `ultracrew_server` now depends exclusively on the UltraCrew Solution Adapter public API. The Phase 3 section heading retains the original wording for historical continuity, but the implementation satisfied the rephrased objective. No further action required.

---

## Implementation Milestone: M-001 — Solution Adapter Boundary Established

**Date**: 2026-07-21

UltraCrew domain validation, optimization, scoring, and domain types were extracted from `ultracrew_server` into the UltraCrew Solution Adapter (`adapters/ultracrew`). The application now orchestrates HTTP requests while the Solution Adapter owns business workflows and platform composition. This is the first production implementation of Rules 5 and 6.

**Files moved to `adapters/ultracrew/src/inrc/`:**
- `types.rs` — `ViolationDetail`, `ValidationReport` (pure domain types)
- `validator.rs` — `validate_schedule()` (INRC constraint checker)
- `schedule_optimizer.rs` — `ScheduleGenome`, `UltraCrewEvaluator`, `UltraCrewMutator`
- `observer.rs` — `score_inrc_official()`, `to_inrc_genome()`, `InrcScoreComponents`

**Build verification**: `cargo build -p ultracrew` → exit 0; `cargo build -p ultracrew_server` → exit 0.

**Significance**: This is the point where the governance stops being purely constitutional and becomes observable in the implementation. Future products (AirlineOps, ChronoSentiment) can now follow an implementation pattern that has been exercised rather than one that exists only in documentation.

### M-001 Audit — Remaining Direct `coralys_*` Imports in `ultracrew_server`

**Production server violations (Rule 5):**

| File | Import | Status |
|------|--------|--------|
| `src/optimizer.rs` line 4 | `coralys_moga::engine_proof::{Genome, Evaluator, MutationPolicy, FitnessVector}` | Redundant — canonical version now in adapter |
| `src/inrc_observer.rs` line 33 | `coralys_moga::traits::FitnessEvaluator` | Redundant — canonical version now in adapter |
| `src/main.rs` line 1174 | `coralys_moga::engine_proof::EvolutionEngine` | Server still runs the engine directly |

**Research/benchmark binaries (different category — AI-2):**

| File | Imports |
|------|---------|
| `src/bin/benchmark.rs` | `coralys_moga::engine_proof::*` |
| `src/bin/validation_pass.rs` | `coralys_moga::engine_proof::*`, `coralys_moga::ecology::*`, `coralys_moga::traits::*` |
| `src/bin/m9a_search_observatory.rs` | `coralys_moga::engine_proof::*`, `coralys_ecology::*` |
| `src/bin/m8g_ultracrew_validation.rs` | `coralys_moga::engine_proof::*` |
| `src/bin/ecology_validation.rs` | `coralys_moga::engine_proof::*`, `coralys_ecology::*`, `coralys_recommendation::*` |
| `src/bin/policy_seed_runner.rs` | `coralys_moga::*`, `coralys_ecology::*`, `coralys_recommendation::*`, `coralys_policy::*`, `coralys_core::*` |
| `src/bin/cs_governance_validation.rs` | `coralys_ecology::*`, `coralys_recommendation::*`, `coralys_policy::*`, `coralys_core::*` |
| `src/bin/inrc_archive_forensics.rs` | `coralys_moga::engine_proof::*`, `coralys_moga::ecology::*` |

The `bin/` files are research tools and benchmark runners — permanent research assets analogous to the INRC2 benchmark suite (AI-2). They are not subject to Rule 5 in the same way as the production server binary.

**Next steps (Phase C-B follow-up):**
1. Remove redundant `optimizer.rs` and `inrc_observer.rs` from `ultracrew_server/src/` once `main.rs` is updated to use adapter versions.
2. Update `main.rs` to use `ultracrew::inrc::schedule_optimizer::ScheduleGenome` and `ultracrew::inrc::observer::score_inrc_official` — eliminating the remaining Rule 5 violation.
3. Audit `services/ultracrew_server/Cargo.toml` for direct `coralys_*` dependencies and remove those no longer needed by the production binary.

---

## Implementation Milestone: M-002 — `coralys-planning` Execution Contract Established

**Date**: 2026-07-22

The `coralys-planning` platform crate was created and the Phase 1 planning interface was implemented. `InrcScenario` was adapted to `PlanningScenario` without semantic distortion. INRC2 benchmark parity was preserved.

**Files created:**
- `coralys-planning/Cargo.toml` — new platform crate, depends on `coralys-core`
- `coralys-planning/src/lib.rs` — five traits: `Worker`, `PlanningUnit`, `CoverageDemand`, `PlanningSolution`, `PlanningScenario`
- `adapters/ultracrew/src/inrc/planning.rs` — `PlanningScenario` implementation for `InrcScenario`

**Domain mapping (INRC2):**

| `coralys-planning` trait | INRC2 type |
|---|---|
| `Worker` | `InrcNurse` |
| `PlanningUnit` | `InrcShiftAssignment` (nurse × shift × day) |
| `CoverageDemand` | `InrcDemandSlot` (shift × skill × day → min/optimal) |
| `PlanningSolution` | `InrcSchedule` |
| `PlanningScenario` | `InrcPlanningScenario` wrapping `InrcScenario` |

**Governance validity:**
- Dependency direction preserved: `adapters/ultracrew → coralys-planning → coralys-core`
- `InrcScenario` implements `PlanningScenario` without semantic distortion (Principle 10)
- `level4_ecology_ablation` E0063 is pre-existing (missing `scenario` field in `ScheduleRequest`); not introduced by this milestone

**Benchmark results (AI-2 — zero regression):**
- `test_parity_snapshot` (parity_snapshot_n030w4) — **ok**
- `test_f2c_bronze_feasibility` (inrc_score_reproduction) — **ok** (195s)
- `inrc_history_transition` — **ok**

**Observations for future decisions:**
- The `InrcPlanningScenario` adapter pre-computes all `(nurse × shift × day)` planning units at construction time. For large scenarios this may warrant lazy evaluation — candidate for OQ-10 if a second domain reveals the same pattern.
- `PlanningSolution::assignments()` returns `impl Iterator` which requires RPIT. If a future capability needs object-safe `PlanningScenario`, the trait will need revision — candidate for a new decision record at that time.

---

## Architecture Baseline v1.0 — Freeze Declaration

**Date**: 2026-07-19

The platform architecture, terminology, layering, extension model, and governance model are considered stable as of this date.

Future architectural changes must be driven by implementation evidence rather than speculative refinement. The next phase focuses on validating the architecture through additional domain implementations and benchmark-driven development.

**Governance rules from this point:**
- No new architectural concepts without observable evidence from the codebase or a product implementation.
- No renaming for aesthetic reasons.
- No new platform abstractions without at least two independent implementations (Principle 4).
- All changes follow the process: Observe → Propose → Decide → Record.

The success criterion for Phase 1 is not "we found a common `PlanningUnit`." It is: **"Neither INRC2 nor Airline Crew had to distort its own semantics to implement the Coralys execution contract."**

---

## Implementation Milestone: M-003 — Architectural Migration Complete

**Date**: 2026-07-22

The repository is now internally consistent with the constitutional architecture. Phase A (rename `coralys-scheduling` → `adapters/airline`) completed the last outstanding architectural migration on the critical path.

**What was done:**
- `git mv coralys-scheduling adapters/airline` — directory moved, git history preserved
- `adapters/airline/Cargo.toml`: package name updated to `coralys-airline`
- Root `Cargo.toml`: workspace member path updated from `"coralys-scheduling"` to `"adapters/airline"`
- All five integration test files: `coralys_scheduling::` → `coralys_airline::` in `use` paths
- `docs/ARCHITECTURE_EVOLUTION.md`: Layer Model, Current State table, Phase 2 checklist, OQ-7 all updated to reflect implemented reality

**Verification:**
- `cargo build -p coralys-airline` — exit 0
- `cargo test -p coralys-airline` — 182 unit tests + 20 integration tests, 0 failures, 0 regressions

**Alignment achieved:**

| Artifact | Status |
|---|---|
| Repository structure | `adapters/airline` exists; `coralys-scheduling` removed |
| Cargo/workspace topology | Workspace and package names updated |
| Source code | Imports and integration tests updated |
| Architecture governance | `ARCHITECTURE_EVOLUTION.md` reflects implemented reality |

**Significance**: The architecture document now describes the repository as it actually exists, not as an intended future state. The layer model is realized:

```
Platform:          coralys-core  coralys-moga  coralys-planning
                         ↓
Solution Adapters: adapters/ultracrew  adapters/airline  adapters/cvrp  adapters/chronosentiment
                         ↓
Products:          UltraCrew  AirlineOps  ChronoSentiment
                         ↓
Applications:      REST servers  CLI tools  Desktop  Cloud
```

This matches the constitutional baseline exactly. The architectural migration is finished. The critical path is now product delivery.

---

## Dual-Track Execution Model

**Date**: 2026-07-22

With the architectural migration complete, the active execution model transitions from architecture-led to product-led. Two products run in parallel with different evidence goals and different effort allocations.

### Effort allocation

| Track | Effort | Product | Evidence goal |
|---|---|---|---|
| Commercial flagship | 80–90% | UltraCrew | External validation — pilot customers, measurable operational outcomes |
| Internal laboratory | 10–20% | ChronoSentiment | Self-validation — daily use, personal workflow improvement, iterative refinement |

### Rationale

UltraCrew and ChronoSentiment require different validation paths because they have different primary users today.

**UltraCrew**: The end users are airline planners, hospital schedulers, and operations teams — not the engineering team. Real validation requires external customers. The commercial flow is: WOA → WDX → UltraCrew Pilot → Measured Improvement → Commercial Deployment. Everything needed to begin this flow already exists: validated architecture, mature optimization engine, INRC2 benchmark evidence, GTM material, MSP definition, and identifiable pilot customers.

**ChronoSentiment**: The engineering team is part of the target audience. This makes it well-suited to self-validation before commercial investment. Daily use generates sustained real-world evidence on questions that matter: Does it improve trading decision quality? Does it help avoid poor trades? Does post-trade analysis become more insightful? Does the execution ecology reveal patterns not otherwise visible? Would a practitioner continue using it every day?

### How the tracks reinforce each other

ChronoSentiment daily use stress-tests Coralys capabilities — optimization, simulation, recommendation, explainability, execution workflows, observability. Improvements driven by that experience may later prove reusable in UltraCrew or AirlineOps, but only if multiple products independently need them. This is consistent with the two governing rules:

> **No new platform abstractions without implementation evidence.**
> **No major platform investment without product evidence.**

UltraCrew remains the commercial focus, so the commercial rule is respected. ChronoSentiment generates deep product insight through daily use, so the engineering rule is respected. Neither track competes with the other's objective.

### Product roles

| Product | Role | Path |
|---|---|---|
| UltraCrew | Commercial flagship | First commercial success; proves Coralys works in production |
| ChronoSentiment | Internal product laboratory | Matures through daily use; proves Coralys beyond workforce planning |
| AirlineOps | Future product | Built on `adapters/airline`; follows UltraCrew commercial validation |
| Coralys | Enabling platform | Evolves only when repeated product evidence demonstrates genuine shared need |

### Governing milestone

**P-001 — UltraCrew Minimum Sellable Product** remains the governing commercial milestone. ChronoSentiment commercial execution is deferred until UltraCrew reaches P-001 and completes its first pilot. At that point the team will have: a validated platform, a validated commercialization process, reusable deployment infrastructure, reusable GTM material, and credibility that carries into ChronoSentiment.

---

---
## Implementation Milestone: M-004 — Production Boundary Enforcement

**Date**: 2026-07-22

**Summary**

The UltraCrew production application now depends exclusively on the UltraCrew Solution Adapter public API. Platform implementation details (`coralys-moga`) are fully encapsulated within the adapter for the production binary. The application layer is reduced to orchestration responsibilities consistent with Rules 5 and 6.

### Changes

**Adapter — new capability**

`adapters/ultracrew/src/inrc/baseline.rs` — `generate_baseline_schedule` moved from the application layer into the adapter. INRC-specific baseline generation is now an internal adapter responsibility.

**Adapter — public contracts extended**

`adapters/ultracrew/src/public_contracts.rs` now exposes application-facing types:

- `InrcParetoSolution` — replaces direct exposure of `coralys_moga::engine_proof::ParetoSolution`
- `InrcStartupResult` — replaces direct exposure of `coralys_moga::engine_proof::EvolutionEngine` archive

**Adapter — pipeline encapsulation**

`adapters/ultracrew/src/pipeline.rs` now provides:

- `run_pipeline_from_request` — encapsulates `EvolutionConfig` construction; application passes tunable parameters, adapter builds the config
- `run_inrc_startup_pipeline` — encapsulates the 100-step Pareto engine run; application receives `InrcStartupResult`

**Server — production violations removed**

`services/ultracrew_server/src/main.rs` no longer imports:

- `coralys_moga::engine_proof::EvolutionEngine`
- `ultracrew::inrc::schedule_optimizer::{UltraCrewEvaluator, UltraCrewMutator, ScheduleGenome}`

Request handlers no longer construct `EvolutionConfig`. Startup no longer constructs or runs the evolutionary engine directly.

### Production dependency graph

```
ultracrew_server
        │
        ▼
adapters/ultracrew   ← coralys-moga is an internal implementation detail
        │
        ▼
coralys-planning / coralys-moga / coralys-core
```

`coralys-moga` remains in the server's `Cargo.toml` to support research `bin/` targets under AI-2. The production binary has zero direct `coralys_moga` references.

### Governance validity

This milestone does not alter the constitutional architecture. It is an implementation milestone: the production runtime dependency graph now conforms to the dependency direction established in the constitutional layer.

| Prior milestone | Scope |
|---|---|
| M-003 | Repository structure aligned with architecture |
| M-004 | Production runtime dependency graph aligned with architecture |

Together M-003 and M-004 demonstrate that both the codebase organisation and the executable application adhere to the constitutional architecture.

### Verification

`cargo test -p ultracrew_server --bin ultracrew_server`: **5 passed, 0 failed**.

---

## Architecture Program Closure

**Date**: 2026-07-22

**Status**: Complete

The Coralys platform architecture, layering model, governance model, dependency rules, extension strategy, and execution contract have been validated through implementation and repository convergence.

The document has transitioned from an **Architecture Working Document** to a **Constitutional Governance Document**. Constitutional documents change rarely and only with compelling evidence.

**Chain of validation:**

| Milestone | What it established |
|---|---|
| Architecture Baseline v1.0 | Constitutional principles |
| Phase 1 | Abstraction boundaries validated |
| Phase 2 / M-002 | `coralys-planning` execution contract; INRC2 implements without semantic distortion |
| M-001 | Solution Adapter pattern demonstrated in production |
| Phase A / M-003 | Repository aligned with architecture |
| Phase 3 / M-004 | Production runtime dependency graph aligned with architecture |
| Commercial Execution Baseline v1.0 | Governance shifted from architecture-led to product-led |
| Dual-Track Execution Model | UltraCrew and ChronoSentiment coexist without competing |

There is now a complete chain from architectural intent to implementation and commercial execution.

**Governing rules from this point:**

Future architectural changes shall originate from implementation evidence, benchmark evidence, pilot evidence, or repeated product evidence. Architecture shall no longer be treated as an independent workstream. Platform evolution follows product evolution.

**M-series milestone freeze:**

Implementation milestones are frozen at M-004. The M-series has served its purpose:

| Milestone | What it established |
|---|---|
| M-001 | Solution Adapter boundary |
| M-002 | Execution contract (`coralys-planning`) |
| M-003 | Repository conformance |
| M-004 | Production runtime conformance |

That completes the architectural implementation story. Everything after this is product implementation, not architecture implementation. Future engineering records use P-series milestones (P-001, P-001.1, P-001.2, …). No M-005 will be created.

**Governing milestone**: P-001 — UltraCrew Minimum Sellable Product.

**Governance document roles (frozen as of 2026-07-22):**

| Document | Role |
|---|---|
| `ARCHITECTURE_EVOLUTION.md` | Constitutional. Frozen except for evidence-driven architectural decisions. |
| `EXECUTION_DIRECTIVE_2026-07-22.md` | Transition document. Frozen as the formal handover from architecture to execution. |
| Commercial Execution Baseline | Active. Updated as sales, pilot, and go-to-market evidence accumulates. |
| P-series milestone records | Active. Primary engineering record from this point onward. |

---

*This document is updated as decisions are made and phases are completed. The assessment document ([`CODEBASE_ASSESSMENT.md`](CODEBASE_ASSESSMENT.md)) is immutable.*