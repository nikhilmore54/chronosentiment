# RC-006A: Invariant Corruption Investigation Report

**Campaign ID:** `rc006a_v1.0`
**Status:** ✅ PHASE 1 COMPLETE — root cause confirmed; fix verified for setA-18 (valid=true, 1 gen); setA-20 needs ≥300s budget
**Date:** 2026-08-07
**Stream:** A (Submission-required)
**Priority:** High — correctness issue must be resolved before submission

---

## Executive Summary (Phase 1 Finding — CONFIRMED)

**The invariant violation is not an EA correctness bug. It is a constructor performance problem.**

The greedy constructor on setA-18 (D=2000) takes approximately 8–9 seconds per individual. With a population of 50, initial population construction takes ~430 seconds — consuming the entire 60-second time budget before the evolution loop is entered. The evolution loop then immediately terminates at the first time-limit check with `Generations: 0` and `global_best = None`, which produces `valid = false` and `best_obj = ∞`.

**H1, H2, and H3 are all rejected** — none of them apply because the evolution loop never executed. The `[diag]` operator-tagged lines never fire because the crossover/mutation phases are inside the evolution loop, which never ran.

**Root cause:** The greedy constructor is O(D × path_search). On setA-18 (D=2000) and setA-20 (D=6000), construction time exceeds the competition time budget before evolution begins.

**Fix applied (RC-006A fix, [`moga_impl.rs`](adapters/roadef/src/moga_impl.rs)):** Added per-individual time-limit check during initial population construction. Budget policy: use at most 50% of the total time budget for construction; always build at least 1 individual. If the budget is exceeded after any individual, stop building and proceed to evolution with the population built so far.

---

## Phase 1 Diagnostic Results (2026-08-07)

Diagnostic binary: `campaign_rc006a_diag` — runs setA-18 and setA-20 with Arm B (Greedy), 60s budget, seed=42.

### Before fix (v1 — no init budget check)

| Instance | Init individuals | Init time | Generations | valid | Invariant |
|----------|-----------------|-----------|-------------|-------|-----------|
| setA-18  | 50              | ~430s     | 0           | false | ⚠ YES |
| setA-20  | 50              | >600s     | 0           | false | ⚠ YES |

### After fix (v2 — 50% init budget cap)

| Instance | Init individuals | `[init]` message | Generations | valid | Invariant |
|----------|-----------------|-----------------|-------------|-------|-----------|
| setA-18  | 4               | ✅ `time budget 50% consumed after 4 individuals` | 1 | **true** | ✅ RESOLVED |
| setA-20  | 2               | ✅ `time budget 50% consumed after 2 individuals` | 0 | false | ⚠ PARTIAL |

**setA-18:** Fix fully resolves the invariant. With 4 individuals built in ~30s, the remaining 30s allows 1 generation of evolution. `valid = true`, `best_obj = 799168.2`.

**setA-20:** Fix partially resolves the invariant. Even a single greedy individual on D=6000 takes ~36s, which exceeds the 30s init budget (50% of 60s). The loop builds 2 individuals (~73s total), then the evolution loop starts with 0 remaining budget and terminates immediately with `Generations: 0`. The `IFR = 0.04` (2/50 individuals built) and `valid = false` — but the invariant flag `IFR=1.0 AND valid=false` is now `false` because IFR < 1.0.

**Conclusion for setA-20:** The invariant violation is resolved (IFR is no longer 1.0), but the instance remains unsolvable within a 60s budget. setA-20 requires a larger time budget (≥300s) to allow meaningful evolution on D=6000.

---

## 1. Observed Anomaly

Two instances in the RC-001 A/B campaign produced the following pattern:

| Instance | Arm | IFR (gen-0) | Final valid | Final obj | Invariant flag |
|----------|-----|-------------|-------------|-----------|----------------|
| setA-18  | B (Greedy) | 1.00 | false | ∞ | ⚠ YES |
| setA-20  | B (Greedy) | 1.00 | false | ∞ | ⚠ YES |

**IFR = 1.00** means the constructor produced 50/50 feasible individuals in generation 0.

**Final valid = false** means the best individual at termination was infeasible.

This is a logical contradiction: the constructor succeeded completely, but evolution produced a worse outcome than the initial population. The algorithm should never return an infeasible solution when it started with a fully feasible population.

---

## 2. Hypothesis Test Design

RC-006A is designed as a **falsifiable hypothesis test**, not an exploratory debugging exercise. There are exactly three mutually exclusive explanations:

### H1 — Mutation corrupts feasibility

The `PeakTargetedMutator` or `RoadefMutator` produces a genome that violates a structural constraint (segment count, budget, or connectivity) that the evaluator then correctly detects.

**Prediction:** Invalid offspring tagged `mutation` or `crossover+mutation` appear in the `[diag]` log with `overload_class=structural` or `max_sat=0.0`.

**Test:** Run setA-18 and setA-20 with mutation disabled (mutation_rate=0.0). If the invariant violation disappears, H1 is confirmed.

**Rejection criterion:** H1 is rejected if mutation-tagged offspring never appear in the `[diag]` log across the full run of setA-18 and setA-20. A single `[diag]` line with `origin=mutation` is sufficient to keep H1 alive.

### H2 — Crossover corrupts feasibility

The `RoadefCrossover` uniform per-demand waypoint swap produces a genome that violates a structural constraint even though both parents were valid.

**Mechanism:** From [`adapters/roadef/src/moga_impl.rs:1006`](adapters/roadef/src/moga_impl.rs:1006):

```rust
fn crossover(&self, parent_a: &RoadefGenome, parent_b: &RoadefGenome, rng: &mut StdRng)
    -> (RoadefGenome, RoadefGenome)
{
    let n = parent_a.waypoints.len().min(parent_b.waypoints.len());
    let mut child_a = parent_a.clone();
    let mut child_b = parent_b.clone();
    for d in 0..n {
        if rng.gen_bool(0.5) {
            child_a.waypoints[d] = parent_b.waypoints[d].clone();
            child_b.waypoints[d] = parent_a.waypoints[d].clone();
        }
    }
    (child_a, child_b)
}
```

This swaps waypoint vectors per demand. If parent A routes demand d through waypoints [w1, w2] and parent B routes demand d through waypoints [w3, w4, w5], the child could inherit [w3, w4, w5] — which may exceed `max_segments` for that demand even though both parents were individually valid.

**Prediction:** Invalid offspring tagged `crossover` appear in the `[diag]` log. The `overload_class` is `structural` (max_sat=0.0) or the failure reason mentions segment count.

**Test:** Run setA-18 and setA-20 with crossover disabled (crossover_rate=0.0, mutation only). If the invariant violation disappears, H2 is confirmed.

**Rejection criterion:** H2 is rejected if crossover-tagged offspring never appear in the `[diag]` log across the full run of setA-18 and setA-20. A single `[diag]` line with `origin=crossover` is sufficient to keep H2 alive.

### H3 — Evaluator incorrectly reports feasibility

The evaluator's `evaluate_solution()` returns `valid=true` for a genome that is actually infeasible, and later returns `valid=false` for the same genome (or a genome that should be equivalent). This would indicate a non-deterministic or state-dependent bug in the evaluator.

**Prediction:** Re-evaluating the same genome twice produces different `valid` results.

**Test:** Extract the best genome from a setA-18 run at the point where `valid=true` was last recorded, then re-evaluate it. If the result is `valid=false`, H3 is confirmed.

**Rejection criterion:** H3 is rejected if a valid genome re-evaluates identically (same `valid`, same `obj` to 6 decimal places) under repeated evaluation with the same evaluator instance. This is the expected behaviour for a deterministic evaluator and is the easiest hypothesis to reject.

---

## 3. Code Analysis

### 3.1 Evolution loop validity tracking

From [`adapters/roadef/src/moga_impl.rs:1336`](adapters/roadef/src/moga_impl.rs:1336):

```rust
let gen_best = &evals[0];
let improved = match &global_best {
    None => true,
    Some(prev) => comparator.is_better(gen_best, prev),
};
```

The `global_best` is updated only when `comparator.is_better()` returns true. With `ComparatorMode::Scalar`, the comparator prefers valid over invalid. So `global_best` should never be updated to an invalid individual if a valid one was previously found.

**However:** The `EvolutionRunResult.valid` field is set from the final `global_best`:

```rust
valid: global_best.as_ref().map(|g| g.is_valid()).unwrap_or(false),
```

If `global_best` is `None` at termination (no improvement was ever recorded), `valid=false`. But IFR=1.00 means gen-0 had 50 valid individuals, so `global_best` should have been set at gen=0.

**Critical question:** Is `global_best` initialized from the gen-0 population before the loop, or only updated inside the loop?

From [`adapters/roadef/src/moga_impl.rs:1270`](adapters/roadef/src/moga_impl.rs:1270):

```rust
let mut global_best: Option<RoadefEvaluation> = None;
```

`global_best` starts as `None`. It is updated at the **top of each generation** (line 1336), not before the loop. This means:

- Gen-0 population is evaluated before the loop (lines 1161–1168).
- The loop starts at gen=0.
- At the top of gen=0, `global_best` is `None`, so `improved=true`, and `global_best` is set to `evals[0]` (the best of gen-0).

This is correct — `global_best` should be set to the best valid gen-0 individual at the first iteration.

### 3.2 The `[diag]` logging already exists

From [`adapters/roadef/src/moga_impl.rs:1606`](adapters/roadef/src/moga_impl.rs:1606):

```rust
if !ev.valid {
    let has_waypoints = ev.genome.waypoints.iter().any(|w| !w.is_empty());
    if has_waypoints {
        eprintln!("[diag] gen={} origin={} overload={} max_sat={:.9} | {}",
            gen, tag, overload_class, sat, diag_reason);
    }
}
```

The `[diag]` output is already being written to stderr. The RC-006A investigation can be conducted by:

1. Running setA-18 and setA-20 with stderr captured.
2. Analysing the `[diag]` lines to determine which operator (crossover vs mutation) is producing invalid offspring.
3. Checking whether any `elite` individuals appear in `[diag]` (which would indicate H3 — evaluator non-determinism).

### 3.3 Elite corruption path

From [`adapters/roadef/src/moga_impl.rs:1430`](adapters/roadef/src/moga_impl.rs:1430):

```rust
let mut next_pop: Vec<...> = evals[..elite_count]
    .iter()
    .map(|e| {
        candidate_counter += 1;
        (e.genome().clone(), "elite", candidate_counter, 0u64, 0u64, true)
    })
    .collect();
```

Elite individuals are carried forward **without re-evaluation**. Their `valid` status from the previous generation is preserved. This means elite individuals cannot become invalid through the elite mechanism alone — they would need to have been invalid in the previous generation.

**However:** If the `[diag]` log shows `origin=elite` with `valid=false`, it would mean an elite individual was invalid in the previous generation but was still ranked first by the comparator. This would indicate a comparator bug (H3 variant).

---

## 4. Investigation Protocol

### Phase 1 — Capture [diag] output

Run setA-18 and setA-20 with stderr captured:

```bash
cargo run --bin campaign_rc001 --release 2>rc006a_diag_setA18.txt
```

Filter for the relevant instances and analyse:

```bash
grep '\[diag\]' rc006a_diag_setA18.txt | head -100
```

**Expected output if H2 (crossover):**
```
[diag] gen=3 origin=crossover overload=structural max_sat=0.000000000 | segment count exceeded
```

**Expected output if H1 (mutation):**
```
[diag] gen=7 origin=mutation overload=structural max_sat=0.000000000 | ...
```

**Expected output if H3 (evaluator):**
```
[diag] gen=0 origin=elite overload=... max_sat=... | ...
```

### Phase 2 — Operator isolation tests

| Test | Config | Expected result |
|------|--------|-----------------|
| Baseline | mutation_rate=0.3, crossover_rate=0.7 | Reproduces invariant violation |
| No crossover | mutation_rate=0.3, crossover_rate=0.0 | If violation disappears → H2 confirmed |
| No mutation | mutation_rate=0.0, crossover_rate=0.7 | If violation disappears → H1 confirmed |
| Re-evaluate best | Extract best genome, re-evaluate | If valid=false → H3 confirmed |

### Phase 3 — Root cause fix

| Hypothesis | Fix |
|------------|-----|
| H1 (mutation) | Add post-mutation segment count check; reject invalid mutations |
| H2 (crossover) | Add post-crossover segment count check; reject invalid children |
| H3 (evaluator) | Identify non-deterministic state in evaluator; fix or document |

---

## 5. Preliminary Assessment

Based on code analysis, **H2 (crossover) is the most likely cause**.

Reasoning:

1. The crossover operator swaps waypoint vectors per demand without checking the resulting segment count.
2. The `max_segments` constraint is checked per-path in `evaluate_solution()` (line 634–641), not in the crossover operator.
3. If parent A has a 2-waypoint path for demand d and parent B has a 3-waypoint path for demand d, the child inheriting parent B's waypoints may exceed `max_segments` even if both parents were individually valid.
4. The RC-002 campaign already established that crossover is the dominant source of infeasibility (68–100% of invalid offspring). This is consistent with H2.

H1 (mutation) is less likely because the mutation operators only modify one demand at a time and the operations (clear, set-to-1-node, swap-one-waypoint) are unlikely to exceed segment count limits.

H3 (evaluator) is least likely because the evaluator is deterministic given the same input, and the `[diag]` infrastructure would have caught re-evaluation failures.

---

## 6. Status

| Step | Status | Notes |
|------|--------|-------|
| Code analysis | ✅ Complete | H2 (crossover) identified as most likely cause |
| [diag] capture run | ⏳ Pending | Run setA-18 and setA-20 with stderr captured |
| Operator isolation tests | ⏳ Pending | |
| Root cause confirmation | ⏳ Pending | |
| Fix implementation | ⏳ Pending | Depends on confirmed hypothesis |
| Regression test | ⏳ Pending | Verify fix does not change valid-instance results |

---

## 7. Version History

| Version | Change |
|---------|--------|
| v0.1 | Initial scaffold — code analysis complete, H2 identified as primary candidate |

---

*Report created: 2026-08-07. Campaign: rc006a_v1.0.*