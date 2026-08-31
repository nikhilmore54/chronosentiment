# UltraRoster P3.3 — Controlled Reassignment & Change Provenance
## Governance Document — Authorization & Invariants

**Status:** AUTHORIZED — implementation in progress
**Prerequisite:** P3.2 frozen at commit `0623ac904` (branch: `governance-hardening`)
**Author:** UltraRoster product governance
**Date:** 2026-08-30 / updated 2026-08-31

---

## 1. Phase Purpose

P3.3 makes the workflow genuinely useful to a scheduler by closing the gap between *selecting* a roster decision and *acting on it* with confidence.

After a scheduler selects an alternative (P3) and locks manual edits (P3.1/P3.5), UltraRoster redistributes the remaining open assignments. P3.3 ensures that every system-generated change is explicitly identifiable, traceable, and inspectable — and that no scheduler-authored assignment is ever silently overwritten.

---

## 2. Workflow Sequence

```
1. Scheduler selects an alternative (P3)
2. Scheduler makes manual edits → those edits become locked/preserved decisions
3. UltraRoster redistributes remaining open assignments
4. System records exactly what it changed (provenance log)
5. UI distinguishes all four assignment states (see §4)
6. Final coverage / fairness / cost metrics are recalculated
7. Scheduler can inspect why each reassignment happened
8. P2 memory records the resulting decision and modification history
```

---

## 3. Critical Invariant (Frozen Before Implementation)

> **UltraRoster must never silently overwrite a scheduler-authored assignment. Every system-generated reassignment must be explicitly identifiable and traceable to the redistribution operation that produced it.**

This invariant is a hard gate. Any implementation that cannot satisfy it is not authorized to ship.

---

## 4. Assignment Provenance States

The UI must clearly distinguish exactly four states for every assignment cell:

| State | Label | Visual treatment |
|---|---|---|
| `original` | Original assignment | Neutral / default |
| `scheduler_edit` | Scheduler edit | Highlighted — scheduler-authored, locked |
| `system_reassignment` | UltraRoster reassignment | Highlighted — system-generated, traceable |
| `unchanged` | Unchanged | Neutral / default |

No other states are authorized. Ambiguous or unlabeled states are a governance violation.

---

## 5. Provenance Model Requirements

Each `system_reassignment` record must carry:

- `assignmentId` — which assignment was changed
- `previousValue` — what it was before redistribution
- `newValue` — what it became after redistribution
- `redistributionOperationId` — which redistribution run produced this change
- `reason` — human-readable explanation (e.g., "coverage gap on day 14 Night shift")
- `timestamp` — when the redistribution ran

The provenance log must be immutable after the redistribution operation completes. It must be stored alongside the decision in P2 memory.

### Implementation principle: emit, do not diff

**Provenance must be emitted by the redistribution operation itself — not derived after the fact by diffing the final roster against the original roster.**

A final diff can tell us *what changed*, but not reliably *who or what caused the change*. The redistribution function must emit a change record for every assignment it touches at the moment it touches it.

Example of what must be prohibited:

```
Original:       Sarah = OFF
Scheduler edit: Sarah = EARLY       ← locked
Redistribution: Sarah = LATE        ← PROHIBITED — locked cell must not be touched
```

Example of what must produce an explicit system_reassignment record:

```
Original:       Marcus = OFF
Scheduler edit: none
Redistribution: Marcus = EARLY      ← permitted — must emit system_reassignment record
```

---

## 6. Scope Boundaries

### In scope for P3.3
- Provenance model (`AssignmentProvenance` type in `WorkflowTypes.ts`)
- Redistribution operation that respects locked assignments and records provenance
- UI assignment cell rendering for all four states
- "Why was this changed?" inspector panel (per-cell, on demand)
- Final metrics recalculation after redistribution
- P2 memory update to include modification history

### Explicitly out of scope for P3.3
- Rest Pattern Quality soft penalty (W-O-W isolation) — future domain objective, not authorized
- Multi-round redistribution (iterative re-locking) — future phase
- Undo/redo of redistribution — future phase
- Conflict resolution between two scheduler edits — future phase

---

## 7. Phase Progression

```
P1   Explore the Decision
 ↓
P2   Remember the Decision
 ↓
P3   Select the Decision
 ↓
P3.3 Reassign Around the Decision   ← this phase
 ↓
P4   Learn From Past Decisions
```

Rest Pattern Quality remains separate from this progression. It is an objective-model change; P3.3 is about controlled redistribution and provenance.

---

## 8. UI Summary Panel (Required)

After redistribution completes, the UI must display a summary panel. Minimum required content:

```
Redistribution completed

3 scheduler edits preserved
11 assignments reassigned
0 locked assignments changed
Coverage: 196 / 196
```

The "0 locked assignments changed" line is a hard invariant display — it must always be present and must always read 0. A non-zero value here is a governance violation and must block export.

On the roster grid:
- ✎ marks a scheduler edit cell
- ↻ marks a system reassignment cell
- No marker for unchanged or original cells
- Original state is available on demand when inspecting a reassignment cell

---

## 9. Hard Gates (Must Pass Before Ship)

1. A scheduler-locked assignment is never overwritten by redistribution — verified by test.
2. Every `system_reassignment` cell links to a provenance record with all required fields.
3. The provenance log is written atomically with the redistribution result — no partial writes.
4. Final metrics (coverage, fairness, cost) are recalculated from the post-redistribution schedule, not the pre-redistribution schedule.
5. P2 memory stores the modification history alongside the decision record.
6. Build exits 0 (`tsc -b && vite build`).
7. All existing P3.2 regression tests continue to pass (`vitest run`).
8. **Locked-cell preservation test:** Scheduler edit on cell X → redistribution runs → cell X is exactly unchanged. Verified by test.
9. **Adjacent provenance test:** Scheduler edit on cell X → redistribution changes surrounding assignments → every changed surrounding assignment has an individual `system_reassignment` provenance record. This catches the failure mode where locked cells are preserved but adjacent changes are made without provenance.

---

## 9. What Is Not Changing

The following P3.2 invariants remain frozen and must not be touched during P3.3:

- `compareAlternatives()` is presentation-only — no ranking, no `recommendedId`, no `GAP_TOLERANCE` policy.
- `result.recommended_alternative_id` from the optimizer pipeline is propagated unchanged — the adapter does not override it.
- `buildStaffingRequirements()` + `computeCanonicalCoverage()` are the canonical coverage formula.
- Coralys MOGA is the sole optimization authority. The UltraCrew adapter is the sole domain problem definer.
- No second optimization authority will be introduced.

*(Note: `rankAlternatives()` and `GAP_TOLERANCE` were removed in P3.2 cleanup at commit `4da805647`. The above reflects the actual frozen state.)*

---

## 10. Authorization Record

| Item | Decision |
|---|---|
| P3.3 authorized | Yes |
| Rest Pattern Quality authorized | No — future domain objective |
| Multi-round redistribution authorized | No — future phase |
| Undo/redo authorized | No — future phase |
| Prerequisite freeze verified | Yes — `938c6e60f` |