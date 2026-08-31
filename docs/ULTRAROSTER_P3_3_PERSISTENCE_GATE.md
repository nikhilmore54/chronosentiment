# P3.3 Persistence Gap — Scope Gate Finding
## Investigation: A or B?

**Baseline:** `governance-hardening @ 5693eb1ab`
**Scope:** Inspection only. No code changes.
**Date:** 2026-08-31

---

## Question

Is the absence of `RedistributionLog` persistence in P2 Decision Memory:

**A — P3.3 defect:** persistence is required by the existing P3.3 specification and a minimal persistence correction is authorized for consideration.

**B — P4 prerequisite:** P3.3 is functioning as specified; persistent redistribution history was never part of P3.3, so the gap remains a P4 dependency.

---

## Evidence

### P3.3 governance spec — explicit requirements

**§2 Workflow Sequence, step 8:**
> "P2 memory records the resulting decision and modification history"

**§6 In scope for P3.3:**
> "P2 memory update to include modification history"

**§9 Hard Gate 5:**
> "P2 memory stores the modification history alongside the decision record."

### Current P2 memory implementation

`DecisionRepository.ts` persists three localStorage keys:
- `ultracrew_recommendations` — `Recommendation[]`
- `ultracrew_decision_log` — `Decision[]`
- `ultracrew_scheduler_decisions` — `SchedulerDecision[]` (P3)

`SchedulerDecision` records: `decision_id`, `created_at_iso`, `recommended_id`, `selected_id`, `overrode_recommendation`.

**`RedistributionLog` is not persisted.** No `P3_3_KEY`, no `saveRedistributionLog()`, no `loadRedistributionLog()`.

### What "modification history" means in the P3.3 spec

The P3.3 spec defines modification history as the `RedistributionLog` — the record of what the system changed during redistribution, including `ChangeRecord[]` (per-cell provenance with reason, previous/new value, operation ID, timestamp) and `provenanceMap`.

`SchedulerDecision` does not record redistribution. It records only which alternative was selected. The redistribution log is entirely separate.

---

## P3.3 Scope Gate Applied

**Gate question 1:** Is persistence of the redistribution log required for the authorized P3.3 workflow to function correctly?

The P3.3 workflow (§2) ends at step 8: "P2 memory records the resulting decision and modification history." Without persistence, the modification history is lost on page refresh. The scheduler cannot return to a previous session and inspect what the system changed. Hard gate 5 is not satisfied.

**Answer: YES** — persistence is required by the P3.3 workflow as specified.

**Gate question 2:** Is the absence of persistence a defect in an explicitly defined P3.3 invariant?

Hard gate 5 is explicit: "P2 memory stores the modification history alongside the decision record." This gate was not satisfied by the current implementation.

**Answer: YES** — hard gate 5 is an explicitly defined P3.3 invariant that is not satisfied.

**Gate question 3:** Can it be corrected minimally without changing P3.3 intended behavior or architecture?

The correction is: add `saveRedistributionLog(decisionId, log)` and `loadRedistributionLog(decisionId)` to `DecisionRepository`, keyed to the `decision_id`. Call `saveRedistributionLog` from the redistribution completion path. No new objectives, no new UI, no Coralys changes, no new data structures beyond what P3.3 already defines.

**Answer: YES** — the correction is minimal and does not change intended behavior or architecture.

---

## Conclusion

**A — P3.3 defect.**

All three scope gate questions are YES. The absence of `RedistributionLog` persistence is a defect in an explicitly defined P3.3 invariant (hard gate 5). A minimal persistence correction is authorized for consideration.

This is not a P4 requirement retroactively imposed on P3.3. It is a P3.3 requirement that was not implemented.

---

## Authorized correction (minimal)

Subject to approval:

1. Add `private readonly P3_3_KEY = 'ultracrew_redistribution_logs'` to `DecisionRepository`
2. Add `saveRedistributionLog(decisionId: string, log: RedistributionLog): void` — stores log keyed by `decisionId`
3. Add `loadRedistributionLog(decisionId: string): RedistributionLog | null` — retrieves log by `decisionId`
4. Update `clear()` to remove `P3_3_KEY`
5. Call `saveRedistributionLog` from the redistribution completion path in `ReviewSchedule.tsx` (after `setRedistResult(r)`)

No other changes. No new UI. No new objectives. No Coralys changes.

After correction: re-run `vitest run` (26/26 must pass), verify build exits 0, re-freeze P3.3.

---

*This document is the gate finding only. Implementation requires explicit authorization.*