# M19 Foundation Report — v1.0

**Status:** FROZEN  
**Frozen:** 2026-07-10  
**Campaign:** `campaign_engine_v1.0_verify` (v3 — full measurement model)  
**Baseline:** `BASELINE-v1.0.json`  
**Schema:** `BASELINE-v1.0-schema.json`

---

## Scope

This report documents the M19 ROADEF 2026 benchmark campaign for the Coralys platform.
M19 validates that the `coralys-moga` `EvolutionEngine` generalizes to the ROADEF
traffic engineering problem domain without modification, and establishes a reproducible
quantitative baseline for all future Horizon 4 research milestones.

M19 is a **platform validation milestone**, not a competition research milestone.
Evidence produced here characterizes the baseline optimizer; it does not claim
competitive performance against ROADEF 2026 participants.

---

## Executive Summary

| Metric                      | Result                      |
|-----------------------------|-----------------------------|
| Campaign ID                 | campaign_engine_v1.0_verify |
| Instances                   | 20 / 20                     |
| Campaign runtime            | 4,199.3 s (70.0 min)        |
| Valid solutions             | **17**                      |
| Infeasible solutions        | **3**                       |
| Crashes                     | **0**                       |
| Parser failures             | **0**                       |
| Engine modifications        | **0**                       |
| Qualification modifications | **0**                       |
| A-001 violations            | **0**                       |

---

## Architectural Invariant A-001

> **`valid == true ⇒ obj.is_finite()`**

This invariant is enforced in `RoadefFitnessEvaluator::evaluate()` and verified
across all 20 instances in three independent campaign runs. Zero violations observed.

A-001 is now an evidence-backed architectural property of the Coralys platform,
not merely a design assertion.

---

## Campaign Configuration

| Parameter            | Value                                                        |
|----------------------|--------------------------------------------------------------|
| Population size      | 50                                                           |
| Elite count          | 5                                                            |
| Generation limit     | 500                                                          |
| Mutation rate        | 0.30                                                         |
| Crossover rate       | 0.70                                                         |
| No-improvement limit | 20                                                           |
| Seed policy          | Random (unseeded)                                            |
| Budget policy        | Adaptive: clamp(0.5 ms × demands × links, 30 s, 300 s)      |
| Termination policy   | Or(FixedGenerations(500), Or(NoImprovement(20), MaxRuntime)) |

---

## Search Mode Classification

| Mode              | Count | Meaning                                              |
|-------------------|-------|------------------------------------------------------|
| SearchLimited     | 2     | GA converged before budget (NoImprovement)           |
| EvaluationLimited | 15    | Budget exhausted while still searching (TimeBudget)  |
| Infeasible        | 3     | No feasible solution found within budget             |

The optimizer itself is rarely the limiting factor. The evaluator dominates runtime
for all medium and large instances.

---

## Per-Instance Results

| # | Instance | Demands | Nodes | Links | Slots | Budget(s) | Obj | MLU | Valid | Class | ms | Gens | ms/gen | Mode | Termination |
|---|----------|---------|-------|-------|-------|-----------|-----|-----|-------|-------|----|------|--------|------|-------------|
| 1 | setA-01 | 40 | 20 | 80 | 2 | 30 | 47.9176 | 0.7105 | ✓ | Competitive | 14965 | 114 | 131 | SearchLimited | NoImprovement |
| 2 | setA-02 | 45 | 30 | 150 | 2 | 30 | 52.6536 | 0.7258 | ✓ | Competitive | 27292 | 103 | 264 | EvaluationLimited | TimeBudget |
| 3 | setA-03 | 20 | 50 | 250 | 2 | 30 | 59.1585 | 0.6793 | ✓ | Competitive | 13665 | 71 | 192 | SearchLimited | NoImprovement |
| 4 | setA-04 | 200 | 50 | 250 | 2 | 30 | 64.8094 | 0.6900 | ✓ | Weak | 31368 | 18 | 1,742 | EvaluationLimited | TimeBudget |
| 5 | setA-05 | 100 | 100 | 396 | 2 | 30 | 13.2801 | 0.1859 | ✓ | Good | 30224 | 17 | 1,777 | EvaluationLimited | TimeBudget |
| 6 | setA-06 | 500 | 100 | 500 | 2 | 125 | 48.6073 | 0.5352 | ✓ | Competitive | 132909 | 16 | 8,306 | EvaluationLimited | TimeBudget |
| 7 | setA-07 | 800 | 100 | 500 | 2 | 200 | 261.6493 | 0.8360 | ✓ | Poor | 211961 | 14 | 15,140 | EvaluationLimited | TimeBudget |
| 8 | setA-08 | 200 | 150 | 654 | 2 | 65 | 53.3019 | 0.5576 | ✓ | Competitive | 70806 | 12 | 5,900 | EvaluationLimited | TimeBudget |
| 9 | setA-09 | 200 | 150 | 750 | 2 | 75 | 153.7590 | 0.7682 | ✓ | Poor | 78831 | 14 | 5,630 | EvaluationLimited | TimeBudget |
| 10 | setA-10 | 1000 | 150 | 966 | 2 | 300 | 86.9824 | 0.6576 | ✓ | Weak | 303699 | 11 | 27,609 | EvaluationLimited | TimeBudget |
| 11 | setA-11 | 400 | 200 | 1000 | 2 | 200 | 110.1578 | 0.7297 | ✓ | Poor | 200172 | 13 | 15,397 | EvaluationLimited | TimeBudget |
| 12 | setA-12 | 400 | 200 | 898 | 2 | 179 | 18.4086 | 0.7600 | ✓ | Good | 179134 | 10 | 17,913 | EvaluationLimited | TimeBudget |
| 13 | setA-13 | 2000 | 200 | 1000 | 2 | 300 | 125.0976 | 0.9573 | ✓ | Poor | 308621 | 3 | 102,873 | EvaluationLimited | TimeBudget |
| 14 | setA-14 | 600 | 250 | 1108 | 2 | 300 | 88.5745 | 0.5595 | ✓ | Weak | 309630 | 10 | 30,963 | EvaluationLimited | TimeBudget |
| 15 | setA-15 | 600 | 250 | 1250 | 2 | 300 | 240.8649 | 0.8620 | ✓ | Poor | 320860 | 11 | 29,169 | EvaluationLimited | TimeBudget |
| 16 | setA-16 | 4800 | 250 | 1452 | 2 | 300 | ∞ | 1.2275 | ✗ | Invalid | 365567 | 0 | — | Infeasible | TimeBudget |
| 17 | setA-17 | 2000 | 300 | 1270 | 2 | 300 | 60.6930 | 0.3948 | ✓ | Weak | 350061 | 3 | 116,687 | EvaluationLimited | TimeBudget |
| 18 | setA-18 | 2000 | 300 | 1500 | 2 | 300 | 799260.9785 | 0.8399 | ✓ | Poor | 313839 | 3 | 104,613 | EvaluationLimited | TimeBudget |
| 19 | setA-19 | 6000 | 300 | 1998 | 2 | 300 | ∞ | 1.1557 | ✗ | Invalid | 378406 | 0 | — | Infeasible | TimeBudget |
| 20 | setA-20 | 6000 | 400 | 2000 | 2 | 300 | ∞ | 1.1457 | ✗ | Invalid | 539734 | 0 | — | Infeasible | TimeBudget |

*Note: ms/gen is undefined (—) for infeasible instances where no generation completed.*  
*Table generated from `BASELINE-v1.0.json` (canonical source).*

---

## Performance Frontier: ms/gen

The `ms_per_generation` metric quantifies evaluation cost per generation.
It is the primary engineering KPI for RP-310 (incremental evaluation).

| Instance | Demands × Links | ms/gen  |
|----------|----------------:|--------:|
| setA-01 | 3,200 | 131 |
| setA-02 | 6,750 | 264 |
| setA-03 | 5,000 | 192 |
| setA-04 | 50,000 | 1,742 |
| setA-05 | 39,600 | 1,777 |
| setA-06 | 250,000 | 8,306 |
| setA-07 | 400,000 | 15,140 |
| setA-08 | 130,800 | 5,900 |
| setA-09 | 150,000 | 5,630 |
| setA-10 | 966,000 | 27,609 |
| setA-11 | 400,000 | 15,397 |
| setA-12 | 359,200 | 17,913 |
| setA-13 | 2,000,000 | 102,873 |
| setA-14 | 664,800 | 30,963 |
| setA-15 | 750,000 | 29,169 |
| setA-17 | 2,540,000 | 116,687 |
| setA-18 | 3,000,000 | 104,613 |

Evaluation cost increased by approximately 890× between setA-01 (3,200 demand-link
products, 131 ms/gen) and setA-17 (2,540,000 demand-link products, 116,687 ms/gen).
The observed scaling is strongly correlated with routing workload (particularly
demands × links), motivating RP-310 (incremental evaluation). This is an empirical
observation from the campaign data; it does not constitute a formal complexity proof.

**RP-310 success criterion:** ≥2× reduction in ms/gen on setA-10 with no
degradation in objective quality or feasibility rate.

---

## Research Observations

### O-001 — A-001 invariant holds across all 20 instances

`valid == true ⇒ obj.is_finite()` was verified in all 20 instances across three
independent campaign runs (v1, v2, v3). Zero violations observed. This invariant
is now an evidence-backed architectural property.

### O-002 — Peak MLU is not a reliable predictor of objective value

Instances with higher MLU may exhibit substantially lower objective values.
Example: setA-12 (MLU=0.76, obj=18.4) vs setA-11 (MLU=0.73, obj=110.2).
The ROADEF objective evaluates the complete link utilization distribution,
not peak utilization alone. Research should optimize for objective, not MLU.

### O-003 — setA-13 is stochastically feasible

setA-13 (2000 demands, 1000 links) was infeasible in campaign v1 but feasible
in campaigns v2 and v3. The feasible region exists but is difficult to reach
under random initialization. This makes setA-13 an ideal benchmark for
evaluating search strategies (RP-301, RP-302, RP-303).

### O-004 — Evaluation cost grows rapidly with routing workload

Across the benchmark suite, evaluation cost increased by approximately 890×
between setA-01 and setA-17. The observed scaling is strongly correlated with
routing workload (particularly demands × links), motivating RP-310 (incremental
evaluation). This is an empirical observation from the campaign data; it does
not constitute a formal complexity proof.

### O-005 — Network topology is a significant factor in optimization behaviour

Benchmark instances with similar demand counts exhibited markedly different
feasibility and objective characteristics. For example, setA-13 and setA-17
both have 2000 demands yet differ substantially in feasibility and objective
quality across multiple campaigns. This indicates that network topology is a
significant factor influencing optimization behaviour, independent of demand
count alone.

### O-006 — Evaluation throughput is the primary constraint on optimization progress

Under the M19.5 baseline, the majority of benchmark instances are
evaluation-limited rather than search-limited (SearchLimited=2,
EvaluationLimited=15, Infeasible=3). This indicates that evaluation
throughput is the primary constraint on optimization progress for medium and
large instances, and directly justifies the ordering of RP-310 ahead of
search operator improvements (RP-301 through RP-309).

---

## Behavioural Classification

| Class                         | Instances                          | Characteristics                                    |
|-------------------------------|------------------------------------|----------------------------------------------------|
| Search-Limited                | setA-01, setA-03                   | Converge before budget; NoImprovement termination  |
| Evaluation-Limited (stable)   | setA-02, setA-04–15, setA-17–18    | Consistently feasible; budget exhausted            |
| Search-Sensitive              | setA-13                            | Feasible in some runs; stochastic initialization   |
| Infeasible (current baseline) | setA-16, setA-19, setA-20          | No feasible solution in any of three campaigns     |

*"Infeasible (current baseline)" is a property of the frozen M19.5 configuration,
not a statement about the benchmark instances themselves.*

---

## Stochastic Note

This baseline characterizes one canonical execution (v3) under the frozen
configuration. Individual objective values may vary across executions due to
stochastic initialization and evolutionary search. Instances classified as
Infeasible in this run may be stochastically feasible (see O-003).

The canonical baseline consists of the recorded v3 execution rather than a fixed
random seed. Future comparisons shall be made against the recorded baseline
artifacts, not against newly generated random executions.

---

## Benchmark Roles for Horizon 4

| Role                | Instance  | Reason                                                              |
|---------------------|-----------|---------------------------------------------------------------------|
| Fast development    | setA-04   | 30 s budget, quick iteration, moderate size                         |
| Search challenge    | setA-13   | Stochastically feasible; ideal for evaluating search strategies     |
| Scalability limit   | setA-19   | Robustly infeasible; measures evaluation efficiency advances        |
| Regression target   | setA-18   | Stable behaviour across campaigns (~799k obj, 3 gens, 104k ms/gen) |

---

## Defect Record

| ID   | Description                          | Root Cause                              | Status  |
|------|--------------------------------------|-----------------------------------------|---------|
| D-01 | valid=true, obj=inf (setA-02, v0)    | Missing finite check in evaluator       | Fixed   |

---

## Evidence Artifacts

| Artifact                                | Description                              | Generated By                      |
|-----------------------------------------|------------------------------------------|-----------------------------------|
| `campaign_engine_v1.0_verify.json`      | Full campaign results (v3, 20 instances) | campaign_engine binary v3         |
| `EVIDENCE-engine-v1.0.md`               | Markdown evidence report (v3)            | campaign_engine binary v3         |
| `BASELINE-v1.0.json`                    | Frozen baseline contract                 | Derived from v3 campaign          |
| `BASELINE-v1.0-schema.json`             | JSON Schema for baseline contract        | Manual                            |
| `M19-FOUNDATION-REPORT-v1.0.md`         | This document                            | Generated from BASELINE-v1.0.json |

---

## Horizon 4 Research Priorities (Evidence-Based)

Based on M19 campaign evidence, the following research priorities are ordered
by expected impact:

**RP-310 — Incremental evaluation** (highest ROI): Reduce evaluation cost per
generation. Target: ≥2× ms/gen reduction on setA-10 with no degradation in
objective quality or feasibility rate. Justified by O-006 (15/20 instances
evaluation-limited).

**RP-301 — Topology-aware initialization**: Increase probability of entering
feasible regions. Target: reduce infeasible instance count from 3 to ≤1.
Justified by O-005 (topology affects feasibility) and O-003 (setA-13 stochastic).

**RP-302 — Constructive initialization**: Improve first feasible population quality.

**RP-303 — Repair operators**: Recover infeasible individuals instead of discarding.

**RP-304 — Local search**: Improve solutions after feasibility is established.

**RP-305 — Adaptive mutation**: Escape poor basins; target search-sensitive instances.

### Subsequent Research Programme

**RP-306 — Diversity preservation**: Prevent premature convergence.

**RP-307 — Hyper-heuristics**: Adaptive operator selection.

**RP-308 — Parallel evolution**: Increase search throughput.

**RP-309 — Incremental routing structures**: Reduce routing recomputation.

---

This report establishes the frozen M19.5 baseline against which all subsequent
ROADEF optimization research (M20 and Horizon 4) shall be evaluated. Unless
explicitly stated otherwise, performance claims shall be made relative to this
baseline.

*M19 Foundation Report v1.0 — Frozen 2026-07-10*  
*Coralys Platform — ROADEF 2026 Benchmark Campaign*  
*Per-instance table generated from `BASELINE-v1.0.json` (canonical source of truth)*
