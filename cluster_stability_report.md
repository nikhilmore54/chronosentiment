# Ecological Structure Validation Report

This report presents evidence that the Q1 session‑metric space contains non‑random cluster structure. No regime names or interpretations are provided at this stage.

## Summary Table
| k | Silhouette | Silhouette p | Bootstrap ARI (mean±std) | Perturbation ARI σ=0.02 |
|---|------------|--------------|--------------------------|------------------------|
| 2 | 0.357 | 0.032 | 0.563±0.234 | 0.455 |
| 3 | 0.185 | 0.097 | 0.337±0.100 | 0.290 |
| 4 | 0.163 | 0.258 | 0.367±0.098 | 0.501 |
| 5 | 0.184 | 0.065 | 0.332±0.095 | 0.464 |
| 6 | 0.201 | 0.065 | 0.427±0.108 | 0.575 |
| 7 | 0.212 | 0.032 | 0.400±0.078 | 0.449 |
| 8 | 0.219 | 0.032 | 0.421±0.094 | 0.465 |
| 9 | 0.230 | 0.032 | 0.490±0.107 | 0.475 |
| 10 | 0.233 | 0.032 | 0.460±0.101 | 0.489 |

## Detailed Evidence per k
### k = 2

**Real‑data metrics**

- silhouette: 0.3571

- db: 1.1094

- ch: 64.3845

**Bootstrap stability (ARI)**

- Mean ARI: 0.5633
- Std ARI: 0.2337

**Perturbation robustness (ARI vs. noisy data)**

- σ=0.005: ARI = 0.9644

- σ=0.01: ARI = 0.6885

- σ=0.02: ARI = 0.4552

- σ=0.05: ARI = 0.3161

**Null‑model comparison (empirical p‑values)**

- silhouette p‑value: 0.0323

- db p‑value: 1.0000

- ch p‑value: 0.0323

---

### k = 3

**Real‑data metrics**

- silhouette: 0.1847

- db: 1.6019

- ch: 46.4015

**Bootstrap stability (ARI)**

- Mean ARI: 0.3372
- Std ARI: 0.1000

**Perturbation robustness (ARI vs. noisy data)**

- σ=0.005: ARI = 0.2804

- σ=0.01: ARI = 0.2710

- σ=0.02: ARI = 0.2895

- σ=0.05: ARI = 0.3842

**Null‑model comparison (empirical p‑values)**

- silhouette p‑value: 0.0968

- db p‑value: 0.9677

- ch p‑value: 0.0323

---

### k = 4

**Real‑data metrics**

- silhouette: 0.1635

- db: 1.4186

- ch: 41.5714

**Bootstrap stability (ARI)**

- Mean ARI: 0.3667
- Std ARI: 0.0985

**Perturbation robustness (ARI vs. noisy data)**

- σ=0.005: ARI = 0.3797

- σ=0.01: ARI = 0.3258

- σ=0.02: ARI = 0.5007

- σ=0.05: ARI = 0.3445

**Null‑model comparison (empirical p‑values)**

- silhouette p‑value: 0.2581

- db p‑value: 1.0000

- ch p‑value: 0.0323

---

### k = 5

**Real‑data metrics**

- silhouette: 0.1841

- db: 1.2149

- ch: 39.9849

**Bootstrap stability (ARI)**

- Mean ARI: 0.3322
- Std ARI: 0.0946

**Perturbation robustness (ARI vs. noisy data)**

- σ=0.005: ARI = 0.4468

- σ=0.01: ARI = 0.2961

- σ=0.02: ARI = 0.4637

- σ=0.05: ARI = 0.3051

**Null‑model comparison (empirical p‑values)**

- silhouette p‑value: 0.0645

- db p‑value: 1.0000

- ch p‑value: 0.0323

---

### k = 6

**Real‑data metrics**

- silhouette: 0.2007

- db: 1.1833

- ch: 40.8246

**Bootstrap stability (ARI)**

- Mean ARI: 0.4265
- Std ARI: 0.1085

**Perturbation robustness (ARI vs. noisy data)**

- σ=0.005: ARI = 0.6878

- σ=0.01: ARI = 0.4643

- σ=0.02: ARI = 0.5746

- σ=0.05: ARI = 0.4698

**Null‑model comparison (empirical p‑values)**

- silhouette p‑value: 0.0645

- db p‑value: 1.0000

- ch p‑value: 0.0323

---

### k = 7

**Real‑data metrics**

- silhouette: 0.2120

- db: 1.1544

- ch: 40.3692

**Bootstrap stability (ARI)**

- Mean ARI: 0.3996
- Std ARI: 0.0777

**Perturbation robustness (ARI vs. noisy data)**

- σ=0.005: ARI = 0.7239

- σ=0.01: ARI = 0.5628

- σ=0.02: ARI = 0.4492

- σ=0.05: ARI = 0.5090

**Null‑model comparison (empirical p‑values)**

- silhouette p‑value: 0.0323

- db p‑value: 1.0000

- ch p‑value: 0.0323

---

### k = 8

**Real‑data metrics**

- silhouette: 0.2187

- db: 1.1819

- ch: 41.2594

**Bootstrap stability (ARI)**

- Mean ARI: 0.4214
- Std ARI: 0.0936

**Perturbation robustness (ARI vs. noisy data)**

- σ=0.005: ARI = 0.8951

- σ=0.01: ARI = 0.6216

- σ=0.02: ARI = 0.4649

- σ=0.05: ARI = 0.5090

**Null‑model comparison (empirical p‑values)**

- silhouette p‑value: 0.0323

- db p‑value: 1.0000

- ch p‑value: 0.0323

---

### k = 9

**Real‑data metrics**

- silhouette: 0.2301

- db: 1.1340

- ch: 40.4653

**Bootstrap stability (ARI)**

- Mean ARI: 0.4897
- Std ARI: 0.1066

**Perturbation robustness (ARI vs. noisy data)**

- σ=0.005: ARI = 0.8314

- σ=0.01: ARI = 0.4756

- σ=0.02: ARI = 0.4748

- σ=0.05: ARI = 0.5992

**Null‑model comparison (empirical p‑values)**

- silhouette p‑value: 0.0323

- db p‑value: 1.0000

- ch p‑value: 0.0323

---

### k = 10

**Real‑data metrics**

- silhouette: 0.2334

- db: 1.0475

- ch: 39.9966

**Bootstrap stability (ARI)**

- Mean ARI: 0.4597
- Std ARI: 0.1008

**Perturbation robustness (ARI vs. noisy data)**

- σ=0.005: ARI = 0.4537

- σ=0.01: ARI = 0.6446

- σ=0.02: ARI = 0.4889

- σ=0.05: ARI = 0.4281

**Null‑model comparison (empirical p‑values)**

- silhouette p‑value: 0.0323

- db p‑value: 1.0000

- ch p‑value: 0.0323

---
