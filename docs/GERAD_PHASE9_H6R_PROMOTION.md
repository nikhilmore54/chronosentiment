# Phase 9 P9-B — H6-revised: Generation-Scoped Dijkstra Cache — PROMOTED

**Commit:** 1919018aa  
**Date:** 2026-08-25  
**Status:** PROMOTED  

---

## Hypothesis

H6-revised: Memoize `backward_dijkstra(graph, target, disabled_arcs)` results within each
generation using a thread-local `HashMap<(target_node_id: u64, time_slot: usize), Arc<DijkstraResult>>`.
Reset the cache at each generation boundary. This eliminates the dominant repeated computation
inside `improve → expand_sr_path → backward_dijkstra`.

**Correctness invariant:** `disabled_arcs` is derived solely from `scenario.interventions[ts]`
(immutable scenario data, not genome-dependent), so all candidates in a generation share the
same `disabled_arcs` for a given `time_slot`. The cache key `(target, ts)` is therefore
sufficient to guarantee result correctness across all genomes in a generation.

---

## Precondition Measurements (from characterization phase)

### setA-01 (nodes=20, time_slots=2, demands=40)

| Metric | Value |
|--------|-------|
| Total Dijkstra calls | 233,216 |
| Calls per generation | 4,664.3 avg |
| Dijkstra / improve_ms | 59.9% |
| Dijkstra / wall | 40.7% |
| Unique targets observed | 20/20 |
| Reuse lower bound | 100.0% |
| Worst-case cache footprint | 0.03 MB |

### setA-14 (nodes=250, time_slots=2, demands=600)

| Metric | Value |
|--------|-------|
| Total Dijkstra calls | 3,647,518 |
| Calls per generation | 72,950.4 avg |
| Dijkstra / improve_ms | 68.7% |
| Dijkstra / wall | 51.3% |
| Unique targets observed | 250/250 |
| Reuse lower bound | 100.0% |
| Worst-case cache footprint | 4.81 MB |

All four preconditions satisfied: high call volume, small key space, heavy reuse, acceptable memory.

---

## Implementation

### Files modified

- **`adapters/roadef/src/ecmp.rs`**
  - Added `use std::sync::Arc`
  - Added thread-local `DIJKSTRA_CACHE: RefCell<HashMap<(u64, usize), Arc<DijkstraResult>>>`
  - Added `dijkstra_cache_reset()` — clears the cache; called at generation boundaries
  - Added `expand_sr_path_cached(graph, source, target, waypoints, disabled_arcs, flow, arc_flow, time_slot)` — cached variant with nested `cached_dijkstra()` helper
  - `cached_dijkstra()`: checks cache first (O(1) Arc clone on hit), falls back to `backward_dijkstra()` on miss and stores `Arc<DijkstraResult>`

- **`adapters/roadef/src/constraints.rs`**
  - Switched `evaluate_violations()` Stage 3+4 routing call from `expand_sr_path` to `expand_sr_path_cached` (passes `ts`)
  - Switched `is_feasible_fast()` Stage 3+4 routing call from `expand_sr_path` to `expand_sr_path_cached` (passes `ts`)

- **`adapters/roadef/src/pipeline_impl.rs`**
  - Added `use crate::ecmp::dijkstra_cache_reset`
  - Added `dijkstra_cache_reset()` call at top of generation loop (before per-generation timer setup), ensuring stale results from the previous generation's `disabled_arcs` are never reused

### Key design decisions

- **`Arc<DijkstraResult>` not clone:** `DijkstraResult` contains two `HashMap`s (`dist`, `preds`). Cloning is O(N). Arc gives O(1) pointer increment on cache hit.
- **Thread-local cache:** Matches existing instrumentation pattern (`DIJKSTRA_CALL_COUNT`, etc.). Avoids locking overhead with Rayon parallel evaluation.
- **ROADEF adapter only:** Cache lives in `adapters/roadef/src/ecmp.rs`, not in `coralys-core`. No changes to the core library.
- **Generation scope:** Cache reset at each generation boundary. `disabled_arcs` is scenario-driven (constant per `ts` within a generation) so results are valid for the entire generation.

---

## Gate Results

### setA-01 (primary gate, seed=42, 50 generations)

| Invariant | Baseline | H6-revised | Match |
|-----------|----------|------------|-------|
| best_obj | 51.0126807372 | 51.0126807372 | ✓ PASS |
| n_actual_evals | 1802 | 1802 | ✓ PASS |
| generations | 50 | 50 | ✓ PASS |
| valid | true | true | ✓ PASS |
| cache_hits | 250 | 250 | ✓ PASS |

| Timing | Value |
|--------|-------|
| Baseline wall_clock_ms | 4,029 |
| H6-revised wall_clock_ms | 2,468 |
| T_net | +1,561 ms |
| Speedup | +38.7% |

**GATE PASS: 5/5 bit-exact, T_net > 0**

### setA-14 (corroboration, seed=42, 50 generations)

| Invariant | Baseline | H6-revised | Match |
|-----------|----------|------------|-------|
| best_obj | 86.1250850504 | 86.1250850504 | ✓ PASS |
| n_actual_evals | 2006 | 2006 | ✓ PASS |
| generations | 50 | 50 | ✓ PASS |
| valid | true | true | ✓ PASS |
| cache_hits | 181 | 181 | ✓ PASS |

| Timing | Value |
|--------|-------|
| Baseline wall_clock_ms | 989,132 |
| H6-revised wall_clock_ms | 473,983 |
| T_net | +515,149 ms |
| Speedup | +52.1% |

**CORROBORATION PASS: 5/5 bit-exact, T_net > 0**

---

## Cumulative Phase 9 Speedup

| Intervention | setA-01 T_net | setA-14 T_net |
|---|---|---|
| H3: eliminate redundant is_feasible() | +2,035ms (+47%) | +1,017,178ms (+50.7%) |
| H6-original: staged early-exit | +316ms (+7.3%) | (not measured) |
| H6-revised: generation-scoped Dijkstra cache | +1,561ms (+38.7%) | +515,149ms (+52.1%) |

Note: H6-original and H6-revised are cumulative on top of H3. The setA-14 baseline for H6-revised
was the H3-promoted codebase (wall=989,132ms), not the original Phase 7 baseline (wall=1,743,710ms).

---

## Evidence Files

- `evidence/phase9_h6r_impl_setA01_gate_summary.txt` — setA-01 gate run output
- `evidence/phase9_h6r_impl_setA14_corroboration_summary.txt` — setA-14 corroboration run output
- `evidence/phase9_h6r_setA01_dijkstra_measure.txt` — setA-01 precondition measurement
- `evidence/phase9_h6r_setA14_dijkstra_measure.txt` — setA-14 precondition measurement
- `docs/GERAD_PHASE9_H6R_CHARACTERIZATION.md` — characterization and authorization document

---

## Decision

**H6-revised PROMOTED.**

The generation-scoped Dijkstra cache is correct (5/5 trajectory invariants bit-exact on both
instances), delivers substantial wall-time reduction on both small (setA-01: +38.7%) and large
(setA-14: +52.1%) instances, and satisfies all gate criteria. The implementation is confined to
the ROADEF adapter and does not touch coralys-core.