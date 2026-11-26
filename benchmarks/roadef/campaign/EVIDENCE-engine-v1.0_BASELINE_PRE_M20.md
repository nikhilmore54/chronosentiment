# ROADEF 2026 — Platform Validation Evidence

**Campaign:** campaign_engine_v1.0_verify  
**Engine:** coralys-moga EvolutionEngine (unchanged)  
**Timestamp:** 2026-07-10T17:01:23.300851+00:00  
**Total runtime:** 4199.3s  

## Platform Evidence

This campaign validates that `coralys-moga EvolutionEngine` generalizes to ROADEF
without modification. The engine's generic bounds (`G: Genome, F: FitnessEvaluator<G>,
`M: MutationOperator<G>, C: CrossoverOperator<G>`) were sufficient to accept a
completely different solution space (SR-path waypoints vs CVRP permutations).

## Summary

| Metric | Value |
|--------|-------|
| Total instances | 20 |
| Valid solutions | 17 |
| Invalid solutions | 3 |

## Per-Instance Results

| # | Instance | Demands | Nodes | Links | Slots | Budget(s) | Obj | MLU | Valid | Class | ms | Gens | ms/gen | Mode | Termination |
|---|----------|---------|-------|-------|-------|-----------|-----|-----|-------|-------|----|------|--------|------|-------------|
| 1 | setA-01 | 40 | 20 | 80 | 2 | 30 | 47.9176 | 0.7105 | ✓ | Competitive | 14965 | 114 | 131 | SearchLimited | NoImprovement |
| 2 | setA-02 | 45 | 30 | 150 | 2 | 30 | 52.6536 | 0.7258 | ✓ | Competitive | 27292 | 103 | 264 | EvaluationLimited | TimeBudget |
| 3 | setA-03 | 20 | 50 | 250 | 2 | 30 | 59.1585 | 0.6793 | ✓ | Competitive | 13665 | 71 | 192 | SearchLimited | NoImprovement |
| 4 | setA-04 | 200 | 50 | 250 | 2 | 30 | 64.8094 | 0.6900 | ✓ | Weak | 31368 | 18 | 1742 | EvaluationLimited | TimeBudget |
| 5 | setA-05 | 100 | 100 | 396 | 2 | 30 | 13.2801 | 0.1859 | ✓ | Good | 30224 | 17 | 1777 | EvaluationLimited | TimeBudget |
| 6 | setA-06 | 500 | 100 | 500 | 2 | 125 | 48.6073 | 0.5352 | ✓ | Competitive | 132909 | 16 | 8306 | EvaluationLimited | TimeBudget |
| 7 | setA-07 | 800 | 100 | 500 | 2 | 200 | 261.6493 | 0.8360 | ✓ | Poor | 211961 | 14 | 15140 | EvaluationLimited | TimeBudget |
| 8 | setA-08 | 200 | 150 | 654 | 2 | 65 | 53.3019 | 0.5576 | ✓ | Competitive | 70806 | 12 | 5900 | EvaluationLimited | TimeBudget |
| 9 | setA-09 | 200 | 150 | 750 | 2 | 75 | 153.7590 | 0.7682 | ✓ | Poor | 78831 | 14 | 5630 | EvaluationLimited | TimeBudget |
| 10 | setA-10 | 1000 | 150 | 966 | 2 | 300 | 86.9824 | 0.6576 | ✓ | Weak | 303699 | 11 | 27609 | EvaluationLimited | TimeBudget |
| 11 | setA-11 | 400 | 200 | 1000 | 2 | 200 | 110.1578 | 0.7297 | ✓ | Poor | 200172 | 13 | 15397 | EvaluationLimited | TimeBudget |
| 12 | setA-12 | 400 | 200 | 898 | 2 | 179 | 18.4086 | 0.7600 | ✓ | Good | 179134 | 10 | 17913 | EvaluationLimited | TimeBudget |
| 13 | setA-13 | 2000 | 200 | 1000 | 2 | 300 | 125.0976 | 0.9573 | ✓ | Poor | 308621 | 3 | 102873 | EvaluationLimited | TimeBudget |
| 14 | setA-14 | 600 | 250 | 1108 | 2 | 300 | 88.5745 | 0.5595 | ✓ | Weak | 309630 | 10 | 30963 | EvaluationLimited | TimeBudget |
| 15 | setA-15 | 600 | 250 | 1250 | 2 | 300 | 240.8649 | 0.8620 | ✓ | Poor | 320860 | 11 | 29169 | EvaluationLimited | TimeBudget |
| 16 | setA-16 | 4800 | 250 | 1452 | 2 | 300 | ∞ | 1.2275 | ✗ | Invalid | 365567 | 0 | — | Infeasible | TimeBudget |
| 17 | setA-17 | 2000 | 300 | 1270 | 2 | 300 | 60.6930 | 0.3948 | ✓ | Weak | 350061 | 3 | 116687 | EvaluationLimited | TimeBudget |
| 18 | setA-18 | 2000 | 300 | 1500 | 2 | 300 | 799260.9785 | 0.8399 | ✓ | Poor | 313839 | 3 | 104613 | EvaluationLimited | TimeBudget |
| 19 | setA-19 | 6000 | 300 | 1998 | 2 | 300 | ∞ | 1.1557 | ✗ | Invalid | 378406 | 0 | — | Infeasible | TimeBudget |
| 20 | setA-20 | 6000 | 400 | 2000 | 2 | 300 | ∞ | 1.1457 | ✗ | Invalid | 539734 | 0 | — | Infeasible | TimeBudget |

## Platform Validation Criteria

| Criterion | Status |
|-----------|--------|
| EvolutionEngine used unchanged | ✓ PASS |
| All instances load | ✓ PASS |
| Engine runs end-to-end | ✓ PASS |
| Zero modifications to coralys-moga | ✓ PASS |
| Zero modifications to Qualification Subsystem v1.0 | ✓ PASS |
