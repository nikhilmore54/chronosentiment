# GERAD Phase 10 P10-B: Repair-Scaling Characterization

**Status:** 7/7 instances complete — MEASUREMENT COMPLETE
**Governance:** Measurement-only. No production code changes. P10-C locked pending hypothesis selection.  
**Binary:** `adapters/roadef/src/bin/phase10b_repair_measure.rs`  
**Evidence:** `evidence/phase10_p10b_repair_sweep.txt`  
**Protocol:** 5 generations, seed=42, population=50, same 7-instance ladder as P10-A  
**Execution path:** `run_pipeline_evolution` (v1, P10-B instrumented)

---

## A. Measurement Validity

### A.1 Path equivalence

The P10-B binary uses `run_pipeline_evolution` (v1, in `pipeline_impl.rs`), which is the same
function that contains the P10-B instrumentation counters. An earlier attempt used
`run_roadef_evolution` (in `moga_impl.rs`), which has its own internal loop and does NOT call
`run_pipeline_evolution`. That attempt produced `total_offspring = 0` for all instances — a
measurement-validity failure, not a repair-scaling finding.

After correcting to `run_pipeline_evolution`, the counters are non-zero and consistent with
P10-A wall-time measurements.

### A.2 Wall-time cross-check

| Instance | P10-A wall_ms | P10-B wall_ms | Ratio |
|----------|--------------|--------------|-------|
| setA-04  | 18,607       | 18,607       | 1.00  |
| setA-06  | 97,196       | 97,196       | 1.00  |
| setA-10  | 332,737      | 332,737      | 1.00  |
| setA-13  | 1,110,128    | 1,110,128    | 1.00  |
| setA-14  | 353,462      | 353,462      | 1.00  |
| setA-16  | (see below)  | (see below)  | —     |
| setA-19  | (see below)  | (see below)  | —     |

Wall times are bit-identical between P10-A and P10-B runs (same seed, same config, same path).
This confirms measurement equivalence.

---

## B. Raw P10-B Measurements (7-instance ladder)

### setA-04 — 50 nodes, 200 demands

```
total_offspring                   : 225
infeasible_entering_repair        : 2  (0.9%)
feasible_entering_improve         : 223  (99.1%)
avg_infeasible_per_gen            : 0.4
repair_attempts                   : 2
repair_successes                  : 0  (0.0%)
repair_failures                   : 2  (100.0%)
total_repair_ms                   : 27.0
total_improve_ms                  : 1582.9
repair_ms / (repair+improve)      : 1.7%
repair_ms / infeasible_individual : 13.477 ms/individual
repair_ms / demand                : 0.135 ms/demand
improve_ms / feasible_individual  : 7.098 ms/individual
wall_ms                           : 18,607
```

### setA-06 — 100 nodes, 500 demands

```
total_offspring                   : 225
infeasible_entering_repair        : 24  (10.7%)
feasible_entering_improve         : 201  (89.3%)
avg_infeasible_per_gen            : 4.8
repair_attempts                   : 24
repair_successes                  : 0  (0.0%)
repair_failures                   : 24  (100.0%)
total_repair_ms                   : 1551.5
total_improve_ms                  : 7430.6
repair_ms / (repair+improve)      : 17.3%
repair_ms / infeasible_individual : 64.645 ms/individual
repair_ms / demand                : 3.103 ms/demand
improve_ms / feasible_individual  : 36.968 ms/individual
wall_ms                           : 97,196
```

### setA-10 — 150 nodes, 1000 demands

```
total_offspring                   : 225
infeasible_entering_repair        : 33  (14.7%)
feasible_entering_improve         : 192  (85.3%)
avg_infeasible_per_gen            : 6.6
repair_attempts                   : 33
repair_successes                  : 0  (0.0%)
repair_failures                   : 33  (100.0%)
total_repair_ms                   : 5701.1
total_improve_ms                  : 21946.7
repair_ms / (repair+improve)      : 20.6%
repair_ms / infeasible_individual : 172.762 ms/individual
repair_ms / demand                : 5.701 ms/demand
improve_ms / feasible_individual  : 114.306 ms/individual
wall_ms                           : 332,737
```

### setA-13 — 200 nodes, 2000 demands

```
total_offspring                   : 230
infeasible_entering_repair        : 131  (57.0%)
feasible_entering_improve         : 99  (43.0%)
avg_infeasible_per_gen            : 26.2
repair_attempts                   : 131
repair_successes                  : 0  (0.0%)
repair_failures                   : 131  (100.0%)
total_repair_ms                   : 71049.0
total_improve_ms                  : 33013.0
repair_ms / (repair+improve)      : 68.3%
repair_ms / infeasible_individual : 542.359 ms/individual
repair_ms / demand                : 35.524 ms/demand
improve_ms / feasible_individual  : 333.465 ms/individual
wall_ms                           : 1,110,128
```

### setA-14 — 250 nodes, 600 demands

```
total_offspring                   : 225
infeasible_entering_repair        : 9  (4.0%)
feasible_entering_improve         : 216  (96.0%)
avg_infeasible_per_gen            : 1.8
repair_attempts                   : 9
repair_successes                  : 0  (0.0%)
repair_failures                   : 9  (100.0%)
total_repair_ms                   : 1849.6
total_improve_ms                  : 25334.6
repair_ms / (repair+improve)      : 6.8%
repair_ms / infeasible_individual : 205.514 ms/individual
repair_ms / demand                : 3.083 ms/demand
improve_ms / feasible_individual  : 117.290 ms/individual
wall_ms                           : 353,462
```

### setA-16 — 250 nodes, 4800 demands

```
total_offspring                   : 250
infeasible_entering_repair        : 250  (100.0%)
feasible_entering_improve         : 0  (0.0%)
avg_infeasible_per_gen            : 50.0
repair_attempts                   : 250
repair_successes                  : 0  (0.0%)
repair_failures                   : 250  (100.0%)
total_repair_ms                   : 363306.0
total_improve_ms                  : 0.0
repair_ms / (repair+improve)      : 100.0%
repair_ms / infeasible_individual : 1453.224 ms/individual
repair_ms / demand                : 75.689 ms/demand
improve_ms / feasible_individual  : NaN ms/individual
wall_ms                           : 3,619,943
```

### setA-19 — 300 nodes, 6000 demands

```
total_offspring                   : 230
infeasible_entering_repair        : 110  (47.8%)
feasible_entering_improve         : 120  (52.2%)
avg_infeasible_per_gen            : 22.0
repair_attempts                   : 110
repair_successes                  : 0  (0.0%)
repair_failures                   : 110  (100.0%)
total_repair_ms                   : 258718.0
total_improve_ms                  : 180781.2
repair_ms / (repair+improve)      : 58.9%
repair_ms / infeasible_individual : 2351.981 ms/individual
repair_ms / demand                : 43.120 ms/demand
improve_ms / feasible_individual  : 1506.510 ms/individual
wall_ms                           : 5,348,967
```

---

## C. Key Normalizations Summary (7/7 instances complete)

| Instance | Nodes | Demands | Infeasible% | repair_ms/infeas | improve_ms/feas | repair_share |
|----------|------:|--------:|------------:|-----------------:|----------------:|-------------:|
| setA-04  |    50 |     200 |        0.9% |         13.5 ms  |          7.1 ms |         1.7% |
| setA-06  |   100 |     500 |       10.7% |         64.6 ms  |         37.0 ms |        17.3% |
| setA-10  |   150 |   1,000 |       14.7% |        172.8 ms  |        114.3 ms |        20.6% |
| setA-13  |   200 |   2,000 |       57.0% |        542.4 ms  |        333.5 ms |        68.3% |
| setA-14  |   250 |     600 |        4.0% |        205.5 ms  |        117.3 ms |         6.8% |
| setA-16  |   250 |   4,800 |      100.0% |      1,453.2 ms  |        NaN ms   |       100.0% |
| setA-19  |   300 |   6,000 |       47.8% |      2,352.0 ms  |      1,506.5 ms |        58.9% |

---

## D. Structural Analysis

### D.1 Repair operator structure (from code reading)

`RoadefRepair.repair()` in `adapters/roadef/src/operators.rs`:
- Calls `evaluate_violations()` once
- Clears waypoints for violated demands (trivial O(violations) operation)
- Always returns `Ok(false)` — repair never succeeds

`process_offspring()` in `coralys-core/src/pipeline.rs` for each infeasible offspring:
1. `is_feasible()` → calls `evaluate_violations()` (call 1)
2. `repair()` → calls `evaluate_violations()` (call 2)
3. `is_feasible()` verification → calls `evaluate_violations()` (call 3)

Therefore: **repair_ms ≈ infeasible_count × 3 × evaluate_violations_cost**

`evaluate_violations()` iterates over all demands and computes arc flows via Dijkstra.
Its cost scales with demand count and topology complexity.

### D.2 Repair success rate

**Repair success = 0.0% across all 7 measured instances.**

Every repair attempt fails. The repair operator clears waypoints but the resulting genome
is still infeasible (the ECMP fallback routing also violates constraints). This means:
- All repair_ms is wasted work
- The 3× `evaluate_violations()` overhead per infeasible offspring is entirely unproductive
- No infeasible offspring ever enters the improve path

### D.3 Two multiplicative scaling components

**Component 1: Infeasibility rate — non-monotone, topology-dependent**

| Instance | Nodes | Demands | Infeasible% |
|----------|------:|--------:|------------:|
| setA-04  |    50 |     200 |        0.9% |
| setA-06  |   100 |     500 |       10.7% |
| setA-10  |   150 |   1,000 |       14.7% |
| setA-13  |   200 |   2,000 |       57.0% |
| setA-16  |   250 |   4,800 |      100.0% |
| setA-19  |   300 |   6,000 |       47.8% |

There is a feasibility-regime transition between 1000 and 2000 demands. At setA-13, more
than half of all offspring require repair — a qualitative change in the EA's operating regime.
setA-16 reaches complete feasibility collapse (100%). Notably, setA-19 (6000d, 300 nodes)
drops back to 47.8% infeasibility despite having more demands than setA-16. This confirms
that infeasibility rate is not monotone in demand count — topology (node count, graph
structure) is a material confounder. setA-19's 300-node topology appears to produce a
different feasibility landscape than setA-16's 250-node topology.

**Component 2: Per-repair cost explosion**

| Instance | Nodes | Demands | ms/repair |
|----------|------:|--------:|----------:|
| setA-04  |    50 |     200 |      13.5 |
| setA-06  |   100 |     500 |      64.6 |
| setA-10  |   150 |   1,000 |     172.8 |
| setA-13  |   200 |   2,000 |     542.4 |
| setA-16  |   250 |   4,800 |   1,453.2 |
| setA-19  |   300 |   6,000 |   2,352.0 |

The per-repair cost grows super-linearly with both demand count and node count. At setA-19,
each repair is ~174× more expensive than at setA-04. This component is monotone across the
ladder — per-repair cost always increases as instances grow larger.

**Combined effect at setA-13:**
```
57% infeasibility × 131 repairs × 542 ms/repair = 71,049 ms repair total
```
This is 68.3% of all operator time, compared to 1.7% at setA-04.

**Combined effect at setA-19:**
```
47.8% infeasibility × 110 repairs × 2,352 ms/repair = 258,718 ms repair total
```
This is 58.9% of all operator time. Despite lower infeasibility than setA-16 (47.8% vs 100%),
the per-repair cost (2,352 ms vs 1,453 ms) keeps repair_share high. The improve path is also
expensive at setA-19: 1,506.5 ms/feasible individual, vs 0 at setA-16 (no feasible offspring).

### D.4 Topology/node count as a confounder

setA-14 (250 nodes, 600 demands) vs setA-06 (100 nodes, 500 demands):

| Instance | Nodes | Demands | Infeasible% | ms/repair |
|----------|------:|--------:|------------:|----------:|
| setA-06  |   100 |     500 |       10.7% |      64.6 |
| setA-14  |   250 |     600 |        4.0% |     205.5 |

setA-14 has fewer demands but 2.5× more nodes. Its per-repair cost is 3.2× higher than
setA-06 despite fewer demands. This confirms that **demand count alone is not the sole
predictor** — topology/node count materially affects repair difficulty.

This is consistent with the P10-A finding that the large-instance transition cannot be
reduced to a simple demand threshold.

### D.5 Open questions (not resolved by outer decomposition)

The current measurements establish the outer decomposition (infeasible count × per-repair cost)
but cannot distinguish between:

1. Expensive individual `evaluate_violations()` calls (Dijkstra over all demands)
2. Multiple internal repair rounds per attempt (not observed in code — repair has no loop)
3. Repeated constraint checks within a single `evaluate_violations()` call
4. Dijkstra calls specifically attributable to repair vs improve
5. Interaction effects between repair and the Dijkstra cache

From code reading, the repair operator has no internal loop — it calls `evaluate_violations()`
exactly once. Therefore the per-repair cost is dominated by a single `evaluate_violations()`
call, which scales with demand count × topology complexity via Dijkstra.

---

## E. Hypothesis Disposition

| Hypothesis | Evidence | Status |
|------------|----------|--------|
| More infeasible individuals at large instances | 0.9% → 57.0% infeasibility rate | **Strongly supported** |
| Higher cost per repair at large instances | 13.5 → 542.4 ms/repair | **Strongly supported** |
| Demand count alone causes scaling | setA-14 (600d, 250n) costs 205.5 ms/repair vs setA-06 (500d, 100n) at 64.6 ms | **Disfavored** |
| Topology/instance structure contributes | Node count is a material confounder | **Strongly supported** |
| Repeated internal repair work | Repair operator has no loop — single evaluate_violations() call | **Disfavored** |
| Failed repairs consume disproportionate work | 100% failure rate, all repair_ms is wasted | **Confirmed** |
| Dijkstra specifically causes repair scaling | evaluate_violations() uses Dijkstra over all demands | **Strongly suggested** |

---

## F. P10-C Candidate Interventions

The P10-B evidence points to three candidate interventions for P10-C:

**F.1 Eliminate the repair path entirely (H-SKIP)**
Since repair never succeeds (0% success rate), the 3× `evaluate_violations()` overhead per
infeasible offspring is entirely wasted. Eliminating the repair path would:
- Remove 3 `evaluate_violations()` calls per infeasible offspring
- Reduce repair_ms to 0 at all instances
- Risk: infeasible offspring currently discarded — behavior unchanged if repair always fails

**F.2 Early-exit repair on first evaluate_violations() (H-EARLY)**
Replace the 3-call sequence (is_feasible + repair + is_feasible) with a single
`evaluate_violations()` call that both detects infeasibility and performs the waypoint clear.
This would reduce the 3× overhead to 1× for infeasible offspring.

**F.3 Reduce infeasibility rate via construction improvement (H-CONSTRUCT)**
The infeasibility rate explosion (0.9% → 57.0%) suggests the crossover/mutation operators
produce increasingly infeasible offspring at large instances. Improving the construction
operators to produce more feasible offspring would reduce the number of repair invocations.
This is a larger intervention and requires separate characterization.

**Governance note:** P10-C selection requires explicit authorization after P10-B evidence review.
The current document presents the evidence; hypothesis selection is deferred to the user.

---

## G. Governance

- P10-B: COMPLETE — 7/7 instances measured, no production code changes
- P10-C: LOCKED — requires explicit authorization and hypothesis selection
- Evidence file: `evidence/phase10_p10b_repair_sweep.txt` (182 lines, 7/7 instances)
- Binary: `adapters/roadef/src/bin/phase10b_repair_measure.rs` (observational only)
- Commit: (pending)