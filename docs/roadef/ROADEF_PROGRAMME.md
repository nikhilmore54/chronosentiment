# ROADEF Research Programme

**Programme:** EURO/ROADEF 2026 Challenge — T-Adaptive Segment Routing
**Status:** Active
**Version:** 1.13
**Date:** 2026-08-03

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

### 1.2 Programme Scientific Objectives

Beyond producing a competitive ROADEF submission, the programme seeks to advance reusable optimisation capabilities within Coralys. The intended scientific contributions are:

1. Construction methods for ECMP-aware segment routing (RP-401).
2. Budget-aware multi-period adaptation (RP-402).
3. Construction portfolio selection (RP-403).
4. Large neighbourhood search for adaptive routing (RP-404).
5. Hyper-heuristic operator selection (RP-405).
6. Cross-domain optimisation capabilities identified through the ROADEF programme and promoted into Coralys where supported by evidence.

Each contribution is validated on the ROADEF benchmark and promoted into the Coralys platform as a reusable capability. The programme follows the principle: identify the dominant bottleneck, solve it, promote the capability, identify the next bottleneck.

### 1.3 Programme Research Cycle

The programme follows a repeating evidence-driven cycle. Each iteration produces one promoted capability and identifies the next dominant bottleneck:

```
Research Question
        ↓
Hypothesis
        ↓
Implementation
        ↓
Benchmark Evidence
        ↓
Capability Assessment
        ↓
Platform Promotion
        ↓
Next Research Question
```

This cycle is enforced by the standard RP lifecycle (§4.1). No RP may proceed to implementation without a stated hypothesis. No RP may be declared complete without a termination gate decision. No capability may advance without a filed evidence record.

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

### 4.1 Standard RP Lifecycle

Every research programme item follows the same lifecycle. This template is mandatory for all future RPs and applies equally to ROADEF, CVRP, UltraCrew, and any future Coralys research programme.

```
Research Question
      ↓
Hypothesis
      ↓
Implementation
      ↓
Benchmark
      ↓
Capability Assessment
      ↓
Root-Cause Analysis
      ↓
Termination Gate
      ↓
Next RP
```

**Termination gate (standard):** Every RP must explicitly answer one of four outcomes before the next RP may begin:

| Outcome | Symbol | Meaning |
|---------|--------|---------|
| Capability promoted | ✅ | Hypothesis confirmed; capability advances to next maturity level; next RP proceeds |
| Capability refined | 🔄 | Partial confirmation; capability scope narrowed; current RP extended or redefined |
| Capability archived | 📦 | Hypothesis not confirmed; capability remains at current level; RP archived with negative result |
| Hypothesis rejected | ❌ | Evidence contradicts hypothesis; RP archived; programme direction reconsidered |

A negative result (📦 or ❌) is a valid and valuable outcome. It prevents engineering effort from being spent on the wrong problem. The programme records the negative result, archives the RP, and either redefines scope or advances to the next independent research question.

No RP may proceed to implementation before its hypothesis is stated. No RP may be declared complete without a termination gate decision. No next RP may begin without the preceding RP's termination gate being cleared.

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

### RP-402 — Budget-Aware t=1 Adaptation *(🔒 FROZEN 2026-08-03)*

**Status:** Complete. All 20 Dataset A instances executed. RP-402 is frozen.

**Scientific conclusion:** Budget-aware transition planning is a reusable Coralys capability. Selectively re-routing demands with the largest traffic change |v[1]−v[0]| for t=1 within the transition budget recovered 3 of the 5 targeted infeasible instances and achieved the best finite solution count (18/20) and improvement count (15/20) across all RP-401/402 variants. The remaining infeasible instances (setA-12, setA-17) are the open research questions for RP-403.

| Metric | Result |
|--------|--------|
| Improved vs empty | 15/20 (best so far) |
| Finite solutions | 18/20 (best so far) |
| Total improvement vs empty | 2,584,436.44 |
| Target instances recovered | 3/5 (setA-02, setA-07, setA-09) |
| Remaining infeasible | setA-12 (budget=13), setA-17 (budget=1) |
| Runtime | ~58 min total (unchanged vs RP-401D) |

**Capabilities promoted:**
- Budget-aware transition planning: C1 → **C2** (benchmark validated)
- Budget-constrained re-routing: C1 → **C2** (subsumed by same evidence)

**Full evidence:** [`RP402_FINAL_REPORT.md`](RP402_FINAL_REPORT.md) v1.0

---

### RP-403 — Construction Strategy Evaluation and Selection *(✅ Hypothesis Confirmed — 2026-08-03)*

**Status:** Complete. Corrected RP-403 benchmark (20/20 instances) executed after Validation Task V1 (Commit C, `e9296dfa`). Termination gate: ✅ Hypothesis Confirmed. RP-404 is the active work item.

**Root-cause analysis:** [`RP403_ROOT_CAUSE_ANALYSIS.md`](RP403_ROOT_CAUSE_ANALYSIS.md) v1.1 (Phase 1A mining, 2026-08-03)
**Benchmark report:** [`RP403_BENCHMARK_REPORT.md`](RP403_BENCHMARK_REPORT.md) v1.3 (2026-08-03) — authoritative; supersedes v1.2
**Validation report:** [`RP403_V1_VALIDATION_REPORT.md`](RP403_V1_VALIDATION_REPORT.md) v1.0 (2026-08-03)
**Binary:** `src/bin/rp403_construction_portfolio.rs` (commit `e9296dfa`, corrected)

**Corrected 20/20 benchmark results (commit `e9296dfa`):**

| Metric | RP-402 | Corrected RP-403 | Change |
|--------|--------|-----------------|--------|
| Finite solutions | 18/20 | **19/20** | **+1** |
| setA-08 | inf | **45.6696** | RECOVERED |
| setA-12 | inf | **26.1166** | RECOVERED |
| setA-17 | inf | inf | still unsolved |
| Remaining infeasible | 2 | **1** | **−1** |

**Scientific findings:**
1. Construction strategy selection materially affects downstream optimisation quality (setA-08 and setA-12 both recovered from infeasibility).
2. The optimization pipeline exhibits strong coupling between construction and adaptation: construction-time objective is not a reliable predictor of post-adaptation objective. Changing the construction changes almost every downstream solution (14 objective regressions on previously feasible instances).
3. The portfolio demonstrates heuristic complementarity: RP-401D selected on 8/20 instances; it is not a generally better constructor but a complementary heuristic that succeeds where RP-401C fails (most critically setA-08). The pre-adaptation selection criterion is the principal limitation; post-adaptation selection would be more reliable.
4. setA-17 remains the single open instance across all deterministic construction strategies (RP-401 through RP-403). It becomes the primary target for RP-404.

**Termination gate outcome:** ✅ Hypothesis Confirmed. Implementation equivalence confirmed (Validation Task V1). Both setA-08 and setA-12 recovered. Capability outcome: Construction portfolio selection satisfies the C2 exit criteria (benchmark validated on Dataset A).

---

#### RP-403 Validation Task V1 — RP-401C Behavioural Equivalence *(✅ Closed — 2026-08-03)*

**Status:** Closed. All closure criteria met.

**Question:** Does the embedded `solve_rp401c` function produce the same waypoint assignments as the standalone `rp401c_ecmp_construction` binary?

**Evidence collected (Commits A–C):**
- Commit A (`5aecb4d9`): validator binary `rp403v1_validate_rp401c` added
- Commit B (`1bd13257`): original divergence documented — 232/400 demands differ on setA-12; first divergence at demand 0 (src=106, dst=178); root cause: multiplicative vs additive penalty
- Commit C (`e9296dfa`): corrective patch applied — additive penalty matching standalone; 400/400 waypoint assignments confirmed identical

**Closure criteria met:**
- ✅ Divergence localised: 232/400 demands differ; first at demand 0
- ✅ Root cause identified: multiplicative vs additive penalty (three formula differences)
- ✅ Correction applied and verified: 400/400 match on setA-12
- ✅ RP-403 re-run on all 20 instances with corrected implementation (Commit D)

**After closure:** RP-403 Hypothesis Confirmed (✅). RP-404 is now the active work item.

---

### RP-404 — Large Neighbourhood Search *(active — RP-403 gate cleared)*

**Gate:** ✅ Cleared. RP-403 Hypothesis Confirmed (2026-08-03). Use the validated RP-403 construction portfolio (commit `e9296dfa`) as the deterministic baseline. Primary target: setA-17 (the single remaining infeasible instance across all deterministic construction strategies). The benchmark also reveals strong initialization sensitivity — 14 objective regressions on previously feasible instances — suggesting that escaping local optima via LNS may yield substantial improvements beyond the deterministic baseline.

**Scientific lineage:** RP-403 demonstrated that deterministic construction strongly influences downstream optimisation and identified initialization sensitivity as the principal remaining limitation (14 objective regressions on previously feasible instances; setA-17 unrecovered by all deterministic strategies). RP-404 therefore investigates whether neighbourhood search can escape these construction-induced local optima.

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

**Gate:** Proceed only if RP-404 identifies hyper-heuristic operator selection as the dominant remaining limitation.

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

A Research binary becomes a Candidate when its approved benchmark evidence record demonstrates:
- Net improvement over the current best result on Dataset A (more finite solutions, lower aggregate objective, or both)
- No feasibility regressions on instances where the current best has a finite objective
- Runtime documented and reproducible from a clean build

Promotion is based on the approved benchmark evidence record rather than a fixed numerical threshold. The termination gate mechanism (§4.1) already enforces evidence-driven promotion; a separate numerical threshold would be redundant and could exclude a solver with fewer improvements but critical feasibility recoveries.

A Candidate becomes the Competition Submission when:
- It beats the previous Competition Submission on the full instance set
- It is reproducible from a clean build
- It has been validated against the official checker

### 5.2 Archive Policy

Research binaries that do not satisfy the promotion criteria or whose evidence does not justify promotion are archived (not deleted). They remain as evidence of the research lineage. This follows the same policy as the CVRP and UltraCrew experiment archives.

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
| C2 | Benchmark validated | Reproducible benchmark evidence demonstrating that the capability provides measurable benefit on a recognised instance set |
| C3 | Cross-domain validated | Same capability succeeds in ≥ 2 independent problem domains with separate evidence records |
| C4 | Production validated | Deployed in a production or near-production context; performance documented |
| C5 | Industry-proven | Externally validated through competition result, peer-reviewed publication, or customer deployment |

Promotion follows the same evidence-driven gate model as the RP evidence record schema: a capability cannot advance without a filed evidence record that satisfies the exit criteria for the target level.

The authoritative capability register is maintained at [`docs/governance/CAPABILITY_REGISTER.md`](../governance/CAPABILITY_REGISTER.md). The snapshot below reflects the state at the time of this programme version.

### 6.3 Capability Snapshot (v1.6 — post RP-403 Hypothesis Confirmed)

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
| **Budget-aware transition planning** | **C2** | RP-402 — 15/20 improved, 18/20 finite, 3/5 targets recovered |
| **Budget-constrained re-routing** | **C2** | RP-402 — subsumed by budget-aware transition planning evidence |
| Oracle-guided candidate selection | C1 | RP-401D — exploratory evidence only |
| Construction portfolio selection | **C2** | RP-403 — 19/20 finite, 2 instances recovered from infeasibility; Hypothesis Confirmed |
| Multi-path candidate generation | C0 | Deferred research hypothesis |
| LNS for routing | C0 | RP-404 target |
| Hyper-heuristic operator selection | C1 | RP-405 target (cross-domain: CVRP + ROADEF) |

### 6.4 Expected Platform Capability Contributions

| ROADEF Result | Coralys Capability | Target Level |
|---------------|--------------------|-------------|
| ECMP-aware load estimation | `coralys-core` routing module | C3 (cross-domain after CVRP validation) |
| Budget distance metric | `coralys-planning` multi-period planning | C2 |
| Construction portfolio selection | `coralys-planning` construction framework | C2 |
| LNS operators | `coralys-planning` neighbourhood search | C2 |
| Hyper-heuristic selection | `coralys-ecology` | C3 |
| MOGA on network routing | `coralys-moga` | C3 |

Evidence that generalises beyond ROADEF should be promoted to the platform. Evidence that is ROADEF-specific remains in the adapter.

---

## 7. Evidence Promotion to Coralys Platform

Each research programme produces platform-level evidence. ROADEF evidence targets:

| Evidence | Platform Component | Expected RP |
|----------|--------------------|-------------|
| ECMP-aware load estimation | `coralys-core` routing module | RP-401 |
| Budget-constrained re-routing | `coralys-planning` | RP-402 |
| Construction portfolio selection | `coralys-planning` construction framework | RP-403 |
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

### 8.1 Amendment Log

| Version | Date | Change |
|---------|------|--------|
| 1.0 | 2026-08-02 | Initial programme document. Baseline v1.0 established from `campaign_engine` (commit `ec4d3821`): 11/20 finite solutions, shared-path strategy, ECMP mismatch identified as primary weakness. |
| 1.1 | 2026-08-02 | RP-000 (Budget Semantics Validation) added as completed foundational finding: shared SR paths guarantee zero budget cost. Standard evidence record schema added. RP sequence reordered: MOGA deferred to RP-406 after LNS and hyper-heuristic (metaheuristics perform better when decoder and neighbourhoods are already strong). |
| 1.2 | 2026-08-02 | Four-stage RP-401 structure added (401A–401D): measurement before optimisation. §6 Capability Maturity Model (C0–C5) added with current capability register and ROADEF contribution targets. |
| 1.3 | 2026-08-02 | CMM exit criteria added to §6.2 (evidence-driven promotion gates). CAPABILITY_REGISTER.md (GOV-CR-001 v1.0) created as platform-wide governance artefact tracking 14 capabilities. |
| 1.4 | 2026-08-02 | RP-401 frozen: all four stages complete; ECMP-aware load estimation and oracle-guided constructive routing both promoted to C2; 13/20 improved, 0 regressed. Capability snapshot v1.3: "ECMP-aware routing" split into three distinct capabilities. |
| 1.5 | 2026-08-02 | Conditional evidence gates added to RP-403 through RP-407: each RP now requires evidence from the preceding RP before proceeding. RP-402 entry sharpened: target instances named (setA-02, 07, 09, 12, 17). |
| 1.6 | 2026-08-03 | RP-402 frozen: 15/20 improved, 18/20 finite, 3/5 targets recovered; budget-aware transition planning and budget-constrained re-routing promoted to C2. RP-403 reframed as "Adaptive Candidate Generation and Diversity Recovery" with pre-coding root-cause analysis required. |
| 1.7 | 2026-08-03 | Explicit termination gate added to RP-403: proceed to implementation only if root-cause analysis identifies at least one failure mode plausibly addressable by candidate-generation methods. |
| 1.8 | 2026-08-03 | Standard RP lifecycle template formalised as §4.1: eight-stage lifecycle with four termination outcomes (✅ promoted, 🔄 refined, 📦 archived, ❌ rejected). Template applies to all future Coralys research programmes. |
| 1.9 | 2026-08-03 | RP-403 redefined following Phase 1A root-cause analysis: original path-diversity hypothesis not supported; all three failures occur at the construction layer (RP-402 adaptation never fires for setA-12/17). RP-403 renamed "Construction Strategy Evaluation and Selection"; binary renamed `rp403_construction_portfolio.rs`. |
| 1.10 | 2026-08-03 | RP-403 initial benchmark completed (20/20 instances): 19/20 finite, setA-08 RECOVERED. setA-12 classified CONFOUNDED (embedded vs standalone RP-401C divergence). Validation Task V1 added as blocking prerequisite for RP-403 termination-gate closure. |
| 1.11 | 2026-08-03 | Governance refinements: RP-404 gate updated to reference Validation Task V1 closure explicitly; capability snapshot v1.5 (construction portfolio selection at C1, validation pending); §6.4 and §7 evidence tables updated to reflect RP-403's actual contribution. |
| 1.12 | 2026-08-03 | RP-403 closed following corrected benchmark validation (commit `e9296dfa`): 19/20 finite, setA-08 RECOVERED, setA-12 RECOVERED (400/400 waypoint equivalence confirmed; root cause: multiplicative vs additive penalty). Validation Task V1 closed. RP-404 gate cleared with initialization-sensitivity motivation. Capability snapshot v1.6: construction portfolio selection promoted C1 → C2. |
| 1.13 | 2026-08-03 | Programme-level refinements: §1.2 Scientific Objectives added; RP-404 scientific lineage from RP-403 stated explicitly; C2 exit criteria sharpened to "reproducible benchmark evidence demonstrating measurable benefit"; promotion criteria made evidence-driven (fixed numerical threshold removed); RP-405 gate reframed as bottleneck-driven; §6.4 renamed to Expected Platform Capability Contributions; §7 renamed to Evidence Promotion to Coralys Platform; amendment log entries enriched with substantive summaries. |
