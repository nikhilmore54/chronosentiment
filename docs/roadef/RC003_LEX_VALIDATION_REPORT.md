# RC-003: Lexicographic Validation Report

**Campaign ID:** `rc003_lex_v1.0`
**Status:** 🔬 IN PROGRESS — code analysis complete; benchmark run pending
**Date:** 2026-08-07
**Stream:** A (Submission-required)
**Gate:** SUBMISSION GATE — must pass before ROADEF submission

---

## 1. Research Question

Does the Coralys surrogate objective preserve the official ROADEF lexicographic ordering?

Specifically: when Arm B (Greedy) produces a lower surrogate objective than Arm A (Random) on the same instance, does the official ROADEF evaluator agree that Arm B's solution is better?

Without this validation, the RC-001 A/B results cannot be interpreted as evidence of competition-relevant improvement.

---

## 2. Objective Architecture (Code Analysis)

### 2.1 Surrogate objective (Coralys internal)

From [`adapters/roadef/src/evaluator.rs:680`](adapters/roadef/src/evaluator.rs:680):

```rust
total_obj += loads.mlu + loads.inv_load_cost;
```

The surrogate is a **scalar sum** across all time slots:

```
surrogate_obj = Σ_t ( MLU_t + inv_load_cost_t )
```

where `MLU_t` = maximum link utilization at time slot t, and `inv_load_cost_t` is an inverse-load penalty term.

### 2.2 Official ROADEF objective

The official objective is the **sorted load vector** evaluated lexicographically:

```
lex_obj = sort_descending( { link_saturation(l, t) : l ∈ links, t ∈ time_slots } )
```

Two solutions are compared by their sorted load vectors: the solution with the lower value at the first differing rank wins.

### 2.3 Comparator modes in the codebase

From [`adapters/roadef/src/telemetry.rs:64`](adapters/roadef/src/telemetry.rs:64):

```rust
ComparatorMode::Scalar        // default — used by RC-001 campaign
ComparatorMode::Lexicographic // experimental — used by RP-408B campaign
```

The RC-001 campaign uses `ComparatorMode::Scalar` (the default). The `LexicographicComparator` exists and has unit tests in [`adapters/roadef/src/moga_impl.rs:1904`](adapters/roadef/src/moga_impl.rs:1904) but was not used in the RC-001 campaign.

### 2.4 Alignment risk

The surrogate and official objectives are **not identical**:

| Property | Surrogate | Official |
|----------|-----------|----------|
| Type | Scalar sum | Sorted vector |
| Scope | MLU + inv_load_cost | All link saturations |
| Comparison | Lower is better | Lexicographic |
| Time slots | Summed | Per-slot, then merged |

A solution that minimizes `Σ MLU_t` may not minimize the sorted saturation vector. In particular:

- A solution with lower mean MLU but higher peak saturation on one link could win on the surrogate but lose on the official objective.
- The `inv_load_cost` term is not part of the official objective and may introduce ordering inversions.

---

## 3. Validation Protocol

### 3.1 What must be demonstrated

For each of the 20 setA instances where at least one arm produced a valid solution:

1. Export the best solution from each arm as a `Solution` struct.
2. Compute the official lex_vector: sort all per-link saturations descending across all time slots.
3. Compare Arm A vs Arm B using the official lex ordering.
4. Record whether the official winner matches the surrogate winner.

### 3.2 Deliverables

| Deliverable | Description |
|-------------|-------------|
| `rc003_lex_results.json` | Per-instance: surrogate winner, lex winner, match/inversion |
| Objective Winner vs Lex Winner table | 20-row table showing agreement/disagreement |
| Inversion count | Number of instances where surrogate and lex disagree |
| Spearman rank correlation (ρ) | Quantitative surrogate fidelity across all candidate solutions |
| Inversion analysis | For each inversion: what property caused the disagreement |

### 3.3 Spearman rank correlation

In addition to the binary pass/fail inversion count, compute the Spearman rank correlation between the surrogate ranking and the lexicographic ranking across all candidate solutions evaluated during the campaign.

For each instance where both arms produced valid solutions, rank all evaluated genomes by (a) surrogate objective and (b) official lex objective. Compute ρ between the two rankings.

| ρ range | Interpretation |
|---------|---------------|
| ρ ≥ 0.99 | Surrogate is an excellent proxy for the official objective |
| 0.95 ≤ ρ < 0.99 | Surrogate is a good proxy; minor ordering differences |
| 0.90 ≤ ρ < 0.95 | Surrogate is a moderate proxy; some ordering differences |
| ρ < 0.90 | Surrogate is a poor proxy; significant ordering differences |

A high ρ (e.g. ρ = 0.998) is much stronger evidence of surrogate fidelity than zero inversions alone, because it quantifies the degree of agreement across the full ranking rather than just the top-1 comparison.

### 3.4 Acceptance criterion

**Pass:** Zero ordering inversions across all 20 instances (surrogate winner = lex winner on every instance where both arms are valid), AND ρ ≥ 0.95 across all evaluated solutions.

**Conditional pass:** ≤ 1 inversion with documented explanation, AND ρ ≥ 0.90.

**Fail:** ≥ 2 inversions, or ρ < 0.90, or any inversion on an instance where the surrogate winner is Arm A but the lex winner is Arm B (i.e. the greedy constructor appears better on the surrogate but worse on the official objective).

---

## 4. Implementation Plan

### Step 1 — Add `lex_vector` export to evaluator

Add a method to `RoadefEvaluator` that returns the sorted saturation vector for a given solution:

```rust
pub fn compute_lex_vector(&self, solution: &Solution) -> Option<Vec<f64>> {
    // Collect all link saturations across all time slots
    // Sort descending
    // Return sorted vector
}
```

### Step 2 — Add RC-003 campaign binary

Create `adapters/roadef/src/bin/campaign_rc003.rs`:

- Load best solutions from `benchmarks/roadef/rc001/rc001_ab_report.json`
- Re-evaluate each best solution with `compute_lex_vector()`
- Compare Arm A vs Arm B using lex ordering
- Write `rc003_lex_results.json`

### Step 3 — Run and analyse

```
cargo run --bin campaign_rc003 --release
```

### Step 4 — Produce Objective Winner vs Lex Winner table

| Instance | Surrogate Winner | Lex Winner | Match? | Notes |
|----------|-----------------|------------|--------|-------|
| setA-01  | Arm B           | ?          | ?      | |
| ...      | ...             | ...        | ...    | |

---

## 5. Preliminary Risk Assessment

Based on code analysis, the following scenarios could cause inversions:

**Risk 1 — inv_load_cost term:** The `inv_load_cost` is not part of the official objective. If it dominates the surrogate on some instances, it could cause the surrogate to prefer a solution that the official objective does not.

**Risk 2 — MLU vs sorted saturation:** Minimizing `Σ MLU_t` is not equivalent to minimizing the sorted saturation vector. A solution with lower mean MLU but a single very high saturation link could win on the surrogate but lose on the official objective.

**Risk 3 — Time slot aggregation:** The surrogate sums across time slots; the official objective merges all time slots into a single sorted vector. These aggregation methods are not equivalent.

**Mitigation:** The `LexicographicComparator` already exists in the codebase and has been tested. If inversions are found, switching the campaign to `ComparatorMode::Lexicographic` may resolve them without any algorithmic changes.

---

## 6. Status

| Step | Status | Notes |
|------|--------|-------|
| Code analysis | ✅ Complete | Objective architecture understood |
| `compute_lex_vector()` implementation | ⏳ Pending | |
| `campaign_rc003` binary | ⏳ Pending | |
| Benchmark run | ⏳ Pending | |
| Objective Winner vs Lex Winner table | ⏳ Pending | |
| Pass/fail determination | ⏳ Pending | |

---

## 7. Version History

| Version | Change |
|---------|--------|
| v0.1 | Initial scaffold — code analysis complete, protocol defined |

---

*Report created: 2026-08-07. Campaign: rc003_lex_v1.0.*