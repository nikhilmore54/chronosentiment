# Coralys Platform — Implementation Gap Register

**Document type:** Gap Register
**Version:** 1.0
**Status:** Baseline
**Date:** 2026-07-26
**Owner:** Platform / Engineering

---

## Document Governance

| Field | Value |
|-------|-------|
| Document Status | Baseline v1.0 |
| Review Trigger | Sprint completion; implementation status change; new baseline document added |

**Relationship to other documents:**
- Informed by: `CORALYS_ARCHITECTURE_TRACEABILITY.md` (primitive → crate mapping)
- Informed by: `CORALYS_PRODUCT_TRACEABILITY.md` (capability → crate mapping)
- Informed by: All Baseline product blueprints (UC-B-001, CS-E-B-001, CS-P-B-001)
- Informs: Engineering sprint planning; pilot readiness assessment

---

## Purpose

This document is the single consolidated implementation gap register for the Coralys platform and all products built on it. It identifies every capability that is documented in a Baseline document but not yet fully implemented in the codebase.

Gaps are categorised by severity and ordered by implementation priority within each product.

---

## Gap Severity Legend

| Severity | Meaning |
|----------|---------|
| **Critical** | Blocks MVP; product cannot be demonstrated or piloted without this |
| **High** | Required for commercial launch; significant customer value |
| **Medium** | Enhances product quality; required for scale |
| **Low** | Nice-to-have; v2.0 candidate |

---

## Platform Primitive Gaps

These gaps affect all products built on the Coralys platform.

| Gap | Severity | Description | Blocking |
|-----|----------|-------------|---------|
| **Intent primitive** | High | No standalone `Intent` trait at the platform level. Intent is implicit in `Scenario` configuration. Without a formal Intent primitive, the platform cannot enforce the invariant "every Workspace has exactly one Intent". | Platform invariant enforcement |
| **Context primitive** | Medium | No `Context` primitive implemented. Operational context is embedded in `Scenario` configuration. Without a formal Context primitive, context cannot be versioned, queried, or tracked independently. | Knowledge Graph completeness |
| **Actor primitive** | Medium | No standalone `Actor` trait at the platform level. Actor identity is carried through `Worker::id()` in the planning domain. Without a formal Actor primitive, actor-level provenance is not tracked across all domains. | Cross-domain provenance |
| **Hypothesis versioning** | High | `DecisionProposal` does not support versioning (v1 → v2 → v3). Hypothesis versioning is a core platform capability documented in the architecture. | Investment Thesis versioning; Roster Strategy versioning |
| **Evidence immutability enforcement** | High | Evidence immutability is a platform invariant but is not enforced at the platform level. Any adapter could mutate historical evidence without the platform detecting it. | Platform invariant enforcement |
| **Workspace lifecycle enforcement** | Medium | Workspace state transitions (Active → Completed → Archived) are not enforced at the platform level. Each adapter manages its own lifecycle. | Governance |
| **Outcome-Workspace ownership enforcement** | Medium | The invariant "every Outcome belongs to exactly one Workspace" is not enforced at the platform level. | Platform invariant enforcement |
| **Pattern maturity model** | Medium | Pattern maturity levels (Candidate → Observed → Repeated → Validated → Institutionalised) are documented in Architecture Observation 4 but not implemented. | Knowledge Graph completeness |
| **Knowledge Graph persistence** | High | The Knowledge Graph (`coralys-ecology`) has trait-level foundations (`MemoryModel`, `TopologyModel`) but no persistence layer. Patterns and knowledge are lost between sessions. | All products |
| **Knowledge Graph traversal** | High | No traversal capability in the Knowledge Graph. Cannot query relationships between entities. | All products |
| **Knowledge Graph semantic retrieval** | Medium | No semantic retrieval (similarity search, contextual retrieval) in the Knowledge Graph. | v2.0 Knowledge Graph Services |
| **Review primitive** | Medium | No structured `Review` primitive at the platform level. Review is currently implemented as `DecisionPlugin::evaluate()` — a function call, not a structured record with attendees, outcomes, and conditions. | Committee Review; Quarterly Research Review |

---

## UltraCrew Gaps

### Critical Gaps (block MVP)

| Gap | Description | Implementing crate |
|-----|-------------|-------------------|
| **Disruption recovery workflow** | Real-time re-optimisation when operational plans change is not yet a complete workflow. Disruption modelling exists in `adapters/airline` but the end-to-end workflow (disruption recorded → re-optimisation triggered → recovery options ranked → recovery accepted) is not implemented. | `adapters/ultracrew` |
| **Disruption evidence recording** | Disruptions are not yet recorded as evidence items in the Scheduling Workspace. | `adapters/ultracrew` |

### High Gaps (required for commercial launch)

| Gap | Description | Implementing crate |
|-----|-------------|-------------------|
| **Workforce Operations Learning Loop workflow** | The learning loop is partially implemented (innovation tracking, ecology-aware optimisation) but the end-to-end workflow (cycle completed → outcomes reviewed → patterns identified → insights added to Knowledge Graph) is not implemented. | `adapters/ultracrew` |
| **Operational Knowledge Graph persistence** | The Knowledge Graph has trait-level foundations but no persistence. Operational patterns are lost between scheduling cycles. | `coralys-ecology` |
| **Workspace status transitions** | Scheduling Workspace status transitions (Active → Completed → Archived) are not enforced. | `adapters/ultracrew` |
| **Recovery option ranking** | When a disruption occurs, recovery options are not yet ranked by impact on the remaining schedule. | `adapters/ultracrew` |

### Medium Gaps (required for scale)

| Gap | Description | Implementing crate |
|-----|-------------|-------------------|
| **Timeline filtering** | The Scheduling Timeline cannot be filtered by type, date range, or crew member. | `adapters/ultracrew` |
| **Cycle review report** | The Learning Loop does not yet produce a structured cycle review report. | `adapters/ultracrew` |
| **Pattern accumulation** | Workforce behaviour patterns are not yet accumulated across scheduling cycles in a queryable form. | `coralys-ecology` |

---

## ChronoSentiment Enterprise Gaps

**Overall status:** The ChronoSentiment adapter (`adapters/chronosentiment`) is an empty stub. All Enterprise capabilities are documented in the blueprint but not yet implemented.

### Critical Gaps (block MVP)

| Gap | Description | Implementing crate |
|-----|-------------|-------------------|
| **Decision Workspace** | No implementation. The core container for investment decisions does not exist in the codebase. | `adapters/chronosentiment` |
| **Investment Thesis with versioning** | No implementation. The core hypothesis primitive for investment decisions does not exist. | `adapters/chronosentiment` |
| **Evidence management** | No implementation. Research evidence capture and immutability enforcement do not exist. | `adapters/chronosentiment` |
| **Decision Timeline** | No implementation. The `DecisionLineage` foundation exists in `coralys-core` but is not wired to the ChronoSentiment adapter. | `adapters/chronosentiment` |
| **Decision Outcome recording** | No implementation. Investment outcomes cannot be recorded. | `adapters/chronosentiment` |

### High Gaps (required for commercial launch)

| Gap | Description | Implementing crate |
|-----|-------------|-------------------|
| **Committee Review workflow** | No implementation. Structured committee review with attendees, discussion, decision, and conditions does not exist. | `adapters/chronosentiment` |
| **Organisational Decision Learning Loop** | No implementation. The post-decision review process does not exist. | `adapters/chronosentiment` |
| **Institutional Decision Knowledge Graph** | No implementation. The `MemoryModel` foundation exists in `coralys-ecology` but is not wired to the ChronoSentiment adapter. | `adapters/chronosentiment` |

### Medium Gaps (required for scale)

| Gap | Description | Implementing crate |
|-----|-------------|-------------------|
| **AI conversation documentation** | No implementation. AI research conversations cannot be captured as evidence items. | `adapters/chronosentiment` |
| **Regulatory compliance reporting** | No implementation. Automated AI documentation reports for FCA, SEC, EU AI Act do not exist. | `adapters/chronosentiment` |

---

## ChronoSentiment Personal Gaps

**Overall status:** The ChronoSentiment adapter (`adapters/chronosentiment`) is an empty stub. All Personal capabilities are documented in the blueprint but not yet implemented. Personal shares the adapter stub with Enterprise.

### Critical Gaps (block MVP)

| Gap | Description | Implementing crate |
|-----|-------------|-------------------|
| **Research Workspace** | No implementation. The core container for investment research does not exist. | `adapters/chronosentiment` |
| **Research Dossier** | No implementation. Structured, accumulated research records do not exist. | `adapters/chronosentiment` |
| **Investment Thesis with versioning** | No implementation. Shared with Enterprise — the core hypothesis primitive does not exist. | `adapters/chronosentiment` |
| **Research Timeline** | No implementation. The `DecisionLineage` foundation exists in `coralys-core` but is not wired to the ChronoSentiment adapter. | `adapters/chronosentiment` |
| **Investment Outcome recording** | No implementation. Investment outcomes cannot be recorded. | `adapters/chronosentiment` |

### High Gaps (required for commercial launch)

| Gap | Description | Implementing crate |
|-----|-------------|-------------------|
| **Quarterly Research Review** | No implementation. Structured periodic review of active theses does not exist. | `adapters/chronosentiment` |
| **Personal Investment Learning Loop** | No implementation. The post-outcome review process does not exist. | `adapters/chronosentiment` |
| **Personal Investment Knowledge Graph** | No implementation. The `MemoryModel` foundation exists in `coralys-ecology` but is not wired to the ChronoSentiment adapter. | `adapters/chronosentiment` |

### Medium Gaps (required for scale)

| Gap | Description | Implementing crate |
|-----|-------------|-------------------|
| **AI conversation documentation** | No implementation. AI research conversations cannot be captured as research sources. | `adapters/chronosentiment` |
| **Research quality scoring** | No implementation. v2.0 candidate. | `adapters/chronosentiment` |

---

## Gap Summary by Product

| Product | Critical | High | Medium | Low | Total |
|---------|----------|------|--------|-----|-------|
| Platform (all products) | 0 | 4 | 8 | 0 | 12 |
| UltraCrew | 2 | 4 | 3 | 0 | 9 |
| ChronoSentiment Enterprise | 5 | 3 | 2 | 0 | 10 |
| ChronoSentiment Personal | 5 | 3 | 2 | 0 | 10 |
| **Total** | **12** | **14** | **15** | **0** | **41** |

---

## Implementation Priority Order

Based on the gap register, the recommended implementation priority is:

**Phase 1 — UltraCrew MVP completion (highest ROI; implementation already substantial)**

1. Disruption recovery workflow (Critical)
2. Disruption evidence recording (Critical)
3. Workforce Operations Learning Loop workflow (High)
4. Operational Knowledge Graph persistence (High)
5. Workspace status transitions (High)

**Phase 2 — Platform primitive formalisation (enables all products)**

6. Hypothesis versioning at platform level (High)
7. Evidence immutability enforcement (High)
8. Knowledge Graph traversal (High)
9. Intent primitive (High)
10. Review primitive (Medium)

**Phase 3 — ChronoSentiment adapter (enables both CS products)**

11. Decision Workspace / Research Workspace (Critical — shared foundation)
12. Investment Thesis with versioning (Critical — shared)
13. Evidence management (Critical — shared)
14. Decision/Research Timeline wiring (Critical — shared)
15. Outcome recording (Critical — shared)

**Phase 4 — ChronoSentiment Enterprise differentiation**

16. Committee Review workflow (High)
17. Organisational Decision Learning Loop (High)
18. Institutional Decision Knowledge Graph (High)

**Phase 5 — ChronoSentiment Personal differentiation**

19. Quarterly Research Review (High)
20. Personal Investment Learning Loop (High)
21. Personal Investment Knowledge Graph (High)

---

*Coralys Platform Implementation Gap Register v1.0 | July 2026 | Status: Baseline*
*Single consolidated gap register from all Baseline documents.*
*Review trigger: Sprint completion; implementation status change; new baseline document added.*