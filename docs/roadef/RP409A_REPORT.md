# RP-409A: Per-Operator Accepted Contribution Analysis and Zone Transition Matrix

**Status:** FROZEN — post-review revision (date 2026-08-06)
**Date:** 2026-08-06
**Data source:** RP-408B scalar arm (20 instances × setA, seed 42)
**Instances with accepted moves:** 14 / 20 (6 instances produced no global-best improvements)

---

## 1. Research Question

Among accepted global-best improvements in the scalar comparator baseline, which operators
contribute to Peak/Shoulder/Transition zone improvements? What is the zone-to-zone
transition structure of accepted improvements? Which operators are the limiting contributors
for each zone?

**Scope note.** RP-409A measures *accepted contributions* — the subset of offspring that
survive the full evolutionary pipeline (generation → tournament → population → elite →
global-best replacement) and produce a new global-best solution. It does not measure
operator generation behaviour. An operator may generate many Peak-improving offspring that
lose tournaments, fail elite replacement, or improve Peak while worsening the scalar
objective. RP-409A cannot distinguish these possibilities. True operator-level generation
capability remains the subject of future instrumentation.

---

## 2. Metric Definitions

RP-409A introduces the following metrics, distinct from the COR/PE/OSR metrics used in
earlier reports (which are defined over generated candidates, not accepted improvements):

**ACR (Accepted Contribution Rate):**
`zone_accepted_moves_by_operator / total_accepted_moves_by_operator`
The fraction of this operator's accepted global-best improvements that fall in this zone.

**APS (Accepted Promotion Share):**
`zone_accepted_moves_by_operator / total_accepted_moves_all_operators`
This operator's share of all accepted global-best improvements in this zone.

**AOR (Accepted Objective Rate):**
`zone_accepted_moves_by_operator / n_generations`
Accepted improvements in this zone per generation, for this operator.

**Mean ΔObj:**
`mean(prev_obj − new_obj)` per accepted move (positive = improvement).

These metrics describe the *output* of the full evolutionary pipeline attributed to each
operator. They do not describe what each operator generates.

---

## 3. Accepted Contribution Table

Mean values across 14 instances with accepted improvements.

| Operator  | Moves | Mean ΔObj | Peak ACR | Peak APS | Peak AOR | Shldr ACR | Shldr APS | Shldr AOR | Trans ACR | Trans APS | Trans AOR |
|-----------|------:|----------:|---------:|---------:|---------:|----------:|----------:|----------:|----------:|----------:|----------:|
| crossover |   5.1 |    2.3708 |   0.0143 |   0.0071 |   0.0060 |    0.1316 |    0.0719 |    0.0462 |    0.1013 |    0.0551 |    0.0285 |
| mutation  |   0.7 |    0.0926 |   0.0000 |   0.0000 |   0.0000 |    0.0381 |    0.0056 |    0.0016 |    0.1143 |    0.0187 |    0.0102 |

---

## 4. Zone Breakdown per Operator (ACR)

| Operator  | Peak   | Shoulder | Transition | Tail   | Mixed  | Neutral |
|-----------|-------:|---------:|-----------:|-------:|-------:|--------:|
| crossover | 0.0143 |   0.1316 |     0.1013 | 0.3548 | 0.3265 |  0.0000 |
| mutation  | 0.0000 |   0.0381 |     0.1143 | 0.0714 | 0.0619 |  0.0000 |

---

## 5. Zone Transition Matrix

Zone-to-zone transitions between consecutive accepted global-best improvements, aggregated
across all 14 instances. Rows = from-zone, columns = to-zone. Values = count (% of row).

| From \ To  | mixed    | neutral | peak    | shoulder | tail     | transition |
|------------|----------|---------|---------|----------|----------|------------|
| mixed      | 9 (28%)  | 0 (0%)  | 1 (3%)  | 4 (12%)  | 12 (38%) | 6 (19%)    |
| neutral    | 1 (100%) | 0 (0%)  | 0 (0%)  | 0 (0%)   | 0 (0%)   | 0 (0%)     |
| peak       | 1 (50%)  | 0 (0%)  | 0 (0%)  | 0 (0%)   | 0 (0%)   | 1 (50%)    |
| shoulder   | 6 (24%)  | 0 (0%)  | 1 (4%)  | 6 (24%)  | 5 (20%)  | 7 (28%)    |
| transition | 5 (17%)  | 0 (0%)  | 0 (0%)  | 13 (45%) | 7 (24%)  | 4 (14%)    |
| tail       | 10 (32%) | 0 (0%)  | 0 (0%)  | 3 (10%)  | 8 (26%)  | 10 (32%)   |

---

## 6. Findings

### F1 — Crossover is the dominant contributor among accepted improvements

Among accepted global-best improvements, crossover is the dominant contributing operator:
~88% of accepted moves by count (mean 5.1 vs. mutation's 0.7), and 25× larger mean ΔObj
per accepted move (2.37 vs. 0.09 units). This reflects the output of the full pipeline
attributed to each operator; it does not establish that crossover generates more
improvements overall.

### F2 — No accepted Peak improvements from mutation were observed

Mutation's Peak ACR = 0.000 across all 14 instances. No accepted global-best Peak
improvements attributed to mutation were observed during this campaign. This is a
significant attribution finding, but the underlying cause is not established by RP-409A.
Possible explanations include: (a) mutation rarely generates Peak-improving offspring;
(b) mutation generates Peak-improving offspring that lose tournaments or fail elite
replacement; (c) mutation generates Peak-improving offspring that worsen the scalar
objective and are therefore not accepted as global-best. RP-409B should investigate which
of these mechanisms is operative.

Crossover's Peak ACR = 0.014 — also very low, but non-zero. Peak accepted improvements
are rare for both operators, consistent with the RP-408B baseline Peak PE of 0.007.

### F3 — Crossover contributes substantially more accepted Shoulder improvements

Crossover Shoulder ACR = 0.132 vs. mutation's 0.038 — a 3.5× difference in accepted
Shoulder contribution rate. Mutation contributes fewer accepted Shoulder improvements per
accepted move. Whether this reflects generation capability or pipeline attrition is not
determined by RP-409A.

### F4 — Transition is the most balanced zone among accepted contributions

Transition ACR: crossover = 0.101, mutation = 0.114. This is the only zone where mutation
is competitive with crossover (ratio 1.1×). Transition accepted improvements are the most
evenly distributed between operators.

### F5 — Crossover's accepted improvements are concentrated in Tail and Mixed

Among crossover's accepted improvements, 35% are Tail and 33% are Mixed — together 68%.
These are broad multi-zone improvements (Mixed) or improvements to the least-loaded arcs
(Tail). These are accepted by the scalar comparator because they reduce the aggregate
objective, but they do not specifically target Peak or Shoulder. This characterises the
*accepted output* of crossover; it does not imply that crossover generates primarily
Tail/Mixed offspring.

### F6 — Transition→Shoulder feedback loop in accepted improvement sequences

> **Figure 3. Preferred zone transition pathways among accepted global-best improvements.**
>
> ```
> Transition → Shoulder  (45% of transitions from Transition)
>     ↑              ↓
>     └──────────────┘  (28% of transitions from Shoulder)
> ```

The transition matrix reveals a productive search corridor: after an accepted Transition
improvement, the most likely next accepted improvement is Shoulder (45%). After an accepted
Shoulder improvement, the next is most likely Transition (28%) or Shoulder again (24%).
This Transition↔Shoulder feedback loop is the dominant sequential pattern in the accepted
improvement sequence.

Peak improvements are rare entry points (only 2 Peak accepted moves observed in total) and
do not form a stable loop. This suggests that the search trajectory through solution space
— as measured by accepted global-best improvements — is primarily a Transition↔Shoulder
corridor, with occasional Tail/Mixed improvements and very rare Peak improvements.

This zone transition structure may represent preferred evolutionary pathways through
solution space. It is a finding about the *sequence of accepted improvements*, not about
operator behaviour directly.

### F7 — Six instances produced zero accepted improvements

setA-02, setA-07, setA-13, setA-16, setA-19, setA-20 produced no accepted global-best
improvements in the scalar arm. These are the same instances that were invalid at
termination (RP-408B Level 1). The constructor never produced a feasible individual
(IFR = 0), and the evolutionary search never found a valid solution. These instances are
excluded from the attribution analysis.

---

## 7. Limiting Contributor Summary

| Zone       | Dominant  | ACR    | Limiting | ACR    | Ratio  |
|------------|-----------|-------:|----------|-------:|-------:|
| Peak       | crossover | 0.0143 | mutation | 0.0000 | ∞      |
| Shoulder   | crossover | 0.1316 | mutation | 0.0381 | 3.5×   |
| Transition | mutation  | 0.1143 | crossover| 0.1013 | 1.1×   |

Mutation is the limiting contributor for Peak (zero accepted Peak improvements) and
Shoulder (3.5× fewer accepted Shoulder improvements per accepted move). Crossover is
marginally limiting for Transition (ratio 1.1×, not practically significant).

These are *accepted contribution* gaps, not generation capability gaps. The distinction
matters for RP-409B design.

---

## 8. Implications for RP-409B

RP-409A establishes the accepted contribution profile of each operator. It does not
establish why mutation contributes fewer Peak and Shoulder improvements. RP-409B should
address both the attribution gap and its cause.

**Research question for RP-409B:** Does mutation fail to contribute Peak improvements
because it fails to *generate* them, or because generated Peak improvements are rejected
by the pipeline (tournament, elite, or objective)?

**Operator redesign priorities (from accepted contribution gaps):**

Priority 1 — Mutation: investigate Peak-zone accepted contribution gap. Zero accepted Peak
improvements from mutation across 14 instances is a strong signal. Whether the fix is in
generation (add Peak-targeting perturbation) or in promotion (reduce pipeline attrition
for Peak-improving offspring) depends on which mechanism is operative.

Priority 2 — Mutation: investigate Shoulder-zone accepted contribution gap. Mutation's
Shoulder ACR (0.038) is 3.5× lower than crossover's (0.132). Same diagnostic question
applies.

Priority 3 — Crossover: characterise Tail/Mixed concentration. 68% of crossover's accepted
improvements are Tail/Mixed. Whether this is a generation property or a selection property
is not established.

**Primary outcome for RP-409B (unchanged from RP-408 refinement):** Peak OSR ↑ *and*
final objective ↓ (both required). Zone-level metrics (ACR, APS, AOR) serve as diagnostic
explanations, not surrogate objectives.

---

## 9. Artefacts

| File | Description |
|------|-------------|
| `docs/roadef/rp409a_data/operator_attribution.csv` | Per-instance per-operator accepted contribution |
| `docs/roadef/rp409a_data/operator_attribution_summary.csv` | Aggregated summary |
| `docs/roadef/rp409a_data/transition_matrix.csv` | Zone transition matrix |
| `docs/roadef/rp409a_data/rp409a_analysis.log` | Narrative log |
| `scripts/rp409a_operator_attribution.py` | Analysis script |

---

*Report frozen. Data in `docs/roadef/rp409a_data/`. Do not modify.*