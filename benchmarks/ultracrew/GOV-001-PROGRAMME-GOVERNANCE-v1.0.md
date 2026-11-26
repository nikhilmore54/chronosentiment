# GOV-001 — Programme Governance
## Version 1.0 — Frozen 2026-07-13

This document defines the permanent governance model for the ChronoSentiment
programme. It changes only when the programme itself evolves, not sprint by
sprint. Sprint reports reference this document rather than restating governance
decisions.

---

## Phase I — Platform Foundation (COMPLETE)

Phase I ended when the optimizer was proven correct and capable.

| Sprint | Outcome |
|---|---|
| M19/M20 | Coralys MOGA core |
| Sprint 1 | UltraCrew workflow foundation |
| Sprint 2 | Coralys integrated into UltraCrew |
| Sprint 3 | Skill-aware initialization |
| Sprint 4 | Constraint-aware initialization |
| Sprint 5 | Bottleneck identification (diversity/runtime eliminated) |
| Sprint 6 | Analytical optimality proof for UB-001 |
| Sprint 7 | Ecological objective (SC2) introduced; governance model finalized |

---

## Phase II — Adoption (ACTIVE)

Phase II ends when planners prefer using UltraCrew.

Phase I success criterion: "The optimizer works."
Phase II success criterion: "Planners prefer using UltraCrew."

These are different milestones. One is an engineering achievement. The other
is product validation.

---

## Programme Pillars

| Pillar | Purpose |
|---|---|
| Coralys Platform | Optimization engine and scientific foundation |
| UltraCrew | Commercial workforce scheduling product |
| Research Station | Evidence, methodology, and benchmark archive |
| ROADEF Campaign | Independent external validation of platform capability |

Each pillar has a clear role. None competes with the others.

---

## Stream Allocation (Phase II)

| Stream | Allocation | Focus |
|---|---|---|
| A — Coralys Platform | ~20–25% | Characterization and maintenance |
| B — UltraCrew | ~50–60% | Product execution (primary) |
| C — Research Station | ~10–15% | Evidence and methodology |
| D — ROADEF | Independent | External validation |

---

## Platform Governance Rules

Two categories of Coralys work are distinguished permanently.

### Platform Maintenance
Preserves correctness. May happen at any time.

Examples: regression fixes, performance improvements, bug fixes, observability,
documentation.

### Platform Research
Changes optimization behaviour. May only begin after all four conditions are met:

1. A product question exists.
2. Existing benchmarks cannot answer it.
3. A benchmark is frozen.
4. A measurable hypothesis is written.

Examples: new objectives, new operators, new ecology models, new benchmarks.

---

## Benchmark Creation Rule

A new benchmark instance is introduced only when an existing benchmark cannot
answer a product or platform engineering question.

Benchmarks are instruments for answering engineering questions. They are not
destinations or roadmap milestones in their own right.

---

## Direction of Information

```
UltraCrew
      ↓
Planner evidence
      ↓
Research question
      ↓
Benchmark
      ↓
Coralys improvement
      ↓
UltraCrew
```

Every Coralys enhancement traces back to a product question. Platform Research
that cannot be traced to a product question is deferred.

---

## Phase II Sprint Milestones

| Sprint | Primary outcome | Lead KPI |
|---|---|---|
| 8 | Characterize UB-002 + integrate product datasets | SC1/SC2 stability, PAS correlation |
| 9 | Planner trust | Planner Acceptance Score, planner editing time |
| 10 | Publishable operations | Successful publish workflow, zero hard violations |
| 11 | Pilot readiness | Pilot success metrics and planner feedback |
| 12 | Product evidence review | Decide whether existing benchmarks suffice or UB-003 is justified |

---

## References

- See [`GOV-002-RESEARCH-METHODOLOGY-v1.0.md`](GOV-002-RESEARCH-METHODOLOGY-v1.0.md) for hypothesis-driven research process
- See [`GOV-003-BENCHMARK-LIFECYCLE-v1.0.md`](GOV-003-BENCHMARK-LIFECYCLE-v1.0.md) for benchmark creation and retirement rules