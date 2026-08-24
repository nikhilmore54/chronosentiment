# Phase 9 P9-B — H6 Precondition Characterization: Staged Early-Exit in `evaluate_violations()`

**Status: PRECONDITIONS CONFIRMED — H6 VIABLE**
**Date: 2026-08-24**
**Baseline: post-H3 (`bb9672750`)**

---

## 1. Hypothesis Statement

**H6 (Staged Early-Exit)**: `evaluate_violations()` in
[`adapters/roadef/src/constraints.rs`](adapters/roadef/src/constraints.rs:26)
currently runs all 4 constraint stages unconditionally and collects every
violation. The caller (`is_feasible()`) only checks `.is_empty()` — it never
inspects the violation details. Therefore, returning early on the first
violation found is semantically equivalent to the current implementation and
eliminates the expensive Stage 3+4 routing computation for any offspring that
fails Stage 1 or Stage 2.

---

## 2. Precondition Analysis

### PC-H6-1: Is early-exit semantically correct?

**YES.**

[`is_feasible()`](coralys-core/src/operators.rs) is defined as:

```rust
fn is_feasible(&self, candidate: &G) -> bool {
    self.evaluate_violations(candidate).is_empty()
}
```

The caller only needs to know whether the violations vector is empty. It never
reads violation details. Returning early with a non-empty vector on the first
violation found produces the same `.is_empty()` result as running all stages.

**Semantic equivalence: confirmed.**

### PC-H6-2: Cost ordering of the 4 stages

From [`constraints.rs`](adapters/roadef/src/constraints.rs:26):

| Stage | Operation | Complexity | Notes |
|-------|-----------|------------|-------|
| 1 | Segment limit | O(D) | One comparison per demand per time slot; no routing |
| 2 | Budget | O(D×T) | `SrPathBit::dist()` per demand per time slot pair |
| 3+4 | Routing + Capacity | O(T × D × `expand_sr_path`) | `backward_dijkstra` per waypoint segment per demand per time slot — **dominant cost** |

Stage 3+4 is the dominant cost. Stage 1 is essentially free. Stage 2 is
moderate. Early-exit after Stage 1 or Stage 2 avoids the entire Stage 3+4
routing computation for any offspring that fails those checks.

### PC-H6-3: Can Stage 1/2 violations occur in practice?

**Stage 1 (segment limit)**: Fires when `waypoints.len() + 1 > max_segments`.
Both [`RoadefMutator`](adapters/roadef/src/moga_impl.rs:952) and
[`PeakTargetedMutator`](adapters/roadef/src/moga_impl.rs:1071) can set
waypoints to a random node (op 1) or add a waypoint (op 2). If `max_segments`
is small (e.g., 2 or 3), Stage 1 violations are structurally possible.
[`RoadefCrossover`](adapters/roadef/src/moga_impl.rs:1139) inherits waypoints
from parents; if a parent has many waypoints, the child can inherit them.

**Stage 2 (budget)**: Fires when the Hamming distance between consecutive
time-slot waypoint assignments exceeds the per-slot budget. This depends on
instance parameters (`scenario.budget`). Crossover mixing waypoints from
different parents across time slots can produce large budget violations.

Whether these violations occur frequently enough to produce meaningful early-exit
savings is **unknown from source alone** — it requires runtime measurement.
However, the key asymmetry is: even a small early-exit rate on Stage 1 is
essentially free (Stage 1 is O(D)), while the cost avoided (Stage 3+4) is very
large.

### PC-H6-4: Is there any ordering dependency between stages?

**NO.** Stages 1, 2, 3, and 4 are independent constraint checks. Stage 3 does
not depend on Stage 1 or Stage 2 results. The current code runs them in order
only because they are written sequentially. Early-exit after Stage 1 or Stage 2
is safe.

### PC-H6-5: Does early-exit affect the repair path?

The repair path in [`operators.rs`](adapters/roadef/src/operators.rs:28) calls
`evaluate_violations()` and inspects the violation types to decide which demands
to repair:

```rust
let violations = model.evaluate_violations(candidate);
if violations.is_empty() {
    return Ok(true); // Already feasible
}
for v in violations {
    match v {
        RoadefViolation::SegmentLimit { demand_id, .. } | ...
```

**The repair path DOES inspect violation details.** Early-exit in
`evaluate_violations()` would return only the first violation found, not all
violations. This would break the repair operator's ability to identify all
demands needing repair.

**Resolution**: H6 must be implemented as a separate `is_feasible_fast()` method
(or an `early_exit: bool` parameter) that is called only from `is_feasible()`,
not from the repair operator. The repair operator must continue to call the
full `evaluate_violations()`.

This is a clean separation: `is_feasible()` → `evaluate_violations_early_exit()`
for the feasibility gate; `repair()` → `evaluate_violations()` (unchanged) for
the repair path.

---

## 3. H6 Intervention Design

### Change 1: Add `is_feasible_fast()` to `RoadefConstraintModel`

```rust
/// Fast feasibility check: returns false as soon as the first violation is found.
/// Semantically equivalent to is_feasible() but avoids Stage 3+4 routing
/// computation when Stage 1 or Stage 2 violations are present.
/// Must NOT be used by the repair operator (which needs all violations).
pub fn is_feasible_fast(&self, candidate: &RoadefGenome) -> bool {
    let solution = candidate.to_solution();
    let scenario = &self.evaluator.scenario;

    // Stage 1: Segment limit (O(D)) — early exit
    if scenario.max_segments >= 0 {
        for path in &solution.srpaths {
            if path.w.len() + 1 > scenario.max_segments as usize {
                return false;
            }
        }
    }

    // Stage 2: Budget (O(D×T)) — early exit
    let tm = &self.evaluator.tm;
    let mut prev_paths: HashMap<u64, SrPathBit> = HashMap::new();
    for ts in 0..tm.num_time_slots {
        let mut budget_cost = 0;
        let mut curr_paths: HashMap<u64, SrPathBit> = HashMap::new();
        for (d_id, demand) in tm.demands.iter().enumerate() {
            let mut bitpath = SrPathBit::new_uninitialized();
            if let Some(srpath) = solution.srpaths.iter().find(|p| p.d == d_id && p.t == ts) {
                bitpath = SrPathBit::new_explicit(demand.s, demand.t, &srpath.w);
            }
            if ts > 0 {
                let uninit = SrPathBit::new_uninitialized();
                let prev_bitpath = prev_paths.get(&(d_id as u64)).unwrap_or(&uninit);
                budget_cost += bitpath.dist(prev_bitpath);
            }
            curr_paths.insert(d_id as u64, bitpath);
        }
        if ts > 0 {
            let budget_val = scenario.budget.iter().find(|b| b.t == ts).map(|b| b.value).unwrap_or(0);
            if budget_cost > budget_val {
                return false;
            }
        }
        prev_paths = curr_paths;
    }

    // Stage 3+4: Routing + Capacity (expensive) — only reached if Stage 1+2 pass
    // ... (same as current evaluate_violations Stage 3+4, but returns false on first violation)
    true
}
```

### Change 2: Override `is_feasible()` on `RoadefConstraintModel` to call `is_feasible_fast()`

```rust
impl ConstraintModel<RoadefGenome> for RoadefConstraintModel {
    fn is_feasible(&self, candidate: &RoadefGenome) -> bool {
        self.is_feasible_fast(candidate)
    }
    // evaluate_violations() unchanged — still used by repair operator
}
```

### Change 3: No change to repair operator

[`RoadefRepair::repair()`](adapters/roadef/src/operators.rs:22) calls
`model.evaluate_violations(candidate)` directly and inspects violation types.
This is unchanged.

---

## 4. Trajectory Preservation Analysis

The feasibility decision is identical: `is_feasible_fast()` returns `false` iff
`evaluate_violations()` would return a non-empty vector. The repair/improve
routing in `process_offspring()` is unchanged. The evolutionary trajectory
(which offspring are accepted, which are repaired, which are dropped) is
bit-exact.

**Trajectory preservation: confirmed by construction.**

---

## 5. Expected Benefit

The benefit depends on the fraction of infeasible offspring that fail Stage 1
or Stage 2 before reaching Stage 3+4. This is unknown without runtime
measurement. However:

- If even 10% of infeasible offspring fail Stage 1 (O(D) check), those avoid
  the full O(T×D×routing) Stage 3+4 computation.
- The 94% feasible offspring are unaffected — they pass all stages and reach
  Stage 3+4 regardless.
- The overhead on the feasible path is: Stage 1 check (O(D)) + Stage 2 check
  (O(D×T)) before Stage 3+4. This is the same work as the current
  implementation — no overhead added.

**The feasible path cost is unchanged. The infeasible path cost is reduced or
equal. H6 cannot make things worse.**

---

## 6. Governance Decision

H6 preconditions are confirmed. The intervention is:
1. Structurally sound (PC-H6-1, PC-H6-4)
2. Semantically safe (PC-H6-1, PC-H6-5 resolved)
3. Cannot increase cost on the feasible path (PC-H6-5)
4. Requires no API changes to `coralys-core`
5. Repair operator is unaffected

**H6 is APPROVED for intervention.**

Next step: implement `is_feasible_fast()`, override `is_feasible()` on
`RoadefConstraintModel`, run setA-01 gate (5/5 trajectory invariants + T_net),
then corroborate on setA-14.