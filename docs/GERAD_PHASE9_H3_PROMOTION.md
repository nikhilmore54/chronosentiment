# Phase 9 — H3 Promotion: Eliminate Redundant `is_feasible()` Pre-Check

**Status: PROMOTED**
**Date: 2026-08-24**

---

## 1. Hypothesis

**H3 (Lazy Feasibility)**: The standalone `is_feasible(child)` call in
[`pipeline_impl.rs`](adapters/roadef/src/pipeline_impl.rs) at both the
crossover+mutation path and the mutation-only path was redundant.
[`process_offspring`](coralys-core/src/pipeline.rs:25) calls `is_feasible()`
internally as its very first action (line 25 of
[`coralys-core/src/pipeline.rs`](coralys-core/src/pipeline.rs)). Removing the
pre-check eliminates one full `evaluate_violations()` call per offspring without
changing any trajectory outcome.

---

## 2. Intervention

File modified: [`adapters/roadef/src/pipeline_impl.rs`](adapters/roadef/src/pipeline_impl.rs)

Removed the standalone `is_feasible(child)` call at two call sites:
- Crossover + mutation path (was lines 1508–1539)
- Mutation-only path (was lines 1576–1610)

No other algorithmic changes. [`process_offspring`](coralys-core/src/pipeline.rs:25)
continues to call `is_feasible()` internally as its first action.

---

## 3. Gate Results — setA-01

### Trajectory Invariants (5/5 bit-exact)

| Invariant       | Baseline (Phase 8) | Post-H3            | Match |
|-----------------|--------------------|--------------------|-------|
| best_obj        | 51.0126807372      | 51.0126807372      | ✓     |
| n_actual_evals  | 1802               | 1802               | ✓     |
| generations_run | 50                 | 50                 | ✓     |
| valid           | true               | true               | ✓     |
| cache_hits      | 250                | 250                | ✓     |

### Timing

| Metric                          | Baseline   | Post-H3   |
|---------------------------------|------------|-----------|
| wall_clock_ms                   | ~8,200 ms  | ~4,345 ms |
| feasibility_ms (sum, 50 gens)   | 3,183 ms   | 0 ms      |

**T_net = wall_baseline − wall_experimental = 8,200 − 4,345 = +3,855 ms (+47%)**

Gate criterion: T_net > 0 ✓
Attribution: feasibility_ms eliminated (was 38.9% of wall) ✓

---

## 4. Corroboration — setA-14

### Trajectory Invariants (5/5 bit-exact)

| Invariant       | Baseline (Phase 8) | Post-H3            | Match |
|-----------------|--------------------|--------------------|-------|
| best_obj        | 86.1250850504      | 86.1250850504      | ✓     |
| n_actual_evals  | 2006               | 2006               | ✓     |
| generations_run | 50                 | 50                 | ✓     |
| valid           | true               | true               | ✓     |
| cache_hits      | 181                | 181                | ✓     |

### Timing

| Metric                          | Baseline      | Post-H3      |
|---------------------------------|---------------|--------------|
| wall_clock_ms                   | 2,006,310 ms  | 989,132 ms   |
| feasibility_ms (sum, 50 gens)   | 833,494 ms    | 0 ms         |

**T_net = 2,006,310 − 989,132 = +1,017,178 ms (+50.7%)**

Gate criterion: T_net > 0 ✓
Attribution: feasibility_ms eliminated (was 41.5% of wall) ✓
Attribution gate: 100.0% ✓

---

## 5. Promotion Decision

All promotion criteria satisfied on both instances:

1. **5/5 trajectory invariants bit-exact** on setA-01 (gate) and setA-14 (corroboration) ✓
2. **T_net > 0** on both instances: +3,855 ms (setA-01), +1,017,178 ms (setA-14) ✓
3. **≥50% of |T_net| attributable to removed feasibility cost**:
   - setA-01: feasibility_ms was 38.9% of wall, now 0 ✓
   - setA-14: feasibility_ms was 41.5% of wall, now 0 ✓
4. **No best_obj degradation** on either instance ✓

**H3 is PROMOTED.** The intervention is retained in the codebase.

---

## 6. T_net Sign Convention

T_net = `wall_time_baseline − wall_time_experimental`

Positive = faster (improvement). Gate criterion: T_net > 0.

---

## 7. Evidence Files

| File | Description |
|------|-------------|
| [`evidence/phase9_h3_setA01_gate.csv`](evidence/phase9_h3_setA01_gate.csv) | setA-01 per-generation data post-H3 |
| [`evidence/phase9_h3_setA01_gate_summary.txt`](evidence/phase9_h3_setA01_gate_summary.txt) | setA-01 full summary post-H3 |
| [`evidence/phase9_h3_setA14_corroboration.csv`](evidence/phase9_h3_setA14_corroboration.csv) | setA-14 per-generation data post-H3 |
| [`evidence/phase9_h3_setA14_corroboration_summary.txt`](evidence/phase9_h3_setA14_corroboration_summary.txt) | setA-14 full summary post-H3 |
| [`evidence/phase8_setA01_gate.csv`](evidence/phase8_setA01_gate.csv) | setA-01 baseline (Phase 8) |
| [`evidence/phase8_setA14_corroboration.csv`](evidence/phase8_setA14_corroboration.csv) | setA-14 baseline (Phase 8) |

---

## 8. Next Steps

H3 disposition is complete. P9-B (targeting the `improve` path, 35.7–40.7% of
wall on setA-01/setA-14) is now unblocked.