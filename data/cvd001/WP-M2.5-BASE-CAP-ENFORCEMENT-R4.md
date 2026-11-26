# WP-M2.5 — Base-Cap Enforcement Semantics (R4)
## CVD-001 Benchmark Reconstruction Project — Milestone 2

**Document ID:** WP-M2.5-v1.0  
**Work Package:** WP-M2.5 (Base-Cap Enforcement Semantics — Research Question R4)  
**Status:** DRAFT  
**Governance baseline:** MILESTONE2-MATHEMATICAL-RECONSTRUCTION-PLAN-v1.0.md (frozen at `fc505cba`)  
**Notation baseline:** WP-M2.1-MATHEMATICAL-BENCHMARK-MODEL-v1.0.md (frozen at `e0407ded`)  
**Workload baseline:** WP-M2.2-CREDITED-WORKLOAD-EQUATION-R1.md (frozen at `eb38a8d0`)  
**Objective baseline:** WP-M2.3-OBJECTIVE-FUNCTION-R2.md (frozen at `34c84327`)  
**HC3 baseline:** WP-M2.4-HC3-MATHEMATICAL-DEFINITION-R3.md (frozen at `58ceae03`)  
**Evidence baseline:** Sprint 10 artifacts frozen at `721c086c`  
**Created:** 2026-07-17  
**Revised:** 2026-07-17 (B2 rationale softened — separate enforcement not excluded; absence of visible counter is negative evidence not proof of absence; ER-009 integrated into Evidence Review §2 and candidate analysis §3.2 before being cited in §3.4 synthesis; "simple per-crew comparison" consistently labelled as preferred aggregation reconstruction)

---

## 0. Research Question

**R4:** Validate the base-cap enforcement semantics — specifically, how the contractual credit base (W^min_n) and cap (W^max_n) are enforced in the benchmark, and whether enforcement occurs at the duty level, the monthly level, or both.

This work package inherits the open sub-questions from WP-M2.2 (Sub-questions A and C) and resolves them to the extent possible from available evidence. Classification tags are restricted to the frozen protocol: [Recovered] / [Derived] / [Hypothesized] / [Engineering approximation].

---

## 1. Inherited Open Sub-Questions

The following sub-questions were deferred from WP-M2.2 to this work package:

**Sub-question A (from WP-M2.2 §2.2):** Is the credit function linear in flight duration, or does it apply a cap at the duty level before monthly aggregation (cap-then-sum vs sum-then-cap)?

**Sub-question C (from WP-M2.2 §2.2):** Is the base-level aggregation (ER-007 Stage 5) a simple sum of individual crew workloads, or does it apply a secondary transformation?

Both sub-questions were classified [Hypothesized | Moderate] in WP-M2.2.

---

## 2. Evidence Review

### E1 — Evaluator Source Code

The evaluator source ([`adapters/roadef/src/evaluator.rs`](adapters/roadef/src/evaluator.rs)) implements:

```rust
// Monthly workload cap enforcement (HC3-A)
if workload[nurse_id] > contract.cap {
    hard_violations += 1;
}

// Workload balance penalty (objective)
let penalty = (workload[nurse_id] - target).abs();
total_cost += penalty;
```

**Key observations from E1:**
- The cap (W^max_n) is enforced as a hard constraint at the monthly level (HC3-A, confirmed in WP-M2.4)
- The workload balance penalty Δ_n = |W_n − t_n| is accumulated into `total_cost` — this penalizes both over-target and under-target deviations
- No duty-level cap is visible in the evaluator code path — accumulation is `workload[nurse_id] += assignment.weight` without intermediate capping
- No separate hard base violation counter is visible in the evaluator code path for W^min_n

**Evidence classification note:** E1 is an implementation artifact. Per the confidence calibration rule (WP-M2.1 §0), E1-only evidence is capped at High confidence. The absence of a visible base violation counter in E1 is negative evidence, not proof of absence.

### ER-007 — Credit Accumulation Pipeline

ER-007 Stage 3 (Duty Credit) establishes that a duty credit value c_t is computed before monthly aggregation. Stage 5 (Base-Level Aggregation) establishes that monthly totals are compared against contractual limits.

**Key finding:** The pipeline structure (Stage 3 → Stage 4 → Stage 5) is consistent with both cap-then-sum and sum-then-cap models. ER-007 does not resolve Sub-question A. Stage 5 describes comparison against contractual limits but does not specify whether base enforcement is hard or soft.

### ER-009 — Resource Model (Montréal Monthly Crew Rostering)

ER-009 confirms the standard Montréal monthly crew rostering resource model. In this model:
- The monthly workload cap is typically enforced as a hard constraint
- The monthly workload base is typically enforced as a soft target (workload equity objective) rather than a hard constraint
- The asymmetric enforcement pattern (hard cap, soft base) is characteristic of the Montréal model

**Relevance to R4:** ER-009 provides independent corroboration for the asymmetric enforcement model (hard cap, soft base) that is also suggested by E1. This corroboration is incorporated into the candidate analysis in §3.2.

**Evidence classification note:** ER-009 is a reconstructed resource model (Sprint 10 WP3-C). It provides convergent evidence for the enforcement pattern but is not authoritative benchmark documentation from G-2014-22.

### WP-M2.4 — HC3-A Preferred Reconstruction

HC3-A (W_n ≤ W^max_n) is the preferred reconstruction for the cap enforcement constraint. This is [Hypothesized | Moderate] overall, with constraint structure [Recovered | High].

---

## 3. Reconstruction of R4

### 3.1 Cap Enforcement (W^max_n)

**Preferred enforcement reconstruction:**

> Cap enforcement: W_n ≤ W^max_n for all n ∈ N (HC3-A, from WP-M2.4)

This is enforced at the monthly level. The evaluator source (E1) confirms `workload[nurse_id] > contract.cap` triggers a hard violation, with no duty-level cap visible in the accumulation path.

**Classification:** [Hypothesized | Moderate] — inherits from HC3-A (WP-M2.4) and W_n (WP-M2.2)

**Sub-question A resolution (cap-then-sum vs sum-then-cap):**

The evaluator source (E1) accumulates `workload[nurse_id] += assignment.weight` without an intermediate duty-level cap. This is consistent with a sum-then-cap model (monthly accumulation, then cap check). However, the cap could also be pre-computed into `assignment.weight` (cap-then-sum at the weight computation stage, invisible in the accumulation loop).

**Preferred enforcement reconstruction under the Minimal Reconstruction Principle:** sum-then-cap at the monthly level, because this is the most parsimonious interpretation of the visible code path.

**Classification of cap-then-sum vs sum-then-cap:** [Hypothesized | Moderate] — the visible code path supports sum-then-cap, but cap-then-sum cannot be excluded if the cap is pre-computed into assignment weights.

**Negative finding:** The exact enforcement mechanism (cap-then-sum vs sum-then-cap) is not definitively recoverable from E1 alone. Recoverability: Low.

---

### 3.2 Base Enforcement (W^min_n)

**Evidence assessment:**

The evaluator source (E1) references `contract.base` in the workload balance target computation (`(contract.base + contract.cap) / 2.0`). No separate hard base violation counter is visible in the evaluator code path. The workload balance penalty Δ_n = |W_n − t_n| (WP-M2.3) penalizes both over-target and under-target deviations, which includes under-base deviation when W_n < t_n.

The Montréal monthly crew rostering model (ER-009) characteristically enforces the base as a soft target rather than a hard constraint, with workload equity handled through the objective function rather than a hard violation counter. This pattern is consistent with the E1 evidence.

**Candidate formulations:**

| Candidate | Description | Evidence assessment | Classification | Confidence |
|-----------|-------------|---------------------|----------------|------------|
| B1: Hard constraint | W_n ≥ W^min_n enforced as hard violation | No hard base violation counter visible in E1; inconsistent with ER-009 Montréal model | [Hypothesized] | Low |
| B2: Soft penalty | W_n < W^min_n contributes to objective penalty via Δ_n | Consistent with E1 (Δ_n penalizes under-target); consistent with ER-009 (soft base in Montréal model) | [Hypothesized] | Moderate |
| B3: Target only | W^min_n used only to compute t_n; no direct enforcement | Consistent with E1 (no separate base enforcement visible); cannot be distinguished from B2 without further evidence | [Hypothesized] | Low |

**Preferred enforcement reconstruction under the Minimal Reconstruction Principle:** B2 (soft penalty via Δ_n). This is preferred because: (a) the reconstructed objective already penalizes deviations from the target workload, providing a parsimonious explanation for the absence of an independently evidenced base-enforcement mechanism; (b) this pattern is consistent with ER-009 (Montréal model soft base enforcement). However, separate enforcement mechanisms (B1 or B3) cannot be excluded from available evidence — the absence of a visible base violation counter in E1 is negative evidence, not proof of absence.

**Classification of base enforcement:** [Hypothesized | Moderate]

**Negative finding:** The exact base enforcement mechanism is not recoverable from current public artifacts. The preferred reconstruction B2 is the most parsimonious interpretation consistent with E1 and ER-009, but it is not confirmed from G-2014-22. Recoverability: Low.

---

### 3.3 Sub-question C Resolution (Base-Level Aggregation)

**Sub-question C (from WP-M2.2):** Is the base-level aggregation (ER-007 Stage 5) a simple sum of individual crew workloads, or does it apply a secondary transformation?

**Evidence assessment:**

ER-007 Stage 5 describes aggregation at the base level but does not specify the aggregation operator. The evaluator source (E1) computes workload per crew member and compares against contractual limits; no base-level secondary transformation is visible in the evaluator code path. The Montréal model (ER-009) uses per-crew workload comparison without a secondary base-level transformation.

**Preferred aggregation reconstruction under the Minimal Reconstruction Principle:** Base-level aggregation is a per-crew comparison (W_n against W^max_n for each n ∈ N), with no secondary transformation. This is consistent with HC3-A (WP-M2.4), E1, and ER-009.

**Classification:** [Hypothesized | Moderate] — the per-crew comparison is the most parsimonious interpretation; a secondary transformation cannot be excluded from available evidence.

**Negative finding:** A secondary base-level transformation is not evidenced but cannot be excluded. Recoverability: Low.

---

### 3.4 Complete Base-Cap Enforcement Model

Combining the preferred aggregation reconstructions from §3.1, §3.2, and §3.3:

**Preferred reconstruction of base-cap enforcement semantics:**

> Cap enforcement (hard): W_n ≤ W^max_n for all n ∈ N — [Hypothesized | Moderate]
>
> Base enforcement (soft, via objective): Δ_n = |W_n − t_n| penalizes under-base deviation — [Hypothesized | Moderate]
>
> Base-level aggregation: per-crew comparison, no secondary transformation — [Hypothesized | Moderate]

**Note on asymmetric enforcement:** The preferred reconstruction exhibits an asymmetric enforcement pattern — hard cap, soft base. This pattern is consistent with both E1 (no visible hard base violation counter; hard cap violation counter present) and ER-009 (Montréal model characteristically uses hard cap, soft base). The asymmetry is a meaningful benchmark semantic finding, but it remains a preferred reconstruction rather than a confirmed benchmark specification.

**Overall classification:** [Hypothesized | Moderate] — all three components are preferred aggregation reconstructions under the Minimal Reconstruction Principle; none is definitively confirmed from G-2014-22.

---

## 4. WP-M2.5 Exit Criteria Assessment

| Criterion | Status | Notes |
|-----------|--------|-------|
| All symbols defined in WP-M2.1 | ✓ Complete | W_n, W^min_n, W^max_n, N, t_n all from WP-M2.1/M2.2 |
| Classification tags and confidence levels complete | ✓ Complete | All components [Hypothesized\|Moderate] or [Hypothesized\|Low] |
| Sub-question A resolved to extent possible | ✓ Complete | Sum-then-cap preferred; cap-then-sum not excluded; negative finding documented |
| Sub-question C resolved to extent possible | ✓ Complete | Per-crew comparison preferred; secondary transformation not excluded |
| Base enforcement mechanism reconstructed | ✓ Complete | B2 preferred with softened rationale; B1 and B3 retained as alternatives |
| B2 rationale does not imply exclusive enforcement through Δ_n | ✓ Complete | §3.2 explicitly states separate enforcement cannot be excluded; absence of visible counter is negative evidence not proof of absence |
| ER-009 integrated into Evidence Review before being cited in conclusions | ✓ Complete | ER-009 appears in §2 and §3.2 candidate analysis before §3.4 synthesis |
| Preferred reconstruction presented, not unique recovery | ✓ Complete | All three components presented as preferred aggregation reconstructions |
| Hypothesis Propagation Rule verified | ✓ Complete | All components inherit [Hypothesized\|Moderate] from W_n and HC3-A |
| Negative findings documented | ✓ Complete | §3.1, §3.2, §3.3 each contain explicit negative finding records |
| Only frozen classification tags used | ✓ Complete | [Recovered], [Hypothesized] only |

**Summary:** The base-cap enforcement semantics are reconstructed as: hard cap (HC3-A, [Hypothesized|Moderate]), soft base via Δ_n (B2, [Hypothesized|Moderate]), per-crew aggregation with no secondary transformation ([Hypothesized|Moderate]). All three components are preferred aggregation reconstructions under the Minimal Reconstruction Principle. The asymmetric enforcement model (hard cap, soft base) is consistent with E1 and ER-009.

---

## 5. Proposed Evidence Records for BENCHMARK-KNOWLEDGE-MATRIX-v1.1

**ER-017 (proposed):** Cap Enforcement Semantics — W_n ≤ W^max_n enforced as hard constraint at monthly level (sum-then-cap preferred). Classification [Hypothesized | Moderate]. Evidence: E1 (hard violation counter) + WP-M2.4 HC3-A reconstruction.

**ER-018 (proposed):** Base Enforcement Semantics — W^min_n enforced softly via workload balance penalty Δ_n = |W_n − t_n| (B2 preferred; separate enforcement not excluded). Classification [Hypothesized | Moderate]. Evidence: E1 (no visible hard base violation counter; Δ_n penalizes under-target) + ER-009 (Montréal model soft base pattern).

**ER-019 (proposed — negative finding):** Duty-Level Cap — not evidenced in evaluator source. Sum-then-cap preferred over cap-then-sum. Recoverability: Low. This negative finding shall not be resolved by speculative selection in the benchmark specification.

**ER-020 (proposed — negative finding):** Base-Level Secondary Transformation — not evidenced in evaluator source or ER-009. Per-crew comparison preferred. Recoverability: Low.

---

## 6. Dependency Notes for WP-M2.6

- **WP-M2.6 (Consistency Validation):** Must validate: (a) cap enforcement (HC3-A) is consistent with WP-M2.4; (b) base enforcement (B2 via Δ_n) is consistent with WP-M2.3 objective; (c) asymmetric enforcement model (hard cap, soft base) is internally consistent across WP-M2.3, WP-M2.4, and WP-M2.5; (d) all negative findings are explicitly documented and not silently resolved; (e) Hypothesis Propagation Rule applied consistently from W_n through HC3-A to base-cap enforcement; (f) ER-009 corroboration is used consistently and not promoted beyond its evidence basis.

---

## Configuration Control

| Version | Date | Change |
|---------|------|--------|
| v1.0 draft | 2026-07-17 | Initial WP-M2.5 execution — cap enforcement reconstructed, base enforcement reconstructed, sub-questions A and C resolved to extent possible, negative findings documented |
| v1.0 revised | 2026-07-17 | B2 rationale softened (separate enforcement not excluded; absence of visible counter is negative evidence not proof of absence); ER-009 integrated into Evidence Review (§2) and candidate analysis (§3.2) before being cited in §3.4 synthesis; "simple per-crew comparison" consistently labelled as preferred aggregation reconstruction |