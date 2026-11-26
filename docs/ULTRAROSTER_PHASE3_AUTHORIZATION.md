# UltraCrew Phase 3 — Authorization

**Status:** Authorized  
**Authorized by:** User (session 2026-09-02)  
**Prerequisite:** `docs/ULTRAROSTER_PHASE3_MOGA_TRACE.md` (read-only trace, no code changes)

---

## Authorization Statement

> Implement genuine Pareto candidate generation in the UltraCrew adapter using the existing generic `coralys-moga::engine_proof` machinery. Preserve the existing UltraCrew scheduling representation, constraints, objectives, and evaluation semantics. Derive the Pareto `FitnessVector` from the underlying existing objective components, never by decomposing the scalar weighted fitness. Adapt the existing mutation operator without changing its semantics. Add the minimum backend API contract required to expose the resulting genuine Pareto candidates. Do not modify the generic Coralys MOGA Pareto machinery, do not add new objectives, do not change objective weights, do not introduce frontend scheduling/optimization logic, and do not change P3.3-CR or P4 learning. Initially keep the existing scalar production path intact and introduce the Pareto path separately for validation.

---

## Architecture Boundary (Enforced)

```
                 UltraCrew Request
                        │
                        ▼
                UltraCrew Adapter
                        │
             ┌──────────┴──────────┐
             │                     │
             ▼                     ▼
     Domain evaluation       Domain mutation
             │                     │
             └──────────┬──────────┘
                        ▼
               Coralys MOGA
          engine_proof::EvolutionEngine
                        │
                        ▼
                  ParetoArchive
                        │
              ┌─────────┼─────────┐
              ▼         ▼         ▼
           Soln A    Soln B    Soln C
              │         │         │
              └─────────┼─────────┘
                        ▼
                 Backend response
                        │
                        ▼
                       UI
                        │
                        ▼
                Scheduler chooses
```

Coralys MOGA remains domain-agnostic. All UltraCrew domain logic stays in the adapter.

---

## Invariants

### I1 — Genuine candidates only
Every `AlternativeSchedule` in the API response must come from `ParetoArchive`. No copies, mutations, or fabrications at the API layer.

```
AlternativeSchedule
    ↓ came from ParetoArchive
    ↓ contains complete genome
    ↓ evaluated by UltraCrew evaluator
    ↓ contains genuine objective values
```

### I2 — No scalar decomposition
The `FitnessVector` for Pareto evaluation must be derived from the **underlying objective components directly** (as `UltraCrewEvaluator` in `schedule_optimizer.rs` already does for INRC). It must **not** be derived by reverse-engineering the scalar weighted fitness:

```
F = w₁f₁ + w₂f₂ + ... + wₙfₙ   ← cannot recover f₁...fₙ from F
```

### I3 — Existing objectives only
No new objectives. No new penalties. No changed weights. No changed constraint semantics. The existing production objectives are the source.

### I4 — Coralys MOGA unchanged
`coralys-moga/src/engine_proof.rs`, `ParetoArchive`, `ParetoSolution` — no modifications.

### I5 — Existing scalar path preserved
`run_optimization()` → scalar GA → `global_best` remains intact and unchanged. The Pareto path is introduced as a separate, explicitly selected path (`run_pareto_pipeline()`). The two paths coexist for validation.

### I6 — Frontend receives, does not compute
The frontend must not acquire any new optimization or scheduling logic. The hard stop in `PlannerWorkflow.tsx` lifts automatically when the backend sends alternatives. No new frontend logic is required.

---

## Implementation Scope

### Step 1 — `Hash` for production `ScheduleGenome`
`adapters/ultracrew/src/optimization.rs`

Production `ScheduleGenome` is `HashMap<u64, u64>` which is not `Hash`. Options:
- Deterministic canonical serialization wrapper
- Sorted key-value pairs hashed via `DefaultHasher`

Must not change the meaning of assignments.

### Step 2 — Production Pareto evaluator
`adapters/ultracrew/src/optimization.rs` or new file

Implement `engine_proof::Evaluator<ProductionScheduleGenome>` returning a `FitnessVector` with the existing objective components. The components must be read from the constraint engine output directly — not from the scalar fitness.

The existing `ScheduleOptimizer::evaluate()` already calls `InrcConstraintEvaluator::evaluate()` which returns a `ConstraintReport`. The Pareto evaluator reads the same report and decomposes it into a `Vec<f64>` of objective components.

### Step 3 — Mutation adapter
`adapters/ultracrew/src/optimization.rs` or new file

Implement `engine_proof::MutationPolicy<ProductionScheduleGenome>` as a thin adapter over the existing `MutationOperator`. Signature change only: `&G → G` instead of `&mut G`. Mutation semantics unchanged.

### Step 4 — `run_pareto_pipeline()`
`adapters/ultracrew/src/pipeline.rs`

Mirror `run_inrc_startup_pipeline()` for the production genome:
- Instantiate production Pareto evaluator + mutation adapter
- Instantiate `engine_proof::EvolutionEngine`
- Seed with a baseline genome
- Run N steps
- Return `Vec<ProductionParetoSolution>` from `engine.archive.solutions`

### Step 5 — `ProductionParetoSolution` type
`adapters/ultracrew/src/public_contracts.rs`

New serializable type:
```rust
pub struct ProductionParetoSolution {
    pub schedule: HashMap<u64, u64>,
    pub objectives: Vec<f64>,          // raw FitnessVector
    pub metrics: HashMap<String, f64>, // analyzed metrics (analyze_solution)
}
```

### Step 6 — API contract
`services/ultracrew_server/src/main.rs`

Add to `ScheduleResponse`:
```rust
pub alternatives: Vec<ProductionParetoSolution>,
```

Populate from `run_pareto_pipeline()` result. The existing `schedule` field continues to hold `global_best` from the scalar path.

---

## Out of Scope

- Modifying `coralys-moga` in any way
- Adding new objectives or changing objective weights
- Changing `ScheduleOptimizer` scalar evaluation
- Changing `run_optimization()` or the existing GA path
- Adding coverage metrics (separate unresolved backend issue)
- Changing P3.3-CR redistribution
- Changing P4 pattern accumulation or decision recording
- Any frontend optimization or scheduling logic

---

## Validation Gate

Before the Pareto path is considered production-ready:

1. `run_pareto_pipeline()` returns ≥ 2 distinct `ProductionParetoSolution` entries for a standard test scenario
2. Each solution's `schedule` is a valid `HashMap<u64, u64>` (complete assignments)
3. Each solution's `objectives` has the same length as the `FitnessVector` produced by the Pareto evaluator
4. No solution in the archive is dominated by another on all objectives (Pareto correctness)
5. The existing scalar path (`run_optimization()`) produces identical results before and after the Pareto path is added
6. All existing tests pass (47/47 minimum)
7. `tsc --noEmit` exits 0 (no TypeScript changes expected, but must verify)