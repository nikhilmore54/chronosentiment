# Phase 9 P9-B — Characterization: The "Improve" Path

**Status: CHARACTERIZATION COMPLETE**
**Date: 2026-08-24**
**Baseline: post-H3 (`bb9672750`)**

---

## 1. Motivation

After H3 promotion, the post-H3 Phase 8 attribution on setA-14 shows:

| Component | Time | % wall |
|-----------|------|--------|
| Improve (process_offspring) | 733,600 ms | 74.2% |
| Repair (process_offspring) | 45,842 ms | 4.6% |
| Feasibility pre-check | 0 ms | 0% (eliminated by H3) |

The "Improve" label accounts for 74.2% of wall-clock. P9-B must characterize
what this actually consists of before proposing any intervention.

---

## 2. Source Trace

### 2.1 What `t_improve_ms` measures

In [`adapters/roadef/src/pipeline_impl.rs`](adapters/roadef/src/pipeline_impl.rs:1515),
`t_improve_ms` accumulates the elapsed time of the entire
[`process_offspring()`](coralys-core/src/pipeline.rs:23) call **when it returns `Ok(true)`**:

```rust
let t_proc_start = Instant::now();
let proc_result = pipeline_obj.process_offspring(&mut child);
let t_proc_elapsed = t_proc_start.elapsed().as_secs_f64() * 1000.0;
let success = match proc_result {
    Ok(true) => {
        t_improve_ms += t_proc_elapsed;   // ← entire process_offspring() on success path
        ...
    }
    Ok(false) | Err(_) => {
        t_repair_ms += t_proc_elapsed;    // ← entire process_offspring() on failure path
        ...
    }
};
```

### 2.2 What `process_offspring()` does on the `Ok(true)` path

From [`coralys-core/src/pipeline.rs`](coralys-core/src/pipeline.rs:23):

```rust
pub fn process_offspring(&self, candidate: &mut G) -> Result<bool, E> {
    // Step 1: Repair Gate
    if !self.constraint_model.is_feasible(candidate) {   // ← evaluate_violations() call
        // ... repair operators ...
        if !repaired || !self.constraint_model.is_feasible(candidate) {
            return Ok(false);   // → goes to t_repair_ms
        }
    }

    // Step 2: Improvement Gate
    for op in &self.improvement_operators {
        op.improve(candidate, &self.constraint_model, &self.improve_budget)?;
    }

    Ok(true)   // → goes to t_improve_ms
}
```

When `Ok(true)` is returned, the path taken was:
1. [`is_feasible(candidate)`](coralys-core/src/pipeline.rs:25) returned `true` (offspring was feasible) — **one full `evaluate_violations()` call**
2. [`op.improve()`](coralys-core/src/pipeline.rs:40) was called for each improvement operator

### 2.3 What `RoadefImprovement::improve()` actually does

From [`adapters/roadef/src/operators.rs`](adapters/roadef/src/operators.rs:70):

```rust
fn improve(
    &self,
    _candidate: &mut RoadefGenome,
    _model: &RoadefConstraintModel,
    _budget: &OperatorBudget,
) -> Result<bool, Self::Error> {
    // TODO: Implement bottleneck-relief local search or LNS here.
    // For now, this is a no-op that just returns true (preserves feasibility).
    Ok(true)
}
```

**`RoadefImprovement::improve()` is a no-op.** It takes no parameters by value,
performs no computation, and returns immediately.

---

## 3. Conclusion: What the 733,600 ms Actually Is

The 733,600 ms / 74.2% labeled "Improve (process_offspring)" is **entirely the
cost of the `is_feasible()` call at line 25 of `pipeline.rs`** on the feasible-offspring
path. The `improve()` operator itself contributes zero time.

This is the same `evaluate_violations()` function that H3 eliminated on the
pre-check path. The difference is:

| Call site | H3 status | Semantic role |
|-----------|-----------|---------------|
| Standalone pre-check in `pipeline_impl.rs` | **Eliminated by H3** | Redundant — `process_offspring` would call it anyway |
| Internal gate in `process_offspring()` line 25 | **Still present** | Decides whether to enter repair path |

The internal gate is semantically necessary **in general** — it is the mechanism
by which infeasible offspring are routed to repair operators. However, it may be
**redundant in specific cases** where the caller already knows the feasibility
state of the offspring.

---

## 4. The Eight Characterization Questions

**Q1. What does `process_offspring()` do after the feasibility check?**
On the `Ok(true)` path: calls `op.improve()` for each improvement operator, then
returns. With the current no-op `RoadefImprovement`, this is zero additional work.

**Q2. What exactly constitutes the Improve path?**
The `Ok(true)` return path of `process_offspring()`. The dominant cost is the
`is_feasible()` gate at line 25, not the improvement operators.

**Q3. Which operations dominate inside Improve?**
`evaluate_violations()` called by `is_feasible()` at line 25. This is the same
4-stage constraint evaluation (segment limit, budget, routing via `expand_sr_path`,
capacity) as characterized in P9-A.

**Q4. How many times is each operation invoked per offspring?**
Once per offspring that reaches `process_offspring()`. On the `Ok(true)` path,
exactly one `evaluate_violations()` call.

**Q5. Are there repeated calculations or allocations?**
`evaluate_violations()` allocates a `Vec<RoadefViolation>` that is immediately
discarded (only `.is_empty()` is checked). The routing computation
(`expand_sr_path` → `backward_dijkstra`) is repeated in full for every call.

**Q6. Which state is recomputed versus reusable?**
`backward_dijkstra` results are genome-independent for a given (target, slot,
disabled_arcs) triple. They are not cached between calls. The genome-dependent
part is the waypoint sequence used to construct SR paths.

**Q7. What parts are semantically required for trajectory preservation?**
The `is_feasible()` gate at line 25 is required to correctly route infeasible
offspring to repair. However, if the caller can guarantee the offspring is
feasible (e.g., because crossover/mutation provably preserves feasibility, or
because a lightweight pre-check already confirmed it), the call is redundant.

**Q8. What candidate optimizations can be formulated without changing the evolutionary contract?**
See Section 5.

---

## 5. Candidate Hypotheses for P9-B

### H4 — Pass feasibility state into `process_offspring()` (avoid redundant re-check)

**Observation**: After H3, the caller in `pipeline_impl.rs` no longer knows
whether the offspring is feasible before calling `process_offspring()`. The
internal gate at line 25 must therefore always call `evaluate_violations()`.

However, the caller *could* perform a lightweight feasibility check and pass the
result in, allowing `process_offspring()` to skip the gate when the offspring is
already known feasible. This would require a signature change to
`process_offspring()` or a new entry point.

**Risk**: Requires changing the `EvolutionaryPipeline` API in `coralys-core`.
Trajectory preservation must be verified carefully.

### H5 — Cache `evaluate_violations()` results keyed on genome hash

**Observation**: The evaluation cache already exists for `RoadefEvaluation`
(fitness). A similar cache for feasibility results could avoid re-running
`evaluate_violations()` on genomes seen in the same generation.

**Risk**: Cache invalidation correctness. Genome equality semantics must be
exact. Memory overhead.

### H6 — Lazy `evaluate_violations()`: check only the constraints that crossover/mutation can violate

**Observation**: Crossover and mutation in the ROADEF adapter operate on
waypoints only. Segment-limit and connectivity violations can only be introduced
by waypoint changes. Budget and capacity violations depend on the full routing.
A staged check (cheap constraints first, expensive only if needed) could reduce
average cost.

**Risk**: Requires understanding which constraints are affected by which
operators. Incorrect staging would change the feasibility decision and break
trajectory invariants.

### H7 — Eliminate `evaluate_violations()` on the `Ok(true)` path entirely by restructuring `process_offspring()`

**Observation**: If `process_offspring()` is restructured so that the repair
gate is only entered when the caller explicitly requests it (i.e., the caller
passes a `known_feasible: bool` flag), then feasible offspring bypass the gate
entirely. The `improve()` no-op then costs nothing.

**Risk**: Same as H4 — API change required.

---

## 6. Recommended Next Step

**H4 is the primary candidate** because it directly addresses the root cause:
the `is_feasible()` gate at line 25 is called unconditionally even when the
offspring is already known to be feasible. The intervention is surgical and
the trajectory impact is verifiable.

Before implementing H4, the following must be confirmed from source:
1. What fraction of offspring reaching `process_offspring()` are feasible vs infeasible?
2. Does the crossover operator preserve feasibility? Does mutation?
3. Is there a cheap way to pass feasibility state from the call site into `process_offspring()`?

---

## 7. Evidence Files Referenced

| File | Description |
|------|-------------|
| [`evidence/phase9_h3_setA14_corroboration_summary.txt`](evidence/phase9_h3_setA14_corroboration_summary.txt) | Post-H3 setA-14 profile (Improve=74.2%) |
| [`adapters/roadef/src/operators.rs`](adapters/roadef/src/operators.rs) | `RoadefImprovement::improve()` — confirmed no-op |
| [`coralys-core/src/pipeline.rs`](coralys-core/src/pipeline.rs) | `process_offspring()` — `is_feasible()` gate at line 25 |
| [`adapters/roadef/src/pipeline_impl.rs`](adapters/roadef/src/pipeline_impl.rs) | `t_improve_ms` accumulator — times entire `process_offspring()` on `Ok(true)` |