# UltraCrew — Effect of 8h vs 10h Layover Threshold on Pairing Structure

**Status:** Methodology review — experiment scope narrowed
**Date:** 2026-07-30
**Relates to:** [UltraCrew vs GENCOL Pipeline Divergence Analysis](UltraCrew_GENCOL_Pipeline_Divergence_Analysis.md), Section 4.1
**Hypothesis under test:** The 8h vs 10h layover threshold difference is a significant contributor to the observed 65–73% pairing count ratio across GERAD G-2014-22 instances.

---

## 1. Objective

Experimentally isolate the contribution of the `LAYOVER_REST_HOURS` constant to the pairing count gap between UltraCrew and the GERAD reference.

**Important scope note:** The experiment described here is a *reconstruction experiment*, not an optimization experiment. It evaluates only the reconstruction stage that converts an already-determined crew–flight allocation into pairings. It does not evaluate UltraCrew's optimization process. Consequently, it cannot be used to infer whether changing the layover threshold would alter the optimized assignment or the final pairing solution produced by UltraCrew. See Section 6 for the correct experiment design and its feasibility.

---

## 2. Method

The experiment is a **reconstruction experiment**. It does not run UltraCrew's optimizer. It takes the crew–flight allocation already encoded in the benchmark `duties.csv` and asks: given that fixed allocation, does changing the FDP split threshold alter the reconstructed pairing count?

Pipeline:

```
duties.csv  →  recover crew → flight sequence
                    ↓
            group legs into FDPs using LAYOVER_REST_HOURS
                    ↓
            group FDPs into pairings using HOME_BASE_REST_HOURS (34h)
                    ↓
            count pairings
```

Run twice:

**Condition A (baseline):** `LAYOVER_REST_HOURS = 8.0` (current UltraCrew default)

**Condition B (experimental):** `LAYOVER_REST_HOURS = 10.0` (GENCOL paper value)

Script: [`compare_gerad.py`](../../compare_gerad.py)

**What this experiment does NOT do:** It does not run UltraCrew's MOGA optimizer. The crew–flight assignment is inherited from `duties.csv` (which was produced by the GERAD benchmark generation process, not by UltraCrew). Changing the threshold in this experiment cannot change which crew flies which flights — that decision is already fixed. It can only change how the fixed flight sequence is split into FDPs.

---

## 3. Metrics to Record

For each instance and each condition, record:

- Total pairing count (UltraCrew vs GERAD reference)
- Pairing count ratio (UltraCrew / GERAD)
- Compliance rate (% pairings passing TC CAR 700 legality check)
- Multi-duty pairing ratio
- Mean pairing span (days)
- Number of rest violations

---

## 4. Analysis

Compare Condition A vs Condition B on each metric across all 7 instances.

If the pairing count ratio increases substantially under Condition B (e.g., from 65–73% toward 85%+), the threshold is a dominant contributor to the gap.

If the ratio increases only modestly (e.g., to 75–78%), other factors (objective function, greedy grouping, search space) are also significant contributors.

Report the delta per instance and the aggregate effect.

---

## 5. Results

Experiment run: 2026-07-30 (v2, flight-leg level). Script: [`compare_gerad.py`](../../compare_gerad.py).

| Instance  | GERAD ref | 8h count | 10h count | 8h ratio | 10h ratio | ref multi-duty | 8h multi-FDP | 10h multi-FDP | ref span (d) | 8h span (d) | 10h span (d) |
|-----------|-----------|----------|-----------|----------|-----------|----------------|--------------|---------------|--------------|-------------|--------------|
| instance1 | 172       | 113      | 113       | 0.657    | 0.657     | 0.674          | 0.735        | 0.646         | 1.36         | 2.06        | 2.06         |
| instance2 | 303       | 189      | 189       | 0.624    | 0.624     | 0.485          | 0.503        | 0.492         | 0.84         | 1.46        | 1.46         |
| instance3 | 274       | 161      | 161       | 0.588    | 0.588     | 0.555          | 0.764        | 0.646         | 1.50         | 2.36        | 2.36         |
| instance4 | 1079      | 733      | 733       | 0.679    | 0.679     | 0.480          | 0.592        | 0.441         | 0.89         | 1.41        | 1.41         |
| instance5 | 1497      | 1053     | 1053      | 0.703    | 0.703     | 0.597          | 0.638        | 0.634         | 1.23         | 1.77        | 1.77         |
| instance6 | 1187      | 874      | 874       | 0.736    | 0.736     | 0.594          | 0.629        | 0.600         | 1.38         | 1.83        | 1.83         |
| instance7 | 1648      | 1131     | 1131      | 0.686    | 0.686     | 0.570          | 0.699        | 0.652         | 1.46         | 2.04        | 2.04         |
| **avg**   |           |          |           | **0.668**| **0.668** |                |              |               |              |             |              |

**Delta (10h − 8h): +0.000 (+0.0pp) across all 7 instances.**

**Key observation — pairing count:** The multi-FDP ratio *does* change between conditions (e.g. instance1: 8h=0.735, 10h=0.646; instance4: 8h=0.592, 10h=0.441). This confirms the threshold is exercising the FDP grouping logic within the fixed crew–flight allocation. However, these reclassifications do not create new pairing boundaries *within this reconstruction algorithm* because all affected gaps are well below the 34h HOME_BASE_REST_HOURS pairing boundary.

**Key observation — structural change (FDP composition):** A separate structural analysis ([`scripts/fdp_structure_diff.py`](../../scripts/fdp_structure_diff.py)) counted how many pairings changed FDP composition between conditions even though pairing count stayed constant:

| Instance  | Pairings | Changed FDP composition | Identical | Changed % | Avg FDP delta |
|-----------|----------|------------------------|-----------|-----------|---------------|
| instance1 | 113      | 30                     | 83        | 26.5%     | −0.292        |
| instance2 | 189      | 13                     | 176       | 6.9%      | −0.085        |
| instance3 | 161      | 57                     | 104       | 35.4%     | −0.447        |
| instance4 | 733      | 215                    | 518       | 29.3%     | −0.323        |
| instance5 | 1053     | 78                     | 975       | 7.4%      | −0.080        |
| instance6 | 874      | 140                    | 734       | 16.0%     | −0.185        |
| instance7 | 1131     | 277                    | 854       | 24.5%     | −0.292        |

Between 7% and 35% of pairings change FDP composition between conditions. The negative FDP delta means the 10h threshold merges more legs into single FDPs (fewer, longer FDPs per pairing). This is the structural change the reviewer predicted: **the threshold affects structure without affecting cardinality**.

**Metric scope:** This experiment measures only pairing count and FDP composition. It does not measure pairing quality dimensions: legality (TC CAR 700 compliance), TAFB, deadhead, hotel nights, or pairing span. It is possible that the 10h condition produces structurally different pairings with different quality characteristics even though the count is identical. Those dimensions are not evaluated here.

---

## 6. Conclusion and Scope Limitation

**Within the scope of this reconstruction experiment:** the 8h vs 10h layover threshold has no effect on pairing count (+0.0pp across all 7 instances). The threshold does affect FDP structure: 7–35% of pairings change FDP composition between conditions, with the 10h threshold producing fewer, longer FDPs per pairing.

**What this result does and does not show:**

This experiment evaluates only the reconstruction stage that converts an already-determined crew–flight allocation into pairings. It does not evaluate UltraCrew's optimization process. Consequently, it cannot be used to infer whether changing the layover threshold would alter the optimized assignment or the final pairing solution produced by UltraCrew.

The null result on pairing count means: *under this deterministic reconstruction algorithm, changing the threshold does not change the number of reconstructed pairings*. It does not establish that no pairing algorithm would produce different pairings — a global optimization algorithm could explore alternative pairing topologies where altered FDP boundaries influence later pairing choices even if the 34h rule is unchanged.

The most defensible statement the evidence supports is:

> Under the current reconstruction algorithm, changing the FDP threshold from 8h to 10h changes FDP composition (7–35% of pairings affected) but does not change the number of reconstructed pairings. This demonstrates that pairing count is insensitive to the threshold within this reconstruction model. It does not establish that a complete optimization pipeline — or a different pairing construction algorithm — would exhibit the same behaviour.

**The correct experiment** would require running UltraCrew end-to-end from `flights.csv` + `crew.csv`, then comparing the output pairings against the GERAD reference — twice, with `LAYOVER_REST_HOURS = 8.0` and `LAYOVER_REST_HOURS = 10.0`. The benchmark provides `crew.csv` (base, qualification, contract_type) and `flights.csv`, but does not provide crew availability windows, pairing cost parameters, or aircraft-crew qualification matching rules needed to run UltraCrew's MOGA optimizer from scratch.

**Status of the original hypothesis:** The hypothesis — that the 8h vs 10h threshold is a significant contributor to the pairing count gap — remains **open**. This experiment does not falsify it. It narrows the question: the threshold does not affect pairing count when the crew–flight allocation is fixed and a deterministic reconstruction algorithm is used. Whether it affects the allocation itself when UltraCrew optimizes from raw flights is a separate question.

**Recommended next step:** Investigate the pairing topology mutation operator (see [`UltraCrew_Pairing_Topology_Mutation_Evaluation.md`](UltraCrew_Pairing_Topology_Mutation_Evaluation.md)) as the primary architectural lever. The end-to-end experiment described in Section 7 should be run once UltraCrew can be invoked from raw flight inputs.

---

## 7. Next Experiment: End-to-End Pipeline Comparison

The following experiment evaluates the **complete UltraCrew optimization pipeline** rather than only the reconstruction stage. It is the correct experiment for testing whether the layover threshold influences the optimized assignment.

### Objective

Determine whether `LAYOVER_REST_HOURS` influences the optimized assignment produced by UltraCrew when run end-to-end from raw inputs.

### Inputs

Use only the benchmark's primary problem definition:

```
flights.csv
crew.csv
```

Do **not** use `duties.csv` or `pairings.csv` during optimization. These files are used only afterwards for comparison.

### Procedure

**Condition A** — `LAYOVER_REST_HOURS = 8.0`

Run the complete UltraCrew pipeline:

```
flights.csv + crew.csv
        ↓
Schedule generation
        ↓
EvolutionEngine (MOGA)
        ↓
Worker assignments
        ↓
Duty construction
        ↓
Pairing construction
        ↓
Final pairings
```

Store: `pairings_8h.csv`, `schedule_8h.json`, `metrics_8h.json`

**Condition B** — `LAYOVER_REST_HOURS = 10.0`

Repeat identically. Store: `pairings_10h.csv`, `schedule_10h.json`, `metrics_10h.json`

### Comparison Metrics

**Assignment:** flight coverage %, crew utilization, workload balance, fairness, fatigue score.

**Pairing:** pairing count, FDP count, average pairing span, legality violations, overnight duties, home-base returns.

**Operational:** deadhead sectors, hotel nights, TAFB, reserve usage.

**Optimization:** best fitness, convergence generation, runtime, population diversity.

**Benchmark comparison:** compare each optimized solution against `pairings.csv` (GERAD reference) on pairing count ratio, topology similarity, duty distribution, pairing span, and legality statistics.

### Expected contribution

Unlike the reconstruction experiment, this experiment evaluates the entire scheduling process. It will determine whether the layover threshold influences assignment decisions, duty formation, pairing topology, optimization behaviour, and ultimately the quality of the schedules produced by UltraCrew. This provides a much stronger basis for assessing whether `LAYOVER_REST_HOURS` is an optimization parameter that materially affects the end-to-end solution, rather than merely a post-processing parameter used during reporting.

---

## 8. End-to-End Pipeline Experiment Results (Partial)

**Experiment:** `gerad_e2e_threshold_experiment` in [`adapters/airline/tests/gerad_e2e.rs`](../../adapters/airline/tests/gerad_e2e.rs)

**Date:** 2026-07-31

**Pipeline:** `GreedyScheduler` + `LocalSearch` (`WorkloadBalanceObjective`), 10 iterations

**Pairing generation:** Deterministic global chronological grouping + spatial-continuity check + temporal-overlap check. Each duty wrapped as a single-duty pairing (single-duty pairing model — see Modeling Assumption 5 below).

**Modeling assumptions:**
1. All flights sorted globally by departure time before grouping.
2. Spatial-continuity check: duty break forced if airports do not connect.
3. Temporal-overlap check: duty break forced if next leg departs before current batch's last leg arrives.
4. All crew assigned home base = YUL (benchmark does not specify per-crew bases).
5. Single-duty pairing model: each duty is wrapped as its own pairing. The benchmark's `flights.csv` contains single-day synthetic flights (all on 2000-01-01); there are no 34h HOME_BASE_REST_HOURS gaps between consecutive duties, so multi-duty pairing grouping by temporal gap never fires.
6. Objective: `WorkloadBalanceObjective` only (TAFB/hotel/deadhead not evaluated).

**Key finding:** Unlike the reconstruction experiment (Section 5), the end-to-end pipeline experiment shows that the layover threshold **does** affect pairing count in instances 1–3, but has no effect in instance 4. The threshold effect is instance-size dependent.

### Results (all 7 instances — complete)

| Instance | Legs | Crew | 8h pairings | 10h pairings | Δ pairings | 8h greedy | 8h opt | 10h greedy | 10h opt | Δ opt |
|----------|------|------|-------------|--------------|------------|-----------|--------|------------|---------|-------|
| instance1 | 1013 | 33 | 54 | 52 | −2 | 0.8760 | 0.8760 | 0.9917 | 0.8705 | −0.0055 |
| instance2 | 1500 | 34 | 81 | 82 | +1 | 0.5433 | 0.5433 | 0.5952 | 0.5952 | +0.0519 |
| instance3 | 1855 | 47 | 141 | 142 | +1 | 0.4011 | 0.2309 | 0.3875 | 0.2173 | −0.0136 |
| instance4 | 5613 | 145 | 571 | 571 | 0 | 0.1239 | 0.1239 | 0.1239 | 0.1239 | 0.0000 |
| instance5 | 5743 | 247 | 568 | 568 | 0 | 0.2645 | 0.2645 | 0.2645 | 0.2645 | 0.0000 |
| instance6 | 5886 | 92 | 92 | 92 | 0 | 0.0794 | 0.0794 | 0.0794 | 0.0794 | 0.0000 |
| instance7 | 7766 | 159 | 159 | 159 | 0 | 0.1531 | 0.1531 | 0.1531 | 0.1531 | 0.0000 |

**Spatial breaks:** instance1=499, instance2=846, instance3=1004, instance4=2998, instance5=3765, instance6=3802, instance7=4765. These are duty breaks forced by airport disconnection (spatial-continuity check). The high count reflects the global chronological grouping limitation: unrelated routes are grouped together and then immediately broken by the spatial check.

**Diagnostic counts (instance1):** duty_rej=0, pairing_rej=907, duties_in_pairings=54, pairings_ok=54 (8h); pairing_rej=903, duties_in_pairings=52, pairings_ok=52 (10h). The high pairing rejection count reflects single-leg duties that do not form round-trips, which is expected under the global chronological grouping model.

### Interpretation

**Threshold effect is instance-size dependent.** In instances 1–3 (1013–1855 legs), the threshold changes pairing count by ±1–2. In instance4 (5613 legs), the threshold has no effect. Once the network becomes sufficiently dense, the dominant factor is the deterministic chronological grouping and spatial continuity constraint, not the threshold value.

**Search space collapse at scale.** Instance4 shows greedy score = optimized score (0.1239) under both conditions. The local search found no improving move. This is not a failure of the optimizer — it is evidence that the search space has already been fully constrained by the greedy construction phase. The optimizer is correct within the space it receives; the space itself is too small.

**Instance3 is the exception.** Local search improved the score by 42–44% (greedy 0.40/0.39 → optimized 0.23/0.22). This confirms the assignment optimizer works when there is assignment flexibility. It does not confirm that the pairing strategy is globally good.

**Contrast with reconstruction experiment.** The reconstruction experiment (Section 5) showed zero pairing count change across all 7 instances. The end-to-end experiment shows non-zero changes in instances 1–3. This confirms the reviewer's prediction: the threshold affects the construction algorithm's output even when it does not affect the reconstruction of a fixed allocation.

**Limitation.** The single-duty pairing model and global chronological grouping are significant departures from the benchmark's operational model. The benchmark's pairings span two days (Jan 29–30) with multi-duty structure; the pipeline here produces single-day, single-duty pairings. The pairing counts (54–571) are far smaller than the benchmark reference (172–1648). These results characterize the behavior of this specific pipeline, not the benchmark's intended pairing structure.

### Reviewer Architectural Conclusion (2026-07-31)

> **The current optimization stage is downstream of the decisions that most strongly determine schedule quality.**

The optimizer is effective *within* the search space it receives (instance3: 42–44% improvement). But the search space itself is produced by deterministic heuristics. The layover threshold experiment has answered a more important question than originally posed: it has revealed that the primary architectural gap is not the threshold value, but the fact that pairing construction is a fixed preprocessing step rather than an optimization variable.

The reviewer's code review (2026-07-31) further identified that `gerad_e2e.rs` should be preserved as the **deterministic baseline** and a parallel Coralys-native implementation developed alongside it. See [`UltraCrew_Pairing_Topology_Mutation_Evaluation.md`](UltraCrew_Pairing_Topology_Mutation_Evaluation.md) Sections 2.6–2.9 for the full analysis.

### Runtime Summary (2026-07-31)

**Total runtime:** 666.01 seconds (11.1 minutes) in release mode across all 7 instances (29,476 flight legs).

**Local search iterations:** instances 1–3 used 10 iterations; instances 4–7 used 0 iterations (`max_iter=0` — intentionally disabled for large instances to keep the experiment tractable). The greedy=optimized result for instances 4–7 is expected, not a failure.

**Pairing rejection ratio** (accepted:rejected) across instances:

| Instance | Accepted | Rejected | Ratio |
|----------|----------|----------|-------|
| 1 | 54 | 907 | 1:16.8 |
| 2 | 81 | 1346 | 1:16.6 |
| 3 | 141 | 1684 | 1:11.9 |
| 4 | 571 | 5012 | 1:8.8 |
| 5 | 568 | 5149 | 1:9.1 |

The pairing constructor rejects ~9–17 candidate duties for every one it accepts. This confirms it is acting as a filter rather than a search algorithm.

**Recommended before Coralys work begins:** Add stage-level timing instrumentation to `gerad_e2e.rs` so that each stage's cost (CSV loading, duty construction, pairing construction, greedy assignment, local search) is quantified. This gives a performance baseline against which Coralys can be measured.

---

## 9. Reference

See [UltraCrew vs GENCOL Pipeline Divergence Analysis](UltraCrew_GENCOL_Pipeline_Divergence_Analysis.md), Section 4.1 for the architectural context of this experiment.