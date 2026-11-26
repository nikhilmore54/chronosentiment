# Phase 9 H6-revised: Dijkstra Cache Precondition Characterization

**Commit:** 8eb63eb1e  
**Date:** 2026-08-25  
**Status:** CHARACTERIZATION COMPLETE — PROMOTED TO IMPLEMENTATION CANDIDATE

---

## Background

After H3 (redundant `is_feasible()` elimination) was promoted, Phase 9 P9-B turned to the
`improve` path. Within `improve`, the dominant cost is `backward_dijkstra()` inside
`expand_sr_path()` → `evaluate_violations()`.

H6-revised proposes a per-generation memoization cache keyed on `(target_node_id, time_slot)`.
The correctness basis is established: the Dijkstra result for a given `(target, slot)` pair
is genome-independent — it depends only on graph topology and disabled arcs for that slot,
not the candidate solution. The cache is invalidated at generation boundaries.

---

## Measurement Results

### setA-01 (nodes=20, time_slots=2, demands=40)

| Metric | Value |
|---|---|
| Total `backward_dijkstra()` calls | 233,216 |
| Calls per generation (avg) | 4,664.3 |
| dijkstra_ms / improve_ms | **60.3%** |
| dijkstra_ms / wall_ms | **40.7%** |
| Max unique (target,slot)/gen | 40 (N=20 × T=2) |
| Observed unique target IDs (run) | 20 (all N=20 nodes) |
| Observed reuse lower bound | **100.0%** |
| Calls / (max_unique × gens) | 116.61× |
| Calls saved per unique target | 11,659 |
| Worst-case cache size | 35,840 bytes (0.03 MB) |

### setA-14 (nodes=250, time_slots=2, demands=600)

| Metric | Value |
|---|---|
| Total `backward_dijkstra()` calls | 3,647,518 |
| Calls per generation (avg) | 72,950.4 |
| dijkstra_ms / improve_ms | **68.7%** |
| dijkstra_ms / wall_ms | **51.3%** |
| Max unique (target,slot)/gen | 500 (N=250 × T=2) |
| Observed unique target IDs (run) | 250 (all N=250 nodes) |
| Observed reuse lower bound | **100.0%** |
| Calls / (max_unique × gens) | 145.90× |
| Calls saved per unique target | 14,589 |
| Worst-case cache size | 5,048,000 bytes (4.81 MB) |

---

## Key Findings

**1. Call multiplicity is extreme.** On setA-14, 3,647,518 Dijkstra calls are made against
a key space of at most 500 unique `(target, slot)` pairs per generation — approximately
146 executions per maximum cache state. This is the strongest possible case for memoization.

**2. Dijkstra dominates the improve path.** At 68.7% of `improve_ms` on setA-14 and 51.3%
of total wall time, Dijkstra is the single largest computational cost in the system post-H3.
This is not a micro-optimization — it attacks more than half of total runtime.

**3. All nodes are observed.** Both instances use every node as a Dijkstra target across the
run. The observed reuse lower bound is 100.0% on both instances.

**4. Memory is bounded.** setA-01 worst-case is 0.03 MB (trivial). setA-14 worst-case is
4.81 MB — acceptable for a per-generation cache discarded at generation boundaries.

**5. Remaining characterization gap.** The current measurement tracks unique target IDs,
not unique `(target, slot)` pairs. The true cache key is `(target_node_id, time_slot)`.
For T=2, the actual unique key count is at most 2× the observed unique target count.
This gap does not change the promotion decision given the extreme call multiplicity.

---

## Architectural Decision

The cache belongs in the ROADEF adapter (`adapters/roadef/`), not in `coralys-core`:

```
Coralys Core
    │
    │  generic process_offspring()
    │  generic feasibility contract
    ▼
ROADEF Adapter
    │
    ├── evaluate_violations()
    ├── expand_sr_path()
    ├── backward_dijkstra()
    └── generation-scoped Dijkstra cache  ← H6-revised
```

H3 was a genuine core-level redundancy. H6-revised is an adapter-local implementation
optimization. These are distinct interventions at distinct levels.

**Cache value type: `Arc<DijkstraResult>`**

`DijkstraResult` contains two `HashMap`s (`dist: HashMap<u64,f64>` and
`preds: HashMap<u64,Vec<usize>>`). Cloning is O(N) per cache hit. On setA-14 with N=250,
this would replace expensive Dijkstra calls with expensive HashMap clones — undermining the
optimization. Using `Arc<DijkstraResult>` gives O(1) pointer increment on cache hit.

**Cache key: `(target_node_id: u64, time_slot: usize)`**

**Cache scope: per-generation** — invalidated at generation boundaries since `disabled_arcs`
may change between generations.

---

## Gate Protocol (same as H3)

1. 5/5 trajectory invariants bit-exact (best_obj, n_eval, cache_hits, valid, generations)
2. T_net > 0 (wall_baseline − wall_experimental > 0)
3. Dijkstra call count reduced (confirms cache is being used)
4. Routing semantics unchanged (bit-exact trajectory invariants confirm this)

---

## Governance

- This is a **measurement-only characterization commit**. No production code changed.
- H6-original (+7.3% on setA-01) and H6-revised are independent mechanisms.
- H6-original's result is not evidence for H6-revised's economics.
- Intervention authorized. Next step: implement cache in ROADEF adapter, gate on setA-01,
  corroborate on setA-14.