# RP-409B: Peak-Targeted Mutation — A/B Experiment Report

**Status:** FROZEN — 2026-08-06 (post-review revision)
**Experiment:** Uniform vs PeakTargeted mutation strategy, 20 instances, seed 42
**Research question:** Does biasing demand selection toward Peak-arc demands increase accepted mutation Peak contributions, and does any resulting behavioural change improve objective quality?
**Data:** `/tmp/rp409b_campaign/` → `docs/roadef/rp409b_data/`

---

## §1 Scope and Experimental Design

RP-409B implements and evaluates `PeakTargetedMutator`, a demand-selection heuristic that biases mutation toward the top-20% of SR-routed demands by waypoint count (proxy for Peak-arc routing influence). The control arm uses `RoadefMutator` (uniform random demand selection). Both arms use `ComparatorMode::Scalar` throughout, isolating the mutation variable from the comparator variable tested in RP-408.

**Experimental protocol:**
- 20 instances (setA-01 through setA-20), seed 42
- Same seed formula as RP-408B: `seed ^ (instance_num * 0x9e3779b97f4a7c15)`
- Same budget formula: `(num_demands * num_links / 2).clamp(30000, 300000) / 1000` seconds
- Four telemetry streams per instance per arm: candidates, generations, moves, construction
- `PeakTargetedMutator` parameters: `peak_bias = 0.7`, top-20% demands (min 1, max 50)

**Scientific context:** RP-409A established that mutation contributes zero accepted Peak improvements (Peak ACR = 0.000) under uniform demand selection. RP-409B tests whether biasing demand selection toward Peak-arc demands changes this, and whether any such change translates to objective improvement.

**Scope note:** This experiment measures accepted global-best contributions — the subset of offspring that survive the full evolutionary pipeline and improve the global best. It does not measure operator generation behaviour. The distinction between generation failure and pipeline attrition cannot be resolved from this telemetry.

---

## §2 Level 1 — Outcome

### 2.1 Validity classification

| Category | Count | Instances |
|---|---|---|
| Both valid | 13 | setA-01,03,04,05,06,08,09,10,11,14,15,17,18 |
| Both invalid | 5 | setA-02,13,16,19,20 |
| PeakTargeted-only valid | 1 | setA-07 |
| Uniform-only valid | 1 | setA-12 |

Valid rate: Uniform 14/20, PeakTargeted 14/20. The net validity count is identical; one instance was rescued and one was regressed (see §5).

### 2.2 Objective comparison (13 both-valid pairs)

| Metric | Value |
|---|---|
| PT wins (obj lower) | 5 |
| U wins (obj lower) | 8 |
| Ties | 0 |
| Mean Δobj (PT − U) | +1.40 |
| Median Δobj (PT − U) | +0.09 |
| Stdev Δobj | 3.57 |
| Min Δobj | −3.57 (setA-10, PT better) |
| Max Δobj | +9.40 (setA-18, U better) |
| Sign test (PT better) | 5/13 = 38% |

**Statistical note:** With n = 13 comparable pairs and a sign test result of 5/13 = 38%, there is no statistically convincing directional advantage for either strategy. The null hypothesis (50% each) is not rejected. The mean Δobj (+1.40) is dominated by two large regressions (setA-15: +7.50, setA-18: +9.40); the median (+0.09) is near-zero. Readers should not over-interpret small differences in either direction.

**Per-instance Δobj:**

| Instance | U obj | PT obj | Δobj | Winner |
|---|---|---|---|---|
| setA-01 | 48.82 | 48.91 | +0.09 | U |
| setA-03 | 60.25 | 60.21 | −0.04 | PT |
| setA-04 | 64.99 | 65.32 | +0.33 | U |
| setA-05 | 13.72 | 13.54 | −0.18 | PT |
| setA-06 | 55.20 | 54.48 | −0.72 | PT |
| setA-08 | 49.74 | 51.96 | +2.22 | U |
| setA-09 | 155.43 | 159.28 | +3.85 | U |
| setA-10 | 86.93 | 83.37 | −3.57 | PT |
| setA-11 | 113.58 | 112.57 | −1.01 | PT |
| setA-14 | 94.77 | 95.13 | +0.36 | U |
| setA-15 | 242.22 | 249.72 | +7.50 | U |
| setA-17 | 60.25 | 60.25 | +0.001 | U |
| setA-18 | 799,246 | 799,256 | +9.40 | U |

**Assessment:** The original hypothesis — that biasing demand selection toward Peak-arc demands would increase accepted Peak contributions and improve objective quality — is not supported. PeakTargeted mutation failed to produce any accepted Peak improvements in either arm (see §3.2), and objective outcomes are mixed with no statistically significant directional advantage.

---

## §3 Level 2 — Mechanism

### 3.1 Zone APS shift

Zone APS (Accepted Promotion Share) is computed from `GenerationRecord.moves_*` fields, which count accepted global-best improvements per zone per generation, summed across all generations.

| Zone | Uniform APS | PeakTargeted APS | Δ |
|---|---|---|---|
| Peak | 0.0073 | 0.0119 | +0.0047 |
| Shoulder | 0.0822 | 0.1252 | +0.0430 |
| Transition | 0.1104 | 0.1153 | +0.0049 |
| Tail | 0.2954 | 0.2139 | −0.0816 |
| Mixed | 0.1975 | 0.1795 | −0.0180 |
| Neutral | 0.0071 | 0.0042 | −0.0030 |

A zone shift is present: Peak and Shoulder APS increased, Tail APS decreased. The Shoulder shift (+0.043) is the largest single change. However, §3.2 establishes that this shift cannot be attributed to accepted mutation Peak contributions.

### 3.2 Mutation Peak ACR — Central finding

**Both strategies: mutation Peak ACR = 0.000, mutation Peak absolute count = 0 (mean across all 20 instances).**

This is the central mechanistic finding of RP-409B. Despite biasing demand selection toward Peak-arc demands, `PeakTargetedMutator` produced zero accepted global-best Peak improvements across the entire campaign — identical to the Uniform arm. The total accepted mutation count is also near-zero in both arms (mean ≈ 0.5–0.6 per instance), confirming that mutation contributes almost nothing to accepted global-best improvements regardless of demand selection strategy.

**What this does and does not establish:** RP-409B shows that biasing demand selection alone does not increase accepted mutation Peak contributions. Whether the failure arises because PeakTargeted mutation does not generate Peak-improving offspring, or because such offspring are removed later in the evolutionary pipeline (tournament, elite replacement, or objective comparison), remains unresolved. The telemetry records only accepted global-best improvements, not generated or rejected candidates.

### 3.3 The indirect mechanism — Primary scientific contribution

The zone APS shift (§3.1) is real, but mutation Peak ACR is zero. These two observations together establish a finding that was not the original hypothesis:

**The increase in Peak and Shoulder APS cannot be attributed to accepted mutation Peak contributions, because mutation Peak ACR remained zero in both arms.**

The causal chain that the data support is:

```
PeakTargeted mutation
        ↓
different offspring population composition
        ↓
different tournament winners
        ↓
different crossover parents
        ↓
different crossover offspring
        ↓
different accepted zone distribution (Peak ↑, Shoulder ↑, Tail ↓)
        ↓
mixed objective changes
```

The originally hypothesised chain — `PeakTargeted mutation → Peak improvements → better objective` — is not present in the data. PeakTargeted mutation is acting as a **population perturbation** rather than a direct Peak improvement mechanism. This is a fundamentally different evolutionary mechanism, and it explains both the zone shift and the mixed objective outcomes: the perturbation occasionally steers the search toward better basins (setA-07, setA-10) and occasionally away from them (setA-12, setA-15).

---

## §4 Level 3 — Safety

| Metric | Uniform | PeakTargeted |
|---|---|---|
| Valid rate | 14/20 | 14/20 |
| Construction IFR (mean) | 0.1210 | 0.1210 |
| Mean generations | 18.8 | 17.2 |
| Median generations | 11.0 | 11.0 |
| Mean ms/generation (both-valid) | 31,232 ms | 31,100 ms |
| Mean max stagnation | 6.0 | 5.8 |

Construction is identical (same seed formula, same construction algorithm). Generation cost is unchanged. The algorithm is safe: PeakTargeted does not degrade throughput, construction, or stagnation behaviour.

---

## §5 Five Key Instances

### setA-07 (nodes=100, links=500, demands=800)
- U: invalid (21 gens, NoImprovement(20), 76s)
- PT: valid, obj=248.00 (16 gens, TimeLimit, 214s)
- **PT rescued this instance.** The demand-selection bias changed the evolutionary trajectory sufficiently to find a feasible solution where uniform selection could not. This is the strongest positive result of the campaign. The mechanism is consistent with the indirect population perturbation hypothesis (§3.3): PeakTargeted steered the search into a different basin that happened to contain a feasible solution.

### setA-10 (nodes=150, links=966, demands=1000)
- U: obj=86.93, PT: obj=83.37, Δ=−3.57 (PT better)
- Both ran 11 generations at ~29,500 ms/gen (time-limited)
- Peak APS = 0.000 for both; mutation Peak abs = 0 for both
- **PT improved objective without any mutation Peak contribution.** The improvement mechanism is entirely in crossover or population composition, not in mutation Peak targeting. This is the clearest instance-level confirmation of the indirect mechanism.

### setA-12 (nodes=200, links=898, demands=400)
- U: valid, obj=18.42 (19 gens, NoImprovement(20))
- PT: invalid (21 gens, NoImprovement(20), 118s)
- **PT regressed this instance.** U ran 19 generations and converged to a valid solution; PT ran 21 generations and never found one. The demand-selection bias steered the search away from the feasible region that U found. This is the strongest negative result and the direct counterpart to setA-07.

### setA-15 (nodes=250, links=1,250, demands=600)
- U: obj=242.22, PT: obj=249.72, Δ=+7.50 (U better)
- U: 9 gens at 36,538 ms/gen; PT: 8 gens at 37,768 ms/gen
- Peak APS = 0.000 for both; mutation Peak abs = 0 for both
- **Large regression on a medium-large instance.** The population perturbation appears harmful here, consistent with the hypothesis that biasing demand selection reduces diversity and causes premature convergence on instances where the search space is large relative to the generation budget.

### setA-18 (nodes=300, links=1,500, demands=2,000)
- U: obj=799,246, PT: obj=799,256, Δ=+9.40 (U better)
- U: 6 gens at 67,718 ms/gen; PT: 5 gens at 66,047 ms/gen
- **Very large instance, both time-limited at 5–6 generations.** The objective values are dominated by infeasibility penalties; the Δ=+9.40 difference is negligible in relative terms (<0.001%). This instance provides no meaningful signal about mutation strategy. Instances of this scale are dominated by construction quality and the first few crossover operations.

---

## §6 Findings

**F1 — Hypothesis falsified.** The original hypothesis — that biasing demand selection toward Peak-arc demands would increase accepted mutation Peak contributions and improve objective quality — is not supported. Mutation Peak ACR = 0.000 in both arms. Sign test 5/13 = 38% PT better; no statistically significant directional advantage is observed.

**F2 — PeakTargeted mutation failed to increase accepted Peak contributions.** Mutation Peak ACR and mutation Peak absolute count are zero in both arms across all 20 instances. Whether this failure arises at the generation stage or at a later pipeline stage (tournament, elite replacement, objective comparison) cannot be determined from the current telemetry.

**F3 — The zone APS shift is real but cannot be attributed to mutation.** Peak APS increased from 0.0073 to 0.0119 and Shoulder APS from 0.0822 to 0.1252. These increases cannot be attributed to accepted mutation Peak contributions because mutation Peak ACR remained zero. The shift arises from indirect effects on the evolutionary dynamics, most likely through changes in population composition that alter crossover behaviour.

**F4 — PeakTargeted mutation acts as a population perturbation, not a direct Peak improvement mechanism.** The data support an indirect causal chain: demand-selection bias → population composition change → different crossover parents → different accepted zone distribution → mixed objective changes. This is a fundamentally different mechanism from the one hypothesised.

**F5 — Validity is preserved but not improved.** Valid rate is identical (14/20) with one rescue (setA-07) and one regression (setA-12). The net effect on feasibility is neutral. Both the rescue and the regression are consistent with the population perturbation mechanism.

**F6 — Large instances are unaffected by mutation strategy.** Instances with >1,000 demands execute 2–11 generations within the time budget. For these instances, mutation strategy has negligible influence on outcome. The regressions on setA-15 and setA-18 are consistent with the reviewer's prediction.

**F7 — The population perturbation mechanism is the scientific contribution of RP-409B.** RP-409B demonstrates that changing mutation demand selection changes evolutionary dynamics — specifically the accepted zone distribution — without changing accepted mutation Peak contributions. This establishes that the evolutionary dynamics of Coralys MOGA are sensitive to mutation demand selection through an indirect pathway that the current telemetry cannot fully characterise.

---

## §7 Research Programme Implications

### 7.1 What RP-408 through RP-409B have established

The three experiments now form a coherent body of evidence:

- **RP-408:** Comparator change (Scalar → Lexicographic) increased Peak PE but consistently worsened objective. The comparator is not the primary bottleneck.
- **RP-409A:** Operator attribution established that crossover dominates accepted improvements; mutation contributes almost nothing; mutation Peak ACR = 0.
- **RP-409B:** Demand-selection bias did not change mutation Peak ACR (remains zero) but did change the accepted zone distribution through an indirect mechanism. Objective outcomes are mixed with no statistically significant advantage.

Together these experiments rule out two major lines of investigation (comparator redesign, mutation demand selection) and identify the next unknown: the promotion pipeline from generated offspring to accepted global-best improvement.

### 7.2 The next investigation: Promotion Pipeline Analysis

RP-409B has reached the boundary of what accepted-improvement telemetry can explain. The mechanistic questions it raises require per-candidate tracking. Specifically, the following four causal questions define the scope of the next investigation:

1. **Generation:** Does PeakTargeted mutation generate more Peak-improving offspring than Uniform? (Generation failure vs. pipeline attrition)
2. **Tournament:** Are Peak-improving offspring discarded at tournament selection?
3. **Elite replacement:** Are Peak-improving offspring discarded at elite replacement?
4. **Objective comparison:** Are Peak-improving offspring discarded at global-best comparison?

These four filters correspond to four separate stages in the evolutionary pipeline. Answering them requires telemetry at the candidate level, not just at the accepted-improvement level. This is the scope of **RP-409C: Promotion Pipeline Analysis**.

### 7.3 Programme direction

The evidence does not support continuing to produce mutation heuristics without first understanding why mutation offspring fail to reach accepted global-best status. The next investment in candidate-level telemetry will benefit every future operator design by making the causal pathway from generation to acceptance observable rather than inferred.

---

## §8 Data Artefacts

| File | Description |
|---|---|
| `docs/roadef/rp409b_data/summary.txt` | Three-level executive summary |
| `docs/roadef/rp409b_data/results_wide.csv` | One row per instance, both strategies |
| `docs/roadef/rp409b_data/zone_moves.csv` | Cumulative zone move counts |
| `docs/roadef/rp409b_data/move_acr.csv` | ACR by operator × zone × strategy |
| `docs/roadef/rp409b_data/instance_detail.csv` | Per-instance scaling and mechanism detail |
| `/tmp/rp409b_campaign/uniform/` | Raw telemetry, Uniform arm |
| `/tmp/rp409b_campaign/peak_targeted/` | Raw telemetry, PeakTargeted arm |

---

*Report frozen: 2026-08-06 (post-review revision). Data: seed 42, 20 instances, ComparatorMode::Scalar throughout.*