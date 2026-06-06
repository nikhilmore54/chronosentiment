# SD-007 Sprint 3.9 Charter — Discovery Failure Root Cause Investigation

**Defect ID:** SD-007  
**Sprint:** 3.9  
**Branch:** `governance-hardening`  
**Opened by:** Sprint 3.8 `sd005_resolution.md` (commit `67db87c0`)  
**Status:** OPEN  
**Priority:** P0 — blocks all feasibility-dependent objectives  

---

## 1. Defect Statement

The MOGA system running INRC-II instance `n050w4` with seed=61 over 5000 generations produces **zero feasible genomes**. The Sprint 3.8 census further reveals **zero near-feasible genomes** (HC_Total ≤ 5 or HC_Total ≤ 10) at all 51 census checkpoints (every 100 generations). The search is not approaching the feasibility boundary — it is converging to a region that is proxy-optimal but structurally far from feasibility.

**Null hypothesis (H₀):** The mutation operators are capable of reaching the feasible region but have not done so in 5000 generations due to stochastic factors alone.

**Alternative hypothesis (H₁):** The mutation operators are structurally incapable of reaching the feasible region within the INRC-II n050w4 constraint landscape, or the proxy objectives actively guide search away from feasibility.

---

## 2. Evidence Inherited from Sprint 3.8

From `feasible_lineage_report.md` (seed=61, 5000 gens):

| Metric | Value |
|--------|-------|
| Total feasible discovered | 0 |
| Total near-feasible (HC ≤ 5) at any checkpoint | 0 |
| Total near-feasible (HC ≤ 10) at any checkpoint | 0 |
| Census checkpoints sampled | 51 (every 100 gens) |
| Archive size at gen 5000 | unknown (see Step 1) |
| Best proxy score observed | −1.00 (all archive members) |

The `best_proxy=-1.00` sentinel for all archive members is a secondary anomaly: it indicates the proxy evaluator is returning a degenerate constant rather than a meaningful gradient. This may be a separate instrumentation defect or may indicate the evaluator is short-circuiting on infeasible inputs.

---

## 3. Four Candidate Root Causes

### RC-1: Mutation Operator Structural Incapacity
`UltraCrewMutator` applies shift/swap/reassign moves on individual assignments. The INRC-II n050w4 instance has 50 nurses, 4 weeks, and a dense constraint graph (consecutive shifts, coverage requirements, skill matching, rest periods). A single-point mutation may be insufficient to satisfy multiple simultaneously-violated hard constraints. The feasibility boundary may require coordinated multi-constraint satisfaction that single-point operators cannot achieve.

**Probe:** Measure HC_Total distribution across all archive members at each generation. If HC_Total is not decreasing over 5000 gens, the operator is not making progress toward feasibility.

### RC-2: Proxy Objective Misalignment
Objectives O1–O5 are proxy metrics (e.g., coverage balance, shift preference, workload distribution). If these proxies are orthogonal to or anti-correlated with HC_Total reduction, NSGA-II selection pressure will drive the population away from feasibility even if individual mutations occasionally reduce HC_Total.

**Probe:** Compute Pearson correlation between each proxy Oi and HC_Total across all archive members at gen 5000. If |ρ(Oi, HC_Total)| < 0.1 for all i, proxies provide no feasibility gradient.

### RC-3: Baseline Initialization Depth
The initial population may be generated with HC_Total values so high (e.g., HC_Total > 100) that 5000 generations of single-point mutation cannot reach HC_Total = 0. The search starts too far from the feasibility boundary.

**Probe:** Record HC_Total of all initial genomes (gen=0 archive snapshot). If median HC_Total > 50, initialization is the primary bottleneck.

### RC-4: `best_proxy=-1.00` Evaluator Anomaly
All archive members report `best_proxy=-1.00`. This is a sentinel value, not a real score. If the proxy evaluator is returning −1.00 for all infeasible genomes (short-circuit on HC_Total > 0), then NSGA-II has no gradient signal at all — it is performing random walk, not directed search.

**Probe:** Inspect `UltraCrewEvaluator` source for the infeasibility short-circuit path. Determine whether proxy scores are computed for infeasible genomes or zeroed/sentineled.

---

## 4. Instrumentation Plan

### Step 1: HC_Total Distribution Probe

Add a `hc_distribution` sampling block to `inrc_archive_forensics.rs` that records, at every census checkpoint:
- `min_hc`, `max_hc`, `mean_hc`, `median_hc` across all archive members
- Count of members with HC_Total = 0 (feasible), ≤ 5, ≤ 10, ≤ 20, ≤ 50, > 50

Output: `hc_distribution.jsonl` (one record per census checkpoint).

This directly tests RC-1 (no progress) and RC-3 (initialization depth).

### Step 2: Proxy–Feasibility Correlation Probe

At gen 5000, for all archive members, record `(hc_total, o1, o2, o3, o4, o5)` tuples. Compute Pearson ρ for each proxy vs HC_Total.

Output: `proxy_correlation.jsonl` (one record per archive member at gen 5000) + correlation summary in `sd007_landscape_report.md`.

This directly tests RC-2 (proxy misalignment).

### Step 3: Evaluator Source Inspection

Read `adapters/ultracrew/src/inrc/` evaluator source. Locate the infeasibility handling path. Determine whether proxy scores are computed for HC_Total > 0 genomes.

Output: Code citation in `sd007_landscape_report.md` Section 3.

This directly tests RC-4 (evaluator anomaly).

### Step 4: Initialization Depth Snapshot

Add a gen=0 snapshot to the forensics binary that records HC_Total for all initial archive members before any mutation occurs.

Output: `init_snapshot.jsonl` + summary in `sd007_landscape_report.md` Section 4.

This directly tests RC-3 (initialization depth).

---

## 5. Target Artifact

**`sd007_landscape_report.md`** — 5-section report:

1. **HC_Total Trajectory** — min/max/mean/median HC_Total across 5000 gens. Is the search making progress toward feasibility?
2. **Proxy–Feasibility Correlation** — ρ(Oi, HC_Total) for i=1..5. Are proxies aligned with feasibility?
3. **Evaluator Anomaly Assessment** — Is `best_proxy=-1.00` a sentinel or a real score? Code citation.
4. **Initialization Depth** — HC_Total distribution at gen=0. How far is the starting population from feasibility?
5. **Root Cause Classification** — Which RC(s) are confirmed? What is the recommended fix?

---

## 6. Frozen Classification Table

| Classification | Criterion |
|----------------|-----------|
| **RC-1 Confirmed: Operator Incapacity** | HC_Total shows no downward trend over 5000 gens (slope ≈ 0 or positive) |
| **RC-2 Confirmed: Proxy Misalignment** | \|ρ(Oi, HC_Total)\| < 0.1 for all i=1..5 |
| **RC-3 Confirmed: Initialization Depth** | Median HC_Total at gen=0 > 50 AND HC_Total at gen=5000 ≈ HC_Total at gen=0 |
| **RC-4 Confirmed: Evaluator Anomaly** | Evaluator source confirms sentinel return for HC_Total > 0 |
| **Multiple RC** | Two or more of the above confirmed simultaneously |
| **SD-007 Falsified** | HC_Total shows clear downward trend but stochastic bad luck; recommend longer run |

---

## 7. Exit Criterion

Sprint 3.9 is complete when:
1. `sd007_landscape_report.md` is produced with all 5 sections populated
2. At least one RC is confirmed or falsified with code/data citation
3. `sd007_resolution.md` is written and committed to `governance-hardening`

---

## 8. Anti-Patterns

- **Do not** classify SD-007 without HC_Total trajectory data — "mutation is probably broken" is not evidence
- **Do not** inspect evaluator source before running the HC_Total probe — the probe may reveal the answer without source inspection
- **Do not** open new defects for RC-2/RC-3/RC-4 until RC-1 is confirmed or falsified — operator incapacity is the most parsimonious explanation
- **Do not** modify `UltraCrewMutator` or `UltraCrewEvaluator` during this sprint — forensics only, no fixes

---

## 9. Commit History

| Commit | Description |
|--------|-------------|
| `19233818` | Sprint 3.8 charter (`sd005_sprint38_charter.md`) |
| `6a16df0f` | FeasibleLifecycle tracking + report writer |
| `2d9919ba` | `feasible_lineage_report.md` (seed=61, 5000 gens) |
| `67db87c0` | `sd005_resolution.md` (SD-005 CLOSED, SD-007 OPENED) |
| *(pending)* | Sprint 3.9 charter (`sd007_sprint39_charter.md`) |

---

## 10. Scientific Debt Ledger

| Defect | Status | Sprint Closed |
|--------|--------|---------------|
| SD-003 | CLOSED | Sprint 3.6 |
| SD-005 | CLOSED | Sprint 3.8 |
| SD-006 | CLOSED | Sprint 3.7 |
| **SD-007** | **OPEN** | — |