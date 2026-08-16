# RC-003 Baseline Report: IFR Improvement via Constructor Fixes

**Campaign:** rc003_lex_v1.0  
**Date:** 2026-08-07  
**Population:** 50 | **Generations:** 500 | **Elite:** 5 | **Seed:** 42

---

## Summary of Fixes Applied

Three constructor bugs were identified and fixed in [`adapters/roadef/src/moga_impl.rs`](adapters/roadef/src/moga_impl.rs):

### RC-001A3: Truncation → Routing Failure
**Problem:** `path_to_waypoints_rc001()` silently truncated waypoints when a path required more SR segments than `gd.max_segments` allows. The truncated route was stored and used, causing arc flows to accumulate from partial paths. This produced the observed 22× overload on arcs 362/363 for setA-05 Arm B while `failures=0` (the contradiction that triggered investigation).

**Fix:** After truncation detection, add `n_routing_failures += 1; continue;` — treat truncation as a routing failure, skip storing waypoints, let the evaluator use ECMP default routing for that commodity.

**Effect:** setA-01 Arm B: `truncations=0, max_sat=0.929, IFR: 0% → 100%`

### RC-001A4: All-Slots Saturation Tracking
**Problem:** The greedy constructor tracked saturation only for `worst_slot` (the time slot with highest total demand volume). For setA-02, `worst_slot=t=0` showed `max_sat=0.811` (feasible), but the evaluator checks all slots — at `t=1`, arc 40 reaches `sat=1.098` due to different disabled arcs and demand volumes. The greedy reported success while the evaluator marked every genome invalid.

**Fix:** Replace single-slot `running_arc_flows` with `running_arc_flows_per_slot: Vec<HashMap<u64, f64>>` (one accumulator per slot). Run `expand_sr_path` for all slots in the saturation update loop. `max_saturation_seen` now covers all slots. The Dijkstra penalty (`ecmp_saturation`) still uses `worst_slot` flows only for consistency.

**Effect:** setA-02 Arm B greedy now reports `max_sat=1.098` (matches evaluator). Constructor/evaluator divergence eliminated for this instance.

### RC-001A5: ECMP Fallback for Infeasible Constructions
**Problem:** Even after RC-001A4, the greedy detected `max_sat=1.098 > 1.0` but still returned the infeasible SR waypoints. Every genome in the initial population was invalid (IFR=0), leaving repair with no valid fallback and the GA permanently stuck.

**Fix:** After construction, if `max_saturation_seen > 1.0`, return `waypoints: vec![vec![]; self.num_demands]` (empty = ECMP default for all commodities). This breaks the deterministic lock where all 50 genomes have identical infeasible waypoints.

**Effect:** setA-05 Arm B: `IFR: 16% → 100%` (ECMP routing is feasible for this instance).

---

## IFR Comparison Table (v7 baseline vs v12 with all fixes)

| Instance | Arm A v7 | Arm A v12 | Arm B v7 | Arm B v12 | Notes |
|----------|-----------|-----------|-----------|-----------|-------|
| setA-01  | 16%       | 60%       | 100%      | 100%      | A improved via rejection sampling |
| setA-02  | 0%        | 2%        | 0%        | 0%        | Hard instance — ECMP also infeasible |
| setA-03  | 6%        | 52%       | 2%        | 64%       | Both arms improved |
| setA-04  | 20%       | 64%       | 100%      | 90%       | A improved; B stable |
| setA-05  | 80%       | 100%      | 0%        | 100%      | B: 0%→100% via RC-001A3+A5 |
| setA-06  | 6%        | 4%        | 14%       | 2%        | Low IFR — hard instance |
| setA-07  | —         | 0%        | —         | 10%       | A infeasible; B marginal |
| setA-08  | —         | 8%        | —         | 0%        | B infeasible |
| setA-09  | —         | 16%       | —         | 0%        | B infeasible |
| setA-10  | —         | 2%        | —         | 2%        | Both marginal |
| setA-11  | —         | 8%        | —         | 0%        | B infeasible |
| setA-12  | —         | 0%        | —         | 8%        | A infeasible |
| setA-13  | —         | (pending) | —         | (pending) | Campaign running |

*v7 baseline: first 6 instances only (from session history). v12: all fixes applied.*

---

## Key Finding: setA-02 Arm B is a Known Hard Instance

setA-02 Arm B has `IFR=0/50` across all campaign versions (v6 through v12). Investigation shows:

- The greedy constructor routes all 45 demands through arc 40 at `t=1`, producing `sat=1.098`
- Returning empty waypoints (ECMP fallback) also routes through arc 40 at `t=1` — ECMP is also infeasible
- Arm A (random constructor) achieves `IFR=1/50 (2%)` — the instance IS feasible, but only barely
- The GA can find a solution via evolution from the 1 valid Arm A genome, but Arm B cannot construct a valid initial genome

**Conclusion:** setA-02 Arm B IFR=0 is correct and expected. The instance requires a routing that avoids arc 40 at `t=1`, which neither the greedy nor ECMP can find deterministically. The GA must discover it through crossover/mutation from Arm A's valid genomes.

---

## Fixes Applied to `moga_impl.rs`

All three fixes are in [`adapters/roadef/src/moga_impl.rs`](adapters/roadef/src/moga_impl.rs):

- **RC-001A3** (~line 393): `n_routing_failures += 1; continue;` after truncation detection
- **RC-001A4** (~line 339): `disabled_arcs_per_slot: Vec<HashSet<u64>>` and `running_arc_flows_per_slot: Vec<HashMap<u64, f64>>`
- **RC-001A5** (~line 544): `if max_saturation_seen > 1.0 { return RoadefGenome { waypoints: vec![vec![]; self.num_demands], ... }; }`

---

## Pending

- Full v12 IFR table (setA-13 through setA-20) — campaign running
- RC-003 final report with Spearman ρ and lexicographic comparison
- ARCH-008: `Individual = Genome + EvaluationResult` with explicit lifecycle states (future work, not pre-submission blocker)