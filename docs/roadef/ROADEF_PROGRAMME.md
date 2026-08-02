# ROADEF Research Programme

**Programme:** EURO/ROADEF 2026 Challenge — T-Adaptive Segment Routing  
**Status:** Active  
**Version:** 1.0  
**Date:** 2026-08-02  

---

## 1. Programme Context

The ROADEF programme is a formal research programme within the Coralys platform. It is not an adapter, not an experiment series, and not a one-off submission effort. It is a structured research programme that produces:

1. External validation of the Coralys optimisation engine on a recognised industrial benchmark.
2. Evidence that feeds back into the Coralys platform (not ROADEF-specific modifications).
3. A publishable case study demonstrating the platform on a demanding real-world problem.

The programme sits within the Coralys platform hierarchy:

```
Coralys Platform
        │
        ├── UltraCrew
        ├── CVRP Research
        ├── ROADEF Challenge          ← this programme
        └── Future Domains
```

### 1.1 Governance Foundation

This programme is enabled by the completion of RR1–RR4:

| Milestone | Outcome |
|-----------|---------|
| RR1 Repository Census | 26 workspace members inventoried |
| RR2 Structural Analysis | 468 source files mapped; 9 orphans classified |
| RR3 Evolutionary Lineage | 64 experiment binaries across 3 research streams |
| RR4 Governance Baseline | 98 governed artefacts; lifecycle decisions recorded |

The repository structure will not change underneath this programme. Research can proceed without governance disruption.

---

## 2. Benchmark Manifest

**This section is the single reference point for all experiments.**

### 2.1 Challenge Edition

| Field | Value |
|-------|-------|
| Challenge | EURO/ROADEF 2026 |
| Problem | T-Adaptive Segment Routing |
| Organiser | Orange SA |
| Repository | `adapters/roadef/repo/challenge-roadef-2026-main/` |
| Rules document | `doc/Challenge_Orange_ROADEF_2026_Rules.pdf` |
| Problem document | `doc/Challenge_Orange_ROADEF_2026_Subject.pdf` |

### 2.2 Instance Sets

| Dataset | Release | Instances | Horizon | Status |
|---------|---------|-----------|---------|--------|
| Dataset A | 2026-03-06 | 20 (setA-01 to setA-20) | ≤ 2 time slots | Available |
| Dataset B | 2026-06-15 | TBD | Unbounded | Pending |
| Dataset C | 2026-10-05 | TBD | Highest complexity | Pending |

### 2.3 Objective Function

Minimise over all time slots `t`:

```
obj = Σ_t [ MLU(t) + inv_load_cost(t) ]
```

Where:
- `MLU(t)` = Maximum Link Utilisation at time slot `t`
- `inv_load_cost(t)` = Σ over links `l` of `1/(1 - sat(l)) - 1` for `sat(l) > 0`
- A solution is **invalid** if: segment count violated, budget exceeded, or any demand disconnected
- `obj = ∞` is valid but scores poorly

### 2.4 Constraints

| Constraint | Description |
|------------|-------------|
| `max_segments` | SR path waypoints + 1 ≤ max_segments (typically 6) |
| Budget | Σ_d dist(path_d_t, path_d_{t-1}) ≤ budget(t) for each t > 0 |
| Connectivity | Every demand must be routable at every time slot |

### 2.5 Budget Distance Metric

`dist(path_A, path_B)` = symmetric difference of edge-transition sets.  
`dist(uninitialized, explicit(len=N))` = N.  
`dist(uninitialized, uninitialized)` = 0.

**Critical implication:** Emitting t=0 srpaths without matching t=1 srpaths costs `path_length` per demand in budget. The shared-path strategy (same waypoints for both slots) guarantees budget cost = 0.

### 2.6 Scoring Methodology

The official checker (`checker/src/`) computes the objective. Our internal evaluator (`adapters/roadef/src/evaluator.rs`) replicates the checker logic exactly (validated against checker output for setA-01).

### 2.7 Runtime Limits

Per the challenge rules: solver runtime is not formally constrained for Dataset A. For competition submissions, solutions must be reproducible. Document wall-clock time per instance in all experiment reports.

### 2.8 Hardware Specification

All experiments must record:

```
CPU: [model]
Cores used: [n]
RAM: [GB]
OS: [version]
Rust toolchain: [version]
```

### 2.9 Reproducibility Requirements

- All experiments must be reproducible from a single `cargo run` command.
- Random seeds must be fixed and documented.
- Solution files must be deterministic given the same seed and instance.

---

## 3. Baseline v1.0

**Established:** 2026-08-02  
**Commit:** `ec4d3821`  
**Solver:** `campaign_engine` — greedy load-balanced Dijkstra with shared-path strategy

### 3.1 Solver Description

The baseline solver uses:
- Load-aware Dijkstra routing (penalises saturated links exponentially above 80% utilisation)
- Demands processed in descending volume order
- Shared-path strategy: same waypoints emitted for t=0 and t=1 (budget cost = 0)
- Automatic fallback to empty solution when ours is worse

### 3.2 Dataset A Results

| Instance | Nodes | Links | Demands | Budget | Our obj | Empty obj | Decision |
|----------|-------|-------|---------|--------|---------|-----------|----------|
| setA-01 | 20 | 80 | 40 | 51 | 260.32 | ∞ | ours |
| setA-02 | 30 | 150 | 45 | 63 | ∞ | ∞ | ours |
| setA-03 | 50 | 250 | 20 | 53 | ∞ | ∞ | ours |
| setA-04 | 50 | 250 | 200 | 44 | 70.77 | ∞ | ours |
| setA-05 | 100 | 396 | 100 | 1 | ∞ | 72,329 | empty |
| setA-06 | 100 | 500 | 500 | 13 | ∞ | ∞ | ours |
| setA-07 | 100 | 500 | 800 | 90 | 204.97 | ∞ | ours |
| setA-08 | 150 | 654 | 200 | 13 | ∞ | ∞ | ours |
| setA-09 | 150 | 750 | 200 | 18 | 153.51 | ∞ | ours |
| setA-10 | 150 | 966 | 1000 | 1 | 96.24 | ∞ | ours |
| setA-11 | 200 | 1000 | 400 | 89 | ∞ | ∞ | ours |
| setA-12 | 200 | 898 | 400 | 13 | ∞ | ∞ | ours |
| setA-13 | 200 | 1000 | 2000 | 12 | 89.19 | 986,957 | ours |
| setA-14 | 250 | 1108 | 600 | 13 | ∞ | ∞ | ours |
| setA-15 | 250 | 1250 | 600 | 54 | 223.50 | ∞ | ours |
| setA-16 | 250 | 1452 | 4800 | 13 | 127.03 | 3,355,568 | ours |
| setA-17 | 300 | 1270 | 2000 | 1 | ∞ | ∞ | ours |
| setA-18 | 300 | 1500 | 2000 | 89 | 799,166 | 799,169 | ours |
| setA-19 | 300 | 1998 | 6000 | 13 | 159.42 | 5,592,518 | ours |
| setA-20 | 400 | 2000 | 6000 | 90 | 447.86 | 1,525,646 | ours |

**Summary:** 11 instances with finite objective beating empty solution. 8 instances where both approaches give ∞ (inherently congested). 1 instance correctly falls back to empty (budget=1 prevents any useful re-routing).

### 3.3 Known Weaknesses

1. **ECMP mismatch:** The solver tracks flow based on the exact Dijkstra path, but the evaluator uses ECMP routing between waypoints. This causes the solver to underestimate congestion on some instances, leading to `obj=inf` on instances that should be solvable.
2. **No t=1 adaptation:** The shared-path strategy sacrifices t=1 quality for budget safety. Instances with large traffic changes between slots (setA-05, setA-10, setA-17) cannot benefit from re-routing.
3. **Greedy ordering:** Demands are processed in volume order without backtracking. High-volume demands may block good paths for many smaller demands.
4. **No multi-path diversity:** Only one path per demand is considered. ECMP-aware routing would allow deliberate load splitting.

---

## 4. Experimental Programme

Research questions are numbered RP-4xx. Each produces a measurable result against Baseline v1.0.

### RP-401 — ECMP-Aware Flow Estimation

**Question:** Can we eliminate the ECMP mismatch by simulating ECMP flow during path selection?

**Hypothesis:** If the solver uses the same ECMP routing logic as the evaluator to estimate link loads, the `obj=inf` instances will become solvable.

**Approach:**
- Replace the greedy flow tracker with a call to `evaluator.compute_loads()` after each demand assignment
- Accept the higher computational cost in exchange for accurate load estimates
- Measure: number of instances with finite objective, objective improvement on currently-inf instances

**Expected binary:** `src/bin/rp401_ecmp_aware.rs`

---

### RP-402 — Budget-Aware t=1 Adaptation

**Question:** Can we improve t=1 quality while respecting the budget constraint?

**Hypothesis:** For instances with budget > 0, selectively re-routing the demands with the largest traffic change between t=0 and t=1 will reduce t=1 objective without violating the budget.

**Approach:**
- After computing the shared path, identify demands where `|v[1] - v[0]|` is largest
- Re-route those demands for t=1 only, counting budget cost
- Stop when budget is exhausted
- Measure: objective improvement on setA-05, setA-10, setA-17 (budget=1 instances)

**Expected binary:** `src/bin/rp402_budget_adapt.rs`

---

### RP-403 — Iterative Load Balancing

**Question:** Does iterating the greedy assignment (re-routing demands that ended up on saturated links) improve the objective?

**Hypothesis:** A second pass that re-routes demands whose assigned paths are now saturated will reduce `obj=inf` instances.

**Approach:**
- After the first greedy pass, identify demands routed through links with sat > 0.9
- Re-route those demands using updated link saturations
- Repeat for up to K iterations
- Measure: convergence rate, objective improvement, runtime

**Expected binary:** `src/bin/rp403_iterative_lb.rs`

---

### RP-404 — Coralys MOGA Integration

**Question:** Can the existing Coralys MOGA engine improve on the greedy baseline?

**Hypothesis:** A population-based search using the MOGA engine with SR path assignment as the genome will find better solutions on large instances.

**Approach:**
- Define genome as a vector of waypoint assignments (one per demand)
- Use `evaluator.evaluate_solution()` as the fitness function
- Initialise population from the greedy baseline
- Run for a fixed time budget (e.g. 60 seconds per instance)
- Measure: objective improvement, runtime, population diversity

**Expected binary:** `src/bin/rp404_moga_solver.rs`

---

### RP-405 — Large Neighbourhood Search

**Question:** Can LNS with destroy/repair operators improve on the greedy baseline?

**Hypothesis:** Destroying and repairing subsets of demand assignments will escape local optima that the greedy solver gets stuck in.

**Approach:**
- Start from the greedy baseline solution
- Destroy operator: remove waypoints for K randomly selected demands
- Repair operator: re-route removed demands using load-aware Dijkstra
- Accept if objective improves
- Measure: objective improvement, convergence, operator effectiveness

**Expected binary:** `src/bin/rp405_lns.rs`

---

### RP-406 — Hyper-Heuristic Operator Selection

**Question:** Can adaptive operator selection (using Coralys memory structures) improve LNS performance?

**Hypothesis:** Tracking which destroy/repair operator combinations succeed on which instance types will improve operator selection over time.

**Approach:**
- Extend RP-405 with a Coralys vault tracking operator success/failure rates
- Use pressure-guided selection to prefer operators with lower failure rates
- Measure: improvement over RP-405, vault convergence rate

**Expected binary:** `src/bin/rp406_hyper_lns.rs`

---

### RP-407 — Hybrid Exact Subproblem

**Question:** Can solving a small exact subproblem (e.g. single-commodity flow for the most congested link) improve the overall solution?

**Hypothesis:** Identifying the bottleneck link and solving the routing problem for demands that use it exactly will reduce MLU more effectively than heuristic re-routing.

**Approach:**
- Identify the link with highest saturation
- Collect all demands routed through it
- Solve the re-routing problem for those demands exactly (small enough for exhaustive search)
- Measure: MLU reduction, runtime, scalability

**Expected binary:** `src/bin/rp407_hybrid_exact.rs`

---

## 5. Executable Classification

All ROADEF executables are classified into three tiers. Only one executable may occupy the Competition Submission tier at any time.

```
Research
    ↓ (evidence threshold met)
Candidate
    ↓ (validated against all instances, reproducible)
Competition Submission
```

| Tier | Naming | Lifecycle | Current |
|------|--------|-----------|---------|
| Research | `rp4xx_*.rs` | Experiment → Archive | RP-401 through RP-407 (planned) |
| Candidate | `candidate_*.rs` | Candidate → Promote or Archive | None yet |
| Competition Submission | `coralys-roadef-submit` | Single binary, versioned | None yet (baseline is `campaign_engine`) |

### 5.1 Promotion Criteria

A Research binary becomes a Candidate when:
- It beats Baseline v1.0 on ≥ 15 of 20 Dataset A instances
- It produces no regressions on instances where the baseline has finite objective
- Runtime is documented and within acceptable bounds

A Candidate becomes the Competition Submission when:
- It beats the previous Competition Submission on the full instance set
- It is reproducible from a clean build
- It has been validated against the official checker

### 5.2 Archive Policy

Research binaries that do not meet the promotion threshold are archived (not deleted). They remain as evidence of the research lineage. This follows the same policy as the CVRP and UltraCrew experiment archives.

---

## 6. Evidence Feedback to Coralys Platform

Each research programme produces platform-level evidence. ROADEF evidence targets:

| Evidence | Platform Component | Expected RP |
|----------|--------------------|-------------|
| ECMP-aware load estimation | `coralys-core` routing module | RP-401 |
| Budget-constrained re-routing | `coralys-planning` | RP-402 |
| Iterative load balancing | `coralys-core` | RP-403 |
| MOGA on network routing | `coralys-moga` | RP-404 |
| LNS operators for routing | `coralys-planning` | RP-405 |
| Hyper-heuristic selection | `coralys-ecology` | RP-406 |

Evidence that generalises beyond ROADEF should be promoted to the platform. Evidence that is ROADEF-specific remains in the adapter.

---

## 7. Programme Governance

| Role | Responsibility |
|------|---------------|
| Programme Owner | Defines research questions, approves promotion decisions |
| Solver Engineer | Implements RP binaries, maintains `campaign_engine` |
| Platform Engineer | Integrates generalised evidence into Coralys platform |

### 7.1 Amendment Log

| Version | Date | Change |
|---------|------|--------|
| 1.0 | 2026-08-02 | Initial programme document. Baseline v1.0 established from `campaign_engine` (commit `ec4d3821`). |