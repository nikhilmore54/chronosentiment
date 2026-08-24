# GERAD Phase 9 — P9-A Feasibility Characterization

**Status: COMPLETE — characterization only, no implementation**
**Unblocked by:** Phase 9 scope document (`2dc19de84`)
**Source files read:**
- [`coralys-core/src/operators.rs`](../coralys-core/src/operators.rs) — trait definition
- [`adapters/roadef/src/constraints.rs`](../adapters/roadef/src/constraints.rs) — ROADEF implementation
- [`adapters/roadef/src/ecmp.rs`](../adapters/roadef/src/ecmp.rs) — routing primitives
- [`adapters/roadef/src/moga_impl.rs`](../adapters/roadef/src/moga_impl.rs) — genome, `to_solution()`
- [`adapters/roadef/src/pipeline_impl.rs`](../adapters/roadef/src/pipeline_impl.rs) — call sites

---

## Q1: What does `is_feasible()` compute?

`is_feasible(genome)` is a default trait method in `ConstraintModel<G>`:

```rust
fn is_feasible(&self, candidate: &G) -> bool {
    self.evaluate_violations(candidate).is_empty()
}
```

It delegates entirely to `evaluate_violations`, which for the ROADEF adapter (`RoadefConstraintModel`) performs four sequential constraint stages:

**Stage 0: Solution expansion**
`candidate.to_solution()` — expands the genome's flat `Vec<Vec<u64>>` (one waypoint list per demand) into a `Solution` containing one `SrPath` per (demand, time_slot) pair. This is an O(D × T) allocation where D = number of demands, T = number of time slots.

**Stage 1: Segment limit**
For each `SrPath` in the solution, checks `path.w.len() + 1 > scenario.max_segments`. O(D × T) scan.

**Stage 2: Budget**
For each time slot, computes the total reconfiguration cost (sum of `SrPathBit::dist()` between consecutive time slots for each demand). Checks against `scenario.budget[ts].value`. O(D × T) with `SrPathBit` construction per demand per slot.

**Stage 3 & 4: Routing and Capacity (dominant cost)**
For each time slot `ts`:
1. Builds `disabled_arcs: HashSet<u64>` from `scenario.interventions` — O(|interventions|)
2. Initialises `arc_flows: HashMap<u64, f64>` over all arcs — O(|arcs|)
3. For each demand with positive flow at `ts`:
   - Calls `expand_sr_path(graph, src, dst, waypoints, disabled_arcs, flow, arc_flows)`
   - `expand_sr_path` calls `backward_dijkstra(graph, waypoint_i, disabled_arcs)` once per segment (waypoints.len() + 1 times)
   - Each `backward_dijkstra` is a full O((V + E) log V) Dijkstra on the reversed graph
   - Then calls `route_ecmp` which sorts all reachable nodes by distance — O(V log V)
4. Checks all arcs for capacity violation: `flow / capacity >= 1.0 - 1e-6` — O(|arcs|)

**Total complexity per `is_feasible()` call:**
O(T × D × (W+1) × (V + E) log V)
where W = average waypoints per demand (typically 0–1 in this baseline).

The result is a `Vec<RoadefViolation>` which is immediately discarded — only `.is_empty()` is checked.

---

## Q2: Why is it called once per offspring?

`is_feasible()` is called in `pipeline_impl.rs` at two call sites:

**Call site 1 (line 1510):** After crossover + mutation, before `process_offspring`:
```rust
let was_feasible = pipeline_obj.constraint_model.is_feasible(&child);
// ... then:
pipeline_obj.process_offspring(&mut child, was_feasible, ...)
```

**Call site 2 (line 1578):** After mutation-only, before `process_offspring`:
```rust
let was_feasible = pipeline_obj.constraint_model.is_feasible(&child);
// ... then:
pipeline_obj.process_offspring(&mut child, was_feasible, ...)
```

The `was_feasible` boolean is passed to `process_offspring` to determine whether to call the **repair operator** (infeasible → repair) or the **improve operator** (feasible → improve). This is the sole use of the result.

---

## Q3: Which parts of its computation are repeated across offspring in the same generation?

**Genome-independent (same for all offspring in a generation):**
- `disabled_arcs` per time slot — determined solely by `scenario.interventions[ts]`, which is fixed for the entire run
- `scenario.max_segments` — fixed
- `scenario.budget` — fixed
- The graph topology (`graph.arcs`, `graph.in_arcs`, `graph.nodes`) — fixed

**Genome-dependent (must be recomputed per offspring):**
- `candidate.to_solution()` — depends on genome waypoints
- `arc_flows` — depends on which demands have which waypoints
- The Dijkstra results — depend on `disabled_arcs` (fixed per slot) AND the waypoint sequence (genome-dependent)

**Critical observation:** `backward_dijkstra(graph, target, disabled_arcs)` depends only on `target` and `disabled_arcs` — **not on the genome**. For a given time slot, the Dijkstra result for a given target node is identical for every offspring that routes a demand to that target. The L2 Dijkstra cache (Phase 6) already exploits this for the fitness evaluator. The constraint model does **not** use the L2 cache.

---

## Q4: Can its result be derived incrementally?

**Partial yes — with important caveats.**

The feasibility result depends on:
1. Segment limit — depends only on waypoint count, not routing. Cheap to check incrementally.
2. Budget — depends on waypoint changes between time slots. Cheap to check incrementally if the diff is known.
3. Routing (connectivity) — depends on whether `expand_sr_path` succeeds for each demand. This is the expensive part. It is NOT easily incremental because a single waypoint change can reroute flow across many arcs, affecting capacity for other demands.
4. Capacity — depends on the full arc flow map, which is a global property of all demands' routes.

**The key structural observation:**
The constraint model recomputes the full arc flow map from scratch for every offspring. This is necessary for capacity checking (Stage 4) but is also the dominant cost. The connectivity check (Stage 3) is a byproduct of the flow computation.

**Incremental feasibility is not straightforward** because capacity is a global constraint — changing one demand's waypoints affects arc flows for all demands sharing those arcs.

---

## Q5: What invariants must an optimization preserve?

1. **Feasibility classification must be bit-exact**: `was_feasible` must be identical to the current implementation for every offspring. Any optimization that changes the classification for even one offspring will alter the trajectory (repair vs improve decision changes → different genome → different population → different best_obj).

2. **Trajectory invariants (5/5)**: `best_obj`, `n_actual_evals`, `generations_run`, `valid`, `cache_hits` must remain bit-exact vs Phase 8 baseline.

3. **No change to constraint semantics**: The four constraint stages (segment limit, budget, connectivity, capacity) must produce the same violation set for every genome.

4. **No change to `evaluate_violations` output**: Even if `is_feasible()` is optimized, `evaluate_violations` is the authoritative constraint check. Any optimization must be consistent with it.

---

## Q6: What constitutes a meaningful speedup?

- **Minimum threshold**: T_net > 0 on setA-01 (paired contemporaneous control, same seed)
- **Corroboration**: T_net > 0 on setA-14
- **Attribution**: `feasibility_ms` reduction ≥ 50% of measured T_net
- **Magnitude**: Given `feasibility_ms` = 3,183ms (setA-01) and 848,013ms (setA-14), a 50% reduction would save ~1,600ms and ~424,000ms respectively — material on both instances

---

## Q7: What solution-quality regression is unacceptable?

- `best_obj` must not increase (worse solution) on either instance
- `valid` must remain true on both instances
- Any regression on setA-14 (larger instance) is disqualifying

---

## Hypothesis evaluation (from Phase 9 scope H1–H4)

### H1: Memoization / caching of feasibility results

**Assessment: VIABLE — with important constraints.**

The genome is `Hash + Eq` (derived). A `HashMap<RoadefGenome, bool>` keyed by genome could cache `is_feasible()` results. However:
- The genome hash includes all waypoints for all demands — hashing is O(D × W) per lookup
- Cache hits only occur when the same genome appears multiple times in the same generation (e.g., after crossover produces a child identical to an existing population member)
- The L1 Dijkstra cache already handles the case where the same genome is evaluated twice (cache_hits in Phase 8 data: 250/1802 = 13.9% on setA-01, 181/2006 = 9.0% on setA-14)
- A feasibility cache would only help if the same genome is both evaluated AND feasibility-checked — these are separate code paths

**Verdict:** Low expected hit rate. The genome is mutated before feasibility check, so the same genome rarely appears twice in the feasibility check path. **Not the primary candidate.**

### H2: Incremental feasibility check

**Assessment: NOT VIABLE in the current architecture.**

Capacity is a global constraint. A single waypoint change can reroute flow across many arcs, invalidating the capacity check for all demands sharing those arcs. Incremental checking would require tracking which arcs are affected by each demand's route — a significant architectural change that risks introducing bugs.

**Verdict:** Too complex and risky for a single-intervention experiment. **Reject.**

### H3: Lazy feasibility — defer to process_offspring

**Assessment: VIABLE — this is the strongest candidate.**

`process_offspring` already determines feasibility internally as part of repair/improve. The repair operator calls `evaluate_violations` to find violations and fix them. The improve operator is only called on feasible genomes (by contract). Therefore:

- If `process_offspring` is called without the `was_feasible` hint, it could determine feasibility itself as a byproduct of its first `evaluate_violations` call
- This would eliminate the standalone `is_feasible()` call entirely
- The cost would be zero additional `evaluate_violations` calls — the repair operator already calls it

**However:** This requires reading `process_offspring` to confirm it calls `evaluate_violations` internally. If it does, the `is_feasible()` call is redundant and can be eliminated. If it does not, this hypothesis is invalid.

**Verdict:** Requires reading `process_offspring` implementation. **Primary candidate for investigation.**

### H4: Cheaper feasibility proxy

**Assessment: VIABLE but complex.**

A necessary condition for infeasibility could be checked cheaply (e.g., segment count only — O(D) vs O(D × T × V log V)). If the proxy says "definitely feasible" (no segment violations), skip the full check. If inconclusive, fall back to full `is_feasible()`.

**However:** The dominant cost is capacity checking (Stage 4), not segment checking (Stage 1). A segment-only proxy would only help for genomes that violate segment limits — which may be rare in practice (the genome is constructed to respect `max_segments`).

**Verdict:** Proxy would need to be capacity-related to be effective. Capacity is the expensive part and cannot be cheaply approximated without routing. **Secondary candidate.**

---

## Recommended next action

**Read `process_offspring` in `pipeline_impl.rs`** to determine whether it calls `evaluate_violations` internally. If it does, H3 (lazy feasibility) is the primary optimization candidate: eliminate the standalone `is_feasible()` call and let `process_offspring` determine feasibility as a byproduct.

This is a single-line change (remove the `is_feasible()` call, pass a sentinel to `process_offspring`) with a clear, testable hypothesis and a well-defined gate.