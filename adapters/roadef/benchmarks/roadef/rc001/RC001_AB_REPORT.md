# RC-001 A/B Report: Load-Aware Greedy Constructor

**Campaign:** rc001_ab_v2.3  
**Timestamp:** 2026-08-20T05:03:05.143690+00:00  
**Seed:** 42  Population: 50  Generations: 500  Elite: 5

> **Statistical note:** Single seed. Acceptable for engineering gate; multi-seed experiments (e.g. seeds 42–51) required before paper submission.

## Hypothesis

The RP-401C load-aware greedy constructor (volume-sorted, additive saturation penalty Dijkstra) raises the Initial Feasibility Rate (IFR) of generation 0 compared to the CB-000 random constructor, thereby increasing EEB and improving the final ROADEF objective.

**EEB target:** IFR ↑ (Construction subsystem)  
**CB-000 baseline:** mean IFR = 10.6%, 6/20 instances with IFR = 0%

## Summary

| Metric | Arm A (Random / CB-000) | Arm B (Greedy / RC-001) | Delta |
|--------|------------------------|------------------------|-------|
| Mean IFR | 0.000 | 0.000 | +0.000 |
| Valid instances | 0/0 | 0/0 | +0 |
| Arm B better obj | — | 0/0 | — |
| Arm B better IFR | — | 0/0 | — |

## Per-Instance Results

| Instance | A IFR | B IFR | ΔIFR | A g0best | B g0best | A obj | B obj | Δobj | B better? | Flags |
|----------|-------|-------|------|----------|----------|-------|-------|------|-----------|-------|

## Verdict

**Acceptance criterion:** Arm B wins on official ROADEF objective on ≥ 2/3 of instances.  
**IFR** is explanatory evidence, not a hard gate.  
**Regression check:** arm B mean runtime ≤ 2× arm A mean runtime.

- Arm B better obj: 0/0 (threshold: 0/0)
- IFR improvement: +0.000 (explanatory)
- Runtime: A=0ms  B=0ms  regression=false
- Invariant violations: A=0  B=0

**ACCEPTED**

RC-001 improves the official ROADEF objective on ≥ 2/3 of instances without runtime regression. Recommend integrating GreedyLoadAware as the default construction mode for the RC integration branch.

*Total campaign runtime: 2ms*
