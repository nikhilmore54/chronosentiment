# Coralys Feasibility & Execution Qualification Framework
## GOV-009 — Platform Normative Document

*Applies to: CVRP, Workforce Scheduling, Crew Scheduling, Routing, and all future Coralys optimization domains.*
*Companion to GOV-008 (Benchmark Qualification Specification). These are orthogonal governance concerns:*
*GOV-008 answers "Can we trust the benchmark comparison?" GOV-009 answers "Is the problem feasible, and how should it be executed?"*
*Version: 1.2 — expanded to Execution Qualification Framework, 2026-07-08.*

---

## Normative Principle

> **The Feasibility & Execution Qualification Framework SHALL minimize unnecessary optimization
> effort by detecting impossible instances early, estimating optimization difficulty, and
> determining the most appropriate execution strategy before search begins.**
>
> Every qualification stage SHALL produce an **Execution Decision**, not merely a classification.
> The pipeline is: Qualification → Execution Planning → Optimization.
> Optimization is one stage in a larger decision pipeline, not the entry point.
>
> Coralys distinguishes four mathematically distinct states:
> - **Proven Infeasible** — a necessary condition violation (FC-2 through FC-4) or exact solver proof (FC-5 UNSAT). No optimization runs.
> - **Proven Feasible** — a solution satisfying every benchmark constraint has been verified. The certificate may be produced by Coralys or by an exact solver. No separate exact solver is required if Coralys itself produces a valid solution.
> - **Feasibility Undetermined** — all analytical tests passed; exact proof not attempted.
> - **Solver Failed** — optimizer exhausted its budget; feasibility unresolved.
>
> Attributing optimizer shortcomings to benchmark infeasibility, or vice versa, is a qualification error.

---

## §1 Mathematical Background

### 1.1 CVRP Feasibility Definition

A CVRP instance `(G=(V,E), depot=0, customers=1..n, demands d_i, capacity Q, fleet K)`
is **feasible** iff there exists a partition of customers into routes `R_1, R_2, ..., R_K` such that:

1. Every customer appears in exactly one route.
2. Every route starts and ends at the depot.
3. Route load ≤ Q for every route.
4. Every customer is reachable from the depot.
5. Every route is connected (no subtours).
6. Exactly K routes are used (or ≤ K if fleet minimization is permitted).

### 1.2 Necessary vs. Sufficient Conditions

| Class | Definition | Example |
|---|---|---|
| **Necessary condition** | Must hold for feasibility; violation proves infeasibility | Fleet capacity ≥ total demand |
| **Sufficient condition** | Guarantees feasibility if satisfied | Constructing an explicit valid solution |
| **Necessary and sufficient** | Equivalent to feasibility | Exact MILP/CP-SAT SAT result; or a verified valid solution |

**Critical:** Passing all known necessary conditions does **not** guarantee feasibility.
Capacity interactions are combinatorial. A bin-packing counterexample:

```
Q=16, K=3, demands=[10, 10, 10, 6, 6, 6]
Total demand = 48 ≤ 48 = K×Q  ← passes fleet capacity (NC1)
Each demand ≤ Q               ← passes individual demand (NC2)
K=3 ≥ ⌈48/16⌉=3              ← passes vehicle lower bound (NC5)
Yet no valid packing exists.  ← INFEASIBLE
```

Passing FC-1 through FC-4 **fails to disprove** feasibility. It does not increase the
probability of feasibility in a mathematically rigorous sense. "Undetermined" is the
correct classification, not "Likely Feasible."

### 1.3 Feasibility Certificates

A **certificate of feasibility** is a constructive proof: a solution satisfying every
constraint. A Coralys solution that passes constraint validation IS a mathematical
certificate of feasibility. No separate exact solver is required.

A **certificate of infeasibility** is a proof that no feasible solution exists:
a violated necessary condition, a bin-packing UNSAT (exact), or an exact solver UNSAT.

---

## §2 Execution Decisions per Stage

Every FC stage produces both a **feasibility result** and an **execution decision**.
The execution decision is normative — the optimizer SHALL respect it.

| Stage | Result | Execution Decision |
|---|---|---|
| FC-1 FAIL | `STRUCTURAL_INVALID` | **Abort immediately** — do not run optimizer |
| FC-2.5 FAIL | `BENCHMARK_INVALID` | **Skip benchmark qualification** — may still optimize if instance is structurally valid |
| FC-2 FAIL | `PROVEN_INFEASIBLE_FC2` | **Abort immediately** — mathematically impossible |
| FC3-Heuristic FAIL | `LIKELY_INFEASIBLE` | **Reduce budget or switch strategy** — signal only, not proof |
| FC3-Exact FAIL | `PROVEN_INFEASIBLE_FC3` | **Abort immediately** |
| FC-4 FAIL | `PROVEN_INFEASIBLE_FC4` | **Abort immediately** |
| FC-5 SAT | `PROVEN_FEASIBLE_FC5` | **Store feasibility certificate** — optimization optional |
| FC-5 UNSAT | `PROVEN_INFEASIBLE_FC5` | **Abort immediately** |
| FC-6 | `ExecutionPlan` | **Select profile, budget, escalation policy** |

---

## §3 Feasibility Certification Levels

### FC-1 — Structural Validation

**Complexity:** O(n) | **Execution Decision on FAIL:** Abort
**Tests:**
- [ ] Graph is connected (depot reachable from all customers)
- [ ] Distance matrix is complete (no undefined entries)
- [ ] All demands are non-negative integers
- [ ] Depot is correctly identified
- [ ] No duplicate customer IDs
- [ ] Customer count matches declared `n`
- [ ] Capacity is positive

**Outcome on failure:** `STRUCTURAL_INVALID` — instance is malformed. Not an optimizer failure.
**Outcome on pass:** Proceed to FC-2.5.

---

### FC-2.5 — Benchmark Consistency

**Complexity:** O(1) | **Execution Decision on FAIL:** Skip benchmark qualification
**Tests:**
- [ ] Vehicle count present and positive
- [ ] Vehicle capacity present and positive
- [ ] BKS present (or explicitly marked as absent)
- [ ] Customer numbering valid (1..n, no gaps)
- [ ] Coordinate system valid (EUC_2D, GEO, EXPLICIT, etc.)
- [ ] Depot defined and within customer set
- [ ] Distance metric supported by Coralys
- [ ] Metadata internally consistent (no contradictory fleet size, capacity, or BKS)

**Outcome on failure:** `BENCHMARK_INVALID` — the benchmark metadata is flawed.
Distinct from `STRUCTURAL_INVALID` (instance graph may be perfectly valid).
CMT/Tai BKS provenance issues and X-family missing BKS belong here.

**Outcome on pass:** Proceed to FC-2.

---

### FC-2 — Capacity Validation

**Complexity:** O(n) | **Execution Decision on FAIL:** Abort
**Tests:**
- [ ] **NC1 Fleet Capacity:** `Σ d_i ≤ K × Q`
- [ ] **NC2 Individual Demand:** `d_i ≤ Q` for all i
- [ ] **NC5 Vehicle Lower Bound:** `K ≥ ⌈Σ d_i / Q⌉`

**Outcome on failure:** `PROVEN_INFEASIBLE_FC2` — mathematically impossible. No optimizer can solve it.
Runtime saved: O(n) instead of 600 seconds.

**Certificate format:**
```
FC-2 Certificate
Instance:        <name>
Total demand:    D
Fleet capacity:  K × Q
Min vehicles:    ⌈D/Q⌉
Available:       K
NC1:             PASS | FAIL (D=X > KQ=Y)
NC2:             PASS | FAIL (customer i: d_i=X > Q=Y)
NC5:             PASS | FAIL (K=X < ⌈D/Q⌉=Y)
Outcome:         PROVEN_INFEASIBLE_FC2 | PASSED_FC2
```

---

### FC-3 — Bin Packing Relaxation

**Complexity:** NP-hard in general; practical for n ≤ 200 | **Execution Decision on FAIL:** Abort (Exact) or Reduce budget (Heuristic)

**FC-3 has two sub-levels:**

| Sub-level | Method | Failure outcome | Execution Decision |
|---|---|---|---|
| **FC3-Heuristic** | First-Fit Decreasing | `LIKELY_INFEASIBLE` | Reduce budget; switch to feasibility-focus profile |
| **FC3-Exact** | Exact bin-packing solver | `PROVEN_INFEASIBLE_FC3` | Abort immediately |

Only FC3-Exact may produce `PROVEN_INFEASIBLE`. FC3-Heuristic failure is a signal, not a proof.

**Outcome on pass (either sub-level):** `PASSED_FC3` — necessary condition satisfied; feasibility not proven.

---

### FC-4 — Capacity-Cut and Flow Analysis

**Complexity:** Polynomial to exponential | **Execution Decision on FAIL:** Abort

**Asymmetric outcomes:**
- **Pass (no violated cut found):** No conclusion. Feasibility remains undetermined.
- **Fail (violated cut found):** `PROVEN_INFEASIBLE_FC4` — abort immediately.

FC-4 cannot prove feasibility. It can only prove infeasibility when a violated cut is found.
*Optional: triggered only for instances where FC-3 passes but optimizer consistently produces `SOLVER_FAILED`.*

---

### FC-5 — Exact Feasibility Solver

**Complexity:** Exponential worst case; practical for n ≤ 200 | **Execution Decision:** Store certificate or Abort

**Method:** MILP or CP-SAT with objective = 0 (feasibility only).

**Outcomes:**
- `SAT` → **`PROVEN_FEASIBLE_FC5`** — store feasibility certificate; optimization optional
- `UNSAT` → **`PROVEN_INFEASIBLE_FC5`** — abort; mathematical proof of infeasibility

This is the **gold standard**. FC-5 is necessary and sufficient.
*Run post-campaign on all `SOLVER_FAILED` instances.*

---

### FC-6 — Execution Qualification

**Complexity:** O(1) after model training | **Execution Decision:** Select profile, budget, escalation policy

**Purpose:** Determine the most appropriate execution strategy before optimization begins.

**Outputs:**
```
FC-6 Execution Qualification
Instance:              <name>
Difficulty Index:      74 / 100
Difficulty Class:      HARD
Fleet Utilization:     94.2%
Capacity Slack:        5.8%
Demand Variance:       HIGH
Spatial Dispersion:    CLUSTERED
Estimated Runtime:     82 ± 14 s
Confidence:            91%
Execution Profile:     INTENSIVE
Recommended Config:    pop=300, gen=500, enhanced_repair=true
Expected Convergence:  generation 210
Escalation Policy:     FC-5 if SOLVER_FAILED
```

---

## §4 Difficulty Index

The Difficulty Index (DI ∈ [0, 100]) is a composite score computed from instance
characteristics before optimization. Higher DI = harder instance.

### DI Contributors

| Factor | Weight | Description |
|---|---|---|
| Fleet utilization `Σd_i / (K×Q)` | High | Near 1.0 = very tight; near 0.5 = slack |
| Customer count `n` | Medium | More customers = harder routing |
| Vehicle count `K` | Medium | Fewer vehicles = harder packing |
| Demand variance `σ(d_i) / mean(d_i)` | Medium | High CV = harder bin packing |
| Spatial dispersion | Medium | Clustered = easier; dispersed = harder |
| Avg nearest-neighbor distance | Low | Proxy for route structure difficulty |
| Capacity slack per vehicle `(K×Q - Σd_i) / K` | High | Less slack = harder |
| Historical campaign performance | High | Available after first campaign run |

### DI Formula (Phase 1 — analytical, no training data required)

```
utilization  = Σd_i / (K×Q)
demand_cv    = σ(d_i) / mean(d_i)          # coefficient of variation
size_factor  = n / 200.0                    # normalized to max campaign size
slack_factor = 1.0 - utilization

DI = 100 × (
    0.40 × utilization       +
    0.25 × min(demand_cv, 1) +
    0.20 × size_factor       +
    0.15 × (1.0 - slack_factor)
)
```

### DI Classification and Execution Profiles

| DI | Class | Execution Profile | Optimizer Strategy |
|---|---|---|---|
| 0–25 | Easy | FAST_QUALIFICATION | Reduced population/generations; standard repair |
| 26–50 | Moderate | STANDARD | Default production configuration |
| 51–75 | Hard | INTENSIVE | Larger population, stronger repair, more diversity |
| 76–100 | Extreme | MAXIMUM | Maximum budget, enhanced local search, escalate to FC-5 if SOLVER_FAILED |

### DI Examples (Campaign v1.1 data)

| Instance | n | K | Utilization | DI (est.) | Class |
|---|---|---|---|---|---|
| A-n32-k5 | 31 | 5 | ~0.62 | ~18 | Easy |
| B-n57-k7 | 56 | 7 | ~0.71 | ~35 | Moderate |
| CMT9 | 150 | 14 | ~0.88 | ~72 | Hard |
| Tai150a | 150 | 12 | ~0.94 | ~91 | Extreme |

---

## §5 Execution Profiles

Rather than exposing raw genetic algorithm parameters, Coralys defines execution profiles
selected automatically by FC-6.

| Profile | Intended Use | Population | Generations | Repair | Local Search |
|---|---|---|---|---|---|
| `FAST_QUALIFICATION` | Quick validation, regression testing | 100 | 75 | Standard | Disabled |
| `STANDARD` | Default production qualification | 200 | 150 | Standard | Enabled |
| `INTENSIVE` | Difficult benchmark instances | 300 | 300 | Enhanced | Enabled |
| `MAXIMUM` | Extreme instances, capability boundary | 500 | 500 | Maximum | Enabled |
| `FEASIBILITY_FOCUS` | FC3-Heuristic FAIL signal | 150 | 100 | Maximum | Disabled |
| `EXACT_ESCALATION` | Post-SOLVER_FAILED exact solver | — | — | — | FC-5 invoked |

*Phase 1: profiles are defined but not yet wired to optimizer config. Campaign v1.2 uses STANDARD for all instances.*

---

## §6 Feasibility Outcome Classification

Every Coralys benchmark instance receives one of six feasibility outcomes:

| Outcome | Code | Meaning | Evidence Required |
|---|---|---|---|
| **Proven Feasible** | `PROVEN_FEASIBLE` | A solution satisfying every constraint has been verified | Valid Coralys solution OR FC-5 SAT |
| **Proven Infeasible** | `PROVEN_INFEASIBLE` | Mathematical proof exists | Specific violated condition (FC-2/FC3-Exact/FC-4/FC-5 UNSAT) |
| **Feasibility Undetermined** | `FEASIBILITY_UNDETERMINED` | Passed FC-1 through FC-4; exact proof not attempted | FC-3 pass record |
| **Solver Failed** | `SOLVER_FAILED` | Optimizer exhausted budget; feasibility unresolved | Runtime log, generation count |
| **Benchmark Invalid** | `BENCHMARK_INVALID` | Metadata or registry error | FC-2.5 failure record |
| **Structural Invalid** | `STRUCTURAL_INVALID` | Instance graph is malformed | FC-1 failure record |

### Mapping to Qualification Outcomes (GOV-008)

| Feasibility Outcome | Qualification Outcome |
|---|---|
| `PROVEN_FEASIBLE` (Coralys solution) | Qualified / NearOptimal / etc. per gap |
| `PROVEN_INFEASIBLE` | Not Comparable — instance property, not optimizer failure |
| `FEASIBILITY_UNDETERMINED` + `SOLVER_FAILED` | Under Investigation — optimizer shortcoming, not instance property |
| `BENCHMARK_INVALID` | Not Comparable — registry issue |
| `STRUCTURAL_INVALID` | Invalid |

---

## §7 Feasibility Confidence Ladder

| Level | Status | Meaning |
|---|---|---|
| **F0** | Invalid | Malformed benchmark (`STRUCTURAL_INVALID` or `BENCHMARK_INVALID`) |
| **F1** | Structurally Valid | Instance parsed correctly; FC-1 passed |
| **F2** | Capacity Valid | Necessary capacity conditions satisfied; FC-2 passed |
| **F3** | Benchmark Consistent | Metadata and benchmark semantics verified; FC-2.5 passed |
| **F4** | Feasible Solution Verified | Coralys (or another solver) produced and validated a feasible solution |
| **F5** | Exact Proof | Exact solver proved SAT or UNSAT; definitive certificate exists |

Every campaign instance SHALL report its Feasibility Confidence Level (F0–F5).
F4 is the normal outcome for instances where Coralys finds a solution.
F5 is reserved for post-campaign exact solver runs on `SOLVER_FAILED` instances.

---

## §8 Execution Pipeline

Every campaign run SHALL execute this pipeline:

```
Parse Instance
      │
      ▼
FC-1: Structural Validation
      │ FAIL → STRUCTURAL_INVALID (F0) → ABORT
      ▼
FC-2.5: Benchmark Consistency
      │ FAIL → BENCHMARK_INVALID (F0) → SKIP BENCHMARK QUALIFICATION
      ▼
FC-2: Capacity Validation
      │ FAIL → PROVEN_INFEASIBLE_FC2 (F2) → ABORT
      ▼
FC-3: Bin Packing Relaxation
      │ FC3-Exact FAIL → PROVEN_INFEASIBLE_FC3 (F2) → ABORT
      │ FC3-Heuristic FAIL → LIKELY_INFEASIBLE → FEASIBILITY_FOCUS profile
      ▼
[FC-4: Capacity Cuts — optional, triggered by prior SOLVER_FAILED]
      │ FAIL → PROVEN_INFEASIBLE_FC4 (F2) → ABORT
      ▼
FC-6: Execution Qualification
      │ → Difficulty Index computed
      │ → Execution Profile selected
      │ → Runtime predicted
      │ → Escalation policy set
      ▼
Optimization (using selected profile)
      │
      ▼
Solution found?
      │
      ├── YES → Validate solution (all constraints)
      │           │
      │           ├── Valid   → PROVEN_FEASIBLE (F4, gap computed)
      │           └── Invalid → SOLVER_FAILED (constraint violation in reported solution)
      │
      └── NO  → SOLVER_FAILED (F1/F2/F3, budget exhausted)
                    │
                    └── [If escalation policy = FC-5] → Exact solver post-campaign
```

---

## §9 Feasibility & Execution Certificate Format

Every instance in a campaign report SHALL include a certificate:

```
Feasibility & Execution Certificate
Instance:              <name>
FC-1 Structural:       PASS
FC-2.5 Benchmark:      PASS
FC-2 Capacity:         PASS  (D=12345, KQ=15000, K_min=3, K=4)
FC-3 Bin Packing:      PASS  (FC3-Heuristic: 3 bins of Q=3750 sufficient)
FC-4 Cap Cuts:         NOT_RUN
FC-5 Exact:            NOT_RUN
FC-6 Execution:        DI=74 (HARD), est=82±14s, profile=INTENSIVE, escalate=FC-5
Confidence Level:      F3 (Benchmark Consistent)
Optimizer Result:      SOLVER_FAILED (dist=1000000.0, 611391ms)
Feasibility Status:    FEASIBILITY_UNDETERMINED
Execution Decision:    SOLVER_FAILED → escalate to FC-5 post-campaign
```

---

## §10 Application to Campaign v1.1 X-Family Results

| Instance | Customers | Vehicles | Optimizer Result | Current Classification | Action |
|---|---|---|---|---|---|
| X-n172-k51 | 171 | 51 | INFEASIBLE (dist=1000000.0) | `SOLVER_FAILED` pending FC-2 | Run FC-2 in v1.2 |
| X-n181-k23 | 180 | 23 | best=25656 [No-ref] | `PROVEN_FEASIBLE` (F4) | No BKS → `BENCHMARK_INVALID` (FC-2.5) |
| X-n186-k15 | 185 | 15 | best=24797 [No-ref] | `PROVEN_FEASIBLE` (F4) | No BKS → `BENCHMARK_INVALID` (FC-2.5) |
| X-n190-k8  | 189 | 8  | best=17569 [No-ref] | `PROVEN_FEASIBLE` (F4) | No BKS → `BENCHMARK_INVALID` (FC-2.5) |

**Key distinction:** X-n181-k23, X-n186-k15, X-n190-k8 are `PROVEN_FEASIBLE` — Coralys found valid
solutions. These are NOT optimizer failures. The absence of a BKS is a `BENCHMARK_INVALID`
(FC-2.5) issue, not a feasibility issue.

---

## §11 Implementation Roadmap

### Phase 1 — FC-1, FC-2.5, FC-2 ✅ Complete

Implemented in [`adapters/cvrp/src/qualification/feasibility.rs`](adapters/cvrp/src/qualification/feasibility.rs).
Exposed via [`adapters/cvrp/src/lib.rs`](adapters/cvrp/src/lib.rs) as `cvrp::qualification`.
Build: clean (0 errors, 2026-07-08).
**Pending:** wire into [`campaign.rs`](adapters/cvrp/src/bin/campaign.rs) pre-optimization call site.

### Phase 2 — FC-3 Bin Packing (Campaign v1.3)

FC3-Heuristic (First-Fit Decreasing) is fast and sufficient as a signal.
FC3-Exact for n ≤ 200 is tractable and produces `PROVEN_INFEASIBLE_FC3`.

### Phase 3 — FC-5 Exact Solver (Post-Campaign v1.2)

Run CP-SAT or HiGHS on all `SOLVER_FAILED` instances from Campaign v1.1/v1.2.
Produces definitive `PROVEN_INFEASIBLE_FC5` or confirms `FEASIBILITY_UNDETERMINED`.
Results feed into QDR v2.0 and update Confidence Level to F5.

### Phase 4 — FC-6 Execution Qualification (Post-Campaign v1.2 data)

Implementation order:
1. Compute Difficulty Index (DI) analytically — no training data required.
2. Fit runtime prediction model on Campaign v1.1/v1.2 data.
3. Implement execution profile selection based on DI class.
4. Wire profile into optimizer config in `campaign.rs`.
5. Integrate FC-6 output into campaign log and report.

---

## §12 Qualification History

| Version | Date | Author | Change |
|---|---|---|---|
| 1.0 | 2026-07-08 | Coralys Engineering | Initial specification — drafted from Campaign v1.1 X-family evidence |
| 1.1 | 2026-07-08 | Coralys Engineering | OR-community review: FC-2.5 added; FC-3 split heuristic/exact; FC-4 asymmetric; "Likely Feasible" → "Feasibility Undetermined"; BENCHMARK_INVALID added; Feasibility Confidence Ladder (F0–F5); solution validation step; Proven Feasible clarified |
| 1.2 | 2026-07-08 | Coralys Engineering | Expanded to Execution Qualification Framework: each FC stage now produces an Execution Decision; FC-6 Execution Qualification added; Difficulty Index (DI) defined with formula and classification; Execution Profiles defined (FAST_QUALIFICATION / STANDARD / INTENSIVE / MAXIMUM / FEASIBILITY_FOCUS / EXACT_ESCALATION); pipeline updated to Qualification → Execution Planning → Optimization; Phase 1 implementation complete (qualification module builds clean) |

---

*This document is normative. All Coralys campaign reports SHALL classify feasibility
outcomes according to this framework. "INFEASIBLE" in a campaign log means
`SOLVER_FAILED` until FC-2 or FC-5 evidence is produced.*