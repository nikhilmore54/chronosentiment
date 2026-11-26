# Milestone 2 — Mathematical Benchmark Reconstruction Plan
## CVD-001 Benchmark Reconstruction Project

**Document ID:** MILESTONE2-PLAN-v1.0  
**Status:** ACTIVE  
**Branch:** governance-hardening  
**Baseline:** Sprint 10 artifacts frozen at commit `721c086c`  
**Created:** 2026-07-17  
**Revised:** 2026-07-17 (v1.0 final — WP-M2.6 added, confidence dimension, hypothesis propagation rule, Mathematical Traceability Principle, Minimal Reconstruction Principle, per-WP exit criteria)

---

## 0. Scope and Purpose

This document defines the work plan for Milestone 2 — Mathematical Benchmark Reconstruction.

Sprint 10 (Milestone 1) established a stable, evidence-based semantic understanding of the CVD-001 benchmark and documented the limits of public knowledge. The frozen Sprint 10 artifacts constitute the permanent archival baseline for all Milestone 2 work.

Milestone 2 does not revisit evidence acquisition. Its sole objective is:

> **Recover the missing mathematics of an already-understood benchmark.**

All mathematical definitions produced in Milestone 2 shall be tagged according to the Definition Classification Scheme defined in Section 2. This preserves the Benchmark Reconstruction Principle and the Mathematical Traceability Principle throughout the mathematical reconstruction effort.

---

## 1. Research Questions

| ID | Research Question | Priority |
|----|-------------------|----------|
| R1 | Recover the exact credited workload equation | High |
| R2 | Recover the objective function aggregation and weighting | High |
| R3 | Recover the HC3 mathematical definition | High |
| R4 | Validate the mathematical semantics of base-cap enforcement | Medium |

These questions were identified in `BENCHMARK-KNOWLEDGE-MATRIX-v1.0.md` as the remaining open items after Sprint 10 mathematical recovery (ER-007, ER-008, ER-009).

---

## 2. Definition Classification Scheme

Every mathematical definition, equation, and constraint introduced in Milestone 2 shall carry a **classification tag** and a **confidence level**.

### 2.1 Classification Tags

| Tag | Meaning |
|-----|---------|
| **[Recovered]** | Directly supported by evidence from the Sprint 10 evidence hierarchy (E1–E5). The evidence source shall be cited. |
| **[Derived]** | Logically implied by one or more Recovered definitions. The derivation chain shall be stated. |
| **[Hypothesized]** | A candidate reconstruction consistent with available evidence but not uniquely determined by it. Alternatives shall be noted. |
| **[Engineering approximation]** | Introduced for implementation tractability. Not claimed to reproduce benchmark semantics. |

### 2.2 Confidence Levels

Not all recovered facts are equally certain. Each tag shall be accompanied by a confidence level:

| Confidence | Meaning |
|------------|---------|
| **Very High** | Multiple independent evidence sources; no contradicting evidence |
| **High** | Single strong evidence source or multiple weaker sources; no contradicting evidence |
| **Moderate** | Evidence is suggestive but not conclusive; alternatives exist |
| **Low** | Evidence is indirect or inferential; significant uncertainty remains |

**Examples:**
- Planning horizon (monthly): `[Recovered | Very High]` — confirmed by G-2014-22 and generator code
- Deadhead treatment: `[Recovered | Moderate]` — inferred from credit pipeline structure, not directly stated
- HC3 candidate definition: `[Hypothesized | Moderate]` — consistent with evidence but not uniquely determined

### 2.3 Hypothesis Propagation Rule

> **A [Hypothesized] definition shall not be used to derive a [Recovered] or [Derived] definition.**

The permitted dependency direction is strictly:

```
[Recovered]
     ↓
[Derived]
     ↓
[Hypothesized]
```

A definition that depends on a [Hypothesized] definition shall itself be classified [Hypothesized], regardless of how strong its other evidence sources are. This prevents hypothesis contamination from silently propagating through the derivation chain.

---

## 3. Governing Principles

All Milestone 2 work is governed by the principles established in Sprint 10, plus one new principle introduced for Milestone 2.

### Benchmark Reconstruction Principle (Sprint 10)
> Coralys shall reproduce benchmark semantics only when supported by sufficient evidence. Unknown benchmark behavior shall remain explicitly documented as unknown rather than replaced by speculative implementations.

### Three Statement Classes (Sprint 10)
- **Implementation findings:** what Coralys currently computes
- **Empirical findings:** observations from running Coralys on CVD-001
- **Benchmark findings:** semantics supported by authoritative evidence

### Mathematical Traceability Principle (Milestone 2)
> Every mathematical symbol appearing in the benchmark specification shall be traceable to a semantic concept recovered during Sprint 10 or explicitly identified as a hypothesis.

This principle ensures that the mathematical reconstruction remains grounded in the evidence base established by Sprint 10. Symbols introduced purely for notational convenience shall be flagged as such in the notation glossary (D-M2.1.5).

### Minimal Reconstruction Principle (Milestone 2)
> When multiple mathematical formulations are equally consistent with the recovered evidence, prefer the formulation that introduces the fewest additional assumptions. If two formulations remain equally plausible, retain both as alternative hypotheses rather than selecting one without evidential justification.

This principle provides guidance for hypothesis selection without forcing an unsupported choice. It aligns with the Benchmark Reconstruction Principle: unknown benchmark behavior is documented explicitly rather than resolved by arbitrary selection. When the Minimal Reconstruction Principle is applied to select among candidates, the selection rationale shall be recorded in the relevant work package deliverable.

---

## 4. Work Packages

### WP-M2.1 — Mathematical Benchmark Model Foundation

**Objective:** Establish the formal mathematical language — the mathematical vocabulary of the project — that all subsequent work packages will use. This is not merely a preliminary step; it is the foundation on which the entire Milestone 2 reconstruction rests.

**Rationale:** Jumping directly into HC3 or objective weighting without a shared notation risks inconsistency across work packages and makes the final specification harder to audit. WP-M2.1 produces the mathematical scaffolding first.

**Deliverables:**

- **D-M2.1.1 — Index Sets:** Crew members, duties, pairings, bases, days, flight legs, months, resource types. Each set tagged with classification and confidence.
- **D-M2.1.2 — Parameters:** Flight leg durations, scheduled departure/arrival times, base assignments, contractual limits. Each parameter tagged with evidence source.
- **D-M2.1.3 — Decision Variables:** Pairing-to-crew assignment variables, duty construction variables, and any auxiliary variables required for constraint modelling.
- **D-M2.1.4 — Derived Quantities:** Credited workload per duty, credited workload per month, duty elapsed time, rest periods. Each derivation tagged and linked to ER-007.
- **D-M2.1.5 — Notation Glossary:** Symbol table with tag, confidence, definition, evidence source, and cross-reference to Sprint 10 evidence records.

**Baseline evidence:** ER-007 (credit accumulation pipeline), ER-009 (resource model), `BENCHMARK-KNOWLEDGE-MATRIX-v1.0.md` Section 3.

**Exit Criteria:**
- ✓ All index sets defined with classification tags and confidence levels
- ✓ All parameters defined with evidence source citations
- ✓ All decision variables defined with scope and domain
- ✓ Notation glossary (D-M2.1.5) complete — no symbol used in WP-M2.2 through WP-M2.6 is undefined
- ✓ Mathematical Traceability Principle verified: every symbol traceable to a Sprint 10 semantic concept or flagged as hypothesis

---

### WP-M2.2 — Credited Workload Equation (R1)

**Objective:** Recover the exact equation by which flight leg durations are transformed into credited hours at the duty level and aggregated to the monthly level.

**Baseline evidence:** ER-007 (credit accumulation semantic pipeline — five-stage model: Flight Legs → Duty Construction → Duty Credit → Monthly Credited Workload → Base-Level Aggregation).

**Open sub-questions:**
- Is the credit function linear in flight duration, or does it apply a cap-then-sum vs sum-then-cap logic?
- Are deadhead legs credited at full, partial, or zero rate?
- Is the base-level aggregation a simple sum or does it apply a secondary transformation?

**Deliverables:**
- **D-M2.2.1 — Duty Credit Equation:** Formal equation mapping flight leg durations within a duty to a single duty credit value. Tagged per sub-question resolution.
- **D-M2.2.2 — Monthly Aggregation Equation:** Formal equation mapping duty credits within a month to monthly credited workload.
- **D-M2.2.3 — Deadhead Treatment:** Explicit statement of deadhead credit semantics with classification tag and confidence.

**Exit Criteria:**
- ✓ All symbols defined in D-M2.1.5
- ✓ Classification tags and confidence levels complete for all equations
- ✓ Alternatives documented for all [Hypothesized] sub-equations
- ✓ Uncertainty explicitly stated where evidence is insufficient
- ✓ Reviewed against ER-007 — no contradiction with five-stage pipeline
- ✓ Hypothesis propagation rule verified — no [Hypothesized] input used to derive [Recovered] output

---

### WP-M2.3 — Objective Function (R2)

**Objective:** Recover the objective function aggregation structure and any weighting coefficients.

**Baseline evidence:** ER-008 (objective function characterization — multi-component minimization with cost and workload balance terms; negative finding: no weighting coefficients recovered from public artifacts).

**Open sub-questions:**
- What are the relative weights of cost vs workload balance components?
- Is the objective a weighted sum, lexicographic, or Pareto formulation?
- Are penalty terms present for constraint violations?

**Deliverables:**
- **D-M2.3.1 — Objective Structure:** Formal statement of the objective function components and their aggregation operator. Tagged [Recovered] where ER-008 provides evidence; [Hypothesized] elsewhere.
- **D-M2.3.2 — Weighting Candidates:** Enumeration of candidate weighting schemes consistent with the recovered structure, with rationale and confidence for each.
- **D-M2.3.3 — Negative Finding Record:** Explicit documentation of what could not be recovered and why, consistent with the three-statement-class methodology.

**Exit Criteria:**
- ✓ All symbols defined in D-M2.1.5
- ✓ Classification tags and confidence levels complete
- ✓ Negative finding record (D-M2.3.3) explicitly states what remains unrecovered
- ✓ Weighting candidates enumerated with rationale — no candidate presented as definitive without [Recovered | High] or better evidence
- ✓ Reviewed against ER-008 — consistent with multi-component minimization characterization

---

### WP-M2.4 — HC3 Mathematical Definition (R3)

**Objective:** Recover the mathematical definition of constraint HC3.

**Baseline evidence:** `BENCHMARK-KNOWLEDGE-MATRIX-v1.0.md` HC3 entry — Semantic Understanding: Partial; Mathematical Reconstruction: Not recovered. Bounded Unknown: almost certainly not a weekly 40h cap; candidates are contractual credit upper bound, bidline legality, monthly workload legality, collective agreement limit.

**Sequencing note:** WP-M2.4 is sequenced after WP-M2.2 because the credited workload equation (R1) is a prerequisite for formally stating any workload-based constraint. HC3 candidates that reference credited workload must use the notation and equations established in WP-M2.2.

**Open sub-questions:**
- Is HC3 a hard constraint or a soft penalty?
- Does it operate at the duty, pairing, or monthly level?
- Which of the four candidate interpretations is most consistent with the recovered credit accumulation model (WP-M2.2)?

**Deliverables:**
- **D-M2.4.1 — HC3 Candidate Definitions:** Formal mathematical statement of each candidate interpretation, expressed in D-M2.1 notation and referencing D-M2.2 equations.
- **D-M2.4.2 — Consistency Analysis:** For each candidate, assessment of consistency with ER-007, ER-008, ER-009, and the CVD-001 generator code observations from S0.
- **D-M2.4.3 — Recommended Definition:** The most evidence-consistent candidate, tagged [Hypothesized] with explicit statement of remaining uncertainty and documented alternatives.

**Exit Criteria:**
- ✓ All symbols defined in D-M2.1.5; all equations reference D-M2.2 deliverables
- ✓ All four candidate interpretations formally stated
- ✓ Consistency analysis complete for each candidate against ER-007, ER-008, ER-009, S0
- ✓ Recommended definition tagged [Hypothesized] — not promoted to [Recovered] without new evidence
- ✓ Alternatives documented with explicit rationale for rejection or deferral
- ✓ Hypothesis propagation rule verified

---

### WP-M2.5 — Base-Cap Enforcement Semantics (R4)

**Objective:** Validate the mathematical semantics of base-cap enforcement — specifically whether the base credit cap is applied before or after monthly aggregation, and whether it is a hard constraint or a soft penalty.

**Baseline evidence:** `BENCHMARK-KNOWLEDGE-MATRIX-v1.0.md` Base Credit Caps entry — Evidence Status: Convergent Evidence; Mathematical Reconstruction: Partial.

**Sequencing note:** WP-M2.5 is sequenced after WP-M2.2 (credited workload equation) and WP-M2.4 (HC3 definition) because base-cap semantics interact with both. The aggregation order question (cap-then-sum vs sum-then-cap) cannot be formally stated without the monthly aggregation equation from WP-M2.2.

**Open sub-questions:**
- Cap-then-sum vs sum-then-cap: which aggregation order is used?
- Is the cap a hard feasibility constraint or a soft objective penalty?
- Does the cap apply uniformly across bases or is it base-specific?

**Deliverables:**
- **D-M2.5.1 — Base-Cap Constraint Formulation:** Formal mathematical statement of the base-cap constraint in D-M2.1 notation, referencing D-M2.2 and D-M2.4.
- **D-M2.5.2 — Aggregation Order Analysis:** Formal demonstration of the mathematical difference between cap-then-sum and sum-then-cap, with evidence assessment for each.
- **D-M2.5.3 — Enforcement Mechanism:** Statement of whether the cap is hard or soft, with classification tag and confidence.

**Exit Criteria:**
- ✓ All symbols defined in D-M2.1.5; equations reference D-M2.2 and D-M2.4 deliverables
- ✓ Aggregation order analysis formally demonstrates the difference between the two candidates
- ✓ Recommended formulation tagged with classification and confidence
- ✓ Enforcement mechanism (hard vs soft) explicitly stated — not left implicit
- ✓ Hypothesis propagation rule verified

---

### WP-M2.6 — Internal Consistency Validation

**Objective:** Verify that the reconstructed mathematics is internally consistent with every recovered semantic claim from Sprint 10, and that no circular derivations or hypothesis contamination have been introduced.

**Rationale:** This work package is the scientific QA gate before freezing `BENCHMARK-SEMANTICS-v1.0.md`. It ensures that the reconstruction as a whole is coherent, not merely that each work package is individually correct.

**Deliverables:**

- **D-M2.6.1 — Traceability Matrix:** For every definition in D-M2.1 through D-M2.5, a row recording:

  | Definition | Classification | Confidence | Evidence Record | Sprint 10 Semantic Concept | Status |
  |---|---|---|---|---|---|

- **D-M2.6.2 — Dependency Graph:** A directed graph showing the derivation dependencies among all definitions, annotated with classification tags. Used to verify the hypothesis propagation rule and detect circular derivations.

  Example structure:
  ```
  ER-007
     │
     ▼
  Duty Credit Equation [Recovered | High]
     │
     ▼
  Monthly Credit Equation [Derived | High]
     │
     ▼
  HC3 Candidate [Hypothesized | Moderate]
  ```

- **D-M2.6.3 — Consistency Report:** Answers to the following questions:
  - Does every recovered semantic concept from Sprint 10 have a mathematical representation in D-M2.1–D-M2.5?
  - Does any equation contradict ER-007, ER-008, or ER-009?
  - Does any [Derived] definition depend on a [Hypothesized] definition (hypothesis propagation violation)?
  - Are there circular derivations in the dependency graph?
  - Does every [Hypothesized] definition have an explicit rationale explaining why the available evidence is insufficient to classify it as [Recovered] or [Derived]?

**Exit Criteria:**
- ✓ Traceability matrix complete — every definition in D-M2.1–D-M2.5 has a row
- ✓ Dependency graph complete — no undefined edges
- ✓ No hypothesis propagation violations found (or all violations documented and resolved)
- ✓ No circular derivations found
- ✓ Every [Hypothesized] definition has an explicit insufficiency rationale
- ✓ Consistency report signed off — ready to produce `BENCHMARK-SEMANTICS-v1.0.md`

---

## 5. Work Package Sequencing

```
WP-M2.1 — Mathematical Benchmark Model Foundation
    │
    ├──► WP-M2.2 — Credited Workload Equation (R1)
    │         │
    │         ├──► WP-M2.4 — HC3 Definition (R3)
    │         │         │
    │         │         └──► WP-M2.5 — Base-Cap Enforcement (R4)
    │         │                   │
    │         └──────────────────►┘
    │
    └──► WP-M2.3 — Objective Function (R2)  [parallel with WP-M2.2]
    
All WP-M2.1–M2.5 complete
    │
    ▼
WP-M2.6 — Internal Consistency Validation
    │
    ▼
BENCHMARK-SEMANTICS-v1.0.md
```

WP-M2.1 is a strict prerequisite for all subsequent work packages. WP-M2.3 may proceed in parallel with WP-M2.2. WP-M2.4 and WP-M2.5 require WP-M2.2 to be substantially complete. WP-M2.6 requires all of WP-M2.1 through WP-M2.5 to be complete.

---

## 6. Milestone 2 Deliverables

| ID | Deliverable | Depends On | Status |
|----|-------------|------------|--------|
| D-M2.1 | Mathematical Benchmark Model Foundation | Sprint 10 baseline | Pending |
| D-M2.2 | Credited Workload Equation (R1) | D-M2.1 | Pending |
| D-M2.3 | Objective Function Definition (R2) | D-M2.1 | Pending |
| D-M2.4 | HC3 Mathematical Definition (R3) | D-M2.2 | Pending |
| D-M2.5 | Base-Cap Enforcement Formulation (R4) | D-M2.2, D-M2.4 | Pending |
| D-M2.6 | Internal Consistency Validation | D-M2.1–D-M2.5 | Pending |
| D-M2.7 | `BENCHMARK-SEMANTICS-v1.0.md` | D-M2.6 | Pending |

---

## 7. Milestone 2 Completion Criteria

Milestone 2 is complete when all of the following are satisfied:

1. All four research questions (R1–R4) have a formal mathematical answer, tagged according to the Definition Classification Scheme (Section 2).
2. Every definition in the answer set is expressed in the notation established by WP-M2.1.
3. The Mathematical Traceability Principle is satisfied: every symbol is traceable to a Sprint 10 semantic concept or explicitly identified as a hypothesis.
4. The Hypothesis Propagation Rule is satisfied: no [Hypothesized] definition has been used to derive a [Recovered] or [Derived] definition.
5. WP-M2.6 Internal Consistency Validation is complete and the consistency report is signed off.
6. `BENCHMARK-SEMANTICS-v1.0.md` has been produced, reviewed, and frozen.
7. All [Hypothesized] definitions have documented alternatives and explicit uncertainty statements.
8. Every [Hypothesized] definition has an explicit rationale explaining why the available evidence is insufficient to classify it as [Recovered] or [Derived].
9. No [Engineering approximation] definitions are present in the benchmark semantics specification (they may appear in implementation documents but not in the benchmark specification itself).

---

## 8. Relationship to Sprint 10 Frozen Artifacts

| Sprint 10 Artifact | Role in Milestone 2 |
|--------------------|---------------------|
| `BENCHMARK-KNOWLEDGE-MATRIX-v1.0.md` | Primary baseline — open questions drive R1–R4 |
| `CVD-001-MILESTONE4-EVALUATION-v1.0.md` | Benchmark Reconstruction Status table defines starting point |
| `SPRINT10-M1-EVIDENCE-ACQUISITION.md` | Evidence records ER-007, ER-008, ER-009 are primary inputs |
| `SPRINT10-CLOSURE-REPORT-v1.0.md` | Transition section defines Milestone 2 scope |

Sprint 10 artifacts are frozen. Milestone 2 findings shall not be backported into Sprint 10 documents. If genuinely new evidence emerges (e.g., from S5 author correspondence), it shall be recorded in `BENCHMARK-KNOWLEDGE-MATRIX-v1.1.md` as ER-010+ without modifying v1.0.

---

## 9. Project Phase Context

| Phase | Nature | Central Question |
|-------|--------|-----------------|
| Phase 1 (pre-Sprint 10) | Engineering | Can Coralys solve the benchmark? |
| Phase 2 — Sprint 10 | Scientific Reconstruction | What does the benchmark actually mean? |
| Phase 3 — Milestone 2 | Mathematical Reconstruction | What equations best represent the recovered semantics? |

Milestone 2 is construction-oriented, not discovery-oriented. The semantic understanding is substantially complete (Sprint 10). The remaining work is to give that understanding a rigorous mathematical form.

---

## Configuration Control

| Version | Date | Change |
|---------|------|--------|
| v1.0 draft | 2026-07-17 | Initial plan — Milestone 2 opened following Sprint 10 closure |
| v1.0 final | 2026-07-17 | WP-M2.6 (Internal Consistency Validation) added; confidence dimension added to classification scheme; Hypothesis Propagation Rule added; WP-M2.1 renamed to Foundation; per-WP exit criteria added; Mathematical Traceability Principle added; completion criterion 8 added; Minimal Reconstruction Principle added |