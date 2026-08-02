# RP-401D — ECMP Oracle-Guided Path Selection Report

**Status:** Complete (binary written; awaiting execution results)  
**Date:** 2026-08-02  
**Experiment:** RP-401D (ECMP Oracle-Guided Path Selection)  
**Binary:** `adapters/roadef/src/bin/rp401d_ecmp_path_selection.rs`

---

## 1. Research Question

Does selecting paths by minimising the ECMP-oracle MLU increase (rather than
using a penalty-weighted Dijkstra) further improve solution quality on Dataset A
beyond RP-401C?

---

## 2. Design

### 2.1 Change from RP-401C

| Aspect | RP-401C | RP-401D |
|--------|---------|---------|
| Path generation | Single load-aware Dijkstra | K=5 diverse candidates (perturbed Dijkstra) |
| Path selection criterion | Lowest penalty-weighted metric | Lowest post-assignment MLU (oracle-evaluated) |
| Oracle calls per demand | 1 (saturation update) | K+1 (K evaluations + 1 commit update) |
| Total oracle calls | O(D) | O(D × K) |

### 2.2 Candidate Generation Strategy

For each demand, K=5 candidates are generated:

| Candidate | Strategy |
|-----------|----------|
| 0 | Unperturbed shortest path (metric only) |
| 1 | Load-aware: multiply metrics by ECMP saturation penalty |
| 2 | Inflate highest-saturation link (force detour) |
| 3 | Inflate top-2 saturation links |
| 4 | Inflate top-3 saturation links |

This produces diverse paths that explore different parts of the network. The
inflation strategy is a lightweight approximation of Yen's K-shortest paths
algorithm, avoiding the O(K × D × Dijkstra) cost of the full algorithm.

### 2.3 Selection Criterion

For each candidate waypoint set `w`:

```
trial_solution = partial_solution + {demand d, waypoints w}
loads = evaluator.compute_loads(time_slot, trial_solution)
score = loads.mlu
```

The candidate with the lowest `score` is selected. This directly minimises
the maximum link utilisation after each demand assignment, which is the
dominant term in the ROADEF objective function.

### 2.4 Complexity

- RP-401C: O(D² × Dijkstra) construction
- RP-401D: O(D × K × D × Dijkstra) = O(K × D² × Dijkstra)

For K=5, D=200, |V|=128: approximately 5× slower than RP-401C. Estimated
runtime < 30s per instance. Acceptable for research purposes.

---

## 3. Key Design Decisions

### 3.1 Why MLU as Selection Criterion?

The ROADEF objective is `sum_t(MLU_t + inv_load_cost_t)`. MLU is the dominant
term for most instances (it grows linearly with saturation, while inv_load_cost
grows super-linearly only above 80% saturation). Minimising MLU at each step
is a greedy approximation of minimising the final objective.

An alternative would be to minimise `mlu + inv_load_cost` directly. This is
implemented in the oracle (`compute_loads()` returns both). The current binary
uses MLU only for simplicity; a future variant could use the full objective.

### 3.2 Why K=5?

K=5 provides a good balance between diversity and runtime:
- K=1: equivalent to RP-401C (single path, no oracle selection)
- K=5: covers the main alternatives (shortest, load-aware, 3 detour variants)
- K=10: diminishing returns; most additional candidates are duplicates

### 3.3 Shared-Path Strategy

RP-401D retains the shared-path strategy (same waypoints for t=0 and t=1),
guaranteeing zero budget cost. The oracle is called with `time_slot=0` during
construction. A future experiment (RP-402) will explore t=1-specific paths
within the budget constraint.

---

## 4. Expected Results

RP-401D is expected to outperform RP-401C on instances where:
- Multiple paths have similar penalty-weighted costs (the Dijkstra tie-breaking
  is arbitrary; oracle selection picks the genuinely better one)
- The load-aware Dijkstra penalty function is poorly calibrated for a specific
  instance's topology (oracle selection is topology-agnostic)

RP-401D may underperform RP-401C on instances where:
- The greedy MLU-minimisation is myopic (a locally good path blocks a globally
  better assignment for later demands)
- The K=5 candidates do not include the optimal path

---

## 5. Evidence Record

| Field | Value |
|-------|-------|
| Experiment | RP-401D |
| Status | Binary written; execution pending |
| Research Question | Does oracle-guided path selection outperform penalty-guided selection? |
| Baseline | RP-401C (`rp401c_ecmp_construction.rs`) |
| Metric | Objective value per instance vs RP-401C and empty |
| Result | Pending execution of `rp401d_ecmp_path_selection` binary |
| Runtime | Estimated < 30s per instance (O(K × D²) oracle calls, K=5) |
| Statistical Confidence | Deterministic — greedy construction is deterministic |
| Platform Impact | If confirmed: oracle-guided selection becomes standard for RP-402+ |
| Decision | Pending results; if RP-401D ≥ RP-401C, proceed to RP-402 with RP-401D as base |
| Key Files | `adapters/roadef/src/bin/rp401d_ecmp_path_selection.rs` |

---

## 6. RP-401 Series Summary

The four-stage RP-401 series establishes the ECMP oracle as the foundation
for all subsequent ROADEF research:

| Stage | Finding | Artefact |
|-------|---------|---------|
| RP-401A | `compute_loads()` is a correct ECMP oracle (verified vs C++ checker) | `rp401a_ecmp_oracle_verification.md` |
| RP-401B | Heuristic overestimates load by (k-1)/k; penalty inflation 1–1000× | `rp401b_load_divergence_report.md` |
| RP-401C | ECMP-oracle construction: O(D²) oracle calls, ECMP-accurate saturation | `rp401c_ecmp_construction.rs` |
| RP-401D | Oracle-guided selection: K=5 candidates, MLU-minimising choice | `rp401d_ecmp_path_selection.rs` |

The best-performing binary from RP-401C/D becomes the baseline for RP-402
(budget-aware t=1 adaptation).