# UltraCrew Sprint 2 — Coralys Integration
**Milestone:** Coralys Integration
**Frozen:** 2026-07-12
**Branch:** governance-hardening
**Commits:** cfaab447, 7ebeb6ab, 83346dd9

---

## What was claimed

> Replace the synthetic scheduler with the real Coralys MOGA pipeline and establish the PAS measurement infrastructure.

## What was delivered

### 1. Real Coralys MOGA pipeline wired (cfaab447)

`ui/ultracrew/src/workflow/WorkflowUtils.ts` — `buildSchedulePayload()` sends a valid `ScheduleRequest` to `POST /api/schedule` on the live `ultracrew_server`. Shift slots are within the 0–167 hour weekly model required by the constraint engine. Rule preset keys stripped (backend does not accept them). `buildEditableSchedule()` inverts the `shift_id → worker_id` response into a 28-day grid by expanding the weekly pattern 4×.

### 2. Zero hard and rest violations (7ebeb6ab)

Reduced `WEEKLY_SHIFT_SLOTS` from 10 (2 per day) to 5 (1 per day, alternating Early/Late). Minimum gap between consecutive slots is 8h, satisfying the constraint engine's rest period check.

Smoke test (50 generations, 3 workers, 5 shifts):

| Constraint | Result |
|---|---|
| HC1 (skill match) | 0 violations |
| HC2 (double booking) | 0 violations |
| HC3 (max hours) | 0 violations |
| Rest (8h gap) | 0 violations |
| Valid | True |
| Fitness | 9857.8 |
| Runtime | ~144ms |

### 3. Planner Acceptance Score measurement (83346dd9)

`ExportRoster.tsx` displays the primary product KPI:

```
PAS = (generated assignments − manual edits) / generated assignments × 100%
```

Edit count flows from `ReviewSchedule` → `PlannerWorkflow` → `ExportRoster`. Colour-coded: ≥ 95% green, ≥ 80% amber, < 80% red.

---

## What was NOT claimed

Sprint 2 does not claim planner-quality scheduling. The PAS infrastructure is in place, but PAS has not been measured under realistic workloads. The target of ≥ 95% PAS is Sprint 3's acceptance criterion.

---

## PAS Baseline Table

This table will be updated after each optimizer improvement. Sprint 2 establishes the measurement infrastructure; the first real PAS measurement will be recorded in Sprint 3.

| Build | PAS | Manual edits | Hard violations | Rest violations | Runtime |
|---|---|---|---|---|---|
| Sprint 2 (baseline) | TBD — first real measurement in Sprint 3 | — | 0 | 0 | ~144ms |

---

## Sprint 3 Definition

**Milestone name:** Planner-Quality Schedule Generation

**Objective:** Achieve PAS ≥ 95% on a realistic workforce dataset (≥ 10 staff, 28 days, mixed skills).

**Definition of Done:**

| KPI | Target |
|---|---|
| PAS | ≥ 95% |
| Hard violations | 0 |
| Rest violations | 0 |
| Coverage | 100% of required shifts |
| Runtime | < 5 seconds for 10–30 staff |

**Engineering location:** `adapters/ultracrew/src/optimization.rs` — not the UI.

**Engineering priority (in order):**

1. Better initialization — seed population with feasible schedules rather than random assignments
2. Repair operators — eliminate any remaining hard violations before soft objective optimization
3. Local search — improve fairness, workload balance, preferences
4. Planner-aware objectives — weight assignments that historically required manual edits more heavily

**Acceptance criterion:** Run the full workflow with a real CSV import (≥ 10 staff), generate a schedule, make zero manual edits, and observe PAS ≥ 95% on the Export screen.

**What Sprint 3 does not include:**

- UI changes (unless needed to expose new optimizer capability)
- New workflow steps
- New report formats
- Architecture changes

---

## Platform feedback loop

Every optimizer improvement in Sprint 3 is evaluated against two metrics simultaneously:

| Platform metric | Product metric |
|---|---|
| Objective value | PAS |
| Runtime | Planner edits |
| Constraint violations | Coverage |
| Benchmark score | Planner acceptance |

This is the healthy feedback loop between Coralys (platform) and UltraCrew (product).