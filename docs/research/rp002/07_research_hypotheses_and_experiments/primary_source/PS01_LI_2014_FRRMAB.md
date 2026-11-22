# Primary-Source Analysis — Li et al. (2014) FRRMAB

**Citation:** Ke Li, Álvaro Fialho, Sam Kwong, Qingfu Zhang. *Adaptive Operator Selection With Bandits for a Multiobjective Evolutionary Algorithm Based on Decomposition.* IEEE TEVC 18(1):114–130, 2014. DOI: 10.1109/TEVC.2013.2239648  

**Local PDF:** [`../../02_adaptive_operator_selection/papers/li2014_frrmab_moead.pdf`](../../02_adaptive_operator_selection/papers/li2014_frrmab_moead.pdf)  

**Analysis posture:** What the **paper demonstrates** vs what **Coralys may propose** are separated below. No CLA architecture freeze follows from this note. Authority 0%.

---

## 1. Research problem and motivation

**Problem:** Adaptive Operator Selection (AOS) — choose which variation operator to apply online during an EA run, based on recent performance (credit assignment + selection rule), under the explore/exploit dilemma.

**Motivation:**

- Operator quality changes across search stages; static rates are suboptimal.
- Classical MABs (e.g. UCB) assume stationary reward distributions; AOS is dynamic.
- Most prior AOS work is **single-objective**. In dominance-based MOEAs, quantifying improvement for credit is hard.
- **MOEA/D** decomposes an MOP into scalar subproblems, making quantitative FIR feasible.

---

## 2. Mathematical formulation (as in paper)

**MOP:** minimise \(F(x)=(f_1(x),\ldots,f_m(x))\); Pareto set / front as usual.

**MOEA/D (Tchebycheff):** for weight \(\lambda^j\),

\[
g^{te}(x\mid\lambda^j,z^*)=\max_{1\le i\le m}\{\lambda^j_i\,|f_i(x)-z^*_i|\}.
\]

**Fitness improvement rate** for operator \(i\) at time \(t\):

\[
\mathrm{FIR}_{i,t}=\frac{pf_{i,t}-cf_{i,t}}{pf_{i,t}}
\]

(parent vs child scalar fitness on the relevant subproblem(s)).

**Sliding window:** FIFO of length \(W\) storing \((\mathrm{op},\mathrm{FIR})\).

**Credit (Fitness-Rate-Rank):** sum FIRs per operator in window → \(\mathrm{Reward}_i\); rank; decay

\[
\mathrm{Decay}_i=D^{\mathrm{Rank}_i}\times\mathrm{Reward}_i,\qquad
\mathrm{FRR}_{i,t}=\frac{\mathrm{Decay}_i}{\sum_j\mathrm{Decay}_j}.
\]

**Selection (UCB-like on FRR):**

\[
\mathrm{op}_t=\arg\max_i\left[\mathrm{FRR}_{i,t}+C\sqrt{\frac{2\ln\sum_j n_{j,t}}{n_{i,t}}}\right]
\]

with \(n_{i,t}\) = uses of \(i\) inside the window. Warm-start: each operator used ≥1 before FRRMAB.

---

## 3. Proposed algorithm and mechanisms

| Module | Mechanism |
| ------ | --------- |
| Credit | FIR (normalised) + sliding window + rank + decay \(D\) |
| Selection | UCB-style on FRR with exploration \(C\) |
| Host EA | MOEA/D-DRA; DE mutation pool of 4 operators |
| Integration | Algorithm 3 MOEA/D-FRRMAB: select op → generate/update neighbourhood → accumulate FIR into window |

**Operators pool:** DE/rand/1, DE/rand/2, DE/current-to-rand/2, DE/current-to-rand/1 (+ binomial crossover where applicable).

---

## 4. Assumptions and limitations (paper + reading)

**Assumptions**

- Scalar subproblem progress is a valid operator credit (MOEA/D decomposition).
- Recent window approximates current search dynamics.
- Rank+decay improves robustness vs raw fitness magnitudes.
- Operator pool is fixed and homogeneous (DE variants).

**Limitations demonstrated or admitted**

- Extra CPU (~40% vs MOEA/D-DRA/DE on reported UF suite) for credit/selection.
- Weaker on UF3 / UF10 (many local PFs) — authors suggest diversity in credit as future work.
- Not a multi-criteria credit vector; one scalar FIR per application.
- Not across-instance learning; within-run only.
- Does not address dominance-only MOEAs without decomposition.

---

## 5. Experimental methodology and findings

**Benchmarks:** UF1–UF10 (CEC’09); also many-objective cases in §V-G.  
**Metrics:** IGD, \(I_H\) (hypervolume-related); Wilcoxon rank-sum α=0.05; 30 runs.  
**Controls:** MOEA/D-DE, MOEA/D-DRA, ENS-MOEA/D; single-operator MOEA/D variants; other AOS schemes (§V-D).  
**Sensitivity:** \(W\), \(D\), \(C\) (§V-F). Defaults noted: e.g. \(W=0.5\times N\), \(D=1.0\) in reported settings.

**Findings (paper claims, supported by reported stats)**

- MOEA/D-FRRMAB best among compared MOEA/D variants on most UF instances (Table I: many “−” vs FRRMAB).
- Pool+FRRMAB beats each single DE operator alone (Table III).
- Operator trajectories show stage-dependent arm use (§V-E).
- Not free: measurable CPU overhead.

---

## 6. Relevance to Coralys

| Coralys concern | Paper relevance |
| --------------- | --------------- |
| Within-run AOS | **Direct** — first reproduce candidate (RP-002-E0) |
| EvolutionEngine | Needs arm pool + FIR telemetry (not present today) |
| ROADEF RP-405 | Same *problem class* (choose destroy arm online); different credit (weight×1.5 vs FIR+FRR+UCB) |
| Multi-objective controller (RD-1) | Partial — uses MOEA/D scalarisation for credit, **not** vector operator effectiveness |
| Cross-instance (RD-2) | **Out of scope** of this paper |

---

## 7. Architectural implications (proposals — **not frozen**)

**Demonstrated by paper → safe to treat as prior art for E0 design**

- Separate credit assignment from selection.
- Prefer normalised/rank credit over raw Δobjective.
- Use sliding window for non-stationarity (alternative: DaCosta PH-reset).

**Coralys proposals (extensions, not paper results)**

- Attach FRRMAB to ROADEF 5-arm destroy portfolio (E0).
- Later multi-criteria credit (convergence + diversity / Schaffer-style) — authors themselves flag diversity credit as future work.
- Do not freeze EvolutionEngine architecture on FRRMAB alone.

---

## 8. Unresolved research questions (for CLA)

1. Does FRRMAB beat RP-405 weight bandit on Coralys ROADEF under matched budget?  
2. How should FIR be defined when repair fails or destroy is empty?  
3. Can credit include diversity/feasibility without collapsing to one scalar?  
4. When does ~40%-class overhead fail Coralys cost gates?  
5. Does FRRMAB preserve complementary arms late in search (entropy), or converge to one winner?

---

*PS-01 v1.0 | 2026-11-16 | Primary source · Architecture unfrozen*
