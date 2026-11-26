# CVD-001 Benchmark Knowledge Matrix

**Document:** BENCHMARK-KNOWLEDGE-MATRIX-v1.0.md  
**Date:** 2026-07-16  
**Status:** FROZEN v1.0 — under configuration control; future updates increment version  
**Role:** Bridge between Milestone 1 (evidence acquisition) and Milestone 2 (semantic reconstruction)

---

## Purpose

This matrix records, for every benchmark concept, what is known from five independent sources:

1. **Coralys** — current implementation behavior (Verified)
2. **Literature** — peer-reviewed publications and technical reports (E2)
3. **Dataset** — benchmark artifact contents (E4)
4. **Generator** — dataset-generation code (E3)
5. **Evaluator** — benchmark evaluator source (E1, not yet recovered)

The matrix distinguishes two independent axes for each concept:

- **Semantic Understanding** — how well we understand what the concept represents
- **Mathematical Reconstruction** — how well we have recovered the exact equations or implementation

Sprint 10 has been substantially more successful at semantic reconstruction than mathematical reconstruction. The remaining work is not "find more documents" but "formalize the semantics we already understand."

---

## Status Key

### Semantic Understanding

| Symbol | Meaning |
|---|---|
| ✅ Complete | Concept fully understood from E1/E2 evidence |
| 🔶 High | Concept well understood from E2/E3/E4; minor gaps remain |
| 🟡 Partial | Concept partially understood; key aspects unresolved |
| ❓ Low | Concept poorly understood; significant uncertainty |

### Mathematical Reconstruction

| Symbol | Meaning |
|---|---|
| ✅ Complete | Exact equation or implementation recovered |
| 🔶 Partial | Partial equation or implementation recovered; gaps remain |
| 🔷 High | Most of the mathematical structure recovered; details missing |
| ❓ Not recovered | No mathematical formulation recovered |
| — | Not applicable |

### Evidence Status

| Category | Meaning |
|---|---|
| Verified | Supported by E1 or E2 evidence |
| Convergent Evidence | Three or more independent evidence streams (E2 + E3 + E4) converging |
| Inferred | Supported by E3 or E4 evidence |
| Partially Characterized | Semantics partially recovered; mathematical form unknown |
| Bounded Unknown | Concept constrained by what it is not; candidates identified |
| Unknown | No evidence above E5/E6 |

### Recoverability

| Level | Meaning |
|---|---|
| Complete | Already recovered; no further search needed |
| Medium | Recoverable with targeted effort (thesis audit, author correspondence) |
| Low | Unlikely to be recovered from public sources; requires evaluator or unpublished material |
| Very Low | Almost certainly requires evaluator source or private correspondence |

---

## Two-Axis Knowledge Matrix (post-WP3)

| Concept | Semantic Understanding | Mathematical Reconstruction | Evidence Status | Confidence | Recoverability | Evidence Records |
|---|---|---|---|---|---|---|
| **Planning horizon** | ✅ Complete | ✅ Complete | Verified (E2) | Very High | Complete | ER-005 |
| **Credited hours meaning** | ✅ Complete | 🔶 Partial | Verified (E2) | Very High | Complete | ER-006, ER-007 |
| **Dataset provenance** | ✅ Complete | — | Verified (E2) | Very High | Complete | F17 |
| **Resource model** | 🔶 High | 🔷 High | Convergent Evidence (E2+E3+E4) | Very High | Complete | ER-009 |
| **Base credit caps** | 🔶 High | 🔶 Partial | Convergent Evidence (E2+E3+E4) | High | Complete | F5, ER-009 |
| **Objective function** | 🔶 High | 🔶 Partial | Partially Characterized (E2) | Moderately High | Medium | ER-008 |
| **Deadhead handling** | 🔶 High | ❓ Not recovered | Inferred (E3) | High | Medium | F10 |
| **Duty boundary** | 🔶 High | ❓ Not recovered | Inferred (E3) | High | Medium | F11 |
| **Credit accumulation formula** | 🔶 High | ❓ Not recovered | Partially Characterized (E2+E3+E4) | High | Low | ER-007 |
| **HC3 semantics** | 🟡 Partial | ❓ Not recovered | Bounded Unknown | Moderate | Low | F1–F18 |
| **Briefing/debriefing credit** | 🟡 Partial | ❓ Not recovered | Inferred (E3) | Low | Low | F13 |
| **SC4 preference enforcement** | ❓ Low | ❓ Not recovered | Inferred (E3) | Low | Low | F14 |
| **SC5 vacation enforcement** | ❓ Low | ❓ Not recovered | Inferred (E3) | Low | Low | — |
| **Evaluator source** | — | — | Verified (E2, negative) | Moderately High | Very Low | F18 |

---

## WP3 Evidence Records

### ER-007 — Credit Accumulation Semantic Model

**Finding:** The benchmark computes contractual credited workload from duty-level activities before optimization. The exact accumulation equation remains unrecovered, but its semantic role has been reconstructed with high confidence.

**Reconstructed semantic pipeline:**
```
Flight Legs
      ↓
Duty Construction
      ↓
Duty Credit
      ↓
Monthly Credited Workload
      ↓
Base-Level Aggregation
```

**Evidence:** E2 (literature) + E3 (generator) + E4 (dataset)  
**Confidence:** High  
**Source:** Montréal crew rostering literature; bidline scheduling papers; generator analysis (F13)

---

### ER-008 — Objective Function Characterization

**Finding:** The benchmark objective is a constrained monthly crew rostering objective emphasizing legality under collective agreements together with equitable distribution of contractual credited workload. The precise mathematical aggregation remains unrecovered. No evidence recovered indicates that the benchmark optimizes raw flight hours.

**Recovered components:**
- Legality (safety rules, collective agreement rules)
- Contractual feasibility
- Workload equity (credited-hour balance)
- Monthly schedule quality
- Days-off balance
- Preference satisfaction (when personalized rostering is used)

**Unknown:** weighting, aggregation method, lexicographic ordering

**Evidence:** E2  
**Confidence:** Moderately High  
**Source:** GERAD G-2014-22; Montréal rostering lineage

---

### ER-009 — Resource Model Reconstruction

**Finding:** The CVD-001 dataset is consistent with the standard Montréal monthly crew rostering resource model rather than a simplified academic benchmark. The dataset's resources correspond closely to the full resource model described in the literature.

**Literature resource model:** pairings, duties, rest, days off, reserve blocks, leave, training, union activities, crew base, monthly planning horizon, contractual workload, qualification, legality resources.

**CVD-001 dataset resources confirmed:**
- ✓ Crew base
- ✓ creditedHours
- ✓ Vacations
- ✓ Preferences
- ✓ Daily legs
- ✓ Base caps
- ✓ Monthly horizon

**Note:** Whether reserve, training, and union duty resources exist in CVD-001 is not confirmed. They exist in the Montréal model but have not been identified in the dataset.

**Evidence:** E2 + E3 + E4  
**Confidence:** Very High  
**Source:** Multi-commodity flow crew rostering literature; Montréal lineage; dataset artifact analysis

---

## Open Questions (reordered by dependency)

| Question | Matrix Row | Current Status | Next Action |
|---|---|---|---|
| Q1: Is HC3 a hard feasibility constraint or soft penalty? | HC3 semantics | Bounded Unknown | Milestone 2 |
| Q3: How are credited hours accumulated? | Credit accumulation formula | Partially Characterized (High) | Milestone 2 |
| Q2: Are base caps enforced as hard constraints? | Base credit caps | Convergent Evidence | Freeze after S5 unless contradicted |
| Q6: Does briefing/debriefing credit affect evaluator? | Briefing/debriefing credit | Inferred (E3) | Milestone 2 |
| Q4: Does a public evaluator exist? | Evaluator source | Verified (negative, E2) | S5 (author correspondence) |
| Q5: What is the planning horizon? | Planning horizon | Complete (E2) | Closed |

---

## Remaining Research Questions (post-WP3)

Sprint 10 has answered most semantic questions. What remains are largely **mathematical reconstruction** problems. The distinction is explicit below.

| ID | Question | Nature | Semantic Understanding | Mathematical Reconstruction | Phase |
|---|---|---|---|---|---|
| R1 | Recover exact credited workload equation | Mathematical Recovery | High | Not recovered | Milestone 2 — Mathematical Benchmark Reconstruction |
| R2 | Recover objective aggregation and weighting | Mathematical Recovery | High | Partial | Milestone 2 — Mathematical Benchmark Reconstruction |
| R3 | Recover HC3 mathematical definition | Mathematical Recovery | Partial (Bounded Unknown) | Not recovered | Milestone 2 — Mathematical Benchmark Reconstruction |
| R4 | Validate base-cap enforcement semantics | Semantic Validation | High | Partial | Milestone 2 — Mathematical Benchmark Reconstruction |
| R5 | Reproduce benchmark semantics in Coralys without compromising domain independence | Engineering | — | — | Milestone 3B |

---

## Sprint 10 Final Status

| Domain | Status |
|---|---|
| Dataset provenance | ✅ Complete |
| Scientific lineage | ✅ Complete |
| Semantic reconstruction | 🔶 Substantially Complete |
| Resource model reconstruction | ✅ Complete |
| Objective characterization | 🔶 Substantially complete |
| Mathematical reconstruction | 🟡 Partial |
| Evaluator reconstruction | ❓ Not recovered |

---

## Scientific Stopping Rule

Sprint 10 closes when public evidence has been exhausted and the remaining unknowns are explicitly documented.

**Completed (required for closure):**
- ✅ Dataset provenance recovered
- ✅ Literature lineage recovered
- ✅ Planning semantics recovered
- ✅ Resource semantics recovered
- ✅ Workload semantics recovered
- ✅ Objective characterized
- ✅ Remaining unknowns explicitly documented with recoverability assessment

**Optional post-sprint activity:**
- Author correspondence (S5) — if successful, new evidence shall be treated as a future revision of this matrix (v1.1+) and recorded as ER-010 or later. S5 does not block Sprint 10 closure.

Remaining unknowns are limited to mathematical implementation details. Further general literature search is unlikely to increase E2 evidence. Remaining recovery requires the evaluator, author correspondence, or unpublished material.

Sprint 10 stopped because the marginal scientific value of continued general search became low — not because the search was abandoned. Research should not depend on external events outside its control.

---

## Milestone 2 Priority Order

1. **HC3 semantics** — highest impact; blocks all reproduction decisions; Bounded Unknown (candidates: contractual credit upper bound, bidline legality, monthly workload legality, collective agreement limit)
2. **Credit accumulation formula** — semantic role recovered; exact equation needed
3. **Evaluator objective** — substantially characterized; mathematical aggregation needed
4. **Base cap enforcement type** — Convergent Evidence; freeze unless S5 contradicts

---

## Benchmark Reconstruction Principle

> Coralys shall reproduce benchmark semantics only when supported by sufficient evidence. Unknown benchmark behavior shall remain explicitly documented as unknown rather than replaced by speculative implementations.

This principle distinguishes this project from reproduction efforts that quietly fill gaps with assumptions. Every claim in this matrix is individually traceable to its evidence source.

---

## Configuration Control

This document is frozen at v1.0. Future updates must:
- Increment the version number (v1.1, v1.2, ...)
- Add new evidence records (ER-010, ER-011, ...)
- Update only affected rows, preserving the evolution history
- Record the change in the Version History table

---

## Version History

| Version | Date | Change |
|---|---|---|
| v1.0 | 2026-07-16 | Initial frozen version — post S0–S4b–WP3 evidence; two-axis model (Semantic Understanding / Mathematical Reconstruction); nuanced status categories; WP3 results (ER-007, ER-008, ER-009); Scientific Stopping Rule; Recoverability column; Configuration Control |