# WP-M2.3 — Objective Function (R2)
## CVD-001 Benchmark Reconstruction Project — Milestone 2

**Document ID:** WP-M2.3-v1.0  
**Work Package:** WP-M2.3 (Objective Function — Research Question R2)  
**Status:** DRAFT  
**Governance baseline:** MILESTONE2-MATHEMATICAL-RECONSTRUCTION-PLAN-v1.0.md (frozen at `fc505cba`)  
**Notation baseline:** WP-M2.1-MATHEMATICAL-BENCHMARK-MODEL-v1.0.md (frozen at `e0407ded`)  
**Workload baseline:** WP-M2.2-CREDITED-WORKLOAD-EQUATION-R1.md (frozen at `eb38a8d0`)  
**Evidence baseline:** Sprint 10 artifacts frozen at `721c086c`  
**Created:** 2026-07-17  
**Revised:** 2026-07-17 (objective presented as preferred reconstruction under Minimal Reconstruction Principle; Δ_n decomposed into derived operation + hypothesized quantity; cost_n rephrased as evidenced placeholder; "formally stated" replaced with "mathematically reconstructed")

---

## 0. Research Question

**R2:** Recover the objective function aggregation structure and any weighting coefficients.

This work package produces the formal mathematical reconstruction of the objective function, using only symbols defined in WP-M2.1 and quantities reconstructed in WP-M2.2. Classification tags are restricted to the frozen protocol: [Recovered] / [Derived] / [Hypothesized] / [Engineering approximation].

---

## 1. Evidence Review

### ER-008 — Objective Function Characterization

ER-008 establishes the following from Sprint 10 evidence:

**Positive findings:**
- The objective is a minimization problem
- The objective contains at least two components: a cost-related component and a workload balance component
- The workload balance component references the deviation between W_n and t_n
- The objective is evaluated per crew member and aggregated across N

**Negative finding (explicitly recorded):**
- No weighting coefficients were recovered from public artifacts. The relative weights of the cost and workload balance components are not known from available evidence.

### E1 — Evaluator Source Code

The evaluator source ([`adapters/roadef/src/evaluator.rs`](adapters/roadef/src/evaluator.rs)) implements:

```rust
// workload balance penalty
let penalty = (w - target).abs();
total_cost += penalty;
```

This confirms:
- The workload balance penalty is the absolute deviation |W_n − t_n|
- Penalties are accumulated additively into a total cost
- The accumulation is a simple sum (no visible weighting coefficient in this code path)

**Evidence classification note:** E1 is an implementation artifact. Per the confidence calibration rule (WP-M2.1 §0), E1-only evidence is capped at High confidence.

### ER-009 — Resource Model

ER-009 confirms the standard Montréal monthly crew rostering model uses a multi-objective or weighted-sum formulation balancing cost minimization and workload equity. This independently corroborates the multi-component structure of ER-008.

---

## 2. Reconstruction of R2

### 2.1 Objective Function — Preferred Structural Reconstruction

The evidence establishes: (a) minimization direction, (b) per-crew aggregation over N, (c) a workload balance component, (d) a cost-related component. The evidence does not uniquely determine the aggregation structure.

**Preferred structural reconstruction under the Minimal Reconstruction Principle:**

> minimize Z = Σ_{n∈N} [ α · cost_n + β · Δ_n ]

where:
- N is the set of crew members (WP-M2.1 §2.1)
- cost_n is a placeholder for the cost-related component for crew member n (see §2.2)
- Δ_n is the workload deviation for crew member n (see §2.3)
- α, β are weighting coefficients (classification: [Hypothesized | Low] — see §2.4)

**Note on algebraic equivalence:** The preferred reconstruction presents the objective as a per-crew weighted sum. Algebraically equivalent decompositions — such as α·Σ_n cost_n + β·Σ_n Δ_n — cannot be excluded from available evidence. The preferred form is chosen because it is the most parsimonious formulation consistent with the per-crew aggregation structure confirmed by E1. Other algebraically equivalent decompositions remain possible.

**Decomposition of R2 by component:**

| Component | Description | Classification | Confidence |
|-----------|-------------|----------------|------------|
| Minimization direction | Objective is minimized | [Recovered] | High |
| Per-crew aggregation over N | Aggregation is per-crew, summed | [Recovered] | High |
| Workload balance component | Absolute deviation from target exists | [Recovered] | High |
| Cost-related component | Evidenced; mathematical definition not recovered | [Recovered] | Moderate |
| Weighting coefficients α, β | Not recovered from public artifacts | [Hypothesized] | Low |
| Additive aggregation operator | Additive (weighted sum) | [Recovered] | High |

**Overall objective structure classification:** [Recovered | High] for the structural form (minimization of a per-crew sum with two components); [Hypothesized | Low] for the weighting coefficients. The preferred reconstruction is the formulation introducing the fewest additional assumptions consistent with the evidence.

---

### 2.2 Cost-Related Component

The evidence (ER-008) indicates the objective contains a cost-related component. For reconstruction purposes this component is denoted cost_n. Its mathematical definition has not been recovered from current public artifacts.

**Classification of cost_n existence:** [Recovered | Moderate] — ER-008 confirms a cost component exists; E1 accumulates `total_cost` which includes at least the workload balance penalty.

**Classification of cost_n mathematical definition:** [Hypothesized | Moderate] — the exact form is not recoverable from current public artifacts.

**Candidate formulations (per Minimal Reconstruction Principle):**

| Candidate | Description | Classification | Confidence |
|-----------|-------------|----------------|------------|
| C1: Duty cost sum | cost_n = Σ_{t assigned to n} c^cost_t | [Hypothesized] | Moderate |
| C2: Pairing cost | cost_n = Σ_{pairings assigned to n} c^pairing | [Hypothesized] | Low |
| C3: Implicit (zero) | cost_n = 0 (workload balance only) | [Hypothesized] | Low |

**Evidence assessment:** E1 accumulates `total_cost` which includes the workload balance penalty. Whether a separate duty cost term exists in the evaluator is not confirmed from the visible code path. ER-008 characterizes the objective as having a cost component, but does not specify its form. Candidate C1 is most consistent with the standard Montréal model (ER-009) and is therefore preferred under the Minimal Reconstruction Principle, but it remains [Hypothesized | Moderate].

**Negative finding record:** The mathematical definition of cost_n is not recoverable from current public artifacts. This is explicitly documented as an open question for WP-M2.6 consistency validation.

---

### 2.3 Workload Deviation — Decomposed Reconstruction

The workload deviation Δ_n (WP-M2.1 §5.3) is mathematically reconstructed using the quantities from WP-M2.2.

**Decomposition of Δ_n:**

**Component 1 — Absolute-value transformation (derived from W_n and t_n):**

The absolute-value operation |W_n − t_n| is a mathematical derivation from W_n and t_n. This derivation step is [Derived | High]: the operation is directly confirmed by E1 (`(w - target).abs()`), and ER-008 independently confirms the existence of a workload balance component.

**Component 2 — Dependency on W_n (hypothesized):**

W_n is classified [Hypothesized | Moderate] overall in WP-M2.2 (due to the qualification indexing over K). Therefore Δ_n, which depends on W_n, inherits [Hypothesized | Moderate] under the Hypothesis Propagation Rule.

**Reconstructed equation:**

> Δ_n = |W_n − t_n|

where:
- W_n is the monthly credited workload (WP-M2.2 R1 equation, [Hypothesized | Moderate])
- t_n = (W^min_n + W^max_n) / 2 is the workload balance target (WP-M2.2 §2.4, [Recovered | High])

**Classification of absolute-value operation:** [Derived | High]

**Classification of complete quantity Δ_n:** [Hypothesized | Moderate] — inherits from W_n per Hypothesis Propagation Rule

**Rationale:** This decomposition mirrors the approach used in WP-M2.2 for R1: the mathematical operation is classified on its own evidence, while the complete quantity inherits the pessimistic classification from its hypothesized dependency. The uncertainty is localised to the qualification indexing dimension of W_n, not to the absolute-value operation itself.

---

### 2.4 Weighting Coefficients

**α** (cost weight) and **β** (workload balance weight) are the relative weights of the two objective components.

**Classification:** [Hypothesized | Low]

**Rationale:** No weighting coefficients were recovered from public artifacts (ER-008 negative finding). The evaluator source (E1) does not show explicit weighting in the visible code path — `total_cost += penalty` suggests either equal weighting (α = β = 1) or that the cost component is not separately weighted in the implementation. The Minimal Reconstruction Principle requires retaining competing hypotheses rather than selecting one without evidential justification:

| Hypothesis | Description | Consistency with evidence |
|------------|-------------|--------------------------|
| H1: Equal weights | α = β = 1 | Consistent with E1 (no visible weighting) |
| H2: Unequal weights | α ≠ β, values unknown | Consistent with ER-008 (multi-component objective) |
| H3: Single component | β = 1, α = 0 (workload balance only) | Partially consistent with E1 visible code path |

Per the Minimal Reconstruction Principle, H1 (equal weights) is preferred as the formulation introducing the fewest additional assumptions. However, H1 is not promoted to [Recovered] or [Derived] because no authoritative benchmark documentation confirms it.

**Negative finding record:** Weighting coefficients are not recoverable from current public artifacts. This is the primary open question of R2 and is explicitly documented for WP-M2.6.

---

## 3. WP-M2.3 Exit Criteria Assessment

| Criterion | Status | Notes |
|-----------|--------|-------|
| All symbols defined in WP-M2.1 | ✓ Complete | N, Δ_n, W_n, t_n, W^min_n, W^max_n all from WP-M2.1/M2.2 |
| Classification tags and confidence levels complete | ✓ Complete | All components classified; see §2.1 decomposition table |
| Objective presented as preferred reconstruction, not unique recovery | ✓ Complete | Algebraic equivalence note in §2.1 |
| Δ_n decomposed into derived operation + hypothesized quantity | ✓ Complete | §2.3 mirrors WP-M2.2 decomposition approach |
| Negative finding record (D-M2.3.3) explicitly states what remains unrecovered | ✓ Complete | Weighting coefficients and cost_n definition documented as unrecovered |
| Weighting candidates enumerated with rationale | ✓ Complete | H1, H2, H3 in §2.4; H1 preferred under Minimal Reconstruction Principle |
| Reviewed against ER-008 — consistent with multi-component minimization | ✓ Complete | Structural reconstruction consistent with ER-008 positive and negative findings |
| Hypothesis Propagation Rule verified | ✓ Complete | Δ_n inherits [Hypothesized|Moderate] from W_n; documented in §2.3 |
| Only frozen classification tags used | ✓ Complete | [Recovered], [Derived], [Hypothesized] only |

**Summary:** The objective function structure is reconstructed as [Recovered | High] for the structural form (preferred reconstruction under Minimal Reconstruction Principle). The weighting coefficients remain [Hypothesized | Low] — the primary unresolved question of R2. The workload deviation Δ_n is decomposed: absolute-value operation [Derived | High]; complete quantity [Hypothesized | Moderate] after Hypothesis Propagation Rule from W_n.

---

## 4. Proposed Evidence Records for BENCHMARK-KNOWLEDGE-MATRIX-v1.1

**ER-012 (proposed):** Objective Function Structure — R2 mathematically reconstructed from available evidence. Preferred structural reconstruction: minimize Z = Σ_{n∈N} [α · cost_n + β · Δ_n]. Structural form [Recovered | High]; weighting coefficients [Hypothesized | Low]. Algebraically equivalent decompositions not excluded. Evidence: E1 (direct implementation) + ER-008 (multi-component characterization) + ER-009 (Montréal model corroboration).

**ER-013 (proposed):** Workload Deviation Equation — Δ_n = |W_n − t_n| mathematically reconstructed from available evidence. Absolute-value operation [Derived | High]; complete quantity [Hypothesized | Moderate] after Hypothesis Propagation Rule from W_n. Evidence: E1 (direct implementation) + ER-008 (workload balance component confirmed).

**ER-014 (proposed — negative finding):** Objective Weighting Coefficients — not recoverable from current public artifacts. No weighting coefficients found in G-2014-22, generator code (S0), or evaluator source (E1). Recoverability: Low. This negative finding is authoritative and shall not be replaced by speculative values in the benchmark specification.

---

## 5. Dependency Notes for Subsequent Work Packages

- **WP-M2.4 (R3 — HC3 Definition):** May now use W_n, W^min_n, W^max_n, and the reconstructed objective structure. HC3 candidates are independent of the objective function weighting.
- **WP-M2.5 (R4 — Base-Cap Enforcement):** May use W_n, W^max_n, and the reconstructed objective structure. Base-cap enforcement is a constraint, not an objective term.
- **WP-M2.6 (Consistency Validation):** Must validate: (a) Δ_n decomposition chain from W_n and t_n; (b) Hypothesis Propagation Rule application to Δ_n; (c) negative finding for weighting coefficients is explicitly documented and not silently resolved; (d) preferred reconstruction is not promoted beyond its evidence basis.

---

## Configuration Control

| Version | Date | Change |
|---------|------|--------|
| v1.0 draft | 2026-07-17 | Initial WP-M2.3 execution — R2 objective structure reconstructed, weighting coefficients documented as unrecovered, Δ_n mathematically reconstructed with propagation rule applied |
| v1.0 revised | 2026-07-17 | Objective presented as preferred reconstruction under Minimal Reconstruction Principle (algebraic equivalence noted); Δ_n decomposed into derived operation [Derived\|High] + hypothesized quantity [Hypothesized\|Moderate]; cost_n rephrased as evidenced placeholder with unrecovered definition; "formally stated" replaced with "mathematically reconstructed" throughout |