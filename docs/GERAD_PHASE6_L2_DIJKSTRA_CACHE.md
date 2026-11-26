# GERAD Phase 6: L2 Cross-Evaluation Dijkstra Cache

**Status:** 🟢 CLOSED — PROMOTED  
**Commit:** 59e8af59a (implementation) + evidence commit (see below)  
**Date:** 2026-08-24  

---

## 1. Objective

Reduce evaluator runtime by caching `backward_dijkstra` results across evaluations.

**Hypothesis:** The key `(target_node, time_slot)` is genome-independent — `disabled_arcs` is
determined solely by `scenario.interventions[time_slot]` (immutable scenario data). Therefore
the same Dijkstra result can be served to any evaluation that needs the same `(target, slot)` pair,
regardless of which genome is being evaluated.

**Expected benefit:** On large instances with many demands sharing the same target nodes, the L2
cache eliminates redundant Dijkstra computations across the 2,000+ evaluations per run.

---

## 2. Implementation

### 2.1 `ecmp.rs`
- Added `#[derive(Clone)]` to `DijkstraResult` (required for L2 cache insertion).

### 2.2 `evaluator.rs`
- Added `L2DijkstraCache` type alias: `Arc<RwLock<HashMap<(u64, usize), DijkstraResult>>>`
- Added `evaluate_solution_l2cached()` — semantically identical to `evaluate_solution_cached()`
  but additionally accepts a run-level `L2DijkstraCache`.
- Cache key: `(target_node, time_slot)` — genome-independent.
- L2 hit: serve cached `DijkstraResult` (read lock, no recomputation).
- L2 miss: run `backward_dijkstra`, clone result into L2 cache (write lock).
- Within-evaluation L1 Dijkstra cache retained as fast path for demands sharing the same target
  within a single evaluation call.
- Thread-safe: `Arc<RwLock<...>>` allows concurrent reads (Rayon) and exclusive writes.

### 2.3 `moga_impl.rs`
- Added `l2_cache: Option<L2DijkstraCache>` field to `RoadefFitnessEvaluator`.
- `evaluate()` dispatches to `evaluate_solution_l2cached()` when `Some`, falls back to
  `evaluate_solution_cached()` (Phase 4 baseline) when `None`.
- All 14 existing construction sites updated with `l2_cache: None` (backward-compatible).

### 2.4 `bin/phase6_l2_ab.rs`
- A/B harness: Arm A = Phase 4 baseline (L1 + Rayon, no L2), Arm B = Phase 4 + L2.
- Checks all 5 invariants: `best_obj`, `n_actual_evals`, `generations_run`, `valid`, `cache_hits`.
- Reports `T_net = eval_time_A - eval_time_B` (paired comparison, not vs historical reference).

---

## 3. Governance Protocol

### 3.1 Promotion criterion
Per the governance analysis established during this phase:

> L2 is promoted only if it:
> 1. Preserves the exact Phase 4 trajectory (all 5 invariants bit-exact identical)
> 2. Demonstrates positive net evaluator savings against the **contemporaneous** Arm A control
>    (not against the historical 185,159ms reference, which has run-to-run variance)

### 3.2 Invariants checked
| # | Invariant | Rationale |
|---|---|---|
| 1 | `best_obj` bit-exact | Search trajectory unchanged |
| 2 | `n_actual_evals` exact | No evaluations skipped or added |
| 3 | `generations_run` exact | Termination condition unchanged |
| 4 | `valid` exact | Feasibility classification unchanged |
| 5 | `cache_hits` (L1) exact | L1 genome cache behaviour unchanged |

---

## 4. Evidence

### 4.1 setA-01 gate (required)

```
Instance   : setA-01
Seed       : 42
Generations: 50
Pop size   : 50

Arm A (Phase 4 baseline, l2_cache=None):
  best_obj         : 51.0126807372
  valid            : true
  generations_run  : 50
  n_actual_evals   : 1802
  cache_hits       : 250
  eval_time_ms     : 928.01
  wall_clock_ms    : 7139

Arm B (Phase 4 + L2, l2_cache=Some):
  best_obj         : 51.0126807372
  valid            : true
  generations_run  : 50
  n_actual_evals   : 1802
  cache_hits       : 250
  eval_time_ms     : 785.90
  wall_clock_ms    : 7141
  l2_cache_entries : 40

Invariant Verification:
  [PASS] best_obj bits identical
  [PASS] n_actual_evals identical
  [PASS] generations_run identical
  [PASS] valid identical
  [PASS] cache_hits identical

Performance:
  T_net (A_eval - B_eval) : +142.11ms  ← PROMOTION CRITERION MET
  eval_speedup            : 1.18x
  wall_clock_speedup      : 1.00x  (eval is only 13% of wall on setA-01)
```

### 4.2 setA-14 corroboration (large instance)

```
Instance   : setA-14
Seed       : 42
Generations: 50
Pop size   : 50

Arm A (Phase 4 baseline, l2_cache=None):
  best_obj         : 86.1250850504
  valid            : true
  generations_run  : 50
  n_actual_evals   : 2006
  cache_hits       : 181
  eval_time_ms     : 199179.72
  wall_clock_ms    : 1781788

Arm B (Phase 4 + L2, l2_cache=Some):
  best_obj         : 86.1250850504
  valid            : true
  generations_run  : 50
  n_actual_evals   : 2006
  cache_hits       : 181
  eval_time_ms     : 159602.90
  wall_clock_ms    : 1709382
  l2_cache_entries : 500

Invariant Verification:
  [PASS] best_obj bits identical: A=4635760931014063151  B=4635760931014063151
  [PASS] n_actual_evals identical: A=2006  B=2006
  [PASS] generations_run identical: A=50  B=50
  [PASS] valid identical: A=true  B=true
  [PASS] cache_hits identical: A=181  B=181

Performance:
  T_net (A_eval - B_eval) : +39,576.83ms  ← PROMOTION CRITERION MET
  eval_speedup            : 1.25x
  wall_clock_speedup      : 1.04x
  l2_cache_entries        : 500
```

### 4.3 Cross-phase trajectory verification (setA-01)

| Metric | Ph3 Arm A (L1 seq) | Ph3 Arm B (Rayon) | Ph6 Arm A (Ph4 baseline) | Ph6 Arm B (Ph4+L2) |
|---|---|---|---|---|
| best_obj | 51.0126807372 | 51.0126807372 | 51.0126807372 | 51.0126807372 |
| n_actual_evals | 1802 | 1802 | 1802 | 1802 |
| generations_run | 50 | 50 | 50 | 50 |
| valid | true | true | true | true |
| cache_hits | 250 | 250 | 250 | 250 |

All 5 invariants are bit-exact identical across all four arms across Phase 3 and Phase 6.

### 4.4 Phase 4 baseline variance note

The frozen Phase 4 reference eval_time (185,159ms) vs this run's Arm A (199,180ms) represents
+7.6% run-to-run variance — normal machine noise, not a regression. The promotion criterion
correctly uses the paired Arm A as the control, not the historical reference.

---

## 5. Performance Summary

| Instance | L2 entries | eval_speedup | T_net | wall_speedup |
|---|---|---|---|---|
| setA-01 | 40 | 1.18x | +142ms | 1.00x |
| setA-14 | 500 | 1.25x | +39,577ms | 1.04x |

The L2 cache is more effective on larger instances (setA-14: 500 unique (target, slot) pairs vs
setA-01: 40). The wall-clock speedup is modest because eval is ~10% of wall-clock (Rayon already
parallelises the dominant cost); the eval-phase savings are real and compound with future phases.

---

## 6. Promotion Decision

**PROMOTED.** Both gates pass. All 5 invariants are bit-exact identical. T_net > 0 on both
instances. The L2 cache is now the production baseline.

**This promotion does not constitute:**
- Evidence of policy effectiveness
- Production readiness
- Universal historic equivalence across all instances

---

## 7. Commit Chain

```
342adbc8   Phase 2 L1 genome cache baseline
68e3a51be  Phase 3 Rayon parallel evaluation
abc0a43ce  Validated state (44/44 tests pass)
4a691cdd2  Phase 4 setA-14 corroborating evidence
ca2b58928  Phase 5 evaluator decomposition profile
59e8af59a  Phase 6 L2 Dijkstra cache implementation
[next]     Phase 6 evidence + closure commit
```

---

## 8. Next Phase

Phase 7 options (to be scoped separately):
- L3: cross-run warm-start cache (persist L2 across runs)
- Operator-level parallelism (crossover/mutation in parallel)
- Instance-adaptive Rayon thread pool sizing