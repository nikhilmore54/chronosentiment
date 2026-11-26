# WP-M2.2 — Credited Workload Equation (R1)
## CVD-001 Benchmark Reconstruction Project — Milestone 2

**Document ID:** WP-M2.2-v1.0  
**Work Package:** WP-M2.2 (Credited Workload Equation — Research Question R1)  
**Status:** DRAFT  
**Governance baseline:** MILESTONE2-MATHEMATICAL-RECONSTRUCTION-PLAN-v1.0.md (frozen at `fc505cba`)  
**Notation baseline:** WP-M2.1-MATHEMATICAL-BENCHMARK-MODEL-v1.0.md (frozen at `e0407ded`)  
**Evidence baseline:** Sprint 10 artifacts frozen at `721c086c`  
**Created:** 2026-07-17  
**Revised:** 2026-07-17 (removed non-protocol [Confirmed] and [Inferred] classifications; decomposed R1 into reconstructed aggregation structure [Recovered|High] + hypothesized qualification indexing [Hypothesized|Moderate]; overall equation [Hypothesized|Moderate] per Hypothesis Propagation Rule; replaced "formally established" with "mathematically reconstructed from available evidence")

---

## 0. Research Question

**R1:** Recover the exact equation by which flight leg durations are transformed into credited hours at the duty level and aggregated to the monthly level.

This work package produces the formal mathematical equation for W_n (monthly credited workload), using only symbols defined in WP-M2.1 and only classification tags from the frozen Milestone 2 protocol ([Recovered] / [Derived] / [Hypothesized] / [Engineering approximation]).

---

## 1. Evidence Review

The following Sprint 10 evidence records are directly relevant to R1.

### ER-007 — Credit Accumulation Semantic Pipeline

ER-007 establishes a five-stage credit accumulation model:

| Stage | Name | Description |
|-------|------|-------------|
| 1 | Flight Legs | Atomic scheduling units with block time δ_f |
| 2 | Duty Construction | Flight legs grouped into duties t ∈ T |
| 3 | Duty Credit | Each duty t assigned a credit value c_t |
| 4 | Monthly Credited Workload | Duty credits aggregated to monthly total W_n |
| 5 | Base-Level Aggregation | Monthly totals compared against contractual limits |

**Key finding from ER-007:** The credit accumulation is a sequential pipeline. Monthly workload W_n is the output of Stage 4, which aggregates duty credits c_t over all duties assigned to crew member n in the planning month.

### E1 — Evaluator Source Code

The evaluator source ([`adapters/roadef/src/evaluator.rs`](adapters/roadef/src/evaluator.rs)) implements workload accumulation as:

```rust
workload[nurse_id] += assignment.weight
```

This confirms:
- Workload is accumulated additively (summation structure)
- The unit of accumulation is `assignment.weight` — a per-assignment quantity
- Accumulation is indexed by crew member (`nurse_id`)

**Evidence classification note:** E1 is an implementation artifact. It provides strong evidence for the summation structure but does not constitute authoritative benchmark documentation. Per the confidence calibration rule (WP-M2.1 §0), E1-only evidence is capped at High confidence.

### ER-009 — Resource Model

ER-009 confirms the standard Montréal monthly crew rostering resource model, in which credited workload is accumulated over flight duties within a planning month. This independently corroborates the monthly aggregation structure of ER-007.

---

## 2. Reconstruction of R1

### 2.1 Decomposition of the R1 Equation

The R1 equation references K (qualification categories), which is classified [Hypothesized | Moderate] in WP-M2.1 §2.4. The Hypothesis Propagation Rule (MILESTONE2-PLAN §2.3) requires that a definition depending on a [Hypothesized] definition shall itself be classified [Hypothesized].

To apply this rule correctly, R1 is decomposed into two components with distinct epistemic status:

**Component 1 — Aggregation Structure (recovered independently of K):**

The evaluator source (E1) directly implements:

```
workload[nurse_id] += assignment.weight
```

This establishes that W_n is a sum of per-assignment weights over all assignments of crew member n. This summation structure is [Recovered | High] from E1, independently of the exact index set K. Whether K has cardinality 1 or greater, the additive accumulation over assignments is confirmed.

**Component 2 — Qualification Indexing (hypothesized):**

The explicit indexing over k ∈ K in the summation is [Hypothesized | Moderate], because K itself is [Hypothesized | Moderate] in WP-M2.1. The qualification dimension may be implicit (|K| = 1, all crew members share a single qualification category) or explicit (|K| > 1, multiple qualification categories exist). The evidence does not distinguish between these cases.

### 2.2 Monthly Credited Workload Equation

Using the notation from WP-M2.1, the reconstructed equation is:

**R1 (Credited Workload Equation):**

> W_n = Σ_{d∈D} Σ_{s∈S} Σ_{k∈K} w_{n,d,s,k} · x_{n,d,s,k}

where:
- W_n is the monthly credited workload of crew member n ∈ N (WP-M2.1 §5.1)
- D is the set of planning days (WP-M2.1 §2.2)
- S is the set of shift types (WP-M2.1 §2.3)
- K is the set of qualification categories (WP-M2.1 §2.4)
- w_{n,d,s,k} is the assignment weight (WP-M2.1 §3.5)
- x_{n,d,s,k} is the binary assignment decision variable (WP-M2.1 §4.1)

**Classification of reconstructed aggregation structure:** [Recovered | High]

**Classification of qualification indexing (k ∈ K dimension):** [Hypothesized | Moderate]

**Overall equation classification:** [Hypothesized | Moderate] — per the Hypothesis Propagation Rule, the complete equation inherits the pessimistic classification from its hypothesized component (K). No overall classification beyond the constituent components is required; the decomposition above expresses the uncertainty precisely.

**Rationale:** The reconstructed summation structure W_n = Σ w · x is directly evidenced by E1 and corroborated by ER-007 Stage 4 and ER-009. The explicit indexing over k ∈ K is a notational consequence of the WP-M2.1 vocabulary, which classified K as [Hypothesized | Moderate]. The equation is the most parsimonious formulation consistent with the recovered evidence (Minimal Reconstruction Principle). If K is later confirmed to have cardinality 1, the equation reduces to W_n = Σ_{d∈D} Σ_{s∈S} w_{n,d,s} · x_{n,d,s} without changing the reconstructed aggregation structure.

**Hypothesis Propagation Rule compliance:** The reconstructed aggregation structure is recovered independently of K. The qualification indexing dimension is explicitly flagged as [Hypothesized | Moderate]. The overall equation is classified [Hypothesized | Moderate] to reflect this dependency. This is consistent with the propagation rule: the equation is not promoted to [Recovered] despite the strong evidence for its aggregation structure, because it contains a hypothesized index. The decomposition makes clear that the uncertainty is localised to the qualification dimension, not the summation structure itself.

---

### 2.3 Open Sub-Questions

The following sub-questions from the WP-M2.2 work package plan remain partially or fully unresolved.

#### Sub-question A: Duty-level credit cap (cap-then-sum vs sum-then-cap)

**Question:** Is the credit function linear in flight duration, or does it apply a cap at the duty level before monthly aggregation?

**Evidence:** ER-007 Stage 3 (Duty Credit) establishes that a duty credit value c_t is computed before monthly aggregation. This is consistent with a cap-then-sum model (cap applied at duty level, then sum duties). The evaluator source (E1) accumulates `assignment.weight` directly without an explicit duty-level cap in the visible code path, which is consistent with either: (a) the cap being pre-computed into `assignment.weight`, or (b) no duty-level cap existing.

**Resolution:** Unresolved. The equation R1 as stated is consistent with both interpretations, because w_{n,d,s,k} may or may not incorporate a duty-level cap. This sub-question is deferred to WP-M2.5 (Base-Cap Enforcement Semantics, R4).

**Classification:** [Hypothesized | Moderate]

#### Sub-question B: Deadhead leg treatment

**Question:** Are deadhead legs credited at full, partial, or zero rate?

**Evidence:** ER-007 Stage 1 (Flight Legs) does not distinguish between operating and deadhead legs in the recovered pipeline description. The evaluator source (E1) accumulates `assignment.weight` uniformly without a visible deadhead flag. The Montréal model (ER-009) typically credits deadhead legs at a reduced rate, but this has not been confirmed for CVD-001 specifically.

**Resolution:** Unresolved. The equation R1 as stated subsumes deadhead treatment within w_{n,d,s,k}: if deadhead legs are credited at a reduced rate, that reduction is encoded in the weight. The exact deadhead credit rate is not recoverable from current public artifacts.

**Classification:** [Hypothesized | Low]

#### Sub-question C: Base-level aggregation

**Question:** Is the base-level aggregation (ER-007 Stage 5) a simple sum of individual crew workloads, or does it apply a secondary transformation?

**Evidence:** ER-007 Stage 5 (Base-Level Aggregation) describes aggregation at the base level but does not specify the aggregation operator. The evaluator source (E1) computes workload per crew member and compares against contractual limits; no base-level secondary transformation is visible in the evaluator code path.

**Resolution:** Unresolved for the base-level aggregation operator. This sub-question affects constraint evaluation (WP-M2.4, WP-M2.5) rather than the individual crew workload equation R1. The equation W_n as stated is the per-crew monthly workload, which is the input to base-level aggregation, not the output.

**Classification:** [Hypothesized | Moderate] — deferred to WP-M2.5.

---

### 2.4 Workload Balance Target Equation

As a direct consequence of R1, the workload balance target t_n (WP-M2.1 §5.2) can now be formally stated:

**t_n = (W^min_n + W^max_n) / 2**

where W^min_n and W^max_n are the contractual credit base and cap from WP-M2.1 §3.2 and §3.3.

**Classification:** [Recovered | High]

**Rationale:** The evaluator source (E1) directly computes `(contract.base + contract.cap) / 2.0`. This is the sole direct evidence. Confidence is High per the confidence calibration rule (E1-only evidence capped at High). No independent benchmark documentation from G-2014-22 has been found specifying this formula. The formula is classified [Recovered] because it is directly instantiated in the evaluator with no structural ambiguity; the confidence is High rather than Very High because E1 is the sole evidence source.

**Note on c_t:** The duty credit concept c_t (WP-M2.1 §3.4) is the per-duty credit value that underlies w_{n,d,s,k}. Its formal equation — the mapping from flight leg block times δ_f to duty credit c_t — is not directly recoverable from E1 or ER-007 at the level of precision needed for a formal equation. The five-stage pipeline (ER-007) establishes that c_t exists and is computed from δ_f values, but the exact aggregation rule (sum of block times, maximum block time, or a more complex function) is not confirmed. This sub-question is subsumed within Sub-question A above and deferred to WP-M2.5.

---

## 3. WP-M2.2 Exit Criteria Assessment

| Criterion | Status | Notes |
|-----------|--------|-------|
| All symbols defined in WP-M2.1 | ✓ Complete | W_n, D, S, K, w_{n,d,s,k}, x_{n,d,s,k} all from WP-M2.1 |
| Classification tags and confidence levels complete | ✓ Complete | Reconstructed aggregation structure [Recovered\|High]; qualification indexing [Hypothesized\|Moderate]; overall equation [Hypothesized\|Moderate] per Hypothesis Propagation Rule; t_n [Recovered\|High] |
| Alternatives documented for all [Hypothesized] sub-equations | ✓ Complete | Sub-questions A, B, C documented with evidence and resolution status |
| Uncertainty explicitly stated where evidence is insufficient | ✓ Complete | Deadhead treatment, duty-level cap, base aggregation all documented |
| Reviewed against ER-007 — no contradiction with five-stage pipeline | ✓ Complete | R1 is consistent with all five stages |
| Hypothesis Propagation Rule verified | ✓ Complete | K is [Hypothesized]; equation decomposed into recovered structure + hypothesized indexing; overall classification is conservative |
| Only frozen classification tags used | ✓ Complete | [Recovered], [Hypothesized] only — no new tags introduced; [Inferred] removed |

---

## 4. Proposed Evidence Records for BENCHMARK-KNOWLEDGE-MATRIX-v1.1

The following new evidence records are proposed for addition to `BENCHMARK-KNOWLEDGE-MATRIX-v1.1.md` when that document is unfrozen:

**ER-010 (proposed):** Credited Workload Equation — R1 mathematically reconstructed from available evidence as W_n = Σ w_{n,d,s,k} · x_{n,d,s,k}. Aggregation structure [Recovered | High]; qualification indexing [Hypothesized | Moderate]; overall equation inherits [Hypothesized | Moderate] under the Hypothesis Propagation Rule. Evidence: E1 (direct implementation) + ER-007 (pipeline structure) + ER-009 (monthly aggregation scope).

**ER-011 (proposed):** Workload Balance Target — t_n = (W^min_n + W^max_n) / 2 mathematically reconstructed from available evidence. Classification [Recovered | High]. Evidence: E1 (direct implementation). Note: no independent benchmark documentation found in G-2014-22.

---

## 5. Dependency Notes for Subsequent Work Packages

- **WP-M2.3 (R2 — Objective Function):** May now use W_n (R1 aggregation structure recovered) and t_n (equation recovered). Both are [Recovered | High] for the aggregation structure.
- **WP-M2.4 (R3 — HC3 Definition):** May now use W_n, W^min_n, W^max_n. HC3 candidates can be formally stated in terms of W_n. The [Hypothesized | Moderate] classification of the complete R1 equation propagates to any HC3 candidate that depends on the qualification indexing. The reconstructed aggregation structure remains available as an independently evidenced component.
- **WP-M2.5 (R4 — Base-Cap Enforcement):** Inherits Sub-questions A and C from this work package. The duty-level cap question and base-level aggregation question are deferred here.

---

## Configuration Control

| Version | Date | Change |
|---------|------|--------|
| v1.0 draft | 2026-07-17 | Initial WP-M2.2 execution — R1 equation reconstructed, sub-questions A/B/C documented |
| v1.0 revised | 2026-07-17 | Removed non-protocol [Confirmed] classification; decomposed R1 into recovered aggregation structure + hypothesized qualification indexing with explicit Hypothesis Propagation Rule analysis; replaced "formally established" with "mathematically reconstructed from available evidence" in proposed evidence records |