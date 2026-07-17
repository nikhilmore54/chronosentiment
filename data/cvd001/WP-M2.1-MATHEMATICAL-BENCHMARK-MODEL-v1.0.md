# WP-M2.1 — Mathematical Benchmark Model Foundation
## CVD-001 Benchmark Reconstruction Project — Milestone 2

**Document ID:** WP-M2.1-v1.0  
**Work Package:** WP-M2.1 (Mathematical Benchmark Model Foundation)  
**Status:** DRAFT  
**Governance baseline:** MILESTONE2-MATHEMATICAL-RECONSTRUCTION-PLAN-v1.0.md (frozen at `fc505cba`)  
**Evidence baseline:** Sprint 10 artifacts frozen at `721c086c`  
**Created:** 2026-07-17  
**Revised:** 2026-07-17 (confidence calibration — evaluator-only evidence capped at High; equations for WP-M2.2 concepts deferred; Mathematical Scope Boundary added)

---

## 0. Purpose

This document establishes the formal mathematical language for Milestone 2. Every symbol, set, parameter, variable, and derived quantity used in WP-M2.2 through WP-M2.6 shall be defined here before use.

Each definition is a structured record with the following fields:

| Field | Purpose |
|-------|---------|
| **Symbol** | Mathematical notation |
| **Definition** | Formal meaning (concept only — equations deferred to reconstruction WPs) |
| **Classification** | [Recovered] / [Derived] / [Hypothesized] |
| **Confidence** | Very High / High / Moderate / Low |
| **Evidence source** | Sprint 10 evidence record or artifact |
| **Semantic concept** | Corresponding Sprint 10 semantic concept |
| **Rationale** | Why this classification and confidence were assigned |

**Confidence calibration rule:** Per the Three Statement Classes (MILESTONE2-PLAN Section 3), evaluator source code (E1) is an implementation artifact, not authoritative benchmark documentation. Confidence is capped at **High** when the sole direct evidence is evaluator code, unless independently corroborated by a benchmark source (G-2014-22, generator code S0, or peer-reviewed literature). Very High confidence requires independent corroboration from at least two distinct evidence types.

Engineering approximations ([Engineering approximation]) shall not appear in this document. They are permitted only in implementation documents.

---

## 1. Mathematical Scope Boundary

WP-M2.1 defines the mathematical vocabulary of the benchmark reconstruction. It establishes the sets, parameters, variables, and derived quantity concepts that all subsequent work packages will use.

WP-M2.1 intentionally does not define:
- the credited workload equation (deferred to WP-M2.2, R1),
- the objective function (deferred to WP-M2.3, R2),
- HC3 semantics (deferred to WP-M2.4, R3),
- base-cap enforcement (deferred to WP-M2.5, R4).

Formal equations for quantities whose reconstruction is assigned to WP-M2.2 through WP-M2.5 are not introduced here. WP-M2.1 defines the concept and defers the equation.

---

## 2. Index Sets (D-M2.1.1)

### 2.1 Crew Members

| Field | Value |
|-------|-------|
| **Symbol** | N |
| **Definition** | The finite set of crew members (pilots or cabin crew) covered by the scheduling problem. Each element n ∈ N is associated with a contract type and a home base. |
| **Classification** | [Recovered] |
| **Confidence** | Very High |
| **Evidence source** | ER-009 (resource model); G-2014-22 Section 2 (crew rostering problem statement); S0 (generator code enumerates crew members with contract assignments) |
| **Semantic concept** | Crew resource — the schedulable unit of the benchmark |
| **Rationale** | The existence of a finite crew set is directly stated in G-2014-22 and confirmed by the generator code (S0). Two independent evidence types (E2 benchmark documentation + E3 generator code) corroborate this definition. Very High confidence is warranted. |

---

### 2.2 Planning Days

| Field | Value |
|-------|-------|
| **Symbol** | D |
| **Definition** | The ordered finite set of calendar days in the planning horizon. Days are indexed d = 1, 2, …, |D|. |
| **Classification** | [Recovered] |
| **Confidence** | Very High |
| **Evidence source** | G-2014-22 (monthly planning horizon explicitly stated); S0 (generator code produces monthly schedules); ER-007 (credit accumulation pipeline operates over days) |
| **Semantic concept** | Planning horizon — the temporal scope of the scheduling problem |
| **Rationale** | G-2014-22 explicitly describes a monthly crew rostering problem. The generator code (S0) produces monthly schedules. Two independent evidence types corroborate this definition. Very High confidence is warranted. |

---

### 2.3 Shift Types

| Field | Value |
|-------|-------|
| **Symbol** | S |
| **Definition** | The finite set of shift types (duty types) available for assignment. Each element s ∈ S represents a category of work assignment (e.g., flight duty, reserve duty, day off). |
| **Classification** | [Recovered] |
| **Confidence** | High |
| **Evidence source** | ER-009 (resource model — flight duties, reserve duties, days off confirmed); S0 (generator code enumerates shift categories) |
| **Semantic concept** | Shift type — the categorical classification of a daily assignment |
| **Rationale** | The generator code (S0) produces assignments with distinct type codes. ER-009 confirms the standard Montréal resource model includes flight duties and reserve duties. Confidence is High rather than Very High because the complete enumeration of shift types in CVD-001 has not been independently verified against an authoritative list from G-2014-22. |

---

### 2.4 Skill / Qualification Categories

| Field | Value |
|-------|-------|
| **Symbol** | K |
| **Definition** | The finite set of skill or qualification categories. Each element k ∈ K represents a crew qualification (e.g., aircraft type rating, position). Assignments are made within a qualification category. |
| **Classification** | [Hypothesized] |
| **Confidence** | Moderate |
| **Evidence source** | ER-009 (resource model — qualification structure inferred from Montréal model); S0 (generator code references qualification-related fields) |
| **Semantic concept** | Crew qualification — the eligibility constraint linking crew members to assignments |
| **Rationale** | The Montréal monthly crew rostering model includes qualification categories, and the generator code references qualification-related fields. However, the exact structure of K in CVD-001 (whether it is a single category or multiple) has not been directly confirmed from authoritative documentation. Classified [Hypothesized | Moderate] pending direct evidence. Downstream WPs referencing K shall apply the Hypothesis Propagation Rule. |

---

### 2.5 Flight Legs

| Field | Value |
|-------|-------|
| **Symbol** | F |
| **Definition** | The finite set of flight legs in the planning horizon. Each element f ∈ F is a scheduled flight with a departure station, arrival station, departure time, arrival time, and block time (scheduled flight duration). |
| **Classification** | [Recovered] |
| **Confidence** | Very High |
| **Evidence source** | ER-007 (credit accumulation pipeline — Flight Legs is Stage 1); S0 (generator code reads flight leg data from CVD-001 dataset); G-2014-22 (flight legs are the atomic scheduling unit) |
| **Semantic concept** | Flight leg — the atomic unit of the credit accumulation pipeline |
| **Rationale** | ER-007 explicitly identifies Flight Legs as the first stage of the five-stage credit accumulation pipeline. The generator code (S0) reads and processes flight leg records. G-2014-22 describes flight legs as the atomic scheduling unit. Three independent evidence types corroborate this definition. Very High confidence is warranted. |

---

### 2.6 Duties

| Field | Value |
|-------|-------|
| **Symbol** | T |
| **Definition** | The finite set of duties (pairings or work blocks) available for assignment. Each duty t ∈ T consists of one or more flight legs on consecutive days, with an associated credit value (defined as derived quantity c_t in Section 4.1). |
| **Classification** | [Recovered] |
| **Confidence** | High |
| **Evidence source** | ER-007 (credit accumulation pipeline — Duty Construction is Stage 2, Duty Credit is Stage 3) |
| **Semantic concept** | Duty — the intermediate aggregation unit between flight legs and monthly workload |
| **Rationale** | ER-007 establishes that flight legs are grouped into duties before credit is computed. The duty set T is therefore a necessary intermediate structure. Confidence is High rather than Very High because the exact duty construction rules (rest requirements, connection times) have not been recovered from public artifacts. |

---

### 2.7 Bases

| Field | Value |
|-------|-------|
| **Symbol** | B |
| **Definition** | The finite set of crew bases (home stations). Each crew member n ∈ N is assigned to exactly one base b(n) ∈ B. |
| **Classification** | [Recovered] |
| **Confidence** | High |
| **Evidence source** | ER-009 (resource model — base assignment confirmed); S0 (generator code assigns crew members to bases) |
| **Semantic concept** | Crew base — the home station determining crew assignment eligibility and workload aggregation |
| **Rationale** | The generator code (S0) assigns crew members to bases, and ER-009 confirms the base structure is part of the resource model. Confidence is High because the role of bases in workload aggregation (Base-Level Aggregation, Stage 5 of ER-007) is confirmed, but the exact number of bases in CVD-001 has not been independently verified from G-2014-22. |

---

## 3. Parameters (D-M2.1.2)

### 3.1 Flight Leg Block Time

| Field | Value |
|-------|-------|
| **Symbol** | δ_f |
| **Definition** | The scheduled block time (flight duration) of flight leg f ∈ F, measured in hours. δ_f > 0 for all f. |
| **Classification** | [Recovered] |
| **Confidence** | Very High |
| **Evidence source** | ER-007 (credit accumulation pipeline — flight duration is the input to Stage 1); S0 (generator code reads block times from dataset); G-2014-22 (block time is the standard airline credit unit) |
| **Semantic concept** | Flight leg duration — the primary input to the credit accumulation computation |
| **Rationale** | Block time is the standard airline industry measure of flight duration and is the direct input to credit computation per ER-007. The generator code reads this value from the dataset. G-2014-22 uses block time as the credit unit. Three independent evidence types corroborate this definition. Very High confidence is warranted. |

---

### 3.2 Contractual Credit Base

| Field | Value |
|-------|-------|
| **Symbol** | W^min_n |
| **Definition** | The minimum monthly credited workload (credit base) specified in the contract of crew member n ∈ N, measured in hours. |
| **Classification** | [Recovered] |
| **Confidence** | High |
| **Evidence source** | ER-007 (credit accumulation pipeline — base-level aggregation references contractual limits); BENCHMARK-KNOWLEDGE-MATRIX-v1.0.md (Base Credit Caps — Convergent Evidence); E1 evaluator source (`contract.base`) |
| **Semantic concept** | Contractual credit base — the lower bound on monthly credited workload |
| **Rationale** | The Knowledge Matrix records Convergent Evidence for base credit caps. The evaluator source (E1) references `contract.base` directly. Confidence is High: the evaluator provides strong implementation evidence, but the exact mapping between contract types and base values in CVD-001 has not been independently verified from G-2014-22. Evaluator-only evidence is capped at High per the confidence calibration rule. |

---

### 3.3 Contractual Credit Cap

| Field | Value |
|-------|-------|
| **Symbol** | W^max_n |
| **Definition** | The maximum monthly credited workload (credit cap) specified in the contract of crew member n ∈ N, measured in hours. |
| **Classification** | [Recovered] |
| **Confidence** | High |
| **Evidence source** | ER-007 (credit accumulation pipeline); BENCHMARK-KNOWLEDGE-MATRIX-v1.0.md (Base Credit Caps — Convergent Evidence); E1 evaluator source (`contract.cap`) |
| **Semantic concept** | Contractual credit cap — the upper bound on monthly credited workload |
| **Rationale** | Same evidence basis as W^min_n. The evaluator source (E1) references `contract.cap` directly. Confidence is High per the confidence calibration rule (evaluator-only evidence capped at High). |

---

### 3.4 Duty Credit Value

| Field | Value |
|-------|-------|
| **Symbol** | c_t |
| **Definition** | The credited hours associated with duty t ∈ T. Computed from the flight legs comprising duty t. The exact computation rule is deferred to WP-M2.2 (R1). This parameter is listed here because it appears as a fixed input to the assignment model once duties are constructed. |
| **Classification** | [Derived] |
| **Confidence** | High |
| **Evidence source** | ER-007 (Duty Credit — Stage 3 of the credit accumulation pipeline) |
| **Semantic concept** | Duty credit — the credited workload contribution of a single duty |
| **Rationale** | ER-007 establishes that duty credit is computed from flight leg durations. The existence of this parameter follows necessarily from the five-stage pipeline structure. Its exact value depends on the WP-M2.2 equation. Classified [Derived | High]: the concept is [Recovered] from ER-007, but the derivation chain passes through the WP-M2.2 equation which is not yet formally established. |

---

### 3.5 Assignment Weight

| Field | Value |
|-------|-------|
| **Symbol** | w_{n,d,s,k} |
| **Definition** | The credited workload contribution of assigning crew member n ∈ N to shift type s ∈ S under qualification k ∈ K on day d ∈ D. This is the weight used in the workload accumulation equation (deferred to WP-M2.2, R1). |
| **Classification** | [Recovered] |
| **Confidence** | High |
| **Evidence source** | ER-007 (credit accumulation pipeline); E1 evaluator source (`assignment.weight`) |
| **Semantic concept** | Assignment weight — the per-assignment credit contribution |
| **Rationale** | The evaluator source (E1) directly references `assignment.weight` in the workload accumulation step. This is one of the most directly evidenced parameters in the model. Confidence is High per the confidence calibration rule: the evaluator provides strong implementation evidence, but the exact computation of `assignment.weight` from underlying flight data has not been independently verified from G-2014-22. |

---

## 4. Decision Variables (D-M2.1.3)

### 4.1 Assignment Variable

| Field | Value |
|-------|-------|
| **Symbol** | x_{n,d,s,k} |
| **Definition** | Binary decision variable. x_{n,d,s,k} = 1 if crew member n ∈ N is assigned to shift type s ∈ S under qualification k ∈ K on day d ∈ D; x_{n,d,s,k} = 0 otherwise. |
| **Classification** | [Recovered] |
| **Confidence** | High |
| **Evidence source** | ER-007 (credit accumulation pipeline — workload is accumulated over assignments); E1 evaluator source (iterates over assignments to accumulate workload) |
| **Semantic concept** | Schedule assignment — the primary decision in the crew rostering problem |
| **Rationale** | The evaluator source (E1) iterates over assignments and accumulates workload, confirming the existence of an assignment structure. The binary formulation is standard for crew rostering and consistent with the Montréal model (ER-009). Confidence is High per the confidence calibration rule: the binary formulation is inferred from the standard model rather than directly stated in G-2014-22. The index structure (whether k is explicit or implicit) is subject to the [Hypothesized | Moderate] classification of K. |

---

## 5. Derived Quantity Concepts (D-M2.1.4)

**Note:** This section defines the *concepts* of derived quantities. Formal equations are deferred to the reconstruction work packages (WP-M2.2 through WP-M2.5) as specified in the Mathematical Scope Boundary (Section 1).

### 5.1 Monthly Credited Workload

| Field | Value |
|-------|-------|
| **Symbol** | W_n |
| **Definition** | The total monthly credited workload of crew member n ∈ N. Computed by accumulating assignment weights over the planning horizon. Formal equation deferred to WP-M2.2 (R1). |
| **Classification** | [Recovered] |
| **Confidence** | High |
| **Evidence source** | ER-007 (Monthly Credited Workload — Stage 4 of the credit accumulation pipeline); E1 evaluator source (`workload[nurse_id] += assignment.weight`) |
| **Semantic concept** | Monthly credited workload — the aggregate workload measure used in constraint evaluation |
| **Rationale** | The evaluator source (E1) directly implements workload accumulation as a sum over assignments. ER-007 identifies this as Stage 4 of the pipeline. The concept is [Recovered | High]: the summation structure is confirmed by E1, but the exact treatment of deadhead legs and duty-level caps is unresolved and will be addressed in WP-M2.2. Confidence is High per the confidence calibration rule (E1 is the primary direct evidence). |

---

### 5.2 Workload Balance Target

| Field | Value |
|-------|-------|
| **Symbol** | t_n |
| **Definition** | The workload balance target for crew member n ∈ N. A reference value derived from the contractual credit range. Formal equation deferred to WP-M2.2 (R1) or WP-M2.3 (R2) as appropriate. |
| **Classification** | [Recovered] |
| **Confidence** | High |
| **Evidence source** | E1 evaluator source (`let target = (contract.base + contract.cap) / 2.0`) |
| **Semantic concept** | Workload balance target — the reference point for the workload balance soft constraint |
| **Rationale** | The evaluator source (E1) directly computes this quantity as `(contract.base + contract.cap) / 2.0`. This is strong implementation evidence. Confidence is High per the confidence calibration rule: E1 is the sole direct evidence source; no independent benchmark documentation (G-2014-22) has been found that specifies this formula. The formula may well be correct, but the evidence hierarchy reserves Very High for independently corroborated facts. |

---

### 5.3 Workload Deviation

| Field | Value |
|-------|-------|
| **Symbol** | Δ_n |
| **Definition** | The deviation of crew member n's monthly credited workload from the balance target. A non-negative quantity used in the workload balance penalty. Formal equation deferred to WP-M2.3 (R2). |
| **Classification** | [Derived] |
| **Confidence** | High |
| **Evidence source** | E1 evaluator source (`(w - target).abs()`); ER-008 (objective function characterization — workload balance component confirmed) |
| **Semantic concept** | Workload deviation — the input to the workload balance penalty term |
| **Rationale** | Derived from W_n and t_n. The absolute deviation formulation is confirmed by E1. ER-008 independently confirms the existence of a workload balance component in the objective. Confidence is High: E1 provides the formula and ER-008 provides independent confirmation of the concept, but neither is authoritative benchmark documentation from G-2014-22. |

---

## 6. Notation Glossary (D-M2.1.5)

The following table provides the canonical symbol reference for all Milestone 2 work packages. Every symbol used in WP-M2.2 through WP-M2.6 shall appear in this table.

| Symbol | Type | Definition (brief) | Classification | Confidence | Evidence | Section |
|--------|------|--------------------|----------------|------------|----------|---------|
| N | Set | Crew members | [Recovered] | Very High | ER-009, G-2014-22, S0 | 2.1 |
| D | Set | Planning days | [Recovered] | Very High | G-2014-22, S0, ER-007 | 2.2 |
| S | Set | Shift types | [Recovered] | High | ER-009, S0 | 2.3 |
| K | Set | Qualification categories | [Hypothesized] | Moderate | ER-009, S0 | 2.4 |
| F | Set | Flight legs | [Recovered] | Very High | ER-007, S0, G-2014-22 | 2.5 |
| T | Set | Duties | [Recovered] | High | ER-007 | 2.6 |
| B | Set | Crew bases | [Recovered] | High | ER-009, S0 | 2.7 |
| δ_f | Parameter | Block time of flight leg f | [Recovered] | Very High | ER-007, S0, G-2014-22 | 3.1 |
| W^min_n | Parameter | Contractual credit base for crew n | [Recovered] | High | ER-007, E1 | 3.2 |
| W^max_n | Parameter | Contractual credit cap for crew n | [Recovered] | High | ER-007, E1 | 3.3 |
| c_t | Parameter | Duty credit value for duty t (equation in WP-M2.2) | [Derived] | High | ER-007 | 3.4 |
| w_{n,d,s,k} | Parameter | Assignment weight (equation in WP-M2.2) | [Recovered] | High | ER-007, E1 | 3.5 |
| x_{n,d,s,k} | Variable | Assignment decision (binary) | [Recovered] | High | ER-007, E1 | 4.1 |
| W_n | Derived | Monthly credited workload (equation in WP-M2.2) | [Recovered] | High | ER-007, E1 | 5.1 |
| t_n | Derived | Workload balance target (equation in WP-M2.2/M2.3) | [Recovered] | High | E1 | 5.2 |
| Δ_n | Derived | Workload deviation (equation in WP-M2.3) | [Derived] | High | E1, ER-008 | 5.3 |

---

## 7. WP-M2.1 Exit Criteria Assessment

| Criterion | Status | Notes |
|-----------|--------|-------|
| All index sets defined with classification tags and confidence levels | ✓ Complete | Sections 2.1–2.7 |
| All parameters defined with evidence source citations | ✓ Complete | Sections 3.1–3.5 |
| All decision variables defined with scope and domain | ✓ Complete | Section 4.1 |
| Notation glossary (D-M2.1.5) complete | ✓ Complete | Section 6 |
| Mathematical Traceability Principle verified | ✓ Complete | Every symbol traceable to Sprint 10 evidence or flagged [Hypothesized] |
| Confidence calibration rule applied | ✓ Complete | Evaluator-only evidence capped at High throughout |
| Equations for WP-M2.2–M2.5 concepts deferred | ✓ Complete | W_n, t_n, Δ_n, c_t equations deferred to reconstruction WPs |
| No symbol used in WP-M2.2–M2.6 is undefined | Pending | To be verified as subsequent WPs are executed |

**One open item:** K (qualification categories) is classified [Hypothesized | Moderate]. This is the only set whose structure has not been directly confirmed from authoritative documentation. WP-M2.2 and WP-M2.4 shall note this dependency and apply the Hypothesis Propagation Rule if K appears in their equations.

---

## 8. Dependency Notes for Subsequent Work Packages

- **WP-M2.2 (R1 — Credited Workload Equation):** Will use N, D, S, K, w_{n,d,s,k}, x_{n,d,s,k}, W_n, δ_f, c_t. Note that K is [Hypothesized | Moderate]; any equation referencing K inherits at most [Hypothesized] classification per the Hypothesis Propagation Rule.
- **WP-M2.3 (R2 — Objective Function):** Will use N, W_n, t_n, Δ_n, W^min_n, W^max_n. All inputs are [Recovered | High] or [Derived | High].
- **WP-M2.4 (R3 — HC3 Definition):** Will use N, W_n, W^min_n, W^max_n. All inputs are [Recovered | High] or better.
- **WP-M2.5 (R4 — Base-Cap Enforcement):** Will use N, B, W_n, W^max_n. All inputs are [Recovered | High] or better.

---

## Configuration Control

| Version | Date | Change |
|---------|------|--------|
| v1.0 draft | 2026-07-17 | Initial WP-M2.1 execution — D-M2.1.1 through D-M2.1.5 produced |
| v1.0 revised | 2026-07-17 | Confidence calibration applied (evaluator-only evidence capped at High); equations for WP-M2.2–M2.5 concepts deferred; Mathematical Scope Boundary added (Section 1); confidence calibration rule documented in Section 0 |