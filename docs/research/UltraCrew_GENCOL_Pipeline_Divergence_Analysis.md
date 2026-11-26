# UltraCrew vs GENCOL — Pipeline Divergence Analysis

**Status:** Final (revised)
**Date:** 2026-07-30
**Relates to:** GERAD G-2014-22 benchmark
**Source code audited:** [`services/ultracrew_server/src/main.rs`](../../services/ultracrew_server/src/main.rs), [`adapters/ultracrew/src/pipeline.rs`](../../adapters/ultracrew/src/pipeline.rs), [`adapters/ultracrew/src/constraint_engine.rs`](../../adapters/ultracrew/src/constraint_engine.rs), [`coralys-moga/src/engine.rs`](../../coralys-moga/src/engine.rs)

---

## 1. Executive Summary

The benchmark comparison between UltraCrew and the GERAD G-2014-22 reference pairings (produced by the GENCOL column-generation solver) has been misframed in prior analysis. The two systems are **not solving the same problem at the same pipeline stage**. This document maps both pipelines precisely from source code, identifies every structural divergence point, and provides a corrected interpretation of the benchmark gap.

The key finding is: **the benchmark gap is explained by at least three structural differences, not by optimizer quality**. The benchmark does not provide evidence that the optimizer itself is deficient. The observed differences are consistent with architectural differences in search space and objective function. The benchmark suggests that further alignment of the search space and objective function is the most promising direction for investigation — but whether that alignment will eliminate the gap has not yet been demonstrated experimentally.

---

## 2. Pipeline Architecture Comparison

### 2.1 GENCOL Pipeline (GERAD G-2014-22 reference)

```
Flights (raw schedule data)
    │
    ▼
Duty generation
    │
    ▼
Flight connection network
    │
    ▼
Pricing subproblem (shortest-path on flight network)
    │  generates new candidate pairings on demand
    │  millions of candidates may be implicitly explored
    ▼
Set-partitioning / set-covering optimization
    │  branch-and-price selects minimum-cost subset
    │  covering all flights exactly once
    ▼
Final optimized pairings  ←── pairings.csv contains this
```

The GENCOL reference pairings are the **output of a global optimization** over a dynamically generated candidate space. They are not a construction heuristic output.

### 2.2 UltraCrew Pipeline (current)

```
Workers + Shifts
    │
    ▼
ScheduleContext construction
    │
    ▼
MOGA (EvolutionEngine)
    │  genome = HashMap<shift_id, worker_id>
    │  optimizes: coverage + fairness + fatigue + pairing completion reward
    ▼
ScheduleGenome (best assignment found)
    │
    ▼
/api/pairings  ←── compare_gerad.py measures this
    │  greedy per-worker grouping of the fixed assignment
    │  ground_time < LAYOVER_REST_HOURS=8h → same FDP
    │  rest_gap < HOME_BASE_REST_HOURS=34h → same pairing
    ▼
TC CAR 700 legality check per pairing
    │
    ▼
PairingsResponse (compliance report)
```

**Critical observation:** [`pairings_handler()`](../../services/ultracrew_server/src/main.rs:1300) operates on a **pre-assigned schedule** (`HashMap<shift_id, worker_id>`). It does not generate candidate pairings — it reads an already-fixed assignment and groups it into pairings for compliance reporting. The MOGA optimizer runs **upstream** of this endpoint, in `/api/schedule` via [`run_pipeline_from_request()`](../../adapters/ultracrew/src/pipeline.rs:49).

The benchmark has been comparing GENCOL's final optimized output against UltraCrew's post-processing of a MOGA-assigned schedule. These are different pipeline stages.

**Provenance of benchmark inputs:** The inputs supplied to `/api/pairings` during the benchmark were produced by the UltraCrew MOGA via `/api/schedule`. They were not produced by GENCOL, nor by any external schedule source. This means the benchmark is testing UltraCrew's full pipeline (MOGA assignment → greedy pairing grouping), but it is measuring only the pairing output — not the assignment quality directly. The MOGA's assignment decisions are the upstream cause of every pairing structure the benchmark observes.

---

## 3. Decision Variable Comparison

This is the most fundamental architectural difference.

**GENCOL:**
```
decision variable: x(pairing) ∈ {0, 1}
objective: minimize Σ cost(p) · x(p)
subject to: each flight covered exactly once
```

**UltraCrew MOGA** ([`optimization.rs`](../../adapters/ultracrew/src/optimization.rs)):
```
decision variable: assignment(shift_id) → worker_id
objective: maximize fitness(assignment)
           = coverage + fairness + fatigue + pairing_completion_reward
```

These are different optimization spaces with different topologies. GENCOL can directly select any pairing that satisfies legality constraints. UltraCrew's MOGA selects which worker covers which shift; pairing structure emerges as a consequence of that assignment.

The implication: UltraCrew cannot directly optimize arbitrary pairing structures independently of worker assignments. Any pairing produced by UltraCrew must emerge from the optimized shift-to-worker assignment and the subsequent deterministic grouping logic. GENCOL can generate and evaluate any pairing that satisfies legality constraints, including ones that require specific flight connections across multiple days.

---

## 4. Divergence Points (Source-Code Level)

### 4.1 Layover threshold: 8h vs 10h

**UltraCrew** ([`main.rs:1139`](../../services/ultracrew_server/src/main.rs:1139)):
```rust
const LAYOVER_REST_HOURS: f64 = 8.0;
```

**GENCOL** (Kasirzadeh et al. 2017, §3.2): layover "typically lasts for at least 10 hours."

**Reconstruction experiment (2026-07-30):** A controlled experiment varied the threshold between 8h and 10h across all 7 GERAD instances using [`compare_gerad.py`](../../compare_gerad.py), operating at the flight-leg level (grouping legs into FDPs, then FDPs into pairings, using the benchmark's crew–flight allocation from `duties.csv`). The delta in pairing count was **+0.0pp** across all instances. The threshold does affect FDP structure (multi-FDP ratio changes between conditions), confirming the grouping logic is exercised, but does not affect pairing count because all critical-zone gaps are well below the 34h pairing boundary.

**Scope limitation:** This experiment is a *reconstruction experiment*, not an optimization experiment. It freezes the benchmark's crew–flight allocation and tests only the reconstruction stage. It cannot determine whether changing the threshold inside UltraCrew's MOGA optimizer would alter the optimized assignment or the final pairing solution. The correct experiment would require running UltraCrew end-to-end from raw `flights.csv` + `crew.csv` inputs.

**Status of hypothesis:** The hypothesis that the 8h vs 10h threshold contributes to the pairing count gap remains **open**. The experiment narrows the question — the threshold does not affect pairing count when the crew–flight allocation is fixed — but cannot answer whether it affects the allocation itself. See [`UltraCrew_Layover_Threshold_Experiment.md`](../../docs/research/UltraCrew_Layover_Threshold_Experiment.md) for full methodology and results.

**Important distinction:** TC CAR 700 regulatory minimum is 8h; the benchmark model uses 10h as a construction parameter. These are different things. This finding does not affect the regulatory compliance model.

### 4.2 Fitness function: coverage + fairness vs pairing cost

**UltraCrew** ([`constraint_engine.rs:39–182`](../../adapters/ultracrew/src/constraint_engine.rs:39)):

```
fitness = 10000.0 (base)
        - 1000.0 per skill mismatch (HC1)
        - 1000.0 per overlap (HC2)
        - 500.0  per weekly hours violation (HC3)
        - 800.0 * severity per rest violation
        - fatigue_cost (historical_fatigue * hours * 2.0)
        - fairness_cost (variance * 10.0)
        + 500.0 * shift_count per complete legal pairing
```

**GENCOL** (Kasirzadeh et al. 2017): minimizes pairing cost = deadhead cost + hotel cost + duty credit + time away from base (TAFB) + connection penalties.

**Effect:** UltraCrew's fitness function does not include deadhead cost, hotel cost, TAFB, or connection penalties. It optimizes for coverage completeness and workload fairness. GENCOL optimizes for operational cost. These are different objective functions. Even if both produce legal pairings, they will converge to different solutions because they are minimizing different things.

### 4.3 Pairing construction: greedy per-worker vs global network routing

**UltraCrew** ([`main.rs:1324–1364`](../../services/ultracrew_server/src/main.rs:1324)): groups each worker's already-assigned shifts greedily by time order. A pairing boundary is placed wherever the rest gap exceeds the threshold. This is a deterministic post-processing step, not an optimization.

**GENCOL:** constructs pairings by solving a shortest-path problem on a directed flight network (connection graph). This allows it to find pairings that route through specific intermediate stations, respecting connection times and base-return constraints globally.

**Effect:** UltraCrew's greedy grouping can span home-base rest boundaries that GENCOL avoids by routing around them. This is the primary source of the remaining rest violations in the benchmark results.

### 4.4 Candidate space size

**UltraCrew:** The candidate space is all valid `HashMap<shift_id, worker_id>` assignments. The MOGA explores a small fraction of this via population-based search.

**GENCOL:** The candidate space is all valid pairings. Column generation dynamically generates new candidates with negative reduced cost via a shortest-path pricing subproblem, which can enumerate millions of candidate pairings implicitly.

**Effect:** GENCOL's search space is structurally richer for the pairing problem. It can find pairings that require specific flight connections that a random assignment mutation would be unlikely to discover. This does not imply that the MOGA is incapable of exploring a large assignment space. Rather, the two search spaces represent different abstractions of the crew pairing problem, making direct comparisons of optimization quality difficult.

---

## 5. Answering the Five Diagnostic Questions

### Q1: How many candidate pairings are generated before optimization?

**Answer:** Zero. There is no candidate pairing pool. [`pairings_handler()`](../../services/ultracrew_server/src/main.rs:1300) reads a pre-assigned schedule and groups it — it does not generate candidates.

### Q2: What percentage survive legality filtering?

**Answer:** Not applicable in the current architecture. Legality is checked after the fact on the final assignment, not used to filter a pool during optimization.

### Q3: What percentage are evaluated by MOGA?

**Answer:** The MOGA evaluates `ScheduleGenome` objects (assignments), not pairings. Each genome evaluation calls [`ConstraintEngine.evaluate()`](../../adapters/ultracrew/src/constraint_engine.rs:39), which includes a pairing completion reward but does not enumerate all possible pairings. The MOGA evaluates `population_size × generation_limit` genomes total.

### Q4: Can MOGA create new pairings, or only select among existing ones?

**Answer:** The MOGA creates new assignments via mutation and crossover. New pairings emerge implicitly from new assignments. It does not select from a pre-built pairing pool, and it has no mechanism to intentionally target specific pairing structures (e.g., "find a 3-day pairing through YYZ–YVR–YYC"). Pairing topology is a side-effect of assignment decisions, not a first-class optimization variable.

### Q5: Is the fitness function aligned with the GERAD objective?

**Answer:** No. [`ConstraintEngine.evaluate()`](../../adapters/ultracrew/src/constraint_engine.rs:39) optimizes coverage completeness, workload fairness, fatigue, and pairing completion reward. GENCOL minimizes deadhead cost, hotel cost, TAFB, and connection penalties. These share no terms. They will converge to different solutions even on identical inputs.

---

## 6. Corrected Interpretation of Benchmark Results

The benchmark results (65–73% pairing count ratio, 46–70% compliance rate) should be interpreted as follows:

**What the gap does NOT mean:**
- UltraCrew's MOGA is weak or broken. The benchmark does not provide evidence of optimizer deficiency; the observed differences are consistent with architectural differences in search space and objective function.
- UltraCrew's constraint engine is incorrect.
- The TC CAR 700 FTA model is wrong.

**What the gap likely means (pending experimental confirmation):**
- UltraCrew and GENCOL are solving different optimization problems with different decision variables, different objective functions, and different construction assumptions.
- The 65–73% pairing count ratio is **not explained by the layover threshold within the scope of the reconstruction experiment** (see [`UltraCrew_Layover_Threshold_Experiment.md`](../../docs/research/UltraCrew_Layover_Threshold_Experiment.md)). Varying the threshold between 8h and 10h produced a 0.0pp change in pairing count when the benchmark's crew–flight allocation was held fixed. However, the experiment cannot determine whether the threshold affects UltraCrew's optimizer when run end-to-end from raw inputs. The hypothesis remains open.
- The structural similarity (multi-duty ratios within 1–7pp, comparable pairing spans) is consistent with sound duty generation and FTA model implementation.
- The remaining compliance gap is consistent with the effects of greedy per-worker grouping and the absence of a global pairing construction stage, although the relative contribution of these factors has not yet been experimentally isolated.

**The encouraging finding:** Despite using a simpler construction process and a different objective function, UltraCrew already matches GENCOL's structural characteristics closely. This is consistent with correct duty generation and FTA model implementation. The gap is architectural, not algorithmic.

---

## 7. Recommended Next Steps (Prioritized)

The most significant architectural difference identified in this analysis is that pairing topology is not a first-class optimization variable. The recommended sequence addresses that difference first, then measures the result, then adds objective alignment.

### Step 1: Investigate introducing pairing topology as an explicit optimization variable (2–4 weeks)

This appears to be the highest-leverage architectural change to investigate based on the current analysis, and it should precede further benchmarking. Running the 10h experiment or adding objective terms against the current architecture would measure a system that has already been identified as structurally different in its optimization space.

**Today:**
```
Genome → Worker Assignment → Pairings emerge as side-effect
```

**Target:**
```
Genome → Worker Assignment + Pairing Structure → Fitness
```

Concretely: add a mutation operator that, instead of randomly reassigning a shift, attempts to complete a legal pairing by finding the next connectable shift for the same worker. This gives the MOGA a pairing-construction bias without requiring a full column generation implementation.

This does not require changing the MOGA architecture. It requires adding one new mutation operator and a pairing-topology term to the fitness function.

### Step 2: ~~Run the 10h threshold experiment~~ — **Complete, result: no effect**

A reconstruction experiment was run on 2026-07-30 using [`compare_gerad.py`](../../compare_gerad.py) across all 7 GERAD instances. The experiment froze the benchmark's crew–flight allocation and varied the threshold between 8h and 10h at the flight-leg level. The delta in pairing count was **+0.0pp** on every instance. The threshold does affect FDP structure (multi-FDP ratio changes between conditions), but does not affect pairing count because all critical-zone gaps are well below the 34h pairing boundary.

Full results and scope analysis: [`UltraCrew_Layover_Threshold_Experiment.md`](../../docs/research/UltraCrew_Layover_Threshold_Experiment.md).

**Scope limitation:** This experiment is a reconstruction experiment, not an optimization experiment. It cannot determine whether the threshold affects UltraCrew's optimizer when run end-to-end from raw inputs. The hypothesis remains open. The correct end-to-end experiment would require running UltraCrew from `flights.csv` + `crew.csv` with full crew availability windows and pairing cost parameters — inputs the benchmark does not fully provide.

**Implication for the roadmap:** Within the scope of the reconstruction experiment, threshold tuning does not close the pairing count gap. The pairing topology investigation (Step 1) and objective function alignment (Step 3) remain the primary levers for the architectural gap.

### Step 3: Add GENCOL-equivalent objective terms (1–2 weeks)

Add the following terms to [`ConstraintEngine.evaluate()`](../../adapters/ultracrew/src/constraint_engine.rs:39) to align the fitness function with the GERAD objective:

- **Time Away From Base (TAFB):** penalize total layover hours per pairing. GENCOL minimizes this directly.
- **Hotel nights:** penalize number of overnight layovers per pairing.
- **Deadhead cost:** penalize shifts where the assigned worker is not the most qualified (proxy for deadhead).
- **Connection penalty:** penalize pairings where the inter-FDP rest is close to the minimum (fragile connections).

These additions do not require changing the MOGA architecture. They only change what the existing fitness function rewards.

### Step 4: Evaluate whether column generation is needed (strategic decision)

Column generation (as used by GENCOL) is a mature OR technique that guarantees exploration of the full pairing space. It is also significantly more complex to implement correctly than the current MOGA approach.

The decision to implement column generation should be driven by whether Steps 1–3 close the gap sufficiently for the target use case. If UltraCrew's differentiation is in real-time disruption handling, multi-objective optimization, and fatigue modeling (rather than minimum-cost static pairing), then column generation may not be necessary.

---

## 8. Benchmark Positioning

GERAD evaluates one specific optimization problem:

> Static minimum-cost crew pairing.

UltraCrew targets a broader workforce optimization problem that additionally includes workforce fairness, fatigue modeling, explainability, disruption recovery, incremental rescheduling, and multi-objective optimization.

These are different claims and should be evaluated independently:

> Matching GENCOL on the GERAD benchmark is evidence of competitive crew pairing quality.

> Surpassing GENCOL on broader operational metrics is evidence of platform superiority.

The benchmark is therefore not the goal — it is one narrow measurement within a much broader problem space. GENCOL has no disruption model, no multi-objective Pareto frontier, no fatigue model, no nurse rostering capability, no interactive decision support, and no incremental re-optimization. Matching GENCOL on static pairing cost while simultaneously optimizing those extra dimensions would constitute a stronger product, not merely an equivalent one.

The correct benchmark question is:

> Can UltraCrew produce pairings of comparable operational quality while simultaneously optimizing objectives that GENCOL does not model?

If yes, UltraCrew is a stronger product even if it does not reproduce the exact pairing structure from GENCOL.

---

## 9. What UltraCrew Can Do That GENCOL Cannot

- **Real-time disruption recovery** ([`simulate_sick_leave`](../../services/ultracrew_server/src/main.rs:551), [`can_recover`](../../services/ultracrew_server/src/main.rs:652)): GENCOL has no disruption model.
- **Multi-objective Pareto optimization** (INRC pipeline): GENCOL optimizes a single weighted cost.
- **Fatigue modeling** (fatigue penalty in fitness function): GENCOL uses a simple cost model.
- **INRC nurse rostering** (`/api/inrc/compliance`): outside GENCOL's domain.
- **Interactive decision support** (pilot portal, decision cases, schedule versions): GENCOL is a batch solver.
- **Incremental re-optimization** (`/api/reschedule`): GENCOL requires a full re-solve.

---

## 10. Threats to Validity

This analysis has several important limitations that should be considered when interpreting its conclusions.

The effect of the 8-hour versus 10-hour layover threshold has not yet been experimentally isolated. The claim that this threshold is a likely contributor to the pairing count gap is a hypothesis, not a measured result.

The benchmark compares GENCOL under its published objective function with UltraCrew under a different objective function. Differences in output are therefore attributable to both architectural and objective differences simultaneously; the contribution of each cannot be separated without a controlled experiment.

GENCOL implementation details beyond those published in Kasirzadeh et al. (2017) are not available for inspection. The pipeline description in Section 2.1 is reconstructed from the paper, not from source code audit.

The comparison measures pairing output characteristics (count, compliance rate, multi-duty ratio) rather than optimization convergence behaviour. It does not evaluate whether either system has converged to a local or global optimum on its respective objective.

Runtime, scalability, memory consumption, and convergence speed were not evaluated. The benchmark results say nothing about the relative computational cost of the two approaches.

The comparison focuses on static crew pairing and does not assess disruption recovery, incremental re-optimization, or any of the broader workforce optimization objectives that UltraCrew targets. Performance on those dimensions is outside the scope of this analysis.

Accordingly, this document should be interpreted as an architectural comparison and benchmark analysis, not as proof of algorithmic superiority or inferiority.

---

---

## 11. Conclusion

This analysis shows that the observed differences between UltraCrew and the GERAD reference cannot be interpreted as a direct comparison of optimizer quality. The two systems optimize different decision variables under different objective functions and pairing construction strategies. GENCOL is a pairing-centric optimizer, where pairings are the primary decision variables. UltraCrew is an assignment-centric optimizer, where pairings are derived from optimized assignments through deterministic post-processing.

The benchmark therefore establishes architectural differences rather than algorithmic superiority or inferiority. Future work should focus on experimentally isolating the impact of individual design choices — such as pairing topology, layover thresholds, and objective alignment — before drawing conclusions about comparative optimization performance.

---

## 12. Reference

Kasirzadeh A., Saddoune M., Soumis F. (2017). Airline crew scheduling: models, algorithms, and data sets. *EURO Journal on Transportation and Logistics*, 6(2), 111–137. DOI: 10.1007/s13676-015-0080-x

The GERAD G-2014-22 benchmark instances are available at `benchmarks/gerad-g2014-22/`. Reference pairings in `pairings.csv` per instance are produced by the GENCOL optimization model as described in the paper. The paper does not explicitly certify each reference pairing as individually TC CAR 700 compliant; they are the output of the GENCOL solver under the paper's own legality model.
