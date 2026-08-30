# UltraRoster P3 — Governance Frozen State

**Date:** 2026-08-30
**Branch:** governance-hardening
**Last commit:** 82c694377

---

## Frozen Capability State

| Capability | Status | Commit |
|---|---|---|
| Candidate alternatives | WORKING | 8eb536a22 |
| Canonical coverage (demand-based, 196 req. positions) | WORKING | 5c43d657d |
| Coverage-dominant recommendation | WORKING | 82c694377 |
| Recommendation vs scheduler selection (override tracking) | WORKING | 8eb536a22 |
| Decision Memory (P2) | WORKING | 4c19355e2 |
| Single-alternative honesty | WORKING | 8eb536a22 |
| P3 UI (SelectDecision, ReviewSchedule, ExportRoster) | WORKING | 98d5ad960 |
| Regression test suite (7/7 pass) | WORKING | 82c694377 |
| P1.1 diversity optimization | DEFERRED | — |
| Rest Pattern Quality | FUTURE DOMAIN OBJECTIVE | — |
| Coralys | FROZEN | — |
| Optimizer / MOGA | FROZEN | — |

---

## Architectural Invariant (P3.2)

**Optimizer/API scoring ≠ UltraRoster recommendation.**

The API can supply candidate alternatives, but UltraRoster's product-level
decision policy (`rankAlternatives()` in `WorkflowUtils.ts`) is authoritative
when determining what to recommend.

Call path (Render / synthetic scenario):
```
GenerateSchedule
    → onResult
    → buildSyntheticAlternatives(staff, sched)   [or API alternatives]
    → rankAlternatives(altsToRank)               [AUTHORITATIVE]
    → setRecommendedId(ranked.recommendedId)
    → SelectDecision renders the adapter's result
```

The API's `recommended_alternative_id` is **never** used directly.

---

## Decision Policy (rankAlternatives)

Priority hierarchy — explicit, not a magic weight:

1. **Minimize uncovered required positions** (primary operational objective)
2. Among alternatives within `GAP_TOLERANCE = 5` positions, minimize
   `fairness_penalty + cost / 100` (secondary objectives)
3. Deterministic tie-break: first alternative wins (stable)

### Regression gate cases

| Case | Expected | coverageDominant |
|---|---|---|
| 40/196 vs 194/196 | 194/196 wins | true |
| 190/196 vs 196/196 (gap=6) | 196/196 wins | true |
| 191/196 vs 196/196 (gap=5, boundary) | secondary decides | false |
| 194/196 vs 194/196 | lower fairness+cost wins | false |
| 196/196 vs 196/196 equal | first wins (stable) | false |
| single alternative | that alternative | false |
| empty | empty id | — |

All 7 cases pass in `rankAlternatives.test.ts` (vitest, 187ms).

---

## Future Domain Objective: Rest Pattern Quality

**Status: RECORDED, NOT AUTHORIZED**

Observation (2026-08-30): The deployed UI exposes technically valid schedules
whose human-operational quality is not adequately represented by the current
metrics. The alternating W-O-W pattern (work → isolated day off → work) is
undesirable even when all hard constraints are satisfied.

**Proposed formulation:**
- Name: **Rest Pattern Quality**
- Type: configurable **soft penalty** (not a hard constraint)
- Penalize isolated days off surrounded by working days (W-O-W)
- Reward consecutive rest blocks (W-O-O-W preferred)
- Strength: configurable parameter

**Penalty table (proposed):**

| Pattern | Treatment |
|---|---|
| W-O-W | Strong penalty |
| W-O-O-W | Preferred |
| W-O-O-O-W | Preferred, subject to max-rest rules |
| W-W-O-W | Moderate penalty |
| W-W-O-O-W | Good |

**Architecture:** belongs in the UltraCrew domain adapter / constraint model,
not the UI. The UI renders the metric; it does not calculate it.

**Authorization required before implementation.** Do not open optimizer or
Coralys until a separately authorized objective-model phase is defined.

---

## Frozen Boundaries

**IN (P3, frozen):**
- Decision selection UI
- Canonical coverage (demand-based)
- Coverage-priority recommendation
- Adapter-layer ranking (`rankAlternatives`)
- Decision memory (P2)
- Regression test suite

**OUT (explicitly deferred):**
- MOGA / optimizer changes
- Alternative-generation diversity (P1.1)
- Rest Pattern Quality implementation
- Coralys changes
- P4 memory recommendation
- UI redesign beyond current P3 scope

---

## Next Authorized Capability

**Manual scheduler edits → preserve edits → intelligently redistribute
remaining work** (previously discussed as P3.2 Edit & Rebalance, now
deferred until P3 is frozen and separately authorized).

This is a cleaner product progression than reopening the optimizer.