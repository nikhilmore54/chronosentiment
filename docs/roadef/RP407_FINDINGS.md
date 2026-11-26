# RP-407 Findings — Feasibility and Basin Collapse

**Status:** FROZEN  
**Telemetry source:** `/tmp/rp410_telemetry_v2`  
**Campaign:** 20 setA instances, single seed, adaptive time budget (30–300s)  
**Analysis script:** `scripts/rp407_basin_analysis.py`  

---

## 1. Executive Summary

The campaign reveals that **5 of 20 instances produce no valid solution** within the time budget:

| Instance | Generations | Peak Valid Count | Validity Collapse Gen | Outcome |
|----------|------------:|-----------------:|----------------------:|---------|
| setA-02 | 21 | 0 | 0 | Never establishes viable population |
| setA-07 | 21 | 0 | 0 | Never establishes viable population |
| setA-16 | 5 | 0 | 0 | Never establishes viable population |
| setA-19 | 3 | 0 | 0 | Never establishes viable population |
| setA-20 | 2 | 0 | 0 | Never establishes viable population |

The critical observation is that `validity_collapse_gen = 0` and `peak_valid_count = 0` for all five instances. The solver does not **gradually lose** validity — it **never establishes** a viable population. These are two completely different phenomena, and the distinction determines what intervention is appropriate.

---

## 2. Two Distinct Failure Modes

### Type I — Initial Feasibility Failure

The five invalid instances show `valid_count = 0` from generation 0. Evolution never meaningfully starts. The construction heuristic (random initialisation) cannot produce a single valid genome for these instances within the time budget.

This is a **construction problem**, not an evolutionary problem. The research question is:

> Why can't the constructor generate a valid genome for these instances?

Relevant sub-questions:
- What constraint is violated in all initialised genomes?
- Is the feasibility region sparse in the search space, or is the construction heuristic systematically biased away from it?
- Does the repair operator fail, or is it not invoked?

### Type II — Evolutionary Collapse

This would look like:

```
generation 0:  valid = 50
generation 20: valid = 40
generation 50: valid = 3
generation 60: valid = 0
```

**This pattern is not observed in the current campaign.** No instance shows a valid population that subsequently collapses to zero. The current telemetry cannot confirm or refute whether evolutionary collapse occurs, because the instances that eventually become invalid were already invalid at generation 0.

The distinction matters: Type I requires fixing the construction phase. Type II would require fixing selection, crossover, or mutation. Conflating them leads to wrong interventions.

---

## 3. Revised Instance Classification

The original RP-407 framing classified instances as "collapsed basin" vs "shape competition" based on RP-406C lexicographic analysis. The campaign data requires a revised classification:

**Type I — Initial feasibility failure (construction problem):**
- setA-02, setA-07 — small instances (50–100 nodes), 21 generations, peak valid count = 0
- setA-16, setA-19, setA-20 — large instances (250–400 nodes), 2–5 generations, peak valid count = 0

Note: For setA-16, setA-19, setA-20, the invalidity may be **throughput-limited** rather than structural. With only 2–5 generations, the search has not had time to discover valid solutions even if they are reachable. Distinguishing structural invalidity from throughput-limited invalidity requires either a much larger time budget or per-generation evaluation cost profiling.

**Valid but low-SDI (originally "collapsed basin"):**
- setA-04, setA-05, setA-06, setA-08 — all produce valid solutions; SDI 0.43–1.78

**Valid and high-SDI (originally "shape competition"):**
- setA-01, setA-03, setA-09 through setA-15, setA-17, setA-18 — SDI 1.24–2.64

---

## 4. Basin Analysis Results

### 4.1 Validity Collapse

| Category | Instances | Validity Collapse Rate | Mean Zero-Valid Fraction |
|----------|----------:|-----------------------:|-------------------------:|
| Collapsed basin (original) | 6 | 33.3% | 33.3% |
| Shape competition (original) | 11 | 9.1% | 9.1% |

Collapsed-basin instances show a 3.7× higher validity collapse rate. However, in all cases where collapse occurs, it occurs at generation 0 — meaning it is construction failure, not evolutionary collapse.

### 4.2 The Diversity Collapse Interpretation Requires Caution

The analysis reports `unique_fitness_count = 1` for all instances where `valid_count = 0`. This does **not** demonstrate diversity collapse in the genetic sense.

When all individuals in the population are invalid, they may all receive the same penalty score (e.g., `obj = inf`). In that case, `unique_fitness_count = 1` reflects identical fitness values, not identical genomes. The genomes themselves may be highly diverse — the fitness function simply cannot distinguish them because all are infeasible.

Demonstrating true diversity collapse would require:
- Genome Hamming distance between individuals
- Routing path overlap metrics
- Edge utilisation similarity

None of these are currently instrumented. The `unique_fitness_count = 1` finding for invalid instances is therefore **not evidence of premature convergence**.

### 4.3 SDI Separation

| Category | Mean Final SDI |
|----------|---------------:|
| Collapsed basin (original) | 0.883 |
| Shape competition (original) | 1.785 |

The SDI gap is the clearest quantitative signal separating the two categories, and it is consistent with RP-406C. Collapsed-basin instances that do find valid solutions converge to solutions with more uniform arc saturation. This is a reproducible observation across two independent datasets (RP-406C lexicographic analysis and RP-410A telemetry).

### 4.4 Initial Feasibility Rate — A New Metric

The campaign data motivates a new metric: **Initial Feasibility Rate** = number of valid genomes at generation 0 / population size. This directly evaluates the construction algorithm, independent of evolutionary dynamics.

The current telemetry does not record this directly. `peak_valid_count` records the **maximum** valid count observed across the entire run — not necessarily at generation 0. For example, if a run produces 10 valid individuals at generation 0, then 25 at generation 1, then 40 at generation 2, `peak_valid_count = 40` describes the best evolutionary outcome, not the constructor quality.

For the five instances where `peak_valid_count = 0`, the distinction does not matter — no valid individual was ever found. But for instances like setA-17 (`peak_valid_count = 32`) or setA-18 (`peak_valid_count = 30`), the peak may have been reached at generation 5 or 10, not generation 0. Using `peak_valid_count` as a proxy for Initial Feasibility Rate is therefore not strictly valid.

**Recommendation:** Add `generation0_valid_count` as an explicit telemetry field, recorded before any selection or variation occurs. This is the correct metric for evaluating constructor quality and is a prerequisite for distinguishing Type I failure (construction) from Type II failure (evolution).

The observed `peak_valid_count` values are still informative as a lower bound on evolutionary reach:

| Instance | Peak Valid Count | Notes |
|----------|----------------:|-------|
| setA-01 | 50 | Full population valid at some point |
| setA-03 | 50 | Full population valid at some point |
| setA-17 | 32 | Partial validity — generation unknown |
| setA-18 | 30 | Partial validity — generation unknown |
| setA-02 | 0 | Never valid — construction failure confirmed |
| setA-07 | 0 | Never valid — construction failure confirmed |

---

## 5. What the Data Supports

The following conclusions are directly supported by the telemetry:

1. **Five instances never establish a viable population.** `peak_valid_count = 0` for setA-02, setA-07, setA-16, setA-19, setA-20.

2. **Validity collapse occurs at generation 0, not during evolution.** No instance shows a valid population that subsequently collapses.

3. **Low SDI is consistently associated with collapsed-basin instances.** Mean SDI 0.883 vs 1.785, consistent with RP-406C.

4. **The two failure modes (Type I and Type II) are distinct.** The current campaign provides evidence only for Type I.

The following conclusions are **not** supported by the current telemetry:

- That premature convergence causes collapsed basins (not observed)
- That diversity collapse precedes validity collapse (not observed; `unique_fitness=1` with `valid_count=0` reflects identical penalties, not identical genomes)
- That random immigrants, diversity preservation, or operator balance would address the observed failures (these address Type II; the observed failures are Type I)

---

## 6. Recommended Research Priorities

### Priority 1 — Construction Diagnostics

Instrument the initialisation phase to record:
- `valid_count` at generation 0 (before any selection or variation)
- Constraint violation type for each invalid genome (which constraint is violated, by how much)
- Whether the repair operator is invoked and whether it succeeds

This directly evaluates the construction algorithm and determines whether the feasibility failure is due to the initialisation heuristic, the repair operator, or the constraint structure.

### Priority 2 — Throughput Characterisation (RP-411)

For setA-16, setA-19, setA-20, determine whether invalidity is structural or throughput-limited. Profile per-generation evaluation cost to determine whether 10× more generations are achievable within the same time budget. If they are, re-run these instances with extended budget to determine whether valid solutions emerge.

### Priority 3 — Evolutionary Dynamics Instrumentation

Only after establishing that valid populations exist initially (Priority 1) and that sufficient generations are available (Priority 2): instrument the transition from valid to invalid populations to determine whether mutation, crossover, or selection is responsible for any evolutionary collapse. This is the investigation that would confirm or refute Type II failure.

### Priority 4 — Diversification Mechanisms

Only after confirming that Type II failure (evolutionary collapse) actually occurs. Diversity preservation, random immigrants, and operator balance are appropriate responses to Type II. They are not appropriate responses to Type I (construction failure).

---

## 7. Four-Subsystem Decomposition

The cumulative evidence from RP-406C, RP-410A, and RP-407 supports viewing the solver as four distinct subsystems, each with its own failure mode and diagnostic metric:

| Subsystem | Question | Current Metric | Status |
|-----------|----------|----------------|--------|
| **Construction** | Can it generate feasible solutions? | `generation0_valid_count` (not yet recorded; `peak_valid_count` is a partial proxy) | **Active failure** — 5/20 instances |
| **Execution** | How many evolutionary cycles are achievable? | Generation rate (gens/min); per-component cost breakdown | **Active constraint** — 2–6 gens for large instances |
| **Evolution** | Which improvements are generated and accepted? | Operator fingerprints, zone distribution, SDI trajectory | **Characterised** for small/medium instances only |
| **Objective** | What does the search preferentially optimise? | Lexicographic vs scalar outcomes; Peak/Shoulder acceptance rate | **Not yet addressed** — requires RP-408 |

This decomposition makes the research programme modular: improvements can be evaluated against the subsystem they are intended to affect, rather than relying only on end-to-end benchmark scores.

**Problem 1 — Feasibility (Construction subsystem):** The primary finding of RP-407. Five instances never establish a viable population. The root cause is in the construction phase, not the evolutionary dynamics. This is a prerequisite for all other research streams.

**Problem 2 — Throughput (Execution subsystem):** Large instances complete 2–6 generations. This may be masking additional feasibility failures (setA-16, setA-19, setA-20 may be solvable with more time). Throughput characterisation (RP-411) is required before feasibility conclusions can be drawn for large instances.

**Problem 3 — Search Behaviour (Evolution + Objective subsystems):** The RP-410A findings (Peak near-absence, operator fingerprints, SDI separation) characterise search behaviour on instances where the search actually runs. These findings are valid for instances completing ≥10 generations and are not yet generalisable to large instances.

---

## 8. Data Files

- [`docs/roadef/rp407_data/RP407_BASIN_ANALYSIS_REPORT.md`](docs/roadef/rp407_data/RP407_BASIN_ANALYSIS_REPORT.md) — generated analysis report
- [`docs/roadef/rp407_data/collapse_summary.csv`](docs/roadef/rp407_data/collapse_summary.csv) — per-instance collapse event table
- `docs/roadef/rp407_data/trajectory_<instance>_seed0.csv` — per-generation trajectory for each instance

*End of RP-407 Findings*