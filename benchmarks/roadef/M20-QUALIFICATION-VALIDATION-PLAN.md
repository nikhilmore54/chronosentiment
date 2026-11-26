# M20 — Qualification Validation Plan

**Document ID:** M20-PLAN-v1.0  
**Status:** FROZEN — Pre-experiment protocol  
**Date:** 2026-07-09  
**Milestone:** M20 — Cross-Domain Qualification Validation  
**Depends on:** M19A complete, M19B campaign evidence

---

## Objective

Determine whether Qualification v1.0 is domain-independent without modification.

This is not:
> Make ROADEF fit Qualification.

This is:
> Let the evidence determine which abstractions generalize.

Both outcomes — full generalization and partial failure — are valid engineering results
provided they are supported by empirical evidence rather than assumption.

---

## Success Metric

The objective of M20 is **evidence generation**, not platform promotion.

The experiment is successful if it conclusively determines whether
Qualification v1.0 generalizes across domains.

Both confirmation and falsification satisfy the experiment objective,
provided the conclusion is supported by reproducible evidence.

> Qualification failed ≠ the experiment failed.

Those are different things. A falsification result is more valuable than a forced
mapping that obscures the real abstraction boundary.

---

## Hypothesis

The frozen Qualification Subsystem v1.0 API (`CertificateInput → ExecutionCertificate`)
can be consumed by a ROADEF adapter without modification to any frozen component.

---

## Falsification Criterion

Any required modification to the following constitutes falsification:

- `FeasibilityCertificate` struct or its fields
- `FleetSemanticCheck` struct or its fields
- `FleetUtilizationCertificate` struct or its fields
- `CertificateInput` struct or its fields
- `ExecutionCertificate` struct or its fields
- `CertificateStatus` priority ordering
- `qualification/mod.rs` public API

Modifications to ROADEF adapter code only (within `adapters/roadef/`) are permitted
and expected — that is the adapter's job.

---

## Pre-Experiment Predictions

The following predictions are recorded before any M20 code is written.
They will be compared against evidence after M20 completes.

| Component | Prediction | Confidence | Reasoning |
|-----------|------------|------------|-----------|
| `FeasibilityCertificate` | Reusable | High | Every optimization domain has structural validity, constraint satisfaction, and evaluation invariants. The concept is domain-independent. |
| `FleetSemanticCheck` | Partial failure | High | Designed around `routes_used ≤ K`. ROADEF has no fleet concept. The analogous concept (SR-path budget compliance) is structurally different. |
| `FleetUtilizationCertificate` | Reusable | Medium | Fields describe resource utilization distribution. For ROADEF: link loads instead of vehicle loads. Mathematics identical; name is CVRP-specific. |
| `ExecutionCertificate` | Reusable | Very High | Identity, Objective, Runtime, Status, Governance, Hash — none are CVRP concepts. |

### Architectural prediction (recorded before implementation)

Based on M19A evidence (MOGA engine reused unchanged for ROADEF):

| Layer | Expected Result |
|-------|----------------|
| MOGA Evolution Engine | **Already validated** (M19A) |
| FeasibilityCertificate | **Pass** |
| FleetUtilizationCertificate | **Pass** |
| ExecutionCertificate | **Pass** |
| FleetSemanticCheck | **Likely partial failure** |

---

## ROADEF Domain Mapping

### FeasibilityCertificate mapping

| FCF Check | CVRP concept | ROADEF equivalent |
|-----------|-------------|-------------------|
| FC-1 (structural) | Route structure valid | SR-path structure valid (waypoints reachable) |
| FC-2.5 (benchmark) | BKS reference exists | No published BKS — `NoReference` status |
| FC-2 (capacity) | Vehicle capacity not exceeded | Arc capacity not exceeded (`obj.is_finite()`) |
| FC-3 (demand) | All customers served | All demands routed (connectivity check) |

Expected: all four checks map naturally. No schema change required.

### FleetSemanticCheck mapping

| FCS concept | CVRP meaning | ROADEF equivalent |
|-------------|-------------|-------------------|
| `constraint_type` | ATMOST(K) or EXACT(K) | No fleet constraint — `Unspecified` |
| `declared_k` | Published fleet size | N/A |
| `routes_used` | Number of routes in solution | Number of SR-paths with non-empty waypoints |
| `outcome` | Valid / NotComparable / PendingVerification | `PendingVerification` (no fleet semantics defined) |

The current prediction is that `PendingVerification` is the most semantically faithful
outcome because the ROADEF benchmark defines no fleet semantics. M20 will determine
whether this representation remains informative or merely exposes a limitation of the
current abstraction. This is an experiment, not an assumption.

### FleetUtilizationCertificate mapping

| FUC field | CVRP meaning | ROADEF equivalent |
|-----------|-------------|-------------------|
| `avg_utilization` | Mean vehicle load / capacity | Mean link saturation (avg MLU) |
| `median_utilization` | Median vehicle load | Median link saturation |
| `load_cv` | Coefficient of variation of loads | CV of link saturations |
| `residual_concentration_ratio` | Residual capacity concentration | Residual capacity concentration |
| `packing_classification` | HighlyConsolidated / WellPacked / etc. | Same classification applied to link saturation |
| `capacity_violations` | Routes exceeding capacity | Links with saturation ≥ 1.0 |
| `total_demand` | Sum of customer demands | Sum of traffic demands |
| `fleet_capacity_used` | Total load across all routes | Total traffic routed |

Expected: all fields map naturally. The ROADEF adapter will compute these from
`TimeSlotLoads` (arc_saturations, arc_flows) rather than from route loads.

---

## Evidence Produced

M20 shall produce the following artifacts:

- ROADEF Execution Certificates — one JSON per setA instance in `benchmarks/roadef/campaign/certificates/`
- Qualification mapping report — per-component pass/fail with reason
- Prediction vs Observation table — comparing pre-experiment predictions against actual results
- Qualification compatibility assessment — which components generalize, which do not
- Recommendation for GOV-011 — which components are eligible for platform promotion

---

## M20 Acceptance Criteria

M20 is complete when:

1. A ROADEF adapter produces all three qualification structs for each setA instance.
2. `ExecutionCertificate::generate(CertificateInput)` runs successfully for each instance.
3. Certificates are written to `benchmarks/roadef/campaign/certificates/`.
4. The experiment outcome is recorded: which predictions were correct, which were wrong.
5. Zero modifications to the frozen Qualification Subsystem v1.0.

---

## Non-Goals

M20 is not intended to:

- Improve ROADEF solution quality
- Optimize runtime
- Outperform the reference solver
- Redesign Qualification
- Invent new qualification concepts (e.g., `BudgetSemanticCheck`) inside the adapter

These activities belong to later milestones.

---

## What M20 Must Not Do

- Modify any frozen Qualification Subsystem component to make ROADEF fit.
- Force a mapping that distorts the ROADEF domain semantics.

If a mapping is unnatural, record it as evidence of abstraction boundary mismatch.
That evidence is more valuable than a forced mapping that obscures the real result.

---

## Possible Outcomes

### Outcome A — Full generalization

All four components reuse unchanged. `FleetSemanticCheck` produces `PendingVerification`
for all ROADEF instances (correct behavior for a domain without fleet semantics).

**Consequence:** Qualification v1.0 is eligible for platform promotion under GOV-011.

### Outcome B — FCS is domain-specific

`FleetSemanticCheck` cannot produce a meaningful result for ROADEF without schema changes.
FCF, FUC-001, and ExecutionCertificate reuse unchanged.

**Consequence:** FCF + FUC-001 + ExecutionCertificate are eligible for platform promotion.
FCS remains adapter-level until a more general semantic qualification abstraction emerges
from a third domain. GOV-011 would promote the proven components only.

### Outcome C — Deeper incompatibility

One or more of FCF, FUC-001, or ExecutionCertificate require schema changes.

**Consequence:** Qualification v1.0 is not yet platform-ready. The required changes
are documented as evidence for Qualification v2.0 design.

---

## Decision Tree After M20

```
             Qualification v1.0
                     │
          ┌──────────┴──────────┐
          │                     │
      Generalizes          Does not generalize
          │                     │
          ▼                     ▼
   GOV-011 candidate     Qualification v2 research
```

---

## Version History

| Version | Date | Changes |
|---------|------|---------|
| v1.0 | 2026-07-09 | Initial plan — pre-experiment predictions recorded, frozen as protocol |