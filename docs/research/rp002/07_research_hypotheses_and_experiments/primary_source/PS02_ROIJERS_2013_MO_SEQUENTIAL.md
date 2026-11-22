# Primary-Source Analysis — Roijers et al. (2013)

**Citation:** Diederik M. Roijers, Peter Vamplew, Shimon Whiteson, Richard Dazeley. *A Survey of Multi-Objective Sequential Decision-Making.* JAIR 48:67–113, 2013. DOI: 10.1613/jair.3987  

**Local PDF:** [`../../03_multi_objective_learning/papers/roijers2013_mo_sequential_decision_making.pdf`](../../03_multi_objective_learning/papers/roijers2013_mo_sequential_decision_making.pdf)  

**Analysis posture:** Survey of MOMDP planning/learning — not an optimisation AOS paper. Separate **survey claims** from **Coralys proposals**. Architecture unfrozen. Authority 0%.

---

## 1. Research problem and motivation

**Problem:** Sequential decision-making with **vector rewards** (MOMDPs), where literature often fails to say *when* multi-objective methods are actually required.

**Motivation:**

- Single-objective MDP theory (additive infinite-horizon) guarantees a deterministic stationary optimum (Theorem 1).
- Multiple objectives yield only a **partial order** on policies.
- Scalarising early can be impossible, infeasible, or undesirable — yet many methods silently assume a scalar reward.

---

## 2. Mathematical formulation (as in paper)

**MOMDP:** MDP with vector reward \(r_t\); policy value

\[
V_\pi=\mathbb{E}\!\left[\sum_{k=0}^{\infty}\gamma^k r_{k+1}\mid\pi\right]
\]

(vector of expected cumulative rewards).

**Scalarisation** \(f\): maps multi-objective value → scalar utility (Definition 1). Conversion to single-objective MDP = choose \(f\) (and weights \(w\) if applicable) + solve resulting MDP.

**Undominated set / coverage set** (Defs 2–3): solution concepts when one policy is insufficient.

**Linear scalarisation:** \(f_w(V)=w\cdot V\) → **convex hull / CCS** of deterministic stationary policies (Defs 4–6).

**Monotone scalarisation:** Pareto dominance on value vectors → **Pareto front / PCS** (Defs 7–10); may require non-stationary or stochastic policies (Theorems 2+).

**SER vs ESR:** scalarise expectation vs expect scalarisation — survey notes ESR largely neglected despite medical/single-episode relevance (§8.4).

---

## 3. Proposed “algorithm” and mechanisms

This is a **taxonomy + survey**, not a single new algorithm.

**Three motivating scenarios (§3)**

| Scenario | Scalarisation before plan/learn | Why |
| -------- | ------------------------------- | --- |
| Unknown weights | Impossible now; trivial at selection time | e.g. fluctuating prices |
| Decision support | Infeasible throughout | Hard/fuzzy \(w\) or \(f\); committees |
| Known weights | Possible but may be undesirable | Nonlinear \(f\) → hard non-additive MDP |

**Taxonomy factors (§4)**

1. Single policy vs coverage set  
2. Linear vs monotonically increasing scalarisation  
3. Deterministic stationary / non-stationary / stochastic policies  

**Surveyed mechanisms:** MOMDP planning (§5) and learning/MORL (§6) organised by taxonomy cells; applications (§7).

**Utility-based stance:** scalarisation is part of the *problem*; Pareto front is not always the required solution (Observation 1 in conclusions).

---

## 4. Assumptions and limitations

**Assumptions of the taxonomy**

- Utility-based framing (vs purely axiomatic Pareto).
- Infinite-horizon discounted models dominate the surveyed theory.
- Coverage sets are the right targets when preferences unknown/delayed.

**Limitations (survey-acknowledged)**

- Fragmented literature; many methods omit scenario assumptions.
- Model-based MORL underdeveloped (§8.1).
- Many-objective sequential DM still open (§8.3).
- ESR formulation almost absent (§8.4).
- Taxonomy does not cover every published method neatly.

---

## 5. Experimental methodology and findings

**Not an empirical paper.** Findings are conceptual:

- When scalarisation is valid early → often a **single** policy suffices (esp. linear).
- When not → optimal object is a **CCS** or **Pareto coverage set**.
- Computing full Pareto fronts is often unnecessary under utility-based goals.
- Planning vs learning literatures differ in emphasised cells of the taxonomy.

---

## 6. Relevance to Coralys

| Coralys concern | Relevance |
| --------------- | --------- |
| Multi-objective credit / RD-1 | **Foundational** — explains why collapsing operator feedback to one reward can erase specialists |
| Decision Intelligence | Aligns with **late** preference injection / coverage sets vs forced early weights |
| FRRMAB / E0 | Complementary but different layer: FRRMAB is within-run **scalar** credit on MOEA/D subproblems |
| Schaffer & Grefenstette F-02 | Same intellectual family: preserve complementary knowledge under multi-aspect feedback |
| Cross-instance AS (Tornede) | Orthogonal — Roijers is about preference/structure of MO sequential problems, not instance→solver maps |

---

## 7. Architectural implications (proposals — **not frozen**)

**Demonstrated by survey → design vocabulary for CLA**

- Name the Coralys controller problem cell: single adaptive policy vs CCS/PCS of specialised strategies.
- Prefer Decision-layer selection among alternatives when preferences are fuzzy/committee-like.
- Do not assume “Pareto front of everything” is the product requirement.

**Coralys proposals (extensions)**

- Treat operator strategies as policies in a meta-MDP with vector credits (quality, diversity, feasibility, cost) — **not demonstrated** by Roijers for EA operators.
- Map unknown-weights scenario to Coralys products that change preference weights over time.

**Do not freeze:** MOMDP controller architecture; choice of SER vs ESR; requirement to compute full Pareto sets.

---

## 8. Unresolved research questions (for CLA)

1. Which Roijers taxonomy cell describes Coralys learning-controller design for EvolutionEngine?  
2. Should Decision MVP expose a coverage set of optimiser strategies or a single recommendation?  
3. Can operator credit be formulated as MOMDP rewards without abandoning determinism of the sim path?  
4. Is SER or ESR the right scalarisation timing for single optimisation *run* vs *fleet* of runs?  
5. How does this interact with ChronoSentiment multi-scenario fitness aggregation (Optimization concern)?

---

*PS-02 v1.0 | 2026-11-16 | Primary source · Architecture unfrozen*
