# UltraCrew Phase 3 — Coralys MOGA Architecture Trace

**Status:** Read-only investigation. No code changes.  
**Scope:** `coralys-moga/src/engine_proof.rs`, `coralys-moga/src/engine.rs`, `adapters/ultracrew/src/inrc/schedule_optimizer.rs`, `adapters/ultracrew/src/optimization.rs`, `adapters/ultracrew/src/pipeline.rs`, `adapters/ultracrew/src/public_contracts.rs`

---

## 1. What generic interfaces does `engine_proof::EvolutionEngine` require?

`engine_proof::EvolutionEngine<G, F, M>` requires three type parameters:

```rust
// engine_proof.rs lines 87–92
pub struct EvolutionEngine<G: Genome, F: Evaluator<G>, M: MutationPolicy<G>> {
    pub evaluator: F,
    pub mutator: M,
    pub archive: ParetoArchive<G>,
    pub rng: StdRng,
}
```

The traits are defined locally in `engine_proof.rs`:

```rust
pub trait Genome: Clone + Send + Sync + Hash {}

pub trait Evaluator<G: Genome> {
    fn evaluate(&self, genome: &G) -> FitnessVector;  // FitnessVector = Vec<f64>
}

pub trait MutationPolicy<G: Genome> {
    fn mutate(&self, genome: &G) -> G;
}
```

**Key observation:** `engine_proof` defines its own `Genome`, `Evaluator`, and `MutationPolicy` traits — they are **not** the same as the production `coralys_moga::traits::{Genome, FitnessEvaluator, MutationOperator}`. These are two separate, parallel trait hierarchies.

---

## 2. What is different from the production `EvolutionEngine`?

The production engine (`engine.rs`) uses:

```rust
pub struct EvolutionEngine<G: Genome, F: FitnessEvaluator<G>, M: MutationOperator<G>,
                           C: CrossoverOperator<G>, Factory: GenomeFactory<G>>
```

Trait differences:

| Aspect | `engine_proof` | Production `engine` |
|---|---|---|
| Genome trait | `engine_proof::Genome` (requires `Hash`) | `coralys_moga::traits::Genome` (no `Hash`) |
| Evaluator | `Evaluator<G>` → `FitnessVector` (Vec<f64>) | `FitnessEvaluator<G>` → `type Evaluation: Evaluated` |
| Mutation | `MutationPolicy<G>` → returns `G` (immutable) | `MutationOperator<G>` → mutates `&mut G` in place |
| Crossover | Not present | `CrossoverOperator<G>` required |
| Factory | Not present | `GenomeFactory<G>` required |
| Archive | `ParetoArchive<G>` built-in | None — only `global_best` tracked |
| Selection | Random parent from archive | Tournament selection |
| Evolution loop | `seed()` + `step()` (caller controls loop) | `run_ga_evolution()` (engine controls loop) |
| Result type | `ParetoArchive<G>` (multiple solutions) | `GaResult { global_best, top_10, ... }` (one best) |
| Fitness representation | `Vec<f64>` (multi-objective vector) | Scalar `f64` via `Evaluated::fitness()` |

**The two engines are architecturally distinct.** The production engine is a standard single-objective GA with tournament selection and crossover. The proof engine is a simple (1+1)-style evolutionary strategy that maintains a Pareto archive.

---

## 3. Is `ParetoArchive<G>` genuinely generic?

Yes. `ParetoArchive<G: Genome>` is fully generic over any type implementing `engine_proof::Genome`:

```rust
// engine_proof.rs lines 27–84
pub struct ParetoArchive<G: Genome> {
    pub solutions: Vec<ParetoSolution<G>>,
}

pub struct ParetoSolution<G: Genome> {
    pub genome: G,
    pub fitness: FitnessVector,  // Vec<f64>
    pub uid: u64,
    pub parent_uid: u64,
}
```

The dominance logic in `ParetoArchive::add()` operates purely on `FitnessVector` (Vec<f64>) — it has no knowledge of UltraCrew domain semantics. It is a correct, generic Pareto dominance filter.

**Constraint:** `G` must implement `Hash` (used for `uid` generation via `DefaultHasher`). The production `ScheduleGenome` (`HashMap<u64, u64>`) does **not** currently implement `Hash` because `HashMap` is not `Hash` in Rust.

---

## 4. Can the existing UltraCrew evaluator/objective vector plug into `engine_proof` unchanged?

**Two separate evaluators exist:**

### A. INRC evaluator (`schedule_optimizer.rs`) — already plugged in

`UltraCrewEvaluator` implements `engine_proof::Evaluator<ScheduleGenome>` (the INRC `ScheduleGenome`, not the production one). It returns a 6-objective `FitnessVector`:

```
[s6_assignment_penalty, s7_weekend_penalty, recovery_penalty,
 workload_balance, temporal_load_balance, total_hard_penalty]
```

This already works with `engine_proof`. It is used in `run_inrc_startup_pipeline()`.

### B. Production evaluator (`optimization.rs`) — NOT plugged in

`ScheduleOptimizer` implements `coralys_moga::traits::FitnessEvaluator<ScheduleGenome>` (the production `ScheduleGenome` = `HashMap<u64, u64>`). It returns a `ScheduleEvaluation` with a scalar `fitness: f64`.

To plug the production evaluator into `engine_proof`, three changes would be required:

1. **`Hash` for production `ScheduleGenome`:** `HashMap<u64, u64>` is not `Hash`. A wrapper or custom hash implementation would be needed.
2. **New `Evaluator<G>` impl:** A new impl of `engine_proof::Evaluator<ProductionScheduleGenome>` that returns a `FitnessVector` (multi-objective) rather than a scalar. This requires deciding which objectives to expose — the existing scalar fitness is a weighted sum, not a decomposed vector.
3. **New `MutationPolicy<G>` impl:** `engine_proof::MutationPolicy` takes `&G` and returns `G` (immutable). The production `MutationOperator` takes `&mut G`. A thin adapter would be needed.

**The production evaluator cannot plug in unchanged.** The INRC evaluator already can.

---

## 5. How are Pareto solutions represented and recovered?

Each `ParetoSolution<G>` retains:
- `genome: G` — the full schedule genome (all assignments)
- `fitness: FitnessVector` — the multi-objective score vector
- `uid: u64` — hash of the genome
- `parent_uid: u64` — lineage tracking

Recovery from the archive is direct:

```rust
// pipeline.rs lines 133–144
let pareto_solutions: Vec<InrcParetoSolution> = engine
    .archive
    .solutions
    .iter()
    .map(|sol| InrcParetoSolution {
        s6_assignment_penalty: sol.fitness.get(0).copied().unwrap_or(0.0),
        ...
        schedule: sol.genome.to_flat_schedule(),
    })
    .collect();
```

The genome is preserved in full inside each `ParetoSolution`. Recovery is O(n) over archive size with no reconstruction needed.

---

## 6. How is `global_best` currently selected in the production path?

In `engine.rs` `run_ga_evolution()` (lines 397–434):

```rust
let gen_best = evals[0].clone();  // highest scalar fitness after sort
if global_best.is_none() || gen_best.fitness() > global_best.as_ref().unwrap().fitness() {
    global_best = Some(gen_best.clone());
}
```

Selection is purely by scalar `fitness()`. The `GaResult` also exposes `top_10` (top 10 by scalar fitness from the final population), but these are not Pareto-optimal candidates — they are the 10 highest-fitness individuals from the last generation, which may be near-identical due to convergence.

`top_10` is **not** a substitute for a Pareto archive. It does not guarantee diversity across objectives.

---

## 7. Can the production pipeline obtain multiple genuine candidates without moving domain logic into Coralys?

**Yes — and this is the correct integration boundary.**

The architecture already demonstrates the pattern in `run_inrc_startup_pipeline()`:

```
UltraCrew adapter (pipeline.rs)
    ↓ creates UltraCrewEvaluator + UltraCrewMutator (domain logic stays here)
    ↓ instantiates engine_proof::EvolutionEngine (generic Coralys machinery)
    ↓ calls engine.seed() + engine.step() (no domain logic in Coralys)
    ↓ reads engine.archive.solutions (Vec<ParetoSolution<G>>)
    ↓ maps to InrcParetoSolution (domain-specific serialization stays in adapter)
```

No UltraCrew domain logic enters `coralys-moga`. The evaluator, mutator, and serialization all live in the UltraCrew adapter. Coralys provides only the generic Pareto machinery.

The same pattern can be applied to the production path. The required work is entirely in the UltraCrew adapter, not in Coralys.

---

## 8. What API changes would be needed afterward?

The current `ScheduleResponse` (main.rs) has no `alternatives` field. To expose genuine Pareto candidates, a new field would be added:

```rust
// Proposed addition to ScheduleResponse
struct ScheduleResponse {
    schedule: HashMap<u64, u64>,           // existing: global_best
    metrics: HashMap<String, f64>,          // existing
    constraint_report: Option<...>,         // existing
    recommendations: Vec<...>,              // existing
    telemetry: Option<...>,                 // existing
    alternatives: Vec<AlternativeSchedule>, // NEW
}

struct AlternativeSchedule {
    schedule: HashMap<u64, u64>,
    objectives: Vec<f64>,            // raw FitnessVector (6 values)
    metrics: HashMap<String, f64>,   // analyzed metrics for this candidate
}
```

On the TypeScript side, `WorkflowTypes.ts` `ScheduleResult` already declares `alternatives?: RosterAlternative[]` — the field exists but is never populated because the backend never sends it. The shape would need to be reconciled with whatever `AlternativeSchedule` serializes to.

---

## Summary: Minimal Change Required

The minimal production-pipeline integration path is:

**Step 1 (Coralys MOGA — no change needed):** `engine_proof::EvolutionEngine`, `ParetoArchive<G>`, and `ParetoSolution<G>` are already generic and correct. No changes to `coralys-moga`.

**Step 2 (UltraCrew adapter — new production Pareto pipeline):**
- Add `Hash` to production `ScheduleGenome` (or use a wrapper)
- Implement `engine_proof::Evaluator<ProductionScheduleGenome>` that returns a `FitnessVector` decomposing the existing objectives (not inventing new ones)
- Implement `engine_proof::MutationPolicy<ProductionScheduleGenome>` as a thin adapter over the existing mutator
- Add `run_pareto_pipeline()` in `pipeline.rs` mirroring `run_inrc_startup_pipeline()` but using the production genome/evaluator
- Map `ParetoArchive` solutions to a new `ProductionParetoSolution` type in `public_contracts.rs`

**Step 3 (UltraCrew backend — API contract):**
- Add `alternatives: Vec<AlternativeSchedule>` to `ScheduleResponse`
- Populate from `run_pareto_pipeline()` result

**Step 4 (Frontend — no new logic):**
- `ScheduleResult.alternatives` is already declared; Step 4 UI already exists
- The hard stop in `PlannerWorkflow.tsx` will lift automatically when the backend sends alternatives

**Critical constraint:** The objective decomposition in Step 2 must use **existing optimization semantics**. The production `ScheduleOptimizer` currently produces a scalar fitness via a weighted sum. To produce a genuine `FitnessVector`, the weights must be separated back into individual objective components — which requires reading the constraint engine output directly, as `UltraCrewEvaluator` (INRC) already does. This is the most significant design decision in the integration.

---

## Compatibility Matrix

| Question | Answer |
|---|---|
| `engine_proof::EvolutionEngine` generic interfaces | `Genome` (+ `Hash`), `Evaluator<G>` (→ `Vec<f64>`), `MutationPolicy<G>` (immutable) |
| Difference from production engine | Separate trait hierarchy; no crossover; no factory; archive vs scalar best; (1+1)-ES vs GA |
| `ParetoArchive<G>` genuinely generic | Yes — operates on `Vec<f64>` only, no domain knowledge |
| Production evaluator plugs in unchanged | No — needs `Hash`, new `Evaluator` impl, new `MutationPolicy` adapter |
| INRC evaluator plugs in unchanged | Yes — already does, proven in `run_inrc_startup_pipeline()` |
| Pareto solutions retain full genome | Yes — `ParetoSolution.genome` is the complete schedule |
| `global_best` selection | Scalar fitness max across all generations |
| Domain logic stays in adapter | Yes — the INRC path already proves this pattern |
| API changes needed | New `alternatives` field in `ScheduleResponse`; `AlternativeSchedule` type |
| Frontend changes needed | None — `ScheduleResult.alternatives` already declared; hard stop lifts automatically |