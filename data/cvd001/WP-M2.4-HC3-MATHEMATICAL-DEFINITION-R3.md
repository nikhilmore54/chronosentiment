# WP-M2.4 — HC3 Mathematical Definition (R3)
## CVD-001 Benchmark Reconstruction Project — Milestone 2

**Document ID:** WP-M2.4-v1.0  
**Work Package:** WP-M2.4 (HC3 Mathematical Definition — Research Question R3)  
**Status:** DRAFT  
**Governance baseline:** MILESTONE2-MATHEMATICAL-RECONSTRUCTION-PLAN-v1.0.md (frozen at `fc505cba`)  
**Notation baseline:** WP-M2.1-MATHEMATICAL-BENCHMARK-MODEL-v1.0.md (frozen at `e0407ded`)  
**Workload baseline:** WP-M2.2-CREDITED-WORKLOAD-EQUATION-R1.md (frozen at `eb38a8d0`)  
**Objective baseline:** WP-M2.3-OBJECTIVE-FUNCTION-R2.md (frozen at `34c84327`)  
**Evidence baseline:** Sprint 10 artifacts frozen at `721c086c`  
**Created:** 2026-07-17  

---

## 0. Research Question

**R3:** Recover the mathematical definition of HC3 — the third hard constraint in the CVD-001 benchmark.

This work package produces the formal mathematical reconstruction of HC3, using only symbols defined in WP-M2.1 and quantities reconstructed in WP-M2.2. Classification tags are restricted to the frozen protocol: [Recovered] / [Derived] / [Hypothesized] / [Engineering approximation].

---

## 1. Evidence Review

### BENCHMARK-KNOWLEDGE-MATRIX-v1.0 — HC3 as Bounded Unknown

The Knowledge Matrix (frozen at `ea6cc00b`) records HC3 as a "Bounded Unknown" with the following candidate interpretations:

| Candidate | Description |
|-----------|-------------|
| HC3-A | Contractual credit upper bound (W^max_n) |
| HC3-B | Bidline legality constraint |
| HC3-C | Monthly workload legality constraint |
| HC3-D | Collective agreement limit |

**Key finding:** HC3 is almost certainly not a weekly 40-hour cap. The monthly rostering context (G-2014-22) and the credit accumulation pipeline (ER-007) both operate at the monthly level, making a weekly constraint structurally inconsistent with the benchmark design.

**Recoverability:** Low — the exact mathematical definition of HC3 has not been recovered from public artifacts.

### E1 — Evaluator Source Code

The evaluator source ([`adapters/roadef/src/evaluator.rs`](adapters/roadef/src/evaluator.rs)) references constraint checking against contractual limits. The visible code path compares W_n against W^max_n:

```rust
if workload[nurse_id] > contract.cap {
    hard_violations += 1;
}
```

This confirms:
- HC3 involves a comparison of W_n against a contractual limit
- Violations are counted as hard constraint violations (not soft penalties)
- The contractual limit referenced is `contract.cap` (W^max_n in WP-M2.1 notation)

**Evidence classification note:** E1 is an implementation artifact. Per the confidence calibration rule (WP-M2.1 §0), E1-only evidence is capped at High confidence.

### WP-M2.2 — Reconstructed W_n

W_n (monthly credited workload) is reconstructed in WP-M2.2 as [Hypothesized | Moderate] overall (aggregation structure [Recovered | High]; qualification indexing [Hypothesized | Moderate]). HC3 candidates that reference W_n inherit this classification under the Hypothesis Propagation Rule.

### ER-007 — Credit Accumulation Pipeline

ER-007 Stage 5 (Base-Level Aggregation) establishes that monthly totals are compared against contractual limits. This independently corroborates the E1 evidence that HC3 involves a comparison of W_n against W^max_n.

---

## 2. Reconstruction of R3

### 2.1 HC3 Candidate Analysis

The four candidates from the Knowledge Matrix are evaluated against the available evidence:

#### HC3-A: Contractual Credit Upper Bound

**Proposed definition:**

> HC3-A: W_n ≤ W^max_n for all n ∈ N

where W_n is the monthly credited workload (WP-M2.2) and W^max_n is the contractual credit cap (WP-M2.1 §3.3).

**Evidence assessment:**
- E1 directly implements `if workload[nurse_id] > contract.cap { hard_violations += 1; }` — this is a direct implementation of HC3-A
- ER-007 Stage 5 independently corroborates the comparison of monthly workload against contractual limits
- W^max_n is [Recovered | High] in WP-M2.1

**Classification:** [Recovered | High] for the constraint structure; [Hypothesized | Moderate] for the complete constraint (inherits from W_n per Hypothesis Propagation Rule)

**Assessment:** HC3-A is the strongest candidate. It is directly evidenced by E1 and corroborated by ER-007. It is consistent with the monthly rostering context and the contractual credit cap parameter already established in WP-M2.1.

#### HC3-B: Bidline Legality Constraint

**Proposed definition:** HC3-B would constrain the sequence of assignments to satisfy bidline legality rules (e.g., minimum rest between duties, maximum consecutive duty days).

**Evidence assessment:**
- No direct evidence for bidline legality as HC3 in E1 or ER-007
- Bidline legality is a common constraint in crew rostering but its presence as HC3 specifically is not confirmed
- The Knowledge Matrix records this as a candidate but not as evidenced

**Classification:** [Hypothesized | Low]

**Assessment:** HC3-B cannot be excluded but has no direct evidential support. It is retained as an alternative hypothesis per the Minimal Reconstruction Principle.

#### HC3-C: Monthly Workload Legality Constraint

**Proposed definition:** HC3-C would constrain W_n to satisfy a regulatory or contractual monthly workload legality threshold, potentially distinct from W^max_n.

**Evidence assessment:**
- E1 references `contract.cap` as the limit — this is consistent with HC3-A (contractual credit cap) rather than a separate legality threshold
- No evidence for a distinct legality threshold separate from W^max_n
- HC3-C and HC3-A may be mathematically equivalent if the legality threshold equals W^max_n

**Classification:** [Hypothesized | Low]

**Assessment:** HC3-C is not independently evidenced. If the monthly workload legality threshold equals W^max_n, HC3-C collapses to HC3-A. Retained as an alternative hypothesis.

#### HC3-D: Collective Agreement Limit

**Proposed definition:** HC3-D would constrain W_n to satisfy a collective agreement limit, potentially distinct from the individual contractual cap W^max_n.

**Evidence assessment:**
- No direct evidence for a collective agreement limit as HC3 in E1 or ER-007
- The Knowledge Matrix records this as a candidate but not as evidenced
- Collective agreement limits are common in airline crew contracts but their presence as HC3 specifically is not confirmed

**Classification:** [Hypothesized | Low]

**Assessment:** HC3-D cannot be excluded but has no direct evidential support. Retained as an alternative hypothesis.

---

### 2.2 Preferred Reconstruction (Candidate HC3-A)

**Preferred reconstruction under the Minimal Reconstruction Principle:**

> Preferred reconstruction (Candidate HC3-A): W_n ≤ W^max_n for all n ∈ N

This is HC3-A. It is preferred because:
1. It is directly evidenced by E1 (`workload[nurse_id] > contract.cap`)
2. It is corroborated by ER-007 Stage 5
3. It uses only symbols already established in WP-M2.1 (W_n, W^max_n, N)
4. It introduces the fewest additional assumptions of the four candidates

**Note on threshold equivalence:** HC3-C and HC3-D may be mathematically equivalent to HC3-A under specific threshold assumptions — specifically, if their respective thresholds equal W^max_n. The preferred reconstruction does not exclude these interpretations; it selects the formulation most directly supported by evidence.

**Decomposition of HC3-A:**

| Component | Description | Classification | Confidence |
|-----------|-------------|----------------|------------|
| Constraint direction (≤) | Upper bound on W_n | [Recovered] | High |
| Contractual limit W^max_n | Credit cap parameter | [Recovered] | High |
| Scope (all n ∈ N) | Per-crew constraint | [Recovered] | High |
| Hard constraint status | Violations counted as hard | [Recovered] | High |
| W_n dependency | Monthly credited workload | [Hypothesized] | Moderate |

**Overall HC3 classification:** [Hypothesized | Moderate] — inherits from W_n per Hypothesis Propagation Rule. The constraint structure (≤ W^max_n, hard, per-crew) is [Recovered | High]; the complete constraint inherits [Hypothesized | Moderate] from W_n.

**Rationale:** This decomposition mirrors the approach used in WP-M2.2 and WP-M2.3. The constraint structure is reconstructed from recovered evidence; the complete constraint inherits the pessimistic classification from its dependency on W_n. The uncertainty is localised to the qualification indexing dimension of W_n, not to the constraint structure itself.

---

### 2.3 Negative Finding Record

**What is not recovered:**
- The exact identity of HC3 among the four candidates (HC3-A through HC3-D) cannot be definitively established from public artifacts alone
- Whether HC3 is exactly W_n ≤ W^max_n or a related but distinct constraint (e.g., with a different threshold) is not confirmed from G-2014-22
- The relationship between HC3 and any other hard constraints (HC1, HC2) is not established

**Recoverability:** Low — the exact HC3 definition is not recoverable from current public artifacts without author correspondence (S5) or access to the original benchmark specification.

**Preferred reconstruction confidence:** The preferred reconstruction HC3-A is the most evidenced candidate, but it remains [Hypothesized | Moderate] rather than [Recovered] because E1 is an implementation artifact and no authoritative benchmark documentation from G-2014-22 has been found specifying HC3 explicitly.

---

## 3. WP-M2.4 Exit Criteria Assessment

| Criterion | Status | Notes |
|-----------|--------|-------|
| All symbols defined in WP-M2.1 | ✓ Complete | W_n, W^max_n, N all from WP-M2.1/M2.2 |
| Classification tags and confidence levels complete | ✓ Complete | HC3-A [Hypothesized\|Moderate] overall; constraint structure [Recovered\|High] |
| HC3 presented as preferred reconstruction, not unique recovery | ✓ Complete | Four candidates enumerated; HC3-A preferred under Minimal Reconstruction Principle |
| Hypothesis Propagation Rule verified | ✓ Complete | HC3-A inherits [Hypothesized\|Moderate] from W_n; documented in §2.2 |
| Negative finding record explicitly states what remains unrecovered | ✓ Complete | §2.3 documents unrecovered aspects and recoverability |
| Reviewed against Knowledge Matrix — consistent with Bounded Unknown classification | ✓ Complete | All four candidates from Knowledge Matrix evaluated |
| Only frozen classification tags used | ✓ Complete | [Recovered], [Hypothesized] only |

**Summary:** HC3 is reconstructed as W_n ≤ W^max_n (HC3-A) under the Minimal Reconstruction Principle. The constraint structure is [Recovered | High]; the complete constraint is [Hypothesized | Moderate] after Hypothesis Propagation Rule from W_n. Three alternative candidates (HC3-B, HC3-C, HC3-D) are retained as [Hypothesized | Low] alternatives.

---

## 4. Proposed Evidence Records for BENCHMARK-KNOWLEDGE-MATRIX-v1.1

**ER-015 (proposed):** HC3 Preferred Reconstruction — HC3 mathematically reconstructed from available evidence as W_n ≤ W^max_n for all n ∈ N (HC3-A). Constraint structure [Recovered | High]; complete constraint [Hypothesized | Moderate] after Hypothesis Propagation Rule from W_n. Evidence: E1 (direct implementation `workload > contract.cap`) + ER-007 Stage 5 (base-level aggregation against contractual limits).

**ER-016 (proposed — negative finding):** HC3 Exact Definition — not definitively recoverable from current public artifacts. Four candidate interpretations documented (HC3-A through HC3-D). HC3-A is preferred but not confirmed from G-2014-22. Recoverability: Low. This negative finding shall not be resolved by speculative selection in the benchmark specification.

---

## 5. Dependency Notes for Subsequent Work Packages

- **WP-M2.5 (R4 — Base-Cap Enforcement):** May now use the preferred HC3 reconstruction (W_n ≤ W^max_n). Base-cap enforcement semantics will build on this preferred reconstruction. The [Hypothesized | Moderate] classification propagates.
- **WP-M2.6 (Consistency Validation):** Must validate: (a) HC3-A constraint structure is consistent with WP-M2.1 parameters; (b) Hypothesis Propagation Rule application from W_n to HC3; (c) negative finding for HC3 exact definition is explicitly documented; (d) preferred reconstruction is not promoted beyond its evidence basis.

---

## Configuration Control

| Version | Date | Change |
|---------|------|--------|
| v1.0 draft | 2026-07-17 | Initial WP-M2.4 execution — HC3 candidates evaluated, HC3-A preferred reconstruction established, negative finding documented |
| v1.0 revised | 2026-07-17 | "algebraically equivalent" replaced with "mathematically equivalent under specific threshold assumptions" for terminological precision |