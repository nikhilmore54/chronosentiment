# Architectural Specification: Capability Cohorts & Holistic Primary-Skill Rostering

**Platform Standard:** RP-600 Compliant  
**Target Domain:** Combinatorial Workforce Scheduling (`adapters/ultracrew/src/inrc/`)  
**Core Thesis:** Resolving weekly horizon tunnel vision and single-employee greed through Dynamic Capability Cohorts, Primary Specialization Hierarchies, and Multi-Week Duty Staggering.

---

## 1. Executive Summary & Problem Analysis

In combinatorial nurse rostering (INRC-II), current heuristics frequently converge to sub-optimal local minima due to two foundational architectural pitfalls:

### 1.1 The Single-Employee Greedy Allocation Trap
When optimization algorithms evaluate moves one nurse at a time ($n \in \{0, \dots, N-1\}$):
- **Skill Cannibalization:** Multi-skilled workers (e.g., nurses holding both `HeadNurse` and `Nurse` certifications) are greedily assigned to lower-tier slots (e.g., standard `Nurse` or `Caretaker`) early in the calendar week.
- **Deficit Cascade:** When high-tier demand (e.g., `HeadNurse`) arises later in the week, specialized personnel are either exhausted, locked in incompatible shift successions, or at their statutory work streak cap. This triggers severe hard constraint violations ($HC_2$ Skill Shortage) or heavy soft penalties ($S_8$ Optimal Coverage Deficit).

### 1.2 The Weekly Horizon Tunnel-Vision Dilemma
In multi-week competitions (e.g., 4-week and 8-week horizons), individual contracts—particularly non-full-time contracts (`PartTime`, `HalfTime`, `20Percent`)—exhibit structural cross-week coupling:
- **Contract Parameter Conflict:** Consider a `PartTime` contract with:
  - $\text{Total Assignments} \in [7, 15]$ over 4 weeks ($\approx 1.75$ to $3.75$ shifts/week).
  - $\text{Minimum Consecutive Working Days} = 5$.
  - $\text{Minimum Consecutive Days Off} = 5$.
- **The Single-Week Paradox:** Under an isolated single-week planning window, a naive target band of $[1, 4]$ shifts is mathematically impossible to fulfill ergonomically:
  - If the nurse is assigned $0$ shifts: they violate weekly minimum assignment pacing ($S_1$).
  - If the nurse is assigned $1$ to $4$ shifts: they violate minimum consecutive working days ($S_2$, penalty 30–120 pts).
  - If the nurse is assigned $5$ shifts: they exceed the naive weekly maximum ($S_1$, penalty 20 pts).
- **The Holistic Reality:** The true global optimum requires **multi-week cohort staggering**: the nurse should be completely idle ($0$ shifts) in Week 0 and Week 2, and work a complete legal 5-day block ($5$ shifts) in Week 1 and Week 3, perfectly satisfying all constraints across the 4-week horizon.

---

## 2. Dynamic Capability Hierarchy: Primary vs. Secondary Skills

To prevent skill cannibalization, Coralys establishes a dynamic, data-driven skill hierarchy based on fleet rarity.

### 2.1 Skill Rarity Function
For any scenario with nurse fleet $\mathcal{N}$ and skill alphabet $\mathcal{S}$:
$$\text{Rarity}(s) = \lvert \{ n \in \mathcal{N} \mid s \in \text{Skills}(n) \} \rvert, \quad \forall s \in \mathcal{S}$$

In canonical hospital scenarios (e.g., `n030w4` through `n120w4`), the fleet rarity strictly induces the clinical qualification ordering:
$$\text{Rarity}(\text{HeadNurse}) < \text{Rarity}(\text{Nurse}) < \text{Rarity}(\text{Caretaker}) < \text{Rarity}(\text{Trainee})$$
$$\implies \text{HeadNurse} \succ \text{Nurse} \succ \text{Caretaker} \succ \text{Trainee}$$

### 2.2 Primary Skill Specialization
For each employee $n \in \mathcal{N}$:
- **Primary Skill ($s_{\text{primary}}(n)$):** The highest / rarest capability possessed by nurse $n$:
  $$s_{\text{primary}}(n) = \arg\min_{s \in \text{Skills}(n)} \text{Rarity}(s)$$
- **Secondary Skills ($\mathcal{S}_{\text{sec}}(n)$):** All subordinate capabilities possessed by nurse $n$:
  $$\mathcal{S}_{\text{sec}}(n) = \text{Skills}(n) \setminus \{ s_{\text{primary}}(n) \}$$

### 2.3 The Primary Specialization Rostering Principle
When allocating demand slot $\text{Slot}(d, \sigma, s_{\text{req}})$:
1. **Tier 1 (Primary Qualification):** Filter eligible nurses to those for whom $s_{\text{primary}}(n) == s_{\text{req}}$.
2. **Tier 2 (Secondary Overflow Fallback):** Only if Tier 1 candidate pool is exhausted (or statutory contract limits prevent assignment) are secondary skill holders permitted to bid for the slot.

**Invariant:** A worker is never assigned to a secondary skill slot if another available worker possesses that skill as their primary qualification.

---

## 3. Capability Cohorts (Equivalence Classes)

Instead of scheduling individual nurses independently, the fleet $\mathcal{N}$ is partitioned into **Capability Cohorts**:

$$\mathcal{K}_{(c, s)} = \{ n \in \mathcal{N} \mid \text{Contract}(n) = c \;\land\; s_{\text{primary}}(n) = s \}$$

### 3.1 Properties of Capability Cohorts
1. **Ergonomic Homogeneity:** Every member in $\mathcal{K}_{(c, s)}$ is governed by identical statutory bounds:
   - Minimum / maximum working streak $[W_{\min}, W_{\max}]$.
   - Minimum / maximum off streak $[O_{\min}, O_{\max}]$.
   - Maximum working weekends $K_{\text{wend}}$.
   - Multi-week assignment quota $[A_{\min}, A_{\max}]$.
2. **Functional Interchangeability:** Any member of $\mathcal{K}_{(c, s)}$ can legally execute the primary line of work of any other member in the same cohort without altering skill coverage feasibility ($HC_2$).
3. **Intra-Cohort Kempe Rotations:** Swapping entire 7-day shift patterns between nurses $u, v \in \mathcal{K}_{(c, s)}$ produces $\Delta \text{Coverage} = 0$, guaranteeing strict objective monotonicity.

---

## 4. Multi-Week Holistic Duty Staggering

To eliminate the weekly horizon tunnel-vision penalty:

```
+---------------------------------------------------------------------------------------+
|                 MULTI-WEEK COHORT DUTY STAGGERING (PartTime / HalfTime)                |
+-------------------+-------------------+-------------------+---------------------------+
| Cohort Member     | Week 0            | Week 1            | Week 2            | Week 3|
+-------------------+-------------------+-------------------+---------------------------+
| Nurse 1 (Sub-A)   | Active (5-day blk)| Rest (0-day off)  | Active (5-day blk)| Rest  |
| Nurse 2 (Sub-A)   | Active (5-day blk)| Rest (0-day off)  | Active (5-day blk)| Rest  |
| Nurse 3 (Sub-B)   | Rest (0-day off)  | Active (5-day blk)| Rest (0-day off)  | Active|
| Nurse 4 (Sub-B)   | Rest (0-day off)  | Active (5-day blk)| Rest (0-day off)  | Active|
+-------------------+-------------------+-------------------+---------------------------+
| Weekly Supply     | 10 shifts         | 10 shifts         | 10 shifts         | 10 sh |
| Statutory Health  | 100% Legal Streaks| 100% Legal Streaks| 100% Legal Streaks| 100%  |
+-------------------+-------------------+-------------------+---------------------------+
```

### 4.1 Cohort Phasing Algorithm
For each cohort $\mathcal{K}$ where $W_{\min} > \text{WeeklyMax}(c)$:
1. Subdivide the cohort into $P = \lceil W_{\min} / \text{WeeklyMin}(c) \rceil$ alternating sub-groups.
2. In week $w$, activate sub-group $g = (w \pmod P)$.
3. Active members receive full continuous ergonomic blocks $[W_{\min}, W_{\max}]$; inactive members rest $[O_{\min}, O_{\max}]$.
4. Total cumulative assignments over horizon $W$ equal $W / P \times \text{BlockLength} \in [A_{\min}, A_{\max}]$, achieving exact zero penalty.

---

## 5. Integration into the Reasoning Engine (`CoralysInrcVndEngine`)

The capability cohort and primary skill specialization architecture is integrated across the 4-phase reasoning pipeline:

```
+----------------------------------------------------------------------------------------------------+
|                                    CORALYS REASONING PIPELINE                                      |
+----------------------------------------------------------------------------------------------------+
| Phase 1: Dynamic Cohort Partitioning & Skill Rarity Ranking                                        |
| - Compute fleet-wide skill rarity R(s) across all scenario nurses                                  |
| - Designate primary skill s_primary(n) and secondary skills S_sec(n) for each nurse                |
| - Group nurses into Capability Cohorts K(c, s_primary)                                             |
+----------------------------------------------------------------------------------------------------+
| Phase 2: Primary-Skill-First Constructive Seeding (InvertedContext)                                |
| - Sort demand slots: rarest primary skill first                                                    |
| - Slot matching: primary cohort candidates evaluated first; secondary candidates masked           |
| - Multi-week duty staggering applied to non-full-time cohorts                                      |
+----------------------------------------------------------------------------------------------------+
| Phase 3: Intra-Cohort Holistic Local Search (VND Ladder)                                           |
| - N1: Intra-Cohort Single-Day Swaps (exact skill & contract parity)                                |
| - N2: Cohort Weekend Harmonizer (harmonizes split weekends within cohort)                          |
| - N3: Intra-Cohort Multi-Day Kempe Block Swaps (2 to 5 days, ΔC = 0)                               |
| - N4: RCSP Dynamic Pricing on DAG with Primary Specialization Credits                              |
+----------------------------------------------------------------------------------------------------+
| Phase 4: Rolling Horizon Transition & Boundary Reconciliation                                      |
| - Sunday border lookahead accounts for cohort phasing into week w + 1                              |
| - Pacing targets evaluate remaining horizon capacity against cohort obligations                    |
+----------------------------------------------------------------------------------------------------+
```
