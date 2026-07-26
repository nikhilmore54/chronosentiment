# Codebase Assessment

> **Date**: 2026-07-19
> **Version**: 1.0 — frozen (Architecture Baseline v1.0)
> **Purpose**: Record what the codebase actually contains. No proposals. No roadmap.  
> **Standard**: Every statement in this document is directly supported by observable code.  
> **Relationship**: This document feeds [`ARCHITECTURE_EVOLUTION.md`](ARCHITECTURE_EVOLUTION.md), which contains proposals and decisions.

---

## 1. Workspace Inventory

The workspace is defined in [`Cargo.toml`](../Cargo.toml) with the following members:

### Platform crates

| Crate | Maturity | Evidence |
|---|---|---|
| `coralys-core` | Mature | Generic traits: `Scenario`, `Solution`, `Outcome`, `State`, `Action`, `DecisionPlugin`, `ReasoningEngine`. No domain coupling. |
| `coralys-moga` | Mature | Full MOGA engine: `engine.rs`, `ecology/`, `traits/`, `benchmark.rs`, `observatory.rs`. Falsification tests present. |
| `coralys-ecology` | Stub | 6 files: `lib.rs`, `traits.rs`, `models.rs`, `state.rs`, `progress.rs`, `diagnostics.rs`. No implementation beyond trait definitions. |
| `coralys-eval` | Stub | 5 files: `lib.rs`, `adapter.rs`, `pipeline.rs`, `registry.rs`, `types.rs`. |
| `coralys-decision` | Stub | 2 files: `lib.rs`, `traits.rs`. |
| `coralys-simulation` | Stub | 2 files: `lib.rs`, `traits.rs`. |
| `coralys-recommendation` | Stub | 3 files: `lib.rs`, `recommender.rs`, `traits.rs`. |
| `coralys-matching` | Unknown | Not fully read. |
| `coralys-infrastructure` | Stub | 1 file: `lib.rs`. |
| `coralys-v2` | Stub | 2 files: `lib.rs`. |

### Infrastructure crates (not platform)

| Crate | Notes |
|---|---|
| `infrastructure/core` | Shared infrastructure primitives |
| `infrastructure/optimization` | Optimization infrastructure |
| `infrastructure/observatory/api` | Observatory API |

### Domain implementation crates

| Crate | Domain | Maturity | Evidence |
|---|---|---|---|
| `adapters/ultracrew` | Nurse rostering (INRC2) | Mature | See §3 |
| `coralys-scheduling` | Airline crew scheduling | Partial | See §4 |
| `adapters/cvrp` | Vehicle routing | Present | `src/analysis.rs`, `src/moga_impl.rs`, qualification module |
| `adapters/cvd001` | CVD001 | Present | `src/credit.rs`, `src/evaluator.rs`, `src/workload.rs` |
| `adapters/roadef` | ROADEF challenge | Present | In workspace manifest |
| `adapters/chronosentiment` | Financial sentiment | Present | `src/lib.rs` |

### Application / service crates

| Crate | Notes |
|---|---|
| `services/ultracrew_server` | HTTP server wrapping the INRC2 solver |
| `services/cvrp_server` | CVRP research server |
| `apps/cvrp-playground` | CVRP playground |

### Financial crates

| Crate | Notes |
|---|---|
| `financial/core` | Financial domain core |
| `financial/ese` | ESE module |
| `financial/strategies` | Strategy implementations |

### What does not exist

The following were assumed in earlier discussions but are absent from the workspace manifest:

- `services/chrono_server`
- `adapters/inrc` (INRC logic lives inside `adapters/ultracrew/src/inrc/`)
- `adapters/airline`
- `coralys-policy` (directory exists, not a workspace member)
- `coralys-workforce` (does not exist)

---

## 2. Dependency Observations

From reading source files directly:

- [`services/ultracrew_server/src/optimizer.rs`](../services/ultracrew_server/src/optimizer.rs) line 5: `use ultracrew::inrc::models::InrcScenario` — the server imports an INRC2 type directly.
- [`services/ultracrew_server/src/optimizer.rs`](../services/ultracrew_server/src/optimizer.rs) line 56: `UltraCrewEvaluator` holds `pub scenario: InrcScenario`.
- `coralys-scheduling` has no observed dependency on `adapters/ultracrew` or any INRC type.
- `adapters/ultracrew` has no observed dependency on `coralys-scheduling`.

---

## 3. `adapters/ultracrew` — Detailed Inventory

### Module structure

```
src/
  inrc/
    models.rs       ← InrcScenario, Nurse, Shift, Contract, Coverage
    parser.rs       ← JSON parser for INRC2 instance files
    evaluator.rs    ← INRC2 constraint evaluator (H1–H4, S1–S7)
    exporter.rs     ← solution export
    history.rs      ← multi-week history tracking
    bipartite_matching.rs
    audit.rs
    optimization.rs
    mod.rs
  workforce/
    mod.rs
    ecology_adapter.rs
    workforce_metrics.rs
  constraint_engine.rs
  decision_intelligence.rs
  ecology.rs
  helpers.rs
  lib.rs
  models.rs
  optimization.rs
  pipeline.rs
  public_contracts.rs
  recommendation.rs
  schedule_solution.rs
  config/
    mod.rs
    optimization_profiles.rs
```

### Test data present

INRC2 benchmark instances for: n030w4, n030w8, n040w4, n050w4, n060w8, n080w8, n100w4, n120w8.

### Benchmark artifacts present

`inrc_m22_benchmark.csv`, `ablation_matrix_30seed.csv`, `ecology_response_curve_30seed.csv`, `m23a_survival_results.csv`, `m23a3_recovery_results.csv`, and others.

### Observations

- `src/inrc/` is the mature, primary implementation. It is INRC2-specific.
- `src/workforce/` contains 3 files and is not functional as a generic layer.
- The `src/workforce/` module exists as a stub, indicating intent to generalize, but the generalization has not been implemented.
- The crate name `adapters/ultracrew` suggests it is an adapter, but it contains the primary domain logic, not a thin adapter over a separate library.

---

## 4. `coralys-scheduling` — Detailed Inventory

### Module structure

```
src/
  lib.rs
  domain/
    crew.rs
    duty.rs
    flight.rs
    pairing.rs
    roster.rs
    rotation.rs
    mod.rs
  legality/
    base_return.rs
    coverage.rs
    duty_connectivity.rs
    duty_time.rs
    fdp.rs
    minimum_rest.rs
    qualification.rs
    mod.rs
  optimization/
    cost.rs
    metrics.rs
    objective.rs
    neighborhood/
      relocate.rs
      swap.rs
    search/
      greedy.rs
      local_search.rs
    mod.rs
  planner/
    incremental.rs
    summary.rs
    whatif.rs
    mod.rs
  resilience/
    disruption.rs
    reserve.rs
    robustness.rs
    mod.rs
tests/
  benchmark.rs
  robustness.rs
  scalability.rs
  scenario_validation.rs
  solution_quality.rs
```

### Public API (from `src/lib.rs`)

The crate re-exports at root level:

```
AircraftType, AirportCode, CrewId, CrewMember, CrewRole,
Duty, DutyError, DutyId,
FlightLeg, FlightLegId, FlightNumber,
Pairing, PairingError, PairingId,
PlanningPeriod, Qualification,
Roster, RosterError, RosterId,
Rotation, RotationError, RotationId,
EntityRef, LegalityChecker, LegalityRule, LegalityViolation, ViolationSeverity
```

### Documentation (from `src/lib.rs` line 3)

> "Airline crew scheduling domain model for the Coralys platform."

### Observations

- Every public type is airline-specific. There are no generic scheduling types in the public API.
- The legality module contains `fdp.rs` (Flight Duty Period) — an aviation regulatory concept with no analogue in generic workforce scheduling.
- The crate name `coralys-scheduling` does not match its contents.
- The crate's own documentation explicitly identifies it as an airline domain model.
- The crate has no observed dependency on `adapters/ultracrew` or any INRC type.
- The optimization and resilience modules are implemented (not stubs), indicating this is a working domain model, not a skeleton.

---

## 5. `services/ultracrew_server` — Detailed Inventory

### Module structure

```
src/
  lib.rs
  main.rs
  builder.rs
  inrc_observer.rs    ← INRC-specific observer
  models.rs
  optimizer.rs
  persistence.rs
  simulation.rs
  simulation_test.rs
  tracker.rs
  validator.rs
  bin/
    acceptance_test.rs
    benchmark.rs
    cs_governance_validation.rs
    ecology_validation.rs
    inrc_archive_forensics.rs
    m8g_cs_validation.rs
    m8g_ultracrew_validation.rs
    m9a_search_observatory.rs
    policy_seed_runner.rs
    validation_pass.rs
```

### Key observations from source

From [`src/optimizer.rs`](../services/ultracrew_server/src/optimizer.rs):
- Line 5: `use ultracrew::inrc::models::InrcScenario`
- `UltraCrewEvaluator` struct holds `pub scenario: InrcScenario`
- Fitness function references INRC constraint names: `"min_assignments"`, `"max_working_weekends"`, `"complete_weekends"`, `"min_consecutive_days_off"`, `"max_consecutive_working_days"`, `"min_consecutive_working_days"`
- `UltraCrewMutator` iterates `self.scenario.nurses`

From [`src/models.rs`](../services/ultracrew_server/src/models.rs):
- `ScheduleVersion.schedule` field comment: `// nurse_id -> daily shifts`
- Type names (`DecisionCase`, `Recommendation`, `ScheduleVersion`, `DecisionLog`) are generic-looking but backed by INRC data structures

---

## 6. Naming Inconsistencies

| Name | What it says | What it actually contains |
|---|---|---|
| `coralys-scheduling` | Generic scheduling platform | Airline crew scheduling domain implementation |
| `adapters/ultracrew` | An adapter | Primary UltraCrew product library (INRC2 solver) |
| `src/workforce/` in ultracrew | Generic workforce layer | 3-file stub, not functional |
| `services/ultracrew_server` | UltraCrew server | INRC2 application server |

---

## 7. Evidence vs Decision Table

This table separates observable facts from the architectural decisions they inform. Decisions are recorded in [`ARCHITECTURE_EVOLUTION.md`](ARCHITECTURE_EVOLUTION.md).

| Observable evidence | Architectural decision (see evolution doc) |
|---|---|
| `coralys-scheduling` exports only airline types | Rename decision deferred pending intent clarification |
| `coralys-scheduling` doc says "Airline crew scheduling domain model" | Confirms current content; does not confirm long-term intent |
| `adapters/ultracrew/src/inrc/` is mature | Preserve INRC implementation; do not discard |
| `adapters/ultracrew/src/workforce/` is a 3-file stub | Generic workforce abstraction is planned but not implemented |
| `ultracrew_server` imports `InrcScenario` directly | Server is coupled to INRC2; decoupling is future work |
| `coralys-workforce` does not exist | Generic workforce platform crate is a candidate, not a decision |
| INRC2 and airline domains both exist as implementations | Two concrete implementations available for abstraction comparison |
| `coralys-moga` and `coralys-core` are genuinely generic | These are the stable platform foundation |

---

*This document records observations only. It does not change. Proposals and decisions are in [`ARCHITECTURE_EVOLUTION.md`](ARCHITECTURE_EVOLUTION.md).*