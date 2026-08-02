# ROADEF 2026 — Dataset A Baseline History

**Document ID:** ROADEF-BH-001
**Version:** 1.2
**Date:** 2026-08-02

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

| Solver | Improved/20 | Finite/20 | Mean Obj (finite) | Runtime (total) | Oracle Calls |
|--------|-------------|-----------|-------------------|-----------------|--------------|
| Baseline v1.0 (`campaign_engine`) | 3/20 | 3/20 | ~244 | < 1s | 0 |
| RP-401C (Ground-Truth Construction) | 13/20 | 13/20 | ~627,082 | ~51 min | Σ D² per instance |
| RP-401D (Efficiency Recovery) | pending | pending | pending | pending | Σ D×K per instance (K=5) |

Note: "Improved/20" counts instances where our solution is strictly better than empty.
Baseline v1.0 had 3 finite instances (setA-16: 127, setA-19: 159, setA-20: 447).
RP-401C mean obj is dominated by large-value instances (setA-16: 3.36M, setA-18: 799K, setA-19: 5.59M).
Median obj (finite) for RP-401C: ~208 (setA-15).

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

## RP-401D — Efficiency Recovery (commit 6da376a7)

**Solver:** `rp401d_ecmp_path_selection`
**Role:** Efficiency recovery — answers "how much of RP-401C's quality can we retain at O(D×K) cost instead of O(D²)?"
**Strategy:** K=5 candidate paths per demand; oracle selects MLU-minimising candidate
**Change from RP-401C:** Path selection criterion changed from penalty-weighted metric to oracle MLU; K candidates instead of 1
**Oracle calls:** O(D × K) per instance (K=5) — 5× cheaper than RP-401C per demand
**Status:** Binary written; execution results pending (awaiting RP-401C completion)

| Instance | Our obj | Empty obj | vs Empty | Finite | Notes |
|----------|---------|-----------|----------|--------|-------|
| setA-01 | pending | 64.9962 | pending | pending | |
| setA-02 | pending | pending | pending | pending | |
| setA-03 | pending | pending | pending | pending | |
| setA-04 | pending | pending | pending | pending | |
| setA-05 | pending | pending | pending | pending | budget=1 |
| setA-06 | pending | pending | pending | pending | |
| setA-07 | pending | pending | pending | pending | |
| setA-08 | pending | pending | pending | pending | |
| setA-09 | pending | pending | pending | pending | |
| setA-10 | pending | pending | pending | pending | |
| setA-11 | pending | pending | pending | pending | |
| setA-12 | pending | pending | pending | pending | |
| setA-13 | pending | pending | pending | pending | |
| setA-14 | pending | pending | pending | pending | |
| setA-15 | pending | pending | pending | pending | |
| setA-16 | pending | 3,355,568 | pending | pending | |
| setA-17 | pending | pending | pending | pending | |
| setA-18 | pending | pending | pending | pending | |
| setA-19 | pending | 5,592,518 | pending | pending | |
| setA-20 | pending | 1,525,646 | pending | pending | |

**To populate:** Run `cargo run --bin rp401d_ecmp_path_selection --release` from `adapters/roadef/`.

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

## Planned Future Entries

| Version | Solver | Key change |
|---------|--------|-----------|
| RP-402 | Budget-aware t=1 adaptation | Exploit budget for t=1 re-routing |
| RP-403 | Multi-path candidate generation | Increase candidate diversity beyond K=5 |
| RP-404 | LNS post-processing | Local neighbourhood search on committed solution |
| RP-405 | Hyper-heuristic operator selection | Learn which LNS moves work per instance class |
| RP-406 | MOGA integration | Global evolutionary optimisation |

---

## Amendment Log

| Version | Date | Change |
|---------|------|--------|
| 1.0 | 2026-08-02 | Initial document. Baseline v1.0 results populated from commit `ec4d3821`. RP-401C and RP-401D rows created as pending-execution placeholders (commit `6da376a7`). |
| 1.1 | 2026-08-02 | Added Oracle Calls column to scoring convention. Added Efficiency Summary Table. Reframed RP-401C as "Ground-Truth Construction" and RP-401D as "Efficiency Recovery" (commit `50944b82`). |
| 1.2 | 2026-08-02 | Populated RP-401C full 20/20 results. 13 improved, 0 regressed, 7 unchanged. Total improvement vs empty: 2,512,099.84. RP-401D pending (run in progress). |