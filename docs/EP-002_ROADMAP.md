# Coralys Platform — Post EP-001 Roadmap

**Document type:** Roadmap
**Version:** 2.0
**Status:** Operational
**Date:** 2026-07-27
**Owner:** Product / Engineering Leadership

> **Supersedes:** v1.0 (2026-07-27) — Platform-centric framing replaced by product-first governance principle.
> The phase structure and evidence gates are preserved. The framing, scope of EP-002, and governing principle have changed.
> See [`CS-S-001_Product_First_Governance_Principle.md`](strategy/CS-S-001_Product_First_Governance_Principle.md) for the authoritative statement of the product-first principle.

---

## Document Governance

| Field | Value |
|-------|-------|
| Document Status | Operational v2.0 |
| Review Trigger | Phase completion; evidence acquisition changes priority; product milestone reached |

**Relationship to other documents:**
- Informed by: `EP-001_MILESTONE.md` (EP-001 completion state)
- Informed by: `CORALYS_GAP_REGISTER.md` v2.0 (remaining gaps)
- Informed by: `CORALYS_ARCHITECTURE_TRACEABILITY.md` v2.0 (implementation state)
- Governed by: `CS-S-001_Product_First_Governance_Principle.md` (product-first principle — authoritative)
- Informs: Engineering sprint planning; commercial execution; pilot planning

---

## Architectural Stability Declaration

As of EP-001, the repository is considered **architecturally stable**.

This means:

> Changes from this point onward should be driven by evidence from product validation and customer outcomes rather than by introducing new architectural abstractions.

The governing principle for all future platform evolution is:

> **Products discover abstractions; architects consolidate them.**

No new platform abstraction may be introduced unless required by at least two independent domain adapters or supported by operational evidence from product use. Platform work is now secondary to product work. See [`CS-S-001`](strategy/CS-S-001_Product_First_Governance_Principle.md) for the full governing principle.

---

## Capability Maturity Model

All traceability documents use the following five-level maturity model. This is a first-class governance concept, not a status label.

| Level | Label | Meaning |
|-------|-------|---------|
| **L0** | Documented | Exists in architecture or blueprint only; no implementation |
| **L1** | Implemented | Code exists and automated tests pass |
| **L2** | Demonstrated | Exercised successfully in benchmark, simulation, or pilot |
| **L3** | Operational | Used successfully in production operations |
| **L4** | Commercially Validated | Customer has confirmed measurable business value |

**Current state of key capabilities (post EP-001):**

| Capability | Level |
|------------|-------|
| MOGA optimisation engine | L2 — Demonstrated (INRC-II benchmarks, ablation matrices) |
| UltraCrew scheduling | L2 — Demonstrated (SunAir canonical scenario, 100% coverage) |
| UltraCrew disruption recovery | L1 — Implemented (EP-001; awaiting SunAir pilot) |
| UltraCrew operational learning loop | L1 — Implemented (EP-001; awaiting SunAir pilot) |
| ChronoSentiment Personal adapter foundation | L1 — Implemented (EP-001; awaiting prototype workflow) |
| Platform Evidence primitive (adapter level) | L1 — Implemented (EP-001; awaiting platform promotion) |
| Platform Hypothesis primitive (adapter level) | L1 — Implemented (EP-001; awaiting platform promotion) |
| Knowledge Graph persistence | L0 — Documented |

---

## Phase Roadmap

```
EP-001  Platform Foundations Operational  ✅ Complete
            │
            ▼
EP-002  Platform Consolidation
            │
            ▼
P-001   SunAir Operational Demonstration
            │
            ▼
CV-001  Commercial Validation
            │
            ▼
P-002   Knowledge Graph Persistence
        Cross-Workspace Learning
        Deterministic Engine Completion
```

Each phase has a distinct purpose. No phase should begin until its predecessor has produced the evidence required to justify the next step.

---

## EP-002 — Platform Consolidation

> **v2.0 reframe:** EP-002 was previously titled "Platform Primitive Formalisation." That title implied a platform-driven agenda. The correct framing is **Platform Consolidation**: removing duplication that products have already exposed, driven by demonstrated product need rather than architectural completeness.

**Objective:** Consolidate proven adapter concepts into `coralys-core` where two or more products have independently demonstrated the same need. This is extraction of what already exists, not invention of new capabilities.

**Governing principle:** Every abstraction consolidated into `coralys-core` must already be used by at least one product in demonstrated operation, and a second product must demonstrably need the same semantics. EP-002 scope is therefore conditional on product evidence, not predetermined.

**Scope (conditional on product evidence):**

| Item | Consolidation trigger | Source |
|------|----------------------|--------|
| Platform `Evidence` trait | When UltraCrew operational evidence also requires Evidence semantics | `adapters/chronosentiment/src/evidence.rs` |
| Platform `Hypothesis` trait | When a second product demonstrates the same versioning need | `adapters/chronosentiment/src/hypothesis.rs` |
| Platform `Intent` primitive | When two products expose the same one-Intent-per-Workspace invariant | Implicit in `InvestmentWorkspace.research_objective` |
| Workspace-Outcome invariant | When both adapters confirm semantics are genuinely shared | `adapters/chronosentiment/src/workspace.rs` |
| Adapter refactoring | After consolidation triggers are met | Both adapters |

**What EP-002 is not:**
- Not a new feature sprint
- Not a Knowledge Graph implementation (that is P-002)
- Not a new primitive invention — only extraction of what products have already proven
- Not a prerequisite for P-001 or CV-001 — product work proceeds in parallel

**Exit criterion:** Consolidated traits are used by at least two adapters with no behaviour change. All existing tests pass. No consolidation is performed speculatively.

---

## P-001 — SunAir Operational Demonstration

**Objective:** Move UltraCrew capabilities from L1 (Implemented) to L2 (Demonstrated) through a realistic operational scenario.

**This is an evidence gate, not a deployment.** The goal is to answer specific questions that create new entries in the Evidence Linkage document.

**Questions to answer:**

| Question | Capability under test |
|----------|-----------------------|
| Does disruption recovery reduce planner workload? | `disruption_recovery::DisruptionRecoveryEngine` |
| Are recovery recommendations acceptable to dispatchers? | Recovery option ranking |
| Is the operational learning loop producing useful patterns rather than noise? | `decision_intelligence::OperationalLearningLoop` |
| Does explainability increase confidence in scheduling decisions? | Explanation Engine (S3-03) |
| Are generated rosters operationally acceptable? | MOGA engine + constraint engine |

**Evidence produced:** New entries in `CORALYS_EVIDENCE_LINKAGE.md` moving capabilities from L1 to L2.

**Existing readiness:** All pilot infrastructure is complete (P-001 programme closed 2026-07-23). The SunAir pilot guide (`docs/sunair_pilot_guide.md`) and runbook (`docs/P001_PILOT_RUNBOOK.md`) are ready.

---

## CV-001 — Phase 1B ChronoSentiment Enterprise Commercial Validation

**Objective:** Move ChronoSentiment hypotheses H1–H7 from Confidence D toward validated evidence through customer discovery and prototype workflows.

**This is an evidence programme, not a product launch.** The current implementation is sufficient to begin customer discovery. The goal is determining whether organisations perceive enough value to adopt the product.

**Governing framework:** Hypotheses H1–H7 from `CS-R-015_Investment_Thesis.md`. Each interview, prototype session, workshop, or design-partner engagement should either strengthen confidence, weaken confidence, refine the hypothesis, or invalidate it.

**Evidence types (from `EL-001_Phase1B_Evidence_Ledger.md`):**

| Type | Weight | Description |
|------|--------|-------------|
| INT | Highest | Customer interview — direct purchase intent signal |
| EXP | High | Expert interview — cross-firm pattern visibility |
| OBS | Medium | Public observation — first-hand statement |
| DEM | High | Product demonstration — behavioural signal |
| POC | Highest | Prototype evaluation — strongest behavioural signal |

**Kill criteria:** Defined in `CV-001_Commercial_Validation_Playbook.md`. Evidence acquisition stops and a go/no-go decision is made when kill criteria are met or when 5 evidence records have been acquired (rolling synthesis trigger).

---

## P-002 — Knowledge Graph Persistence and Cross-Workspace Learning

**Objective:** Implement the Knowledge Graph persistence layer and cross-Workspace pattern accumulation. This is the long-term platform capability that enables the full Coralys vision.

**Trigger condition:** P-002 begins only after P-001 and CV-001 have produced sufficient operational and commercial evidence to justify the investment. The Knowledge Graph is the most complex remaining capability; it should not be built speculatively.

**Scope (indicative):**

| Item | Description |
|------|-------------|
| KG persistence | `coralys-ecology` — persist patterns between sessions |
| KG traversal | Query relationships between entities |
| Cross-Workspace learning | Accumulate patterns across scheduling cycles |
| Deterministic engine | Wire `deterministic_rng` into MOGA engine (GAP-UC-007) |
| KG semantic retrieval | Similarity search, contextual retrieval (v2.0 candidate) |

---

## Governance Principle: Products Discover Abstractions

The following rule governs all future platform evolution:

> **Products discover abstractions; architects consolidate them.**

No new platform abstraction may be introduced unless required by at least two independent domain adapters or supported by operational evidence from product use.

This rule:
- Prevents speculative platform design
- Ensures abstractions are grounded in real domain requirements
- Keeps the platform lean and the adapters expressive
- Ensures platform evolution is driven by product outcomes, not architectural completeness

**Effort allocation (6–12 month horizon):**

| Area | Allocation |
|------|-----------|
| Product development and validation | 60–70% |
| Platform consolidation | 20–30% |
| Research | 10–20% |

The authoritative statement of this principle is [`CS-S-001_Product_First_Governance_Principle.md`](strategy/CS-S-001_Product_First_Governance_Principle.md).

---

*Coralys Platform Post-EP-001 Roadmap v2.0 | July 2026 | Status: Operational*
*Supersedes: v1.0 (2026-07-27) — Platform-centric framing replaced by product-first governance.*
*Review trigger: Phase completion; evidence acquisition changes priority; product milestone reached.*