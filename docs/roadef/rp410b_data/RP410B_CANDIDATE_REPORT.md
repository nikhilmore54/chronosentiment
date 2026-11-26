# RP-410B Candidate Pipeline Funnel Analysis

**Telemetry source:** `/tmp/rp410_telemetry_v3`
**Total candidates observed:** 14,600

---

## Executive Summary

RP-410B provides the first evidence-based answer to the question RP-410A could not resolve:

> **Are Peak improvements absent because they are never generated, or because they are filtered out by repair/evaluation?**

The answer is unambiguous: **Peak-improving candidates are routinely generated and remain fully feasible after evaluation. The bottleneck is not generation and not repair.**

| Stage | Count | Conversion |
|-------|------:|-----------:|
| Generated | 14,600 | 100% |
| Valid | 10,503 | 71.94% |
| Accepted (new global best) | 351 | 2.40% |
| Peak-improving generated | 133 | 0.91% |
| Peak-improving valid | 133 | 100.00% of Peak generated |
| Peak-improving accepted (new global best) | 4 | 3.01% of Peak valid |
| Shoulder-improving generated | 1,023 | 7.01% |
| Shoulder-improving valid | 1,023 | 100.00% of Shoulder generated |
| Shoulder-improving accepted (new global best) | 65 | 6.35% of Shoulder valid |

**The repair stage does not destroy Peak candidates.** All 133 generated Peak candidates survived evaluation. This eliminates repair as a bottleneck hypothesis.

**What remains open:** Why do only 4 of 133 valid Peak candidates become new global-best solutions? The current telemetry records whether a candidate becomes the new global best, but does not record tournament selection outcomes or scalar objective deltas. The bottleneck could be population selection, the scalar objective, or simply the rarity of candidates that are simultaneously better on all objectives. RP-410C is the experiment that can separate these possibilities.

---

## 1. Per-Operator Funnel

| Operator | Generated | Valid | Valid% | Accepted | Accept% | Peak Gen | Peak Accept |
|----------|----------:|------:|-------:|---------:|--------:|---------:|------------:|
| crossover | 7,530 | 5,413 | 71.89% | 231 | 3.07% | 70 | 3 |
| crossover+mutation | 3,232 | 2,237 | 69.21% | 96 | 2.97% | 35 | 1 |
| elite | 1,460 | 1,162 | 79.59% | 0 | 0.00% | 14 | 0 |
| mutation | 2,378 | 1,691 | 71.11% | 24 | 1.01% | 14 | 0 |

**Peak discovery is dominated by crossover.** Crossover generates 70 of 133 Peak candidates (52.6%) and accounts for 3 of 4 Peak acceptances. Mutation generates 14 Peak candidates but none are accepted as new global bests. This is an observational profile — it describes what was generated and accepted, not what was attempted or rejected by tournament selection.

**Structural bias:** Variation operators generate Shoulder candidates at nearly 8× the rate of Peak candidates (1,023 vs 133). This is a structural property of the current operator design, not a selection artefact.

---

## 2. Per-Instance Funnel

| Instance | Generated | Valid% | Accepted% | Peak Gen | Peak Accept |
|----------|----------:|-------:|----------:|---------:|------------:|
| setA-01 | 3,800 | 96.74% | 1.00% | 0 | 0 |
| setA-02 | 1,050 | 0.00% | 0.00% | 0 | 0 |
| setA-03 | 1,950 | 91.23% | 0.82% | 1 | 1 |
| setA-04 | 700 | 90.86% | 5.00% | 43 | 2 |
| setA-05 | 500 | 100.00% | 3.60% | 8 | 0 |
| setA-06 | 650 | 80.46% | 5.38% | 12 | 1 |
| setA-07 | 750 | 66.27% | 4.27% | 13 | 0 |
| setA-08 | 450 | 80.67% | 3.56% | 23 | 0 |
| setA-09 | 500 | 89.60% | 4.60% | 0 | 0 |
| setA-10 | 650 | 61.23% | 4.31% | 22 | 0 |
| setA-11 | 500 | 90.60% | 5.60% | 0 | 0 |
| setA-12 | 500 | 77.20% | 5.60% | 0 | 0 |
| setA-13 | 750 | 0.00% | 0.00% | 0 | 0 |
| setA-14 | 450 | 80.89% | 4.44% | 7 | 0 |
| setA-15 | 400 | 87.25% | 4.25% | 0 | 0 |
| setA-16 | 250 | 0.00% | 0.00% | 0 | 0 |
| setA-17 | 100 | 84.00% | 8.00% | 4 | 0 |
| setA-18 | 400 | 11.75% | 2.25% | 0 | 0 |
| setA-19 | 150 | 0.00% | 0.00% | 0 | 0 |
| setA-20 | 100 | 0.00% | 0.00% | 0 | 0 |

**Notable contrast:** setA-04 generates 43 Peak candidates and converts 2 to new global bests. setA-08 generates 23 Peak candidates and converts none. Both instances produce Peak candidates; the difference lies downstream of generation and repair. This is consistent with the hypothesis that the bottleneck is in the promotion step (population selection or scalar objective), but the current telemetry cannot distinguish between these.

---

## 3. Peak Zone Analysis

Peak candidates generated: 133. Of these, 133 (100.00%) were valid after evaluation. Of valid Peak candidates, 4 (3.01%) became new global-best solutions.

**The strongest statement the data supports:**

> Peak-improving candidates are routinely generated and remain feasible after evaluation, but only a very small fraction (3.01%) become new global-best solutions.

This statement is observational. It does not claim that the scalar objective rejects Peak candidates, that tournament selection eliminates them, or that they are dominated by other candidates. Those are hypotheses that require RP-410C instrumentation to test.

---

## 4. Candidate Pipeline Interpretation Framework

The funnel determines which subsystem is the bottleneck for Peak improvement. Based on RP-410B data:

**Generation failure** (`peak_generated = 0`): Variation operators cannot produce Peak-improving candidates.
*Status: Ruled out. 133 Peak candidates generated.*

**Repair failure** (`peak_valid = 0` but `peak_generated > 0`): Peak candidates are generated but invalidated by evaluation.
*Status: Ruled out. All 133 Peak candidates survived evaluation.*

**Global-best promotion failure** (`peak_accepted = 0` but `peak_valid > 0`): Valid Peak candidates exist but only 4 become new global-best solutions.
*Status: Observed. The bottleneck is in the promotion step. The current telemetry cannot distinguish between:*
- *Population selection eliminating Peak candidates before they can compete for global best*
- *The scalar objective ranking Peak candidates below non-Peak alternatives*
- *Peak candidates being genuinely dominated (better on rank-1 but worse on ranks 2–50)*

**Required next experiment (RP-410C):** For each generated candidate, record the scalar objective delta, the tournament selection outcome, and the per-zone delta breakdown. This will directly test whether the scalar objective is the bottleneck or whether the issue lies in population selection.

---

## 5. Implications for Research Programme

RP-410B has substantially narrowed the hypothesis space:

The pipeline up to and including repair is **not** the bottleneck for Peak improvement. Changing the constructor (RP-412) or the repair heuristic will not increase Peak acceptance unless it also changes the scalar objective landscape.

The bottleneck is in the promotion step — either population selection, the scalar objective, or both. RP-410C is the experiment that can separate these possibilities. Only after RP-410C is complete will it be possible to determine whether RP-408 (lexicographic objective) is the correct intervention.

**Updated dependency graph:**

```
RP-412 (Construction)
    │
    ▼
RP-411 (Execution throughput)
    │
    ▼
RP-410B (Candidate pipeline) ← this document
    │
    ▼
RP-410C (Selection decision analysis) ← highest-value next experiment
    │
    ▼
RP-408 (Lexicographic objective)
    │
    ▼
RP-409 (Operator redesign)
```

These are observational profiles from a single-seed campaign. They describe what was generated and accepted, not what was attempted or rejected by tournament selection.
