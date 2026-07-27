# Coralys Platform — Post EP-001 Roadmap

**Document type:** Roadmap
**Version:** 1.0
**Status:** Operational
**Date:** 2026-07-27
**Owner:** Platform / Engineering

---

## Document Governance

| Field | Value |
|-------|-------|
| Document Status | Operational v1.0 |
| Review Trigger | Phase completion; evidence acquisition changes priority; new architectural constraint identified |

**Relationship to other documents:**
- Informed by: `EP-001_MILESTONE.md` (EP-001 completion state)
- Informed by: `CORALYS_GAP_REGISTER.md` v2.0 (remaining gaps)
- Informed by: `CORALYS_ARCHITECTURE_TRACEABILITY.md` v2.0 (implementation state)
- Informs: Engineering sprint planning; commercial execution; pilot planning

---

## Architectural Stability Declaration

As of EP-001, the repository is considered **architecturally stable**.

This means:

> Changes from this point onward should be driven by evidence from implementation and customer validation rather than by introducing new architectural abstractions.

The governing principle for all future platform evolution is:

> **No new platform abstraction may be introduced unless required by at least two independent domain adapters or supported by operational evidence.**

This preserves the discipline that Coralys evolves from proven patterns rather than speculative design.

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
EP-002  Platform Primitive Formalisation
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

## EP-002 — Platform Primitive Formalisation

**Objective:** Promote proven adapter concepts into `coralys-core`. This is extraction and consolidation, not new feature development.

**Governing principle:** Every abstraction promoted into `coralys-core` must already exist in at least one concrete adapter. EP-002 is justified because the ChronoSentiment adapter has already proven Evidence, Hypothesis, and Workspace-Outcome invariants.

**Scope:**

| Item | Description | Source |
|------|-------------|--------|
| Platform `Evidence` trait | Append-only semantics; immutability enforced at platform level | `adapters/chronosentiment/src/evidence.rs` |
| Platform `Hypothesis` trait | Immutable version history; versioning enforced at platform level | `adapters/chronosentiment/src/hypothesis.rs` |
| Platform `Intent` primitive | Standalone trait; enforces one-Intent-per-Workspace invariant | Implicit in `InvestmentWorkspace.research_objective` |
| Workspace-Outcome invariant | Platform-level enforcement of single-Outcome-per-Workspace | `adapters/chronosentiment/src/workspace.rs` |
| Adapter refactoring | UltraCrew and ChronoSentiment adapters consume new platform traits | Both adapters |

**What EP-002 is not:**
- Not a new feature sprint
- Not a Knowledge Graph implementation (that is P-002)
- Not a new primitive invention — only extraction of what already exists

**Exit criterion:** Both adapters compile against the new platform traits with no behaviour change. All existing tests pass.

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

## Governance Principle: Evidence Before Abstraction

The following rule governs all future platform evolution:

> **No new platform abstraction may be introduced unless required by at least two independent domain adapters or supported by operational evidence.**

This rule:
- Prevents speculative platform design
- Ensures abstractions are grounded in real domain requirements
- Keeps the platform lean and the adapters expressive
- Aligns with the Coralys principle that the platform evolves from proven patterns

---

*Coralys Platform Post-EP-001 Roadmap v1.0 | July 2026 | Status: Operational*
*Review trigger: Phase completion; evidence acquisition changes priority; new architectural constraint identified.*