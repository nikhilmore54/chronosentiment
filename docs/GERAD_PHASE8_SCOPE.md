# GERAD Phase 8 — Operator Timing Instrumentation

**Status:** OPEN  
**Date:** 2026-08-24  
**Prerequisite:** Phase 7 CLOSED (commit 59d3c6b1b)  
**Governance:** Measurement-only — add per-operator timing to `GenerationSummary`, no algorithmic changes

---

## 1. Motivation

Phase 7 identified that **87.8% of wall-clock time** on setA-14 is `unattributed_ms` — the accounting residual after subtracting eval and L1 cache time. This residual contains (unmeasured): repair, improvement, crossover, mutation, selection, sort, and Rayon spawn/join overhead.

Phase 8 must instrument these individually before any optimization can be proposed. The governance protocol requires measurement evidence before engineering changes.

**Phase 8 gate criterion:** the sum of all newly instrumented components must account for ≥80% of `unattributed_ms` before any optimization is promoted.

---

## 2. Hypotheses (ranked by prior probability)

| Rank | Hypothesis | Rationale |
|---|---|---|
| H1 | **Repair/Improvement operators dominate** | Up to 10 iterations each × ~40 genomes/gen = 800 repair + 800 improve calls. Operators likely involve constraint checking proportional to instance size. |
| H2 | **Crossover operator is significant** | Applied to ~70% of offspring (~28 pairs/gen). Deep genome cloning or complex recombination could be costly. |
| H3 | **Sort/merge is measurable** | NSGA-II non-dominated sort is O(M·N²). With pop=50 and multiple objectives, this is bounded but measurable. |
| H4 | **Rayon spawn/join is negligible** | Thread pool overhead is typically <1ms/gen. Expected to be <0.1% of unattributed. |
| H5 | **Mutation is negligible** | Applied to ~30% of offspring. Single-point or swap operations are O(genome_length). |

---

## 3. Required Changes

### 3.1 New fields in `GenerationSummary` (in `moga_impl.rs`)

Add the following fields to the [`GenerationSummary`](adapters/roadef/src/moga_impl.rs) struct:

```rust
pub repair_ms: f64,       // total time in RoadefRepair per generation
pub improve_ms: f64,      // total time in RoadefImprovement per generation
pub crossover_ms: f64,    // total time in RoadefCrossover per generation
pub mutation_ms: f64,     // total time in RoadefMutator per generation
pub sort_ms: f64,         // total time in NSGA-II sort per generation
pub selection_ms: f64,    // total time in selection per generation
```

### 3.2 Instrumentation sites

Wrap each operator call site in [`run_pipeline_evolution_v2`](adapters/roadef/src/pipeline_impl.rs) with `Instant::now()` / `.elapsed()` timers and accumulate into the new fields.

### 3.3 Profile binary update

Update [`phase7_loop_profile.rs`](adapters/roadef/src/bin/phase7_loop_profile.rs) (or create `phase8_operator_profile.rs`) to output the new fields in the CSV and verify the accounting closure:

```
repair_ms + improve_ms + crossover_ms + mutation_ms + sort_ms + selection_ms
  ≈ unattributed_ms  (within 5%)
```

---

## 4. Governance Constraints

1. **No algorithmic changes** in Phase 8. Only add timing instrumentation.
2. **Trajectory invariants must be bit-exact** after instrumentation (timers must not affect RNG or operator logic).
3. **Gate criterion:** instrumented components must sum to ≥80% of `unattributed_ms` before Phase 9 optimization is scoped.
4. **setA-01 gate run** required before setA-14 corroboration run (same protocol as Phases 3, 6, 7).

---

## 5. Phase 8 Success Criterion

- All 5 trajectory invariants pass on setA-01 (bit-exact vs Phase 7 baseline)
- All 5 trajectory invariants pass on setA-14 (bit-exact vs Phase 7 baseline)
- Per-operator breakdown CSV produced for both instances
- Accounting closure: `Σ(operator_ms) ≥ 80% of unattributed_ms`
- Top-1 optimization candidate identified with quantified impact estimate

---

## 6. Expected Outcome

Based on Phase 7 data and H1 prior:

If repair+improve accounts for ~70% of unattributed_ms on setA-14, that is:
- 0.70 × 1,531,214ms = ~1,071,850ms = **61.5% of total wall-clock**

This would make repair/improvement the single largest optimization target in the pipeline, exceeding even the Dijkstra evaluator (10.7% of wall).

Potential Phase 9 optimizations (to be scoped only after Phase 8 measurement):
- Reduce repair budget (`max_iterations`, `max_time_ms`)
- Early-exit repair on first feasible solution
- Lazy repair (skip repair for elite genomes)
- Parallel repair (Rayon over new genomes)