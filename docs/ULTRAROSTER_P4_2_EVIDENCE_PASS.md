# P4.2 — Bounded Evidence Pass
## Can existing `SchedulerDecision` fields support a defensible recurring preference signal?

**Baseline:** `governance-hardening @ 5d195c5b7`
**Status:** Evidence pass complete — see Finding below.
**No code changes were made during this investigation.**

---

## 1. The Question

> Can `overrode_recommendation`, `recommended_id`, and `selected_id` alone reveal a defensible recurring preference signal?

---

## 2. What the data model contains

[`SchedulerDecision`](../ui/ultracrew/src/workflow/WorkflowTypes.ts) at the P4.1 baseline:

```typescript
export interface SchedulerDecision {
  decision_id: string;
  created_at_iso: string;
  recommended_id: string;   // which alternative the optimizer recommended
  selected_id: string;      // which alternative the scheduler chose
  overrode_recommendation: boolean;  // selectedId !== recommendedId
}
```

[`DecisionRepository.recordSchedulerDecision()`](../ui/ultracrew/src/services/DecisionRepository.ts) constructs this record at the moment the scheduler confirms their choice in step 4 (Explore Decision). The record is appended to `ultracrew_scheduler_decisions` in localStorage and is never mutated.

The alternative IDs (`recommended_id`, `selected_id`) are opaque strings — they identify which roster alternative was chosen, but carry no information about *why* that alternative was preferred.

---

## 3. What a "recurring preference signal" would require

For a recurring preference signal to be defensible, it must satisfy three criteria:

**C1 — Repeatability:** The same preference must appear across multiple independent decisions (distinct `decision_id` values), not just once.

**C2 — Interpretability:** The signal must be interpretable in terms of *what* the scheduler preferred — not merely *that* they overrode. "Scheduler overrode 7 times" is a count, not a signal. "Scheduler consistently chose the alternative with higher coverage" is a signal.

**C3 — Actionability:** The signal must be specific enough to inform a future system response — e.g., adjusting a weight, surfacing a recommendation, or flagging a pattern.

---

## 4. Evidence assessment

### 4a. `overrode_recommendation` (boolean)

This field tells us whether the scheduler deviated from the optimizer's recommendation. Across a history of decisions, it yields an override rate.

**What it can support:** A count of overrides. "Scheduler has overridden the optimizer's recommendation N times out of M decisions."

**What it cannot support:** Any explanation of *why* the override occurred. Two overrides may have completely different causes — one because the scheduler preferred higher coverage, another because of a staffing constraint the optimizer didn't model. The boolean is silent on this.

**Verdict:** Supports C1 (repeatability of override behavior) but fails C2 (interpretability) and C3 (actionability). An override rate alone is not a defensible preference signal — it is a behavioral frequency with no semantic content.

### 4b. `recommended_id` and `selected_id` (opaque strings)

These fields record which alternative was recommended and which was selected. Across a history, they allow us to compute:

- How often the scheduler chose alt-A vs alt-B vs alt-C
- Whether the scheduler consistently avoided the recommended alternative
- Whether the scheduler consistently chose the same positional slot (e.g., always the second alternative)

**What it can support:** Frequency counts of which alternative IDs were selected. If the same alternative ID recurs across sessions (which it will not — IDs are generated per session), a frequency signal could be constructed.

**Critical structural problem:** Alternative IDs are session-scoped. [`buildSyntheticAlternatives()`](../ui/ultracrew/src/workflow/WorkflowUtils.ts) generates IDs like `alt-A`, `alt-B`, `alt-C` per generation run. The API path generates IDs from the optimizer response. There is no stable cross-session identity for alternatives. "Scheduler selected `alt-B` in session 1 and `alt-B` in session 2" does not mean the scheduler selected the same *kind* of alternative — `alt-B` in session 2 is a completely different roster with different metrics.

**Verdict:** `recommended_id` and `selected_id` cannot be compared across sessions. They fail C1 (cross-session repeatability is structurally impossible with opaque session-scoped IDs), C2 (no semantic content), and C3 (no actionable pattern can be derived from comparing IDs that have no stable meaning).

### 4c. The combination: override + which alternative

Even combining all three fields, the best achievable signal is:

> "The scheduler overrode the recommendation and selected a non-recommended alternative."

This is equivalent to `overrode_recommendation = true`. The identity of the selected alternative adds nothing without knowing what that alternative's metrics were at the time of selection.

---

## 5. The metrics gap

The missing information is precisely what was identified in the P4 evidence pass (committed `5f321a187`):

```text
SchedulerDecision (current)          What would be needed
─────────────────────────────        ──────────────────────────────────────
overrode_recommendation: bool        ✓ present
recommended_id: string               ✓ present (but opaque)
selected_id: string                  ✓ present (but opaque)
                                     ✗ recommended_metrics: absent
                                     ✗ selected_metrics: absent
```

Without `recommended_metrics` and `selected_metrics` captured at decision time, there is no way to answer: "What did the scheduler consistently prefer — higher coverage? lower cost? more fairness?" The alternative IDs are meaningless without the metrics they carried at the moment of selection.

---

## 6. Finding

**Result: B — Insufficient signal.**

The three existing fields (`overrode_recommendation`, `recommended_id`, `selected_id`) cannot alone support a defensible recurring preference signal. Specifically:

- `overrode_recommendation` yields a frequency count with no semantic content.
- `recommended_id` and `selected_id` are session-scoped opaque identifiers that cannot be compared across sessions.
- No combination of the three fields can establish *what* the scheduler preferred, only *that* they deviated.

The evidence gap is structural, not a matter of insufficient history volume. Even with 1000 decisions in the log, the same conclusion holds: the fields record the fact of a choice but not the basis of the choice.

---

## 7. Authorization decision

**P4.2 implementation is NOT authorized on the basis of the existing three fields alone.**

The gap must be closed first. The minimum closure is:

- Capture `recommended_metrics` (the metrics of the optimizer-recommended alternative at decision time)
- Capture `selected_metrics` (the metrics of the scheduler-selected alternative at decision time)

These would be added to `SchedulerDecision` at the moment [`recordSchedulerDecision()`](../ui/ultracrew/src/services/DecisionRepository.ts:82) is called, sourced from the `RosterAlternative.metrics` of the relevant alternatives.

**This closure is a separate authorization decision.** It requires:

1. Confirming that `RosterAlternative.metrics` is available at the point `recordSchedulerDecision()` is called (it is — `SelectDecision` has both alternatives in scope).
2. Deciding whether the metrics gap closure is the right next step, or whether the roadmap should stop at P4.1.
3. Explicitly authorizing the field addition before any code is written.

---

## 8. What this evidence pass does NOT authorize

- Adding `recommended_metrics` or `selected_metrics` to `SchedulerDecision`
- Modifying `recordSchedulerDecision()` or `SelectDecision`
- Any optimizer changes
- Any Coralys changes
- Any adaptive weights or objectives
- Any learning mechanism
- Any modification to P4.1

---

## 9. Stopping condition

This evidence pass is complete. The repository remains frozen at `governance-hardening @ 5d195c5b7`. No code was changed.

The next decision point is: **authorize the metrics gap closure, or close the P4 roadmap at P4.1.**