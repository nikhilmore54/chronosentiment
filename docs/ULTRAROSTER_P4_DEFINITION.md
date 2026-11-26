# UltraRoster P4 — Definition Document
## P4 Definition Gate — Evidence-Based Analysis

**Baseline:** `governance-hardening @ 2e4d2c720`
**Scope:** Inspection of existing P3 structures only. No code changes. No implementation.
**Date:** 2026-08-31

---

## Methodology

This document answers the four P4 definition questions strictly from the existing P3 implementation. No new data collection, no new structures, and no implementation proposals beyond the minimum mechanism identified by the evidence.

Sources inspected:
- `ui/ultracrew/src/workflow/WorkflowTypes.ts` — P3 data structures
- `ui/ultracrew/src/workflow/WorkflowUtils.ts` — redistribution and comparison logic
- `ui/ultracrew/src/workflow/ReviewSchedule.tsx` — provenance rendering
- `docs/ULTRAROSTER_P3_GOVERNANCE_FROZEN.md` — P3.2 authority model
- `docs/ULTRAROSTER_P3_3_GOVERNANCE.md` — P3.3 invariants

---

## Question 1: What signal do we already have?

The following signals are produced by the existing P3 structures per decision session. No new data collection is required to observe them.

### 1a. Recommendation acceptance / override

Source: `SchedulerDecision` (WorkflowTypes.ts:96)

```typescript
interface SchedulerDecision {
  decision_id: string;
  created_at_iso: string;
  recommended_id: string;   // optimizer's recommendation
  selected_id: string;      // scheduler's actual selection
  overrode_recommendation: boolean;
}
```

Observable: did the scheduler accept or override the optimizer recommendation? If override, which alternative did they select instead?

### 1b. Alternative metrics at decision time

Source: `RosterAlternativeMetrics` (WorkflowTypes.ts:44)

```typescript
interface RosterAlternativeMetrics {
  coverage: number;           // 0.0–1.0
  filled_positions: number;
  required_positions: number;
  fairness_penalty: number;
  utilization: number;        // 0.0–1.0
  cost: number;
  diff_from_recommended: number;
}
```

Observable: when the scheduler overrides, what were the metric differences between the recommended alternative and the selected alternative? (e.g., scheduler consistently selects lower fairness_penalty over higher coverage)

### 1c. Scheduler edit count and locked cells

Source: `RedistributionLog` (WorkflowTypes.ts:83)

```typescript
interface RedistributionLog {
  schedulerEditsPreserved: number;   // how many cells the scheduler locked
  assignmentsReassigned: number;     // how many cells the system changed
  lockedAssignmentsChanged: number;  // invariant: always 0
  changeRecords: ChangeRecord[];
  provenanceMap: Record<string, AssignmentProvenanceState>;
}
```

Observable: how many manual edits did the scheduler make before redistribution? Which cells?

### 1d. Per-cell change records

Source: `ChangeRecord` (WorkflowTypes.ts:74)

```typescript
interface ChangeRecord {
  assignmentId: string;              // `${staffId}:${dayIdx}`
  previousValue: string;             // shift before redistribution
  newValue: string;                  // shift after redistribution
  redistributionOperationId: string;
  reason: string;                    // e.g. "coverage gap on day 14 Night shift"
  timestamp: string;
}
```

Observable: what specific assignments did the system change, and why? Recurring `reason` values indicate recurring coverage patterns.

### 1e. Full provenance map

Source: `provenanceMap` in `RedistributionLog`

Observable: for every `staffId:dayIdx` cell, its state: `original` | `scheduler_edit` | `system_reassignment` | `unchanged`. This gives a complete picture of what the scheduler touched vs. what the system touched.

### 1f. Post-redistribution metrics

Source: `computeCanonicalCoverage()` called after redistribution in `ReviewSchedule.tsx`

Observable: coverage, filled positions, required positions, gap positions after redistribution.

### Gap identified

**Redistribution logs are currently in-memory only.** `RedistributionLog` is held in React state (`redistResult`) and is not persisted to P2 memory. `SchedulerDecision` is persisted (P2), but the redistribution log that accompanies it is not.

This means pattern detection across multiple sessions is not currently possible. The signal exists within a session but is lost when the session ends.

---

## Question 2: What could that signal legitimately influence?

For each signal, the possible influence and hard boundaries:

| Signal | Possible influence | Must NOT influence |
|---|---|---|
| Repeated override of optimizer recommendation in favor of lower fairness_penalty | Future soft objective weight for fairness (via adapter problem definition) | HC1 minimum coverage, eligibility |
| Repeated override in favor of higher coverage (beyond HC1) | Future soft objective weight for coverage above minimum | Hard constraint floor |
| Repeated scheduler edits to specific shift types on specific days | Soft preference signal for shift-type distribution | Sequence feasibility, eligibility |
| Recurring redistribution reason (e.g., "coverage gap on Night shift") | Adapter awareness of recurring coverage weakness | HC1 minimum (already enforced) |
| Consistent acceptance of optimizer recommendation (no override) | Confidence signal — optimizer is well-calibrated for this scheduler | Nothing — no change needed |

**Classification of influence type:**

All legitimate influences are **soft objective adjustments via the UltraCrew adapter problem definition**. None of them touch Coralys directly. None of them override hard constraints. None of them remove scheduler authority.

The adapter already controls soft objective weights. P4's influence, if authorized, would be: the adapter reads recorded preferences and adjusts soft weights before constructing the next optimization problem.

---

## Question 3: Acceptance gates

Before any signal may influence future optimization, all of the following must be satisfied:

| Gate | Requirement |
|---|---|
| G-P4-1 | Signal is observable in existing P3 history — no new data collection required |
| G-P4-2 | Signal repeats across at least 3 independent decision sessions (proposed threshold — subject to authorization) |
| G-P4-3 | Signal is distinguishable from one-off intervention (count ≥ threshold, not a single outlier) |
| G-P4-4 | Applying the learned preference does not violate hard constraints (HC1, sequence feasibility, eligibility) — verified by existing regression tests |
| G-P4-5 | Learned influence produces a measurable improvement in the relevant metric (coverage, fairness, or cost) |
| G-P4-6 | Scheduler can inspect and understand the influence — UI must expose what preference was recorded and why |
| G-P4-7 | Scheduler authority is preserved — scheduler can always override and clear recorded preferences |
| G-P4-8 | Behavior without sufficient evidence (< 3 sessions) is identical to current P3 behavior — no change |

These gates are proposed, not yet authorized. Authorization requires explicit approval before implementation.

---

## Question 4: Smallest plausible mechanism

**Mechanism: Explicit preference recording from override patterns**

This is not ML. It is preference inference from explicit behavioral evidence.

```
Session 1: Scheduler overrides recommendation → selects alt with lower fairness_penalty
Session 2: Scheduler overrides recommendation → selects alt with lower fairness_penalty
Session 3: Scheduler overrides recommendation → selects alt with lower fairness_penalty
                    ↓
        Count ≥ threshold (3)
                    ↓
        Record: scheduler prefers lower fairness_penalty
                    ↓
        Next optimization: UltraCrew adapter increases fairness weight
        in the soft objective definition passed to coralys_moga
                    ↓
        Optimizer produces candidates with lower fairness_penalty
                    ↓
        Scheduler inspects, accepts or clears preference
```

**What this requires:**
1. Persist `RedistributionLog` to P2 memory alongside `SchedulerDecision` (currently missing — this is the gap)
2. A preference accumulator that counts override patterns across sessions
3. A preference store (simple key-value: characteristic → count)
4. Adapter reads preference store and adjusts soft weights before constructing the next problem
5. UI exposes recorded preferences and allows scheduler to clear them

**What this does NOT require:**
- ML model
- Adaptive weight algorithm
- New optimizer objectives
- Coralys changes
- New hard constraints
- Automatic constraint learning

**Mechanism ordering (from simplest to most complex):**

```
1. Explicit preference recording (proposed above) ← smallest
2. Statistical preference (weighted by recency)
3. Adaptive parameter (continuous adjustment)
4. Learning model (ML)
```

P4 should start at level 1 and not advance to level 2+ without a separately authorized reason.

---

## Conclusion

**Insufficient evidence for immediate implementation — one gap exists.**

The signal is real and observable within a session. The mechanism is bounded and does not require ML. However, the redistribution log is not currently persisted across sessions, which means the pattern detection required for G-P4-2 (signal repeats across ≥ 3 sessions) cannot be satisfied with the current implementation.

**Before P4 implementation can be authorized, one prerequisite must be resolved:**

> **Persist `RedistributionLog` to P2 memory alongside `SchedulerDecision`.**

This is a P3.3 data-completeness gap, not a new P4 feature. It should be evaluated against the P3.3 scope gate before being authorized as a correction.

Once that gap is resolved, the smallest P4 mechanism (explicit preference recording from override patterns) can be implemented against the acceptance gates defined in Question 3.

**P4 implementation should not yet be authorized.**

---

## What is explicitly out of scope for P4 (even after authorization)

- ML models, neural networks, statistical learning algorithms
- Automatic constraint modification
- Coralys changes
- New hard constraints derived from history
- Rewriting P3 provenance records
- Removing scheduler authority
- Rest Pattern Quality (separate domain objective, separately authorized)
- Multi-session redistribution optimization
- Competitor analysis or alternative optimizer research