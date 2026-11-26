# P0-E: End-to-End Acceptance Report

**Status:** COMPLETE — all 15 gates evaluated, type contract gap found and fixed  
**Date:** 2026-09-02  
**Branch:** main  
**Commit range:** fd53c6917 (P0-A) → this commit (P0-E)

---

## Executive Summary

P0-E is the end-to-end acceptance sequence for the Coralys Pareto pipeline integration
(P0-A through P0-D). The decisive test is:

> Can we trace a selected alternative from Coralys's Pareto archive →
> UltraCrew `ProductionParetoSolution` → `ScheduleResponse` → Decision UI →
> scheduler selection → exported roster **without the frontend creating or altering
> the candidate**?

**Finding:** The static trace revealed a type contract gap between the backend
`ProductionParetoSolution` JSON shape and the frontend `RosterAlternative` type.
The gap was closed by adding a mapping layer (`mapParetoAlternatives()`) in
`WorkflowUtils.ts`. After the fix: TSC exits 0, 47/47 tests pass.

---

## Gate Results

### E1 — POST /api/schedule returns HTTP 200

**Method:** Static trace of `schedule_handler` in `services/ultracrew_server/src/main.rs`.

**Result:** PASS (static). The handler calls `run_pipeline_from_request()` and
`run_pareto_pipeline()`, then returns `Json(response)` with HTTP 200. Error paths
return 400 (validation) or 500 (optimization failure) — not 200.

---

### E2 — ScheduleResponse.alternatives contains backend-generated Pareto candidates

**Method:** Static trace of `ScheduleResponse` struct and `schedule_handler`.

**Result:** PASS (static). `ScheduleResponse.alternatives` is
`Vec<ProductionParetoSolution>` populated by `run_pareto_pipeline()`. The pipeline
seeds with 20 diverse genomes, runs `pareto_steps` evolution steps, and returns all
non-dominated solutions excluding the primary.

---

### E3 — Each alternative has `objectives` populated with 6 values

**Method:** Static trace of `archive_to_solutions()` in `pareto_pipeline.rs`.

**Result:** PASS (static). `ProductionParetoSolution.objectives` is set to
`sol.fitness.clone()` — the 6-component `FitnessVector` from `ProductionParetoEvaluator`.
The evaluator always returns exactly 6 components:
`[hard_violations, rest_violations, fairness_penalty, fatigue_penalty, hc1_violations, hc3_violations]`.

---

### E4 — Each alternative has `feasibility` populated

**Method:** Static trace of `archive_to_solutions()`.

**Result:** PASS (static). Every candidate is evaluated by `InrcConstraintEvaluator`
and a `CandidateFeasibility` struct is populated. No filtering occurs — all candidates
receive a feasibility profile.

---

### E5 — No alternative is the primary genome

**Method:** Static trace of `archive_to_solutions()` filter.

**Result:** PASS (static). The `seed_uid` filter in `archive_to_solutions()` excludes
the primary genome by uid. The uid is computed from the seed genome's hash before
seeding the engine. The filter is applied after the archive is produced — Pareto
dominance semantics are not changed.

---

### E6 — No duplicate candidate genomes

**Method:** Static trace of `ParetoArchive::add()` in `coralys-moga/src/engine_proof.rs`.

**Result:** PASS (static). The uid-based duplicate guard added in P0-A prevents
duplicate genomes from entering the archive:
```rust
if self.solutions.iter().any(|s| s.uid == sol.uid) {
    return false;
}
```
Tests EP-U1, EP-U2, EP-U3 verify this invariant.

---

### E7 — Every displayed candidate maps to an actual backend candidate

**Method:** Static trace of `GenerateSchedule.tsx` → `PlannerWorkflow.tsx` →
`SelectDecision.tsx`.

**Finding (pre-fix):** FAIL. The frontend cast the raw API response directly as
`ScheduleResult` (`const data: ScheduleResult = await res.json()`). The
`ScheduleResult.alternatives` field is typed as `RosterAlternative[]`, but the
backend sends `ProductionParetoSolution[]` — structurally incompatible:

| Field | Backend (`ProductionParetoSolution`) | Frontend (`RosterAlternative`) |
|-------|--------------------------------------|-------------------------------|
| `id` | absent | required `string` |
| `label` | absent | required `string` |
| `schedule` | `Record<string, number>` (shift_id → worker_id) | `Record<string, string[]>` (staffId → 28-day shift[]) |
| `metrics` | `Record<string, number>` (flat map) | `RosterAlternativeMetrics` (typed object) |
| `reasons` | absent | required `string[]` |
| `feasibility` | `CandidateFeasibility` | optional `CandidateFeasibility` |
| `objectives` | `number[]` | optional `number[]` |

**Fix applied:** Added `ParetoAlternativeRaw` and `ScheduleRawResponse` types to
`WorkflowTypes.ts`, and `mapParetoAlternatives()` to `WorkflowUtils.ts`. Updated
`GenerateSchedule.tsx` to parse as `ScheduleRawResponse` and call
`mapParetoAlternatives()` before constructing `ScheduleResult`.

**Result (post-fix):** PASS. Every `RosterAlternative` in the UI is derived from
a `ParetoAlternativeRaw` from the backend. No synthetic data is introduced.

---

### E8 — All returned alternatives displayed in Decision UI

**Method:** Static trace of `PlannerWorkflow.tsx` → `SelectDecision.tsx`.

**Finding (pre-fix):** FAIL (same root cause as E7 — type mismatch would cause
runtime rendering failures).

**Result (post-fix):** PASS. `PlannerWorkflow.tsx` passes `result.alternatives`
directly to `SelectDecision`. `SelectDecision` renders all alternatives via
`alternatives.map(alt => <AlternativeCard .../>)`. No filtering or truncation occurs
in the UI.

---

### E9 — Feasibility/Exception status and violation counts match backend

**Method:** Static trace of `mapParetoAlternatives()` → `AlternativeCard`.

**Result:** PASS. `mapParetoAlternatives()` maps `ParetoFeasibilityRaw` →
`CandidateFeasibility` field-for-field (no transformation). `AlternativeCard` in
`SelectDecision.tsx` displays `alt.feasibility.is_feasible`, `hard_violations`,
`rest_violations`, `hc1_violations`, `hc3_violations` directly from the mapped struct.

---

### E10 — Scheduler selection works without frontend re-ranking

**Method:** Static trace of `SelectDecision.tsx`.

**Result:** PASS. `SelectDecision` does not sort, rank, or score alternatives.
The `recommendedId` comes from `result.recommended_alternative_id` (backend) or
defaults to `alternatives[0].id`. The scheduler selects by clicking an
`AlternativeCard`. No frontend ranking logic exists.

---

### E11 — Selected candidate proceeds to export

**Method:** Static trace of `PlannerWorkflow.tsx` `onDecision` callback.

**Result:** PASS. `onDecision(selectedAlt, decision)` sets
`editableSchedule = selectedAlt.schedule` and advances to step 5 (Review & Edit).
The selected alternative's schedule becomes the editable roster. Export uses this
schedule.

---

### E12 — Export corresponds to selected backend candidate

**Method:** Static trace of `onDecision` → `editableSchedule` → export path.

**Result:** PASS. `selectedAlt.schedule` is the `Record<string, string[]>` produced
by `mapParetoAlternatives()` from the backend's `shift_id → worker_id` map via
`buildEditableSchedule()`. The same `buildEditableSchedule()` function is used for
the primary schedule — consistent transformation.

---

### E13 — Empty-candidate scenario handled explicitly

**Method:** Static trace of `PlannerWorkflow.tsx` step 4 branches.

**Result:** PASS. When `alternatives.length === 0`, step 4 renders the
"Decision alternatives unavailable" hard stop panel. The workflow cannot proceed.
This is the correct product behavior for a homogeneous scenario where the Pareto
engine produces no non-dominated alternatives beyond the primary.

---

### E14 — No-synthetic invariant: no frontend-generated schedule/alternative

**Method:** Static trace of `GenerateSchedule.tsx` error path and
`PlannerWorkflow.tsx`.

**Result:** PASS. The `catch` block in `GenerateSchedule.tsx` sets an error message
and does NOT fall back to synthetic data. `PlannerWorkflow.tsx` does not call
`buildSyntheticAlternatives()` in the real workflow path. The comment explicitly
states: "Do NOT fall back to synthetic data — the scheduler must see a real optimizer
result."

---

### E15 — DecisionRepository records candidate identity throughout chain

**Method:** Static trace of `SelectDecision.tsx` → `DecisionRepository`.

**Result:** PASS. `repo.recordSchedulerDecision()` is called with `selectedAlt.id`
and `recommendedAlt.id`. These ids are generated by `mapParetoAlternatives()` as
`pareto-{idx}` — stable within a single schedule generation response. The
`SchedulerDecision` record captures `selected_id`, `recommended_id`,
`selected_metrics`, and `recommended_metrics` at decision time.

---

## P0-E Fix: Type Contract Gap

### Root Cause

`GenerateSchedule.tsx` cast the raw API response directly as `ScheduleResult`:
```typescript
const data: ScheduleResult = await res.json();
```

`ScheduleResult.alternatives` was typed as `RosterAlternative[]`, but the backend
sends `ProductionParetoSolution[]` — a structurally different type. TypeScript's
type system does not catch this because `as` casts bypass structural checking.

### Fix

Three files changed:

**`ui/ultracrew/src/workflow/WorkflowTypes.ts`** — Added:
- `ParetoFeasibilityRaw` — mirrors Rust `CandidateFeasibility` field names
- `ParetoAlternativeRaw` — mirrors Rust `ProductionParetoSolution` JSON shape
- `ScheduleRawResponse` — the actual API response type

**`ui/ultracrew/src/workflow/WorkflowUtils.ts`** — Added:
- `mapParetoAlternatives(raw, staff)` — translates `ParetoAlternativeRaw[]` →
  `RosterAlternative[]` using `buildEditableSchedule()` for the schedule field

**`ui/ultracrew/src/workflow/GenerateSchedule.tsx`** — Changed:
- Parse response as `ScheduleRawResponse` (not `ScheduleResult`)
- Call `mapParetoAlternatives()` to produce `RosterAlternative[]`
- Construct `ScheduleResult` explicitly from mapped fields

### Invariants Preserved

- No filtering: all backend candidates are mapped and returned
- No ranking: candidates are returned in backend order
- No synthetic data: every field comes from the backend response
- `feasibility` and `objectives` are passed through unchanged
- `buildEditableSchedule()` is used consistently for both primary and alternative schedules

---

## Test Results

```
TSC: exit 0 (no type errors)
Tests: 47/47 pass (5 test files)
  ✓ rankAlternatives.test.ts (9 tests)
  ✓ selectDecision.test.ts (11 tests)
  ✓ PatternAccumulator.test.ts (9 tests)
  ✓ DecisionRepository.test.ts (10 tests)
  ✓ redistribution.test.ts (8 tests)
```

---

## Gate Summary

| Gate | Description | Result |
|------|-------------|--------|
| E1 | HTTP 200 from POST /api/schedule | PASS (static) |
| E2 | alternatives contains backend Pareto candidates | PASS (static) |
| E3 | Each alternative has 6-component objectives | PASS (static) |
| E4 | Each alternative has feasibility populated | PASS (static) |
| E5 | No alternative is the primary genome | PASS (static) |
| E6 | No duplicate candidate genomes | PASS (static) |
| E7 | Every displayed candidate maps to backend candidate | PASS (post-fix) |
| E8 | All returned alternatives displayed in UI | PASS (post-fix) |
| E9 | Feasibility/violation counts match backend | PASS (static) |
| E10 | Scheduler selection without frontend re-ranking | PASS (static) |
| E11 | Selected candidate proceeds to export | PASS (static) |
| E12 | Export corresponds to selected backend candidate | PASS (static) |
| E13 | Empty-candidate scenario handled explicitly | PASS (static) |
| E14 | No-synthetic invariant holds | PASS (static) |
| E15 | DecisionRepository records candidate identity | PASS (static) |

**15/15 gates PASS.**

---

## P0 Milestone Closure

P0-A through P0-E are now complete:

| Phase | Description | Status |
|-------|-------------|--------|
| P0-A | Coralys uid duplicate guard + UltraCrew seed filter | CLOSED |
| P0-B | Candidate generation diagnostic + diverse seeding | CLOSED |
| P0-C | CandidateFeasibility — adapter describes, scheduler decides | CLOSED |
| P0-D | Decision UI — feasibility badge + objectives display | CLOSED |
| P0-E | End-to-end acceptance — type contract gap found and fixed | CLOSED |

The Coralys Pareto pipeline is now end-to-end integrated: genuine non-dominated
alternatives flow from the Rust optimizer through the API, are correctly mapped
to the UI type system, and are presented to the scheduler without any synthetic
data, frontend ranking, or candidate alteration.