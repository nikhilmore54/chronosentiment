# GERAD Phase 8 — Operator Timing Attribution

**Status: CLOSED**
**Commits:** `c2f4153c1` (initial instrumentation + setA-01 gate), `65d79a9a5` (extended instrumentation + setA-01 gate PASS), `31dc8f15c` (setA-14 corroboration evidence)

---

## Objective

Phase 8 was a measurement-only phase. No algorithmic changes were made. The goal was to attribute the 87.8% unattributed overhead identified in Phase 7 to specific operators, and to satisfy the attribution gate (≥ 80% of unattributed overhead attributed) on both setA-01 and setA-14.

---

## Instrumentation

### Fields added to `GenerationSummary` (`moga_impl.rs`)

| Field | Description |
|---|---|
| `crossover_ms` | Time in `crossover.crossover()` calls |
| `mutation_ms` | Time in `mutator.mutate()` calls |
| `repair_ms` | Time in `process_offspring()` — repair path |
| `improve_ms` | Time in `process_offspring()` — improve path |
| `sort_ms` | Time in `next_generation.sort_by()` |
| `selection_ms` | Time in parent index sampling |
| `feasibility_ms` | Time in `constraint_model.is_feasible()` calls |
| `staging_ms` | Time in miss_indices/miss_genomes collection + staging flatten |

### Wrapping in `pipeline_impl.rs`

All operator calls in `run_pipeline_evolution_v2` were wrapped with `Instant::now()` / `.elapsed()` timers. No algorithmic logic was changed. The trajectory is bit-exact with Phase 7 baseline on both instances.

---

## setA-01 Gate Result

**Instance:** setA-01 | **Generations:** 50 | **Seed:** 42

### Trajectory invariants (5/5 PASS — bit-exact vs Phase 7 baseline)

| Invariant | Value |
|---|---|
| `best_obj` | 51.0126807372 |
| `n_actual_evals` | 1802 |
| `generations_run` | 50 |
| `valid` | true |
| `cache_hits` | 250 |

### Operator attribution

| Component | Total (ms) | % wall |
|---|---|---|
| Eval (Phase B parallel) | 1,392 | 17.0% |
| L1 cache total | 12 | 0.1% |
| **Feasibility check (`is_feasible`)** | **3,183** | **38.9%** |
| **Improve (`process_offspring`)** | **2,922** | **35.7%** |
| Repair (`process_offspring`) | 547 | 6.7% |
| Crossover | 6 | 0.1% |
| Mutation / Sort / Selection / Staging | <3 | 0.0% |
| **Attributed total** | **6,661** | **81.4%** |
| **Rayon residual** | **4** | **0.1%** |

**Attribution gate: 99.9% — GATE PASS**

Evidence: [`evidence/phase8_setA01_gate.csv`](../evidence/phase8_setA01_gate.csv), [`evidence/phase8_setA01_gate_summary.txt`](../evidence/phase8_setA01_gate_summary.txt)

---

## setA-14 Corroboration Result

**Instance:** setA-14 | **Generations:** 50 | **Seed:** 42

### Trajectory invariants (5/5 PASS — bit-exact vs Phase 7 baseline)

| Invariant | Value |
|---|---|
| `best_obj` | 86.1250850504 |
| `n_actual_evals` | 2006 |
| `generations_run` | 50 |
| `valid` | true |
| `cache_hits` | 181 |

### Operator attribution

| Component | Total (ms) | % wall |
|---|---|---|
| Eval (Phase B parallel) | 281,798 | 13.9% |
| L1 cache total | 261 | 0.0% |
| **Feasibility check (`is_feasible`)** | **848,013** | **41.7%** |
| **Improve (`process_offspring`)** | **821,659** | **40.4%** |
| Repair (`process_offspring`) | 49,589 | 2.4% |
| Crossover | 126 | 0.0% |
| Mutation / Sort / Selection / Staging | ~109 | 0.0% |
| **Attributed total** | **1,719,496** | **84.6%** |
| **Rayon residual** | **107** | **0.0%** |

**Attribution gate: 100.0% — GATE PASS**

Evidence: [`evidence/phase8_setA14_corroboration.csv`](../evidence/phase8_setA14_corroboration.csv), [`evidence/phase8_setA14_corroboration_summary.txt`](../evidence/phase8_setA14_corroboration_summary.txt)

---

## Closure Checklist

| Requirement | State |
|---|---|
| setA-01 trajectory invariants (5/5) | ✅ PASS |
| setA-01 attribution ≥ 80% | ✅ PASS — 99.9% |
| setA-14 trajectory invariants (5/5) | ✅ PASS |
| setA-14 attribution ≥ 80% | ✅ PASS — 100.0% |
| setA-14 evidence committed | ✅ `31dc8f15c` |
| This document | ✅ |
| **Phase 8 formal closure** | ✅ **CLOSED** |

---

## Key Findings

### 1. Rayon hypothesis falsified

The Phase 7 unattributed overhead (87.8% of wall) was hypothesised to include significant Rayon coordination cost. Phase 8 measurement shows:

- **Rayon residual on setA-01: 0.1% (4ms / 50 gens)**
- **Rayon residual on setA-14: 0.0% (107ms / 50 gens)**

Rayon coordination is negligible. The 41.2% "Rayon residual" in the initial gate run was entirely `constraint_model.is_feasible()` — which was uninstrumented in the first pass.

### 2. Dominant components identified

| Rank | Component | setA-01 % wall | setA-14 % wall |
|---|---|---|---|
| 1 | `constraint_model.is_feasible()` | **38.9%** | **41.7%** |
| 2 | `process_offspring` (improve path) | **35.7%** | **40.4%** |
| 3 | `process_offspring` (repair path) | 6.7% | 2.4% |
| 4 | All other operators | ~0.1% | ~0.0% |

The ranking is **consistent across both instances**. `is_feasible()` is the dominant non-eval component on both setA-01 and setA-14.

### 3. Eval is not the bottleneck

Eval (Phase B parallel) accounts for 17.0% (setA-01) and 13.9% (setA-14) of wall-clock. The Phase 3 Rayon parallelisation has already addressed this component. The remaining bottleneck is in the sequential Phase A loop.

---

## Phase 9 Candidates

Phase 8 establishes two candidates for Phase 9 optimization. **No candidate is promoted here** — Phase 9 must scope and gate each independently.

| Candidate | setA-01 % wall | setA-14 % wall | Notes |
|---|---|---|---|
| `constraint_model.is_feasible()` | 38.9% | 41.7% | Called once per offspring per generation; result used only to classify repair vs improve |
| `process_offspring` (improve path) | 35.7% | 40.4% | Called for every feasible offspring; includes local search |

The governance principle applies: **measure → corroborate → close → scope optimization**. Phase 9 must define its own gate criteria before any optimization is implemented.