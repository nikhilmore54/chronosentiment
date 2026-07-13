# Sprint 9 Plan
**Document:** S9-SPRINT-PLAN-v1.0.md
**Date:** 2025-07-13
**Status:** DRAFT
**Preceding sprint:** S8 (bd7bd825, 7ae81f75)

---

## Context

Sprint 8 completed the "prove the platform" phase. The research programme is now disciplined
rather than exploratory. Constitutional governance (GOV-001/002/003) is in place. The primary
value generator from this point forward is Stream B (UltraCrew product), with platform research
proceeding only when product evidence exposes limitations.

---

## Sprint 9 Stream Allocation

| Stream | Weight | Focus |
|---|---|---|
| Stream B — UltraCrew product | ~70% | CVD-001 import pipeline, airline domain adapter, PAS on realistic data |
| Stream A — Platform research | ~15% | H9 only, if it directly supports a product question |
| Stream C — Research Station | ~10% | Publish S8 findings, update archive |
| Stream D — ROADEF campaign | independent | Continue separately, no cross-dependency |

---

## Stream B — Primary deliverables

### B1: CVD-001 Import Pipeline (highest priority)

Build an importer for the Canadian airline dataset (CVD-001) into UltraCrew's internal
workforce model. This is the first Customer Validation Dataset — realistic operational data
from a major North American airline (anonymized).

Deliverables:
- `scripts/cvd001_adapter.py` — parse instance files into Coralys API payload format
- `benchmarks/customer_validation/CVD-001-v1.0.json` — frozen first instance
- Run Coralys against CVD-001, record PAS, fitness, HC violations
- Document in `docs/CVD-001-IMPORT-v1.0.md`

Governance check: CVD-001 is a product validation dataset, not a benchmark. It answers
"does UltraCrew work on realistic airline data?" not "what is the optimizer's theoretical
performance?" These are different questions.

### B2: Airline Domain Adapter in UltraCrew UI

Extend the UltraCrew planner workflow to accept airline crew scheduling data:
- Duty/pairing import (CSV or JSON)
- Crew base and qualification display
- Airline-specific shift types (short-haul, long-haul, standby)
- PAS measurement on CVD-001 data

### B3: Planner Explanation Improvements (continued)

Following S8 Stream B explanation quality work:
- Per-worker fatigue history panel (SC2 contribution per worker)
- Constraint violation drill-down (which specific constraint, which day)
- "Why not?" modal for rest days (what constraint prevented assignment)

---

## Stream A — H9 (conditional)

**H9 candidate:** Is the observed H7 ordering instability caused primarily by objective
indifference (multiple equal-cost schedules) or by insufficient SC2 discrimination between
neighbouring fatigue groups?

**Trigger condition:** Only execute H9 if CVD-001 data reveals ordering instability in
realistic airline crew assignments. Do not run H9 as pure benchmark curiosity.

**Protocol if triggered:** 3 workload profiles × 10 seeds × 100 generations = 30 API calls.
Classify failures by type (HIGH==MED tie vs MED==LOW tie) as established in H7.

---

## Stream C — Research Station

- Publish S8 characterization findings in Research Station archive
- Update benchmark registry with H6/H7/H8 result JSONs
- Add CVD taxonomy to GOV-003 (UB / CVD / Pilot Archive)
- Draft external benchmark integration plan (SchedulingBenchmarks.org NRP instances)
  as future SB-001 candidate — do not implement yet

---

## Evidence Taxonomy (updated)

| Category | Purpose | Examples |
|---|---|---|
| UB (Universal Benchmark) | Controlled scientific experiments | UB-001, UB-002 |
| CVD (Customer Validation Dataset) | Real operational validation | CVD-001 Canadian Airline |
| SB (Scheduling Benchmarks) | External public benchmarks | SB-001 (future, NRP) |
| OB (OR-Library) | Cross-domain generalization | OB-001 (future) |
| Pilot Archive | Customer-specific deployments | Customer A, B (future) |

---

## Governance Constraints

Per GOV-001: no new UB benchmark (UB-003) unless a specific product question requires it
and cannot be answered by existing benchmarks. CVD-001 is not a benchmark — it is a
validation dataset. These are different evidence classes.

Per GOV-002: H9 is only executed if triggered by product evidence. It is not scheduled
as a standalone research task.

Per GOV-003: CVD-001 lifecycle follows the Customer Validation Dataset track, not the
UB benchmark lifecycle. It is not subject to the same freeze/regression protocol as UB-001/002.

---

## Sprint 9 Entry Criteria

- [x] S8 Stream A frozen and committed (bd7bd825)
- [x] S8 Stream B shipped and committed (7ae81f75)
- [x] CVD taxonomy established
- [x] H9 candidate formulated
- [ ] CVD-001 dataset located and schema documented
- [ ] Sprint 9 formally opened