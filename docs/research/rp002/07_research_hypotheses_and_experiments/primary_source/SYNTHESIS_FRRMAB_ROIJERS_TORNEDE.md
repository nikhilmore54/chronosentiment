# Comparative Synthesis — FRRMAB · Roijers · Tornede

**Milestone:** Primary-source comparative synthesis for RP-002 / CLA  
**Date:** 2026-11-16  
**Inputs:** [PS-01 Li 2014](PS01_LI_2014_FRRMAB.md) · [PS-02 Roijers 2013](PS02_ROIJERS_2013_MO_SEQUENTIAL.md) · [PS-03 Tornede 2023](PS03_TORNEDE_2023_META_AS.md)  
**Freeze status:** Reading register frozen · **CLA architecture unfrozen** · Authority **0%**

**Rule:** Distinguish **demonstrated** (papers) from **proposed** (Coralys). Nothing here freezes architecture.

---

## 1. What each paper is actually about

| Paper | Demonstrated domain | Decision level | Feedback type | Time horizon |
| ----- | ------------------- | -------------- | ------------- | ------------ |
| **Li FRRMAB** | MOEA/D + DE operators on UF benchmarks | Choose **operator** each generation | Scalar FIR → FRR | **Within-run** |
| **Roijers** | MOMDP planning/learning survey | Choose **policy** or **coverage set** | Vector returns; late scalarisation | Sequential / lifelong preference |
| **Tornede** | ASLib runtime AS | Choose **algorithm** via **selectors/ensembles** | Offline \(m(i,a)\); PAR10 | **Across-instance** (offline train, online query) |

They are **not competing solutions to one problem**. They address three stacked layers of Coralys learning:

```text
Tornede  →  which learning/selection *policy* for this instance?
Roijers  →  how to represent multi-objective value without destroying specialists?
FRRMAB  →  which operator *now* inside this run?
```

---

## 2. Complementarity (where mechanisms fit together)

### 2.1 Vertical stack (compatible)

| Layer | Mechanism from literature | Coralys reading (proposal) |
| ----- | ------------------------- | -------------------------- |
| L0 Variation | FRRMAB arms + credit | Destroy/mutation pool + FIR |
| L1 Preference structure | Roijers taxonomy / coverage sets | Multi-criteria operator value; Decision selects among alternatives |
| L2 Instance transfer | Tornede ensembles of selectors | Ensemble of AOS policies / solvers across instances |

**Proposed (not demonstrated as a unit):** Coralys CLA as L2(L1(L0)) — Tornede-style aggregation over Roijers-aware multi-objective credits driving FRRMAB-like within-run AOS.

### 2.2 Shared themes

- **Complementarity of specialists:** FRRMAB (operator pool > single op); Roijers (coverage sets); Tornede (selector ensembles) — all warn against winner-take-all collapse.
- **Separation of selection vs credit/utility:** FRRMAB credit≠selection; Roijers \(f\)≠planning; Tornede ass≠agg.
- **Cost of adaptation:** FRRMAB CPU overhead; Tornede offline train vs online query; Roijers compute coverage once vs replan.

### 2.3 Empirical encouragement (demonstrated separately)

- FRRMAB: pool+AOS beats single DE ops and several MOEA/D baselines (UF suite).  
- Tornede: voting ensembles beat SBAS on many ASLib scenarios; query cost small.  
- Roijers: clarifies *when* scalar collapse is invalid — design constraint, not a benchmark win.

---

## 3. Conflicts and tensions

| Tension | FRRMAB | Roijers | Tornede | CLA implication (proposal) |
| ------- | ------ | ------- | ------- | -------------------------- |
| **Scalar credit** | Requires scalar FIR (via MOEA/D) | Warns scalarisation can erase structure | Uses scalar \(m\) (runtime) | E0 may use scalar FIR; RD-1 must not stop there |
| **Online vs offline** | Online bandit | Plan/learn then select | Offline fit, online query | Do not assume one learning loop |
| **Features** | None (window of FIR) | Preference / value vectors | Instance features often weak at meta level | Test feature value; Tornede suggests **predictions** > raw features for meta |
| **Solution object** | One adaptive policy | Often a **set** of policies | One algorithm (after aggregation) | Product must decide: one strategy vs menu |
| **Oracle gap** | Implicit via baselines | Coverage vs single optimum | Meta AS useless if SBS≈oracle | Measure Coralys oracle–baseline gap before L2 spend |
| **Domain** | Continuous MO UF | Sequential DM / MORL | SAT/CSP runtime AS | Transfer to ROADEF/UltraCrew is **our** experiment |

**Hard conflict if naively merged:** Using FRRMAB’s single scalar FIR as the *only* long-term learning signal contradicts Roijers/Schaffer preservation goals. Tornede does not resolve that — it ensembles scalar selectors.

---

## 4. What the papers do **not** jointly demonstrate

None of the three papers demonstrates:

- Multi-objective **operator** credit with per-component selection (Schaffer mechanism on AOS)  
- Coralys EvolutionEngine integration  
- ROADEF/UltraCrew/CVRP transfer  
- Continual learning across streams of optimisation runs without catastrophic forgetting  
- Authority / governance for Decision execution  

Claiming “CLA = FRRMAB + Roijers + Tornede” as an engineered product would be **premature**.

---

## 5. Evidence-based research sequence (still design-only)

Aligned with existing E0 protocol; refined by this synthesis:

| Stage | Question | Primary evidence base | Status |
| ----- | -------- | --------------------- | ------ |
| **E0** | Does FRRMAB-style within-run AOS beat RP-405 / fixed on ROADEF arms? | Li (+ DaCosta ablate) | Protocol drafted; **not authorised** |
| **E0b** | Does multi-criteria credit (diversity/feasibility) improve retention of complementary arms? | Roijers vocabulary + Lin abstract; Schaffer F-02 | After E0 |
| **E2** | Do AOS policies complement across instances; does voting beat SBAS? | Tornede | Needs Coralys “ASLib” store |
| **E3** | Preference / coverage-set presentation at Decision | Roijers scenarios | Product/Decision track |

---

## 6. Implications for CLA development (unfrozen hypotheses)

**H-CLA-1 (within-run):** Adaptive operator selection with normalised sliding-window credit is a necessary first capability. *Supported as prior art by Li; Coralys proof still required (E0).*

**H-CLA-2 (multi-objective structure):** Long-term CLA should not rely solely on scalar FIR; coverage / multi-criteria feedback is needed to preserve specialists. *Motivated by Roijers + F-02; not shown for EA operators.*

**H-CLA-3 (across-instance):** Cross-instance gain is more likely from **ensembling complementary policies** than from learning a brittle selector-of-selectors on instance features alone. *Supported on ASLib by Tornede; Coralys transfer unknown.*

**H-CLA-4 (preconditions):** Invest in L2 only if Coralys shows (i) oracle–baseline gap and (ii) policy complementarity. *Tornede precondition.*

These are **research hypotheses**, not frozen architecture.

---

## 7. Open questions for the next CLA design iteration

1. Single adaptive controller vs coverage set of specialised controllers?  
2. Where does scalarisation happen — Learning, Optimization, or Decision?  
3. What instance features (if any) beat “vote among recent AOS policies”?  
4. How to define FIR when objectives conflict or repair fails?  
5. What is the Coralys cost budget for within-run AOS overhead (~Li’s ~40% class)?  

---

## 8. Document control

| Item | Status |
| ---- | ------ |
| Research Reading Register v0.1 | Frozen (unchanged by this synthesis) |
| Primary-source analyses PS-01..03 | Complete |
| This synthesis | Complete — evidence for CLA development |
| CLA architecture | **Unfrozen** |
| E0 run | **Not authorised** |

---

*Synthesis v1.0 | 2026-11-16 | Demonstrated ≠ proposed · Authority 0%*
