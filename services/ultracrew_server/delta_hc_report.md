# ΔHC Offspring Probe Report — SD-007 Root Cause Isolation

**Sprint:** 3.10  
**Seed:** 61  
**Instance:** n050w4  
**Generations:** 5000  
**Probe:** ΔHC = child_hc − parent_hc, measured BEFORE archive.add()  
**Scope:** All evaluated offspring (not just archive-admitted ones)  

---

## 1. Raw Offspring Counts

| Category | Count | % of Total |
|---|---|---|
| Total offspring evaluated | 5000 | 100.0% |
| HC-improving (child_hc < parent_hc) | 1438 | 28.76% |
| HC-neutral (child_hc == parent_hc) | 976 | 19.52% |
| HC-worsening (child_hc > parent_hc) | 2586 | 51.72% |

**Archive admission breakdown:**

| Category | Count |
|---|---|
| HC-improving AND admitted | 333 |
| HC-worsening AND admitted | 664 |

## 2. Key Probabilities

| Probability | Value | Interpretation |
|---|---|---|
| P(child_hc < parent_hc) | 0.287600 (28.7600%) | Probability mutation reduces HC_Total |
| P(admitted \| improving) | 0.231572 (23.1572%) | Of HC-improving offspring, fraction admitted |
| Mean ΔHC per offspring | 600.2000 | Positive = operator drifts away from feasibility |

## 3. Root Cause Classification

**Frozen classification table (sd007_resolution.md):**

| Condition | Classification |
|---|---|
| P(delta_hc < 0) ≈ 0 | RC-1 CONFIRMED (operator incapacity) |
| P(delta_hc < 0) > 0.1 AND P(improving AND inserted) ≈ 0 | RC-2 CONFIRMED (selection suppression) |
| Both probabilities > 0 | RC-1 + RC-2 interaction |

**Observed:** P(improving) = 0.287600, P(admitted | improving) = 0.231572  

**RC-1 + RC-2 INTERACTION** — Both P(improving) and P(improving AND inserted) are non-trivial. Operator has some capacity but selection further suppresses it.

## 4. Mean ΔHC Interpretation

Mean ΔHC = 600.2000 (penalty-weighted units; divide by 1000 for actual violation count delta)  

Mean ΔHC is **positive** (600.2000): on average, each mutation step moves the offspring **away** from feasibility. The search is diverging from the feasibility boundary. This is consistent with RC-1 (operator incapacity) and/or RC-2 (selection pressure favouring HC-worsening offspring that improve other proxy objectives).

