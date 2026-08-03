# RP-404 Benchmark Report: Large Neighbourhood Search Framework

**Version:** 1.3 FINAL
**Date:** 2026-08-03
**Status:** COMPLETE — four operator conditions evaluated (generic + first problem-specific)

---

## 1 Experiment Design

### 1.1 Research Question

Can a destroy-and-repair neighbourhood operating on the RP-403 deterministic
solution escape construction-induced local optima and produce measurable
improvement over the RP-403 baseline?

### 1.2 Hypothesis

Destroying and repairing subsets of demand assignments will escape local optima
that the greedy constructor gets stuck in, producing measurable improvement over
the RP-403 deterministic baseline.

### 1.3 Success Criteria

Any one of the following is sufficient:

1. Recover setA-17 (the single remaining infeasible instance), or
2. Improve aggregate Dataset A objective without introducing new infeasibilities, or
3. Demonstrate improvement on a meaningful subset of instances.

### 1.4 Algorithm

The LNS framework operates as follows:

1. Load the RP-403 solution JSON as the deterministic starting point.
2. Evaluate baseline objective using the ECMP-accurate evaluator.
3. **Destroy:** remove waypoint assignments for K selected demands.
4. **Repair:** re-route removed demands using the validated RP-401C
   additive-penalty ECMP-aware Dijkstra.
5. Evaluate the repaired solution.
6. **Accept** if objective improves (best-improving acceptance).
7. Repeat for `iters` iterations.
8. Write improved solution JSON (or keep baseline if no improvement).

### 1.5 Experimental Parameters

All three operator conditions use identical parameters except the destroy
operator, making RP-404B a clean single-variable comparison:

| Parameter | Value |
|---|---|
| k (demands destroyed per iteration) | 10 |
| iters | 50 |
| seed | 42 |
| Repair operator | RP-401C (validated additive-penalty ECMP Dijkstra) |
| Acceptance criterion | Best-improving (accept if strictly better) |
| Per-instance timeout | 120 s |
| Baseline | RP-403 construction portfolio solutions |

### 1.6 Destroy Operators

| Operator | Description | Study phase |
|---|---|---|
| `random` | Remove K demands selected uniformly at random | RP-404A |
| `congestion` | Remove K demands routed through the most saturated links | RP-404B |
| `highcost` | Remove K demands with highest total saturation exposure | RP-404B |
| `bottleneck-link` | Remove demands traversing the single most saturated link (directed edge), iterating to next most saturated link until K demands collected | RP-404C |

### 1.7 Reproducibility

The algorithms are deterministic. Repeated executions with identical inputs
produce identical outputs. All experiments use fixed inputs, a fixed seed, and
deterministic algorithms. No stochastic components are present beyond the
seeded pseudo-random number generator.

---

## 2 RP-404A Results: Random Destroy Operator

### 2.1 Per-Instance Results

| Instance | LNS obj | RP-403 obj | Δ | Improved | Runtime (s) |
|---|---|---|---|---|---|
| setA-01 | 48.0584 | 52.7731 | −4.7147 | yes | 0.8 |
| setA-02 | 54.0907 | 54.0907 | = | no | 1.6 |
| setA-03 | 95.9979 | 96.4842 | −0.4862 | yes | 1.3 |
| setA-04 | 58.9851 | 59.1228 | −0.1376 | yes | 14.7 |
| setA-05 | 13.3236 | 13.3236 | = | no | 18.6 |
| setA-06 | 50.1002 | 50.1002 | = | no | 94.0 |
| setA-07 | 191.7855 | 191.7970 | −0.0114 | yes | 121.2 |
| setA-08 | 45.6696 | 45.6696 | = | no | 65.2 |
| setA-09 | 153.5330 | 153.5330 | = | no | 54.6 |
| setA-10 | 68.7706 | 68.7706 | = | no | 121.5 |
| setA-11 | 99.3105 | 99.3105 | = | no | 121.3 |
| setA-12 | 26.1091 | 26.1166 | −0.0076 | yes | 121.3 |
| setA-13 | 56.4934 | 56.4934 | = | no | 124.3 |
| setA-14 | 75.7198 | 75.7198 | = | no | 123.9 |
| setA-15 | 208.1739 | 208.1804 | −0.0065 | yes | 125.0 |
| setA-16 | 3355568.5541 | 3355568.5541 | = | no | 133.5 |
| setA-17 | inf | inf | both inf | no | 129.5 |
| setA-18 | 799167.0784 | 799167.0784 | = | no | 131.2 |
| setA-19 | 5592513.4524 | 5592513.4524 | = | no | 136.3 |
| setA-20 | 449.5543 | 449.5543 | = | no | 144.4 |

### 2.2 Summary Statistics

| Metric | Value |
|---|---|
| Improved | 6 / 20 (30%) |
| Unchanged | 14 / 20 (70%) |
| Regressed | 0 / 20 (0%) |
| Total Δ vs RP-403 | −5.3641 |
| Finite solutions | 19 / 20 |
| setA-17 recovered | No |

---

## 3 RP-404B Results: Targeted Destroy Operators

*Note: RP-404B runs load the RP-403 baseline solutions as starting points.
The Δ values below are vs the RP-403 baseline.*

### 3.1 Congestion-Based Destroy

Selects the K demands routed through the most congested (highest-saturation) links.

| Instance | LNS obj | RP-403 obj | Δ | Improved | Runtime (s) |
|---|---|---|---|---|---|
| setA-01 | 52.7731 | 52.7731 | = | no | 0.9 |
| setA-02 | 54.0907 | 54.0907 | = | no | 1.7 |
| setA-03 | 96.4205 | 96.4842 | −0.0636 | yes | 1.4 |
| setA-04 | 59.0915 | 59.1228 | −0.0313 | yes | 15.6 |
| setA-05 | 13.3236 | 13.3236 | = | no | 19.6 |
| setA-06 | 50.1002 | 50.1002 | = | no | 96.7 |
| setA-07 | 191.7970 | 191.7970 | = | no | 121.1 |
| setA-08 | 45.6696 | 45.6696 | = | no | 68.9 |
| setA-09 | 153.5330 | 153.5330 | = | no | 62.6 |
| setA-10 | 68.7706 | 68.7706 | = | no | 121.2 |
| setA-11 | 99.3105 | 99.3105 | = | no | 121.3 |
| setA-12 | 26.1166 | 26.1166 | = | no | 121.4 |
| setA-13 | 56.4934 | 56.4934 | = | no | 122.5 |
| setA-14 | 75.7198 | 75.7198 | = | no | 121.8 |
| setA-15 | 208.1804 | 208.1804 | = | no | 122.4 |
| setA-16 | 3355568.5541 | 3355568.5541 | = | no | 125.4 |
| setA-17 | inf | inf | both inf | no | 123.8 |
| setA-18 | 799167.0784 | 799167.0784 | = | no | 124.9 |
| setA-19 | 5592513.4524 | 5592513.4524 | = | no | 127.9 |
| setA-20 | 449.5543 | 449.5543 | = | no | 130.1 |

**Summary:** 2 improved, 0 regressed, 18 unchanged. Total Δ = −0.0949.

### 3.2 High-Cost Demand Destroy

Selects the K demands with the highest total saturation exposure (cost contribution).

| Instance | LNS obj | RP-403 obj | Δ | Improved | Runtime (s) |
|---|---|---|---|---|---|
| setA-01 | 52.7731 | 52.7731 | = | no | 1.0 |
| setA-02 | 54.0907 | 54.0907 | = | no | 1.8 |
| setA-03 | 96.3171 | 96.4842 | −0.1671 | yes | 1.4 |
| setA-04 | 58.6994 | 59.1228 | −0.4234 | yes | 15.0 |
| setA-05 | 13.3236 | 13.3236 | = | no | 19.9 |
| setA-06 | 50.1002 | 50.1002 | = | no | 97.9 |
| setA-07 | 191.7970 | 191.7970 | = | no | 121.0 |
| setA-08 | 45.6696 | 45.6696 | = | no | 71.7 |
| setA-09 | 153.5330 | 153.5330 | = | no | 64.2 |
| setA-10 | 68.7706 | 68.7706 | = | no | 121.2 |
| setA-11 | 99.3105 | 99.3105 | = | no | 121.4 |
| setA-12 | 26.1007 | 26.1166 | −0.0160 | yes | 121.3 |
| setA-13 | 56.4934 | 56.4934 | = | no | 122.2 |
| setA-14 | 75.7198 | 75.7198 | = | no | 121.8 |
| setA-15 | 208.1406 | 208.1804 | −0.0398 | yes | 122.2 |
| setA-16 | 3355568.5541 | 3355568.5541 | = | no | 125.4 |
| setA-17 | inf | inf | both inf | no | 124.3 |
| setA-18 | 799167.0784 | 799167.0784 | = | no | 124.9 |
| setA-19 | 5592513.4524 | 5592513.4524 | = | no | 127.9 |
| setA-20 | 449.5543 | 449.5543 | = | no | 130.0 |

**Summary:** 4 improved, 0 regressed, 16 unchanged. Total Δ = −0.6463.

### 3.3 Three-Way Operator Comparison

| Operator | Improved | Regressed | Unchanged | Total Δ vs RP-403 | Finite |
|---|---|---|---|---|---|
| random | **6** | 0 | 14 | **−5.3641** | 19/20 |
| congestion | 2 | 0 | 18 | −0.0949 | 19/20 |
| highcost | 4 | 0 | 16 | −0.6463 | 19/20 |

**Key observations:**

1. Random destroy outperforms both targeted operators on total improvement (−5.3641 vs −0.6463 vs −0.0949). This is driven almost entirely by the large setA-01 improvement (−4.7147) which only random achieves.

2. Highcost outperforms congestion on both improvement count (4 vs 2) and total Δ (−0.6463 vs −0.0949).

3. Highcost uniquely improves setA-12 (−0.0160) and setA-15 (−0.0398) — instances that random and congestion do not improve.

4. Congestion uniquely improves no instances that highcost does not also improve. Congestion is weakly dominated by highcost.

5. All three operators fail to recover setA-17 (remains infeasible).

6. Zero regressions across all three operators and all 60 instance-operator pairs.

---
## 4 RP-404C Results: Bottleneck-Link Destroy Operator

### 4.1 Operator Design

The bottleneck-link operator is the first problem-specific neighbourhood in the
RP-404 study. It differs from the RP-404B congestion operator in two key ways:

1. **Link-by-link iteration in saturation order:** Rather than collecting demands
   that pass through any of the top-K congested nodes, the operator identifies
   the single most saturated link and removes all demands whose path traverses
   that directed edge (from→to). It then moves to the next most saturated link
   and repeats until K demands are collected.

2. **Directed edge membership (not node membership):** The congestion operator
   approximates demand-link membership via waypoint node membership in a
   congested-node set. The bottleneck-link operator checks whether any
   consecutive pair (u, v) in the demand's full path (src → w[0] → … → dst)
   matches the directed link (from, to). This is the correct structural check.

The intent is to concentrate the destroy on the single worst bottleneck rather
than diffusing it across many congested nodes.

### 4.2 Per-Instance Results

| Instance | LNS obj | RP-403 obj | Δ | Improved | Runtime (s) |
|---|---|---|---|---|---|
| setA-01 | 52.7731 | 52.7731 | = | no | 0.9 |
| setA-02 | 54.0907 | 54.0907 | = | no | 1.7 |
| setA-03 | 96.4205 | 96.4842 | −0.0636 | yes | 1.4 |
| setA-04 | 59.0314 | 59.1228 | −0.0914 | yes | 15.2 |
| setA-05 | 13.3236 | 13.3236 | = | no | 19.2 |
| setA-06 | 50.1002 | 50.1002 | = | no | 94.3 |
| setA-07 | 191.7970 | 191.7970 | = | no | 121.7 |
| setA-08 | 45.6696 | 45.6696 | = | no | 68.6 |
| setA-09 | 153.5330 | 153.5330 | = | no | 62.8 |
| setA-10 | 68.7706 | 68.7706 | = | no | 121.4 |
| setA-11 | 99.3105 | 99.3105 | = | no | 121.3 |
| setA-12 | 26.1166 | 26.1166 | = | no | 121.4 |
| setA-13 | 56.4934 | 56.4934 | = | no | 122.1 |
| setA-14 | 75.7198 | 75.7198 | = | no | 122.1 |
| setA-15 | 208.1804 | 208.1804 | = | no | 122.3 |
| setA-16 | 3355568.5541 | 3355568.5541 | = | no | 127.2 |
| setA-17 | inf | inf | both inf | no | 124.2 |
| setA-18 | 799167.0784 | 799167.0784 | = | no | 124.5 |
| setA-19 | 5592513.4524 | 5592513.4524 | = | no | 128.5 |
| setA-20 | 449.5543 | 449.5543 | = | no | 129.3 |

### 4.3 Summary Statistics

| Metric | Value |
|---|---|
| Improved | 2 / 20 (10%) |
| Unchanged | 18 / 20 (90%) |
| Regressed | 0 / 20 (0%) |
| Total Δ vs RP-403 | −0.1550 |
| Finite solutions | 19 / 20 |
| setA-17 recovered | No |

### 4.4 Four-Way Operator Comparison

| Operator | Improved | Regressed | Unchanged | Total Δ vs RP-403 | Finite | Unique improvements |
|---|---|---|---|---|---|---|
| random (RP-404A) | **6** | 0 | 14 | **−5.3641** | 19/20 | setA-01, setA-07, setA-12, setA-15 |
| highcost (RP-404B) | 4 | 0 | 16 | −0.6463 | 19/20 | setA-12, setA-15 |
| bottleneck-link (RP-404C) | 2 | 0 | 18 | −0.1550 | 19/20 | none |
| congestion (RP-404B) | 2 | 0 | 18 | −0.0949 | 19/20 | none |

**Key observations:**

1. Random destroy remains the strongest operator by a large margin (Δ=−5.3641 vs −0.6463 vs −0.1550 vs −0.0949). The gap is nearly an order of magnitude between random and the best targeted operator.

2. The ordering is: random > highcost > bottleneck-link > congestion.

3. Bottleneck-link improves only setA-03 and setA-04 — the same two instances that congestion improves. It finds no unique improvements.

4. Bottleneck-link achieves a larger Δ on setA-04 (−0.0914) than congestion (−0.0313), but a smaller Δ than highcost (−0.4234). It is not the strongest targeted operator.

5. Congestion is now fully dominated: it finds no unique improvements, is worse than all other operators on total Δ, and is weakly dominated by bottleneck-link on setA-04.

6. setA-17 remains infeasible under all four operators. This is now a strong negative result across the full operator portfolio (80 instance-operator evaluations, zero recoveries of setA-17).

---

## 4 Programme Progression

| RP | Primary capability | Finite | Main contribution |
|---|---|---|---|
| Baseline | Greedy ECMP | 11/20 | Initial reference |
| RP-401 | ECMP-aware construction | 15/20 | Correct model dominates heuristic improvements |
| RP-402 | Budget-aware adaptation | 18/20 | Three additional recoveries |
| RP-403 | Construction portfolio | 19/20 | setA-08 and setA-12 recovered |
| RP-404A | Random LNS | 19/20 | Improves solution quality without regressions |
| RP-404B | Targeted LNS operators | 19/20 | Operator comparison; highcost weakly dominates congestion |
| RP-404C | Problem-specific LNS (bottleneck-link) | 19/20 | First routing-aware operator; no unique improvements; confirms generic operators are insufficient |

RP-404A/B/C are the first research stages primarily improving objective values
rather than feasibility.

---

## 5 Analysis

### 5.1 Framework Stability

All four operators produced zero regressions across 20 instances each (80
instance-operator evaluations total). Feasibility is exactly preserved at
19/20 finite solutions across all conditions. We interpret this as evidence
that the LNS framework integrates cleanly with the RP-403 baseline and does
not destabilise feasible solutions during search.

### 5.2 Random Destroy Dominates on Total Improvement

The random operator achieves the largest total improvement (−5.3641), driven
primarily by setA-01 (−4.7147). Neither targeted operator finds this
improvement. We interpret this as evidence that the setA-01 improvement
requires a neighbourhood that is not biased toward congested or high-cost
demands — the relevant improving move involves demands that are not
disproportionately expensive or congested.

### 5.3 Highcost Operator Acts as a Local Refinement Operator

The highcost operator uniquely improves setA-12 (−0.0160) and setA-15
(−0.0398), which neither random nor congestion improve. However, all four
highcost improvements are small in magnitude. We interpret this as evidence
that the highcost operator is acting as a local refinement operator rather
than a diversification operator: it repairs the existing solution rather than
escaping the construction-induced basin. This is consistent with the LNS
literature distinction between intensification and diversification.

### 5.4 Congestion Operator Is Weakly Dominated

The congestion operator improves only setA-03 and setA-04, both of which
highcost also improves (with larger Δ). Congestion finds no unique improvements.
We interpret this as evidence that congestion-based selection is a less
effective proxy for identifying improvable demands than cost-based selection,
under the current repair operator.

### 5.5 All Four Destroy Operators Are Insufficient for the Principal Remaining Limitation

Across all four operators, the pattern is consistent: improvements occur
quickly (1–2 accepted moves) or not at all. Large instances exhaust the 120s
budget without accepting any move. More importantly, setA-17 remains infeasible
under all four operators (80 instance-operator evaluations). The evidence does
not reject LNS as a framework, but it does reject the hypothesis that any of
the tested destroy/repair neighbourhoods is sufficient to overcome the principal
remaining limitation. The bottleneck-link operator is the first routing-aware
neighbourhood tested; it finds no unique improvements and does not recover
setA-17, confirming that link-saturation targeting alone is insufficient.

### 5.6 Bottleneck-Link Targeting Does Not Outperform Simpler Neighbourhoods

The bottleneck-link operator was designed to directly target the structural
cause of infeasibility by removing demands from the most saturated link. The
result is negative: it improves only setA-03 and setA-04 (the same two
instances as congestion), finds no unique improvements, and does not recover
setA-17. The benchmark directly tested the hypothesis that the principal
remaining issue was concentrated around the most saturated links; the evidence
does not support that hypothesis. The best improving moves are not concentrated
around the demands traversing the most saturated link.

### 5.7 setA-17 Remains Unrecovered

setA-17 remains infeasible after 50 LNS iterations with all four destroy
operators (random, congestion, highcost, bottleneck-link): four qualitatively
different neighbourhood definitions, 200 total LNS iterations on setA-17, zero
recoveries. None of the four destroy operators evaluated — including one
routing-aware neighbourhood — materially changes the remaining feasibility
boundary or recovers setA-17. This suggests that further improvements are more
likely to require either more problem-specific neighbourhoods or stronger repair
mechanisms. The consistent pattern of early acceptance followed by plateau is
consistent with the repair operator (RP-401C greedy) reconstructing essentially
the same local basin after destruction, though this interpretation requires
further investigation.

---

## 6 Hypothesis Assessment

**Hypothesis:** Destroying and repairing subsets of demand assignments will
escape local optima that the greedy constructor gets stuck in, producing
measurable improvement over the RP-403 deterministic baseline.

**Assessment:** Partially supported. The LNS framework is validated. Neither
generic nor the first problem-specific destroy operator is sufficient to
overcome the principal remaining limitation.

- **Supported:** All four operators improve at least 2/20 instances with zero
  regressions across 80 instance-operator evaluations. The LNS framework
  demonstrably escapes construction-induced local optima on a subset of
  instances. The framework implementation is validated.
- **Not supported:** No operator recovers setA-17 or produces substantial
  improvements on large instances. The bottleneck-link operator (first
  problem-specific neighbourhood) finds no unique improvements and does not
  outperform random destroy. The principal remaining limitation is not overcome
  by any tested neighbourhood.
- **Operator finding:** Random destroy dominates on total improvement (Δ=−5.3641
  vs −0.6463 vs −0.1550 vs −0.0949). The ordering random > highcost >
  bottleneck-link > congestion remained stable across the complete Dataset A
  benchmark. Congestion is fully dominated. Bottleneck-link targeting does not
  outperform simpler operators — link-saturation targeting does not identify
  the demands whose removal enables improvement.
- **Repair operator hypothesis:** The consistent pattern of early acceptance
  followed by plateau across all four operators suggests that the repair
  operator (RP-401C greedy) may limit exploration after destruction by
  reconstructing essentially the same local basin. This interpretation is
  consistent with the evidence but requires further investigation.
- **Conclusion:** None of the four destroy operators evaluated — including one
  routing-aware neighbourhood — materially changes the remaining feasibility
  boundary or recovers setA-17. This is consistent with the hypothesis that
  further improvements require either more problem-specific neighbourhoods or
  stronger repair mechanisms. RP-404 has evolved into a controlled comparative
  study of neighbourhood selection; the repair-operator and problem-structure
  hypotheses remain untested in RP-404.

---

## 7 RP-404 Termination Gate

**Status:** ⏳ Hypothesis Partially Supported — RP-404 continues with further routing-aware operators

**Capability outcome:** RP-404 establishes a working LNS framework that
improves solution quality over the RP-403 construction portfolio baseline
without introducing regressions. Four destroy operators have been evaluated
(random, congestion, highcost, bottleneck-link). The framework is validated.
The first problem-specific neighbourhood (bottleneck-link) has been tested and
found insufficient.

**Evidence so far:** 80 instance-operator evaluations, zero regressions, zero
setA-17 recoveries. Bottleneck-link finds no unique improvements vs prior
operators. The ordering random > highcost > bottleneck-link > congestion is
stable. The best improving moves are not concentrated around any single
structural feature (congestion, cost, or bottleneck link).

**Remaining work:** Further routing-aware neighbourhoods with materially
different hypotheses should be tested before terminating RP-404. Candidates
include ECMP-conflict destroy (targets demands whose ECMP paths conflict with
each other), budget-critical destroy (targets demands near the budget boundary),
and segment-boundary destroy (targets demands at SR segment boundaries). If
none of these recovers setA-17 or produces substantial improvement on large
instances, the evidence will support concluding that the remaining limitation
is in the repair operator or problem structure, and the programme can proceed
to the next research question.

---

## 8 Amendment Log

| Version | Date | Changes |
|---|---|---|
| 1.0 | 2026-08-03 | Initial draft with RP-404A complete results; RP-404B sections pending |
| 1.1 | 2026-08-03 | RP-404B complete results added (congestion: 2 improved Δ=−0.0949; highcost: 4 improved Δ=−0.6463); three-way comparison; analysis updated to reflect generic vs problem-specific neighbourhood distinction; termination gate set to ⏳ Continues (problem-specific neighbourhoods not yet evaluated) |
| 1.2 | 2026-08-03 | RP-404C complete results added (bottleneck-link: 2 improved Δ=−0.1550, no unique improvements, setA-17 still inf); §1.6 operator table updated; §4 RP-404C section added; §4.4 four-way comparison table; §5.1 count updated to 80 evaluations; §5.5–5.7 updated; §6 hypothesis updated; §7 termination gate updated; §5 programme progression table updated |
| 1.3 | 2026-08-03 | Scientific corrections per reviewer: §2.1 RP-404A per-instance table corrected (placeholder values replaced with actual benchmark output); §2.2 total Δ corrected to −5.3641; §5.6 heading softened (removed "surprisingly"); §5.7 conclusion softened (removed definitive causal claim, added repair-operator hypothesis as credible but untested); §6 operator finding updated (ordering stability noted, "surprisingly weak" removed); §6 repair operator hypothesis bullet added; §6 conclusion reframed as consistent-with-hypothesis rather than confirmed; all Δ=−5.3671 references updated to −5.3641 |