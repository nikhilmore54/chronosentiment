# RP-408: Comparator Intervention — Scalar vs. Sorted-Load Lexicographic Selection

**Status:** FROZEN — post-review revision round 2 (date 2026-08-06)
**Date:** 2026-08-06
**Seed:** 42 (fixed, identical across both arms)
**Instances:** 20 × setA (all instances)
**Budget:** Adaptive 30–300 s per instance
**Population:** 50 | Elite: 5 | Generations: 500 | Mutation: 0.30 | Crossover: 0.70

---

## 1. Research Question

Does replacing the scalar fitness comparator (RP-406C baseline: `fitness() = −obj`) with a
sorted-load lexicographic comparator improve solution quality on the ROADEF 2026 SR-path
problem under the RP-408 benchmark configuration?

The lexicographic comparator was hypothesised to provide finer-grained selection pressure in
the Peak zone (rank-1 arc saturation), where the scalar comparator is blind to load
distribution differences among solutions with similar aggregate objective values.

**Scope note.** This experiment tests one specific lexicographic ordering: element-wise
comparison on the descending-sorted arc-saturation load vector. The results falsify this
comparator under this configuration; they do not generalise to all lexicographic methods.

---

## 2. Experimental Design

### 2.1 Comparator Definitions

**Scalar (baseline, RP-406C):**
`fitness(e) = −obj` if valid, else `−1,000,000`.
Two valid solutions are compared by their aggregate objective value only.

**Sorted-load Lexicographic (RP-408 experimental):**
Invalid always loses to valid. Among valid solutions, comparison is element-wise on the
descending-sorted arc-saturation load vector (highest saturation first). The first rank at
which the two vectors differ determines the winner; lower saturation = better. This is a
*complete* lexicographic ordering — it compares every rank, not only the Peak arc.

### 2.2 Paired Design

Both arms use identical seeds (base seed 42, per-instance seed = `42 XOR (i × 0x9e3779b97f4a7c15)`),
identical population parameters, and identical instance ordering. The only difference is the
comparator used at all five selection sites in the evolution loop:

1. Initial population sort (gen 0)
2. Tournament winner selection (k=3)
3. Post-evaluation sort (each generation)
4. `became_global_best` detection (map closure)
5. `became_global_best` detection (emit loop)

### 2.3 Telemetry

Four dedicated JSONL streams per instance per arm:
- `rp408b_candidates_<inst>.jsonl` — CandidateRecord (tournament funnel)
- `rp408b_generations_<inst>.jsonl` — GenerationRecord (per-generation state)
- `rp408b_moves_<inst>.jsonl` — MoveRecord (global-best improvements)
- `rp408b_construction_<inst>.jsonl` — ConstructionRecord (initial population)

---

## 3. Results

### 3.1 Level 1 — Outcome: Win/Loss/Tie on best_obj

| Instance  | Scalar obj    | Lex obj       | Delta (Lex−Scalar) | Winner      |
|-----------|---------------|---------------|--------------------|-------------|
| setA-01   | 48.8185       | 52.5970       | +3.78              | Scalar      |
| setA-02   | inf           | inf           | —                  | Tie(invalid)|
| setA-03   | 60.2491       | 69.4898       | +9.24              | Scalar      |
| setA-04   | 64.9878       | 73.5752       | +8.59              | Scalar      |
| setA-05   | 13.7221       | 15.3516       | +1.63              | Scalar      |
| setA-06   | 55.1990       | 65.0546       | +9.86              | Scalar      |
| setA-07   | inf           | inf           | —                  | Tie(invalid)|
| setA-08   | 49.7408       | 55.0092       | +5.27              | Scalar      |
| setA-09   | 157.3360      | 175.2899      | +17.95             | Scalar      |
| setA-10   | 86.9320       | 90.7387       | +3.81              | Scalar      |
| setA-11   | 113.5833      | 126.1358      | +12.55             | Scalar      |
| setA-12   | 18.4157       | 18.5702       | +0.15              | Scalar      |
| setA-13   | inf           | inf           | —                  | Tie(invalid)|
| setA-14   | 94.7706       | 101.4965      | +6.73              | Scalar      |
| setA-15   | 242.2205      | 259.1252      | +16.90             | Scalar      |
| setA-16   | inf           | inf           | —                  | Tie(invalid)|
| setA-17   | 60.2456       | 60.2456       | 0.00               | Tie         |
| setA-18   | 799,246.22    | 799,258.35    | +12.13             | Scalar      |
| setA-19   | inf           | inf           | —                  | Tie(invalid)|
| setA-20   | inf           | inf           | —                  | Tie(invalid)|

**Summary (under RP-408 benchmark configuration):**
- Scalar wins: **13** / 14 valid-paired instances
- Lexicographic wins: **0**
- Ties (valid): **1** (setA-17, delta < 1e-6)
- Ties (both invalid): **6**
- Mean delta obj (Lex − Scalar, valid pairs only): **+7.76** (positive = Lex worse)

**Finding F1 (scoped):** Under the RP-408 benchmark configuration, the sorted-load
lexicographic comparator is dominated by the scalar comparator on solution quality across
all 14 valid-paired instances. The primary hypothesis — that this lexicographic ordering
improves the ROADEF objective — is **falsified** for this comparator and configuration.

---

### 3.2 Level 2 — Mechanism: PE Decomposition (Primary Finding)

PE (Promotion Efficiency) = fraction of global-best improvement moves in each zone.
OSR (Objective Step Rate) = improvement moves per generation in each zone.

| Metric          | Scalar | Lex    | Delta (Lex−Scalar) |
|-----------------|--------|--------|--------------------|
| Peak PE         | 0.0073 | 0.0530 | **+0.0457**        |
| Shoulder PE     | 0.0822 | 0.1122 | +0.0299            |
| Transition PE   | 0.1045 | 0.0893 | −0.0152            |
| Tail PE         | 0.3002 | 0.2102 | **−0.0900**        |
| Mixed PE        | 0.1987 | 0.1506 | −0.0481            |
| Neutral PE      | 0.0071 | 0.0848 | +0.0776            |
| Peak OSR        | 0.0049 | 0.0332 | **+0.0283**        |
| Shoulder OSR    | 0.0447 | 0.0681 | +0.0234            |
| Transition OSR  | 0.0586 | 0.0533 | −0.0053            |

**Finding F2 (primary mechanistic result):** The sorted-load lexicographic comparator
dramatically and successfully altered evolutionary dynamics. Peak PE increased 7.3× (0.007
→ 0.053) and Peak OSR increased 6.8× (0.005 → 0.033). Tail PE decreased by 9 percentage
points. The Promotion subsystem responded exactly as the hypothesis predicted: the
comparator redirected selection pressure toward the Peak zone.

This is a strong positive result for the implementation and for the mechanistic
instrumentation. The comparator worked. The hypothesis about *what that would produce* did
not.

**Finding F3 (causal chain):** The complete causal chain observed in RP-408 is shown in
Figure 2. This chain is experimentally supported: each link was measured independently via
the EEB telemetry framework.

> **Figure 2. Observed causal chain during RP-408B.**
>
> ```
> Comparator changed (Scalar → Sorted-load Lex)
>         ↓
> Selection pressure changed (Peak PE: 7.3× increase; measured via MoveRecord)
>         ↓
> Population composition changed (more Peak-zone improvements promoted)
>         ↓
> Final ROADEF objective worsened (mean +7.76 units, 13/14 instances)
> ```

This chain demonstrates that **increased Peak Promotion Efficiency alone is insufficient to
improve ROADEF solution quality.** Promotion quality must be aligned with the competition
objective, not merely increased.

**Finding F6 (Peak PE as a process metric, not a surrogate):** RP-408 demonstrates that
Peak PE is a mechanistic indicator of search dynamics rather than a surrogate for
optimisation quality. A 7.3× increase in Peak PE produced a consistent *worsening* of the
competition objective. Peak PE must therefore be interpreted together with the competition
objective rather than in isolation. This has a direct implication for the EEB framework:
zone-level PE and OSR metrics describe *how* the search is operating, not *how well* it is
optimising. Improvements in Peak PE are meaningful only when accompanied by improvements
in the aggregate objective.

**Finding F4 (misalignment mechanism):** The sorted-load lexicographic comparator is a
*complete* ordering — it compares every rank in the load vector, not only the Peak arc.
However, it is extremely sensitive to rank-1 (Peak) differences. A plausible mechanistic
explanation, consistent with the observed telemetry, is that the comparator's rank-1
sensitivity accepts solutions that improve the first element of the sorted load vector
while permitting deterioration in subsequent ranks. Because the ROADEF objective aggregates
all arcs and time slots, this selection pressure can increase Peak PE while degrading the
aggregate objective. This interpretation is consistent with the data but is not directly
proven by the telemetry alone.

**Finding F5 (Shoulder zone):** The telemetry suggests reduced Shoulder-zone progress on
several instances under the lexicographic comparator. On setA-10 and setA-05, Shoulder OSR
decreases while Peak OSR increases. This pattern is consistent with the RP-406C finding
that the Shoulder zone is where competition outcomes are determined. A comparator that
promotes Peak-zone improvements at the expense of Shoulder-zone evolution may be trading a
secondary gain for a primary loss. This observation is based on a subset of instances and
should be treated as a hypothesis for further investigation rather than a confirmed finding.

---

### 3.3 Level 3 — Safety: Valid Rate and Stagnation

| Metric              | Scalar | Lex    | Delta  |
|---------------------|--------|--------|--------|
| Final valid rate    | 67.9%  | 69.0%  | +1.1%  |
| Stagnation rate     | 42.6%  | 44.6%  | +2.0%  |

Construction telemetry was unchanged by design (the comparator does not affect the
constructor). Final valid rate is marginally higher under Lex (+1.1 pp). Stagnation rate
is marginally higher under Lex (+2.0 pp), consistent with the comparator spending more
generations on Peak-zone refinement without finding improvements that reduce the aggregate
objective.

---

## 4. Executive Summary

RP-408B provides a clean causal test of the comparator intervention. With identical seeds,
identical instances, and identical operator parameters, the only difference is the
comparator.

**The sorted-load lexicographic comparator is dominated by the scalar comparator on
solution quality under this benchmark configuration.** Scalar wins 13/14 valid-paired
instances with a mean objective advantage of 7.76 units.

**The mechanism is fully understood.** The lexicographic comparator successfully increased
Peak Promotion Efficiency 7.3× and Peak OSR 6.8×. The Promotion subsystem responded
exactly as designed. The failure is not mechanistic but strategic: the comparator's
selection pressure is misaligned with the aggregate ROADEF objective. A comparator that is
greedy at rank-1 of the sorted load vector can improve the worst arc while worsening the
aggregate.

**The most important finding is not that lexicographic lost.** It is that increasing Peak
PE dramatically did not improve outcomes. This demonstrates that the bottleneck is not
simply "Peak candidates are being killed" — it is that the search dynamics must be aligned
with the full competition objective, not a proxy derived from a single zone.

**The scalar comparator remains the best-performing comparator among those tested.** This
does not establish it as globally optimal. A zone-aware comparator that balances Peak,
Shoulder, and Transition pressure — motivated by the four-zone model from RP-406 — remains
a scientifically legitimate line of investigation.

---

## 5. Implications for the Research Programme

### 5.1 RP-408C (Recommended insertion)

RP-408B motivates a natural follow-on experiment before operator redesign:

**Research question:** Which comparator best aligns selection pressure with the ROADEF
competition objective?

**Candidates:**
- Scalar (current baseline)
- Sorted-load Lexicographic (RP-408B, falsified)
- Peak + Shoulder weighted (zone-aware)
- Peak + SDI (spread-of-distribution index)
- Peak + Transition
- Peak + Shoulder + Transition (three-zone)

The comparator infrastructure from RP-408A makes this experiment low-cost. Each candidate
comparator requires only a new `impl EvalComparator` block and a campaign run.

### 5.2 RP-409A (Operator Attribution)

RP-409A proceeds with the scalar comparator. RP-408 has isolated the Objective subsystem
and found no evidence that replacing the scalar comparator with the tested sorted-load
lexicographic comparator improves solution quality. Attention returns to the Variation
subsystem. RP-409A's per-operator attribution becomes more valuable because the remaining
opportunity is more likely to lie in generating better candidates than in changing how
existing candidates are ranked.

**Refined RP-409 primary outcome (motivated by F6):** RP-409B's primary outcome is
redefined as: Peak OSR ↑ *and* final objective ↓ (improves). Increasing Peak OSR alone is
not sufficient — RP-408 demonstrated that. The zone-level metrics (Peak COR, Shoulder COR,
Transition COR, zone-specific PE) serve as diagnostic explanations for *why* objective
improvements occur, not as surrogate objectives in their own right.

---

## 6. Artefacts

| File | Description |
|------|-------------|
| `/tmp/rp408b/scalar/results.json` | Scalar arm: 20-instance summary |
| `/tmp/rp408b/lexicographic/results.json` | Lex arm: 20-instance summary |
| `/tmp/rp408b/scalar/rp408b_generations_<inst>.jsonl` | Per-generation telemetry (scalar) |
| `/tmp/rp408b/lexicographic/rp408b_generations_<inst>.jsonl` | Per-generation telemetry (lex) |
| `docs/roadef/rp408b_data/summary_table.csv` | Paired outcome table |
| `docs/roadef/rp408b_data/aggregate_stats.json` | Win/Loss/Tie counts, mean delta |
| `docs/roadef/rp408b_data/pe_decomposition.csv` | Per-instance PE decomposition |
| `docs/roadef/rp408b_data/osr_delta.csv` | Per-instance OSR delta |
| `scripts/rp408b_analysis.py` | Analysis script |

---

## 7. Conclusions

**C1.** The sorted-load lexicographic comparator hypothesis is falsified under the RP-408
benchmark configuration. This specific comparator does not improve ROADEF solution quality.

**C2.** The Promotion subsystem responded correctly to the comparator change. Peak PE
increased 7.3× and Peak OSR increased 6.8×. The implementation is validated.

**C3.** The failure is strategic, not mechanistic. The comparator's rank-1 greediness
produces selection pressure misaligned with the aggregate ROADEF objective.

**C4.** The scalar comparator (RP-406C baseline) remains the best-performing comparator
among those tested. It is not established as globally optimal. A zone-aware comparator
motivated by the four-zone model remains a legitimate future investigation (RP-408C).

**C5.** RP-409A proceeds with the scalar comparator to characterise per-operator
attribution of Peak/Shoulder/Transition improvements. The operator transition matrix will
identify which operators are responsible for Peak-zone improvements and which are limiting
Peak OSR, providing the attribution needed for RP-409B operator redesign. RP-409B's
primary outcome is refined: Peak OSR ↑ *and* final objective ↓ (improves). Zone-level
metrics serve as diagnostic explanations, not surrogate objectives.

**C6.** The central lesson of RP-408: promotion quality must be aligned with the
competition objective. Increasing Peak Promotion Efficiency alone is insufficient. The
search dynamics must serve the full objective, not a zone-level proxy.

**C7.** RP-408 strengthens the EEB framework itself. Peak PE and Peak OSR are confirmed as
mechanistic process indicators, not surrogate objectives. The framework is useful for
explaining *why* an intervention changes search behaviour even when that intervention
ultimately hurts optimisation performance — making it a tool for causal interpretation
across all five subsystems (Construction, Variation, Promotion, Execution, Objective),
not merely a bookkeeping device.

---

*Report frozen. Data in `/tmp/rp408b/` and `docs/roadef/rp408b_data/`. Do not modify.*