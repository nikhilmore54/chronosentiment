# RP-401 Final Report — ECMP Oracle Integration

**Document ID:** ROADEF-RP-401-FINAL  
**Version:** 1.0 (draft — pending full Dataset A execution)  
**Date:** 2026-08-02  
**Status:** In progress — RP-401C at 15/20 instances; RP-401D pending

---

## 1. Research Question

> Does replacing the heuristic link-load estimator with the verified ECMP oracle
> during greedy construction improve solution quality on Dataset A?
> If so, is the improvement attributable to model fidelity rather than search capability?

---

## 2. Hypothesis

The baseline solver (`campaign_engine`) uses a heuristic load estimator that
accumulates demand volume along Dijkstra paths without accounting for ECMP
traffic splitting. RP-401B quantified this error: the heuristic overestimates
link saturation by a factor of (k−1)/k on k-way ECMP paths.

**Hypothesis:** This overestimation causes the construction heuristic to avoid
paths that are actually feasible under ECMP, producing infeasible or suboptimal
solutions. Replacing the estimator with the ECMP oracle should convert
infeasible instances to feasible ones without any change to the search strategy.

---

## 3. Experimental Design

RP-401 was structured as four sequential stages to isolate causal effects:

| Stage | Binary | Purpose |
|-------|--------|---------|
| RP-401A | `rp401a_ecmp_oracle_verification` | Verify oracle correctness |
| RP-401B | (analysis) | Quantify heuristic vs oracle divergence |
| RP-401C | `rp401c_ecmp_construction` | Replace estimator; measure effect |
| RP-401D | `rp401d_ecmp_path_selection` | Efficiency recovery (K-candidate oracle selection) |

**Control:** The search algorithm (load-aware Dijkstra, demand ordering, shared-path strategy) is identical across Baseline and RP-401C. Only the load model changes.

---

## 4. Results

### 4.1 RP-401A — Oracle Verification

The ECMP oracle (`evaluator.compute_loads()`) was verified to produce results
consistent with the official evaluator on all tested instances. See
[`rp401a_ecmp_oracle_verification.md`](rp401a_ecmp_oracle_verification.md).

### 4.2 RP-401B — Load Divergence

The heuristic overestimates link saturation by (k−1)/k on k-way ECMP paths.
On instances with high ECMP fan-out, this causes the construction heuristic to
treat feasible paths as saturated. See
[`rp401b_load_divergence_report.md`](rp401b_load_divergence_report.md).

### 4.3 RP-401C — Ground-Truth Construction (Dataset A)

**Status:** 15/20 instances complete as of 2026-08-02 11:18 IST.

| Instance | Baseline obj | RP-401C obj | Result |
|----------|-------------|-------------|--------|
| setA-01 | ∞ | 53.3172 | ✓ feasible |
| setA-02 | ∞ | ∞ | both inf |
| setA-03 | ∞ | 96.9447 | ✓ feasible |
| setA-04 | ∞ | 70.3656 | ✓ feasible |
| setA-05 | 72329.3884 | 72329.3884 | = |
| setA-06 | ∞ | 59.6593 | ✓ feasible |
| setA-07 | ∞ | ∞ | both inf |
| setA-08 | ∞ | ∞ | both inf |
| setA-09 | ∞ | ∞ | both inf |
| setA-10 | ∞ | 73.4619 | ✓ feasible |
| setA-11 | ∞ | 99.3105 | ✓ feasible |
| setA-12 | ∞ | 26.1166 | ✓ feasible |
| setA-13 | 986957.8301 | 58.1530 | ✓ −986,899 |
| setA-14 | ∞ | ∞ | both inf |
| setA-15 | ∞ | 208.1804 | ✓ feasible |
| setA-16 | pending | pending | pending |
| setA-17 | pending | pending | pending |
| setA-18 | pending | pending | pending |
| setA-19 | pending | pending | pending |
| setA-20 | pending | pending | pending |

**Partial summary (15/20):**
- Instances improved: 9 (8 ∞→finite + 1 massive finite reduction)
- Both still ∞: 5
- Unchanged (finite): 1 (setA-05, budget=1)
- No regressions observed

### 4.4 RP-401D — Efficiency Recovery (Dataset A)

**Status:** Pending (awaiting RP-401C completion).

RP-401D uses K=5 candidate paths per demand, evaluated by the oracle, selecting
the MLU-minimising candidate. Oracle calls: O(D×K) vs O(D²) for RP-401C.

---

## 5. Summary Statistics

*To be populated after full execution. Run:*
```bash
python3 scripts/rp401_populate_baseline_history.py /tmp/rp401c_output.txt /tmp/rp401d_output.txt
```

| Metric | Baseline v1.0 | RP-401C | RP-401D |
|--------|---------------|---------|---------|
| Instances improved / 20 | 3 | pending | pending |
| Previously ∞ → finite | 0 | pending | pending |
| Both still ∞ | 17 | pending | pending |
| Finite instances | 3 | pending | pending |
| Mean obj (finite) | ~244 | pending | pending |
| Median obj (finite) | ~159 | pending | pending |
| Best improvement | setA-16: −3,355,441 | setA-13: −986,899 (partial) | pending |
| Total runtime | < 1s | pending | pending |
| Oracle calls | 0 | Σ D² | Σ D×K (K=5) |

---

## 6. Attribution Analysis

The key scientific result is the attribution of improvement to model fidelity:

| Factor | Changed in RP-401C? | Effect observed |
|--------|---------------------|-----------------|
| Search algorithm | No | — |
| Demand ordering | No | — |
| Path selection metric | No | — |
| Load model | **Yes** (heuristic → ECMP oracle) | 9+ instances improved |

This confirms the hypothesis: **the dominant bottleneck was modelling fidelity,
not search capability.**

The improvement from RP-401C to RP-401D (when measured) will isolate the
additional effect of oracle-guided path selection.

---

## 7. Capability Review

This section records the formal capability promotion assessment for
**ECMP-aware flow estimation** (see
[`docs/governance/CAPABILITY_REGISTER.md`](../governance/CAPABILITY_REGISTER.md)).

### 7.1 Promotion Criteria (C1 → C2)

| Criterion | Required | Status |
|-----------|----------|--------|
| Implementation exists | ✓ | ✓ Complete (`rp401c_ecmp_construction.rs`) |
| Oracle verified | ✓ | ✓ RP-401A |
| Divergence quantified | ✓ | ✓ RP-401B |
| Benchmark executed on Dataset A (20/20) | ✓ | ⏳ 15/20 complete |
| Improvement reproduced (re-run with tee) | ✓ | ⏳ Pending |
| No regression observed | ✓ | ✓ (partial — 15/20) |
| Results recorded in BASELINE_HISTORY | ✓ | ⏳ Pending |
| Capability recommendation filed | ✓ | ⏳ This document |
| Governance approval | ✓ | ⏳ Pending |

### 7.2 Recommendation

Based on the partial evidence (15/20 instances), the results are strongly
consistent with C2 promotion criteria. The recommendation is:

> **Promote ECMP-aware flow estimation from C1 to C2 (Benchmark Validated)**
> once all 20 Dataset A instances are complete and results are recorded in
> BASELINE_HISTORY.md.

This promotion should be recorded in the Capability Register with a reference
to this document and the BASELINE_HISTORY entry.

### 7.3 Scope of Capability

The capability being promoted is:

> **Accurate ECMP-aware incremental load estimation for constructive network routing.**

This capability is applicable beyond ROADEF to: traffic engineering,
multi-commodity routing, SDN optimization, and other network optimization
domains where ECMP is the forwarding model.

---

## 8. Next Steps

1. Complete RP-401C (20/20 instances)
2. Execute RP-401D
3. Populate `BASELINE_HISTORY.md` with full results
4. Update this report with final summary statistics
5. Promote ECMP-aware routing C1→C2 in Capability Register
6. Freeze RP-401
7. Begin RP-402 (budget-aware t=1 adaptation)

---

## 9. Amendment Log

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.0 | 2026-08-02 | Lyzo | Initial draft (15/20 instances complete) |