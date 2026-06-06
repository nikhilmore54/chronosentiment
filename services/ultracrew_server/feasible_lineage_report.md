# Feasible Lineage Report — SD-005 Causal Dependency Investigation

**Sprint:** 3.8  
**Seed:** 61  
**Instance:** n050w4  
**Observer:** `inrc_official_total`  
**Generations:** 5000  
**Total feasible genomes discovered:** 0  

---

## 1. First Feasible Genome

**No feasible genome was ever discovered.**

Classification: **Discovery Failure** — the evaluator never returned `feasible=true` in 5000 generations.

## 2. Best Feasible Genome

No feasible genome discovered — see Section 1.

## 3. Feasibility Census Timeline

Sampled every 100 generations. `near_feasible_5` = HC_Total ≤ 5; `near_feasible_10` = HC_Total ≤ 10.

| Generation | feasible_count | near_feasible_5 | near_feasible_10 | infeasible_count |
|---|---|---|---|---|
| 100 | 0 | 0 | 0 | 31 |
| 200 | 0 | 0 | 0 | 37 |
| 300 | 0 | 0 | 0 | 45 |
| 400 | 0 | 0 | 0 | 61 |
| 500 | 0 | 0 | 0 | 50 |
| 600 | 0 | 0 | 0 | 56 |
| 700 | 0 | 0 | 0 | 65 |
| 800 | 0 | 0 | 0 | 58 |
| 900 | 0 | 0 | 0 | 64 |
| 1000 | 0 | 0 | 0 | 67 |
| 1100 | 0 | 0 | 0 | 80 |
| 1200 | 0 | 0 | 0 | 90 |
| 1300 | 0 | 0 | 0 | 93 |
| 1400 | 0 | 0 | 0 | 102 |
| 1500 | 0 | 0 | 0 | 113 |
| 1600 | 0 | 0 | 0 | 117 |
| 1700 | 0 | 0 | 0 | 132 |
| 1800 | 0 | 0 | 0 | 110 |
| 1900 | 0 | 0 | 0 | 118 |
| 2000 | 0 | 0 | 0 | 114 |
| 2100 | 0 | 0 | 0 | 111 |
| 2200 | 0 | 0 | 0 | 117 |
| 2300 | 0 | 0 | 0 | 123 |
| 2400 | 0 | 0 | 0 | 132 |
| 2500 | 0 | 0 | 0 | 136 |
| 2600 | 0 | 0 | 0 | 132 |
| 2700 | 0 | 0 | 0 | 135 |
| 2800 | 0 | 0 | 0 | 133 |
| 2900 | 0 | 0 | 0 | 137 |
| 3000 | 0 | 0 | 0 | 130 |
| 3100 | 0 | 0 | 0 | 138 |
| 3200 | 0 | 0 | 0 | 142 |
| 3300 | 0 | 0 | 0 | 137 |
| 3400 | 0 | 0 | 0 | 145 |
| 3500 | 0 | 0 | 0 | 146 |
| 3600 | 0 | 0 | 0 | 151 |
| 3700 | 0 | 0 | 0 | 153 |
| 3800 | 0 | 0 | 0 | 150 |
| 3900 | 0 | 0 | 0 | 154 |
| 4000 | 0 | 0 | 0 | 162 |
| 4100 | 0 | 0 | 0 | 166 |
| 4200 | 0 | 0 | 0 | 156 |
| 4300 | 0 | 0 | 0 | 151 |
| 4400 | 0 | 0 | 0 | 147 |
| 4500 | 0 | 0 | 0 | 155 |
| 4600 | 0 | 0 | 0 | 169 |
| 4700 | 0 | 0 | 0 | 160 |
| 4800 | 0 | 0 | 0 | 152 |
| 4900 | 0 | 0 | 0 | 150 |
| 5000 | 0 | 0 | 0 | 156 |

## 4. SD-005 Classification

Applying frozen classification table from `sd005_sprint38_charter.md`.

| Metric | Count |
|---|---|
| Total feasible genomes discovered | 0 |
| Admitted to archive | 0 |
| Evicted from archive | 0 |
| Evicted by Dominated | 0 |
| Still in archive at gen 5000 | 0 |

### Classification

**Discovery Failure** — No feasible genome was ever produced by the evaluator. The O3 mechanism is not implicated; the root cause is upstream of the archive (evaluator landscape, mutation operators, or constraint structure).

