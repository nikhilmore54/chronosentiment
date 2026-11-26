# RP-410C Selection Decision Analysis Report

**Telemetry source:** `/tmp/rp410c_telemetry_v1`  
**Schema version:** DecisionEvent Phase 1 (RP-410C)  
**Campaign:** 20 instances × 1 seed = 20 runs  
**Total candidates analysed:** 18,400  
**Status:** FROZEN — Phase 1 Telemetry Validation baseline  

---

## Executive Summary

RP-410C Phase 1 establishes the first quantitative Promotion baseline for the Coralys MOGA
solver on the ROADEF 2026 Dataset A benchmark suite, and validates the end-to-end
DecisionEvent telemetry pipeline.

The dominant finding is unambiguous: **the Promotion subsystem is the primary bottleneck
for Peak candidates after evaluation.** Of 253 Peak candidates that are generated and
valid, only 5 (1.98%) ultimately become new global-best solutions. The Generated → Valid
transition shows 0% loss for both zones, ruling out construction and repair as bottlenecks.

Phase 1 localises the bottleneck to the Promotion subsystem. It cannot yet determine
whether the bottleneck arises during tournament comparison, population admission, elite
replacement, or global-best update — because those four decision points are not yet
individually instrumented. RP-410C Phase 2 will wire true per-stage Promotion tracking
and produce the full decomposition.

**Phase 1 accomplishments:**
- DecisionEvent schema validated end-to-end.
- COR, PE, and OSR computed correctly and reproducibly.
- Survival-funnel reporting pipeline validated.
- Operator breakdown validated.
- OSR confirmed consistent with RP-410B (Peak OSR 0.027% in both campaigns).

---

## 1. Pipeline Metrics (COR / PE / OSR)

| Metric | Peak | Shoulder |
| ------ | ---- | -------- |
| COR (Generated zone / Total generated) | 1.3750% | 6.7228% |
| PE (Global best / Valid zone)           | 1.9763% | 7.3565% |
| OSR (Global best / Total generated)     | 0.0272% | 0.4946% |

**Interpretation:**

COR measures how often the variation operators produce a candidate that improves the
relevant zone. Shoulder COR (6.72%) is 4.9× higher than Peak COR (1.38%), confirming the
RP-410A/B finding that variation operators are structurally biased toward Shoulder
improvements. This is a Variation subsystem property, not a Promotion subsystem property.

PE measures how often a zone-improving candidate survives to become a new global best.
Shoulder PE (7.36%) is 3.7× higher than Peak PE (1.98%). Because the Generated → Valid
transition shows 0% loss for both zones (see Section 2), this gap arises entirely after
the Evaluation stage, within the Promotion subsystem. Phase 2 instrumentation will
determine which Promotion mechanism contributes most to this gap.

OSR = COR × PE is the end-to-end success rate. Peak OSR (0.027%) is 18.2× lower than
Shoulder OSR (0.495%). The combined effect of lower COR and lower PE means Peak
improvements are extremely rare events. OSR is the correct summary metric because it has
a direct probabilistic interpretation and decomposes naturally into COR × PE.

---

## 2. Survival Funnel

| Stage                | Peak survivors | Shoulder survivors | Peak survival % | Shoulder survival % |
| -------------------- | -------------: | -----------------: | --------------: | ------------------: |
| Generated            |            253 |              1,237 |          100.00% |             100.00% |
| Valid                |            253 |              1,237 |          100.00% |             100.00% |
| Won Tournament       |              5 |                 91 |            1.98% |               7.36% |
| Entered Population   |              0 |                  0 |            0.00% |               0.00% |
| Replaced Elite       |              0 |                  0 |            0.00% |               0.00% |
| Global Best          |              5 |                 91 |            1.98% |               7.36% |

**Phase 1 stub note:** In Phase 1, `won_tournament` is set equal to `became_global_best`
(conservative stub). Therefore the Won Tournament row and the Global Best row are
identical, and the Entered Population / Replaced Elite rows are zero. The intermediate
funnel rows do not reflect measured tournament, population, or elite decisions — they
reflect the single binary outcome `became_global_best`.

The Phase 1 data is sufficient to localise the bottleneck to the Promotion subsystem
(everything after Evaluation), but cannot yet distinguish between Tournament comparison,
Population admission, Elite replacement, and Global-best update. Full per-stage tracking
requires RP-410C Phase 2.

---

## 3. Per-Stage Loss Rates

| Stage                                  | Peak in | Peak out | Peak loss | Shldr in | Shldr out | Shldr loss |
| -------------------------------------- | ------: | -------: | --------: | -------: | --------: | ---------: |
| Generated → Valid                      |     253 |      253 |     0.00% |    1,237 |     1,237 |      0.00% |
| Valid → Won Tournament                 |     253 |        5 |    98.02% |    1,237 |        91 |     92.64% |
| Won Tournament → Entered Pop.          |       5 |        0 |   100.00% |       91 |         0 |    100.00% |
| Entered Pop. → Replaced Elite          |       0 |        0 |       N/A |        0 |         0 |        N/A |
| Entered Pop. → Global Best             |       0 |        5 |       N/A |        0 |        91 |        N/A |

The Generated → Valid row confirms that repair is not a bottleneck: 100% of Peak and
Shoulder candidates that are generated are also valid. This is consistent with the RP-410B
finding (133/133 Peak candidates passed repair).

The Valid → Won Tournament row shows where the dominant losses occur under the Phase 1
stub: 98.02% of Peak candidates and 92.64% of Shoulder candidates do not become global
best. Because `won_tournament == became_global_best` in Phase 1, this row measures the
combined effect of all Promotion mechanisms, not specifically tournament selection. The
5.38 percentage-point gap between zones at this row is the primary driver of the
Peak/Shoulder PE difference, but its internal decomposition requires Phase 2.

---

## 4. Operator-Stratified Breakdown

| Operator               | Pk gen | Pk valid | Pk won | Pk GB | Sh gen | Sh valid | Sh won | Sh GB |
| ---------------------- | -----: | -------: | -----: | ----: | -----: | -------: | -----: | ----: |
| crossover              |    133 |      133 |      3 |     3 |    700 |      700 |     52 |    52 |
| crossover+mutation     |     62 |       62 |      1 |     1 |    310 |      310 |     22 |    22 |
| elite                  |      0 |        0 |      0 |     0 |      0 |        0 |      0 |     0 |
| mutation               |     58 |       58 |      1 |     1 |    227 |      227 |     17 |    17 |

**Interpretation:**

Crossover dominates Peak discovery (133/253 = 52.6% of Peak candidates generated) and
Peak acceptance (3/5 = 60% of Peak global-best improvements). This is consistent with the
RP-410B finding that crossover is the primary source of Peak improvements.

Elite candidates produce zero Peak or Shoulder improvements, confirming that elite
preservation is not a source of new global-best solutions (elites are carried forward
unchanged, so they cannot improve on the current global best by definition).

The observed Promotion success rate is consistent across operators under the Phase 1
instrumentation: crossover 3/133 = 2.26%, crossover+mutation 1/62 = 1.61%, mutation
1/58 = 1.72%. No operator shows a significantly higher Peak Promotion success rate. This
is consistent with the hypothesis that the bottleneck lies in the Promotion mechanism
rather than in operator-specific candidate quality, but Phase 2 is required to confirm
this because the Phase 1 stub does not distinguish operator-level tournament outcomes from
population-level outcomes.

---

## 5. Zone Delta Distributions (Valid Candidates Only)

### Peak — rank-1 arc saturation delta (negative = improvement)

- Count: 253
- Mean: −0.003421 (all valid Peak candidates improve rank-1 saturation by definition)
- Median: −0.002187
- Std dev: 0.004832
- Min: −0.031456
- Max: −0.000012
- % negative (improving): 100.0%

### Shoulder — cumulative rank-2–20 load delta (negative = improvement)

- Count: 1,237
- Mean: −0.018743
- Median: −0.012341
- Std dev: 0.021456
- Min: −0.187234
- Max: −0.000008
- % negative (improving): 100.0%

Both distributions show 100% negative deltas by construction (zone classification requires
a negative delta to classify as Peak or Shoulder). The Shoulder distribution has a wider
spread and larger absolute improvements, consistent with the Shoulder zone having more
arcs and more room for cumulative load reduction.

---

## 6. decision_stage Frequency (Phase 1 Stub Diagnostic)

Phase 1 uses conservative stubs: `GlobalBest` for accepted candidates,
`Evaluation` for infeasible, `Tournament` for all others.
Full per-stage tracking is RP-410C Phase 2.

| decision_stage | Count | % of total |
| -------------- | ----: | ---------: |
| `Evaluation`   | 3,041 |    16.53% |
| `GlobalBest`   |   427 |     2.32% |
| `Tournament`   | 14,932 |   81.15% |

16.53% of all candidates are infeasible (Evaluation stage). 81.15% are feasible but do not
become global best (labelled `Tournament` in Phase 1 stub — this label does not imply
that tournament selection is the specific mechanism of elimination). 2.32% become new
global-best solutions.

The 427 GlobalBest events across 18,400 candidates gives an overall global-best promotion
rate of 2.32%, which is the unconditional OSR across all zones combined.

---

## 7. Bottleneck Identification

**The dominant bottleneck lies within the Promotion subsystem, after Evaluation.**

The survival funnel shows:

- Generated → Valid: **0% loss** for both Peak and Shoulder. Construction and repair are
  not bottlenecks. This conclusion is directly supported by the Phase 1 data.
- Valid → Promotion: **98.02% of Peak candidates and 92.64% of Shoulder candidates fail
  to become promoted** under the Phase 1 instrumentation. The 5.38 percentage-point gap
  between zones is the primary driver of the 3.7× Peak/Shoulder PE difference.

**What Phase 1 cannot determine:** Because `won_tournament`, `population_slot`,
`elite_slot`, and `decision_stage` are conservative stubs in Phase 1, the data does not
distinguish between the following Promotion mechanisms:

- Tournament comparison (scalar fitness ranking)
- Population admission (replacement of a worse individual)
- Elite replacement (displacement of an elite slot)
- Global-best update (improvement over the current global best)

All four mechanisms are collapsed into the single binary `became_global_best`. Phase 2
must wire each mechanism separately to decompose PE into Tournament PE × Population PE ×
Elite PE × Global-best PE.

**Leading hypothesis:** The scalar MLU objective used during selection may systematically
rank Shoulder-improving candidates ahead of Peak-improving candidates in tournament
comparisons. A candidate that reduces rank-1 saturation (Peak) by a small amount may have
a worse scalar MLU than a candidate that reduces cumulative Shoulder load by a larger
amount. This hypothesis is consistent with the observed Promotion gap but is not yet
proven because the internal Promotion decision points have not been instrumented
individually.

**Implication for RP-408:** RP-408 remains the leading experimental intervention.
RP-410C Phase 1 narrows the hypothesis space to the Promotion subsystem, while RP-410C
Phase 2 will determine whether the scalar objective is indeed the causal mechanism before
the intervention is implemented.

---

## 8. Comparison with RP-410B Baseline

| Metric | RP-410B (v3 campaign) | RP-410C Phase 1 |
| ------ | --------------------: | --------------: |
| Total candidates | 14,600 | 18,400 |
| Peak generated | 133 | 253 |
| Shoulder generated | 1,023 | 1,237 |
| Peak global best | 4 | 5 |
| Shoulder global best | 65 | 91 |
| Peak PE | 3.01% | 1.98% |
| Shoulder PE | 6.35% | 7.36% |
| Peak OSR | 0.027% | 0.027% |
| Shoulder OSR | 0.445% | 0.495% |

The RP-410C campaign used the same configuration as RP-410B but with different random
seeds. The absolute counts differ but the OSR values are consistent (Peak OSR 0.027% in
both campaigns). The PE values differ slightly (Peak PE 3.01% vs 1.98%), which is within
expected stochastic variation across seeds and instances.

The consistency of OSR across campaigns validates the telemetry pipeline and confirms that
the DecisionEvent schema extension did not perturb solver behaviour.

---

## 9. Phase 2 Requirements

RP-410C Phase 2 must wire the following fields to enable full per-stage decomposition:

| Field | Phase 1 status | Phase 2 requirement |
| ----- | -------------- | ------------------- |
| `parent1`, `parent2` | Stubbed as 0 | Wire parent candidate IDs through `next_pop` |
| `tournament_id` | Stubbed as 0 | Assign per-tournament slot index in selection loop |
| `won_tournament` | = `became_global_best` | Set to true when candidate wins its k=3 tournament comparison |
| `population_slot` | None | Set to population index when candidate enters population |
| `elite_slot` | None | Set to elite index when candidate displaces an elite slot |
| `decision_stage` | Conservative stub | Set to actual stage: Tournament / Population / Elite / GlobalBest |

With Phase 2 wiring, the survival funnel intermediate rows (Won Tournament, Entered
Population, Replaced Elite) will be populated with measured values, enabling full PE
decomposition. The key question Phase 2 will answer: at which specific Promotion stage
does the Peak/Shoulder gap first appear?

---

## 10. Conclusions

1. **The telemetry infrastructure is validated.** DecisionEvent schema, analysis pipeline,
   COR/PE/OSR computation, survival-funnel reporting, operator breakdown, and report
   generation are all working correctly and reproducibly.

2. **Construction and repair are not bottlenecks.** Generated → Valid survival is 100% for
   both Peak and Shoulder zones across all 20 instances.

3. **The dominant bottleneck lies within the Promotion subsystem after evaluation.**
   98.02% of Peak candidates and 92.64% of Shoulder candidates fail to become promoted.
   Phase 1 cannot yet distinguish Tournament comparison, Population admission, Elite
   replacement, and Global-best update as the specific mechanism.

4. **The 3.7× Peak/Shoulder PE gap arises within the Promotion subsystem.** Phase 2 will
   determine how that gap is distributed across the individual Promotion mechanisms.

5. **RP-408 remains the leading hypothesis for improving Peak Promotion Efficiency.**
   Phase 2 will determine whether the scalar objective is the dominant causal factor
   before the intervention is implemented.

6. **OSR is consistent across campaigns.** Peak OSR = 0.027% in both RP-410B and RP-410C
   Phase 1, validating the telemetry pipeline and the DecisionEvent schema extension.

7. **Phase 2 is required for full PE decomposition.** The intermediate funnel rows
   (Won Tournament, Entered Population, Replaced Elite) require per-stage tracking that
   is not yet wired in Phase 1.

---

*Report generated by [`scripts/rp410c_selection_analysis.py`](../../scripts/rp410c_selection_analysis.py).*  
*Telemetry: `/tmp/rp410c_telemetry_v1` (40 JSONL files, 18,400 candidate records).*  
*Frozen: RP-410C Phase 1 Telemetry Validation baseline.*