# RP-411 / RP-412 Baseline Report

**Status:** FROZEN — full 20-instance baseline  
**Telemetry:** `/tmp/rp411_baseline` (setA-01 through setA-20, 20 instances)  
**Analysis script:** [`scripts/rp411_412_analysis.py`](../../scripts/rp411_412_analysis.py)  
**Data directory:** [`docs/roadef/rp411_412_data/`](rp411_412_data/)  
**Generation records:** 285  
**Construction records:** 20  
**Date:** 2026-08-05

---

## Executive Summary

RP-411 and RP-412 characterise the execution throughput and construction feasibility of the
baseline ROADEF solver across all 20 setA instances.

> **Evaluation dominates execution time at 99.99% of generation wall-clock time. Throughput
> varies 1,555× across instances (setA-20: 0.4 evals/s vs setA-02: 622 evals/s). The median
> instance completes only 8.5 generations. Six instances have IFR=0% — the constructor
> produces zero feasible individuals. No instance reaches the generation limit. The search
> pipeline is doubly constrained: the evaluator is slow and the constructor is unreliable.**

---

## 1. RP-411: Execution Throughput

### 1.1 Phase Timing Breakdown (aggregate, 20 instances)

| Phase | Total ms | Fraction |
|-------|----------|----------|
| Evaluation | 3,463,372 | **99.99%** |
| Crossover | 219 | 0.01% |
| Mutation | 17 | 0.00% |
| Selection | 80 | 0.00% |
| Telemetry | 0 | 0.00% |
| Other | 184 | 0.01% |
| **Total** | **3,463,872** | 100.00% |

**Finding F-411-1:** Evaluation is the sole bottleneck. Selection, crossover, and mutation
together consume less than 0.01% of generation time. Any throughput improvement must target
the evaluator, not the operator pipeline.

### 1.2 Per-Instance Throughput

| Instance | Gens | Total Evals | Eval ms | Gens/s | Evals/s | Final Stagnation |
|----------|------|-------------|---------|--------|---------|-----------------|
| setA-01 | 51 | 2,550 | 8,495 | 5.99 | 299.5 | 20 |
| setA-02 | 21 | 1,050 | 1,681 | 12.44 | 622.0 | 20 |
| setA-03 | 61 | 3,050 | 16,181 | 3.76 | 188.1 | 20 |
| setA-04 | 14 | 700 | 29,831 | 0.47 | 23.5 | 1 |
| setA-05 | 11 | 550 | 28,562 | 0.39 | 19.3 | 1 |
| setA-06 | 13 | 650 | 132,973 | 0.10 | 4.9 | 0 |
| setA-07 | 21 | 1,050 | 75,948 | 0.28 | 13.8 | 20 |
| setA-08 | 9 | 450 | 69,910 | 0.13 | 6.4 | 0 |
| setA-09 | 10 | 500 | 74,464 | 0.13 | 6.7 | 0 |
| setA-10 | 10 | 500 | 320,433 | 0.03 | 1.6 | 0 |
| setA-11 | 9 | 450 | 194,827 | 0.05 | 2.3 | 1 |
| setA-12 | 10 | 500 | 187,006 | 0.05 | 2.7 | 0 |
| setA-13 | 7 | 350 | 307,879 | 0.02 | 1.1 | 0 |
| setA-14 | 9 | 450 | 328,244 | 0.03 | 1.4 | 0 |
| setA-15 | 9 | 450 | 321,395 | 0.03 | 1.4 | 0 |
| setA-16 | 5 | 250 | 253,204 | 0.02 | 1.0 | 4 |
| setA-17 | 2 | 100 | 257,983 | 0.01 | 0.4 | 0 |
| setA-18 | 8 | 400 | 387,687 | 0.02 | 1.0 | 2 |
| setA-19 | 3 | 150 | 236,230 | 0.01 | 0.6 | 2 |
| setA-20 | 2 | 100 | 230,438 | 0.01 | 0.4 | 1 |

**Finding F-411-2:** Throughput varies **1,555×** across instances (setA-20: 0.4 evals/s vs
setA-02: 622.0 evals/s). This is driven entirely by per-evaluation cost, which reflects
instance graph size and routing complexity.

**Finding F-411-3:** The instance set divides into two throughput tiers:
- **Fast tier** (setA-01 through setA-07): 13–622 evals/s, 11–61 generations
- **Slow tier** (setA-08 through setA-20): 0.4–6.7 evals/s, 2–13 generations

The slow tier instances are evaluation-budget-starved: the evaluator is so slow that the
search cannot explore meaningfully within the time limit.

**Finding F-411-4:** The two slowest instances (setA-17, setA-20) complete only **2
generations** — 100 total evaluations. This is effectively no evolutionary search at all.
The solver is spending its entire time budget on the initial population evaluation.

**Finding F-411-5:** setA-13 completes only 7 generations (350 evaluations) at 1.1 evals/s.
setA-16 completes 5 generations. These instances have essentially no evolutionary capacity.

### 1.3 Stagnation Profile

| Metric | Value |
|--------|-------|
| Mean final stagnation | 4.6 |
| Median final stagnation | 1.0 |
| Max final stagnation | 20 |
| Mean final generation | 13.2 |
| Median final generation | 8.5 |
| Max final generation | 60 |
| Terminated by NoImprovement (stagnation ≥ 20) | 4/20 |
| Terminated by GenerationLimit (gen ≥ 199) | 0/20 |

**Finding F-411-6:** No instance reached the generation limit (200 generations). The search
terminates either by stagnation (4/20 instances) or by implicit time-limit exhaustion (16/20
instances that stagnate at 0–4 after very few generations). The generation budget is never
the binding constraint — evaluation cost is.

**Finding F-411-7:** The median final generation is 8.5. Half of all instances complete
fewer than 9 generations of evolution. With population size 50, this means fewer than 450
total evaluations — a severely constrained search.

**Finding F-411-8:** Only 4/20 instances terminate by NoImprovement (stagnation=20). These
are the fast-tier instances (setA-01, setA-02, setA-03, setA-07) that have enough throughput
to actually exhaust their improvement capacity. The remaining 16 instances terminate because
they run out of time, not because they converge.

---

## 2. RP-412: Construction Diagnostics

### 2.1 Initial Feasibility Rate Summary

| Metric | Value |
|--------|-------|
| Mean IFR | 10.60% |
| Median IFR | 7.00% |
| Min IFR | 0.00% |
| Max IFR | 72.00% |
| StdDev IFR | 16.25% |
| Instances with any_feasible=true | 14/20 |
| Instances with IFR=100% | 0/20 |
| Mean capacity_violation_count | 44.7 |
| Total capacity violations | 894 |

**Finding F-412-1:** No instance achieves 100% initial feasibility. The constructor
consistently produces infeasible individuals, with a mean of 89.4% of the initial
population being infeasible.

**Finding F-412-2:** Six instances (setA-02, setA-07, setA-16, setA-18, setA-19, setA-20)
have IFR=0%: the constructor produces zero feasible individuals. These instances begin
evolution with no valid reference point. The global-best tracker starts empty, and the
search must discover feasibility through evolutionary operators — a much harder problem.

**Finding F-412-3:** The IFR=0% instances are concentrated in the slow tier (setA-16
through setA-20 are all slow and all have IFR=0%). This creates a compounding failure:
slow evaluation + zero initial feasibility = almost no useful search.

**Finding F-412-4:** IFR is highly variable (StdDev 16.25%). setA-05 achieves 72% IFR
while six instances achieve 0%. This suggests the constructor's feasibility is strongly
instance-dependent, likely driven by capacity constraint tightness.

### 2.2 Per-Instance Construction Diagnostics

| Instance | Pop | Valid | Invalid | IFR | Any Feasible | Cap Violations |
|----------|-----|-------|---------|-----|--------------|----------------|
| setA-01 | 50 | 9 | 41 | 18.0% | True | 41 |
| setA-02 | 50 | 0 | 50 | 0.0% | **False** | 50 |
| setA-03 | 50 | 3 | 47 | 6.0% | True | 47 |
| setA-04 | 50 | 5 | 45 | 10.0% | True | 45 |
| setA-05 | 50 | 36 | 14 | 72.0% | True | 14 |
| setA-06 | 50 | 2 | 48 | 4.0% | True | 48 |
| setA-07 | 50 | 0 | 50 | 0.0% | **False** | 50 |
| setA-08 | 50 | 4 | 46 | 8.0% | True | 46 |
| setA-09 | 50 | 7 | 43 | 14.0% | True | 43 |
| setA-10 | 50 | 4 | 46 | 8.0% | True | 46 |
| setA-11 | 50 | 12 | 38 | 24.0% | True | 38 |
| setA-12 | 50 | 1 | 49 | 2.0% | True | 49 |
| setA-13 | 50 | 1 | 49 | 2.0% | True | 49 |
| setA-14 | 50 | 5 | 45 | 10.0% | True | 45 |
| setA-15 | 50 | 7 | 43 | 14.0% | True | 43 |
| setA-16 | 50 | 0 | 50 | 0.0% | **False** | 50 |
| setA-17 | 50 | 10 | 40 | 20.0% | True | 40 |
| setA-18 | 50 | 0 | 50 | 0.0% | **False** | 50 |
| setA-19 | 50 | 0 | 50 | 0.0% | **False** | 50 |
| setA-20 | 50 | 0 | 50 | 0.0% | **False** | 50 |

**Finding F-412-5:** All 894 invalids are attributed to capacity violations
(`capacity_violation_count = invalid_count`). This is consistent with the RP-410C
finding that Tail candidates (which dominate the initial population) are structurally
poor due to capacity constraint violations.

**Note on instrumentation:** `capacity_violation_count` is currently wired as a proxy
equal to `invalid_count`. Per-constraint breakdown (budget violations, repair failures,
routing failures) requires evaluator-level instrumentation not yet available. This is
documented as RP-412 Deliverable D4 (Evaluation Failure Taxonomy).

---

## 3. Joint Interpretation

### 3.1 Compounding Failure in the Slow Tier

The most severe finding is the compounding failure in instances setA-10 through setA-20:

| Instance | Evals/s | Gens | IFR | Stagnation | Outcome |
|----------|---------|------|-----|------------|---------|
| setA-10 | 1.6 | 10 | 8% | 0 | No improvement |
| setA-13 | 1.1 | 7 | 2% | 0 | No improvement |
| setA-16 | 1.0 | 5 | 0% | 4 | Minimal search |
| setA-17 | 0.4 | 2 | 20% | 0 | No improvement |
| setA-18 | 1.0 | 8 | 0% | 2 | No improvement |
| setA-19 | 0.6 | 3 | 0% | 2 | No improvement |
| setA-20 | 0.4 | 2 | 0% | 1 | No improvement |

These instances receive almost no evolutionary search. The solver is spending its entire
time budget evaluating the initial population, which is itself mostly infeasible.

### 3.2 Pipeline Constraint Summary

| Finding | RP-411 | RP-412 | Implication |
|---------|--------|--------|-------------|
| Evaluation dominates | 99.99% of time | — | Throughput improvement requires evaluator optimisation |
| Budget is scarce | Median 8.5 gens | — | Every wasted evaluation is costly |
| Initial population is poor | — | Mean IFR 10.60% | 89% of initial candidates are infeasible |
| Many instances start blind | — | 6/20 IFR=0% | Search must discover feasibility from scratch |
| Slow + infeasible = no search | setA-17: 2 gens | setA-17: IFR 20% | Compounding failure in slow tier |

**Overarching finding:** The search pipeline is doubly constrained. The evaluator is slow
(limiting generation count) and the constructor is unreliable (limiting initial population
quality). Both constraints reduce the effective search budget available to the evolutionary
operators. For 16/20 instances, the search terminates before it can meaningfully explore
the landscape.

**Causal chain:**

```
Poor constructor (mean IFR 10.6%; 6/20 instances IFR=0%)
        ↓
90% infeasible initial population
        ↓
Very expensive evaluator (99.99% of runtime; 1,555× throughput spread)
        ↓
Very few generations (median 8.5; 2 generations for setA-17, setA-20)
        ↓
Little evolutionary search (16/20 instances time-budget-starved)
        ↓
Few elite candidates
        ↓
Few global improvements
```

### 3.3 Implications for RP-408 and RP-409

- **RP-408** (lexicographic comparator): Changes the objective function, not the evaluator
  speed or constructor quality. Will not address the throughput or feasibility constraints.
  Its effect is limited to the 4 fast-tier instances that actually run enough generations
  for the comparator to matter.

- **RP-409** (operator redesign): Changes which candidates are generated, not how fast they
  are evaluated. Will not address the throughput constraint. May improve IFR if operators
  produce more feasible offspring, but the constructor (not the operators) determines the
  initial population.

- **Future work:** Incremental evaluation (reduce per-evaluation cost) and repair-based
  construction (increase IFR) are the highest-leverage interventions identified by this
  programme. These are not in scope for RP-408 or RP-409.

---

## 4. Instrumentation Notes

### RP-411 Phase 2 Fix

The `GenerationRecord` emit was originally placed **before** the selection/crossover/
mutation/eval loop, causing all timing fields to read 0.0. The fix moves the emit to
**after** the eval+sort+candidate-emit block, so all `t_*_ms` accumulators are populated
before the record is written. Verified by inspecting sample records:

```
Gen 0:  eval=105.8ms  sel=0.30ms  xo=0.07ms  mut=0.003ms  other=0.18ms  total=106.4ms
Gen 50: eval=165.6ms  sel=0.12ms  xo=0.05ms  mut=0.005ms  other=0.16ms  total=165.9ms
```

### RP-412 Phase 2 Proxy

`capacity_violation_count = invalid_count` is a proxy. The evaluator does not currently
expose per-constraint diagnostics. The full Evaluation Failure Taxonomy (RP-412 D4)
requires evaluator-level instrumentation.

---

## 5. Frozen Baseline Values (20 instances)

These values are frozen as the RP-411/412 baseline. All future experiments (RP-408,
RP-409) are compared against this reference.

| Metric | Baseline Value |
|--------|---------------|
| Eval fraction of gen time | 99.99% |
| Throughput range | 0.4 – 622.0 evals/s |
| Throughput ratio (max/min) | 1,555× |
| Mean final generation | 13.2 |
| Median final generation | 8.5 |
| Max final generation | 60 |
| Instances reaching gen limit (200) | 0/20 |
| Instances terminated by NoImprovement | 4/20 |
| Mean IFR | 10.60% |
| Median IFR | 7.00% |
| Instances with IFR=0% | 6/20 |
| Instances with any_feasible=true | 14/20 |
| Mean capacity_violation_count | 44.7 |
| Total capacity violations (20 instances) | 894 |