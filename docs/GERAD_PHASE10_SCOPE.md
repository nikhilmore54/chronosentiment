# GERAD Phase 10 — Scope Document

**Status:** AUTHORIZED  
**Date:** 2026-08-25  
**Prerequisite:** Phase 9 P9-B H6-revised PROMOTED (commit 1919018aa)

---

## Motivation

Phase 9 introduced a generation-scoped Dijkstra cache (`HashMap<(target_node_id, time_slot), Arc<DijkstraResult>>`) that delivered substantial per-generation speedup on medium instances:

- setA-01 (40 demands, 20 nodes): +38.7% wall-time reduction (50 fixed gens)
- setA-14 (600 demands, 250 nodes): +52.1% wall-time reduction (50 fixed gens)

However, a full 20-instance setA campaign run on the Phase 9 platform (2026-08-25) revealed that the cache is **counterproductive on very large instances**:

| Instance | Demands | Nodes | Baseline gens | Phase 9 gens | Baseline wall | Phase 9 wall |
|---|---|---|---|---|---|---|
| setA-13 | 2000 | 200 | 7 | 2 | 329s | 412s |
| setA-16 | 4800 | 250 | 5 | 1 | 306s | 563s |
| setA-19 | 6000 | 300 | 3 | 1 | 314s | 797s |
| setA-20 | 6000 | 400 | 2 | 1 | 346s | 1118s |

Phase 9 runs **fewer generations** and **more wall time per generation** on these instances. The cache overhead (population cost + memory pressure) exceeds the savings from avoiding recomputation at extreme demand counts.

The crossover threshold appears to lie between 600 and 2000 demands.

---

## Research Question

> **At what demand count does the generation-scoped Dijkstra cache become counterproductive, and can a demand-count-gated cache enable/disable policy recover the large-instance regression while preserving the medium-instance benefit?**

---

## Phase 10 Structure

### P10-A: Characterization (measurement only, no implementation)

**Goal:** Precisely locate the crossover threshold and understand the cost structure on large instances.

**Measurements to take (on Phase 9 baseline, commit 1919018aa):**

1. **Per-generation timing on large instances** — run `phase7_loop_profile` on setA-13, setA-16, setA-19, setA-20 with fixed generation count (e.g. 5 gens) to measure:
   - `improve_ms` per generation
   - Dijkstra call count per generation
   - Cache hit rate per generation
   - Cache population time (first-gen vs subsequent gens)

2. **Cache overhead measurement** — instrument `dijkstra_cache_reset()` and the cache lookup/insert path to measure:
   - Time spent in cache lookup (hit path)
   - Time spent in cache insert (miss path)
   - Cache size (entries) at end of each generation
   - Memory footprint

3. **Crossover sweep** — run `phase9_dijkstra_measure` on setA-04 (200d), setA-06 (500d), setA-10 (1000d), setA-13 (2000d) to characterize how cache hit rate and per-call overhead scale with demand count.

**Deliverable:** `docs/GERAD_PHASE10_P10A_CHARACTERIZATION.md` with crossover threshold estimate and cost breakdown.

### P10-B: Hypothesis formulation (after P10-A)

Based on P10-A findings, formulate ONE implementation hypothesis. Candidate hypotheses (not yet authorized for implementation):

**H10-A: Demand-count gate** — disable the cache entirely when `num_demands > THRESHOLD`. Simple, zero overhead when disabled. Risk: threshold is instance-specific, not a universal constant.

**H10-B: Cache size limit** — cap the cache at N entries (LRU eviction). Bounds memory pressure. Risk: eviction overhead, partial hit rate.

**H10-C: First-generation warm-up only** — populate cache in generation 0, reuse across all generations (run-scoped, not generation-scoped). Risk: `disabled_arcs` may differ across generations if scenario has time-varying interventions (must verify correctness invariant first).

**H10-D: Lazy cache** — only cache results that are reused ≥ K times within a generation. Risk: requires hit-count tracking, adds per-lookup overhead.

Only ONE hypothesis will be selected for implementation after P10-A evidence is reviewed.

### P10-C: Implementation and gate (after P10-B authorization)

Same gate protocol as Phase 9:
- 5/5 trajectory invariants bit-exact vs Phase 9 baseline (commit 1919018aa)
- T_net > 0 on both a medium instance (setA-14) and a large instance (setA-16 or setA-19)
- Corroboration on a second large instance

---

## Gate Protocol

**Primary gate instance:** setA-14 (600 demands, 250 nodes) — must not regress vs Phase 9 baseline.  
**Large-instance gate:** setA-16 or setA-19 — must show T_net > 0 vs Phase 9 baseline.  
**Invariants:** 5/5 bit-exact (best_obj, n_actual_evals, generations, valid, cache_hits) on both instances.

**Promotion criteria:** T_net > 0 on both gate instances simultaneously. A solution that helps large instances but regresses medium instances is NOT promotable.

---

## Constraints

- Implementation confined to `adapters/roadef` (no coralys-core changes).
- Phase 9 baseline (commit 1919018aa) is the reference for all P10 gates.
- P10-A is measurement-only: no code changes to production paths.
- P10-B hypothesis selection requires explicit authorization after P10-A evidence review.
- P10-C implementation requires explicit authorization after P10-B selection.

---

## Evidence Files (to be created)

- `evidence/phase10_p10a_setA13_timing.txt`
- `evidence/phase10_p10a_setA16_timing.txt`
- `evidence/phase10_p10a_setA19_timing.txt`
- `evidence/phase10_p10a_crossover_sweep.txt`
- `docs/GERAD_PHASE10_P10A_CHARACTERIZATION.md`