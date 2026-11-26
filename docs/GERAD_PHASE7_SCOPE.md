# GERAD Phase 7 — Scope Document

**Status:** 🔵 SCOPING — no engineering started  
**Baseline:** Phase 6 closure commit `947e6a210`  
**Date:** 2026-08-24  

---

## 1. Phase 6 Baseline (authoritative)

From the setA-14 Phase 6 Arm A (contemporaneous control):

| Metric | Value |
|---|---|
| Wall-clock | 1,781,788ms (29.7 min) |
| Eval time | 199,179ms (11.2% of wall) |
| Non-eval overhead | 1,582,609ms (88.8% of wall) |
| L1 cache hits | 181 |
| L2 cache entries | 500 |
| Actual evaluations | 2,006 |

The Phase 5 profile established that the evaluator's internal components are now fully optimised:
- Stage 1 (segment_check): trivial, no-go
- Stage 2 (budget_check): genome-dependent, no-go
- Stage 3 (backward_dijkstra): **DONE** — L2 cache implemented in Phase 6
- Stage 3 (route_ecmp): stateful accumulator, no-go
- Stage 4 (objective): trivial, no-go

**The remaining 88.8% of wall-clock is non-evaluator overhead.** Phase 7 must profile this
overhead before any optimisation is attempted.

---

## 2. Known Non-Evaluator Overhead Components

From the evolution loop in `pipeline_impl.rs` and `moga_impl.rs`, the non-eval wall-clock
includes:

| Component | Location | Estimated cost | Notes |
|---|---|---|---|
| L1 genome cache lookup | `pipeline_impl.rs` | Unknown | HashMap lookup per candidate |
| L1 cache miss materialisation | `pipeline_impl.rs` | Unknown | Clone + insert on miss |
| Selection (tournament) | `moga_impl.rs` | Unknown | O(pop_size × tournament_size) |
| Crossover | `moga_impl.rs` | Unknown | O(demands) per operation |
| Mutation | `moga_impl.rs` | Unknown | O(demands) per operation |
| Repair (pipeline) | `pipeline_impl.rs` | Unknown | Constraint model + repair operators |
| Population merge/sort | `moga_impl.rs` | Unknown | O(pop_size × log(pop_size)) |
| Telemetry/logging | `moga_impl.rs` | Unknown | Suppressed in A/B harness |
| Rayon thread coordination | `pipeline_impl.rs` | Unknown | Spawn + join overhead per gen |

---

## 3. Phase 7 Objective

**Profile the non-evaluator overhead** to identify which component(s) dominate the 88.8% of
wall-clock that is not evaluation time.

This is a **measurement-only phase** (no code changes, no promotion decisions). The output is
a component breakdown analogous to the Phase 5 evaluator profile, but for the evolution loop.

---

## 4. Measurement Approach

### 4.1 Instrument the evolution loop

Add per-generation timing to `run_pipeline_evolution_v2` in `pipeline_impl.rs`:

```rust
// Per-generation breakdown (already partially present in GenerationSummary):
pub struct GenerationSummary {
    pub generation: usize,
    pub best_obj: f64,
    pub n_eval: usize,
    pub duplicate_genomes: usize,
    pub cache_hits: usize,
    pub generation_runtime_ms: f64,      // total gen wall-clock
    pub evaluation_runtime_ms: f64,      // eval phase only
    // NEW fields for Phase 7:
    pub selection_ms: f64,               // tournament selection
    pub crossover_ms: f64,               // crossover operator
    pub mutation_ms: f64,                // mutation operator
    pub repair_ms: f64,                  // pipeline repair
    pub merge_sort_ms: f64,              // population merge + sort
    pub cache_lookup_ms: f64,            // L1 cache lookup
    pub rayon_overhead_ms: f64,          // spawn + join (gen_runtime - sum of above)
}
```

### 4.2 Run profile binary

Create `bin/phase7_loop_profile.rs` — runs setA-14 for 50 generations and emits the per-
generation breakdown to a CSV file for analysis.

### 4.3 Analysis

Compute mean and stddev of each component across 50 generations. Identify the top-2 components
by mean time. These become the Phase 8 candidates.

---

## 5. Candidate Optimisations (hypotheses only — not yet validated)

These are hypotheses to be tested after the Phase 7 profile. **No engineering until profile
confirms the bottleneck.**

| Hypothesis | Component | Mechanism | Risk |
|---|---|---|---|
| H1: Repair is dominant | Repair operators | Reduce repair budget or skip repair for valid genomes | May reduce solution quality |
| H2: L1 cache lookup is significant | L1 HashMap | Replace with faster hash (FxHashMap) | Minimal risk |
| H3: Population sort is significant | merge_sort | Use partial sort (top-k) instead of full sort | May affect selection pressure |
| H4: Rayon spawn overhead is significant | Thread pool | Pre-warm Rayon pool; use scoped threads | Minimal risk |
| H5: Crossover/mutation are significant | Operators | Vectorise waypoint operations | Low risk |

---

## 6. Governance Protocol

Phase 7 follows the same protocol as Phase 5 (observational):

1. **No code changes** until the profile identifies a specific bottleneck.
2. Profile binary is a new `bin/` file — does not modify production code.
3. Output is a markdown evidence document analogous to `GERAD_PHASE5_EVALUATOR_PROFILE.md`.
4. Phase 7 closes when the profile document is committed and the top-2 bottlenecks are identified.
5. Phase 8 scoping begins from the Phase 7 findings.

---

## 7. Phase 7 Baseline Invariants

Any Phase 8 optimisation must preserve:

| Invariant | Value (Phase 6 baseline) |
|---|---|
| `best_obj` | 86.1250850504 (setA-14, seed=42) |
| `n_actual_evals` | 2006 |
| `generations_run` | 50 |
| `valid` | true |
| `cache_hits` (L1) | 181 |

---

## 8. Commit Chain

```
947e6a210  Phase 6 CLOSED ← current HEAD
[next]     Phase 7 profile binary + evidence