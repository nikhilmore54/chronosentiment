# Architectural Specification: Metaheuristics & Dynamic Pricing for Nurse Rostering (ALNS + RCSP)

**Platform Standard:** RP-600 Compliant  
**Target Domain:** Combinatorial Workforce Rostering (`adapters/ultracrew/src/inrc/`)  
**Reference Competitions & Baselines:** INRC-II, Ceschia et al. (2019 / *Annals of OR*), Römer & Mellouli (*Schedule Nurse III*), Legrain et al. (2020)

---

## 1. Executive Summary & Algorithmic Problem Statement

In workforce scheduling (specifically the Second International Nurse Rostering Competition, INRC-II), solutions must balance statutory hard constraints (minimum daily staffing $HC_1$, multi-skill competencies $HC_2$, single-shift assignment $HC_3$, forbidden succession rules $HC_4$) against multi-objective ergonomic soft penalties ($S_1$ assignment quotas, $S_2$ consecutive working streaks, $S_3$ consecutive rest streaks, $S_4$ shift-type streaks, $S_6$ complete weekends, $S_7$ maximum working weekends, and $S_8$ optimal staffing coverage deficits).

### The Inherent Flaw of Static Rotation Caching (`RotationPool`)
Previous iterations relied on an offline dictionary of precomputed 7-day candidate rotations (`RotationPool`). While effective as a quick baseline, static rotation caching exhibits four catastrophic algorithmic limitations:
1. **Combinatorial Explosion ($\lvert\Sigma\rvert^D$):** For horizons exceeding 7 days ($D=14, 28$) or expanded shift alphabets ($\lvert\Sigma\rvert \ge 8$), exhaustive enumeration is intractable ($\approx 9^{14} \approx 2.3 \times 10^{13}$).
2. **Decoupled Ergonomics vs. Collective Demand:** A rotation evaluated in isolation cannot account for the collective staffing state of the hospital. An "ergonomically perfect" rotation for nurse $u$ is catastrophic if it supplies shifts that are already overstaffed while starving shifts in statutory deficit.
3. **Disjoint Basins of Attraction:** Swapping an entire nurse schedule to a pre-cached template causes simultaneous coverage deltas across all 7 days, resulting in high rejection rates ($> 95\%$) in simulated annealing.
4. **Divergence from Literature State of the Art:** Competition winners (Römer & Mellouli, Legrain et al., Ceschia et al.) do not store static rotation tables; they generate lines of work **on the fly** using **Resource-Constrained Shortest Path (RCSP) Pricing** on Directed Acyclic Graphs (DAGs) and optimize schedules via **Adaptive Large Neighborhood Search (ALNS)**.

---

## 2. Metaheuristic Taxonomy & Algorithmic Design

To achieve world-class optimization without static caches, Coralys deploys a unified metaheuristic architecture founded on three synergistic paradigms:

```
+---------------------------------------------------------------------------------------------------+
|                                 CORALYS ROSTERING METAHEURISTIC ENGINE                            |
+-----------------------------------+-----------------------------------+---------------------------+
| 1. Dynamic RCSP Pricing (DAG DP)  | 2. Adaptive LNS (Ruin & Recreate) | 3. Invariance Operators   |
| - State: (day, shift, streak)     | - Destroy: Window, Worst, Weekend | - Intra-Skill Day Swap    |
| - Cost: Ergonomics + Shadow Price | - Repair: Regret-k & Bipartite    | - Kempe Block Swap (ΔC=0) |
| - Complexity: O(D · |Σ|²)         | - Adaptive Roulette Selection     | - Stub & Shift Harmonizers|
+-----------------------------------+-----------------------------------+---------------------------+
|                                    4. Orchestration & Control                                     |
| - Variable Neighborhood Descent (VND) Ladder: N1 (1-day) -> N2 (Kempe) -> N3 (ALNS) -> N4 (RCSP)  |
| - Simulated Annealing with Strategic Oscillation & Stagnation Reheating                           |
+---------------------------------------------------------------------------------------------------+
```

---

## 3. Paradigm 1: Resource-Constrained Shortest Path (RCSP) Dynamic Pricing

Instead of storing precomputed rotations in memory, the optimal 7-day line of work for any nurse $n$ is computed dynamically in $< 10\,\mu\text{s}$ via topological Dynamic Programming on a Directed Acyclic Graph (DAG).

### 3.1 State Graph Definition
For nurse $n$ over days $d \in \{0, \dots, D-1\}$:
- **Vertices:** $V = \{ (d, s, k) \}$ where:
  - $d \in \{0, \dots, 6\}$ is the calendar day.
  - $s \in \{0, \dots, \lvert\Sigma\rvert\}$ is the shift type assigned ($0 = \text{Off}$, $1 = \text{Early}$, $2 = \text{Day}$, $3 = \text{Late}$, $4 = \text{Night}$).
  - $k \in \{0, \dots, k_{\max}\}$ is the current active streak length (working streak if $s > 0$, rest streak if $s = 0$).
- **Source Node:** Initial Sunday state $(d=-1, s_{\text{hist}}, k_{\text{hist}})$ loaded from `InrcHistory`.
- **Target Node:** Terminal Sunday state $(d=6, s_6, k_6)$.

### 3.2 Transitions and Edge Weights
A directed edge exists from $(d-1, s_{\text{prev}}, k_{\text{prev}})$ to $(d, s, k)$ if and only if:
1. $HC_2$ is satisfied: Nurse $n$ possesses the required skills for shift $s$ on day $d$.
2. $HC_4$ is satisfied: Transition $s_{\text{prev}} \to s$ is not forbidden by hospital succession rules.
3. Streak continuity:
   $$k = \begin{cases} k_{\text{prev}} + 1 & \text{if } (s_{\text{prev}} > 0 \land s > 0) \lor (s_{\text{prev}} = 0 \land s = 0) \\ 1 & \text{otherwise} \end{cases}$$

The edge weight $W(d-1 \to d)$ incorporates both marginal ergonomic cost and dynamic staffing shadow credits:
$$W = \Delta \text{Soft}(n, d, s, k) - \lambda_d(s)$$
where:
- $\Delta \text{Soft}$ evaluates working streak bounds ($S_2$), rest streak bounds ($S_3$), shift streaks ($S_4$), weekend consistency ($S_6$), and shift preferences ($S_8$).
- $\lambda_d(s) \ge 0$ is the marginal staffing credit (shadow value) rewarding the assignment of nurse $n$ to shift $s$ on day $d$ if that shift currently experiences an optimal or mandatory coverage shortfall.

### 3.3 Complexity & Performance
For $D=7$, $\lvert\Sigma\rvert = 5$, and $k_{\max} = 7$:
- Total nodes: $\lvert V\rvert \le 7 \times 5 \times 8 \approx 280$.
- Total edges: $\lvert E\rvert \le 280 \times 5 \approx 1,400$.
- In topological order (left-to-right along calendar days), finding the exact optimal rotation via Bellman-Ford / DAG shortest path takes **$O(\lvert V\rvert + \lvert E\rvert) \approx 1,680$ operations ($< 5\,\mu\text{s}$)** in Rust, without memory allocations.

---

## 4. Paradigm 2: Adaptive Large Neighborhood Search (ALNS)

ALNS escapes combinatorial traps by destroying correlated sub-regions of the roster and reconstructing them using heuristic or exact matchings.

### 4.1 Destroy (Ruin) Operators
1. **Temporal Window Ruin (`ruin_window`):**
   - Selects a cluster of $M \in [4, 8]$ nurses sharing contract profiles.
   - Selects a contiguous temporal window (e.g., weekend $\{5, 6\}$ or 2 weekdays).
   - Clears all assignments within this window, placing unassigned shifts into a pool $\mathcal{P}$.
2. **High-Penalty Nurse Ruin (`ruin_worst_nurses`):**
   - Evaluates individual nurse soft penalties $\sum_{i=1}^8 S_i(n)$.
   - Ranks nurses and completely unassigns the top $k \in [2, 5]$ worst-scoring nurses.
3. **Weekend Stub Ruin (`ruin_weekend_stubs`):**
   - Identifies nurses with split weekends ($S_6$: working Saturday and off Sunday, or vice versa).
   - Clears assignments across Saturday and Sunday for these nurses.

### 4.2 Repair (Recreate) Operators
1. **Regret-$k$ Insertion (`recreate_regret`):**
   - For every unassigned shift $(d, s) \in \mathcal{P}$, identifies eligible nurses $u \in \mathcal{N}_{\text{cand}}$.
   - Calculates the cost delta $\Delta c_i$ for assigning $(d, s)$ to the $i$-th best nurse.
   - Computes Regret-2: $R(d, s) = \Delta c_2 - \Delta c_1$.
   - Inserts the shift with the largest regret first.
2. **Bipartite Minimum-Cost Matching Reinsertion (`recreate_bipartite`):**
   - For day $d$, builds a bipartite matching graph between unassigned shifts and idle qualified nurses.
   - Solves Kuhn-Munkres matching to maximize coverage and minimize individual soft cost increments.

### 4.3 Adaptive Weight Adjustment
Each operator pair $(D_i, R_j)$ maintains an adaptive weight $w_{ij}$:
$$w_{ij}^{(t+1)} = \rho \cdot w_{ij}^{(t)} + (1 - \rho) \cdot \pi$$
where $\pi \in \{\sigma_1, \sigma_2, \sigma_3\}$ rewards moves that find a new global best ($\sigma_1 = 10$), an improving move ($\sigma_2 = 5$), or an accepted neutral move ($\sigma_3 = 2$). Operators are selected via roulette-wheel sampling.

---

## 5. Paradigm 3: Variable Neighborhood Descent (VND) Ladder

Inside the local optimizer and simulated annealing engine, moves are organized into a strict hierarchical ladder of increasing perturbation radius:

```
[Level 1: O(1) Local Polish]
  -> Intra-skill day swap between two nurses
  -> Single-nurse shift toggle / preference fix
          | (if local minimum reached)
          v
[Level 2: Coverage-Invariant Kempe Exchanges]
  -> Weekend Kempe Harmonizer (swap Sat/Sun pairs, ΔCov = 0)
  -> 2-3 Day Interval Block Swap between compatible nurses
  -> Stub Ejection & Streak Consolidation (S2 / S4)
          | (if local minimum reached)
          v
[Level 3: Dynamic RCSP Pricing Generation]
  -> Dynamically generate optimal 7-day line of work for bottleneck nurses
  -> Replace degraded lines of work with zero-defect DAG paths
          | (if local minimum reached)
          v
[Level 4: Adaptive Large Neighborhood Search (ALNS)]
  -> Ruin 4-8 nurses across weekend or temporal windows
  -> Reconstruct via Regret-k / Bipartite Matching
```

When an improving move is discovered at any level $L > 1$, the search immediately restarts at **Level 1** to exhaustively exploit the new basin of attraction (intensification) before escalating to larger perturbations (diversification).

---

## 6. Implementation Architecture in `adapters/ultracrew`

```
adapters/ultracrew/src/inrc/
├── pricing.rs            <-- NEW: Resource-Constrained Shortest Path (RCSP) on DAG
├── ruin_recreate.rs      <-- UPGRADED: Full ALNS (Window, Worst, Weekend + Regret-k)
├── evaluator.rs          <-- UPGRADED: VND ladder & Dynamic Pricing SA operators
├── optimization.rs       <-- DEPRECATED: Remove candidate_rotations static cache
└── rotation_pool.rs      <-- RETAINED: Offline analysis / reference only
```

---

## 7. Mathematical & Empirical Validation Gate

The transition to pure metaheuristics must satisfy the following invariant criteria:
1. **Zero Hard Constraint Violations:** $HC_1 = HC_2 = HC_3 = HC_4 = 0$ across all 7 canonical instances ($N=30..120$).
2. **Zero Precomputed Rotation Lookups:** The solver must achieve competitive BKS parity without querying a static rotation pool.
3. **Execution Efficiency:** Solve time per 4-week instance must remain within competitive bounds ($< 300\,\text{s}$ multi-week).
4. **Optimality Gap Improvement:** Maintain or improve upon the current $+78.75\%$ mean gap across the canonical INRC-II benchmark suite.
