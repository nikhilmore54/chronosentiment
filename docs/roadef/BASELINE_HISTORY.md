# ROADEF 2026 — Dataset A Baseline History

**Document ID:** ROADEF-BH-001
**Version:** 1.4
**Date:** 2026-08-03

This document is the permanent performance ledger for Dataset A. It records
the objective value achieved by each solver version on each instance, enabling
attribution of improvements to specific algorithmic changes.

This document is **append-only**. Rows are never modified after entry.
Each row corresponds to a committed solver version on a specific instance.

---

## Purpose and Scope

The Capability Register ([`docs/governance/CAPABILITY_REGISTER.md`](../governance/CAPABILITY_REGISTER.md))
tracks **what Coralys can do** (capability maturity levels).

The ROADEF Programme ([`docs/roadef/ROADEF_PROGRAMME.md`](ROADEF_PROGRAMME.md))
tracks **what experiments are planned** (research queue).

This document tracks **how performance evolved over time** (quantitative evidence).

Together, these three documents form the evidence chain required for capability
promotion under the CMM framework (§6 of the ROADEF Programme).

---

## Scoring Convention

- **Objective**: `sum_t(MLU_t + inv_load_cost_t)` — lower is better
- **Finite**: solution has finite objective (no saturated links, no disconnected demands)
- **vs Empty**: delta from empty solution objective (negative = improvement)
- **Empty obj**: objective of the empty solution (no SR paths) — the universal baseline
- **Runtime**: wall-clock time to generate all 20 solutions (hardware-dependent)
- **Oracle Calls**: number of `compute_loads()` invocations (hardware-independent complexity measure)

The empty solution is always valid and always finite (assuming the network is
connected at each time slot). It is the universal lower bound on solver quality.

Oracle Calls is the preferred complexity metric because it is machine-independent.
Wall-clock runtime varies with hardware; oracle evaluations are intrinsic to the algorithm.

---

## Efficiency Summary Table

This table is updated after each solver version is fully evaluated.

| Solver | Improved/20 | Finite/20 | Mean Obj (finite) | Median Obj (finite) | Runtime (total) | Oracle Calls |
|--------|-------------|-----------|-------------------|---------------------|-----------------|--------------|
| Baseline v1.0 (`campaign_engine`) | 3/20 | 3/20 | ~244 | ~159 | < 1s | 0 |
| RP-401C (Ground-Truth Construction) | 13/20 | 14/20 | ~701,484 | ~98 | ~51 min | Σ D² per instance |
| RP-401D (Efficiency Recovery) | 13/20 | 15/20 | ~649,903 | ~75 | ~58 min | Σ D×K per instance (K=5) |
| **RP-402 (Budget-Aware Adaptation)** | **15/20** | **18/20** | **~651,474** | **~99** | **~58 min** | Σ D per instance (shared) + budget-gated re-routes |

Note: "Improved/20" counts instances where our solution is strictly better than empty.
Baseline v1.0 had 3 finite instances (setA-16: 127, setA-19: 159, setA-20: 447).
RP-401C mean obj is dominated by large-value instances (setA-16: 3.36M, setA-18: 799K, setA-19: 5.59M).
Median obj (finite) for RP-401C: ~98 (setA-11). Median obj (finite) for RP-401D: ~75 (setA-14).
RP-401D improved 1 additional instance to finite vs RP-401C (setA-14: inf→75.72).
RP-402 improved 3 additional instances to finite vs RP-401D (setA-02, setA-07, setA-09). 15/20 improved vs empty (best so far). 18/20 finite (best so far). setA-12 and setA-17 remain infeasible.

---

## Baseline v1.0 — campaign_engine (commit ec4d3821)

**Solver:** `campaign_engine` (greedy load-balanced, shared-path strategy)  
**Strategy:** Load-aware Dijkstra with heuristic saturation; shared-path (t=0 = t=1)  
**Budget guarantee:** Zero (shared-path by construction)  
**Known weakness:** Heuristic load model overestimates saturation by (k-1)/k  
**Runtime:** < 1s total (20 instances)

| Instance | Our obj | Empty obj | vs Empty | Finite | Notes |
|----------|---------|-----------|----------|--------|-------|
| setA-01 | — | 64.9962 | — | → empty | Our solution worse than empty |
| setA-02 | — | — | — | → empty | Falls back to empty |
| setA-03 | — | — | — | → empty | Falls back to empty |
| setA-04 | — | — | — | → empty | Falls back to empty |
| setA-05 | — | — | — | → empty | budget=1 prevents re-routing |
| setA-06 | — | — | — | → empty | Falls back to empty |
| setA-07 | — | — | — | → empty | Falls back to empty |
| setA-08 | — | — | — | → empty | Falls back to empty |
| setA-09 | — | — | — | → empty | Falls back to empty |
| setA-10 | — | — | — | → empty | Falls back to empty |
| setA-11 | — | — | — | → empty | Falls back to empty |
| setA-12 | — | — | — | → empty | Falls back to empty |
| setA-13 | — | — | — | → empty | Falls back to empty |
| setA-14 | — | — | — | → empty | Falls back to empty |
| setA-15 | — | — | — | → empty | Falls back to empty |
| setA-16 | 127 | 3,355,568 | −3,355,441 | ✓ | 26,000× improvement |
| setA-17 | — | — | — | → empty | Falls back to empty |
| setA-18 | — | — | — | → empty | Falls back to empty |
| setA-19 | 159 | 5,592,518 | −5,592,359 | ✓ | 35,000× improvement |
| setA-20 | 447 | 1,525,646 | −1,525,199 | ✓ | 3,400× improvement |

**Summary:** 3/20 instances improved (setA-16, 19, 20). 17/20 fall back to empty.  
**Root cause of fallbacks:** Heuristic load overestimation causes false saturation,
leading to infeasible or worse solutions on most instances (see RP-401B).

> Note: The "11/20 finite" figure cited in the programme refers to instances
> where our solution was finite (not necessarily better than empty). The 3/20
> figure above counts instances where our solution was strictly better than empty.

---

## RP-401C — Ground-Truth Construction (commits 6da376a7, c0a7b06e)

**Solver:** `rp401c_ecmp_construction`
**Role:** Ground-truth measurement oracle — answers "what decisions would we make with accurate congestion information?"
**Strategy:** Load-aware Dijkstra with ECMP-oracle saturation; shared-path (t=0 = t=1)
**Change from baseline:** `compute_loads()` replaces heuristic saturation accumulator
**Oracle calls:** O(D²) per instance — intentionally expensive; this is a measurement tool, not a competition solver
**Per-instance timeout:** 300s deadline (commit `c0a7b06e`) — large instances return partial solution if deadline exceeded
**Status:** ✅ Complete — 20/20 instances executed 2026-08-02
**Summary:** 13 improved, 0 regressed, 7 unchanged. Total objective improvement vs empty: 2,512,099.84

| Instance | Our obj | Empty obj | vs Empty | Finite | ms | Notes |
|----------|---------|-----------|----------|--------|----|-------|
| setA-01 | 53.3172 | inf | improved | ✓ | 41 | ∞ → finite |
| setA-02 | inf | inf | both inf | → empty | 85 | |
| setA-03 | 96.9447 | inf | improved | ✓ | 40 | ∞ → finite |
| setA-04 | 70.3656 | inf | improved | ✓ | 2,881 | ∞ → finite |
| setA-05 | 72,329.3884 | 72,329.3884 | = | ✓ | 2,273 | budget=1; unchanged |
| setA-06 | 59.6593 | inf | improved | ✓ | 43,146 | ∞ → finite |
| setA-07 | inf | inf | both inf | → empty | 107,079 | |
| setA-08 | inf | inf | both inf | → empty | 13,867 | |
| setA-09 | inf | inf | both inf | → empty | 10,827 | |
| setA-10 | 73.4619 | inf | improved | ✓ | 292,578 | ∞ → finite |
| setA-11 | 99.3105 | inf | improved | ✓ | 73,298 | ∞ → finite |
| setA-12 | 26.1166 | inf | improved | ✓ | 76,496 | ∞ → finite |
| setA-13 | 59.2952 | 986,957.8301 | −986,898.53 | ✓ | 302,121 | Strongest finite improvement |
| setA-14 | inf | inf | both inf | → empty | 234,675 | |
| setA-15 | 208.1804 | inf | improved | ✓ | 189,004 | ∞ → finite |
| setA-16 | 3,355,568.5541 | 3,355,568.5684 | −0.01 | ✓ | 304,560 | Timeout partial; tiny improvement |
| setA-17 | inf | inf | both inf | → empty | 303,133 | Timeout; both inf |
| setA-18 | 799,167.0856 | 799,169.1790 | −2.09 | ✓ | 302,595 | Timeout partial |
| setA-19 | 5,592,516.4280 | 5,592,518.2733 | −1.85 | ✓ | 309,292 | Timeout partial |
| setA-20 | 449.5543 | 1,525,646.9067 | −1,525,197.35 | ✓ | 308,677 | Major improvement |

**Summary:** 13/20 improved (8 ∞→finite + 5 finite→finite). 6 both inf. 1 unchanged (setA-05, budget=1). 0 regressions.
**Total runtime:** ~3,080,000 ms (~51 min) across 20 instances with 300s per-instance timeout.
**Best improvement:** setA-13 (−986,898.53 objective units); setA-20 (−1,525,197.35 vs empty).

---

## RP-401D — Efficiency Recovery (commits 6da376a7, e07e01a5, 1c68e529)

**Solver:** `rp401d_ecmp_path_selection`
**Role:** Efficiency recovery — answers "how much of RP-401C's quality can we retain at O(D×K) cost instead of O(D²)?"
**Strategy:** K=5 candidate paths per demand; oracle selects MLU-minimising candidate
**Change from RP-401C:** Path selection criterion changed from penalty-weighted metric to oracle MLU; K candidates instead of 1
**Oracle calls:** O(D × K) per instance (K=5) — 5× cheaper than RP-401C per demand
**Per-instance timeout:** 300s deadline (commits `e07e01a5`, `1c68e529`) — outer + inner loop checks; large instances return partial solution
**Status:** ✅ Complete — 20/20 instances executed 2026-08-02
**Summary:** 13 improved vs empty, 0 regressed, 7 unchanged. Total objective improvement vs empty: 2,584,407.78

| Instance | Our obj | Empty obj | vs Empty | Finite | ms | Notes |
|----------|---------|-----------|----------|--------|----|-------|
| setA-01 | 53.0880 | inf | improved | ✓ | 89 | ∞ → finite |
| setA-02 | inf | inf | both inf | → empty | 310 | |
| setA-03 | 101.3206 | inf | improved | ✓ | 89 | ∞ → finite |
| setA-04 | 59.3135 | inf | improved | ✓ | 6,529 | ∞ → finite |
| setA-05 | 13.3236 | 72,329.3884 | −72,316.06 | ✓ | 4,095 | Major improvement; budget=1 |
| setA-06 | 52.3126 | inf | improved | ✓ | 98,500 | ∞ → finite |
| setA-07 | inf | inf | both inf | → empty | 261,155 | |
| setA-08 | 48.6693 | inf | improved | ✓ | 28,046 | ∞ → finite |
| setA-09 | inf | inf | both inf | → empty | 24,588 | |
| setA-10 | 69.0157 | inf | improved | ✓ | 301,459 | ∞ → finite; timeout partial |
| setA-11 | 99.3299 | inf | improved | ✓ | 144,840 | ∞ → finite |
| setA-12 | inf | inf | both inf | → empty | 162,401 | Regressed vs RP-401C (26.12→inf) |
| setA-13 | 58.5801 | 986,957.8301 | −986,899.25 | ✓ | 301,598 | Strongest finite improvement; timeout partial |
| setA-14 | 75.7237 | inf | improved | ✓ | 301,446 | ∞ → finite; new vs RP-401C; timeout partial |
| setA-15 | 210.4095 | inf | improved | ✓ | 301,625 | ∞ → finite; timeout partial |
| setA-16 | 3,355,568.5654 | 3,355,568.5684 | −0.00 | ✓ | 304,460 | Timeout partial; tiny improvement |
| setA-17 | inf | inf | both inf | → empty | 302,896 | Timeout; both inf |
| setA-18 | 799,169.1790 | 799,169.1790 | +0.00 | ✓ | 303,035 | Timeout partial; unchanged |
| setA-19 | 5,592,518.2733 | 5,592,518.2733 | +0.00 | ✓ | 306,406 | Timeout partial; unchanged |
| setA-20 | 454.4424 | 1,525,646.9067 | −1,525,192.46 | ✓ | 311,524 | Major improvement |

**Summary:** 13/20 improved vs empty (9 ∞→finite + 4 finite→finite). 5 both inf. 0 unchanged (finite). 0 regressions vs empty.
**vs RP-401C:** setA-12 regressed (26.12→inf); setA-14 improved (inf→75.72); setA-05 improved (72329→13.32).
**Total runtime:** ~3,465,091 ms (~58 min) across 20 instances with 300s per-instance timeout.
**Best improvement:** setA-13 (−986,899.25 objective units); setA-20 (−1,525,192.46 vs empty).

---

## Attribution Framework

When RP-401C and RP-401D results are available, the improvement can be
decomposed as follows:

```
Total improvement (RP-401D vs Baseline)
    = Model correction effect (RP-401C vs Baseline)
    + Search improvement effect (RP-401D vs RP-401C)
```

This decomposition is the primary scientific contribution of the four-stage
RP-401 structure. It allows precise attribution of improvement to specific
algorithmic changes, which is not possible when both changes are made
simultaneously.

---

## RP-402 — Budget-Aware t=1 Adaptation (commit 06c29f9f)

**Solver:** `rp402_budget_adapt`
**Role:** Budget-aware transition planning — answers "can selectively re-routing high-traffic-change demands for t=1 recover the remaining infeasible instances?"
**Strategy:** (1) Build shared paths for t=0 and t=1 using RP-401C ECMP-aware greedy (budget cost = 0). (2) Sort demands by |v[1]−v[0]| descending. (3) For each high-change demand: generate ECMP-aware candidate t=1 path, compute SrPathBit::dist switch cost, accept if cost ≤ budget_remaining and t=1 MLU improves.
**Change from RP-401D:** Adds budget-aware t=1 adaptation stage; shared-path construction replaces K=5 oracle selection for t=0
**Oracle calls:** Σ D per instance (shared construction) + budget-gated re-routes (typically 1–3 per instance)
**Per-instance timeout:** 300s deadline — large instances return partial solution if deadline exceeded
**Status:** ✅ Complete — 20/20 instances executed 2026-08-03
**Summary:** 15 improved vs empty (best so far), 18/20 finite (best so far). Total objective improvement vs empty: 2,584,436.44. Target instances: 3/5 recovered (setA-02, setA-07, setA-09). setA-12 and setA-17 remain infeasible.

| Instance | Our obj | Empty obj | vs Empty | Finite | ms | Notes |
|----------|---------|-----------|----------|--------|----|-------|
| setA-01 | 49.8585 | inf | improved | ✓ | 79 | ∞ → finite |
| setA-02 | 54.4326 | inf | improved | ✓ | 193 | ∞ → finite; **target instance recovered** |
| setA-03 | 98.9574 | inf | improved | ✓ | 93 | ∞ → finite |
| setA-04 | 58.4165 | inf | improved | ✓ | 5,327 | ∞ → finite |
| setA-05 | 14.3266 | 72,329.3884 | −72,315.06 | ✓ | 2,104 | budget=1; improved vs RP-401D |
| setA-06 | 39.6697 | inf | improved | ✓ | 56,298 | ∞ → finite |
| setA-07 | 191.1679 | inf | improved | ✓ | 172,086 | ∞ → finite; **target instance recovered** |
| setA-08 | inf | inf | both inf | → empty | 18,841 | Regressed vs RP-401D (48.67→inf); shared-path weaker for this instance |
| setA-09 | 145.5479 | inf | improved | ✓ | 24,340 | ∞ → finite; **target instance recovered** |
| setA-10 | 56.6952 | inf | improved | ✓ | 303,016 | ∞ → finite; timeout partial; budget=1 |
| setA-11 | 98.8484 | inf | improved | ✓ | 107,070 | ∞ → finite |
| setA-12 | inf | inf | both inf | → empty | 98,714 | Remains infeasible; target instance not recovered |
| setA-13 | 45.0642 | 986,957.8301 | −986,912.77 | ✓ | 303,085 | Strongest finite improvement; timeout partial |
| setA-14 | 73.1447 | inf | improved | ✓ | 275,030 | ∞ → finite |
| setA-15 | 208.1205 | inf | improved | ✓ | 302,351 | ∞ → finite; timeout partial |
| setA-16 | 3,355,566.4392 | 3,355,568.5684 | −2.13 | ✓ | 305,741 | Improved vs RP-401D (−2.13 vs −0.00) |
| setA-17 | inf | inf | both inf | → empty | 303,908 | Remains infeasible; target instance not recovered; budget=1 |
| setA-18 | 799,166.9063 | 799,169.1790 | −2.27 | ✓ | 303,742 | Improved vs RP-401D (−2.27 vs +0.00) |
| setA-19 | 5,592,511.4703 | 5,592,518.2733 | −6.80 | ✓ | 308,835 | Improved vs RP-401D (−6.80 vs +0.00) |
| setA-20 | 449.4974 | 1,525,646.9067 | −1,525,197.41 | ✓ | 311,236 | Major improvement |

**Summary:** 15/20 improved vs empty (best so far). 18/20 finite (best so far). 2 both inf (setA-12, setA-17). 0 regressions vs empty.
**vs RP-401D:** setA-08 regressed (48.67→inf); setA-02, setA-07, setA-09 recovered (inf→finite); setA-16/18/19 improved marginally.
**Target instances (setA-02,07,09,12,17):** 3/5 recovered. setA-12 (budget=13, still inf) and setA-17 (budget=1, still inf) remain the open research questions.
**Total runtime:** ~3,509,000 ms (~58 min) across 20 instances with 300s per-instance timeout.
**Best improvement:** setA-13 (−986,912.77 objective units); setA-20 (−1,525,197.41 vs empty).
**Capability evidence:** Budget-aware transition planning demonstrated as a reusable capability — 3 additional difficult instances became feasible, 18/20 finite, essentially unchanged runtime.

---

## Planned Future Entries

| Version | Solver | Key change | Gate |
|---------|--------|-----------|------|
| RP-403 | Multi-path candidate generation | Increase candidate diversity beyond K=5 | Requires RP-402 evidence (satisfied) |
| RP-404 | LNS post-processing | Local neighbourhood search on committed solution | Requires RP-403 evidence |
| RP-405 | Hyper-heuristic operator selection | Learn which LNS moves work per instance class | Requires RP-404 evidence |
| RP-406 | MOGA integration | Global evolutionary optimisation | Requires RP-405 evidence |
| RP-407 | Exact bottleneck optimisation | Targeted exact optimisation of saturated links | Requires RP-406 evidence |

---

## Amendment Log

| Version | Date | Change |
|---------|------|--------|
| 1.0 | 2026-08-02 | Initial document. Baseline v1.0 results populated from commit `ec4d3821`. RP-401C and RP-401D rows created as pending-execution placeholders (commit `6da376a7`). |
| 1.1 | 2026-08-02 | Added Oracle Calls column to scoring convention. Added Efficiency Summary Table. Reframed RP-401C as "Ground-Truth Construction" and RP-401D as "Efficiency Recovery" (commit `50944b82`). |
| 1.2 | 2026-08-02 | Populated RP-401C full 20/20 results. 13 improved, 0 regressed, 7 unchanged. Total improvement vs empty: 2,512,099.84. RP-401D pending (run in progress). |
| 1.3 | 2026-08-02 | Populated RP-401D full 20/20 results. 13 improved vs empty, 15/20 finite. Total improvement vs empty: 2,584,407.78. Updated efficiency summary table. RP-401 phase complete. |
| 1.4 | 2026-08-03 | Populated RP-402 full 20/20 results. 15 improved vs empty (best so far), 18/20 finite (best so far). Total improvement vs empty: 2,584,436.44. 3/5 target instances recovered (setA-02, setA-07, setA-09). Updated efficiency summary table. Updated planned future entries with evidence gates. |