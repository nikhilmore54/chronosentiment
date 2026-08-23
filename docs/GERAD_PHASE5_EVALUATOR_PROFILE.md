# GERAD Phase 5 — Evaluator Decomposition Profile

**Date:** 2026-08-23
**Author:** Governance-hardening session
**Status:** OBSERVATIONAL — no code changes
**Phase 4 baseline:** `4a691cdd2` (setA-14 Arm B: wall=1,739,745ms, eval=185,159ms)

---

## 1. Governance Context

Phase 4 (L1 + Rayon combined baseline) is closed. The Phase 4 measurement on setA-14:

| Metric | Value |
|---|---|
| Wall-clock | 1,739,745ms (29.0 min) |
| Eval time | 185,159ms (10.6% of wall) |
| Cache hits (L1) | 181 |
| Actual evaluations | 2,006 |
| Eval speedup vs Phase 2 | 5.72× |

The remaining 89.4% of wall-clock is sequential overhead: selection, crossover, mutation, repair, L1 cache lookup, merge. Phase 5 investigates whether any evaluator-internal computation can be cached across evaluations (L2 component cache) to further reduce the 185,159ms eval budget.

**Zero objective/trajectory delta is mandatory.** Speedup alone is insufficient for promotion.

---

## 2. Evaluator Call Graph

```
RoadefFitnessEvaluator::evaluate(genome)          [pipeline_impl.rs]
  └─ genome → Solution conversion
  └─ RoadefEvaluator::evaluate_solution_cached(solution)   [evaluator.rs:541]
       │
       ├─ Stage 1: segment_check                  O(srpaths)
       │    └─ path.w.len() + 1 > max_segments?
       │
       ├─ Stage 2: budget_check                   O(demands × time_slots)
       │    └─ SrPathBit::new_explicit()
       │    └─ SrPathBit::dist()
       │
       ├─ Stage 3: routing                        DOMINANT COST
       │    ├─ disabled_arcs from scenario.interventions (genome-independent)
       │    ├─ Per-time-slot Dijkstra cache: HashMap<target_node, DijkstraResult>
       │    │    └─ backward_dijkstra(graph, target_node, disabled_arcs)  [ecmp.rs]
       │    └─ route_ecmp(graph, dijkstra_result, src, tgt, flow, arc_flows)
       │
       └─ Stage 4: objective                      O(arcs)
            └─ MLU + inv_load_cost per arc
```

---

## 3. EvalTimings Component Breakdown

The evaluator already instruments all four stages via `EvalTimings` (M20 Phase 1/2). The fields are:

| Field | Measures | Scope |
|---|---|---|
| `segment_check_ns` | Stage 1 | Per evaluation |
| `budget_check_ns` | Stage 2 | Per evaluation |
| `routing_ns` | Stage 3 total | Per evaluation |
| `dijkstra_ns` | Stage 3 → Dijkstra only | Per evaluation |
| `ecmp_ns` | Stage 3 → ECMP only | Per evaluation |
| `objective_ns` | Stage 4 | Per evaluation |
| `dijkstra_cache_hits` | Within-eval Dijkstra cache hits | Per evaluation |
| `dijkstra_cache_misses` | Within-eval Dijkstra cache misses | Per evaluation |

**The within-evaluation Dijkstra cache** (`evaluate_solution_cached`) already eliminates redundant Dijkstra calls for demands sharing the same target node within a single time slot. This is the existing L2-within-eval optimization.

---

## 4. Component Analysis for L2 Cross-Evaluation Caching

### 4.1 Stage 1: segment_check

| Property | Value |
|---|---|
| Cost | Trivial — O(srpaths), nanoseconds |
| Repeated across genomes? | Yes, but cost is negligible |
| Cache key | N/A |
| L2 candidate? | **NO** — cost too low to justify cache overhead |

### 4.2 Stage 2: budget_check

| Property | Value |
|---|---|
| Cost | Low — O(demands × time_slots), SrPathBit construction + Hamming distance |
| Repeated across genomes? | Only if two genomes have identical waypoints for all demands at all time slots |
| Cache key | Full genome waypoints (same as L1 key) |
| L2 candidate? | **NO** — if the genome is identical, L1 already catches it. If different, budget_check must rerun. |

### 4.3 Stage 3: routing — backward_dijkstra

| Property | Value |
|---|---|
| Cost | **DOMINANT** — O(nodes × log(nodes)) per call |
| Input | `(graph, target_node, disabled_arcs)` |
| `graph` | Immutable — same for all evaluations |
| `disabled_arcs` | Determined solely by `scenario.interventions[time_slot]` — **genome-independent** |
| `target_node` | Determined by demand destination + waypoints |
| Repeated across genomes? | **YES** — if two genomes route any demand to the same target node at the same time slot, the Dijkstra result is identical |
| Cache key | `(target_node, time_slot)` |
| Cache scope | Cross-evaluation (run-level) |
| Deterministic? | **YES** — `backward_dijkstra` is deterministic given `(graph, target_node, disabled_arcs)` |
| Numerical risk | **NONE** — Dijkstra result is exact; no floating-point accumulation |
| Reusable across genomes? | **YES** — `disabled_arcs` is genome-independent |
| L2 candidate? | **YES — PRIMARY CANDIDATE** |

### 4.4 Stage 3: routing — route_ecmp

| Property | Value |
|---|---|
| Cost | Moderate — O(path_length × arcs) |
| Input | `(graph, dijkstra_result, src, tgt, flow, arc_flows)` |
| `flow` | Demand volume at time slot — genome-independent (from TrafficMatrix) |
| `arc_flows` | Accumulated across demands — **genome-dependent** (order matters) |
| Repeated across genomes? | Partially — `(src, tgt, flow)` may repeat, but `arc_flows` accumulation is order-dependent |
| Cache key | Cannot cache output (arc_flows is mutable accumulator) |
| L2 candidate? | **NO** — arc_flows accumulation is stateful and order-dependent |

### 4.5 Stage 4: objective

| Property | Value |
|---|---|
| Cost | Trivial — O(arcs), simple arithmetic |
| L2 candidate? | **NO** — cost too low; depends on arc_flows which is genome-dependent |

---

## 5. L2 Primary Candidate: Cross-Evaluation Dijkstra Cache

### Design

A run-level `HashMap<(u64, usize), DijkstraResult>` keyed by `(target_node, time_slot)`.

- Populated on first miss; served from cache on subsequent evaluations
- Cache is shared across all evaluations in a run (not per-generation)
- Thread-safe: `Arc<RwLock<HashMap<...>>>` or `Arc<DashMap<...>>` for Rayon compatibility
- No eviction (run-local, bounded by `num_unique_target_nodes × num_time_slots`)

### Cache key validity

`disabled_arcs` is determined solely by `scenario.interventions[time_slot]` — it is:
- Genome-independent ✅
- Time-slot-specific ✅
- Immutable for the duration of a run ✅

Therefore `(target_node, time_slot)` is a valid, safe cache key.

### Theoretical savings

On setA-14:
- `n_actual_evals` = 2,006 (Phase 4 baseline)
- Each evaluation calls `backward_dijkstra` once per unique `(target_node, time_slot)` pair
- If the network has N unique target nodes and T time slots, the maximum unique Dijkstra calls per run = N × T
- After the first evaluation, all subsequent evaluations for the same `(target_node, time_slot)` are cache hits
- **Upper bound:** if all 2,006 evaluations share the same target nodes, Dijkstra is called only N × T times total instead of 2,006 × N × T times

### Risks

| Risk | Assessment |
|---|---|
| Numerical correctness | **NONE** — Dijkstra is deterministic; cached result is bit-identical |
| Trajectory change | **NONE** — objective values are identical; search trajectory preserved |
| Memory | Bounded: `num_unique_targets × num_time_slots × sizeof(DijkstraResult)` |
| Thread safety | Requires `Arc<RwLock<...>>` or `DashMap` for Rayon compatibility |
| Cache poisoning | **NONE** — key is `(target_node, time_slot)`, both genome-independent |

---

## 6. GO / NO-GO Decision Table

| Component | L2 Candidate? | Decision | Rationale |
|---|---|---|---|
| segment_check | NO | **NO-GO** | Trivial cost |
| budget_check | NO | **NO-GO** | Genome-dependent; L1 already handles identical genomes |
| backward_dijkstra | **YES** | **GO** | Dominant cost, genome-independent key, deterministic, safe |
| route_ecmp | NO | **NO-GO** | arc_flows accumulation is stateful and order-dependent |
| objective | NO | **NO-GO** | Trivial cost |

---

## 7. Phase 6 Specification

**Single candidate:** Cross-evaluation Dijkstra cache keyed by `(target_node, time_slot)`.

**Implementation location:** `RoadefFitnessEvaluator` in `adapters/roadef/src/fitness_eval.rs` (or a new `evaluate_with_l2_cache` method on `RoadefEvaluator`).

**Verification protocol:**
- Same seed, same population, same generations, same genome trajectory
- Same `best_obj`, `n_actual_evals`, `generations_run`, `valid`, `cache_hits` (L1)
- Measure: `eval_time_ms` reduction vs Phase 4 baseline (185,159ms)
- Promotion criterion: `T_net > 0` (net evaluator savings after cache overhead)

**Phase 4 baseline (authoritative):**
```
wall_clock_ms : 1,739,745
eval_time_ms  : 185,159
cache_hits    : 181
n_actual_evals: 2,006
best_obj      : 86.1250850504
valid         : true
generations   : 50
seed          : 42
instance      : setA-14
commit        : 4a691cdd2
```

---

## 8. What Phase 6 Must NOT Do

- Must not change `evaluate_solution()` semantics
- Must not alter genome representation, objective, termination, selection, crossover, mutation
- Must not introduce eviction (run-local cache, no eviction needed)
- Must not use the GERAD performance evidence to establish M1 governance claims