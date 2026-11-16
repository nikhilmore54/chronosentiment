# RP-002 Batch 1 — Within-Run Learning Notes

**Status:** Batch 1 study in progress (2026-11-16)  
**Programme:** [`../../rp002_coralys_learning_architecture.md`](../../rp002_coralys_learning_architecture.md)  
**Register:** [`../RP002_RESEARCH_READING_REGISTER.md`](../RP002_RESEARCH_READING_REGISTER.md)  
**Architecture:** still **unfrozen** — notes are evidence, not design lock  
**Authority:** 0%

---

## Access status

| # | Paper | Full text | Local path |
| - | ----- | --------- | ---------- |
| 1 | Li et al. FRRMAB (2014) | **Obtained** | `02_adaptive_operator_selection/papers/li2014_frrmab_moead.pdf` |
| 2 | DaCosta et al. D-MAB (2008) | **Obtained** | `02_adaptive_operator_selection/papers/dacosta2008_dynamic_mab_aos.pdf` |
| 3 | Roijers et al. (2013) | **Obtained** | `03_multi_objective_learning/papers/roijers2013_mo_sequential_decision_making.pdf` |
| 4 | Lin et al. B-AOS (2021) | **Blocked** (Elsevier) — abstract + metadata only | — |
| 5 | Jiao et al. OVEA/Q-learning (2023) | **Blocked** (Elsevier) — abstract + metadata only | — |

---

## 1. Li et al. (2014) — FRRMAB / MOEA/D-FRRMAB

### Mechanism (two modules)

**A. Credit assignment (FIR + sliding window + rank decay)**

1. Measure impact via **fitness improvement rate**, not raw Δfitness:
   - `FIR = (pf − cf) / pf` (parent vs child fitness on the scalar subproblem).
2. Store recent `(operator_id, FIR)` pairs in a **FIFO sliding window** of size `W`.
3. Sum FIR per operator in the window → `Reward_i`.
4. Rank rewards; apply decay `Decay_i = D^{Rank_i} × Reward_i`.
5. Normalise to **Fitness-Rate-Rank** credit: `FRR_i = Decay_i / Σ Decay_j`.
   - Smaller `D` → stronger preference for the current best operator.

**B. Operator selection (UCB-like on FRR)**

- Warm-start: each operator selected at least once before FRRMAB engages.
- Thereafter: `arg max_i [ FRR_i + C × sqrt( (2 ln Σ n_j) / n_i ) ]` where `n_i` counts uses **inside the window**.
- Exploration constant `C` trades EvE.

### Why MOEA/D (not NSGA-II-style)

Dominance-based MOEAs make **quantitative** operator credit hard. MOEA/D decomposes into **scalar** subproblems (here Tchebycheff), so FIR is well-defined per subproblem update. Integration studied on MOEA/D-DRA with a pool of four DE mutations.

### Experimental claims (paper)

- FRRMAB improves MOEA/D vs several MOEA/D variants and vs other AOS schemes.
- Operator selection dynamics are “reasonable”; sensitivity analysis on `W`, `D`, `C`.
- Robustness emphasised vs raw-fitness credit.

### Coralys implications

| Topic | Implication |
| ----- | ----------- |
| First reproduce | **Yes** — FRRMAB is the clearest EvolutionEngine AOS candidate |
| Credit | Prefer **normalised rate / rank**, not raw objective deltas (scale distortion) |
| Dynamics | Sliding window is the primary non-stationarity handle (vs full-history UCB) |
| MO fit | Easier if Coralys MO path exposes **scalarised subproblem progress**; harder on pure dominance-only paths |
| Gap vs RD-1 | FRRMAB still collapses credit to **one scalar FIR per application** — does **not** yet implement Schaffer-style multi-component preservation of complementary operators |

---

## 2. DaCosta et al. (2008) — Dynamic Multi-Armed Bandits (D-MAB)

### Mechanism

AOS = **credit assignment** + **adaptation rule**. Paper focuses on the adaptation rule:

- Treat operator choice as a **Dynamic MAB** (reward distributions change during search).
- **D-MAB** = UCB1 + **Page-Hinkley** change detection.
- When PH test fires (`M_t − m_t > λ`), **restart** UCB statistics from scratch.

Compared on artificial reward scenarios (EA-independent) against:

- Probability Matching  
- Adaptive Pursuit  
- Static MAB  

Finding (paper): D-MAB outperforms others on one literature scenario; on another, **static** MAB wins — adaptation cost/benefit is scenario-dependent.

### Coralys implications

| Topic | Implication |
| ----- | ----------- |
| Non-stationarity | Explicit **change detection + reset** is an alternative (or complement) to FRRMAB’s sliding window |
| Complexity ladder | Keep Probability Matching / Adaptive Pursuit / UCB / D-MAB / FRRMAB as **ordered baselines** |
| Caution | Adaptation is not free — must prove net search benefit (aligns with eval gap / ablations) |
| Gap | Credit assignment largely out of scope here; Coralys still needs a **multi-criteria** credit story |

---

## 3. Roijers et al. (2013) — Multi-Objective Sequential Decision-Making

### Core claim

Scalarising an MOMDP **before** planning/learning is sometimes **impossible, infeasible, or undesirable**. Three motivating scenarios (unknown weights / interactive preferences / known weights but hard conversion) justify multi-objective methods.

### Taxonomy (utility-based)

Critical factors:

1. **One policy vs coverage set** of policies  
2. **Linear vs monotonically increasing** scalarisation  
3. Allowed **policy types** (deterministic stationary / non-stationary / stochastic mixtures)

Solution concept then becomes:

- a **single** policy, or  
- a **convex coverage set**, or  
- a **Pareto front** of policies  

Vector-valued returns yield only a **partial order** — `V_i` can improve while `V_j` worsens.

### Coralys implications

| Topic | Implication |
| ----- | ----------- |
| Controller design | Decide early: one adaptive operator policy vs a **coverage set** of specialised policies vs preference-conditioned selection |
| Scalarisation timing | Prefer **late** scalarisation / user preference injection at Decision layer when possible |
| Link to F-02 | Formal backbone for “do not extinguish specialists” — coverage / Pareto policy sets |
| Link to FRRMAB | FRRMAB AOS is still a **single-policy** learner with scalar credits; Roijers suggests when that is insufficient |
| Decision MVP | Aligns with Decision evaluating alternatives under governance rather than baking one weight vector into Learning |

---

## 4. Lin et al. (2021) — B-AOS (abstract only)

**Bicriteria Assisted Adaptive Operator Selection** for MOEA/D:

- Two **operator pools** (exploitation vs exploration), each with two DE operators.
- **Pareto criterion** chooses exploration vs exploitation pool.
- **Crowding criterion** + binary tournament picks operator inside the pool.
- Claimed better than prior AOS; improves several MOEA/Ds on most benchmarks.

**Coralys note (provisional):** closest published neighbour to “credit that recognises **convergence and diversity**.” Full-text still required before adopting bicriteria control logic. Complements FRRMAB’s single FIR credit.

---

## 5. Jiao et al. (2023) — OVEA / Q-learning AOS (abstract only)

- Reference-vector MOEA + **Q-learning** to pick crossover operators from offspring performance.
- Elite selection via reference-vector associated subsets.
- Evaluated on 2–10 objectives; claims advantage vs SOTA.

**Coralys note (provisional):** places **Q-learning** on the complexity ladder **after** bandits; useful comparison once FRRMAB baseline exists. Full text still required.

---

## Provisional within-run learning posture (not architecture freeze)

Evidence-supported working hypotheses for Coralys EvolutionEngine AOS:

1. **Start** with FRRMAB-style credit (FIR/rank/window) + UCB selection on a fixed operator pool.  
2. **Add** an explicit non-stationarity check (Page-Hinkley reset **or** window-only — ablate).  
3. **Do not** jump to Q-learning / deep RL until fixed / PM / UCB / FRRMAB / contextual bandit ladder fails.  
4. Treat **multi-criteria credit** (Lin-style Pareto+crowding; Schaffer-style vector selection) as the **differentiation** track (RD-1 / RD-3), not as v0.  
5. Use Roijers taxonomy to decide whether the controller returns one policy or a **coverage set** of operator strategies for Decision.

### Candidate first experiment (design only — not authorised to run)

| Field | Draft |
| ----- | ----- |
| Name | `RP-002-E0` FRRMAB reproduce-or-port |
| Goal | Show within-run AOS beats fixed operator probabilities on a Coralys MO path |
| Control | Fixed equal operator probabilities |
| Treatment | FRRMAB (W, D, C as paper defaults ± sensitivity) |
| Metrics | Primary MO quality + operator-use entropy + wall-clock overhead |
| Bench | ZDT/DTLZ/WFG first for MO purity; CVRP later for operational transfer |
| Gate | Must beat control on quality **or** match quality at lower eval budget; overhead bounded |

---

## Open actions

- [ ] Obtain Lin (2021) and Jiao (2023) full texts (institutional access)
- [ ] Map Coralys `coralys-moga` operator surface → FRRMAB arm set
- [ ] Specify `RP-002-E0` protocol under `07_research_hypotheses_and_experiments/`
- [ ] Begin Batch 2 after E0 design sketch (Tornede / meta-features / continual)

---

*Batch 1 notes v0.1 | 2026-11-16 | Full text: Li, DaCosta, Roijers · Abstract-only: Lin, Jiao*
