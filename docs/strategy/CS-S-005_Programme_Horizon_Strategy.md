# Coralys — Programme Horizon Strategy

**Document type:** Strategic Governance
**Document ID:** CS-S-005
**Version:** 1.0
**Status:** Operational
**Date:** 2026-08-02
**Owner:** Product / Engineering Leadership

---

## Document Governance

| Field | Value |
|-------|-------|
| Document Status | Operational v1.0 |
| Review Trigger | UltraCrew MSP shipped; ROADEF submission complete; RP-408 Platform Consolidation initiated |

**Relationship to other documents:**
- Extends: [`CS-S-001_Product_First_Governance_Principle.md`](CS-S-001_Product_First_Governance_Principle.md) — this document sharpens CS-S-001 for the current three-workstream context
- Informs: [`CS-S-002_UltraCrew_Product_Strategy_v1.0.md`](CS-S-002_UltraCrew_Product_Strategy_v1.0.md) — MSP scope defined here
- Informs: [`docs/roadef/ROADEF_PROGRAMME.md`](../roadef/ROADEF_PROGRAMME.md) — Horizon 2 research sequencing
- Informs: [`docs/governance/CAPABILITY_REGISTER.md`](../governance/CAPABILITY_REGISTER.md) — capability promotion flow
- Defers: RP-408 Platform Consolidation — see §5

---

## Context

As of 2026-08-02, the programme has three active workstreams:

| Workstream | Current state |
|------------|--------------|
| UltraCrew MSP | Engine implemented; MSP product scope not yet shipped |
| ROADEF Competition | RP-401 frozen; RP-402 next |
| Coralys Platform | Stable; awaiting evidence from products before further consolidation |

CS-S-001 established the principle that products discover abstractions and that UltraCrew has execution priority. This document applies that principle to the specific three-workstream context and records the horizon model adopted on 2026-08-02.

---

## 1. The Three-Horizon Model

The programme is organised into three horizons. Horizons are not time-boxes — they are priority tiers. Work in Horizon 1 takes precedence over Horizon 2, which takes precedence over Horizon 3.

### Horizon 1 — UltraCrew MSP (Revenue First)

**Question:** What is the smallest product someone will pay for?

Everything that does not directly contribute to a paying customer should wait. The MSP is not a reduced version of a grand vision — it is the complete version of the smallest valuable thing.

**MSP feature scope:**

| Feature | Rationale |
|---------|-----------|
| Robust schedule generation | Core value proposition |
| Constraint management | Required for legal compliance |
| Explainability ("Why was this shift assigned?") | Adoption barrier — planners need to trust the output |
| Manual edits and re-optimisation | Planners need override capability |
| Import/export (Excel/CSV) | Integration with existing workflows |
| Basic dashboards | Operational visibility |
| Stable REST API | Integration with customer systems |
| Pilot deployment capability | Required for first customer |

**Explicitly not required for MSP:**

- Hyper-heuristics
- Hybrid exact optimisation
- Advanced routing research
- C5 platform maturity
- RP-408 Platform Consolidation

These are valuable. They are not required to get a first paying customer.

### Horizon 2 — ROADEF Competition (Research and Benchmark Evidence)

**Goal:** Competition performance and publishable evidence. Not product features.

The ROADEF programme continues in parallel with UltraCrew MSP development. It strengthens Coralys by producing benchmark-validated capabilities, but it does not block or gate the MSP.

**Research sequence:**

| RP | Title | Status |
|----|-------|--------|
| RP-401 | ECMP-Aware Flow Estimation | FROZEN |
| RP-402 | Budget-Aware Transition Planning | Next |
| RP-403 | Multi-Path Candidate Generation | Planned |
| RP-404 | Large Neighbourhood Search | Planned |
| RP-405 | Hyper-Heuristic Operator Selection | Planned |
| RP-406 | Coralys MOGA Integration | Planned |
| RP-407 | Hybrid Exact Subproblem | Planned |

### Horizon 3 — Coralys Platform (Capability Extraction)

**Role:** Consumer of proven discoveries, not the primary focus.

Coralys evolves from validated capabilities, not speculative abstractions. The promotion flow is:

```
ROADEF proves a capability
        |
        v
UltraCrew MSP needs the same capability
        |
        v
Extract into Coralys platform
```

Not:

```
Build Coralys first
        |
        v
Hope ROADEF and UltraCrew use it
```

This is a direct application of the CS-S-001 governing principle:

> **Products discover abstractions; architects consolidate them.**

---

## 2. Priority Order

The execution priority order is:

1. **UltraCrew MSP** — revenue, first paying customer
2. **ROADEF Competition** — research, benchmark evidence
3. **Coralys Platform** — extraction of proven capabilities

This order is not a statement about relative importance in the long run. It is a statement about what to do next when there is a conflict for time or attention.

---

## 3. Effort Allocation (6-12 Month Horizon)

| Area | Allocation | Rationale |
|------|------------|-----------|
| Product development and validation (UltraCrew MSP) | 60-70% | Revenue-generating; first customer |
| Platform consolidation (Coralys) | 20-30% | Extraction of proven capabilities only |
| Research (ROADEF) | 10-20% | Benchmark evidence; competition performance |

These are directional, not contractual. They should be revisited when the MSP ships or when the ROADEF submission is complete.

---

## 4. Capability Promotion Flow

Capabilities are promoted to the Coralys platform only when:

1. A capability has been validated in a research context (ROADEF), **and**
2. At least one product (UltraCrew MSP) independently needs the same capability

The promotion criterion from CS-S-001 applies:

> **Two independent products must independently discover the same need before a capability is promoted to the platform.**

Capabilities that are only needed by one product remain in that product's adapter. They are not promoted speculatively.

The [`CAPABILITY_REGISTER.md`](../governance/CAPABILITY_REGISTER.md) is the authoritative record of capability status.

---

## 5. RP-408 Platform Consolidation — Deferred

**Decision:** RP-408 Platform Consolidation is deferred until both of the following conditions are met:

1. UltraCrew MSP has shipped (first paying customer)
2. ROADEF submission is complete

**Rationale:** Platform consolidation at this stage would be premature. The evidence base for what to consolidate is still being generated by UltraCrew MSP development and ROADEF research. Consolidating now would mean consolidating the wrong things.

**Trigger for re-evaluation:** When both conditions above are met, RP-408 should be scoped based on the actual capabilities that have been independently discovered by both ROADEF and UltraCrew MSP.

---

## 6. Governing Principle (from CS-S-001)

This document is an application of the CS-S-001 governing principle to the current three-workstream context. The principle is:

> **Products discover abstractions; architects consolidate them.**

The full principle and its implications are documented in [`CS-S-001_Product_First_Governance_Principle.md`](CS-S-001_Product_First_Governance_Principle.md).

---

## 7. Review Triggers

This document should be reviewed when:

- UltraCrew MSP ships (first paying customer)
- ROADEF submission is complete
- RP-408 Platform Consolidation is initiated
- A material change in product positioning, target market, or platform identity occurs

---

*Coralys Programme Horizon Strategy CS-S-005 v1.0 | 2026-08-02 | Status: Operational*
*Extends CS-S-001. Records the three-horizon model adopted 2026-08-02.*
*Review trigger: UltraCrew MSP shipped; ROADEF submission complete; RP-408 initiated.*
