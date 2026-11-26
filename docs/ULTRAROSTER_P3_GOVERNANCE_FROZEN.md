# UltraRoster P3 — Governance Frozen State

**Date:** 2026-08-31
**Branch:** governance-hardening
**Last commit:** 4da805647

---

## Frozen Capability State

| Capability | Status | Commit |
|---|---|---|
| Candidate alternatives | WORKING | 8eb536a22 |
| Canonical coverage (demand-based, 196 req. positions) | WORKING | 5c43d657d |
| HC1 coverage_deficit as objective[5] (weight 1000) | WORKING | 9d106d201 |
| HC1 verification — 38/38 archive members HC1-feasible | VERIFIED | 00ab239d7 |
| Recommendation authority in optimizer pipeline | WORKING | 4da805647 |
| Recommendation vs scheduler selection (override tracking) | WORKING | 8eb536a22 |
| Decision Memory (P2) | WORKING | 4c19355e2 |
| Single-alternative honesty | WORKING | 8eb536a22 |
| P3 UI (SelectDecision, ReviewSchedule, ExportRoster) | WORKING | 98d5ad960 |
| Regression test suite (26/26 pass) | WORKING | 4da805647 |
| P1.1 diversity optimization | DEFERRED | — |
| Rest Pattern Quality | FUTURE DOMAIN OBJECTIVE | — |
| Coralys | FROZEN | — |
| Optimizer / MOGA | FROZEN | — |

---

## Architectural Invariant (P3.2) — UPDATED 2026-08-31

**Recommendation authority is exclusively in the optimizer pipeline; domain policy authority remains in the adapter.**

### Authority model

```
UltraCrew / UltraRoster adapter
        │
        │ defines the optimization problem:
        │ • employees / eligibility
        │ • coverage requirements (HC1 minimum — absolute demand)
        │ • sequence constraints
        │ • soft objectives, penalties, weights
        ▼
   coralys_moga  (generic engine — no domain knowledge)
        │
        │ optimizes the supplied problem
        │ produces Pareto candidate set
        ▼
   result.recommended_alternative_id
        │
        ▼
   UltraCrew adapter
        │
        └── compareAlternatives()
              metrics only: coverage, utilization, fairness, cost, diffFromFirst
              NO ranking, NO recommendedId, NO GAP_TOLERANCE policy
        ▼
          UI presents alternatives
        │
        ▼
      Scheduler makes the human decision
```

### Key distinctions

- **UltraCrew adapter:** defines *what the optimization problem means* (domain policy authority).
- **Coralys MOGA:** performs *the optimization* and produces the candidate set/recommendation (recommendation authority).
- **`compareAlternatives()`:** describes candidates; does **not** rank them.
- **UI:** presents alternatives.
- **Scheduler:** makes the human decision.

Coralys must remain domain-agnostic. It must not contain airline-specific rules,
"minimum 3 consecutive workdays", "Night → Early is forbidden", or any domain semantics.
Those belong in the adapter/domain problem definition.

Coralys receives: `{decision variables, hard constraints, soft constraints, objectives, weights, evaluation function}` and optimizes without knowing whether the problem is airline crew rostering, hospital staffing, or anything else.

### Call path (Render / synthetic scenario)

```
GenerateSchedule
    → onResult
    → buildSyntheticAlternatives(staff, sched)   [or API alternatives]
    → result.recommended_alternative_id          [AUTHORITATIVE — from optimizer]
    → setRecommendedId(recId)
    → compareAlternatives(alts)                  [metrics only — no ranking]
    → SelectDecision renders alternatives + optimizer recommendation
```

The API's `recommended_alternative_id` is propagated **unchanged**.
The adapter does not re-rank or override the optimizer's decision.

### What was removed (P3.2 cleanup, commit 4da805647)

`rankAlternatives()` was a compensating control for the HC1 optimizer defect
(40/196 preferred over 194/196). After the HC1 correction (commit 9d106d201,
verified at 00ab239d7), the compensating control was eliminated:

- `rankAlternatives()` + `RankingResult` removed from `WorkflowUtils.ts`
- `GAP_TOLERANCE`, `coverageDominant`, adapter-side `reason` string removed
- `PlannerWorkflow.tsx` no longer overrides the optimizer recommendation
- `compareAlternatives()` + `AlternativeComparison` added (presentation only)

There is now only one optimization authority.

### Regression test suite (26/26 pass)

- `rankAlternatives.test.ts` — 9 tests for `compareAlternatives()` shape, metrics, diffFromFirst, invariant
- `selectDecision.test.ts` — 11 tests (8 G-10 canonical metric order + 3 G-10 adapter-authority invariant)
- `redistribution.test.ts` — 6 tests

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

## Frozen Boundaries — P3.2 Closure (2026-08-31)

**CLOSED / FROZEN (do not reopen):**
- P3.2 recommendation authority — `compareAlternatives()` is presentation-only; optimizer pipeline is the sole recommendation authority
- HC1 correction — `coverage_deficit` as `objective[5]`, weight 1000, verified 38/38 archive members HC1-feasible
- Sequence-feasibility correction
- Adapter-vs-Coralys boundary — Coralys is a generic engine; domain policy belongs in the adapter
- `compareAlternatives()` presentation-only role — no `GAP_TOLERANCE`, no `recommendedId`, no `coverageDominant`
- Decision memory (P2)
- Regression test suite (26/26 pass)

**NEXT AUTHORIZED (P3.3):**
- Controlled re-optimization around scheduler decisions
- Lock → redistribution → provenance → recalculate metrics → scheduler reviews
- See `docs/ULTRAROSTER_P3_3_GOVERNANCE.md`

**EXPLICITLY NOT OPEN:**
- Pareto ranking redesign
- Adapter recommendation heuristics
- Generalized constraint learning
- Objective-weight research
- Rest Pattern Quality implementation (recorded, not authorized — belongs in adapter/domain model when authorized)
- P4 behavior during P3.3 (P4 is the planned next phase — not active during P3.3)
- Alternative-generation diversity (P1.1)
- Coralys changes
- UI redesign beyond current P3 scope

**Scope discipline:**
> P3 makes the optimizer's decision transparent and safely controllable.
> P4 learns from the decisions.
> These must not be conflated.

---

## P3.3 Scope Gate (active implementation rule)

**P3.3 is the only active implementation scope.** P3.2 stays closed.

For each issue encountered during P3.3 implementation, apply this gate:

1. Is it required for the authorized P3.3 behavior to function correctly?
2. Is it a defect in an already-defined P3.3 invariant?
3. Can it be fixed without changing the product's intended behavior or architecture?

**If yes to all three:** minimal implementation correction is permitted.

**Otherwise:** record it, do not expand scope, continue or stop according to the governance gate.

This rule is particularly important as UltraRoster approaches POC → pilot stage. The goal is a coherent demonstrable product, not continuous optimizer broadening.

### P3.3 authorized scope

- Scheduler edits
- Locking (scheduler edits become hard locks)
- Controlled redistribution of unlocked cells
- Explicit system-reassignment provenance (emitted at moment of change, not post-hoc)
- Recalculation of metrics after redistribution
- Scheduler review of redistributed schedule

### P3.3 not authorized (even if discovered during P3.3)

- Optimizer changes
- New objectives or weights
- Coralys changes
- P4 behavior (P4 is the planned next phase after P3.3 — not active during P3.3)
- Anything that requires reopening P3.2 frozen boundaries

### P4 — planned next phase (not active during P3.3)

P4 is the planned successor to P3.3, not a vague future idea. It is simply outside the current implementation scope.

P4 scope: learning from decisions and modification history — adaptive objectives, historical choice influence, modification-history learning, using scheduler decisions to improve future optimization.

P3 makes the decision transparent and controllable. P4 makes the system learn from those decisions. Nothing from P4 should be implemented prematurely during P3.3.