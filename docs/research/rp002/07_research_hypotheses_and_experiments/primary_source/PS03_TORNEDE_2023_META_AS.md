# Primary-Source Analysis — Tornede et al. (2023)

**Citation:** Alexander Tornede, Lukas Gehring, Tanja Tornede, Marcel Wever, Eyke Hüllermeier. *Algorithm selection on a meta level.* Machine Learning 112:1253–1286, 2023 (OA). DOI: 10.1007/s10994-022-06161-4  

**Local PDF:** [`../../04_meta_learning_and_transfer/papers/tornede2023_algorithm_selection_meta_level.pdf`](../../04_meta_learning_and_transfer/papers/tornede2023_algorithm_selection_meta_level.pdf) (31 pp., LMU OA mirror; verified)  

**Code/data (paper):** [github.com/alexandertornede/as_on_a_meta_level](https://github.com/alexandertornede/as_on_a_meta_level) · ASLib  

**Analysis posture:** Separate **paper results** from **Coralys cross-instance proposals**. Architecture unfrozen. Authority 0%.

---

## 1. Research problem and motivation

**Classical AS (Rice):** given instance \(i\), choose algorithm \(a\in A\) to optimise performance \(m(i,a)\) (often runtime).

**Meta AS (this paper):** algorithm selectors themselves show complementarity → choose **one or more selectors** and aggregate to a single algorithm.

**Motivation:**

- No free lunch → per-instance AS beats single best solver (SBS).
- Selectors also complement each other; little prior work on **ensembling selectors**.
- Naive “selector-of-selectors” via meta-learning may be hard; ensembles may work better.

---

## 2. Mathematical formulation (as in paper)

**AS:** learn \(s: I\to A\) minimising expected \(m(i,s(i))\).

**Runtime loss:** PAR10 / normalised nPAR10 vs oracle and SBS (Eqs. 3–4); cutoff \(C\); timeouts cost \(10C\).

**Meta AS:** given selector set \(S\subseteq\{s:I\to A\}\), find

\[
\mathrm{ass}:I\to 2^{S},\qquad \mathrm{agg}:I\times 2^{S}\to A
\]

such that \(\mathrm{agg}(i,\mathrm{ass}(i))\) optimises \(m\) (Eqs. 5–7). Aggregation may be instance-specific.

**Restricted meta-learning form:** \(s_{\mathrm{meta}}:I\to S\) picking a single selector (Eqs. 8–9); aggregation = identity.

**AS-oracle** on meta level (Eq. 10) can be **weaker** than base oracle if no selector ever picks the true best algorithm.

---

## 3. Proposed algorithm and mechanisms

| Instantiation | Mechanism |
| ------------- | --------- |
| Meta-learning ASS | Train standard AS methods to map instance features → which *selector* to use |
| Voting | Query multiple selectors; aggregate (e.g. Borda / other rank aggregations); optimise ensemble composition |
| Bagging | Bootstrap instance sets; train heterogeneous base selectors |
| Boosting | Sequentially emphasise hard instances |
| Stacking | Meta-learner aggregates selector outputs (optionally + features) |

**Framework claim:** meta AS = (selector selection ∪ ensemble) + aggregation.

---

## 4. Assumptions and limitations

**Assumptions**

- Rich offline performance database (ASLib-style) with instance features.
- Primary target often **runtime** (PAR10); feature compute time counted.
- Base selectors already exist and are complementary.
- Training ensembles offline is acceptable; query-time cost should stay small.

**Limitations (paper)**

- Direct meta-learning ASS **fails to improve** in their study.
- Meta AS only worthwhile when (a) large gap to oracle and (b) selector complementarity; weak when SBS≈oracle.
- AS-oracle can degrade vs base oracle (reported degradations from <1% to >100%, extreme outlier ~900% on one scenario).
- Domain focus: SAT/CSP/ASLib runtime scenarios — not MOEA operator selection, not Coralys CVRP/ROADEF directly.
- Stacking/learning on meta level hard; **simple voting** often best.

---

## 5. Experimental methodology and findings

**Data:** ASLib scenarios (public).  
**Metric:** nPAR10 (0=oracle, 1=SBS).  
**Comparisons:** base selectors; SBAS; voting/bagging/boosting/stacking variants; composition search for voting.

**Findings (paper)**

- Ensembles of selectors can **significantly outperform** single selectors and approach new SOTA on many scenarios.
- Voting (esp. with composition optimisation / Borda) particularly strong; often best among ensembles.
- Learning which single selector to use (Eq. 8) yields **essentially no improvement**.
- Speculative explanation: instance features lack meta-discriminative info; **selector predictions** carry more signal (stacking feature analysis).
- Query-time cost of multiple selectors remains negligible vs solving; training cost higher but offline.
- On scenarios where SBS≈oracle, meta AS has little room.

---

## 6. Relevance to Coralys

| Coralys concern | Relevance |
| --------------- | --------- |
| Cross-instance / RD-2 | **Direct conceptual match** — retain experience across instances via selectors |
| Within-run FRRMAB | **Different level** — Tornede selects solvers/selectors offline-trained; FRRMAB adapts operators inside a run |
| RP-405 / E0 | Complementary later: multiple AOS policies (FRRMAB, D-MAB, fixed) as “selectors” ensembled per instance |
| UltraCrew / ROADEF / CVRP | Need Coralys instance features + performance tables analogous to ASLib — **not provided** by paper |
| Decision / governance | Aggregation of selectors resembles Decision choosing among strategy recommenders |

---

## 7. Architectural implications (proposals — **not frozen**)

**Demonstrated → CLA vocabulary**

- Distinguish **algorithm selection**, **meta-learning of selectors**, and **ensemble aggregation** (do not conflate).
- Prefer starting cross-instance work with **ensemble/voting of policies**, not deep meta-learners.
- Require oracle–SBS gap before investing in meta AS.

**Coralys proposals (extensions)**

- Treat {FixedUniform, FixedRandom, RP405Weights, FRRMAB, …} as base “selectors” over destroy arms; learn when to trust which **across instances**.
- Build an ASLib-like store of (instance features, policy, Δobj, time).
- Do **not** assume instance features alone suffice for meta selection without testing.

**Do not freeze:** meta-learner architecture; feature schema; requirement that voting is the production CLA design.

---

## 8. Unresolved research questions (for CLA)

1. What is Coralys’s analogue of ASLib for ROADEF/UltraCrew/CVRP?  
2. Do FRRMAB vs weight-bandit policies show **complementarity across instances** (Tornede precondition)?  
3. Will voting among AOS policies beat SBAS without instance-feature meta-learning?  
4. How large is the oracle–SBS gap on Coralys benchmarks?  
5. How does continual accumulation of runs interact with offline ensemble training (Batch 2/continual pillar)?

---

*PS-03 v1.0 | 2026-11-16 | Primary source verified OA · Architecture unfrozen*
