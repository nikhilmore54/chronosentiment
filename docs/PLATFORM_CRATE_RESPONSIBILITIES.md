# Platform Crate Responsibilities

> **Status**: v1.0 — Architecture Baseline v1.0
> **Date**: 2026-07-19
> **Purpose**: One-page reference defining the responsibility, knowledge boundary, and forbidden dependencies of each platform crate. This is the constitution of the Coralys platform — the rules that keep platform crates composable and independently evolvable.
> **Relationship**: Enforced by AI-1 through AI-7 in [`ARCHITECTURE_EVOLUTION.md`](ARCHITECTURE_EVOLUTION.md). Vocabulary defined in [`ARCHITECTURE_GLOSSARY.md`](ARCHITECTURE_GLOSSARY.md).

---

## Platform Crate Responsibilities Table

| Crate | Responsibility | Knows About | Must Never Know About |
|---|---|---|---|
| `coralys-core` | Execution contract — the common interface all capabilities and domain libraries speak | `Scenario`, `Solution`, `Outcome`, `Fitness`, `Constraint`, `Objective`, `DecisionPlugin`, `ReasoningEngine` traits | Domain entities (nurses, flights, vehicles), product workflows, optimization algorithms |
| `coralys-moga` | Multi-objective genetic algorithm optimization engine | Search algorithms (NSGA-II, MOGA), population management, mutation, crossover, selection | Domain semantics, nurses, shifts, pairings, routes, product workflows |
| `coralys-simulation` | Scenario execution and what-if analysis | Simulation models, scenario replay, state transitions | Domain-specific entities, product workflows, optimization algorithms |
| `coralys-decision` | Decision intelligence — explanation, comparison, recommendation | Decision lineage, reasoning traces, explanation generation, confidence scoring | Domain-specific rules, product workflows, optimization internals |
| `coralys-eval` | Fitness and solution evaluation | Evaluation metrics, scoring functions, solution quality measurement | Domain semantics, product workflows, optimization algorithms |
| `coralys-ecology` | Population health monitoring and diversity management | Population diagnostics, diversity metrics, convergence detection | Domain semantics, product workflows |
| `coralys-recommendation` | Recommendation generation from evaluated alternatives | Recommendation strategies, ranking, filtering | Domain semantics, product workflows |
| `coralys-matching` | Matching and assignment utilities | Matching algorithms, assignment heuristics | Domain semantics, product workflows |
| `coralys-infrastructure` | Shared infrastructure (logging, telemetry, configuration) | Logging, metrics, configuration primitives | Domain semantics, product workflows, optimization algorithms |
| Planning capability crate (future) | Resource allocation over a Planning Horizon | `Resource`, `PlanningUnit`, `UtilizationPlan`, `PlanningHorizon` | Domain-specific resource types (nurses, crew), product workflows |

---

## Dependency Rules

These rules are enforced by Architectural Invariants AI-1 through AI-7 in [`ARCHITECTURE_EVOLUTION.md`](ARCHITECTURE_EVOLUTION.md).

**Rule 1 — Platform crates never import product crates.**
No `use` statement in any `coralys-*` crate may reference a type from `adapters/ultracrew`, `services/ultracrew_server`, `coralys-scheduling`, or any other domain library or product crate.

**Rule 2 — All platform crates may depend on `coralys-core`.**
`coralys-core` defines the execution contract. All other platform crates may depend on it. `coralys-core` itself has no platform dependencies.

**Rule 3 — Platform crates do not depend on one another except through `coralys-core`.**
`coralys-moga` must not import `coralys-simulation`. `coralys-decision` must not import `coralys-moga`. Each platform crate is independently composable. Products compose them; platform crates do not compose each other.

**Rule 4 — Breaking interface changes require a recorded decision.**
Breaking changes to any interface in this table require an explicit architectural decision recorded in [`ARCHITECTURE_EVOLUTION.md`](ARCHITECTURE_EVOLUTION.md). Domain libraries must not be required to rewrite large portions of code due to arbitrary platform API churn.

**Rule 5 — Application crates must not depend directly on platform crates.**
The dependency hierarchy is strictly one-directional and must pass through the solution adapter:

```
Application crate
        ↓
Solution adapter (domain library)
        ↓
coralys-* (platform crates)
```

For UltraCrew specifically:
```
ultracrew_server
        ↓
ultracrew  (adapters/ultracrew)
        ↓
coralys-*
```

The following dependency is **prohibited**:
```
ultracrew_server  →  coralys-*   ← VIOLATION
```

This rule applies to all Coralys-based applications. For any future application (e.g. `chronosentiment_server`, `ultraroute_server`, a CLI scheduler, or a batch processor), the same constraint holds: the application crate must interact with the Coralys platform exclusively through its solution adapter, never by importing platform crates directly.

**Rationale:** The solution adapter encapsulates all domain knowledge — workforce models, optimization configuration, constraint evaluation, simulation orchestration, recommendation generation, and domain-specific Coralys integration. The application layer is responsible only for HTTP endpoints, request validation, authentication, DTO transformation, persistence, configuration, and orchestration. This separation ensures that application concerns remain independent of optimization internals, business logic is reusable across multiple applications, Coralys remains a domain-agnostic platform, and architectural boundaries cannot be bypassed through convenience dependencies.

**Consequence:** If an application requires new optimization or decision capabilities, they shall first be exposed through the solution adapter rather than imported directly from a `coralys-*` crate. Direct application-to-platform dependencies constitute an architectural violation.

**Rule 6 — Solution adapters own all domain knowledge and must not expose platform implementation details to the application layer.**
A solution adapter is the exclusive integration layer between an application and the Coralys platform. A solution adapter:

- may depend on one or more `coralys-*` platform crates;
- owns all domain models, domain-specific algorithms, configuration, policies, and orchestration logic;
- translates between platform abstractions and application concepts;
- presents a stable, domain-meaningful public API to the application layer.

A solution adapter must **not** expose platform implementation details (e.g. `EvolutionEngine`, `GaResult`, `FitnessVector`, `Genome` trait bounds) in its public API surface. Applications interact only with the solution adapter's public API and remain independent of Coralys implementation details.

Rules 5 and 6 are symmetric and together define the complete boundary:
- Rule 5: applications cannot bypass the adapter (dependency direction).
- Rule 6: the adapter must hide the platform (encapsulation direction).

---

## What This Table Is Not

This table describes the **boundary** of each crate — what it is allowed to know and what it must remain ignorant of. It does not describe internal implementation.

A crate that knows too much becomes a bottleneck. A crate that knows too little cannot do its job. The boundaries in this table are the result of the architectural analysis in [`ARCHITECTURE_EVOLUTION.md`](ARCHITECTURE_EVOLUTION.md) and should be updated only through the decision process defined there.

---

*Update this table when a new platform crate is added or when a crate's responsibility changes. Changes require a recorded decision in [`ARCHITECTURE_EVOLUTION.md`](ARCHITECTURE_EVOLUTION.md).*