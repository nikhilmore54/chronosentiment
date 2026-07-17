# WP-M2.6 — Internal Consistency Validation
## CVD-001 Benchmark Reconstruction Project — Milestone 2

**Document ID:** WP-M2.6-v1.0  
**Work Package:** WP-M2.6 (Internal Consistency Validation — final Milestone 2 work package)  
**Status:** DRAFT  
**Governance baseline:** MILESTONE2-MATHEMATICAL-RECONSTRUCTION-PLAN-v1.0.md (frozen at `fc505cba`)  
**Notation baseline:** WP-M2.1-MATHEMATICAL-BENCHMARK-MODEL-v1.0.md (frozen at `e0407ded`)  
**Validated work packages:**
- WP-M2.2-CREDITED-WORKLOAD-EQUATION-R1.md (frozen at `eb38a8d0`)
- WP-M2.3-OBJECTIVE-FUNCTION-R2.md (frozen at `34c84327`)
- WP-M2.4-HC3-MATHEMATICAL-DEFINITION-R3.md (frozen at `58ceae03`)
- WP-M2.5-BASE-CAP-ENFORCEMENT-R4.md (frozen at `75269f4d`)  
**Created:** 2026-07-17  

---

## 0. Purpose and Scope

This work package validates the internal consistency of the reconstructed mathematical model assembled across WP-M2.1 through WP-M2.5. It does not introduce new mathematical reconstructions or new evidence. Its purpose is to:

1. Verify that all reconstruction targets (R1–R4) are mutually compatible.
2. Confirm that the Hypothesis Propagation Rule has been applied uniformly across all work packages.
3. Confirm that evidence boundaries have been respected — no work package promotes a reconstruction beyond its evidential basis.
4. Identify any contradictions, gaps, or unresolved dependencies between work packages.
5. Produce a consolidated reconstruction summary suitable for use in BENCHMARK-SEMANTICS-v1.0.md.

---

## 1. Validated Reconstruction Inventory

| Target | Description | Resolved in | Frozen commit |
|--------|-------------|-------------|---------------|
| R1 | Credited workload equation | WP-M2.2 | `eb38a8d0` |
| R2 | Objective function | WP-M2.3 | `34c84327` |
| R3 | HC3 mathematical definition | WP-M2.4 | `58ceae03` |
| R4-A | Cap enforcement mechanism | WP-M2.5 | `75269f4d` |
| R4-B | Base enforcement mechanism | WP-M2.5 | `75269f4d` |
| R4-C | Aggregation level | WP-M2.5 | `75269f4d` |

---

## 2. Consistency Checks

### Check MC-1: R1 and R2 compatibility

**R1 (WP-M2.2):** W_n = Σ_{t∈T_n} c_t · x_{n,t,k} [Hypothesized | Moderate]

**R2 (WP-M2.3):** minimize Z = Σ_{n∈N} [α · cost_n + β · Δ_n] [Recovered | High] structural form; [Hypothesized | Low] weighting

**Compatibility check:** W_n appears in both R1 and R2 (via Δ_n = |W_n − t_n|). The definition of W_n is consistent across both work packages — WP-M2.3 explicitly inherits W_n from WP-M2.2 without redefining it.

**Classification propagation check:** Δ_n is classified [Hypothesized | Moderate] in WP-M2.3, correctly inheriting from W_n [Hypothesized | Moderate] per the Hypothesis Propagation Rule. The absolute-value operation itself is [Derived | High]. This decomposition is consistent.

**Result: PASS** — No contradiction. W_n definition is consistent; propagation is correct.

---

### Check MC-2: R1 and R3 compatibility

**R1 (WP-M2.2):** W_n = Σ_{t∈T_n} c_t · x_{n,t,k} [Hypothesized | Moderate]

**R3 (WP-M2.4):** Preferred reconstruction (Candidate HC3-A): W_n ≤ W^max_n for all n ∈ N [Hypothesized | Moderate]

**Compatibility check:** HC3-A applies the same W_n defined in R1. The constraint is a predicate over W_n, not a redefinition of it. W^max_n is defined in WP-M2.1 §3.3 and used consistently in both WP-M2.2 and WP-M2.4.

**Classification propagation check:** HC3-A is classified [Hypothesized | Moderate] overall, correctly inheriting from W_n [Hypothesized | Moderate]. The constraint structure (≤ W^max_n, hard, per-crew) is [Recovered | High]. This decomposition is consistent with the pattern established in WP-M2.2 and WP-M2.3.

**Result: PASS** — No contradiction. W_n definition is consistent; HC3-A constraint structure is compatible with R1.

---

### Check MC-3: R2 and R4-B compatibility

**R2 (WP-M2.3):** Objective includes Δ_n = |W_n − t_n| as the workload balance component [Hypothesized | Moderate]

**R4-B (WP-M2.5):** Base enforcement reconstructed as soft penalty via Δ_n (B2 preferred) [Hypothesized | Moderate]

**Compatibility check:** R4-B (B2) is a semantic interpretation of R2's role with respect to the base constraint. The Δ_n term in R2 penalizes both over-target and under-target deviations, which includes under-base deviation when W_n < t_n. R4-B does not introduce a separate Δ_n — it identifies the existing R2 term as the base enforcement mechanism.

**Consistency of negative finding:** WP-M2.5 explicitly states that separate enforcement mechanisms (B1 or B3) cannot be excluded. WP-M2.3 does not claim that Δ_n is the exclusive base enforcement mechanism. These are consistent — neither work package overclaims.

**Result: PASS** — No contradiction. R4-B is a semantic interpretation of R2; both preserve the appropriate uncertainty.

---

### Check MC-4: R3 and R4-A compatibility

**R3 (WP-M2.4):** Preferred reconstruction (Candidate HC3-A): W_n ≤ W^max_n for all n ∈ N [Hypothesized | Moderate]

**R4-A (WP-M2.5):** Cap enforcement: W_n ≤ W^max_n for all n ∈ N (HC3-A, from WP-M2.4) [Hypothesized | Moderate]

**Compatibility check:** R4-A explicitly inherits HC3-A from WP-M2.4 without modification. The preferred reconstruction is identical in both work packages. The classification [Hypothesized | Moderate] is consistent.

**Result: PASS** — R4-A is a direct inheritance of R3 (HC3-A). No contradiction.

---

### Check MC-5: R4-A and R4-B asymmetric enforcement

**R4-A (WP-M2.5):** Cap enforcement is hard (HC3-A) [Hypothesized | Moderate]

**R4-B (WP-M2.5):** Base enforcement is soft via Δ_n (B2) [Hypothesized | Moderate]

**Compatibility check:** The asymmetric enforcement model (hard cap, soft base) is internally consistent — there is no mathematical contradiction between a hard cap constraint and a soft base penalty. Both are applied to the same W_n. The hard cap (R4-A) constrains W_n from above; the soft base (R4-B) penalizes W_n for deviating from t_n in either direction.

**Corroboration check:** Both E1 and ER-009 support the asymmetric pattern. WP-M2.5 §2 integrates ER-009 into the evidence review before citing it in the synthesis. The evidence flow is consistent.

**Result: PASS** — Asymmetric enforcement is internally consistent and consistently evidenced.

---

### Check MC-6: R4-C aggregation level compatibility

**R4-C (WP-M2.5):** Base-level aggregation is per-crew comparison, no secondary transformation [Hypothesized | Moderate]

**R1 (WP-M2.2):** W_n is computed per crew member n [Hypothesized | Moderate]

**R3 (WP-M2.4):** HC3-A applies per crew member n ∈ N [Hypothesized | Moderate]

**Compatibility check:** R4-C (per-crew aggregation) is consistent with R1 (per-crew workload computation) and R3 (per-crew constraint). All three work packages use the same aggregation level — individual crew member n — without secondary transformation. No contradiction.

**Result: PASS** — Per-crew aggregation is consistent across R1, R3, and R4-C.

---

### Check HP-1: Hypothesis Propagation Rule uniformity

The Hypothesis Propagation Rule (WP-M2.1 §0) states that if any component of a reconstructed quantity is [Hypothesized], the complete quantity inherits [Hypothesized] at the pessimistic confidence level.

**Verification across work packages:**

| Quantity | Hypothesized component | Overall classification | Correct? |
|----------|----------------------|----------------------|----------|
| W_n (WP-M2.2) | Qualification indexing k ∈ K | [Hypothesized \| Moderate] | ✓ |
| Δ_n (WP-M2.3) | W_n dependency | [Hypothesized \| Moderate] | ✓ |
| HC3-A (WP-M2.4) | W_n dependency | [Hypothesized \| Moderate] | ✓ |
| Cap enforcement (WP-M2.5) | HC3-A / W_n dependency | [Hypothesized \| Moderate] | ✓ |
| Base enforcement B2 (WP-M2.5) | W_n dependency | [Hypothesized \| Moderate] | ✓ |
| Aggregation R4-C (WP-M2.5) | W_n dependency | [Hypothesized \| Moderate] | ✓ |

**Result: PASS** — Hypothesis Propagation Rule applied uniformly across all work packages.

---

### Check EB-1: Evidence boundary discipline

Each work package must not promote a reconstruction beyond its evidential basis. Specifically: E1-only evidence is capped at High confidence; no reconstruction may be classified [Recovered] without direct evidence from G-2014-22 or equivalent authoritative source.

**Verification:**

| Work Package | Highest classification used | Justification | Correct? |
|---|---|---|---|
| WP-M2.2 | [Recovered \| High] for aggregation structure | E1 direct + ER-007 corroboration | ✓ |
| WP-M2.3 | [Recovered \| High] for structural form | E1 direct + ER-008 corroboration | ✓ |
| WP-M2.4 | [Recovered \| High] for constraint structure | E1 direct + ER-007 corroboration | ✓ |
| WP-M2.5 | [Hypothesized \| Moderate] throughout | No component exceeds E1 evidence | ✓ |

No work package claims [Recovered] for a complete reconstruction. All [Recovered | High] classifications are applied only to structural components (aggregation structure, constraint direction, minimization direction) that are directly confirmed by E1 and corroborated by at least one additional source.

**Result: PASS** — Evidence boundary discipline maintained throughout.

---

### Check NF-1: Negative finding completeness

Each unresolved question must be explicitly documented as a negative finding rather than silently omitted.

**Verification:**

| Unresolved question | Documented in | Recoverability |
|---|---|---|
| Qualification indexing k ∈ K | WP-M2.2 §2.2 | Moderate |
| Duty-level cap (cap-then-sum vs sum-then-cap) | WP-M2.5 §3.1 | Low |
| Weighting coefficients α, β | WP-M2.3 §2.4 | Low |
| cost_n mathematical definition | WP-M2.3 §2.2 | Moderate |
| HC3 exact identity (HC3-A vs HC3-B/C/D) | WP-M2.4 §2.3 | Low |
| Base enforcement mechanism (B1 vs B2 vs B3) | WP-M2.5 §3.2 | Low |
| Base-level secondary transformation | WP-M2.5 §3.3 | Low |

All seven unresolved questions are explicitly documented with recoverability assessments. None is silently omitted.

**Result: PASS** — Negative finding completeness verified.

---

## 3. Validated Reconstruction Summary

The following table summarises the preferred reconstructions across all R1–R4 targets, suitable for use in BENCHMARK-SEMANTICS-v1.0.md:

| Target | Preferred reconstruction | Classification | Confidence | Source |
|--------|--------------------------|----------------|------------|--------|
| R1: Credited workload | W_n = Σ_{t∈T_n} c_t · x_{n,t,k} | [Hypothesized] | Moderate | WP-M2.2 |
| R1: Workload balance target | t_n = (W^min_n + W^max_n) / 2 | [Recovered] | High | WP-M2.2 |
| R2: Objective structure | minimize Z = Σ_{n∈N} [α · cost_n + β · Δ_n] | [Recovered] | High | WP-M2.3 |
| R2: Workload deviation | Δ_n = \|W_n − t_n\| | [Hypothesized] | Moderate | WP-M2.3 |
| R2: Weighting coefficients | α, β — not recovered | [Hypothesized] | Low | WP-M2.3 |
| R3: HC3 (preferred) | W_n ≤ W^max_n for all n ∈ N | [Hypothesized] | Moderate | WP-M2.4 |
| R4-A: Cap enforcement | Hard constraint at monthly level | [Hypothesized] | Moderate | WP-M2.5 |
| R4-B: Base enforcement | Soft penalty via Δ_n (B2 preferred) | [Hypothesized] | Moderate | WP-M2.5 |
| R4-C: Aggregation level | Per-crew comparison, no secondary transformation | [Hypothesized] | Moderate | WP-M2.5 |

**Asymmetric enforcement pattern:** Hard cap (R4-A) + soft base (R4-B) — consistent with E1 and ER-009.

---

## 4. WP-M2.6 Exit Criteria Assessment

| Criterion | Status | Notes |
|-----------|--------|-------|
| All nine validation checks completed | ✓ Complete | MC-1 through MC-6, HP-1, EB-1, NF-1 |
| No contradictions found between work packages | ✓ Complete | All checks PASS |
| Hypothesis Propagation Rule verified uniformly | ✓ Complete | Check HP-1 |
| Evidence boundary discipline verified | ✓ Complete | Check EB-1 |
| Negative finding completeness verified | ✓ Complete | Check NF-1 — 7 unresolved questions documented |
| Consolidated reconstruction summary produced | ✓ Complete | §3 |
| No new mathematical reconstructions introduced | ✓ Complete | WP-M2.6 is validation only |

**Summary:** All consistency checks pass. The reconstructed mathematical model is internally consistent across WP-M2.1 through WP-M2.5. No contradictions, classification violations, or evidence boundary breaches were found. The consolidated reconstruction summary in §3 is suitable for use as the mathematical foundation of BENCHMARK-SEMANTICS-v1.0.md.

---

## 5. Dependency Notes

- **BENCHMARK-SEMANTICS-v1.0.md:** May now use the validated reconstruction summary from §3 as its mathematical foundation. All preferred reconstructions are [Hypothesized | Moderate] or better; all negative findings are explicitly documented.
- **BENCHMARK-REFERENCE-SPECIFICATION-v1.0.md (M3A):** May use the preferred reconstructions from §3 as the specification baseline, with explicit acknowledgement of the [Hypothesized | Moderate] classification throughout.

---

## Configuration Control

| Version | Date | Change |
|---------|------|--------|
| v1.0 draft | 2026-07-17 | Initial WP-M2.6 execution — six mathematical compatibility checks, hypothesis propagation check, evidence boundary check, negative finding completeness check; all PASS; consolidated reconstruction summary produced |