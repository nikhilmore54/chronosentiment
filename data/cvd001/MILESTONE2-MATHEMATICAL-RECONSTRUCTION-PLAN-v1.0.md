# Milestone 2 — Mathematical Benchmark Reconstruction Plan
## CVD-001 Benchmark Reconstruction Project

**Document ID:** MILESTONE2-PLAN-v1.0  
**Status:** ACTIVE  
**Branch:** governance-hardening  
**Baseline:** Sprint 10 artifacts frozen at commit `721c086c`  
**Created:** 2026-07-17  

---

## 0. Scope and Purpose

This document defines the work plan for Milestone 2 — Mathematical Benchmark Reconstruction.

Sprint 10 (Milestone 1) established a stable, evidence-based semantic understanding of the CVD-001 benchmark and documented the limits of public knowledge. The frozen Sprint 10 artifacts constitute the permanent archival baseline for all Milestone 2 work.

Milestone 2 does not revisit evidence acquisition. Its sole objective is:

> **Recover the missing mathematics of an already-understood benchmark.**

All mathematical definitions produced in Milestone 2 shall be tagged according to the Definition Classification Scheme defined in Section 2. This preserves the Benchmark Reconstruction Principle throughout the mathematical reconstruction effort.

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

Every mathematical definition, equation, and constraint introduced in Milestone 2 shall carry one of the following tags:

| Tag | Meaning |
|-----|---------|
| **[Recovered]** | Directly supported by evidence from the Sprint 10 evidence hierarchy (E1–E5). The evidence source shall be cited. |
| **[Derived]** | Logically implied by one or more Recovered definitions. The derivation chain shall be stated. |
| **[Hypothesized]** | A candidate reconstruction consistent with available evidence but not uniquely determined by it. Alternatives shall be noted. |
| **[Engineering approximation]** | Introduced for implementation tractability. Not claimed to reproduce benchmark semantics. |

This scheme operationalises the Benchmark Reconstruction Principle: unknown benchmark behavior remains explicitly documented as unknown rather than silently replaced by speculative implementations.

---

## 3. Work Packages

### WP-M2.1 — Mathematical Benchmark Model (Foundation)

**Objective:** Establish the formal mathematical language that all subsequent work packages will use.

**Rationale:** Jumping directly into HC3 or objective weighting without a shared notation risks inconsistency across work packages and makes the final specification harder to audit. WP-M2.1 produces the mathematical scaffolding first.

**Deliverables:**

- **D-M2.1.1 — Index Sets:** Crew members, duties, pairings, bases, days, flight legs, months, resource types. Each set tagged [Recovered] or [Hypothesized].
- **D-M2.1.2 — Parameters:** Flight leg durations, scheduled departure/arrival times, base assignments, contractual limits. Each parameter tagged with evidence source.
- **D-M2.1.3 — Decision Variables:** Pairing-to-crew assignment variables, duty construction variables, and any auxiliary variables required for constraint modelling.
- **D-M2.1.4 — Derived Quantities:** Credited workload per duty, credited workload per month, duty elapsed time, rest periods. Each derivation tagged and linked to ER-007.
- **D-M2.1.5 — Notation Glossary:** Symbol table with tag, definition, evidence source, and cross-reference to Sprint 10 evidence records.

**Baseline evidence:** ER-007 (credit accumulation pipeline), ER-009 (resource model), `BENCHMARK-KNOWLEDGE-MATRIX-v1.0.md` Section 3.

**Completion criterion:** All symbols used in WP-M2.2 through WP-M2.5 are defined in D-M2.1.5 with classification tags.

---

### WP-M2.2 — Credited Workload Equation (R1)

**Objective:** Recover the exact equation by which flight leg durations are transformed into credited hours at the duty level and aggregated to the monthly level.

**Baseline evidence:** ER-007 (credit accumulation semantic pipeline — five-stage model: Flight Legs → Duty Construction → Duty Credit → Monthly Credited Workload → Base-Level Aggregation).

**Open sub-questions:**
- Is the credit function linear in flight duration, or does it apply a cap-then-sum vs sum-then-cap logic?
- Are deadhead legs credited at full, partial, or zero rate?
- Is the base-level aggregation a simple sum or does it apply a secondary transformation?

**Deliverables:**
- **D-M2.2.1 — Duty Credit Equation:** Formal equation mapping flight leg durations within a duty to a single duty credit value. Tagged [Recovered] or [Hypothesized] per sub-question resolution.
- **D-M2.2.2 — Monthly Aggregation Equation:** Formal equation mapping duty credits within a month to monthly credited workload.
- **D-M2.2.3 — Deadhead Treatment:** Explicit statement of deadhead credit semantics with evidence tag.

**Completion criterion:** A single tagged equation for monthly credited workload expressible in terms of D-M2.1 symbols.

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
- **D-M2.3.2 — Weighting Candidates:** Enumeration of candidate weighting schemes consistent with the recovered structure, with rationale for each.
- **D-M2.3.3 — Negative Finding Record:** Explicit documentation of what could not be recovered and why, consistent with the three-statement-class methodology.

**Completion criterion:** A tagged objective function definition with explicit uncertainty bounds on unrecovered coefficients.

---

### WP-M2.4 — HC3 Mathematical Definition (R3)

**Objective:** Recover the mathematical definition of constraint HC3.

**Baseline evidence:** `BENCHMARK-KNOWLEDGE-MATRIX-v1.0.md` HC3 entry — Semantic Understanding: Partial; Mathematical Reconstruction: Not recovered. Bounded Unknown: almost certainly not a weekly 40h cap; candidates are contractual credit upper bound, bidline legality, monthly workload legality, collective agreement limit.

**Open sub-questions:**
- Is HC3 a hard constraint or a soft penalty?
- Does it operate at the duty, pairing, or monthly level?
- Which of the four candidate interpretations is most consistent with the recovered credit accumulation model (WP-M2.2)?

**Approach:** WP-M2.4 is sequenced after WP-M2.2 because the credited workload equation (R1) is a prerequisite for formally stating any workload-based constraint.

**Deliverables:**
- **D-M2.4.1 — HC3 Candidate Definitions:** Formal mathematical statement of each candidate interpretation, expressed in D-M2.1 notation.
- **D-M2.4.2 — Consistency Analysis:** For each candidate, assessment of consistency with ER-007, ER-008, ER-009, and the CVD-001 generator code observations from S0.
- **D-M2.4.3 — Recommended Definition:** The most evidence-consistent candidate, tagged [Hypothesized] with explicit statement of remaining uncertainty.

**Completion criterion:** A single recommended HC3 definition tagged [Hypothesized] with documented alternatives.

---

### WP-M2.5 — Base-Cap Enforcement Semantics (R4)

**Objective:** Validate the mathematical semantics of base-cap enforcement — specifically whether the base credit cap is applied before or after monthly aggregation, and whether it is a hard constraint or a soft penalty.

**Baseline evidence:** `BENCHMARK-KNOWLEDGE-MATRIX-v1.0.md` Base Credit Caps entry — Evidence Status: Convergent Evidence; Mathematical Reconstruction: Partial.

**Open sub-questions:**
- Cap-then-sum vs sum-then-cap: which aggregation order is used?
- Is the cap a hard feasibility constraint or a soft objective penalty?
- Does the cap apply uniformly across bases or is it base-specific?

**Approach:** WP-M2.5 is sequenced after WP-M2.2 (credited workload equation) and WP-M2.4 (HC3 definition) because base-cap semantics interact with both.

**Deliverables:**
- **D-M2.5.1 — Base-Cap Constraint Formulation:** Formal mathematical statement of the base-cap constraint in D-M2.1 notation.
- **D-M2.5.2 — Aggregation Order Analysis:** Formal demonstration of the difference between cap-then-sum and sum-then-cap, with evidence assessment for each.
- **D-M2.5.3 — Enforcement Mechanism:** Statement of whether the cap is hard or soft, with evidence tag.

**Completion criterion:** A tagged base-cap constraint definition with explicit statement of aggregation order.

---

## 4. Work Package Sequencing

```
WP-M2.1 (Mathematical Benchmark Model)
    │
    ├──► WP-M2.2 (Credited Workload Equation — R1)
    │         │
    │         ├──► WP-M2.4 (HC3 Definition — R3)
    │         │         │
    │         │         └──► WP-M2.5 (Base-Cap Enforcement — R4)
    │         │
    │         └──► WP-M2.5 (Base-Cap Enforcement — R4)
    │
    └──► WP-M2.3 (Objective Function — R2)  [parallel with WP-M2.2]
```

WP-M2.1 is a strict prerequisite for all subsequent work packages. WP-M2.3 may proceed in parallel with WP-M2.2. WP-M2.4 and WP-M2.5 require WP-M2.2 to be substantially complete.

---

## 5. Milestone 2 Deliverables

| ID | Deliverable | Depends On | Status |
|----|-------------|------------|--------|
| D-M2.1 | Mathematical Benchmark Model (notation, sets, parameters, variables) | Sprint 10 baseline | Pending |
| D-M2.2 | Credited Workload Equation (R1) | D-M2.1 | Pending |
| D-M2.3 | Objective Function Definition (R2) | D-M2.1 | Pending |
| D-M2.4 | HC3 Mathematical Definition (R3) | D-M2.2 | Pending |
| D-M2.5 | Base-Cap Enforcement Formulation (R4) | D-M2.2, D-M2.4 | Pending |
| D-M2.6 | `BENCHMARK-SEMANTICS-v1.0.md` | D-M2.1–D-M2.5 | Pending |

---

## 6. Governing Principles

All Milestone 2 work is governed by the principles established in Sprint 10:

**Benchmark Reconstruction Principle** (from `BENCHMARK-KNOWLEDGE-MATRIX-v1.0.md`):
> Coralys shall reproduce benchmark semantics only when supported by sufficient evidence. Unknown benchmark behavior shall remain explicitly documented as unknown rather than replaced by speculative implementations.

**Three Statement Classes** (from `CVD-001-MILESTONE4-EVALUATION-v1.0.md` Section 0):
- Implementation findings: what Coralys currently computes
- Empirical findings: observations from running Coralys on CVD-001
- Benchmark findings: semantics supported by authoritative evidence

The Definition Classification Scheme (Section 2 of this document) operationalises these principles at the level of individual mathematical definitions.

---

## 7. Relationship to Sprint 10 Frozen Artifacts

| Sprint 10 Artifact | Role in Milestone 2 |
|--------------------|---------------------|
| `BENCHMARK-KNOWLEDGE-MATRIX-v1.0.md` | Primary baseline — open questions drive R1–R4 |
| `CVD-001-MILESTONE4-EVALUATION-v1.0.md` | Benchmark Reconstruction Status table defines starting point |
| `SPRINT10-M1-EVIDENCE-ACQUISITION.md` | Evidence records ER-007, ER-008, ER-009 are primary inputs |
| `SPRINT10-CLOSURE-REPORT-v1.0.md` | Transition section defines Milestone 2 scope |

Sprint 10 artifacts are frozen. Milestone 2 findings shall not be backported into Sprint 10 documents. If genuinely new evidence emerges (e.g., from S5 author correspondence), it shall be recorded in `BENCHMARK-KNOWLEDGE-MATRIX-v1.1.md` as ER-010+ without modifying v1.0.

---

## 8. Milestone 2 Completion Criteria

Milestone 2 is complete when:

1. All four research questions (R1–R4) have a formal mathematical answer, tagged according to the Definition Classification Scheme.
2. Every definition in the answer set is expressed in the notation established by WP-M2.1.
3. `BENCHMARK-SEMANTICS-v1.0.md` has been produced, reviewed, and frozen.
4. All [Hypothesized] definitions have documented alternatives and explicit uncertainty statements.
5. No [Engineering approximation] definitions are present in the benchmark semantics specification (they may appear in implementation documents but not in the benchmark specification itself).

---

## Configuration Control

| Version | Date | Change |
|---------|------|--------|
| v1.0 | 2026-07-17 | Initial plan — Milestone 2 opened following Sprint 10 closure |