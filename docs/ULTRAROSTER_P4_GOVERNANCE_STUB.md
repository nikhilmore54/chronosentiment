# UltraRoster P4 — Learn From Decisions
## Governance Stub — Definition Required Before Implementation

**Status:** NOT ACTIVE — definition phase required before any implementation
**Prerequisite:** P3 complete and frozen at `be67e4b59` (branch: `governance-hardening`)
**Author:** UltraRoster product governance
**Date:** 2026-08-31

---

## Phase Purpose

P4 makes UltraRoster learn from the decisions and modifications that P3 now reliably records.

P3 produces the evidence substrate P4 needs:
- `SchedulerDecision` — what the optimizer recommended, what the scheduler selected, whether they overrode
- `RedistributionLog` — what the system changed during controlled redistribution
- `ChangeRecord` stream — per-cell provenance with reason, previous/new value, operation ID, timestamp

P4 must not begin implementation until the definition questions below are answered.

---

## Definition Questions (Required Before Implementation)

### 1. What constitutes a decision?

- Optimizer recommendation
- Scheduler selection / override
- Scheduler edit (manual lock)
- Subsequent redistribution result

### 2. What constitutes a learning signal?

- What the scheduler changed from the optimizer recommendation
- What the system changed in response to scheduler locks
- Which recommendations were accepted vs. overridden
- Which patterns of intervention recur across sessions

### 3. What is P4 allowed to influence?

Candidates (must be explicitly authorized before implementation):
- Objective preferences / weights
- Alternative selection ordering
- Constraint priorities
- Sequence preferences
- Something else

### 4. What P4 must never learn or override

Hard boundaries — not subject to learning:
- Hard constraints (HC1 minimum coverage, sequence feasibility)
- Eligibility rules
- Safety / legal feasibility
- Scheduler locks (P3.3 invariant)
- Domain invariants defined in the UltraCrew adapter

---

## Implementation Principle

**Do not assume "learning from decisions" means machine learning immediately.**

The first P4 question is:

> What useful, defensible information can be extracted from the decisions P3 has recorded?

Only after answering that should the mechanism be chosen — whether statistical learning, preference inference, adaptive weighting, pattern detection, or something simpler.

---

## P4 Scope Gate (applies when definition is complete)

Same three-part gate as P3.3:

1. Is it required for the authorized P4 behavior to function correctly?
2. Is it a defect in an already-defined P4 invariant?
3. Can it be fixed without changing the product's intended behavior or architecture?

All three YES → minimal correction permitted. Otherwise → record and defer.

---

## What P4 Must Not Do

- Reopen P3.2 or P3.3 frozen boundaries
- Introduce a second optimization authority
- Override hard constraints through learned preferences
- Modify `coralys_moga` with domain-specific logic
- Implement rest-pattern quality (separate domain objective, separately authorized)

---

## Phase Progression

```
P3.2  CLOSED — Decide
P3.3  FROZEN — Control
P4    NEXT / NOT ACTIVE — Learn
  • definition phase required first
  • implementation only after definition is authorized
```

---

*This stub is frozen until P4 definition is explicitly authorized.*