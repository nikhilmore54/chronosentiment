# RC-001 A/B Report: Load-Aware Greedy Constructor

**Campaign:** rc001_ab_v2.3  
**Timestamp:** 2026-08-06T17:29:50.708844+00:00  
**Seed:** 42  Population: 50  Generations: 500  Elite: 5

> **Statistical note:** Single seed. Acceptable for engineering gate; multi-seed experiments (e.g. seeds 42–51) required before paper submission.

## Hypothesis

The RP-401C load-aware greedy constructor (volume-sorted, additive saturation penalty Dijkstra) raises the Initial Feasibility Rate (IFR) of generation 0 compared to the CB-000 random constructor, thereby increasing EEB and improving the final ROADEF objective.

**EEB target:** IFR ↑ (Construction subsystem)  
**CB-000 baseline:** mean IFR = 10.6%, 6/20 instances with IFR = 0%

## Summary

| Metric | Arm A (Random / CB-000) | Arm B (Greedy / RC-001) | Delta |
|--------|------------------------|------------------------|-------|
| Mean IFR | 0.124 | 0.587 | +0.463 |
| Valid instances | 15/20 | 11/20 | -4 |
| Arm B better obj | — | 11/20 | — |
| Arm B better IFR | — | 13/20 | — |
| ⚠ Invariant violations | 0 | 2 | — |

## ⚠ Invariant Violation Warning

One or more instances produced `IFR=1.0, valid=false, obj=inf`. This combination is a potential correctness failure: the constructor counted all genomes as feasible but the evaluator rejected them all. Possible causes:

- IFR measures something different from evaluator feasibility.
- The waypoint conversion produces an invalid representation.
- The evaluator rejects genomes that the constructor counts as feasible.
- A bug in the GreedyLoadAware path generation.

**Investigate before trusting performance conclusions from affected instances.**

## Per-Instance Results

| Instance | A IFR | B IFR | ΔIFR | A g0best | B g0best | A obj | B obj | Δobj | B better? | Flags |
|----------|-------|-------|------|----------|----------|-------|-------|------|-----------|-------|
| setA-01 | 0.160 | 1.000 | +0.840 | 64.8882 | 52.5480 | 47.9952 | 47.9864 | -0.0088 | ✓ obj |  |
| setA-02 | 0.000 | 0.000 | +0.000 | ∞ | ∞ | 54.3719 | inf | +inf | ✗ |  |
| setA-03 | 0.060 | 0.020 | -0.040 | 102.8826 | 95.4646 | 60.4986 | 58.4415 | -2.0571 | ✓ obj |  |
| setA-04 | 0.200 | 1.000 | +0.800 | 81.8163 | 61.1173 | 64.2890 | 60.2371 | -4.0519 | ✓ obj |  |
| setA-05 | 0.800 | 0.000 | -0.800 | 15.9751 | ∞ | 13.2877 | inf | +inf | ✗ |  |
| setA-06 | 0.060 | 0.760 | +0.700 | 73.9347 | 53.3372 | 49.9871 | 46.5986 | -3.3885 | ✓ obj |  |
| setA-07 | 0.000 | 1.000 | +1.000 | ∞ | 195.9247 | 255.4651 | 194.0423 | -61.4228 | ✓ obj |  |
| setA-08 | 0.080 | 0.000 | -0.080 | 60.0606 | ∞ | 46.4887 | inf | +inf | ✗ |  |
| setA-09 | 0.160 | 1.000 | +0.840 | 185.3801 | 142.5698 | 153.6887 | 142.2418 | -11.4468 | ✓ obj |  |
| setA-10 | 0.100 | 0.860 | +0.760 | 118.3143 | 72.5310 | 83.4466 | 69.1803 | -14.2663 | ✓ obj |  |
| setA-11 | 0.260 | 0.100 | -0.160 | 118.5535 | 101.6986 | 108.6026 | 99.6580 | -8.9446 | ✓ obj |  |
| setA-12 | 0.000 | 0.160 | +0.160 | ∞ | 20.6724 | inf | 19.8051 | -inf | ✓ obj |  |
| setA-13 | 0.000 | 0.980 | +0.980 | ∞ | 56.4319 | inf | 56.4319 | -inf | ✓ obj |  |
| setA-14 | 0.120 | 0.000 | -0.120 | 114.8593 | ∞ | 91.0459 | inf | +inf | ✗ |  |
| setA-15 | 0.140 | 1.000 | +0.860 | 268.0020 | 209.4852 | 238.8201 | 209.1634 | -29.6567 | ✓ obj |  |
| setA-16 | 0.000 | 0.880 | +0.880 | ∞ | 108.9102 | inf | inf | NaN | ✓ IFR |  |
| setA-17 | 0.340 | 0.000 | -0.340 | 59.2292 | ∞ | 58.4351 | inf | +inf | ✗ |  |
| setA-18 | 0.000 | 1.000 | +1.000 | ∞ | 799168.2230 | 799256.7468 | inf | +inf | ✓ IFR | ⚠B |
| setA-19 | 0.000 | 0.980 | +0.980 | ∞ | 112.3328 | inf | inf | NaN | ✓ IFR |  |
| setA-20 | 0.000 | 1.000 | +1.000 | ∞ | 447.8051 | inf | inf | NaN | ✓ IFR | ⚠B |

## Verdict

**Acceptance criterion:** Arm B wins on official ROADEF objective on ≥ 2/3 of instances.  
**IFR** is explanatory evidence, not a hard gate.  
**Regression check:** arm B mean runtime ≤ 2× arm A mean runtime.

- Arm B better obj: 11/20 (threshold: 14/20)
- IFR improvement: +0.463 (explanatory)
- Runtime: A=221302ms  B=391060ms  regression=false
- Invariant violations: A=0  B=2

**CORRECTNESS FAILURE — RETURN TO IMPLEMENTATION**

RC-001 produced 2 instance(s) with IFR=1.0, valid=false, obj=inf. This is a constructor defect, not an optimisation result. The benchmark cannot produce a fair acceptance/rejection decision while correctness failures are present. RC-001 must return to the Implementation stage for a bug fix and be re-benchmarked under the same lifecycle.

*Total campaign runtime: 12278076ms*
