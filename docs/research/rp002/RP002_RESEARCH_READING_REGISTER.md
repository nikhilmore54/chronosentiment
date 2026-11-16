# RP-002 — Research Reading Register

**Document ID:** RP-002-RRR-001  
**Protocol:** [`RP-002`](../rp002_coralys_learning_architecture.md) — Coralys Learning Architecture  
**Status:** **INITIAL FREEZE 2026-11-16** (reading register only)  
**Architecture status:** **NOT FROZEN** — literature may revise architectural assumptions  
**Authority:** **0%**  
**Scope:** Literature that directly informs an integrated **learning-enabled evolutionary optimisation** system—not a generic ML reading list.

---

## Freeze posture

| Artifact | Freeze? |
| -------- | ------- |
| This Research Reading Register (v0.1 initial set) | **Yes** — baseline reading set; may *add* entries; removals/reprioritisation require changelog |
| ARCH-001-LI / CLA architecture decisions | **No** — evidence may change assumptions |
| Experiment protocols / harness | **Not yet** |
| Domain freezes (Gate 5, OF ±0.30, OFX) | Unchanged by this register |

**Access note:** Links identify papers; full-text availability varies by publisher. Obtain and study full papers before design commitments—do not rely on abstracts alone.

**Strategic finding:** Substantial prior art already exists on adaptive operator selection, bandit-based optimisation, and multi-objective RL. Coralys should **build on that work**, then investigate the less-explored combination of **multi-objective knowledge preservation**, **cross-instance learning**, and **continual adaptation**.

### Study progress (Batch 1)

| # | Paper | Full text | Notes |
| - | ----- | --------- | ----- |
| 1 | Li et al. FRRMAB (2014) | Local PDF | Studied — see [`07_research_hypotheses_and_experiments/BATCH1_WITHIN_RUN_LEARNING_NOTES.md`](07_research_hypotheses_and_experiments/BATCH1_WITHIN_RUN_LEARNING_NOTES.md) |
| 2 | DaCosta et al. D-MAB (2008) | Local PDF | Studied — same notes |
| 3 | Roijers et al. (2013) | Local PDF | Studied — same notes |
| 4 | Lin et al. B-AOS (2021) | **Blocked** | Abstract-only pending institutional access |
| 5 | Jiao et al. OVEA (2023) | **Blocked** | Abstract-only pending institutional access |

Local PDFs: `02_adaptive_operator_selection/papers/`, `03_multi_objective_learning/papers/`.

---

## Library layout

```text
docs/research/rp002/
  01_evolutionary_learning_foundations/
  02_adaptive_operator_selection/
  03_multi_objective_learning/
  04_meta_learning_and_transfer/
  05_continual_learning/
  06_evaluation_and_benchmarking/
  07_research_hypotheses_and_experiments/
  RP002_RESEARCH_READING_REGISTER.md   ← this file
```

Local PDF notes / annotated extracts go in the matching folder when obtained. Citations remain authoritative even when PDFs are not yet checked in.

---

## Consolidated research-gap matrix

| Research gap | Primary resources | Priority |
| ------------ | ----------------- | -------- |
| Adaptive operator selection | 1, 2, 3 (+ Jiao 2023) | P0 |
| Multi-objective credit assignment | 4, 5, 6 (+ 2025 scaled AOS) | P0 |
| Contextual bandits | 7, 8, 9 | P0 |
| Sequential reinforcement learning | 4, 5, 14 | P1 |
| Cross-instance learning | 10, 11, 12, 13 | P0 |
| Knowledge retention | 14, 15 | P1 |
| Non-stationary adaptation | 2, 9, 14, 16 | P1 |
| Generalisation and benchmarking | 11, 17, 18 | P0 |

---

## Recommended reading order

### Batch 1 — Establish the within-run learning mechanism

| # | Paper | Outcome |
| - | ----- | ------- |
| 1 | Li et al. — FRRMAB (2014) | |
| 2 | DaCosta et al. — Dynamic MAB (2008) | |
| 3 | Roijers et al. — MO sequential decisions (2013) | |
| 4 | Lin et al. — Bicriteria Assisted AOS (2021) | |
| 5 | Jiao et al. — Q-Learning operator adaptation (2023) | |

**Outcome:** Decide how Coralys learns **within** an optimisation run.

### Batch 2 — Establish knowledge transfer

| # | Paper | Outcome |
| - | ----- | ------- |
| 6 | Tornede et al. — Algorithm selection on a meta level (2023) | |
| 7 | Cenikj et al. — Meta-features survey (2024) | |
| 8 | Sehta & Thakar — Combinatorial algorithm selection (2025) | |
| 9 | Khetarpal et al. — Continual RL (2022) | |

**Outcome:** Decide how Coralys learns **across** runs and instances.

### Batch 3 — Theoretical and experimental rigour

| # | Paper | Outcome |
| - | ----- | ------- |
| 10 | 2026 MORL survey | |
| 11 | Pan et al. — Continual RL survey (2025) | |
| 12 | Rice — Algorithm Selection Problem (1976) | |
| 13 | Instance Space Analysis (2023) | |
| 14 | Safe Continual RL survey (2026) | |

**Outcome:** Finalise evaluation, generalisation, safety, and theory posture.

**First implementation-oriented trio (levels of Coralys learning):**

1. **FRRMAB (Li et al., 2014)** — which strategy/operator to use  
2. **Roijers et al. (2013)** — how to preserve competing objectives  
3. **Tornede et al. (2023)** — which knowledge is useful for a new problem  

---

## Complexity ladder (do not skip)

Before deep RL, compare experimentally:

1. Fixed operator probabilities  
2. Probability matching  
3. UCB  
4. FRRMAB  
5. Contextual bandits  
6. Q-learning  
7. Deep RL **only if** simpler models demonstrate limitations  

---

## Section 01 — Evolutionary Learning Foundations

Folder: `01_evolutionary_learning_foundations/`

| ID | Priority | Citation | Why | Coralys application |
| -- | -------- | -------- | --- | ------------------- |
| F-01 | P0 | Grefenstette, J. J. (1993). *Genetic Algorithms and Machine Learning.* ACM COLT ’93. | Populations, fitness selection, genetic operators, sequential decision rules, simulation evaluation, non-stationary adaptation; SAMUEL condition–action rule learning. | Evolutionary learning of knowledge structures; simulator-evaluated strategies. |
| F-02 | P0 | Schaffer, J. D., & Grefenstette, J. J. *Multi-Objective Learning via Genetic Algorithms.* | Scalar fitness extinguishes specialists; fitness vectors + per-component selection preserve complementary knowledge; critic design critical. | Central subject for multidimensional learning-controller feedback (extension beyond classification demo). |

Canonical programme write-up: [`../rp002_coralys_learning_architecture.md`](../rp002_coralys_learning_architecture.md) §2.

---

## Section 02 — Adaptive Operator Selection (highest near-term priority)

Folder: `02_adaptive_operator_selection/`  
Closest literature to current `coralys-moga` / EvolutionEngine.

| ID | Priority | Citation | Link | Why | Coralys application |
| -- | -------- | -------- | ---- | --- | ------------------- |
| 1 | P0 | Li, K., Fialho, Á., Kwong, S., & Zhang, Q. (2014). Adaptive Operator Selection With Bandits for a Multiobjective Evolutionary Algorithm Based on Decomposition. *IEEE TEVC, 18*(1), 114–130. | [Essex repository](https://repository.essex.ac.uk/11564/) | **FRRMAB**: sliding window of recent operator performance; fitness improvement rates; reward ranking; decaying selection probabilities. | Direct reference for AOS in EvolutionEngine; **first paper to reproduce experimentally**. |
| 2 | P0 | DaCosta, L., Fialho, Á., Schoenauer, M., & Sebag, M. (2008). Adaptive Operator Selection with Dynamic Multi-Armed Bandits. *GECCO 2008.* | [PDF](https://gpbib.pmacs.upenn.edu/gecco2008/docs/p913.pdf) | Bandit AOS under changing reward distributions; Page-Hinkley change detection. | Detect when previously effective operators stop working. |
| 3 | P0 | Drake, J. H., Kheiri, A., Özcan, E., & Burke, E. K. (2020). Recent Advances in Selection Hyper-Heuristics. *EJOR, 285*(2), 405–428. | [ScienceDirect](https://www.sciencedirect.com/science/article/pii/S0377221719306526) | Broader selection hyper-heuristics survey. | Distinguish learning which **operator** vs which **higher-level search strategy**. |
| 3b | P0 | Jiao et al. (2023). A Reference Vector Based Multiobjective Evolutionary Algorithm with Q-Learning for Operator Adaptation. | [ScienceDirect](https://www.sciencedirect.com/science/article/pii/S2210650222001912) | Q-learning + operator adaptation + reference-vector MOEA. | Concrete comparison vs simpler bandits. |

---

## Section 03 — Multi-Objective Learning and Credit Assignment

Folder: `03_multi_objective_learning/`  
Opportunity for a distinctive Coralys approach (vector feedback / complementary strategies).

| ID | Priority | Citation | Link | Why | Coralys application |
| -- | -------- | -------- | ---- | --- | ------------------- |
| 4 | P0 | Roijers, D. M., Vamplew, P., Whiteson, S., & Dazeley, R. (2013). A Survey of Multi-Objective Sequential Decision-Making. *JAIR, 48*, 67–113. | [JAIR](https://mlanthology.org/jair/2013/roijers2013jair-survey/) | Multi-objective decision processes; scalarisation; Pareto policies; convex coverage sets; preference change. | Mathematical foundations for multi-objective learning policies. |
| 5 | P0 | Multi-Objective Reinforcement Learning: A Comprehensive Survey… (2026). | [T&F](https://www.tandfonline.com/doi/full/10.1080/21642583.2026.2672169) | MOMDPs; model-based/free; preference representation; Pareto/coverage; benchmarks; safety/alignment. | One adaptive policy vs specialised policies vs preference-conditioned policy. |
| 6 | P0 | Lin et al. (2021). Decomposition-Based Multiobjective Optimization with Bicriteria Assisted Adaptive Operator Selection. | [ScienceDirect](https://www.sciencedirect.com/science/article/pii/S2210650220304430) | Two operator pools (explore/exploit); Pareto + crowding feedback. | Feedback that recognises **convergence and diversity**, not only one reward. |
| 6b | P1 | Adaptive Operator Selection with Bandits for Scaled Problems Based on Decomposition-Based MOEAs (2025). | [T&F](https://www.tandfonline.com/doi/full/10.1080/0305215X.2025.2466119) | Objective-scale distortion of operator rewards. | Highly relevant when Coralys objectives have different numerical ranges. |

---

## Section 04 — Contextual Bandits and Reinforcement Learning

Folder: `02_adaptive_operator_selection/` (bandit theory) and `03_multi_objective_learning/` (MORL) as applicable.  
Decide when simple adaptive learning suffices vs sequential RL.

| ID | Priority | Citation | Link | Why | Coralys application |
| -- | -------- | -------- | ---- | --- | ------------------- |
| 7 | P0 | Auer, P., Cesa-Bianchi, N., & Fischer, P. (2002). Finite-Time Analysis of the Multiarmed Bandit Problem. *Machine Learning, 47*, 235–256. | [DOI](https://doi.org/10.1023/A:1013689704352) | UCB / regret analysis foundation. | Mathematically grounded AOS baseline. |
| 8 | P0 | Li, L., Chu, W., Langford, J., & Schapire, R. E. (2010). A Contextual-Bandit Approach to Personalized News Article Recommendation. *WWW 2010.* | [DOI](https://doi.org/10.1145/1772690.1772758) | Context → action → feedback; offline evaluation from logged interactions. | Context = problem/search state; action = operator; feedback = performance; replay evaluation. |
| 9 | P1 | Auer, P., Cesa-Bianchi, N., Freund, Y., & Schapire, R. E. (2002). The Nonstochastic Multiarmed Bandit Problem. | [SIAM](https://epubs.siam.org/doi/10.1137/S0097539701398375) | Bandits without stationary stochastic rewards. | AOS when operator performance changes during a run. |

---

## Section 05 — Meta-Learning and Transfer (strategically critical gap)

Folder: `04_meta_learning_and_transfer/`  

**Concept distinction (do not conflate):**

| Concept | Meaning |
| ------- | ------- |
| Algorithm selection | Which solver / configuration fits an instance |
| Meta-learning | How to improve the selection mechanism itself |
| Transfer learning | Reuse acquired knowledge on a new task |

| ID | Priority | Citation | Link | Why | Coralys application |
| -- | -------- | -------- | ---- | --- | ------------------- |
| 10 | P0 | Tornede, A., Gehring, L., Tornede, T., Wever, M., & Hüllermeier, E. (2023). Algorithm Selection on a Meta Level. *Machine Learning, 112*, 1253–1286. | [Springer](https://link.springer.com/article/10.1007/s10994-022-06161-4) | Combining algorithm selectors via meta-learning. | Select/combine learning policies by instance. |
| 11 | P0 | Cenikj, G., et al. (2024). A Survey of Meta-features… Black-box Single-objective Continuous Optimization. | [arXiv](https://arxiv.org/abs/2406.06629) | Instance features; selection; configuration; performance prediction; representation learning. | Problem-characterisation subsystem. **Method only**—do not assume direct CVRP/rostering transfer. |
| 12 | P1 | Sehta, N., & Thakar, U. (2025). Study of Meta-Learning Attempts to Select Algorithm for Hard Combinatorial Optimisation Problems. | [Inderscience](https://www.inderscience.com/info/inarticle.php?artid=150111) | Combinatorial algorithm selection + instance features. | Transfer across CVRP, scheduling, etc. |
| 13 | P1 | Rice, J. R. (1976). The Algorithm Selection Problem. | [DOI](https://doi.org/10.1016/s0065-2458(08)60520-3) | Problem / feature / algorithm / performance spaces. | Classical mathematical frame for strategy-vs-instance learning. |

---

## Section 06 — Continual Learning and Adaptation

Folder: `05_continual_learning/`

| ID | Priority | Citation | Link | Why | Coralys application |
| -- | -------- | -------- | ---- | --- | ------------------- |
| 14 | P0 | Khetarpal, K., Riemer, M., Rish, I., & Precup, D. (2022). Towards Continual Reinforcement Learning: A Review and Perspectives. *JAIR, 75*, 1401–1476. | [JAIR](https://mlanthology.org/jair/2022/khetarpal2022jair-continual/) | Taxonomy; non-stationarity forms; retention; adaptation; evaluation. | Learning lifecycle across multiple optimisation runs. |
| 15 | P1 | Pan, C., et al. (2025). A Survey of Continual Reinforcement Learning. | [arXiv](https://arxiv.org/abs/2506.21872) | Knowledge storage/transfer; benchmarks; metrics. | Replay vs regularisation vs modular retention. |
| 16 | P2 | Tomashevskiy, T. (2026). Safe Continual Reinforcement Learning Methods for Nonstationary Environments. | [arXiv](https://arxiv.org/abs/2601.05152) | Safe online adaptation under change. | Adapt without violating hard constraints. **Emerging preprint—not consensus.** |

---

## Section 07 — Evaluation and Benchmarking

Folder: `06_evaluation_and_benchmarking/`  
Prove learning contributes value, not only compute cost.

| ID | Priority | Citation | Link | Why | Coralys application |
| -- | -------- | -------- | ---- | --- | ------------------- |
| 17 | P0 | Instance Space Analysis for Algorithm Testing: Methodology and Software Tools. | [DOI](https://doi.org/10.1145/3572895) | Performance across regions of instance space. | Generalisation vs average-score illusions. |
| 18 | P1 | Measuring Instance Difficulty for Combinatorial Optimization Problems. | [ScienceDirect](https://www.sciencedirect.com/science/article/pii/S0305054811001997) | Instance difficulty ↔ algorithm performance. | Features; train/val/test distributions. |

### Experiment framework derived from the library

| Experiment | What we need to prove |
| ---------- | --------------------- |
| Within-run adaptation | Learning improves search behaviour |
| Across-run learning | Previous experience improves subsequent runs |
| Unseen-instance evaluation | Learning generalises |
| Cross-family evaluation | Knowledge transfers beyond one benchmark family |
| Distribution-shift evaluation | Learning adapts when conditions change |
| Ablation | Each learning component contributes measurable value |

**Benchmark posture (proposed, not frozen):** retain **CVRP** as first operational testbed; add **ZDT / DTLZ / WFG** for general multi-objective evaluation—to separate general MO learning gains from routing-specific gains.

Folder for protocols/findings: `07_research_hypotheses_and_experiments/`.

---

## Candidate research directions (hypotheses — not novelty claims)

Exhaustive prior-art review required before asserting novelty.

| ID | Direction | Thesis |
| -- | --------- | ------ |
| RD-1 | Multi-objective Learning Controller | Retain multidimensional operator effectiveness; select strategies without collapsing all feedback into one reward. |
| RD-2 | Experience-driven Optimisation | Learn across instances: problem characteristics, operator effectiveness, search configurations. |
| RD-3 | Adaptive Knowledge Preservation | Retain complementary optimisation strategies; recognise when knowledge should be reused, revised, or discarded. |

**RD-3** explicitly bridges foundational refs F-01/F-02 with modern AOS / MORL / continual-learning literature.

---

## Change log

| Version | Date | Notes |
| ------- | ---- | ----- |
| 0.1 | 2026-11-16 | Initial Research Reading Register frozen; 18+ resources across six gaps; reading batches; complexity ladder; RD-1..3; architecture **not** frozen |
| 0.1.1 | 2026-11-16 | Batch 1 progress: full text obtained+studied for Li/DaCosta/Roijers; Lin/Jiao blocked; within-run notes + provisional E0 design only |

---

*RP-002 Research Reading Register v0.1 | November 2026 | Reading register frozen · Architecture unfrozen · Authority 0%*
