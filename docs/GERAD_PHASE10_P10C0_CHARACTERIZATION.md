# GERAD Phase 10 — P10-C0: Repair-Effectiveness Characterization

**Status:** SWEEP COMPLETE (7/7 instances: setA-04/06/10/13/14/16/19)
**Governance:** OBSERVATIONAL — no behavioral changes. P10-C hypothesis selection: **H-SKIP+CONSTRUCT AUTHORIZED** (2026-08-26).
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
  zero saturation improvement. Bypassing repair for Capacity-violated offspring eliminates
  loop overhead with no loss of evolutionary information.
- Quantitative timing benefit not yet characterized — a pre/post A/B runtime measurement
  (same 7 instances × same seeds × same optimizer settings) would be required to claim a
  specific speedup. The 40.63s build time is compilation time, not execution time.
- Skipping repair does not address the root cause (infeasible offspring are still produced
  and discarded). It reduces waste but does not improve solution quality.

**H-CONSTRUCT (the dominant problem is production of infeasible offspring):**
- SUPPORTED by P10-B + P10-C0 combined. The infeasibility rate scales from 0.9% to 100%
  with instance size. Repair cannot recover these offspring. The constructor is producing
  offspring that are structurally infeasible (Capacity violations) and repair has no
  mechanism to fix them.
- This hypothesis targets the root cause rather than the symptom.

### E.2 Hypothesis Selection Decision (2026-08-26)

**H-SKIP+CONSTRUCT AUTHORIZED.**

| Hypothesis      | Decision                     | Reason |
|-----------------|------------------------------|--------|
| H-EARLY         | ❌ Rejected                  | Repair is not useful — zero genome changes. No useful work to preserve. |
| H-SKIP          | ✅ Authorized immediately    | Repair is provably a no-op for Capacity violations; continuing to execute it wastes compute. |
| H-CONSTRUCT     | ✅ Authorized as investigation | Evidence identifies a structural feasibility problem; P10-C1 must determine which intervention is correct. |
| H-SKIP+CONSTRUCT| ✅ Authorized                | Immediate safe cleanup + properly governed investigation. |

**Authorized immediately:** H-SKIP — bypass the demonstrated no-op Capacity repair path, with observational telemetry preserved. Implemented in `1ddc6fa84`. Timing benefit not yet quantified — a pre/post A/B runtime measurement would be required to claim a specific speedup.

**Authorized next:** P10-C1 — Bottleneck Arc Characterization.

**Not yet authorized:**
- Implementing Dijkstra rerouting or ECMP fallback
- Changing crossover, mutation, or selection
- Changing Coralys-core interfaces
- Declaring the constructor the root cause

**Governance language:**
> P10-C0 establishes that the current ROADEF Capacity repair path is structurally inert. P10-C1 is authorized to determine where the actual feasibility loss originates. No substantive repair/construction/operator behavior is to be changed until that characterization is complete.

---

## F. P10-C1 Research Questions (Bottleneck Arc Characterization)

P10-C1 is a **forensic lineage experiment**, not a repair implementation. The goal is causal discrimination across five hypotheses before any behavioral intervention.

### F.0 Causal hypotheses to discriminate

1. **Topology-constrained bottleneck** — most/all relevant demand genuinely has to traverse that arc.
2. **Construction bias** — alternatives exist, but the initial constructor overwhelmingly chooses the bottleneck.
3. **Representation limitation** — the genome cannot adequately express the useful alternatives.
4. **Variation destruction** — useful alternatives exist initially but crossover/mutation destroys or fails to preserve them.
5. **Selection dynamics** — alternatives exist but are systematically eliminated by other objectives.

**Do not assume "constructor" yet.** H-CONSTRUCT is the hypothesis family, not the conclusion.

### F.1 Lineage record structure (per dominant arc: 968/658/303/606)

| Field | Question |
|-------|----------|
| First appearance | At what generation does the arc become overloaded? |
| Origin | Initial / crossover / mutation / inherited |
| Parent 1 | What genome produced it? |
| Parent 2 | If crossover, what was the other parent? |
| Parent bottleneck state | Did either parent already use the arc? |
| Child bottleneck state | Did the operation introduce/increase its use? |
| Demand set | Which demands contribute to the arc load? |
| Alternative paths | What alternative topology exists for those demands? |
| Representation | Can those alternatives actually be represented in the genome? |
| Persistence | Does the bottleneck survive selection across generations? |

### F.2 Transition classification (for each first-appearance event)

```
A. parent already overloaded → inheritance / selection question
B. neither parent overloaded → crossover/mutation/construction representation question
C. parent had alternative → child lost alternative → destructive variation candidate
D. alternative is representable but never appears → construction / exploration candidate
E. alternative cannot be represented → representation gap
F. alternative exists topologically but cannot satisfy constraints → constrained topology
```

### F.3 P10-C1 execution order

- **C1-A** — Bottleneck census: exact frequency/load contribution of 968/658/303/606
- **C1-B** — First-appearance lineage: capture first genome and generation for each dominant bottleneck
- **C1-C** — Parent comparison: compare parent genomes and offspring around first appearance
- **C1-D** — Alternative-path availability: determine whether viable alternatives exist for the affected demand set (without implementing rerouting)
- **C1-E** — Representation test: determine whether those alternatives can be expressed by the existing genome
- **C1-F** — Causal classification: assign evidence-supported classification (construction / crossover / mutation / representation / topology)

Only after C1-F should a behavioral intervention be authorized.

### F.4 Scope constraints

- Compiler warnings (unused telemetry variables, unused imports) are non-blocking technical debt. Do not clean them during P10-C1 — behavioral code must remain frozen during the causal experiment.
- Do not implement Dijkstra, ECMP, constructor changes, or operator changes during P10-C1.
- P10-C1 is observational lineage tracing only.

---

## G. Open Questions (Pre-P10-C1)

**G.1** H-SKIP expected wall-time saving:
- Upper bound: eliminate `evaluate_violations()` calls #2 and #3 per infeasible offspring.
- At setA-16 (100% infeasibility, 2690 ms/repair): potentially ~2/3 of repair_ms recovered.
- Infeasible offspring are still discarded (reset to parent). No improvement in solution quality.
- H-SKIP is a performance correction, not a solution to the feasibility problem.

**G.2** Coralys capability gap (currently moot, relevant if H-CONSTRUCT option B is implemented):
- The current MOGA has no mechanism to exploit partially-improved-but-still-infeasible candidates.
- P10-C0 shows repair makes zero improvement, so this gap is moot for now.
- If a real repair implementation is added, this gap becomes relevant.

**G.3** Architectural boundary (confirmed by P10-C0):
- MOGA is excellent at exploiting/diversifying a good foothold.
- MOGA is not demonstrated to discover feasibility from pathological infeasible starting populations.
- Domain intelligence (construction, repair) belongs below the factory boundary.
- `SeededScheduleFactory` abstraction earns its keep: UltraCrew can evolve from 1 seed → portfolio of domain-generated seeds without contaminating Coralys-core.

---

## H. Governance

- P10-B: CLOSED (commits `3a07aa6f0` + `570805df7` + `0c0dd14eb`)
- P10-C0: CLOSED (commit `66674fc96`; all 7 instances; 559 total failed repairs; 0% genome change; 100% violation unchanged)
- P10-C hypothesis selection: CLOSED — H-SKIP+CONSTRUCT authorized 2026-08-26
- H-SKIP: CLOSED (`1ddc6fa84`) — performance correction implemented and behaviorally safe; quantitative timing benefit not yet characterized (no A/B runtime measurement exists)
- P10-C1: CLOSED — Bottleneck Arc Characterization (C1-A through C1-F) COMPLETE. Arc 658 classified CONSTRUCTED (heuristic-biased). Final commit `188d5b32e`.
- P10-C2: CLOSED (2026-08-28) — NEGATIVE RESULT. Saturation-penalty sweep complete.
  - Binary: `adapters/roadef/src/bin/phase10c2_penalty_sweep.rs` (commit `d4223807f`)
  - Evidence: commits `02ed986aa` (control v2) + `197b285b7` (full sweep, 10 files)
  - Sweep results (seed=42, genomes=50, authoritative evaluator path):
    - penalty=100:  overloaded=13/50, arc658_sel=2009, max_sat=1.0128
    - penalty=200:  overloaded=13/50, arc658_sel=2009, max_sat=1.0128
    - penalty=400:  overloaded=13/50, arc658_sel=2009, max_sat=1.0128
    - penalty=800:  overloaded=12/50, arc658_sel=2007, max_sat=1.0128
    - penalty=1600: overloaded=12/50, arc658_sel=2001, max_sat=1.0128
  - Finding: saturation penalty coefficient is NOT the binding control variable.
    16× penalty increase (100→1600) produces only 0.4% reduction in Arc 658 selections.
    Max saturation unchanged at 1.0128 across all conditions.
  - Causal refinement: the penalty term is effectively non-binding for this routing
    decision. Arc 658's base metric advantage exceeds even penalty=1600×0.12=192.
    The heuristic bias is structural, not a coefficient-tuning problem.
  - Production coefficient: UNCHANGED at 100.0. No production change justified.
    The 24% vs 26% overload difference at penalty=800/1600 is within noise and
    the primary causal metric (Arc-658 selection) is essentially unchanged.
  - Causal chain established: Demand → candidate routes → metric comparison →
    Arc 658 repeatedly wins → saturation penalty has negligible influence → overload.
    NOT: penalty too small → increase penalty → problem solved.
- P10-C3: AUTHORIZED (2026-08-28) — Capacity-aware pre-filter experiment.
  - Hypothesis: a capacity feasibility check at route-selection time (before ranking)
    will prevent Arc 658 from winning when it is structurally incapable of satisfying
    capacity constraints, regardless of its base metric advantage.
  - Scope: `adapters/roadef` only. Coralys core FROZEN.
  - Intervention: add capacity-aware pre-filter to `greedy_load_aware_dijkstra` or
    to the construction loop — reject routes that would push any arc above a
    configurable saturation threshold (e.g. 0.9 or 1.0).
  - Measurements: Arc-658 selection count, overloaded genomes, feasible genomes,
    construction time, genome max_sat distribution.
  - Gate (hard): 5/5 trajectory invariants bit-exact vs Phase 9 baseline (commit `1919018aa`)
  - Gate (hard): T_net > 0 on setA-14 (medium) AND setA-16/setA-19 (large)
  - No-go: no production change before experiment completes and gates pass.
  - Authorization basis: P10-C2 negative result; penalty is non-binding; pre-filter
    is the next logical intervention in the causal chain.
- Airline Upgradation / UC-ULTRA-LEVEL4-MEMORY: M5-CLOSED, outside this research chain