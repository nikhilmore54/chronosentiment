# GOV-003 — Benchmark Lifecycle
## Version 1.0 — Frozen 2026-07-13

This document governs how benchmarks are created, evolved, frozen, and retired
in the Coralys programme. It changes only through deliberate governance
decisions. Sprint reports reference this document rather than restating
benchmark rules.

---

## Benchmark Creation Rule

A new benchmark instance is introduced only when an existing benchmark cannot
answer a product or platform engineering question.

Benchmarks are instruments for answering engineering questions. They are not
destinations or roadmap milestones in their own right.

Before creating a new benchmark, the following must be true:
1. A product or platform engineering question exists.
2. The question cannot be answered using any existing benchmark.
3. A measurable hypothesis has been written.
4. The new benchmark differs from the nearest existing benchmark by the minimum
   number of variable changes necessary to isolate the question.

---

## Benchmark States

```
Draft
  ↓
Candidate
  ↓
Frozen
  ↓
Regression
  ↓
Archived
```

**Draft:** Under construction. Not used for measurement.

**Candidate:** Design complete. Under review before first measurement.

**Frozen:** Immutable. Used for measurement and comparison. Version-locked.

**Regression:** A frozen benchmark promoted to permanent regression status.
Used in every sprint to verify correctness. Never modified.

**Archived:** Retired. No longer used for active measurement. Preserved for
historical reference.

---

## Required Contents

Every benchmark must contain:

- Purpose: what engineering question it answers
- Assumptions: what is held constant
- Dataset: staff, shifts, coverage, constraints
- Success metric: what a good result looks like
- Hypothesis: what is being tested
- Acceptance criterion: what result confirms or refutes the hypothesis

---

## Versioning Rule

Every benchmark version is immutable once frozen.

Correct versioning:
```
UB-002-v1.0   (frozen, immutable)
  ↓
UB-002-v1.1   (new version if design change required)
```

Never overwrite a frozen version. Never modify a frozen benchmark in place.

---

## Single Variable Change Rule

Each new benchmark version changes exactly one variable from the nearest
existing benchmark. This isolates the experimental variable and prevents
confounds.

Example: UB-002-v1.0 differs from UB-001-v1.0 by exactly one field:
`historical_workloads` (null → non-null). All other fields are identical.

---

## Active Benchmarks

| Benchmark | State | Purpose |
|---|---|---|
| UB-001-v1.0 | Regression | Canonical correctness and optimality baseline. 20 workers, 4 weeks, SC2=0. Optimum=9918.4 (proven). |
| UB-002-v1.0 | Frozen / Experimental | First ecological benchmark. UB-001 + historical_workloads (SC2>0). Sprint 8 characterization target. |

---

## UB-001 Regression Contract

UB-001-v1.0 is the permanent regression benchmark. Every platform change must
pass this contract before being committed.

| Requirement | Value | Type |
|---|---|---|
| SC2 | 0.0 (all weeks, all seeds) | HARD |
| HC1 = HC2 = HC3 = Rest | 0 (all weeks) | HARD |
| SC1 | 81.6 when optimizer converges | SOFT (seed-dependent) |
| Fitness | 9918.4 when SC1 = 81.6 | DERIVED |

---

## Future Benchmarks

UB-003 and beyond are created only when:
- Stream B (UltraCrew) surfaces a scheduling problem that UB-001 and UB-002
  cannot explain, OR
- A platform engineering question requires a benchmark that does not yet exist.

The benchmark roadmap does not drive benchmark creation. Product evidence does.

---

## References

- [`GOV-001-PROGRAMME-GOVERNANCE-v1.0.md`](GOV-001-PROGRAMME-GOVERNANCE-v1.0.md) — programme governance
- [`GOV-002-RESEARCH-METHODOLOGY-v1.0.md`](GOV-002-RESEARCH-METHODOLOGY-v1.0.md) — research methodology and freeze rules