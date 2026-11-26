# GERAD Phase 10 P10-A — Characterization Report

**Status:** COMPLETE  
**Date:** 2026-08-25  
**Baseline:** commit 1919018aa (Phase 9 H6-revised, generation-scoped Dijkstra cache)  
**Evidence file:** `evidence/phase10_p10a_crossover_sweep.txt`  
**Measurement binary:** `phase9_dijkstra_measure` — 5 fixed generations, seed=42, no production-path changes

---

## A. Cache Characterization

### Raw measurements (5 gens, seed=42)

| Instance | Demands | Nodes | Total calls | Calls/gen | dijkstra_ms | improve_ms | repair_ms | wall_ms | dijkstra/wall | Cache reuse |
|---|---|---|---|---|---|---|---|---|---|---|
| setA-04 | 200 | 50 | 30,262 | 6,052 | 658.7ms | 1,362.9ms | 354.6ms | 4,010ms | 16.4% | 99.8% |
| setA-06 | 500 | 100 | 74,886 | 14,977 | 2,966.3ms | 5,937.1ms | 5,451.5ms | 22,181ms | 13.4% | 99.9% |
| setA-10 | 1000 | 150 | 144,516 | 28,903 | 12,035.8ms | 19,250.6ms | 9,871.9ms | 68,807ms | 17.5% | 99.9% |
| setA-13 | 2000 | 200 | 285,582 | 57,116 | 24,404.8ms | 2,038.3ms | 116,122.9ms | 181,123ms | 13.5% | 99.9% |
| setA-14 | 600 | 250 | 102,340 | 20,468 | 13,760.2ms | 21,316.5ms | 8,535.9ms | 70,594ms | 19.5% | 99.8% |
| setA-16 | 4800 | 250 | 648,120 | 129,624 | 77,569.3ms | 0.0ms | 365,293.9ms | 546,638ms | 14.2% | 100.0% |
| setA-19 | 6000 | 300 | 825,500 | 165,100 | 116,892.2ms | 76,581.8ms | 413,185.8ms | 819,630ms | 14.3% | 100.0% |

### Cache reuse

Cache reuse lower bound (`(calls - unique_targets) / calls`) across the full demand range:

| Instance | Demands | Reuse | Calls/unique target |
|---|---|---|---|
| setA-04 | 200 | 99.8% | 604× |
| setA-06 | 500 | 99.9% | 747× |
| setA-10 | 1000 | 99.9% | 962× |
| setA-13 | 2000 | 99.9% | 1,426× |
| setA-14 | 600 | 99.8% | 408× |
| setA-16 | 4800 | 100.0% | 2,591× |
| setA-19 | 6000 | 100.0% | 2,750× |

**Observed fact:** Cache reuse does not collapse at any measured demand count. Reuse increases with demand count. The cache is being used with increasing efficiency as demand count grows.

**What this does not establish:** High reuse does not by itself prove positive net wall-time benefit. It establishes that the cache is being used effectively. The net benefit question requires a controlled A/B comparison (not done in P10-A, which is measurement-only).

### Dijkstra wall-time share

Dijkstra remains 13–20% of wall time across all instances. It does not grow as a fraction of wall time with demand count.

### Cache memory footprint

| Instance | Nodes | DijkstraResult size | Worst-case cache |
|---|---|---|---|
| setA-04 | 50 | 2,096 bytes | 0.20 MB |
| setA-06 | 100 | 4,096 bytes | 0.78 MB |
| setA-10 | 150 | 6,096 bytes | 1.74 MB |
| setA-13 | 200 | 8,096 bytes | 3.09 MB |
| setA-14 | 250 | 10,096 bytes | 4.81 MB |
| setA-16 | 250 | 10,096 bytes | 4.81 MB |
| setA-19 | 300 | 12,096 bytes | 6.92 MB |

Cache footprint scales with node count (O(N)), not demand count. At 6000 demands, worst-case cache is 6.92 MB.

---

## B. Repair Scaling

### repair_ms vs demand count

| Instance | Demands | repair_ms | repair_ms/wall | improve_ms | improve_ms/wall |
|---|---|---|---|---|---|
| setA-04 | 200 | 355ms | 8.8% | 1,363ms | 34.0% |
| setA-06 | 500 | 5,452ms | 24.6% | 5,937ms | 26.8% |
| setA-10 | 1000 | 9,872ms | 14.3% | 19,251ms | 28.0% |
| setA-13 | 2000 | 116,123ms | 64.1% | 2,038ms | 1.1% |
| setA-14 | 600 | 8,536ms | 12.1% | 21,317ms | 30.2% |
| setA-16 | 4800 | 365,294ms | 66.8% | 0ms | 0.0% |
| setA-19 | 6000 | 413,186ms | 50.4% | 76,582ms | 9.3% |

### Repair scaling rate (non-monotone)

- 200 → 500 demands (2.5× demand): repair_ms +15.4× (355ms → 5,452ms)
- 500 → 1000 demands (2× demand): repair_ms +1.8× (5,452ms → 9,872ms)
- 1000 → 2000 demands (2× demand): repair_ms +11.8× (9,872ms → 116,123ms)
- 2000 → 4800 demands (2.4× demand): repair_ms +3.1× (116,123ms → 365,294ms)

Repair scaling is highly non-linear and non-monotone in its rate. The 200→500 and 1000→2000 transitions show the largest jumps.

### The improve→repair regime shift

At small/medium instances (setA-04, setA-06, setA-10, setA-14): `improve_ms > repair_ms`. The improve operator dominates.

At large instances (setA-13, setA-16): `repair_ms >> improve_ms`. The repair operator dominates.

At setA-16 (4800 demands): `improve_ms = 0.0ms` — the improve operator completed in under 1ms or was skipped. The population is so infeasible that repair consumes the entire generation budget.

At setA-19 (6000 demands): `improve_ms = 76,582ms` — the improve operator is active again, but repair still dominates at 413,186ms.

**The regime shift is not monotone with demand count.** setA-16 shows complete improve collapse while setA-19 shows partial recovery. This may reflect topology differences (setA-16: 250 nodes, 1452 links; setA-19: 300 nodes, 1998 links) rather than demand count alone.

---

## C. Large-Instance Transition

### setA-13 → setA-16 → setA-19

| Instance | Demands | Nodes | Links | repair_ms | improve_ms | valid | best_obj |
|---|---|---|---|---|---|---|---|
| setA-13 | 2000 | 200 | 1000 | 116,123ms | 2,038ms | ✓ | 209.48 |
| setA-16 | 4800 | 250 | 1452 | 365,294ms | 0ms | ✗ | ∞ |
| setA-19 | 6000 | 300 | 1998 | 413,186ms | 76,582ms | ✓ | 159.78 |

The transition is not a simple demand-count threshold. Between setA-13 and setA-16, both demand count and node/link count increase simultaneously. The improve operator collapses completely at setA-16 but partially recovers at setA-19 despite higher demand count.

**The large-instance regression cannot be attributed to demand count alone.** Node count, link count, and topology structure are confounding variables. The campaign-level observation (fewer generations, more wall time per generation) is confirmed, but the causal variable is not yet isolated.

### Trajectory validity

- setA-13: valid=true, n_eval=2 (only 2 evaluations improved best solution in 5 gens)
- setA-16: valid=false, n_eval=0 (population never reached feasibility in 5 gens)
- setA-19: valid=true, n_eval=25

Population feasibility behavior is highly instance-specific and not monotone with demand count.

---

## D. Hypothesis Disposition

### H10-A: Demand-count gate (disable cache when num_demands > THRESHOLD)

**Status: DISFAVORED**

Rationale: Cache reuse remains 99.8–100.0% across all measured instances including 4800 and 6000 demands. Disabling the cache would remove an optimization that is demonstrably being used effectively, while leaving the dominant repair cost untouched. The large-instance regression is not caused by cache reuse collapse or cache overhead being the dominant cost.

### H10-B: Cache size limit with LRU eviction

**Status: NOT JUSTIFIED by current evidence**

Cache size is bounded by N×T (node count × time slots), not demand count. At 6000 demands, worst-case cache is 6.92 MB. There is no evidence of memory pressure causing the regression.

### H10-C: First-generation warm-up only (run-scoped cache)

**Status: NOT EVALUATED — correctness invariant not verified**

This hypothesis requires verifying that `disabled_arcs` does not change across generations. Not investigated in P10-A (measurement-only scope).

### H10-D: Lazy cache (only cache results reused ≥ K times per generation)

**Status: NOT JUSTIFIED by current evidence**

All unique targets are queried in every generation (100% unique target coverage observed). A lazy cache would not reduce overhead.

### Repair-scaling hypothesis (candidate for P10-B)

**Status: CANDIDATE — requires explicit P10-B authorization**

The dominant cost at large instances is `repair_ms`, which scales superlinearly with demand count and eventually overwhelms `improve_ms`. The repair operator is the primary bottleneck for large-instance performance.

Open questions for P10-B investigation:
1. What is the repair operator doing that scales superlinearly with demand count?
2. Is the repair cost driven by the number of infeasible individuals, the cost per repair attempt, or the number of repair iterations?
3. Does the Phase 9 Dijkstra cache interact with repair indirectly (e.g., by changing the distribution of infeasible individuals)?
4. Is the improve→repair regime shift at setA-16 a topology effect or a demand-count effect?

---

## Summary

The original Phase 10 premise — that the Dijkstra cache becomes counterproductive beyond some demand count — is **not supported by the P10-A evidence**. Cache reuse remains near-perfect (99.8–100.0%) at all measured scales from 200 to 6000 demands. The large-instance regression is driven by repair operator scaling, not cache overhead.

**P10-A is complete.** The evidence file is frozen at `evidence/phase10_p10a_crossover_sweep.txt`.

**No implementation work should proceed until P10-B is explicitly authorized.** The repair-scaling hypothesis is the leading candidate for P10-B investigation, but hypothesis selection requires a separate authorization decision after this characterization is reviewed.