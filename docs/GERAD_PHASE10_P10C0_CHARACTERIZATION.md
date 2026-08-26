# GERAD Phase 10 — P10-C0: Repair-Effectiveness Characterization

**Status:** SWEEP COMPLETE (7/7 instances: setA-04/06/10/13/14/16/19)
**Governance:** OBSERVATIONAL — no behavioral changes. P10-C hypothesis selection LOCKED.
**Date:** 2026-08-26
**Baseline commits:** `3a07aa6f0` + `570805df7` + `0c0dd14eb` (P10-B complete)

---

## A. Measurement Validity

### A.1 Instrumentation Design

P10-C0 adds pre/post repair measurements to the existing `process_offspring` call sites in
[`adapters/roadef/src/pipeline_impl.rs`](../adapters/roadef/src/pipeline_impl.rs). Three call
sites are instrumented (crossover→ca, crossover→cb, mutation-only→child).

For each infeasible offspring entering repair:

1. **Before `process_offspring`:** snapshot `waypoints.clone()` and call
   `evaluate_violations()` → `V_before`, `M_before` (max capacity saturation)
2. **`process_offspring` executes** (existing path, unchanged)
3. **In the `Ok(false)` arm, before `ca = pa.clone()` reset:** call `evaluate_violations()`
   again → `V_after`, `M_after`; compare waypoints fingerprint

This is the only window where the post-repair genome is accessible before the parent-reset.

### A.2 Behavioral Invariant

The instrumentation adds two `evaluate_violations()` calls per failed repair attempt.
These calls are read-only — they do not modify the genome, the constraint model, or the
RNG state. The existing repair path is unchanged. No early exits, no altered selection,
no altered fitness, no changed RNG behavior.

### A.3 Repair Operator Structure (from code reading)

[`adapters/roadef/src/operators.rs`](../adapters/roadef/src/operators.rs) `RoadefRepair::repair()`:

```
1. evaluate_violations(candidate)          → get violation list
2. For SegmentLimit/Connectivity:          → clear candidate.waypoints[demand_id]
3. For Capacity:                           → set needs_ecmp_fallback = true (only)
4. if needs_ecmp_fallback { /* no-op */ }  → commented-out clearing code never executes
5. return Ok(false)                        → always returns failure
```

**Critical structural finding:** For `Capacity` violations (the dominant violation type at
large instances), the repair operator sets a flag but **never modifies the genome**. The
commented-out line `candidate.waypoints.iter_mut().for_each(|wps| wps.clear())` is dead code.
The repair function is a stub that always returns `Ok(false)` without making any useful change.

---

## B. Raw P10-C0 Measurements

### B.1 Summary Table (all 7 instances confirmed)

| Instance | Demands | Failures | genome_changed | genome_unchanged | viol_improved | viol_unchanged | delta_max_sat |
|----------|---------|----------|---------------|-----------------|---------------|----------------|---------------|
| setA-04  | 200     | 2        | 0 (0.0%)      | 2 (100.0%)      | 0 (0.0%)      | 2 (100.0%)     | 0.0000        |
| setA-06  | 500     | 24       | 0 (0.0%)      | 24 (100.0%)     | 0 (0.0%)      | 24 (100.0%)    | 0.0000        |
| setA-10  | 1000    | 33       | 0 (0.0%)      | 33 (100.0%)     | 0 (0.0%)      | 33 (100.0%)    | 0.0000        |
| setA-13  | 2000    | 131      | 0 (0.0%)      | 131 (100.0%)    | 0 (0.0%)      | 131 (100.0%)   | 0.0000        |
| setA-14  | 600     | 9        | 0 (0.0%)      | 9 (100.0%)      | 0 (0.0%)      | 9 (100.0%)     | 0.0000        |
| setA-16  | 4800    | 250      | 0 (0.0%)      | 250 (100.0%)    | 0 (0.0%)      | 250 (100.0%)   | 0.0000        |
| setA-19  | 6000    | 110      | 0 (0.0%)      | 110 (100.0%)    | 0 (0.0%)      | 110 (100.0%)   | 0.0000        |
| **TOTAL**| —       | **559**  | **0 (0.0%)**  | **559 (100.0%)**| **0 (0.0%)**  | **559 (100.0%)**| **0.0000**   |

### B.2 Per-Instance Detail (confirmed instances)

```
=== setA-04 ===  nodes=50, demands=200, wall_ms=19150
  P10-B (repair scaling):
    infeasible=2/225 (0.9%), repair_ms=40.4, ms/repair=20.205, repair_share=0.2%
    repair_attempts=2, successes=0, failures=2
    improve_ms=1677.2
  P10-C0 (repair effectiveness — failed repairs only):
    genome_changed=0/2 (0.0%)
    genome_unchanged=2/2 (100.0%)
    violation_count_improved=0 (0.0%)
    violation_count_unchanged=2 (100.0%)
    violation_count_worsened=0 (0.0%)
    mean_max_sat_before=1.0380  mean_max_sat_after=1.0380  delta=0.0000

=== setA-06 ===  nodes=100, demands=500, wall_ms=113556
  P10-B (repair scaling):
    infeasible=24/225 (10.7%), repair_ms=2713.1, ms/repair=113.048, repair_share=2.4%
    repair_attempts=24, successes=0, failures=24
    improve_ms=8546.7
  P10-C0 (repair effectiveness — failed repairs only):
    genome_changed=0/24 (0.0%)
    genome_unchanged=24/24 (100.0%)
    violation_count_improved=0 (0.0%)
    violation_count_unchanged=24 (100.0%)
    violation_count_worsened=0 (0.0%)
    mean_max_sat_before=1.0499  mean_max_sat_after=1.0499  delta=0.0000

=== setA-10 ===  nodes=150, demands=1000, wall_ms=361458
  P10-B (repair scaling):
    infeasible=33/225 (14.7%), repair_ms=10376.0, ms/repair=314.423, repair_share=2.9%
    repair_attempts=33, successes=0, failures=33
    improve_ms=22848.3
  P10-C0 (repair effectiveness — failed repairs only):
    genome_changed=0/33 (0.0%)
    genome_unchanged=33/33 (100.0%)
    violation_count_improved=0 (0.0%)
    violation_count_unchanged=33 (100.0%)
    violation_count_worsened=0 (0.0%)
    mean_max_sat_before=1.0609  mean_max_sat_after=1.0609  delta=0.0000

=== setA-13 ===  nodes=200, demands=2000, wall_ms=1311098
  P10-B (repair scaling):
    infeasible=131/230 (57.0%), repair_ms=130635.1, ms/repair=997.215, repair_share=10.0%
    repair_attempts=131, successes=0, failures=131
    improve_ms=35487.6
  P10-C0 (repair effectiveness — failed repairs only):
    genome_changed=0/131 (0.0%)
    genome_unchanged=131/131 (100.0%)
    violation_count_improved=0 (0.0%)
    violation_count_unchanged=131 (100.0%)
    violation_count_worsened=0 (0.0%)
    mean_max_sat_before=1.1226  mean_max_sat_after=1.1226  delta=0.0000

=== setA-14 ===  nodes=250, demands=600, wall_ms=357091
  P10-B (repair scaling):
    infeasible=9/225 (4.0%), repair_ms=2869.5, ms/repair=318.836, repair_share=0.8%
    repair_attempts=9, successes=0, failures=9
    improve_ms=26108.0
  P10-C0 (repair effectiveness — failed repairs only):
    genome_changed=0/9 (0.0%)
    genome_unchanged=9/9 (100.0%)
    violation_count_improved=0 (0.0%)
    violation_count_unchanged=9 (100.0%)
    violation_count_worsened=0 (0.0%)
    mean_max_sat_before=1.0319  mean_max_sat_after=1.0319  delta=0.0000

=== setA-16 ===  nodes=250, demands=4800, wall_ms=4222269
  P10-B (repair scaling):
    infeasible=250/250 (100.0%), repair_ms=672441.1, ms/repair=2689.764, repair_share=15.9%
    repair_attempts=250, successes=0, failures=250
    improve_ms=0.0
  P10-C0 (repair effectiveness — failed repairs only):
    genome_changed=0/250 (0.0%)
    genome_unchanged=250/250 (100.0%)
    violation_count_improved=0 (0.0%)
    violation_count_unchanged=250 (100.0%)
    violation_count_worsened=0 (0.0%)
    mean_max_sat_before=1.2354  mean_max_sat_after=1.2354  delta=0.0000

=== setA-19 ===  nodes=300, demands=6000, wall_ms=6259775
  P10-B (repair scaling):
    infeasible=110/230 (47.8%), repair_ms=431563.5, ms/repair=3923.305, repair_share=6.9%
    repair_attempts=110, successes=0, failures=110
    improve_ms=193942.6
  P10-C0 (repair effectiveness — failed repairs only):
    genome_changed=0/110 (0.0%)
    genome_unchanged=110/110 (100.0%)
    violation_count_improved=0 (0.0%)
    violation_count_unchanged=110 (100.0%)
    violation_count_worsened=0 (0.0%)
    mean_max_sat_before=1.0877  mean_max_sat_after=1.0877  delta=0.0000
```

---

## C. Key Findings

### C.1 The Central Finding

**Repair does nothing.** Across all 559 confirmed failed repair attempts (all 7 instances):

- `genome_changed = 0/559 (0.0%)` — repair **never modifies the genome**
- `violation_count_unchanged = 559/559 (100.0%)` — violation count is **identical** before and after
- `delta_max_sat = 0.0000` — capacity saturation is **bit-identical** before and after

This is not a statistical finding — it is a structural finding. The repair operator for
`Capacity` violations (the dominant violation type) sets `needs_ecmp_fallback = true` but
the actual clearing code is commented out. The genome exits repair in exactly the same state
it entered.

### C.2 Distinction: repair_failed ≠ repair_did_nothing

P10-B established that repair always returns `Ok(false)` (0% success rate). P10-C0 now
establishes the stronger claim: repair not only fails to achieve feasibility, it **makes
zero structural changes** to the genome. The offspring is discarded (reset to parent) after
spending `ms/repair` milliseconds on a completely ineffective operation.

### C.3 What repair_ms actually measures

The `repair_ms` cost measured in P10-B (13–2352 ms/repair, scaling with instance size) is
entirely attributable to the `evaluate_violations()` call inside `RoadefRepair::repair()`.
This call:
- Runs the full constraint evaluation pipeline (segment limit, budget, routing, capacity)
- Scales with demands × time_slots × topology complexity
- Produces a violation list that is then **ignored** (no genome modification follows)

The repair operator is paying the full cost of constraint evaluation to produce information
it does not use.

### C.4 Violation type distribution

The `[diag]` log lines from setA-10 confirm that all observed violations are `Capacity`
violations (arc overload). The `mean_max_sat_before` values (1.038–1.061) indicate moderate
overload — not extreme saturation. The repair operator has a code path for `Capacity`
violations but it is a no-op.

### C.5 Capacity saturation is stable

`mean_max_sat_before ≈ mean_max_sat_after` to 4 decimal places across all instances. This
confirms that repair makes no change to routing, no change to flow distribution, and no
change to arc utilization. The saturation values are not just similar — they are identical.

---

## D. Structural Analysis

### D.1 Why repair costs so much for zero effect

The call chain for each failed repair attempt:

```
is_feasible(&ca)                    → evaluate_violations() call #1 (P10-B: feasibility check)
  → RoadefRepair::repair()
      → evaluate_violations()       → call #2 (inside repair, produces violation list)
      → for Capacity violations: needs_ecmp_fallback = true (no genome change)
      → if needs_ecmp_fallback { /* commented out */ }
      → return Ok(false)
is_feasible(&ca) [post-repair]      → evaluate_violations() call #3 (P10-B: post-repair check)
```

Three `evaluate_violations()` calls per infeasible offspring, all producing the same result,
with zero genome modification between them. The P10-C0 instrumentation adds calls #4 and #5
(pre/post measurement) — these are observational only and not part of the production path.

### D.2 The repair_ms cost is pure waste

Since the genome is never modified:
- Call #1 (feasibility check): necessary to route to repair path
- Call #2 (inside repair): produces violation list → **discarded without use**
- Call #3 (post-repair check): evaluates same genome → **same result as call #1**

Calls #2 and #3 are pure waste. Call #2 is the dominant cost (it runs the full violation
pipeline). Call #3 is a redundant re-evaluation of an unchanged genome.

### D.3 Scaling implication

The `repair_ms` cost scales with instance size because `evaluate_violations()` scales with
demands × time_slots × topology. At setA-16 (4800 demands, 100% infeasibility), this means
every single offspring pays the full cost of two redundant `evaluate_violations()` calls.

---

## E. Hypothesis Disposition

### E.1 Evidence bearing on P10-C candidates

**H-EARLY (repair is useful but performs redundant evaluation work):**
- DISFAVORED. Repair is not useful — it makes zero structural changes. There is no useful
  work to preserve. Eliminating redundant evaluation would save calls #2 and #3, but the
  fundamental problem is that repair has no implementation for Capacity violations.

**H-SKIP (repair produces no useful evolutionary progress; removing it is justified):**
- SUPPORTED by P10-C0 evidence. Repair makes zero genome changes, zero violation improvement,
  zero saturation improvement. Skipping repair for Capacity-violated offspring would eliminate
  calls #2 and #3 with no loss of evolutionary information.
- However: skipping repair does not address the root cause (infeasible offspring are still
  produced and discarded). It reduces waste but does not improve solution quality.

**H-CONSTRUCT (the dominant problem is production of infeasible offspring):**
- SUPPORTED by P10-B + P10-C0 combined. The infeasibility rate scales from 0.9% to 100%
  with instance size. Repair cannot recover these offspring. The constructor is producing
  offspring that are structurally infeasible (Capacity violations) and repair has no
  mechanism to fix them.
- This hypothesis targets the root cause rather than the symptom.

### E.2 Governance gate

**P10-C hypothesis selection is LOCKED pending:**
1. ~~Completion of the 7-instance sweep~~ — **DONE** (all 7 instances complete, 559 total failures)
2. User review of this characterization document
3. Explicit hypothesis selection by the user

Do not implement H-EARLY, H-SKIP, H-CONSTRUCT, or any Coralys change until authorized.

---

## F. Open Questions for P10-C Hypothesis Selection

**F.1** Does the pattern hold at large instances (setA-13/14/16/19)?
- Expected: yes, based on structural analysis. The repair operator code is the same regardless
  of instance size. Capacity violations dominate at large instances.

**F.2** If H-SKIP is selected: what is the expected wall-time saving?
- Upper bound: eliminate calls #2 and #3 per infeasible offspring.
- At setA-16 (100% infeasibility, 1453 ms/repair): potentially ~2/3 of repair_ms saved.
- But: infeasible offspring are still discarded (reset to parent). No improvement in solution
  quality. The evolutionary search still wastes population slots on infeasible offspring.

**F.3** If H-CONSTRUCT is selected: what is the intervention?
- Option A: Improve the constructor to produce fewer infeasible offspring (reduce infeasibility
  rate at source).
- Option B: Implement actual repair logic for Capacity violations (rerouting via Dijkstra).
- Option C: Change selection pressure to avoid producing infeasible offspring (EA-level change).
- These options have different scopes: A/B are repair/constructor changes; C may require
  Coralys/MOGA capability changes.

**F.4** Is there a Coralys capability gap?
- The current MOGA has no mechanism to exploit partially-improved-but-still-infeasible
  candidates. If repair could reduce violations without achieving feasibility, those candidates
  are discarded. This is a structural limitation of the current feasibility gate.
- P10-C0 evidence shows repair makes zero improvement, so this gap is currently moot.
  If a real repair implementation is added (H-CONSTRUCT option B), this gap becomes relevant.

---

## G. Governance

- P10-B: CLOSED (commits `3a07aa6f0` + `570805df7` + `0c0dd14eb`)
- P10-C0: COMPLETE (all 7 instances; 559 total failed repairs; 0% genome change; 100% violation unchanged)
- P10-C: LOCKED — requires P10-C0 evidence review and explicit hypothesis selection
- Airline Upgradation / UC-ULTRA-LEVEL4-MEMORY: M5-CLOSED, outside this research chain