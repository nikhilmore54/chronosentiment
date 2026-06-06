# Rejection Attribution Report — SD-007 Sub-Hypothesis Isolation

**Sprint:** 3.11  
**Seed:** 61  
**Instance:** n050w4  
**Generations:** 5000  
**Probe:** For every HC-improving offspring (child_hc < parent_hc), record ΔHC magnitude and ΔO3 direction  
**Scope:** All evaluated offspring before archive.add()  

---

## 1. Raw Counts — HC-Improving Offspring

| Category | Count |
|---|---|
| Total HC-improving offspring | 1264 |
| HC-improving AND admitted | 256 |
| HC-improving AND rejected | 1008 |

**ΔHC magnitude distribution (rejected improving only):**

| Bucket | Count | % of rejected improving |
|---|---|---|
| |ΔHC| ≤ 10 (tiny improvement) | 0 | 0.00% |
| 10 < |ΔHC| ≤ 100 (small improvement) | 0 | 0.00% |
| 100 < |ΔHC| ≤ 1000 (medium improvement) | 518 | 51.39% |
| |ΔHC| > 1000 (large improvement) | 490 | 48.61% |

**ΔO3 direction (rejected improving only):**

| O3 Direction | Count | % of rejected improving |
|---|---|---|
| O3 worsened (child_o3 > parent_o3) | 520 | 51.59% |
| O3 improved (child_o3 < parent_o3) | 474 | 47.02% |
| O3 neutral (|Δo3| ≤ 0.5) | 14 | 1.39% |

## 2. Key Probabilities

| Probability | Value | Interpretation |
|---|---|---|
| P(O3 worsened \| rejected improving) | 0.5159 (51.59%) | Sub-B signal: O3 pressure gates HC-improving offspring |
| E[ΔHC \| improving AND rejected] | 1714.29 | Mean HC improvement magnitude for rejected offspring |
| E[ΔHC \| improving AND admitted] | 1699.22 | Mean HC improvement magnitude for admitted offspring |
| E[ΔHC admitted] / E[ΔHC rejected] | 0.99× | Sub-A signal: >2× means admitted improvements are much larger |

**Joint counts:**

| Joint Event | Count | % of rejected improving |
|---|---|---|
| HC-improving AND O3-worsening AND rejected (Sub-B) | 520 | 51.59% |
| HC-improving AND O3-improving AND rejected (Sub-A) | 474 | 47.02% |

## 3. Sub-A vs Sub-B Classification

**Sub-A (step-size asymmetry):** Admitted HC-improving offspring have significantly larger ΔHC than rejected ones.  
Signal: E[ΔHC | admitted] > 2× E[ΔHC | rejected]  

**Sub-B (O3 proxy pressure):** Majority of rejected HC-improving offspring also worsen O3.  
Signal: P(O3 worsened | rejected improving) > 0.5  

| Sub-hypothesis | Signal Threshold | Observed | Active? |
|---|---|---|---|
| Sub-A (step-size asymmetry) | E[ΔHC admitted] > 2× E[ΔHC rejected] | 0.99× | ✗ NO |
| Sub-B (O3 proxy pressure) | P(O3 worsened \| rejected) > 0.5 | 0.5159 | ✓ YES |

### Classification

**Sub-B DOMINANT (O3 proxy pressure)** — The majority of rejected HC-improving offspring also worsen O3. The archive preferentially rejects offspring that trade O3 for HC improvement. O3 acts as an attractor preventing HC accumulation.

## 4. Interpretation and Scientific Debt

### Context

From Sprint 3.10 (sd007_resolution.md, commit deac7d18):

- P(child_hc < parent_hc) = 28.76% — RC-1 Operator Incapacity FALSIFIED
- P(admit | improving) = 23.16% vs P(admit | worsening) = 25.68% — conditional rates close
- Mean ΔHC = +600.2 — operator has positive HC drift (RC-1 Operator Bias CONFIRMED)
- Defensible conclusion: HC-improving offspring exist but do not accumulate

### Sprint 3.11 Adds

This probe isolates the mechanism preventing accumulation:

- **Sub-B active:** 51.59% of rejected HC-improving offspring also worsened O3. The archive is not neutral with respect to the HC/O3 trade-off — it preferentially rejects offspring that improve HC at the cost of O3. This is consistent with the gen-69 O3 attractor event (+15,845 HC penalty traded for 330 O3 units).

- **Sub-A not dominant:** Admitted and rejected improvements are similar in magnitude (ratio=0.99×). Step-size asymmetry is not the primary gate.

### Remaining Scientific Debt

- E[ΔHC | improving AND worsening-O3 AND rejected] vs E[ΔHC | improving AND improving-O3 AND rejected] — joint magnitude not yet measured
- Which proxy objectives (O1, O2, O4) gate the remaining rejected improving offspring
- Whether the O3 attractor is a structural property of the n050w4 constraint landscape or an artifact of the proxy formulation
- Operator redesign: targeted HC-reduction moves (e.g. shift-swap within constraint families) to test RC-1 Operator Bias directly

