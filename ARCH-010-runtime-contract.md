# ARCH-010: Coralys Runtime Contract

**Status:** ACTIVE (Frozen as of v1.0)
**Owner:** Core Architecture Team
**Scope:** The Operational Decision Runtime

## 1. Architectural Principles

Coralys is a platform for evaluating, evolving, and explaining operational decisions. It is not an airline scheduling tool. 

The primary architectural mandate of Coralys is **Separation of Reality from Optimization**.
- The **Domain** defines Reality (Time, Events, Resources, Operational Models, Constraints, Objectives).
- The **Runtime** defines the standard vocabulary for describing Reality and interacting with Optimization.
- The **Optimization Engine** evolves state toward feasibility and optimality.

**Invariant:** The runtime must never depend on a concrete optimization algorithm. (e.g., GA, MILP, CP-SAT). It must only know the `OptimizationEngine` trait.

## 2. Runtime Contracts

Every future engine and adapter must adhere to these foundational marker traits:

- **OperationalModel:** The structural representation of a state in time.
- **DecisionVector:** The subset of variables within an `OperationalModel` that engines can mutate.
- **ConstraintModel:** The boundaries of physical, legal, and business feasibility.
- **ObjectiveModel:** The scoring mechanism that defines "goodness" over a feasible state.
- **OptimizationEngine:** The solver responsible for producing a higher-fitness `OperationalModel`.
- **ConstraintSatisfactionEngine:** The deterministic subsystem responsible for driving an `OperationalModel` toward feasibility.
- **PolicyModel:** The operational and regulatory rules defining the context of evaluation.
- **MetricModel:** Computes numeric indicators of state margin, buffer, and quality.

These contracts contain **no optimization logic** and **no domain logic**.

### Constraint Satisfaction Invariants

1. **Constraint Satisfaction must never optimize.** Its only responsibility is ensuring legality and generating minimal repairs. Objective improvement belongs exclusively to Objective Evaluation.
2. **Constraint Satisfaction must be deterministic for a given input model, repair policy, and repair engine.** Optimization is stochastic, but constraint satisfaction is not. This guarantees reproducibility, debugging, and clear decision lineage.

### Metric Evaluation Invariants

1. **MetricModels are pure, deterministic computations derived solely from the OperationalModel and PolicyModel. They must not mutate operational state, invoke repair, or embed optimization preferences.**

### Acyclic Information Flow Invariant

1. **Every subsystem consumes artifacts from the preceding subsystem and must not recompute upstream information.**
   - The runtime pipeline enforces a strict acyclic information flow: `Operational Model` → `Metric Engine` (produces `MetricReport`) → `Constraint Assessment` (produces `ConstraintReport`) → `Constraint Satisfaction` (produces `ConstraintSatisfactionResult`) → `Objective Evaluation` (produces `Fitness`).
   - Downstream consumers (e.g., Objective Evaluation, Constraint Assessment, Constraint Satisfaction) must rely on the provided artifacts (e.g., `MetricReport`) rather than recalculating upstream facts (e.g., computing rest margins manually).

## 3. Dependency Rules

The platform enforces a strict, acyclic dependency model pointing inward:

1. `Domain Adapters` depend on `Coralys Runtime` and `Coralys Optimization` interfaces.
2. `Optimization Engines` (e.g., GA, CP-SAT) depend on `Coralys Runtime` interfaces.
3. `Coralys Core` depends on **nothing**.

It is strictly forbidden for any module within `coralys-moga` or any future engine module to import or reference domain-specific types (e.g., `UltraCrew`, `Shift`, `ScheduleGenome`).

## 4. Ownership Rules

- **Coralys owns the vocabulary (Traits).**
- **Domains own the representations (Structs).**

If a domain requires a change to the way a decision is represented, it changes its own structs. It **does not** change the Runtime Traits. 

## 5. Extension Rules

When building a new operational domain (e.g., factory scheduling, logistics routing) or a new optimization solver (e.g., MILP, Reinforcement Learning):
- Implement the Runtime traits for your representations.
- Connect your domain representation to the Coralys Ecology via the `OptimizationEngine` boundary.
- Do not bypass the `OperationalModel` abstraction to directly manipulate domain structures from within an engine.

## 6. Stability Guarantees

The traits exported in `coralys_moga::runtime` and `coralys_moga::optimization` constitute the permanent ABI of the Coralys ecosystem. They are structured in three stability tiers:

### Stable
These are effectively part of Coralys' public platform. Changing these requires a major version bump and architectural review.
- `OperationalModel`
- `ConstraintModel`
- `ConstraintAssessment`
- `ObjectiveModel`
- `OptimizationEngine`
- `DecisionVector`
- `ConstraintSatisfactionEngine`
- `RepairOperator`
- `PolicyModel`
- `MetricEngine`
- `MetricReport`
- `PipelineObserver`

### Experimental
Allowed to evolve without strict versioning.
- **Reference Operational Model (OEN)**
- Runtime explanation APIs
- Lineage internals
- Graph implementations

### Compatibility
Temporary structures that can be removed once migration is complete.
- `ScheduleGenome` (Compatibility Operational Model)
- Existing UltraCrew adapter
- Existing ROADEF adapter
