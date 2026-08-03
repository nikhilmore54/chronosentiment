# ROADEF Research Programme

**Programme:** EURO/ROADEF 2026 Challenge — T-Adaptive Segment Routing
**Status:** Active
**Version:** 1.5
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

Every RP must produce a standard evidence record before its result is considered final:

| Field | Description |
|-------|-------------|
| Research Question | What hypothesis is being tested? |
| Baseline | Previous best solver / commit |
| Metric | Official ROADEF objective (sum MLU + inv_load_cost) |
| Result | Improvement / regression per instance |
| Runtime | Wall-clock time per instance |
| Statistical Confidence | Multiple runs with fixed seeds if stochastic |
| Platform Impact | Generalisable to Coralys beyond ROADEF? |
| Decision | Promote / Archive / Continue |

---

### RP-000 — Budget Semantics Validation *(completed)*

**Research Finding:** Shared SR paths eliminate transition-budget expenditure by construction.

The budget distance metric charges `dist(uninitialized, explicit(len=N)) = N` per demand when t=0 has an explicit srpath but t=1 is uninitialized. Emitting identical waypoints for both t=0 and t=1 makes `dist(explicit_A, explicit_A) = 0`, guaranteeing zero budget cost for all demands regardless of path length or instance size.

This is a structural insight into the problem formulation. It establishes the correct baseline interpretation of the challenge rules. Any heuristic improvement built without this understanding would have been built on an invalid budget model.

**Evidence:** `campaign_engine.rs` (commit `ec4d3821`). All 20 Dataset A instances produce valid solutions. Budget constraint is never violated.

**Platform Impact:** The budget distance metric (`SrPathBit::dist`) is a general transition-cost model applicable to any multi-period routing problem. The shared-path strategy is a general technique for zero-cost initialisation in budget-constrained re-routing problems.

**Decision:** Archived as foundational finding. Informs all subsequent RPs.

---

### RP-401 — ECMP-Aware Flow Estimation *(🔒 FROZEN 2026-08-02)*

**Status:** Complete. All four stages executed 20/20. RP-401 is frozen.

**Scientific conclusion:** The primary bottleneck in the baseline solver was modelling fidelity. Correcting the ECMP load model (RP-401C) produced substantially larger improvements than introducing oracle-guided candidate selection (RP-401D). After model correction, search quality became the dominant remaining source of improvement.

| Stage | Outcome | Key result |
|-------|---------|------------|
| RP-401A | ✅ Oracle verified | `compute_loads()` matches official checker |
| RP-401B | ✅ Divergence quantified | Heuristic error: (k−1)/k on k-way ECMP |
| RP-401C | ✅ 13/20 improved, 0 regressed | +2,512,099 obj; 8 ∞→finite transitions |
| RP-401D | ✅ 15/20 finite | +2,584,407 obj vs empty; mixed vs RP-401C |

**Capabilities promoted:**
- ECMP-aware incremental load estimation: C1 → **C2** (benchmark validated)
- Oracle-guided constructive routing: C1 → **C2** (benchmark validated)
- Oracle-guided candidate selection: remains **C1** (exploratory evidence only)

**Full evidence:** [`RP401_FINAL_REPORT.md`](RP401_FINAL_REPORT.md) v1.3

---

### RP-402 — Budget-Aware t=1 Adaptation *(priority 2 — next)*

**Question:** Can budget-aware transition planning recover additional feasible solutions or improve objective values on the remaining infeasible instances?

**Hypothesis:** For instances with budget > 0, selectively re-routing the demands with the largest traffic change between t=0 and t=1 will reduce t=1 objective without violating the budget.

**Target instances (remaining infeasible after RP-401):** setA-02, setA-07, setA-09, setA-12, setA-17.

**Approach:**
- After computing the shared path (using RP-401 ECMP-aware routing), identify demands where `|v[1] - v[0]|` is largest
- Re-route those demands for t=1 only, counting budget cost via `SrPathBit::dist`
- Stop when budget is exhausted
- Measure: objective improvement on target instances; regression check on all 20

**Discipline:** One hypothesis, one capability, one evidence record. RP-402 is complete when the evidence record is filed and the result is recorded in BASELINE_HISTORY.md.

**Expected binary:** `src/bin/rp402_budget_adapt.rs`

---

### RP-403 — Multi-Path Candidate Generation *(conditional on RP-402 evidence)*

**Gate:** Proceed only if RP-402 evidence shows that candidate diversity is the dominant remaining limitation.

**Question:** Does generating K candidate paths per demand and selecting the best combination improve the objective?

**Hypothesis:** The greedy solver considers only one path per demand. Evaluating K shortest paths per demand under ECMP-accurate load estimation will find better combinations, particularly for high-volume demands that dominate the objective.

**Approach:**
- For each demand, generate K shortest paths (K = 3, 5, 10)
- Evaluate each candidate path under ECMP-accurate load estimation
- Select the combination that minimises the objective greedily
- Measure: objective improvement vs RP-402, runtime scaling with K

**Expected binary:** `src/bin/rp403_multipath.rs`

**Note:** This stage increases the search space substantially while keeping the solver deterministic. It is the correct foundation before introducing metaheuristics.

---

### RP-404 — Large Neighbourhood Search *(conditional on RP-403 evidence)*

**Gate:** Proceed only if RP-403 evidence shows that deterministic improvements have plateaued and stochastic search is warranted.

**Question:** Can LNS with destroy/repair operators improve on the deterministic baseline?

**Hypothesis:** Destroying and repairing subsets of demand assignments will escape local optima that the greedy solver gets stuck in.

**Approach:**
- Start from the RP-403 solution
- Destroy operator: remove waypoints for K randomly selected demands
- Repair operator: re-route removed demands using ECMP-aware Dijkstra (RP-401)
- Accept if objective improves
- Measure: objective improvement, convergence, operator effectiveness

**Expected binary:** `src/bin/rp404_lns.rs`

---

### RP-405 — Hyper-Heuristic Operator Selection *(conditional on RP-404 evidence)*

**Gate:** Proceed only if RP-404 evidence shows that operator selection is the dominant bottleneck in LNS performance.

**Question:** Can adaptive operator selection (using Coralys memory structures) improve LNS performance?

**Hypothesis:** Tracking which destroy/repair operator combinations succeed on which instance types will improve operator selection over time.

**Approach:**
- Extend RP-404 with a Coralys vault tracking operator success/failure rates
- Use pressure-guided selection to prefer operators with lower failure rates
- Measure: improvement over RP-404, vault convergence rate

**Expected binary:** `src/bin/rp405_hyper_lns.rs`

---

### RP-406 — Coralys MOGA Integration *(conditional on RP-405 evidence)*

**Gate:** Proceed only if RP-405 evidence shows that evolutionary search has a realistic chance of improving competition scores beyond the LNS baseline.

**Question:** Can the existing Coralys MOGA engine improve on the LNS baseline?

**Hypothesis:** A population-based search using the MOGA engine with SR path assignment as the genome will find better solutions on large instances, particularly after the decoder (RP-401) and neighbourhood (RP-404) are already strong.

**Approach:**
- Define genome as a vector of waypoint assignments (one per demand)
- Use `evaluator.evaluate_solution()` as the fitness function
- Initialise population from the RP-403 deterministic solution
- Run for a fixed time budget (e.g. 60 seconds per instance)
- Measure: objective improvement over RP-404, runtime, population diversity

**Expected binary:** `src/bin/rp406_moga_solver.rs`

**Note:** Evolutionary algorithms perform much better when the decoder and neighbourhoods are already strong. Introducing MOGA before RP-401–403 would waste search effort repairing weak candidate solutions.

---

### RP-407 — Hybrid Exact Subproblem *(conditional on RP-406 evidence)*

**Gate:** Proceed only if RP-406 evidence shows that exact optimisation of bottleneck subproblems offers a measurable return over the MOGA baseline.

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

## 6. Capability Maturity Model

Coralys is an optimisation platform with mature infrastructure and evolving domain capabilities. The ROADEF programme does not build Coralys from scratch — it exercises Coralys against a demanding industrial benchmark and discovers new reusable capabilities in the process.

The distinction matters: ROADEF is a **Platform Capability Discovery Programme**, not a maturity-building exercise. Improvements that emerge from ROADEF are promoted into the platform and reused across UltraCrew and future domains.

### 6.1 Platform Maturity Dimensions

| Dimension | Current Assessment |
|-----------|-------------------|
| Architecture maturity | Very High — modular crates, governance, observability, lifecycle management, benchmark methodology |
| Optimisation maturity | High — evolutionary search, multi-objective optimisation, ecology, adaptive operators, instrumentation |
| Domain maturity | Medium — CVRP (decoder, repair, ecology), UltraCrew (scheduling, explainability), ROADEF (routing, emerging) |
| Evidence maturity | Growing rapidly — architecture → governance → benchmarks → research programmes → external competitions |

### 6.2 Capability Maturity Levels

Each platform capability is tracked independently against a six-level scale. Promotion between levels is evidence-driven: a capability advances only when its exit criteria are met and an evidence record is filed.

| Level | Description | Exit Criteria |
|-------|-------------|---------------|
| C0 | Concept proven | Mathematical formulation documented; theoretical basis established |
| C1 | Unit tested | Implementation exists; unit tests pass; no benchmark validation yet |
| C2 | Benchmark validated | Measurable improvement demonstrated on a recognised benchmark instance set with reproducible evidence |
| C3 | Cross-domain validated | Same capability succeeds in ≥ 2 independent problem domains with separate evidence records |
| C4 | Production validated | Deployed in a production or near-production context; performance documented |
| C5 | Industry-proven | Externally validated through competition result, peer-reviewed publication, or customer deployment |

Promotion follows the same evidence-driven gate model as the RP evidence record schema: a capability cannot advance without a filed evidence record that satisfies the exit criteria for the target level.

The authoritative capability register is maintained at [`docs/governance/CAPABILITY_REGISTER.md`](../governance/CAPABILITY_REGISTER.md). The snapshot below reflects the state at the time of this programme version.

### 6.3 Capability Snapshot (v1.3 — post RP-401 freeze)

| Capability | Level | Evidence |
|------------|-------|---------|
| Evolution Engine | C4 | CVRP, UltraCrew, ROADEF baseline |
| Multi-objective optimisation | C4 | CVRP, UltraCrew |
| Observability / telemetry | C4 | Production deployment |
| Ecology / adaptive search | C3 | CVRP, UltraCrew |
| Workforce scheduling | C3 | UltraCrew |
| Vehicle routing | C3 | CVRP |
| Network routing (SR paths) | C2 | ROADEF Baseline v1.0 (Dataset A, 20 instances) |
| ECMP-aware incremental load estimation | **C2** | RP-401C — 13/20 improved, 0 regressed |
| Oracle-guided constructive routing | **C2** | RP-401C — same evidence; distinct reusable capability |
| Oracle-guided candidate selection | C1 | RP-401D — exploratory evidence only |
| Budget-aware transition planning | C1 | RP-000 (shared-path strategy); RP-402 target |
| Multi-path candidate generation | C0 | RP-403 target |
| LNS for routing | C0 | RP-404 target |
| Hyper-heuristic operator selection | C1 | RP-405 target (cross-domain: CVRP + ROADEF) |

### 6.4 ROADEF Capability Contributions

| ROADEF Result | Coralys Capability | Target Level |
|---------------|--------------------|-------------|
| ECMP-aware load estimation | `coralys-core` routing module | C3 (cross-domain after CVRP validation) |
| Budget distance metric | `coralys-planning` multi-period planning | C2 |
| Multi-path generation | `coralys-planning` decoder | C2 |
| LNS operators | `coralys-planning` neighbourhood search | C2 |
| Hyper-heuristic selection | `coralys-ecology` | C3 |
| MOGA on network routing | `coralys-moga` | C3 |

Evidence that generalises beyond ROADEF should be promoted to the platform. Evidence that is ROADEF-specific remains in the adapter.

---

## 7. Evidence Feedback to Coralys Platform

Each research programme produces platform-level evidence. ROADEF evidence targets:

| Evidence | Platform Component | Expected RP |
|----------|--------------------|-------------|
| ECMP-aware load estimation | `coralys-core` routing module | RP-401 |
| Budget-constrained re-routing | `coralys-planning` | RP-402 |
| Multi-path candidate generation | `coralys-planning` | RP-403 |
| LNS operators for routing | `coralys-planning` | RP-404 |
| Hyper-heuristic selection | `coralys-ecology` | RP-405 |
| MOGA on network routing | `coralys-moga` | RP-406 |

---

## 8. Programme Governance

| Role | Responsibility |
|------|---------------|
| Programme Owner | Defines research questions, approves promotion decisions |
| Solver Engineer | Implements RP binaries, maintains `campaign_engine` |
| Platform Engineer | Integrates generalised evidence into Coralys platform |

### 7.1 Amendment Log

| Version | Date | Change |
|---------|------|--------|
| 1.0 | 2026-08-02 | Initial programme document. Baseline v1.0 established from `campaign_engine` (commit `ec4d3821`). |
| 1.1 | 2026-08-02 | Added RP-000 (Budget Semantics Validation) as completed foundational finding. Added standard evidence record schema. Reordered experimental programme: RP-403 is now Multi-Path Candidate Generation (deterministic); MOGA moved to RP-406 after LNS (RP-404) and hyper-heuristic (RP-405). Rationale: metaheuristics perform better when decoder and neighbourhoods are already strong. |
| 1.2 | 2026-08-02 | Added four-stage RP-401 structure (401A–401D): measurement before optimisation. Added §6 Capability Maturity Model (C0–C5) with current capability register and ROADEF contribution targets. Renumbered §6 Evidence Feedback to §7, §7 Programme Governance to §8. |
| 1.3 | 2026-08-02 | Added CMM exit criteria to §6.2 (evidence-driven promotion gates). Added cross-reference to CAPABILITY_REGISTER.md. Created docs/governance/CAPABILITY_REGISTER.md (GOV-CR-001 v1.0) as platform-wide governance artefact tracking 14 capabilities across Core Optimisation, Planning and Search, Routing, and Domain Adapter categories. |
| 1.4 | 2026-08-02 | RP-401 frozen. §4 RP-401 entry replaced with frozen summary (all four stages, scientific conclusion, capability promotions). §6.3 capability snapshot updated to v1.3: "ECMP-aware routing" split into three distinct capabilities — ECMP-aware incremental load estimation (C2), oracle-guided constructive routing (C2, new), oracle-guided candidate selection (C1). CAPABILITY_REGISTER.md updated to v1.2 with same split. RP401_FINAL_REPORT.md updated to v1.3 with strengthened scientific conclusions, RP-401D renamed, §5 comparison table, timeout caveat, §10 Scientific Contribution. |
| 1.5 | 2026-08-02 | Conditional evidence gates added to RP-403 through RP-407 — each programme item now requires evidence from the preceding RP before proceeding, replacing the old priority-number labels. RP-402 entry sharpened: target instances named (setA-02, 07, 09, 12, 17), "one hypothesis, one capability, one evidence record" discipline recorded. Cross-reference to CS-S-005 Programme Horizon Strategy (three-horizon model, RP-408 deferral). |
