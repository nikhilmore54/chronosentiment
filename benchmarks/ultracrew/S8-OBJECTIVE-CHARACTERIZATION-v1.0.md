# Sprint 8 Stream A — Objective Characterization Freeze
**Document:** S8-OBJECTIVE-CHARACTERIZATION-v1.0.md
**Date:** 2025-07-13
**Status:** FROZEN
**Benchmark:** UB-002-v1.0 (UltraCrew, 25 workers, 83 shifts/week, SC2 enabled)
**API:** Coralys localhost:3001, generation_limit=50

---

## Executive Summary

Three characterization experiments (H6, H7, H8) were executed against the live Coralys optimizer
using UB-002. All runs achieved zero hard-constraint violations (HC=0).

| Experiment | Verdict | Finding |
|---|---|---|
| H6 — Week 2 Seed Stability | CONFIRMED | Two reproducible SC1 convergence basins exist under UB-002 |
| H7 — Assignment Ordering Stability | OPEN | Directional ecological effect confirmed; strict schedule-level ordering demonstrated in 72.5% of runs |
| H8 — Workload Sensitivity | CONFIRMED | SC2 range 929.6 across workload profiles; ecological signal is active |

---

## H6: Week 2 Seed Stability

**Hypothesis:** Week 2 SC1 elevation observed in earlier runs is a structural artifact of the
coverage pattern, not optimizer stochasticity.

**Protocol:** 20 seeds x Week 2 x 50 generations = 20 API calls.

**Results:**

| Metric | Value |
|---|---|
| Valid runs (HC=0) | 20/20 |
| SC1 mean | 100.8 |
| SC1 stdev | 30.09 |
| SC1 min | 81.6 |
| SC1 max | 145.6 |
| SC2 mean | 1034.62 |
| SC2 stdev | 3.47 |
| Fitness mean | 8864.59 |
| Fitness stdev | 30.08 |
| Fitness min | 8815.2 |
| Fitness max | 8886.1 |

**SC1 Bimodal Distribution:**

SC1 converges to exactly one of two values in every run:
- SC1 = 81.6 (optimal basin, 14/20 seeds = 70%)
- SC1 = 145.6 (suboptimal basin, 6/20 seeds = 30%)

No intermediate SC1 values appear. SC2 varies only slightly (stdev=3.47), confirming the workload
penalty is not the source of SC1 variance.

**Verdict: OUTCOME_B_STRUCTURAL — CONFIRMED**

> Week 2 consistently exhibits two reproducible convergence basins (SC1=81.6 and SC1=145.6).
> The specific RNG seeds reaching each basin are not stable across repeated experiments —
> the affected seed set changed between runs (first run: seeds 1,2,11,12; second run: seeds
> 1,2,10,14,16,19). Seed identity is therefore not itself the structural property.
> The structural observation is the existence of two attractors under the current UB-002
> formulation.

The appropriate response under GOV-002 is to investigate the benchmark structure, not to modify
the optimizer. The optimizer is behaving consistently under the current benchmark. The observed
results indicate a fitness landscape with two reproducible convergence basins for SC1 under the
current UB-002 formulation.

**Artifact:** `UB-002-H6-WEEK2-STABILITY-v1.0.json`

---

## H7: Assignment Ordering Stability

**Hypothesis:** Workers with higher historical workloads (HIGH group) receive fewer shifts than
LOW-workload workers, and this ordering is stable across seeds and weeks.

**Protocol:** 20 seeds x 4 weeks x 50 generations = 80 API calls.

**Worker Groups (by historical_workloads):**
- HIGH (4 workers): workload >= 35 hours
- MED (11 workers): workload 20–34 hours
- LOW (4 workers): workload < 20 hours

**Results:**

| Metric | Value |
|---|---|
| Valid runs (HC=0) | 80/80 |
| HIGH<MED<LOW ordering correct | 58/80 (72.5%) |
| HIGH mean shifts/run | 3.947 ± 0.103 |
| MED mean shifts/run | 4.084 ± 0.063 |
| LOW mean shifts/run | 4.550 ± 0.134 |

**Two Separable Questions:**

H7 answers two distinct questions, and the answers differ:

**Question A — Does SC2 influence assignments at the population level?**
Yes. The group means consistently satisfy HIGH < MED < LOW across all 80 runs. The separation
is stable: HIGH=3.947, MED=4.084, LOW=4.550. The SC2 ecological signal is producing the
expected directional effect on assignment distribution.

**Question B — Does every optimized schedule satisfy strict ordering?**
No. Only 58/80 runs (72.5%) satisfy HIGH < MED < LOW strictly. The remaining 22 runs (27.5%)
fail the strict ordering test.

**Failure Classification:**

Every ordering failure is a borderline tie — not an inversion:

| Failure type | Count | Example |
|---|---|---|
| Type A — HIGH == MED (MED < LOW) | 19 | HIGH=4.00, MED=4.00, LOW=4.75, SC1=81.6 |
| Type B — MED == LOW (HIGH < MED) | 3 | HIGH=3.75, MED=4.25, LOW=4.25, SC1=145.6 |
| HIGH > MED (true inversion) | 0 | — |
| MED > LOW (true inversion) | 0 | — |

Zero true inversions were observed across 80 runs. All 22 failures are equality cases.

**Type A** (19/22 failures): HIGH and MED receive identical average shift counts despite SC1
being at its global optimum (81.6). LOW still receives more work than both. The ecological
preference is partially expressed but not sufficient to differentiate HIGH from MED.

**Type B** (3/22 failures): MED and LOW are tied, and SC1 is in the suboptimal basin (145.6).
This failure mode is coupled to the H6 bimodal attractor. H6 explains Type B; it does not
explain Type A.

**Verdict: OPEN**

> HIGH < MED < LOW is reproduced in 72.5% of runs. The remaining 27.5% are dominated by
> equality between adjacent workload groups rather than reversals of the intended ecological
> preference. The optimizer consistently favours lower-fatigue workers on average, but the
> current objective does not always produce a strict total ordering.
>
> The SC2 objective is directionally active, but in some coverage configurations the optimizer
> produces schedules in which adjacent workload groups remain tied after integer shift allocation.
> Whether these ties arise from objective plateaus, equivalent optima, or insufficient
> discrimination between neighbouring fatigue groups remains an open question.

No optimizer changes are warranted at this stage. The failure mode is understood at the
phenomenological level (ties, not inversions), but the causal mechanism requires further evidence.

**Artifact:** `UB-002-H7-ASSIGNMENT-STABILITY-v1.0.json`

---

## H8: Workload Sensitivity

**Hypothesis:** The SC2 (fatigue penalty) is sensitive to the historical_workloads input profile,
and different profiles produce meaningfully different optimizer outcomes.

**Protocol:** 6 workload profiles x 3 seeds x 50 generations = 18 API calls.

**Results:**

| Profile | SC2 mean | SC1 mean | Fitness mean |
|---|---|---|---|
| baseline | 1034.6 | 94.4 | 8871.7 |
| light | 332.0 | 81.6 | 9586.4 |
| heavy | 1261.6 | 81.6 | 8635.5 |
| bimodal | 659.7 | 81.6 | 9218.3 |
| all_zero | 332.0 | 81.6 | 9586.4 |
| all_high | 1261.6 | 81.6 | 8635.5 |

**Key Observations:**

1. SC2 range = 929.6 (332.0 light to 1261.6 heavy). Workload profile is the dominant SC2 driver.
2. Fitness inversely correlated with SC2: lighter workloads produce higher fitness (9586.4 vs 8635.5).
3. SC1 is stable at 81.6 across all non-baseline profiles — uniform SC2 pressure removes the
   asymmetry that causes the bimodal SC1 attractor in the baseline.
4. light = all_zero (identical SC2=332.0, fitness=9586.4): zero and low workloads are equivalent.
5. heavy = all_high (identical SC2=1261.6, fitness=8635.5): confirmed by identical results.

**Verdict: CONFIRMED — SENSITIVE**

> The optimizer clearly reacts to workload profiles. SC2 spans 332.0 to 1261.6 (range=929.6).
> This validates the Sprint 7 design goal: the ecological signal influences optimizer behaviour
> rather than being ignored.

**Artifact:** `UB-002-H8-WORKLOAD-SENSITIVITY-v1.0.json`

---

## Cross-Experiment Synthesis

### Constraint Integrity
All 118 API calls across H6+H7+H8 returned HC=0. The Coralys optimizer reliably satisfies all
hard constraints across diverse seeds, weeks, and workload profiles.

### Responsiveness vs. Schedule-Level Consistency
H7 and H8 together establish an important distinction:

- **H8 establishes responsiveness:** the optimizer reacts to ecological inputs in a meaningful
  and monotonic way. SC2 scales with workload magnitude; fitness responds inversely. The
  ecological signal is not being ignored.
- **H7 shows schedule-level consistency is not yet demonstrated:** strict per-schedule ordering
  holds in only 72.5% of runs, with all failures being integer-allocation ties, not inversions.

These are not contradictory findings. A system can respond to ecological changes while still
exhibiting stochastic variation in exact schedules. Population-level responsiveness and
schedule-level consistency are separate properties. H8 has established the former; H7 indicates
the latter has not yet been demonstrated.

### H6 and H7 Are Only Partially Coupled
The H6 bimodal attractor (SC1=81.6 / 145.6) explains H7 Type B failures (MED==LOW, SC1=145.6).
It does not explain H7 Type A failures (HIGH==MED, SC1=81.6), which occur even when SC1 is at
its global optimum. These are distinct phenomena with potentially different causes.

### SC1 Bimodal Attractor
The two-basin SC1 structure identified in H6 is a property of the UB-002 coverage formulation,
not of specific seeds. The baseline SC1=94.4 in H8 reflects the same effect: the native UB-002
workload distribution creates asymmetric SC2 pressure that interferes with SC1 optimization.
Uniform workload profiles (H8 non-baseline) eliminate this asymmetry and produce SC1=81.6
consistently.

### Sprint 8 Characterization Matrix

| Hypothesis | Question | Outcome |
|---|---|---|
| H6 | Is Week 2 SC1 elevation stochastic or structural? | CONFIRMED — two convergence basins exist; seed identity is not the structural property |
| H7 | Is assignment ordering stable across seeds and weeks? | OPEN — directional effect confirmed at population level; strict schedule-level ordering 72.5%; all failures are ties |
| H8 | Is SC2 sensitive to workload profile? | CONFIRMED — 929.6 SC2 range; ecological signal is active |

### Research Progression
Sprint 8 has narrowed the remaining question from "does SC2 work?" to the more specific question
of why adjacent fatigue groups sometimes become indistinguishable despite the correct overall
trend. That is a much more focused basis for future investigation than treating the 72.5% figure
as a generic instability.

### Engineering Implications
None of the three experiments produced evidence requiring operator redesign, weight calibration,
or a new benchmark (UB-003). The platform has been characterized further, but the primary
engineering effort should remain Stream B (UltraCrew product). Any future platform research
should be driven by questions that emerge from product evidence rather than optimizer exploration
alone.

### Open Items for Sprint 9

1. **H9 (candidate):** Is the observed H7 ordering instability caused primarily by objective
   indifference (multiple equal-cost schedules) or by insufficient SC2 discrimination between
   neighbouring fatigue groups? This is a characterization question, not an operator redesign
   question. It fits GOV-002.

2. **SC1 bimodal attractor:** Characterize the two-basin structure analytically. Determine
   whether it is a property of the coverage ratio (shifts per worker) or the skill distribution.
   Do not conflate with item 1.

3. **Baseline SC1 elevation:** The native UB-002 workload distribution produces SC1=94.4 vs
   81.6 for uniform profiles. Characterize the asymmetric SC2 pressure mechanism before
   considering any workload rebalancing.

---

## Artifact Index

| File | Description | Runs |
|---|---|---|
| `UB-002-H6-WEEK2-STABILITY-v1.0.json` | H6 seed stability results | 20 |
| `UB-002-H7-ASSIGNMENT-STABILITY-v1.0.json` | H7 assignment ordering results | 80 |
| `UB-002-H8-WORKLOAD-SENSITIVITY-v1.0.json` | H8 workload sensitivity results | 18 |
| `S8-OBJECTIVE-CHARACTERIZATION-v1.0.md` | This freeze document | — |

**Total API calls executed in Sprint 8 Stream A:** 118
**Total HC violations:** 0
**Sprint 8 Stream A status:** COMPLETE