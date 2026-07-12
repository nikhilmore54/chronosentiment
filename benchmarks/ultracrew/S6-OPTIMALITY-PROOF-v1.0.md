# Sprint 6 — Optimality Proof for UB-001
## S6-OPTIMALITY-PROOF-v1.0

**Frozen:** 2026-07-13  
**Branch:** governance-hardening  
**Commits:** d898be02 → 43fb56ab

---

## Summary

Sprint 6 proves that the Coralys MOGA optimizer has reached the optimum of the
current UB-001 optimization problem. The proof is analytical, not empirical.

**GA best fitness: 9918.4**  
**Integer-constrained optimum: 9918.4**  
**Gap: 0.0**

---

## Assumptions Bounding the Proof

The following six assumptions must all hold for the proof to apply:

1. **Objective function:** SC1 = `variance(worker_hours) × 10.0`, SC2 = `historical_fatigue × hours × 2.0`
2. **SC2 = 0:** UB-001 runs pass `historical_workloads: null`, so no fatigue penalty applies
3. **Independent weekly optimization:** The server model runs each week as a separate optimization (start_hour < 168 constraint); SC1 is computed per week, not across the full 4-week horizon
4. **Uniform shift duration:** All UB-001 shifts are exactly 8 hours
5. **SC1 definition:** Variance is computed over all 20 workers including those with 0 shifts assigned
6. **Hard constraints satisfied:** HC1=0, HC2=0, HC3=0, Rest=0 (confirmed by H1/H2)

If any assumption changes, the proof no longer applies and the optimum may differ.

---

## Proof (H4d)

Each week: 83 shifts × 8h = 664h total, 20 workers, mean = 33.2h/worker.

With 8h shifts, each worker receives an integer number of shifts. The minimum
variance distribution assigns:
- 3 workers: 5 shifts = 40h
- 17 workers: 4 shifts = 32h

Verification: 3×40 + 17×32 = 120 + 544 = 664h ✓

Mean = 664/20 = 33.2h  
Variance = [3×(40−33.2)² + 17×(32−33.2)²] / 20  
         = [3×46.24 + 17×1.44] / 20  
         = [138.72 + 24.48] / 20  
         = 163.2 / 20  
         = 8.16  
SC1 = 8.16 × 10 = **81.6**  
Maximum fitness = 10000 − 81.6 = **9918.4**

No assignment of 83 shifts of 8h to 20 workers can achieve lower variance than
this distribution. The GA achieves exactly this distribution every week.

---

## Hypothesis Chain

| Hypothesis | Result | What it eliminated |
|---|---|---|
| H1: skill-aware init | ✅ Confirmed | HC1 repair consuming search effort |
| H2: constraint-aware init | ✅ Confirmed | HC2/HC3/rest repair consuming search effort |
| H3a: diversity probe | ❌ Refuted | Premature convergence as bottleneck |
| H3b: long-run plateau | ✅ Confirmed | Insufficient generations as bottleneck |
| H3c-1: workload swap | ✅ Confirmed (ceiling) | Single-shift rebalancing as path to improvement |
| H4: greedy bound (per-skill) | Inconclusive | Greedy not a true lower bound |
| H4b: greedy bound (multi-skill) | Inconclusive | GA outperforms greedy |
| H4c: integer bound (4-week) | Inconclusive | Wrong unit of analysis |
| H4d: integer bound (per-week) | ✅ **Proven** | Any gap between GA and optimum — gap = 0 |

---

## Benchmark Artifacts

| File | Purpose |
|---|---|
| `UB-001-v1.0.json` | Canonical benchmark definition (frozen) |
| `UB-001-BASELINE-v1.0.json` | Sprint 3 baseline (HC1=0, Gen0avg≈−5000) |
| `UB-001-H2-v1.0.json` | Sprint 4 H2 results (Gen0avg +6495) |
| `UB-001-DIVERSITY-v1.0.json` | Sprint 5 H3a diversity probe |
| `UB-001-H3B-v1.0.json` | Sprint 5 H3b long-run plateau (200 gens) |
| `UB-001-H4-LOWER-BOUND-v1.0.json` | H4 greedy bound (per-skill) |
| `UB-001-H4B-MULTISKILL-v1.0.json` | H4b greedy bound (multi-skill) |

---

## UB-001's Permanent Role

UB-001 is solved optimally under its current assumptions. This does not mean
it should be retired. It should be retained as the **permanent regression and
correctness benchmark** for the following reasons:

1. **Regression:** Any future change to the optimizer, constraint engine, or
   server model that causes UB-001 to return a fitness below 9918.4 is a
   regression. The proof provides an exact expected value, not just a direction.

2. **Correctness:** HC1=0, HC2=0, HC3=0, Rest=0 on UB-001 is the baseline
   correctness check. If hard constraints reappear, something broke.

3. **Speed:** UB-001 runs in ~500ms on the release binary (200 gens). It is
   fast enough to run on every commit as a smoke test.

---

## What UB-001 Cannot Test

The following objectives are not exercised by UB-001 and require new benchmarks:

- **SC2 (fatigue):** Requires `historical_workloads` to be non-null
- **Cross-week fairness:** Requires a multi-week objective function
- **Mixed shift durations:** Requires shifts of varying length (4h, 8h, 12h)
- **Employee preferences:** Requires a preference objective
- **Leave and disruption replanning:** Requires locked assignments and partial schedules

---

## Recommended Benchmark Progression

| Benchmark | New capability tested |
|---|---|
| UB-001 | Core feasibility + workload balance (solved, regression only) |
| UB-002 | Historical workload + SC2 fatigue penalty |
| UB-003 | Mixed shift durations (4h/8h/12h) |
| UB-004 | Employee preferences (soft constraint) |
| UB-005 | Leave disruptions and replanning (locked assignments) |

Each new benchmark should follow the same methodology: freeze the instance,
state a quantitative hypothesis, measure, and prove or refute before proceeding.

---

## Conclusion

> **UB-001 is solved optimally under the current objective function and benchmark
> assumptions. Future optimizer research should proceed on more expressive
> benchmark instances (UB-002+) or richer objective functions, while UB-001
> remains the permanent regression and correctness benchmark.**