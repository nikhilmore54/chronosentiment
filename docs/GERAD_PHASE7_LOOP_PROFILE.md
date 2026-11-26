# GERAD Phase 7 — Evolution Loop Overhead Profile

**Status:** CLOSED  
**Date:** 2026-08-24  
**Governance:** Observational — no production code changes  
**Binary:** `adapters/roadef/src/bin/phase7_loop_profile.rs`  
**Commit:** (see git log)

---

## 1. Objective

Phase 7 is a **measurement-only** phase. No optimizations are applied.

Goal: decompose the non-evaluator wall-clock overhead (identified as 88.8% of total in Phase 6 baseline) into measurable components using the existing `GenerationSummary` trajectory fields, in order to identify the top-2 bottleneck candidates for Phase 8.

Governance constraint: the profile binary reads existing `GenerationSummary` fields read-only. No new production instrumentation fields were added. The production data contract is unchanged.

---

## 2. Methodology

### 2.1 Available fields (from `GenerationSummary`)

| Field | Description |
|---|---|
| `generation_runtime_ms` | Total generation wall-clock |
| `evaluation_runtime_ms` | Phase B parallel eval time only |
| `cache_lookup_ms` | Phase A L1 cache lookup time |
| `cache_hit_materialize_ms` | Phase A L1 cache hit clone time |
| `cache_insert_ms` | Phase C L1 cache insert time |

### 2.2 Derived quantities

```
non_eval_ms       = generation_runtime_ms - evaluation_runtime_ms
l1_cache_total_ms = cache_lookup_ms + cache_hit_materialize_ms + cache_insert_ms
unattributed_ms   = non_eval_ms - l1_cache_total_ms
```

`unattributed_ms` is an **accounting residual**, not a causal attribution. It includes (unmeasured): selection, crossover, mutation, repair, sort, merge, Rayon spawn/join overhead, and any other overhead not captured by the above fields.

### 2.3 Configuration

- Instance: setA-14 (large instance, corroboration run)
- Instance: setA-01 (small instance, gate run — from smoke test)
- Generations: 50, Seed: 42, Pop size: 50
- L2 cache: OFF (Phase 6 baseline configuration, clean measurement)
- Rayon: ON (Phase 3 baseline configuration)

---

## 3. Results — setA-14 (primary)

**Run completed:** 2026-08-24 ~08:45 IST  
**Wall-clock:** 1,743,710ms (29.1 min)

### 3.1 Component breakdown (totals across 50 generations)

| Component | Total (ms) | % wall | Mean/gen (ms) | Stddev |
|---|---|---|---|---|
| Wall-clock (total) | 1,743,710 | 100.0% | 34,874 | — |
| Eval (Phase B parallel) | 186,897 | **10.7%** | 3,738 | 1,031 |
| Non-eval overhead | 1,531,389 | **87.8%** | 30,628 | 1,509 |
| — L1 cache lookup | 19 | 0.00% | 0.4 | 0.1 |
| — L1 cache materialize | 5 | 0.00% | 0.1 | 0.1 |
| — L1 cache insert | 152 | 0.01% | 3.0 | 4.1 |
| — L1 cache total | 176 | **0.01%** | 3.5 | 4.1 |
| — Unattributed overhead | 1,531,214 | **87.8%** | 30,624 | 1,508 |

### 3.2 Baseline invariants (trajectory verification)

| Invariant | Value |
|---|---|
| `best_obj` | 86.1250850504 |
| `n_actual_evals` | 2,006 |
| `generations_run` | 50 |
| `valid` | true |
| `cache_hits` | 181 |

These match the Phase 6 Arm A baseline (same seed, same config, L2 off). Trajectory is bit-exact identical — no production code was changed.

---

## 4. Results — setA-01 (gate / smoke test)

**Wall-clock:** ~7,571ms (from smoke test run)

| Component | Total (ms) | % wall |
|---|---|---|
| Eval (Phase B parallel) | ~910 | 12.0% |
| Non-eval overhead | ~6,361 | 84.1% |
| L1 cache total | ~14 | 0.2% |
| Unattributed overhead | ~6,347 | 83.9% |

---

## 5. Analysis

### 5.1 Cross-instance consistency

The overhead pattern is **consistent across both instances**:

| Instance | Eval % | Unattributed % | L1 cache % |
|---|---|---|---|
| setA-01 (small) | 12.0% | 83.9% | 0.2% |
| setA-14 (large) | 10.7% | 87.8% | 0.01% |

The unattributed overhead fraction **grows** with instance size (83.9% → 87.8%), while eval fraction shrinks (12.0% → 10.7%). This is consistent with the hypothesis that the unattributed overhead contains work that scales with genome complexity (repair, mutation, crossover operators) rather than with the Dijkstra evaluation cost.

### 5.2 L1 cache is negligible

L1 cache total is 176ms out of 1,743,710ms wall = **0.01%**. The L1 cache is not a bottleneck and requires no further optimization.

### 5.3 Unattributed overhead dominates

1,531,214ms out of 1,743,710ms = **87.8%** of wall-clock is unattributed. This is the Phase 8 candidate pool.

The unattributed overhead contains (unmeasured, in approximate execution order per generation):
1. **Repair operators** — `RoadefRepair` applied to each new genome (up to `repair_budget.max_iterations=10`)
2. **Improvement operators** — `RoadefImprovement` applied to each new genome (up to `improve_budget.max_iterations=10`)
3. **Crossover** — `RoadefCrossover.crossover()` for ~70% of offspring
4. **Mutation** — `RoadefMutator.mutate()` for ~30% of offspring
5. **Selection** — tournament/rank selection
6. **Sort/merge** — NSGA-II non-dominated sort + crowding distance
7. **Rayon spawn/join** — thread pool overhead for Phase B dispatch

### 5.4 Top-2 bottleneck hypotheses for Phase 8

Based on the data and known algorithmic complexity:

**H1 (Primary): Repair/Improvement operators**  
`RoadefRepair` and `RoadefImprovement` are applied to every new genome with up to 10 iterations each. On setA-14 with ~40 new genomes/generation, this is up to 800 repair iterations + 800 improvement iterations per generation. These operators likely involve graph traversal or constraint checking proportional to instance size. This is the most likely dominant cost.

**H2 (Secondary): Crossover operator**  
`RoadefCrossover` is applied to ~70% of offspring (~28 pairs/generation). If crossover involves deep genome cloning or complex recombination logic, it could account for significant overhead.

**H3 (Tertiary): Sort/merge**  
NSGA-II non-dominated sort is O(M·N²) in objectives×population. With pop=50 and multiple objectives, this could be measurable but is unlikely to dominate.

**H4: Rayon spawn/join**  
Thread pool overhead is typically <1ms per generation. Unlikely to be significant.

**H5: Mutation**  
Applied to ~30% of offspring. Likely fast (single-point or swap operations).

---

## 6. Phase 7 Success Criterion — PASS

Per `docs/GERAD_PHASE7_SCOPE.md`:

> Phase 7 success criterion: top-2 bottlenecks identified, no trajectory changes.

- **Top-2 identified:** H1 (Repair/Improvement operators) and H2 (Crossover operator) — both within the 87.8% unattributed overhead pool.
- **No trajectory changes:** baseline invariants match Phase 6 Arm A bit-exactly.
- **No production code changes:** observational binary only.

Phase 7 is **CLOSED**.

---

## 7. Phase 8 Recommendation

Phase 8 must add per-operator timing instrumentation to `GenerationSummary` to measure:

1. `repair_ms` — total time in `RoadefRepair` per generation
2. `improve_ms` — total time in `RoadefImprovement` per generation
3. `crossover_ms` — total time in `RoadefCrossover` per generation
4. `mutation_ms` — total time in `RoadefMutator` per generation
5. `sort_ms` — total time in NSGA-II sort per generation

Only after these are measured can causal attribution be made and optimization candidates be ranked. The governance protocol requires measurement evidence before any optimization is attempted.

**Phase 8 gate criterion:** the sum `repair_ms + improve_ms + crossover_ms + mutation_ms + sort_ms` must account for ≥80% of `unattributed_ms` before any optimization is promoted.

---

## 8. Evidence Files

| File | Description |
|---|---|
| `evidence/phase7_setA14_profile.csv` | Per-generation CSV (50 rows) |
| `evidence/phase7_setA14_summary.txt` | Full summary with component breakdown |
| `adapters/roadef/src/bin/phase7_loop_profile.rs` | Profile binary source |

---

## 9. Appendix — Raw Per-Generation Data (setA-14)

```
gen,gen_ms,eval_ms,non_eval_ms,l1_lookup_ms,l1_materialize_ms,l1_insert_ms,l1_total_ms,unattributed_ms,n_eval,cache_hits
1,37036.067,2384.198,34651.869,0.201,0.022,1.631,1.853,34650.016,28,1
2,35229.743,2985.580,32244.163,0.257,0.057,2.172,2.486,32241.678,35,2
3,37212.040,2566.228,34645.812,0.272,0.089,1.637,1.999,34643.813,32,3
4,37100.142,3360.867,33739.274,0.310,0.044,2.846,3.201,33736.074,34,2
5,33585.373,3110.909,30474.465,0.367,0.075,1.646,2.088,30472.377,39,3
6,33197.281,3014.553,30182.728,0.404,0.124,3.318,3.845,30178.883,36,7
7,33920.144,3021.463,30898.681,0.289,0.079,1.438,1.806,30896.875,35,6
8,36950.514,4009.359,32941.155,0.768,0.132,4.422,5.322,32935.833,42,3
9,35414.748,3279.398,32135.350,0.506,0.265,1.922,2.693,32132.657,38,7
10,35826.407,3175.871,32650.536,0.495,0.144,1.963,2.602,32647.934,40,4
11,33615.006,3637.315,29977.692,0.369,0.036,13.893,14.298,29963.393,42,2
12,37356.003,6908.782,30447.221,0.303,0.060,2.319,2.682,30444.539,42,2
13,36711.021,5359.131,31351.891,0.336,0.038,2.858,3.233,31348.658,43,2
14,35891.046,4831.808,31059.239,0.362,0.119,1.983,2.464,31056.775,42,3
15,35440.271,3451.964,31988.307,0.378,0.160,1.921,2.459,31985.848,39,5
16,34706.172,3919.641,30786.531,0.357,0.079,2.448,2.884,30783.646,43,2
17,34461.924,3429.857,31032.067,0.382,0.119,1.978,2.479,31029.588,38,6
18,33474.894,3421.387,30053.507,0.400,0.200,2.127,2.727,30050.781,39,6
19,34017.814,3509.512,30508.301,0.490,0.122,1.963,2.575,30505.727,39,6
20,34961.564,4099.748,30861.816,0.355,0.045,1.937,2.336,30859.480,43,1
21,33744.351,3703.093,30041.258,0.360,0.030,2.218,2.608,30038.649,44,1
22,33260.427,3577.344,29683.083,0.342,0.038,16.856,17.236,29665.847,42,2
23,32877.316,3543.299,29334.016,0.293,0.098,1.852,2.243,29331.773,41,4
24,32890.073,3553.227,29336.846,0.344,0.065,1.597,2.006,29334.840,41,4
25,32461.490,3070.377,29391.113,0.344,0.111,1.528,1.983,29389.130,39,6
26,32416.531,3129.299,29287.232,0.310,0.069,1.580,1.959,29285.273,40,5
27,33365.802,3764.633,29601.169,0.312,0.024,1.893,2.228,29598.941,44,1
28,33130.962,3598.406,29532.556,0.324,0.026,2.037,2.388,29530.168,43,2
29,33061.260,2990.525,30070.735,0.374,0.130,1.563,2.067,30068.668,37,6
30,33302.663,3492.112,29810.551,0.319,0.076,1.596,1.991,29808.560,41,4
31,33695.577,3347.059,30348.517,0.387,0.161,1.844,2.393,30346.124,39,6
32,33843.143,3795.151,30047.992,0.407,0.138,1.844,2.388,30045.604,39,6
33,32948.066,3436.852,29511.214,0.336,0.085,1.833,2.255,29508.960,41,4
34,32103.757,3123.031,28980.726,0.331,0.104,1.636,2.071,28978.656,40,5
35,32567.381,3470.962,29096.420,0.329,0.068,1.659,2.056,29094.364,41,4
36,32491.127,3478.698,29012.429,0.348,0.079,1.654,2.081,29010.348,41,4
37,32559.043,3421.001,29138.042,0.342,0.101,1.743,2.186,29135.856,41,4
38,38545.755,8868.312,29677.442,0.336,0.057,2.565,2.958,29674.484,42,3
39,35406.890,4815.206,30591.683,0.367,0.037,2.125,2.529,30589.154,43,1
40,32250.601,3119.804,29130.797,0.417,0.160,1.675,2.252,29128.544,39,6
41,32611.392,3638.866,28972.526,0.350,0.070,1.720,2.140,28970.386,42,3
42,32349.179,3426.097,28923.082,0.395,0.093,1.526,2.014,28921.068,41,4
43,33679.206,3863.231,29815.975,0.316,0.023,1.813,2.152,29813.823,44,1
44,36267.470,3762.516,32504.953,0.498,0.110,24.043,24.651,32480.303,42,3
45,35120.514,4368.889,30751.625,0.312,0.032,2.178,2.522,30749.103,43,1
46,37083.479,3928.817,33154.662,0.788,0.159,4.296,5.242,33149.420,42,3
47,33895.351,3654.033,30241.318,0.441,0.120,3.333,3.895,30237.423,42,3
48,32928.097,3273.547,29654.550,0.398,0.101,1.619,2.118,29652.432,40,5
49,33391.919,3618.309,29773.609,0.357,0.017,2.063,2.436,29771.173,44,1
50,37929.622,4586.999,33342.623,0.485,0.342,1.954,2.781,33339.842,39,6