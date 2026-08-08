# RP-411 / RP-412 Baseline Analysis Report

**Telemetry directory:** `/tmp/rp411_baseline`
**Total generation records:** 285
**Instances:** 20

## 1. Aggregate Phase Timing (RP-411)

| Phase | Total ms | Fraction |
|-------|----------|----------|
| Evaluation | 3,463,372.2 | 99.99% |
| Crossover | 218.7 | 0.01% |
| Mutation | 16.6 | 0.00% |
| Selection | 80.1 | 0.00% |
| Telemetry | 0.0 | 0.00% |
| Other | 184.1 | 0.01% |
| Total | 3,463,871.8 | 100.00% |

## 2. Per-Instance Timing (RP-411)

| Instance | Gens | Pop | Total Evals | Eval ms | Gens/s | Evals/s | Final Stagnation |
|----------|------|-----|-------------|---------|--------|---------|-----------------|
| setA-01 | 51 | 50 | 2,550 | 8,495.4 | 5.99 | 299.5 | 20 |
| setA-02 | 21 | 50 | 1,050 | 1,680.9 | 12.44 | 622.0 | 20 |
| setA-03 | 61 | 50 | 3,050 | 16,180.7 | 3.76 | 188.1 | 20 |
| setA-04 | 14 | 50 | 700 | 29,831.0 | 0.47 | 23.5 | 1 |
| setA-05 | 11 | 50 | 550 | 28,562.4 | 0.39 | 19.3 | 1 |
| setA-06 | 13 | 50 | 650 | 132,973.4 | 0.10 | 4.9 | 0 |
| setA-07 | 21 | 50 | 1,050 | 75,948.3 | 0.28 | 13.8 | 20 |
| setA-08 | 9 | 50 | 450 | 69,910.3 | 0.13 | 6.4 | 0 |
| setA-09 | 10 | 50 | 500 | 74,464.1 | 0.13 | 6.7 | 0 |
| setA-10 | 10 | 50 | 500 | 320,432.6 | 0.03 | 1.6 | 0 |
| setA-11 | 9 | 50 | 450 | 194,826.7 | 0.05 | 2.3 | 1 |
| setA-12 | 10 | 50 | 500 | 187,005.7 | 0.05 | 2.7 | 0 |
| setA-13 | 7 | 50 | 350 | 307,878.9 | 0.02 | 1.1 | 0 |
| setA-14 | 9 | 50 | 450 | 328,244.3 | 0.03 | 1.4 | 0 |
| setA-15 | 9 | 50 | 450 | 321,395.1 | 0.03 | 1.4 | 0 |
| setA-16 | 5 | 50 | 250 | 253,204.2 | 0.02 | 1.0 | 4 |
| setA-17 | 2 | 50 | 100 | 257,983.1 | 0.01 | 0.4 | 0 |
| setA-18 | 8 | 50 | 400 | 387,687.0 | 0.02 | 1.0 | 2 |
| setA-19 | 3 | 50 | 150 | 236,230.4 | 0.01 | 0.6 | 2 |
| setA-20 | 2 | 50 | 100 | 230,437.7 | 0.01 | 0.4 | 1 |

## 3. Stagnation Profile (RP-411)

- Instances: 20
- Mean final stagnation: 4.6
- Median final stagnation: 1.0
- Max final stagnation: 20
- Mean final generation: 13.2
- Median final generation: 8.5
- Max final generation: 60
- Terminated by NoImprovement (stagnation ≥ 20): 4
- Terminated by GenerationLimit (gen ≥ 199): 0

## 4. Construction Diagnostics Summary (RP-412)

- Instances: 20
- Mean IFR: 10.60%
- Median IFR: 7.00%
- Min IFR: 0.00%
- Max IFR: 72.00%
- StdDev IFR: 16.25%
- Instances with any_feasible=true: 14
- Instances with IFR=100%: 0
- Mean capacity_violation_count: 44.7
- Total capacity violations: 894

## 5. Per-Instance Construction Diagnostics (RP-412)

| Instance | Pop | Valid | Invalid | IFR | Any Feasible | Cap Violations |
|----------|-----|-------|---------|-----|--------------|----------------|
| setA-01 | 50 | 9 | 41 | 18.0% | True | 41 |
| setA-02 | 50 | 0 | 50 | 0.0% | False | 50 |
| setA-03 | 50 | 3 | 47 | 6.0% | True | 47 |
| setA-04 | 50 | 5 | 45 | 10.0% | True | 45 |
| setA-05 | 50 | 36 | 14 | 72.0% | True | 14 |
| setA-06 | 50 | 2 | 48 | 4.0% | True | 48 |
| setA-07 | 50 | 0 | 50 | 0.0% | False | 50 |
| setA-08 | 50 | 4 | 46 | 8.0% | True | 46 |
| setA-09 | 50 | 7 | 43 | 14.0% | True | 43 |
| setA-10 | 50 | 4 | 46 | 8.0% | True | 46 |
| setA-11 | 50 | 12 | 38 | 24.0% | True | 38 |
| setA-12 | 50 | 1 | 49 | 2.0% | True | 49 |
| setA-13 | 50 | 1 | 49 | 2.0% | True | 49 |
| setA-14 | 50 | 5 | 45 | 10.0% | True | 45 |
| setA-15 | 50 | 7 | 43 | 14.0% | True | 43 |
| setA-16 | 50 | 0 | 50 | 0.0% | False | 50 |
| setA-17 | 50 | 10 | 40 | 20.0% | True | 40 |
| setA-18 | 50 | 0 | 50 | 0.0% | False | 50 |
| setA-19 | 50 | 0 | 50 | 0.0% | False | 50 |
| setA-20 | 50 | 0 | 50 | 0.0% | False | 50 |
