# RP-310 Design Specification — Redundant Shortest-Path Elimination

**Status:** DRAFT  
**Milestone:** M20  
**Date:** 2026-07-11  
**Evidence base:** M20 Phase 1 Evaluator Performance Model  
**Constitutional authority:** H4-RESEARCH-CONSTITUTION-v1.0.md  
**Baseline:** BASELINE-v1.0.json (frozen M19.5)

---

## 1. Problem Statement

### 1.1 Evidence from M19 (O-004, O-006)

The M19 campaign established two observations that motivate RP-310:

**O-004** — Evaluation cost grows rapidly with routing workload. Across the 20-instance
benchmark suite, evaluation cost increased by approximately 890× between setA-01 and
setA-17. The observed scaling is strongly correlated with demands × links. This is an
empirical observation; it does not constitute a formal complexity proof.

**O-006** — Evaluation throughput is the primary constraint on optimization progress.
15 of 20 instances are evaluation-limited (SearchLimited=2, EvaluationLimited=15,
Infeasible=3). The optimizer is rarely the limiting factor.

### 1.2 Engineering motivation

The constitutional exit criterion for RP-310 is:

> ≥2× reduction in ms/gen on setA-10 with no degradation in objective quality or
> feasibility rate.

M19 established that ms/gen scales with demands × links. To achieve ≥2× reduction,
the evaluator must perform substantially less work per evaluation.

### 1.3 M20 mission

> Reduce evaluator cost while preserving identical optimization semantics.

This is not "improve solutions." It is "produce exactly the same solutions using less
computation."

---

## 2. Evaluator Performance Model (M20 Phase 1)

Collected by `eval_profiler` binary, 20 runs each, empty solution, release build.
One warm-up run discarded. All timings are averages over 20 runs.

### 2.1 Per-stage breakdown

| Stage          | setA-04 (200d, 250l) | %     | setA-10 (1000d, 966l) | %     |
|----------------|---------------------:|------:|----------------------:|------:|
| segment_check  | <1 µs                | 0.0%  | <1 µs                 | 0.0%  |
| budget_check   | 33 µs                | 0.2%  | 201 µs                | 0.1%  |
| routing        | 14,454 µs            | 99.7% | 223,060 µs            | 99.9% |
| objective      | 16 µs                | 0.1%  | 55 µs                 | 0.0%  |
| **TOTAL**      | **14,503 µs**        |       | **223,316 µs**        |       |

### 2.2 Routing scaling indicators

| Metric                | setA-04 | setA-10 |
|-----------------------|--------:|--------:|
| dijkstra_calls        | 392     | 2,000   |
| dijkstra_µs/call      | 36.87   | 111.53  |
| routing_µs/(d×l)      | 0.289   | 0.231   |

### 2.3 Formal observations (promoted from M20 Phase 1)

**O-007 — Routing overwhelmingly dominates evaluator cost.**
Routing consumes 99.7% (setA-04) and 99.9% (setA-10) of evaluation time. No other
stage can contribute meaningfully to a ≥2× speedup. This is an empirical observation
from the M20 Phase 1 profile; it does not constitute a formal complexity proof.

**O-008 — Objective computation is effectively free.**
Objective computation (saturation, MLU, Jain, inv-load-cost) consumes 0.0–0.1% of
evaluation time (55 µs vs 223,060 µs routing on setA-10). Caching objective values,
micro-optimizing penalties, or optimizing constraint accumulation cannot realistically
achieve the constitutional ≥2× target. This is an important negative result: it
prevents engineering effort being spent in the wrong place.

**O-009 — Evaluation cost is proportional to Dijkstra call count, not Dijkstra speed.**
Dijkstra calls scale exactly with demands × time_slots (392 for setA-04, 2,000 for
setA-10). The per-call cost grows with graph size (36.87 µs at 250 links, 111.53 µs
at 966 links), consistent with O(links × log nodes). The engineering target is
therefore to **execute fewer Dijkstra calls**, not to make individual calls faster.
Those are fundamentally different strategies.

---

## 3. Current Evaluator Pipeline

### 3.1 Call graph

```
evaluate_solution(genome)
  │
  ├── segment_check          O(srpaths)                    <1 µs
  │
  └── for each time_slot t:
        │
        ├── budget_check     O(demands)                    33–201 µs total
        │
        └── compute_loads(t, solution)
              │
              └── for each demand d with flow > 0:
                    │
                    └── expand_sr_path(graph, s, target, waypoints, disabled_arcs, flow, arc_flows)
                          │
                          └── for each segment (waypoints.len() + 1):
                                │
                                ├── backward_dijkstra(graph, segment_target, disabled_arcs)
                                │     → DijkstraResult { dist, preds }
                                │
                                └── route_ecmp(graph, dijkstra_result, source, segment_target, flow, arc_flows)
```

### 3.2 Cost centres

| Function            | File         | Cost share | Scaling              |
|---------------------|--------------|-----------|----------------------|
| `backward_dijkstra` | ecmp.rs      | dominant  | O(links log nodes) per call |
| `route_ecmp`        | ecmp.rs      | secondary | O(nodes) per call    |
| `budget_check`      | evaluator.rs | 0.1–0.2%  | O(demands)           |
| `objective`         | evaluator.rs | 0.0–0.1%  | O(links)             |
| `segment_check`     | evaluator.rs | 0.0%      | O(srpaths)           |

Note: the routing stage (99.7–99.9%) contains both `backward_dijkstra` and `route_ecmp`.
The cache eliminates only `backward_dijkstra` calls. The proportion of routing time
attributable to each function will be measured in Phase 2 instrumentation.

### 3.3 Redundancy analysis

In the current implementation, `backward_dijkstra(graph, target_node, disabled_arcs)` is
called once per demand per segment per time slot. However:

- Multiple demands may share the same `target_node` within a time slot.
- The `disabled_arcs` set is determined entirely by `self.scenario.interventions` for
  time slot `t` — scenario data loaded at construction time, never modified by the genome.
  (See Section 4.1 for the formal proof of this property.)
- Therefore, `backward_dijkstra(graph, target_node, disabled_arcs)` produces the same
  result for every demand with the same destination in the same time slot.

**Current behaviour:** D demands with K distinct destinations → D Dijkstra calls.
**Optimal behaviour:** D demands with K distinct destinations → K Dijkstra calls.

The optimization target is: **reduce D to K.**

---

## 4. Correctness Contracts

### 4.1 Cache key validity proof

**Claim:** The cache key `(target_node, time_slot)` is sufficient to uniquely identify
a Dijkstra result that is valid for all demands in that time slot.

**Proof:**

The inputs to `backward_dijkstra(graph, target_node, disabled_arcs)` are:
1. `graph` — the `Digraph` built from `Network` at `RoadefEvaluator::new()`. Immutable
   for the lifetime of the evaluator. Not affected by the genome.
2. `target_node` — the destination node for this segment. Determined by the demand's
   target field or waypoint. Part of the cache key.
3. `disabled_arcs` — built from `self.scenario.interventions.iter().find(|i| i.t == t)`.
   The scenario is loaded at construction time and is immutable. It depends only on `t`,
   not on the genome, the population, or any evolutionary operator.

Therefore, for a fixed `(target_node, time_slot)`, the result of `backward_dijkstra`
is identical for all demands evaluated in that time slot, regardless of genome content.
The simplified cache key `(target_node, time_slot)` is provably sufficient.

**Cache validity contract:**

| Field                  | Value                                                    |
|------------------------|----------------------------------------------------------|
| Cache key              | `(target_node: u64, time_slot: usize)`                   |
| Invalidation condition | Never within a single `evaluate_solution()` call         |
| Lifetime               | Single evaluation call (created and dropped per call)    |
| Semantic guarantee     | Identical `DijkstraResult` to an uncached call           |
| Genome dependence      | None — cache key is independent of genome content        |

### 4.2 A-001 (existing, M19)

> `valid == true ⇒ obj.is_finite()`

Established in M19. Must be preserved by RP-310. The caching optimization does not
touch the validity or objective computation paths, so A-001 is not at risk.

### 4.3 E-001 (new, M20)

> For every genome: `full_evaluator(genome) == incremental_evaluator(genome)`

This is the primary correctness contract for RP-310.

**Operational acceptance protocol:**

For every genome evaluated during the validation campaign, compare all of the following:

| Field           | Comparison method                          |
|-----------------|--------------------------------------------|
| `result.valid`  | Exact equality                             |
| `result.obj`    | `|full - cached| < 1e-9`                   |
| Per-link flows  | `|full[arc] - cached[arc]| < 1e-9` for all arcs |
| MLU             | `|full_mlu - cached_mlu| < 1e-9`           |

If any field mismatches: increment `mismatch_counter`, emit diagnostic with genome
fingerprint and mismatched fields, fall back to full evaluator.

Production cache replaces full evaluator only when `mismatch_counter == 0` over the
agreed validation campaign (full 20-instance benchmark suite + ≥10,000 randomized
genomes per instance).

Comparing per-link flows (not just the final objective) makes it much easier to
localize any discrepancy if one appears.

### 4.4 G-001 (new, M20)

> Every accepted performance claim shall be benchmarked against the frozen
> `BASELINE-v1.0.json` using the same measurement model, protocol, and success metrics.

The RP-310 delta report must use the same campaign runner, same 20 instances, same
budgets, and same measurement model as the M19 campaign.

---

## 5. Incremental Evaluation Strategy

### 5.1 Core hypothesis

Within a single call to `evaluate_solution()`, the `disabled_arcs` set for each time
slot is fixed (determined by the scenario, not the genome). Therefore, the result of
`backward_dijkstra(graph, target_node, disabled_arcs)` is identical for all demands
sharing the same `target_node` in the same time slot.

Caching this result eliminates redundant Dijkstra calls.

### 5.2 Cache structure

```rust
// Per-evaluation Dijkstra cache, one per time slot.
// Key: target_node (u64)
// Value: DijkstraResult for (target_node, disabled_arcs[time_slot])
type DijkstraCache = HashMap<u64, DijkstraResult>;
```

Populated lazily: the first demand with a given destination triggers the Dijkstra call;
subsequent demands reuse the cached result. The cache is created at the start of each
`evaluate_solution_cached()` call and dropped at the end.

### 5.3 Expected cache hit rate

For the empty solution (all demands use default ECMP, no waypoints):
- Each demand has a unique (source, target) pair.
- Multiple demands may share the same target.
- Cache hit rate = 1 - (distinct_targets / total_demands).

The actual hit rate will be measured by the `cache_hit_rate` counter added in Phase 2.
This is the leading indicator of RP-310 effectiveness.

### 5.4 Waypoint handling

With waypoints, `expand_sr_path` calls `backward_dijkstra` once per segment
(waypoints.len() + 1 times per demand). Each segment's destination is a valid cache
key. Intermediate waypoints are also cacheable.

---

## 6. Amdahl's Law Analysis

Given routing = 99.9% of evaluation time (setA-10):

| Routing reduction | Overall speedup |
|------------------:|----------------:|
| 10%               | 1.11×           |
| 25%               | 1.33×           |
| 50%               | **2.00×**       |
| 75%               | 4.00×           |
| 90%               | 10.0×           |

**Important caveat:** The routing stage (99.9%) contains both `backward_dijkstra` and
`route_ecmp`. The cache eliminates only `backward_dijkstra` calls. The actual speedup
depends on the proportion of routing time attributable specifically to `backward_dijkstra`.

If, for example, `backward_dijkstra` accounts for 88% of routing time and `route_ecmp`
accounts for 12%, then a 50% cache hit rate produces approximately 44% routing reduction,
not 50%. The constitutional goal may still be achievable, but:

> **Cache hit rate is a leading indicator of RP-310 effectiveness, not a direct predictor
> of overall speedup. The actual speedup depends on the measured proportion of routing
> time attributable to `backward_dijkstra()` specifically.**

The Phase 2 instrumentation will measure this split explicitly.

---

## 7. Correctness Preservation Matrix

| Optimization              | Affected component       | Correctness risk | Validation method                          |
|---------------------------|--------------------------|------------------|--------------------------------------------|
| Dijkstra result caching   | ecmp.rs / evaluator.rs   | Medium           | E-001 differential testing on full suite   |
| Per-eval cache scope      | evaluator.rs             | Low              | Cache cleared between evaluations          |
| Waypoint segment caching  | ecmp.rs                  | Medium           | E-001 differential testing with waypoints  |
| Instrumentation           | evaluator.rs             | None             | Unit tests; no logic changes               |

---

## 8. Implementation Plan

### Phase 2 — Proof of Correctness (dual-path)

**Objective:** Demonstrate E-001 before any production change.

1. Implement `evaluate_solution_cached()` alongside the existing `evaluate_solution()`.
2. Add `dijkstra_cache_hits` and `dijkstra_cache_misses` counters to `EvalTimings`.
3. Add sub-stage timers within routing to split `backward_dijkstra` vs `route_ecmp`.
4. In a validation harness, run both evaluators on every genome during a campaign run.
5. Assert E-001 across all observable outputs (valid, obj, per-link flows, MLU).

**Exit evidence:** Zero E-001 violations across the full 20-instance benchmark suite
and ≥10,000 randomized genomes per instance.

### Phase 3 — Performance (cached path active)

**Objective:** Replace the full evaluator with the cached path.

1. Make `evaluate_solution_cached()` the production path.
2. Retain the full evaluator in debug/validation builds with E-001 assertion.
3. Run the full 20-instance campaign (Campaign v4).

**Exit evidence:** Zero fallback events during sustained testing.

### Phase 4 — Validation (remove fallback)

**Objective:** Confirm RP-310 exit criterion.

1. Remove the dual-path comparison from production builds.
2. Run Campaign v4 with the same protocol as M19.
3. Generate Delta Report against `BASELINE-v1.0.json`.

**Exit evidence:** ≥2× ms/gen reduction on setA-10; no feasibility or objective
regression; A-001 preserved; G-001 satisfied.

---

## 9. Measurement Plan

### 9.1 New metrics (Phase 2)

Add to `EvalTimings`:

```rust
pub dijkstra_cache_hits: u64,
pub dijkstra_cache_misses: u64,
pub dijkstra_ns: u64,    // time inside backward_dijkstra only
pub ecmp_ns: u64,        // time inside route_ecmp only
```

Report:

```
cache_hit_rate = dijkstra_cache_hits / (dijkstra_cache_hits + dijkstra_cache_misses)
dijkstra_fraction = dijkstra_ns / routing_ns   (proportion of routing in Dijkstra)
```

`cache_hit_rate` is the primary RP-310 leading indicator.
`dijkstra_fraction` determines the maximum achievable speedup from caching.

### 9.2 Campaign v4 metrics

Same measurement model as M19 campaign:
- `ms_per_generation` (primary KPI)
- `termination_reason`, `search_mode`, `best_obj`, `valid`, `runtime_ms`, `generations`

Delta report columns:
- `ms_per_generation_delta` = (v4 - baseline) / baseline × 100%
- `obj_delta` = (v4 - baseline) / baseline × 100%
- `feasibility_delta` = v4_valid_count - baseline_valid_count
- `search_mode_changes` = instances that changed SearchMode classification

---

## 10. Acceptance Criteria

### 10.1 Constitutional exit criterion

> ≥2× reduction in ms/gen on setA-10 with no degradation in objective quality or
> feasibility rate.

Formally:
- `ms_per_generation_v4[setA-10] ≤ ms_per_generation_baseline[setA-10] / 2`
- `valid_count_v4 ≥ valid_count_baseline`
- `best_obj_v4[i] ≤ best_obj_baseline[i]` for all valid instances i (within 1% tolerance)

### 10.2 Secondary indicators

- Previously evaluation-limited instances completing more generations.
- EvaluationLimited → SearchLimited transitions (optimizer now has room to search).
- No increase in infeasible outcomes.
- A-001 preserved across all 20 instances.
- E-001 preserved (zero violations in validation campaign).

### 10.3 Rejection criteria

RP-310 is rejected if any of the following occur:
- E-001 violation in production (any mismatch between full and cached evaluator).
- A-001 violation (valid=true, obj=inf).
- Feasibility regression (valid_count_v4 < valid_count_baseline).
- Objective regression on any previously valid instance (best_obj_v4[i] > best_obj_baseline[i] × 1.01).

---

## 11. Architectural Boundary

The Dijkstra result cache is scoped to a single evaluation call. It does not persist
across evaluations, does not require changes to the genome representation, and does not
depend on ROADEF-specific knowledge beyond the observation that `disabled_arcs` is
determined by the scenario (not the genome).

**Generalizability filter:** Would this improve multiple optimization domains?

Any Coralys domain using repeated shortest-path computation under a fixed graph topology
within a single evaluation call would benefit from the same caching pattern. The
mechanism (per-evaluation Dijkstra cache keyed by target node) is domain-agnostic.

**Decision:** The RP-310 implementation begins in `adapters/roadef`. If the pattern
proves reusable across domains, it will be promoted to `coralys-moga`. Abstractions
are not introduced before there is evidence they generalize.

---

## 12. Research Impact Ledger Entry (to be completed after Phase 4)

| Field           | Value                                                                    |
|-----------------|--------------------------------------------------------------------------|
| RP              | RP-310                                                                   |
| Status          | Pending                                                                  |
| Hypothesis      | Caching Dijkstra results per (target_node, time_slot) eliminates redundant shortest-path computation |
| Primary metric  | ms/gen on setA-10                                                        |
| Baseline        | BASELINE-v1.0.json                                                       |
| Key Finding     | TBD after Campaign v4                                                    |

---

*RP-310 Design Specification v1.0 — 2026-07-11*
*Coralys Platform — M20 Engineering Milestone*
*Grounded in M20 Phase 1 Evaluator Performance Model*
*Evidence chain: O-007 → O-008 → O-009 → cache hypothesis → E-001 → Campaign v4*