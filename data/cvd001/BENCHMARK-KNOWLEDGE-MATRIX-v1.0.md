# CVD-001 Benchmark Knowledge Matrix

**Document:** BENCHMARK-KNOWLEDGE-MATRIX-v1.0.md  
**Date:** 2026-07-16  
**Status:** LIVING — updated as Sprint 10 evidence is acquired  
**Role:** Bridge between Milestone 1 (evidence acquisition) and Milestone 2 (semantic reconstruction)

---

## Purpose

This matrix records, for every benchmark concept, what is known from five independent sources:

1. **Coralys** — current implementation behavior (Verified)
2. **Literature** — peer-reviewed publications and technical reports (E2)
3. **Dataset** — benchmark artifact contents (E4)
4. **Generator** — dataset-generation code (E3)
5. **Evaluator** — benchmark evaluator source (E1, not yet recovered)

The matrix makes explicit which cells are known, which are inferred, and which remain unknown. Milestone 2 is a systematic exercise in reducing the "Unknown" cells using the Evidence Hierarchy.

---

## Evidence Status Key

| Symbol | Meaning |
|---|---|
| ✅ Verified | Supported by E1 or E2 evidence |
| 🔶 Inferred | Supported by E3 or E4 evidence |
| ❓ Unknown | No evidence above E5/E6 |
| ⚠ Hypothesized | E5/E6 only — treat as working hypothesis |
| — | Not applicable |

---

## Knowledge Matrix

| Concept | Coralys | Literature | Dataset | Generator | Evaluator | Confidence |
|---|---|---|---|---|---|---|
| **Planning horizon** | Not modeled (no Scenario) | ✅ Monthly bid period (E2, ER-005) | 🔶 31 days / 31 files | 🔶 31-day window | ❓ Not recovered | Very High (literature) |
| **Credited hours meaning** | Absent from API | ✅ Contractual paid workload (E2, ER-006) | 🔶 `creditedHours` file present | 🔶 Generated from reference solution | ❓ Not recovered | Very High (literature) |
| **Base caps** | Not modeled | ❓ Unknown | 🔶 `credit_constrains.csv` present (BASE1=326.9h, BASE2=1279.4h, BASE3=383.3h) | 🔶 Generated with 3% slack from reference solution (F5) | ❓ Not recovered | High (inferred) |
| **HC3 semantics** | Fixed 40h threshold on `duration_hours` | ❓ Unknown | ❓ Unknown | ❓ Not in generator code | ❓ Not recovered | Low |
| **Credit accumulation formula** | `duration_hours` sum | ❓ Unknown | 🔶 `creditedHours` values imply formula | 🔶 Briefing/debriefing adjustment in generator (F13) | ❓ Not recovered | Medium (inferred) |
| **Deadhead handling** | Excluded from duty counting | — | 🔶 TDH prefix in dataset | ✅ Excluded from duty counting and preferences (F10) | ❓ Not recovered | High |
| **Duty boundary** | New calendar day | — | — | ✅ New calendar day in flight number chars 4–5 (F11) | ❓ Not recovered | High |
| **Briefing/debriefing credit** | Not modeled | ❓ Unknown | ❓ Unknown | 🔶 Generator increments accumulator by 1 per duty (F13) | ❓ Not recovered | Low (observation only) |
| **SC4 preference enforcement** | Not modeled | ❓ Unknown | 🔶 `PreferredAirLegs.csv` present | 🔶 Generator creates preference data (F14) | ❓ Not recovered | Low |
| **SC5 vacation enforcement** | Not modeled | ❓ Unknown | 🔶 `personalizedEmployees.csv` present | 🔶 Generator creates vacation data | ❓ Not recovered | Low |
| **Objective function** | Coralys fitness (10000 − penalties) | ❓ Unknown | ❓ Unknown | ❓ Unknown | ❓ Not recovered | Low |
| **HC3 enforcement type** | Hard constraint (−500 per violation) | ❓ Unknown | ❓ Unknown | ❓ Unknown | ❓ Not recovered | Low |
| **Base cap enforcement type** | Not modeled | ❓ Unknown | ❓ Unknown | ❓ Unknown | ❓ Not recovered | Low |
| **Evaluator existence** | — | ✅ No publicly distributed evaluator found (E2, F18) | — | — | ❓ Not recovered | Moderately High (negative) |
| **Benchmark provenance** | — | ✅ GERAD G-2014-22 (E2, F17) | ✅ G1422-DataSets.zip | ✅ Official supplementary material | — | High |

---

## Open Questions Mapped to Matrix

| Question | Matrix Row | Current Status |
|---|---|---|
| Q1: Is HC3 a hard feasibility constraint or soft penalty? | HC3 enforcement type | ❓ Unknown |
| Q2: Are base caps enforced as hard constraints? | Base cap enforcement type | ❓ Unknown |
| Q3: How are credited hours accumulated? | Credit accumulation formula | 🔶 Inferred (Medium) |
| Q4: Does a public evaluator exist? | Evaluator existence | ✅ No (negative E2, F18) |
| Q5: What is the planning horizon? | Planning horizon | ✅ Monthly (E2, ER-005) |
| Q6: Does briefing/debriefing credit affect evaluator? | Briefing/debriefing credit | 🔶 Generator behavior only (F13) |

---

## Progress Summary

| Status | Count | Concepts |
|---|---|---|
| ✅ Verified (E1/E2) | 4 | Planning horizon, Credited hours meaning, Evaluator existence (negative), Benchmark provenance |
| 🔶 Inferred (E3/E4) | 6 | Base caps, Credit accumulation, Deadhead handling, Duty boundary, SC4/SC5 data generation |
| ❓ Unknown | 5 | HC3 semantics, HC3 enforcement type, Base cap enforcement type, Objective function, Briefing/debriefing evaluator behavior |
| ⚠ Hypothesized | 0 | — |

---

## Milestone 2 Priority

Milestone 2 should address the Unknown cells in priority order:

1. **HC3 semantics** — highest impact; blocks all reproduction decisions
2. **HC3 enforcement type** — determines whether HC3 is a feasibility constraint or penalty
3. **Credit accumulation formula** — needed to validate `creditedHours` interpretation
4. **Base cap enforcement type** — needed to determine whether base caps affect feasibility
5. **Objective function** — needed for fitness comparison in Milestone 3B

---

## Version History

| Version | Date | Change |
|---|---|---|
| v1.0 | 2026-07-16 | Initial matrix — post S0–S4b evidence |