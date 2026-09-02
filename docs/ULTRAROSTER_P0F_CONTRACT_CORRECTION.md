# P0-F: Product-Contract Correction — Candidate Population Gap

**Status:** ASSESSMENT (not implementation)  
**Date:** 2026-09-02  
**Baseline:** P0-E CLOSED at 5890e967b, P1 readiness at 55c343695  
**Method:** Static trace of scalar GA → pipeline → Pareto pipeline → API response

---

## Finding

The live UI shows "Decision alternatives unavailable" for a valid generated roster.
This is not a bug in the P0 architecture — it is a **product-contract gap**: the
current architecture only exposes Pareto-non-dominated alternatives, but the product
requirement is to expose the **generated decision space** so the scheduler can
understand the solution space and make a decision.

---

## Root Cause: `top_10` Is Discarded

The scalar GA (`EvolutionEngine`) produces a `GaResult` with:

```rust
pub struct GaResult<E: Evaluated> {
    pub global_best: E,              // ← used → ScheduleSolution → schedule
    pub top_10: Vec<E>,              // ← DISCARDED in run_pipeline()
    pub generation_history: Vec<E>,  // ← DISCARDED
    pub final_fitnesses: Vec<f64>,   // ← DISCARDED
    ...
}
```

`run_pipeline()` in `adapters/ultracrew/src/pipeline.rs` uses only `global_best`:

```rust
let ga_result = run_optimization(context.clone(), config);
let best_evaluation = ga_result.global_best;  // top_10 is never accessed
let mut solution = ScheduleSolution::from_evaluation(&best_evaluation);
```

`top_10` contains the 10 best `ScheduleEvaluation` records from the scalar GA's
final population — genuine optimizer-generated candidates, weaker than `global_best`
but real. They are computed and then thrown away.

The Pareto pipeline then starts fresh from the primary genome as seed, with no
knowledge of the scalar GA's top-10 population. For a homogeneous dataset (all
workers with the same skills), the Pareto search produces no non-dominated
alternatives beyond the primary — so `alternatives=[]`.

---

## The Three Candidate Layers

```
A. Primary solution
   global_best from scalar GA
   → always present in ScheduleResponse.schedule

B. Pareto-non-dominated alternatives
   from run_pareto_pipeline()
   → currently the only source of ScheduleResponse.alternatives
   → empty for homogeneous datasets

C. Scalar GA top-10 candidates
   top_10 from GaResult
   → currently DISCARDED
   → genuine optimizer-generated, weaker than primary
   → available for every dataset regardless of Pareto diversity
```

The current P0 architecture exposes only **A** (as primary) and **B** (as
alternatives). Layer **C** exists in the optimizer but is never surfaced.

---

## Product Requirement Clarification

The intended product behavior is:

> Show the roster that Coralys generated, and show the weaker/trade-off candidates
> that were actually generated, so the scheduler can understand the solution space
> and make a decision.

This means:

1. The primary schedule (A) should always be visible.
2. Genuine optimizer-generated alternatives (B and/or C) should be available
   for scheduler consideration.
3. Alternatives must come from Coralys — not from frontend synthesis.
4. Infeasible candidates should be described (with `CandidateFeasibility`), not
   silently filtered.
5. The scheduler retains decision authority.

The current hard stop ("Decision alternatives unavailable") violates requirement 2
for homogeneous datasets, even though a valid roster was generated.

---

## Candidate Contract Correction

The corrected candidate contract is:

```
Primary:
    global_best from scalar GA
    → ScheduleResponse.schedule (unchanged)

Alternatives:
    Pareto-non-dominated candidates (B) if any exist
    UNION
    Scalar GA top-10 candidates (C) that are materially different from primary
    → ScheduleResponse.alternatives
    → with CandidateFeasibility for each
    → without frontend ranking/filtering/synthesis
    → empty only if the optimizer genuinely produced no distinct candidates
```

The key change: **Layer C (scalar GA top-10) becomes a fallback candidate pool**
when the Pareto archive produces no non-dominated alternatives.

---

## Implementation Direction (NOT authorized yet)

The minimal implementation requires three changes:

### 1. Expose `top_10` from `run_pipeline()`

`ScheduleSolution` needs a `top_candidates` field:

```rust
pub struct ScheduleSolution {
    pub assignments: HashMap<u64, u64>,
    pub recommendations: Option<Vec<Recommendation>>,
    pub telemetry: Option<OptimizationReport>,
    pub top_candidates: Vec<ScheduleGenome>,  // NEW: from GaResult.top_10
}
```

`run_pipeline()` populates it:

```rust
let ga_result = run_optimization(context.clone(), config);
let best_evaluation = ga_result.global_best;
let top_candidates = ga_result.top_10
    .iter()
    .map(|e| e.schedule.clone())
    .collect();
let mut solution = ScheduleSolution::from_evaluation(&best_evaluation);
solution.top_candidates = top_candidates;
```

### 2. Use `top_candidates` as Pareto seeds

`run_pareto_pipeline()` currently seeds with only the primary genome. It should
also accept the scalar GA's top-10 as additional seeds:

```rust
pub fn run_pareto_pipeline(
    context: Arc<ScheduleContext>,
    seed_genome: ScheduleGenome,
    additional_seeds: Vec<ScheduleGenome>,  // NEW: from top_candidates
    pareto_steps: usize,
) -> Vec<ProductionParetoSolution>
```

This gives the Pareto search a richer initial front, increasing the probability
of finding non-dominated alternatives even for homogeneous datasets.

### 3. Fallback: include top-10 as alternatives when Pareto archive is empty

If `run_pareto_pipeline()` returns empty, the server handler should fall back to
the scalar GA's `top_candidates` as alternatives, processed through
`archive_to_solutions()` equivalent logic:

```rust
let alternatives = if pareto_alternatives.is_empty() {
    // Fallback: use scalar GA top-10 as alternatives
    // These are weaker but genuine optimizer-generated candidates
    solution.top_candidates.iter()
        .filter(|g| genome_uid(g) != seed_uid)
        .map(|g| evaluate_and_describe(g, &context))
        .collect()
} else {
    pareto_alternatives
};
```

---

## What This Does NOT Change

- No frontend synthesis of alternatives
- No frontend ranking or recommendation
- No new objective functions
- No changes to the Pareto dominance algorithm
- No changes to `CandidateFeasibility` semantics
- No changes to the `mapParetoAlternatives()` mapping layer
- `alternatives=[]` remains a legitimate outcome if the optimizer genuinely
  produced no distinct candidates (all top-10 have the same uid as primary)

---

## Acceptance Criteria for P0-F

1. For a homogeneous dataset (all workers same skill), `alternatives` is non-empty
   if the scalar GA's top-10 contains candidates distinct from the primary.
2. For a heterogeneous dataset, Pareto alternatives continue to be returned as
   before (no regression).
3. Every alternative has `feasibility` populated.
4. Every alternative has `objectives` populated.
5. No alternative is the primary genome (uid filter preserved).
6. No duplicate alternatives (uid deduplication preserved).
7. 47/47 frontend tests pass.
8. 106/106 Rust tests pass.
9. TSC exits 0.
10. Build clean.

---

## Authorization Request

This document is an assessment only. No implementation has been performed.

The product-contract correction described here requires authorization before
implementation begins. The key decision is:

> **Should the scalar GA's top-10 candidates be surfaced as alternatives when
> the Pareto archive is empty?**

If yes, P0-F implementation is authorized. The implementation is bounded:
3 files changed (pipeline.rs, pareto_pipeline.rs, main.rs), no algorithm changes,
no frontend changes beyond what `mapParetoAlternatives()` already handles.