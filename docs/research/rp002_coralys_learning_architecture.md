# RP-002 — Coralys Learning Architecture (CLA)

**Protocol ID:** `RP-002`  
**Working name:** Coralys Learning Architecture  
**Disposition:** **REGISTERED · RESEARCH PROGRAMME · AUTHORITY 0%**  
**Date registered:** 2026-11-16  
**Live / production authority:** **None** — research and architecture only  
**Owner:** Platform / Engineering  

---

## 0. Governing intent

RP-002 investigates how Coralys can **acquire, evaluate, preserve and reuse optimisation knowledge** across multiple objectives and problem instances.

Operator selection (“which optimisation operator to select next?”) is the **first experimental application** of that broader research problem—not the framing question itself.

Architecture baseline (capability model, CLE vs Learning Intelligence, MOGA boundary):  
[`docs/CORALYS_LEARNING_INTELLIGENCE_ARCHITECTURE_v1.md`](../CORALYS_LEARNING_INTELLIGENCE_ARCHITECTURE_v1.md).

**Research Reading Register (initial freeze):**  
[`docs/research/rp002/RP002_RESEARCH_READING_REGISTER.md`](rp002/RP002_RESEARCH_READING_REGISTER.md) — literature organised by research gaps; library under [`docs/research/rp002/`](rp002/).  
**Architecture remains unfrozen** — evidence from the literature review may revise architectural assumptions.

**Hard separations:**

| Boundary | Treatment |
| -------- | --------- |
| Learning Intelligence vs Optimization (`coralys-moga`) | ML training is **not** embedded in the MOGA core; GA/MOGA remain Optimization |
| CLE vs Learning Intelligence | CLE = knowledge lifecycle; Learning Intelligence = statistical / policy model infra |
| Decision Intelligence MVP | Learning may inform alternatives; does **not** allocate capital or bypass user-controlled execution |
| ChronoSentiment / RP-608 Gate 5, OF ±0.30, OFX authority | **Domain governance freezes** — unchanged by RP-002 alone |
| Production cutover / live orders | **Not authorized** by this protocol |

---

## 1. Central research question

> **How can Coralys acquire, evaluate, preserve and reuse optimisation knowledge across multiple objectives and problem instances?**

First experimental slice: adaptive **optimisation-controller** learning (e.g. operator / strategy selection) under **multidimensional performance feedback**, such that specialised strategies are not extinguished by scalar “best immediate improvement” selection.

---

## 2. Research Foundation — Evolutionary Learning and Multidimensional Feedback

Both uploaded papers are **foundational references** for RP-002. Their value differs: one grounds evolutionary learning of knowledge structures; the other supplies a concrete mechanism for learning from multiple performance dimensions.

### Foundational reference 01 — Grefenstette (1993)

**Citation:** John J. Grefenstette, *Genetic Algorithms and Machine Learning*, ACM COLT ’93 (survey abstract / position paper).  
**Local artifact:** uploaded PDF (`168304.168305.pdf`).

**What it establishes**

| Principle | Content |
| --------- | ------- |
| Knowledge structures | Genetic algorithms can represent candidate knowledge structures |
| Performance selection | Selection can favour structures based on observed performance |
| Genetic improvement | Genetic operators can improve learned strategies |
| Sequential decisions | GAs can learn sequential decision rules |
| Simulation evaluation | Simulation can be used to evaluate learned strategies |
| Non-stationary adaptation | Genetic learning can adapt to changing environments |

**Particularly relevant:** the **SAMUEL** system, where the chromosome encodes a set of **condition–action rules** for controlling autonomous vehicles/robots, and fitness is measured by evaluating the resulting control strategy on a **simulator** (multi-agent / subtle opponent regularities; interest in co-learning opponents and changing environments).

Grefenstette also lists **open research questions** still useful as CLA prompts (modern literature must be consulted before claiming SOTA answers): convergence characterisation; exploration vs exploitation; noise tolerance in fitness; tracking non-stationary environments; role of mating restrictions; population size / generations / evaluation-accuracy trade-offs; when recombination helps vs mutation alone.

### Foundational reference 02 — Schaffer & Grefenstette

**Citation:** J. David Schaffer and John J. Grefenstette, *Multi-Objective Learning via Genetic Algorithms* (LS-2 / vector-evaluated GA).  
**Local artifact:** uploaded PDF (`Multi-Objective_Learning_via_Genetic_Algorithms…`).

**What it establishes**

| Principle | Content |
| --------- | ------- |
| Scalar fitness hazard | Scalar fitness can eliminate specialised knowledge |
| Multidimensional feedback | Learning systems can use multidimensional performance feedback |
| Specialisation | Different knowledge structures can specialise in different objectives |
| Complementary preservation | Selection can be modified to preserve complementary knowledge |
| Critic design | Design of the performance evaluator (critic) is critical |

**Central mechanism (study carefully):**

1. Candidate knowledge structures receive a **fitness vector**, not a single scalar.
2. Separate performance measures cover different aspects of learning (e.g. objectives A, B, C).
3. **Selection phases** run per vector component (Selection A / B / C), rather than only on one overall score.
4. Result: **preservation of complementary knowledge** that a scalar critic would extinguish.

**Illustrative failure mode:** structure P1 strong on A and B; structure P2 specialist on C. Scalar selection may drive P2 to extinction even though the system needs C’s specialised knowledge. Vector-valued selection with per-component phases retains complementary specialists.

**Demonstration context:** multiclass pattern-discrimination / five-class classification where unmodified scalar-critic GA failed; vector-evaluated approach succeeded (paper pp. 1–3).

**Qualification (mandatory):** the paper demonstrates vector-valued learning feedback on a **classification** task. Applying its mechanism to **adaptive optimisation controllers** is a **Coralys research extension**, not a result claimed by the authors.

### Foundation → Coralys implication table

| Research principle | Source | Coralys research implication |
| ------------------ | ------ | ---------------------------- |
| Evolutionary learning | Grefenstette (1993) | Learning through populations and performance feedback |
| Simulation-based evaluation | Grefenstette (1993) | Evaluate learned strategies through optimisation experiments / simulators |
| Exploration and exploitation | Grefenstette (1993) | Adaptive search control |
| Multidimensional feedback | Schaffer & Grefenstette | Vector-valued learning feedback at the learning-controller level |
| Complementary knowledge preservation | Schaffer & Grefenstette | Retain specialised optimisation strategies |
| Critic design | Schaffer & Grefenstette | Research multi-objective credit assignment |

**Differentiator hypothesis:** Coralys Learning Architecture should investigate learning that **preserves multiple forms of useful optimisation knowledge**, rather than chasing a single performance score. Schaffer & Grefenstette’s multidimensional feedback mechanism is a **central subject of RP-002**, not merely a literature-review citation.

---

## 3. Why multidimensional feedback matters to Coralys

At the **learning-controller** level, different optimisation strategies may specialise in, for example:

- Improving solution quality
- Maintaining population diversity
- Restoring feasibility
- Accelerating convergence
- Reducing computational cost

A learning mechanism that only rewards the **best immediate improvement** may eliminate strategies that contribute elsewhere. Coralys should investigate how to **preserve and exploit complementary optimisation knowledge**.

---

## 4. Coverage map — what the papers do and do not answer

| Research requirement for CLA | Covered by foundational papers? |
| ---------------------------- | ------------------------------- |
| Evolutionary learning foundations | Yes |
| Learning strategies from simulation | Yes |
| Multidimensional performance feedback | Yes |
| Preserving specialised knowledge | Yes |
| Adaptive operator selection | Partially |
| Contextual bandits | No |
| Reinforcement learning architecture | Partially, conceptually |
| Cross-instance knowledge transfer | No |
| Meta-learning | No |
| Continual learning architecture | No |
| Modern multi-objective policy learning | No |

Gaps above are **in-scope research topics** for later RP-002 phases; they are not solved by registering this protocol.

---

## 5. Three foundational research pillars

### Pillar 1 — Knowledge Acquisition

How does Coralys learn from optimisation experiences?

- Evolutionary learning (population + performance feedback)
- Bandits / contextual bandits (future literature)
- Reinforcement learning (conceptual overlap; modern architecture TBD)

### Pillar 2 — Knowledge Preservation

How does Coralys avoid losing useful specialised strategies?

- Multidimensional / vector-valued feedback
- Pareto-aware or per-objective selection phases
- Diversity preservation and critic design

**Priority subject:** Schaffer & Grefenstette mechanism → adaptive optimisation-controller extension.

### Pillar 3 — Knowledge Transfer

How does Coralys apply previously acquired knowledge to new problems?

- Meta-learning
- Problem characterisation
- Continual adaptation across instances / non-stationary environments

---

## 6. Placement relative to platform capabilities

```text
Observe outcomes  →  Learn (RP-002 / Learning Intelligence)
                         ↓ predictions, features, controller policies
                   Optimize (MOGA / coralys-moga)   ← fitness vectors, scenarios
                         ↓ Pareto / alternative sets
                   Decide (Decision Intelligence)
                         ↓
                   Execute → feedback
```

- **RP-002 research** may propose learning-controller policies and multidimensional critics that **consume** optimisation run telemetry.
- **MOGA core** continues to own evolutionary operators, population dynamics, and (domain) fitness evaluation contracts.
- Learned controller outputs do **not** bypass Decision governance or domain freezes.

---

## 7. Non-goals (this registration)

- Embedding ML training loops inside `coralys-moga`
- Changing ChronoSentiment Estimand A / OF ±0.30 / Gate 5 / OFX authority
- Claiming production readiness or unsupervised “authority” from learned models
- Treating the classification-task vector-GA result as already proven for optimisation controllers
- Closing Grefenstette’s open theoretical questions without modern literature review

---

## 8. Immediate research posture

| Item | Status |
| ---- | ------ |
| Architecture baseline (ARCH-001-LI) | Draft baseline present — **not frozen by RP-002** |
| Foundational refs 01 & 02 | **Accepted into RP-002 foundation** |
| Research Reading Register | **Initial freeze v0.1** — AOS / MORL / bandits / meta / continual / eval library |
| Multidimensional feedback as central subject | **Yes** |
| Build-on vs differentiate | Build on AOS + bandits + MORL; investigate MO knowledge preservation + cross-instance + continual combo |
| First implementation-oriented papers | FRRMAB (Li 2014); Roijers (2013); Tornede (2023) |
| Batch 1 study | **In progress** — Li/DaCosta/Roijers full text studied; Lin/Jiao access blocked; notes in `rp002/07_…/BATCH1_WITHIN_RUN_LEARNING_NOTES.md` |
| Operator selection as first experiment | **Authorized as framing for experiment design only** — no implementation mandated by this doc |
| Experiment protocol / harness / metrics | Draft E0 (FRRMAB reproduce) sketched in Batch 1 notes — **not authorised to run** |
| Authority | **0%** |

---

## Change log

| Version | Date | Notes |
| ------- | ---- | ----- |
| 0.1 | 2026-11-16 | Programme registered; foundational refs Grefenstette (1993) + Schaffer & Grefenstette; three pillars; multidimensional feedback central; operator selection demoted to first experimental application |
| 0.2 | 2026-11-16 | Linked RP-002 Research Reading Register (initial freeze); library folders 01–07; architecture explicitly unfrozen |
| 0.3 | 2026-11-16 | Batch 1 within-run notes (FRRMAB / D-MAB / Roijers); provisional E0 design only; architecture still unfrozen |

---

*RP-002 Coralys Learning Architecture | November 2026 | Status: Registered research programme · Authority 0%*  
*Review trigger: first experiment protocol; material change to Learning/Optimization contracts; literature update on open GA questions.*
