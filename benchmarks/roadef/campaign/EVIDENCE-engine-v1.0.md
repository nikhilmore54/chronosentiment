# ROADEF 2026 — Platform Validation Evidence

**Campaign:** campaign_engine_v1.0_verify  
**Engine:** coralys-moga EvolutionEngine (unchanged)  
**Timestamp:** 2026-07-11T03:10:46.371453+00:00  
**Total runtime:** 3958.2s  

## Platform Evidence

This campaign validates that `coralys-moga EvolutionEngine` generalizes to ROADEF
without modification. The engine's generic bounds (`G: Genome, F: FitnessEvaluator<G>,
`M: MutationOperator<G>, C: CrossoverOperator<G>`) were sufficient to accept a
completely different solution space (SR-path waypoints vs CVRP permutations).

## Summary

| Metric | Value |
|--------|-------|
| Total instances | 20 |
| Valid solutions | 18 |
| Invalid solutions | 2 |

## Per-Instance Results

| # | Instance | Demands | Nodes | Links | Slots | Budget(s) | Obj | MLU | Valid | Class | ms | Gens | ms/gen | Mode | Termination |
|---|----------|---------|-------|-------|-------|-----------|-----|-----|-------|-------|----|------|--------|------|-------------|
| 1 | setA-01 | 40 | 20 | 80 | 2 | 30 | 48.3587 | 0.7105 | ✓ | Competitive | 5271 | 54 | 97 | SearchLimited | NoImprovement |
| 2 | setA-02 | 45 | 30 | 150 | 2 | 30 | 52.4210 | 0.7258 | ✓ | Competitive | 25940 | 136 | 190 | SearchLimited | NoImprovement |
| 3 | setA-03 | 20 | 50 | 250 | 2 | 30 | 59.2769 | 0.6793 | ✓ | Competitive | 11401 | 67 | 170 | SearchLimited | NoImprovement |
| 4 | setA-04 | 200 | 50 | 250 | 2 | 30 | 60.6549 | 0.6900 | ✓ | Weak | 30713 | 24 | 1279 | EvaluationLimited | TimeBudget |
| 5 | setA-05 | 100 | 100 | 396 | 2 | 30 | 13.2668 | 0.1859 | ✓ | Good | 30181 | 19 | 1588 | EvaluationLimited | TimeBudget |
| 6 | setA-06 | 500 | 100 | 500 | 2 | 125 | 58.3483 | 0.6338 | ✓ | Competitive | 126432 | 19 | 6654 | EvaluationLimited | TimeBudget |
| 7 | setA-07 | 800 | 100 | 500 | 2 | 200 | 297.3074 | 0.8923 | ✓ | Poor | 206050 | 17 | 12120 | EvaluationLimited | TimeBudget |
| 8 | setA-08 | 200 | 150 | 654 | 2 | 65 | 48.0915 | 0.4913 | ✓ | Competitive | 66403 | 14 | 4743 | EvaluationLimited | TimeBudget |
| 9 | setA-09 | 200 | 150 | 750 | 2 | 75 | 157.4063 | 0.7304 | ✓ | Poor | 79524 | 17 | 4677 | EvaluationLimited | TimeBudget |
| 10 | setA-10 | 1000 | 150 | 966 | 2 | 300 | 88.6543 | 0.6630 | ✓ | Weak | 303118 | 15 | 20207 | EvaluationLimited | TimeBudget |
| 11 | setA-11 | 400 | 200 | 1000 | 2 | 200 | 107.8047 | 0.7274 | ✓ | Poor | 211157 | 17 | 12421 | EvaluationLimited | TimeBudget |
| 12 | setA-12 | 400 | 200 | 898 | 2 | 179 | 19.2967 | 0.7600 | ✓ | Good | 180256 | 13 | 13865 | EvaluationLimited | TimeBudget |
| 13 | setA-13 | 2000 | 200 | 1000 | 2 | 300 | 99.0444 | 0.9188 | ✓ | Weak | 350062 | 5 | 70012 | EvaluationLimited | TimeBudget |
| 14 | setA-14 | 600 | 250 | 1108 | 2 | 300 | 95.9012 | 0.6015 | ✓ | Weak | 304110 | 12 | 25342 | EvaluationLimited | TimeBudget |
| 15 | setA-15 | 600 | 250 | 1250 | 2 | 300 | 238.3203 | 0.8620 | ✓ | Poor | 313660 | 13 | 24127 | EvaluationLimited | TimeBudget |
| 16 | setA-16 | 4800 | 250 | 1452 | 2 | 300 | ∞ | 1.5307 | ✗ | Invalid | 303028 | 0 | — | Infeasible | TimeBudget |
| 17 | setA-17 | 2000 | 300 | 1270 | 2 | 300 | 60.1059 | 0.4372 | ✓ | Weak | 322497 | 4 | 80624 | EvaluationLimited | TimeBudget |
| 18 | setA-18 | 2000 | 300 | 1500 | 2 | 300 | 799246.6026 | 0.8455 | ✓ | Poor | 383668 | 5 | 76733 | EvaluationLimited | TimeBudget |
| 19 | setA-19 | 6000 | 300 | 1998 | 2 | 300 | 241.9896 | 0.9696 | ✓ | Poor | 342059 | 2 | 171029 | EvaluationLimited | TimeBudget |
| 20 | setA-20 | 6000 | 400 | 2000 | 2 | 300 | ∞ | 1.0204 | ✗ | Invalid | 346441 | 0 | — | Infeasible | TimeBudget |

## Platform Validation Criteria

| Criterion | Status |
|-----------|--------|
| EvolutionEngine used unchanged | ✓ PASS |
| All instances load | ✓ PASS |
| Engine runs end-to-end | ✓ PASS |
| Zero modifications to coralys-moga | ✓ PASS |
| Zero modifications to Qualification Subsystem v1.0 | ✓ PASS |
