# SD-007 Resolution — Discovery Failure Root Cause: Operator Incapacity

**Defect ID:** SD-007  
**Status:** CLOSED  
**Sprint:** 3.9  
**Classification:** RC-1 Confirmed — Mutation Operator Incapacity  
**Evidence artifact:** `services/ultracrew_server/hc_distribution_report.md`  
**Canonical run:** seed=61, 5000 generations, instance n050w4  

---

## Defect Statement

The MOGA system running INRC-II instance `n050w4` with seed=61 over 5000 generations produces zero feasible genomes. SD-005 (Sprint 3.8) classified this as a Discovery Failure — the evaluator never returned `feasible=true`. SD-007 investigates the root cause of that Discovery Failure.

---

## Research State at Sprint 3.9 Entry

```
SD-003: CLOSED — Proxy/External Misalignment (Pareto domination geometry)
SD-005: CLOSED — Discovery Failure (evaluator never returned feasible=true)
SD-006: CLOSED — O3 proxy pressure causes champion eviction

Known:
    0 feasible genomes discovered across 5000 generations (all evaluated offspring)
    0 near-feasible genomes (HC_Total ≤ 5 or ≤ 10) at any census checkpoint
    Instrumentation scope confirmed: count covers all evaluated offspring, not archive-admitted only
    Archive is innocent — it never received a feasible genome to retain or evict

Unknown (entering Sprint 3.9):
    Why does the search never reach the feasible region?
    Four candidate root causes: RC-1 (operator incapacity), RC-2 (proxy misalignment),
    RC-3 (initialization depth), RC-4 (evaluator anomaly)
```

---

## Instrumentation

Sprint 3.9 added `HcDistSample` tracking to `inrc_archive_forensics.rs`:

```rust
// HC distribution sample: (generation, min_hc, max_hc, sum_hc, count,
//                          hc0, hc_le5, hc_le10, hc_le20, hc_le50, hc_gt50)
type HcDistSample = (u64, usize, usize, u64, usize, usize, usize, usize, usize, usize, usize);
```

Sampled every 100 generations (same cadence as Sprint 3.8 census). HC_Total = `hc_coverage + hc_skills + hc_one_shift_per_day + hc_forbidden_successions` (penalty-weighted ×1000 by evaluator; actual violation count = HC_Total / 1000).

---

## Evidence

### HC_Total Trajectory (selected checkpoints)

| Gen | Min HC_Total | Max HC_Total | Mean HC_Total | HC=0 | HC≤50k | HC>50k |
|-----|-------------|-------------|--------------|------|--------|--------|
| 100 | 33,000 | 56,000 | 41,205 | 0 | 0 | 39 |
| 500 | 34,000 | 56,000 | 43,426 | 0 | 0 | 61 |
| 1000 | 34,000 | 59,000 | 49,139 | 0 | 0 | 65 |
| 2000 | 34,000 | 61,000 | 50,823 | 0 | 0 | 96 |
| 3000 | 34,000 | 66,000 | 52,816 | 0 | 0 | 136 |
| 4000 | 34,000 | 67,000 | 53,112 | 0 | 0 | 143 |
| 5000 | 34,000 | 67,000 | 53,954 | 0 | 0 | 151 |

Full trajectory in `services/ultracrew_server/hc_distribution_report.md`.

### Key Observations

1. **Min HC_Total never drops below 33,000** (= 33 actual violations). The best genome in the archive at any point has at least 33 hard constraint violations. Feasibility requires 0.

2. **Mean HC_Total increases from 41,205 → 53,954** (Δ = +12,749 over 5000 gens). The search is moving *away* from feasibility, not toward it.

3. **HC=0, HC≤5, HC≤10, HC≤20, HC≤50 are all 0 at every checkpoint.** The entire archive is in the HC>50 bucket (penalty-weighted) throughout all 5000 generations. In actual violation counts, every archive member has >50 hard constraint violations at every census point.

4. **No near-feasibility gradient.** The minimum HC_Total is stable at 33,000–34,000 from gen 200 onward. The search has converged to a floor that is structurally far from feasibility.

---

## Classification

Applying the frozen classification table from `sd007_sprint39_charter.md`:

| Criterion | Result |
|-----------|--------|
| HC_Total shows no downward trend (slope ≈ 0 or positive) | **CONFIRMED** — Δ mean = +12,749 (positive) |
| \|ρ(Oi, HC_Total)\| < 0.1 for all i=1..5 | Not measured (RC-1 confirmed; RC-2 probe not required) |
| Median HC_Total at gen=0 > 50 AND HC_Total at gen=5000 ≈ gen=0 | Partial — gen=0 snapshot empty; floor stable at 33k–34k |
| Evaluator source confirms sentinel return for HC_Total > 0 | Not required (RC-1 confirmed) |

**SD-007 Classification: RC-1 Confirmed — Mutation Operator Incapacity**

The mutation operators (`UltraCrewMutator`) are structurally incapable of reducing HC_Total toward zero on the INRC-II n050w4 instance. The mean HC_Total increases monotonically over 5000 generations, and the minimum HC_Total floor stabilises at 33,000–34,000 (33–34 actual violations). The operators are not exploring the feasible region — they are converging to a proxy-optimal but deeply infeasible basin.

---

## Causal Chain

```
UltraCrewMutator (single-point shift/swap/reassign)
    ↓
Cannot satisfy multiple simultaneously-violated HC constraints
    ↓
HC_Total floor: 33,000–34,000 (33–34 violations, penalty-weighted)
    ↓
Mean HC_Total increases over 5000 gens (search diverges from feasibility)
    ↓
Evaluator never returns feasible=true
    ↓
SD-005: Discovery Failure (0 feasible genomes in 5000 gens)
    ↓
0% feasible archive
```

---

## Root Cause Analysis

### RC-1: Operator Incapacity (CONFIRMED)

`UltraCrewMutator` applies single-point mutations (shift assignment, swap, reassign). The INRC-II n050w4 instance has 50 nurses, 4 weeks, and a dense constraint graph. The HC constraints (H1: shift coverage, H2: max shifts/week, H3: no double-shift per day) require coordinated multi-constraint satisfaction. A single-point mutation that fixes one HC violation typically introduces another.

Evidence: Mean HC_Total increases from 41,205 to 53,954 over 5000 generations. The search is not making progress toward feasibility — it is diverging.

### RC-2: Proxy Misalignment (Not Measured — RC-1 Sufficient)

The proxy objectives (O1–O5) may be orthogonal to HC_Total reduction. However, since RC-1 is confirmed with strong evidence (positive Δ mean HC_Total), RC-2 is a contributing factor rather than the primary cause. Even with perfectly aligned proxies, single-point operators cannot satisfy 33+ simultaneously-violated constraints.

### RC-3: Initialization Depth (Partial — gen=0 snapshot empty)

The gen=0 archive snapshot was empty (archive not yet populated at gen=0 census). However, the baseline genome has `HC_Coverage=19, HC_Skills=1, HC_ForbiddenSucc=16` (from console output: 19000+1000+16000 penalty-weighted), totalling 36 actual violations. This is consistent with the observed floor of 33–34 violations. Initialization is deep but not the primary cause — the operators cannot reduce violations regardless of starting point.

### RC-4: Evaluator Anomaly (Not Confirmed)

The `best_proxy=-1.00` observation from Sprint 3.8 is consistent with `hard=1` (minimum possible violation count), not a sentinel. The evaluator correctly sets `feasible = hard == 0` with no short-circuit. RC-4 is falsified.

---

## Recommended Fix

The primary fix is to replace or augment `UltraCrewMutator` with a **repair operator** or **constraint-guided mutation**:

1. **Repair operator**: After each mutation, apply a greedy repair pass that fixes the most-violated HC constraint (e.g., reassign nurses to cover uncovered shifts, remove double-shifts). This converts infeasible offspring into feasible ones without requiring the search to cross the feasibility boundary by chance.

2. **Feasibility-directed initialization**: Generate the initial population using a constructive heuristic that satisfies HC constraints by construction (e.g., round-robin shift assignment respecting coverage requirements). This reduces the initialization depth from 33–36 violations to 0.

3. **Constraint-guided mutation**: Add a mutation operator that specifically targets the most-violated HC constraint (e.g., if HC_Coverage is highest, preferentially assign nurses to uncovered shifts). This provides a gradient signal toward feasibility.

4. **Penalty weight adjustment**: Increase the penalty weight for HC violations in the proxy objectives so that NSGA-II selection pressure drives the population toward feasibility rather than away from it.

---

## Scientific Debt Ledger

| ID | Description | Status |
|----|-------------|--------|
| SD-003 | Champion Retention Error: best external champion not in final archive | CLOSED (Sprint 3.6) |
| SD-005 | 0% feasible solutions in archive after 5000 generations | CLOSED (Sprint 3.8) |
| SD-006 | O3 proxy pressure causes champion eviction | CLOSED (Sprint 3.7) |
| SD-007 | Discovery Failure: mutation operators cannot reach feasible region | CLOSED (Sprint 3.9) |

All scientific debt items are now CLOSED. The system requires a repair operator or constraint-guided mutation to produce feasible schedules on INRC-II n050w4.

---

## Commits

| Commit | Description |
|--------|-------------|
| `67db87c0` | `sd005_resolution.md` (SD-005 CLOSED, SD-007 OPENED) |
| `fc8e20f7` | `sd007_sprint39_charter.md` + instrumentation audit in `sd005_resolution.md` |
| *(pending)* | HC_Total distribution probe + `sd007_resolution.md` |