# SD-007 Interim Report — Discovery Failure Root Cause Investigation

**Defect ID:** SD-007
**Status:** OPEN — mechanism not yet isolated
**Sprint:** 3.9
**Classification:** RC-1 Strongly Indicated; RC-2 Remains Plausible
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
| \|ρ(Oi, HC_Total)\| < 0.1 for all i=1..5 | Not yet measured |
| Median HC_Total at gen=0 > 50 AND HC_Total at gen=5000 ≈ gen=0 | Partial — gen=0 snapshot empty; floor stable at 33k–34k |
| Evaluator source confirms sentinel return for HC_Total > 0 | Not required (RC-4 falsified in Sprint 3.8) |

**SD-007 Classification: RC-1 Strongly Indicated; RC-2 Remains Plausible**

The HC_Total trajectory probe confirms that the search dynamics diverge from feasibility over 5000 generations. However, the probe measures the **archive/population trajectory** — it cannot isolate whether the failure is in the mutation operator itself (RC-1) or in the selection/proxy geometry that removes HC-improving offspring before they can accumulate (RC-2/RC-3 interaction).

Three hypotheses remain compatible with the observed trajectory:

| Hypothesis | Description | Compatible with data? |
|------------|-------------|----------------------|
| **A (RC-1)** | Operator structurally incapable of reducing HC | Yes |
| **B (RC-2)** | Operator capable, but selection removes HC-improving offspring | Yes |
| **C (RC-1+RC-2)** | Operator occasionally improves HC, but O3 rewards dominate and drive population away | Yes |

The HC distribution probe cannot distinguish A/B/C because it only measures outcomes after selection, not offspring before selection.

---

## Causal Chain (Partial — Mechanism Not Yet Isolated)

```
UltraCrewMutator (single-point shift/swap/reassign)
    ↓
[UNKNOWN: does operator reduce HC in offspring before selection?]
    ↓
HC_Total floor: 33,000–34,000 (33–34 violations, penalty-weighted)
    ↓
Mean HC_Total increases over 5000 gens (search diverges from feasibility)
    ↓
[UNKNOWN: is divergence driven by operator incapacity (RC-1)
          or by O3 selection pressure removing HC-improving offspring (RC-2)?]
    ↓
Evaluator never returns feasible=true
    ↓
SD-005: Discovery Failure (0 feasible genomes in 5000 gens)
    ↓
0% feasible archive
```

The causal chain is confirmed from the HC_Total floor onward. The mechanism upstream of the floor — whether it is operator incapacity or selection pressure — is not yet isolated.

---

## Root Cause Analysis

### RC-1: Operator Incapacity (Strongly Indicated — Not Yet Isolated)

`UltraCrewMutator` applies single-point mutations (shift assignment, swap, reassign). The INRC-II n050w4 instance has 50 nurses, 4 weeks, and a dense constraint graph. The HC constraints (H1: shift coverage, H2: max shifts/week, H3: no double-shift per day) require coordinated multi-constraint satisfaction. A single-point mutation that fixes one HC violation typically introduces another.

Evidence: Mean HC_Total increases from 41,205 to 53,954 over 5000 generations. The search is not making progress toward feasibility — it is diverging. However, this trajectory is measured after selection. It does not prove the operator cannot produce HC-improving offspring — only that such offspring do not survive in the archive.

**Required probe to confirm RC-1:** Measure `P(child_HC < parent_HC)` over raw offspring before selection. If near zero, RC-1 is confirmed. If offspring frequently reduce HC but disappear after selection, RC-2 is implicated.

### RC-2: Proxy Misalignment (Plausible — Not Measured)

Sprint 3.7 confirmed that O3 (HC_Successions proxy) actively rewards behavior that worsens external quality. The same mechanism may be removing HC-improving offspring from the archive before they can accumulate. If offspring occasionally reduce HC_Total but are dominated by O3-superior infeasible genomes, the population will drift away from feasibility even if the operator is capable.

Evidence: O3 pressure confirmed in SD-006. HC_Total trajectory diverges from feasibility. These two findings are compatible with RC-2 as the primary mechanism.

**Required probe to confirm RC-2:** Measure ρ(O3, HC_Total) across archive members. If strongly negative (O3 improvement correlates with HC_Total increase), RC-2 is confirmed.

### RC-3: Initialization Depth (Partial — gen=0 snapshot empty)

The gen=0 archive snapshot was empty (archive not yet populated at gen=0 census). However, the baseline genome has `HC_Coverage=19, HC_Skills=1, HC_ForbiddenSucc=16` (from console output: 19000+1000+16000 penalty-weighted), totalling 36 actual violations. This is consistent with the observed floor of 33–34 violations. Initialization depth is a contributing factor but not independently sufficient to explain the divergence.

### RC-4: Evaluator Anomaly (Falsified)

The `best_proxy=-1.00` observation from Sprint 3.8 is consistent with `hard=1` (minimum possible violation count), not a sentinel. The evaluator correctly sets `feasible = hard == 0` with no short-circuit. RC-4 is falsified.

---

## Required Next Step — ΔHC Offspring Probe

To isolate RC-1 from RC-2, the following probe must be added to `inrc_archive_forensics.rs`:

```rust
// For each offspring, before archive.add():
let parent_hc = score_inrc_official(&parent.genome, ...).hc_total();
let child_hc  = score_inrc_official(&child_genome, ...).hc_total();
let delta_hc  = child_hc as i64 - parent_hc as i64;
// Record: (generation, parent_hc, child_hc, delta_hc, was_inserted)
```

Aggregate over all offspring across 5000 generations:
- `P(delta_hc < 0)` = probability that a mutation reduces HC_Total
- `P(delta_hc < 0 AND was_inserted)` = probability that an HC-improving offspring survives selection

**Classification:**

| Observation | Classification |
|-------------|----------------|
| `P(delta_hc < 0) ≈ 0` | RC-1 CONFIRMED — operator cannot reduce HC |
| `P(delta_hc < 0) > 0.1` AND `P(delta_hc < 0 AND was_inserted) ≈ 0` | RC-2 CONFIRMED — selection removes HC-improving offspring |
| Both probabilities > 0 | RC-1 + RC-2 interaction |

---

## Confidence Assessment

| Claim | Confidence |
|-------|------------|
| SD-005 Discovery Failure | Very High |
| Search dynamics diverge from feasibility | Very High |
| Archive not responsible for feasibility failure | High |
| O3 proxy misalignment exists (SD-006) | Very High |
| Mutation operator incapacity is the sole root cause (RC-1) | Medium |
| SD-007 fully closed | No — ΔHC offspring probe required |

---

## Scientific Debt Ledger

| ID | Description | Status |
|----|-------------|--------|
| SD-003 | Champion Retention Error: best external champion not in final archive | CLOSED (Sprint 3.6) |
| SD-005 | 0% feasible solutions in archive after 5000 generations | CLOSED (Sprint 3.8) |
| SD-006 | O3 proxy pressure causes champion eviction | CLOSED (Sprint 3.7) |
| **SD-007** | Discovery Failure: mechanism not yet isolated (RC-1 vs RC-2) | **OPEN** |

SD-007 remains open. The HC_Total trajectory probe confirms search divergence from feasibility but cannot isolate the mechanism. The ΔHC offspring probe (Sprint 3.10) is required to distinguish operator incapacity (RC-1) from selection pressure removing HC-improving offspring (RC-2).

---

## Commits

| Commit | Description |
|--------|-------------|
| `67db87c0` | `sd005_resolution.md` (SD-005 CLOSED, SD-007 OPENED) |
| `fc8e20f7` | `sd007_sprint39_charter.md` + instrumentation audit in `sd005_resolution.md` |
| `24d9a587` | HC_Total distribution probe + `sd007_resolution.md` (interim, SD-007 OPEN) |