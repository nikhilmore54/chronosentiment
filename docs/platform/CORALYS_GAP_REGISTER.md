# Coralys Platform — Implementation Gap Register

**Document type:** Gap Register
**Version:** 2.0
**Status:** Updated — EP-001 resolutions applied
**Date:** 2026-07-27
**Owner:** Platform / Engineering

---

## Document Governance

| Field | Value |
|-------|-------|
| Document Status | v2.0 — EP-001 post-sprint update |
| Previous Version | v1.0 Baseline (2026-07-26) |
| Review Trigger | Sprint completion; implementation status change; new baseline document added |

**Relationship to other documents:**
- Informed by: `CORALYS_ARCHITECTURE_TRACEABILITY.md` (primitive → crate mapping)
- Informed by: `CORALYS_PRODUCT_TRACEABILITY.md` (capability → crate mapping)
- Informed by: All Baseline product blueprints (UC-B-001, CS-E-B-001, CS-P-B-001)
- Informed by: `EP-001_MILESTONE.md` (sprint resolutions)
- Informs: Engineering sprint planning; pilot readiness assessment

---

## Purpose

This document is the single consolidated implementation gap register for the Coralys platform and all products built on it. It identifies every capability that is documented in a Baseline document but not yet fully implemented in the codebase.

Gaps are categorised by severity and ordered by implementation priority within each product.

---

## EP-001 Sprint Summary

EP-001 closed or substantially resolved the following gaps. Full evidence is recorded in `docs/EP-001_MILESTONE.md`.

| Gap (v1.0 label) | Resolution | Evidence |
|------------------|------------|----------|
| Disruption recovery workflow (UltraCrew Critical) | Resolved — `adapters/ultracrew/src/disruption_recovery.rs` | EP-001 |
| Disruption evidence recording (UltraCrew Critical) | Resolved — evidence recording wired in disruption_recovery.rs | EP-001 |
| Workforce Operations Learning Loop workflow (UltraCrew High) | Resolved — `adapters/ultracrew/src/decision_intelligence.rs` | EP-001 |
| Cycle review report (UltraCrew Medium) | Resolved — `CycleReviewReport` struct in decision_intelligence.rs | EP-001 |
| Recovery option ranking (UltraCrew High) | Resolved — options ranked by feasibility then impact in disruption_recovery.rs | EP-001 |
| Pattern maturity model (Platform Medium) | Resolved — `PatternMaturity` enum in decision_intelligence.rs (UltraCrew) and learning.rs (ChronoSentiment) | EP-001 |
| ChronoSentiment Personal — Research Workspace (Critical) | Resolved — `adapters/chronosentiment/src/workspace.rs` | EP-001 |
| ChronoSentiment Personal — Investment Thesis with versioning (Critical) | Resolved — `adapters/chronosentiment/src/hypothesis.rs` | EP-001 |
| ChronoSentiment Personal — Evidence management (Critical) | Resolved — `adapters/chronosentiment/src/evidence.rs` | EP-001 |
| ChronoSentiment Personal — Research Timeline (Critical) | Resolved — `adapters/chronosentiment/src/timeline.rs` | EP-001 |
| ChronoSentiment Personal — Investment Outcome recording (Critical) | Resolved — `adapters/chronosentiment/src/workspace.rs` | EP-001 |
| ChronoSentiment Personal — Quarterly Research Review (High) | Resolved — `adapters/chronosentiment/src/learning.rs` | EP-001 |
| ChronoSentiment Personal — Personal Investment Learning Loop (High) | Resolved — `adapters/chronosentiment/src/learning.rs` | EP-001 |

**Deferred:**
- GAP-UC-007: `coralys-scheduler` determinism test not wired — deferred to P-002 (pre-existing gap, not introduced by EP-001)

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

| Gap | Severity | Status | Description | Blocking |
|-----|----------|--------|-------------|----------|
| **Intent primitive** | High | Open | No standalone `Intent` trait at the platform level. Intent is implicit in `Scenario` configuration. Without a formal Intent primitive, the platform cannot enforce the invariant "every Workspace has exactly one Intent". | Platform invariant enforcement |
| **Context primitive** | Medium | Open | No `Context` primitive implemented. Operational context is embedded in `Scenario` configuration. Without a formal Context primitive, context cannot be versioned, queried, or tracked independently. | Knowledge Graph completeness |
| **Actor primitive** | Medium | Open | No standalone `Actor` trait at the platform level. Actor identity is carried through `Worker::id()` in the planning domain. Without a formal Actor primitive, actor-level provenance is not tracked across all domains. | Cross-domain provenance |
| **Hypothesis versioning** | High | Open | `DecisionProposal` does not support versioning at the platform level. Hypothesis versioning is implemented in the ChronoSentiment adapter (`hypothesis.rs`) but not yet formalised as a platform primitive. | Investment Thesis versioning; Roster Strategy versioning |
| **Evidence immutability enforcement** | High | Open | Evidence immutability is enforced in the ChronoSentiment adapter (`evidence.rs`) but not at the platform level. Other adapters could mutate historical evidence without the platform detecting it. | Platform invariant enforcement |
| **Workspace lifecycle enforcement** | Medium | Open | Workspace state transitions (Active → Completed → Archived) are not enforced at the platform level. Each adapter manages its own lifecycle. | Governance |
| **Outcome-Workspace ownership enforcement** | Medium | Open | The invariant "every Outcome belongs to exactly one Workspace" is enforced in the ChronoSentiment adapter but not at the platform level. | Platform invariant enforcement |
| **Pattern maturity model** | Medium | ✅ Resolved (EP-001) | `PatternMaturity` lifecycle (Candidate → Observed → Repeated → Validated) implemented in `decision_intelligence.rs` (UltraCrew) and `learning.rs` (ChronoSentiment). Platform-level promotion pending. | Knowledge Graph completeness |
| **Knowledge Graph persistence** | High | Open | The Knowledge Graph (`coralys-ecology`) has trait-level foundations (`MemoryModel`, `TopologyModel`) but no persistence layer. Patterns and knowledge are lost between sessions. | All products |
| **Knowledge Graph traversal** | High | Open | No traversal capability in the Knowledge Graph. Cannot query relationships between entities. | All products |
| **Knowledge Graph semantic retrieval** | Medium | Open | No semantic retrieval (similarity search, contextual retrieval) in the Knowledge Graph. | v2.0 Knowledge Graph Services |
| **Review primitive** | Medium | Open | No structured `Review` primitive at the platform level. Review is currently implemented as `DecisionPlugin::evaluate()` — a function call, not a structured record with attendees, outcomes, and conditions. | Committee Review; Quarterly Research Review |

---

## UltraCrew Gaps

### Critical Gaps (block MVP)

| Gap | Status | Description | Implementing crate |
|-----|--------|-------------|-------------------|
| **Disruption recovery workflow** | ✅ Resolved (EP-001) | End-to-end workflow implemented in `adapters/ultracrew/src/disruption_recovery.rs`. Disruption recorded → re-optimisation triggered → recovery options ranked → recovery accepted. | `adapters/ultracrew` |
| **Disruption evidence recording** | ✅ Resolved (EP-001) | Disruptions recorded as evidence items in the Scheduling Workspace. Wired in `disruption_recovery.rs`. | `adapters/ultracrew` |

### High Gaps (required for commercial launch)

| Gap | Status | Description | Implementing crate |
|-----|--------|-------------|-------------------|
| **Workforce Operations Learning Loop workflow** | ✅ Resolved (EP-001) | End-to-end workflow implemented in `adapters/ultracrew/src/decision_intelligence.rs`. Cycle completed → outcomes reviewed → patterns identified → insights added to Knowledge Graph. | `adapters/ultracrew` |
| **Recovery option ranking** | ✅ Resolved (EP-001) | Recovery options ranked by feasibility then impact score in `disruption_recovery.rs`. | `adapters/ultracrew` |
| **Operational Knowledge Graph persistence** | Open | The Knowledge Graph has trait-level foundations but no persistence. Operational patterns are lost between scheduling cycles. | `coralys-ecology` |
| **Workspace status transitions** | Open | Scheduling Workspace status transitions (Active → Completed → Archived) are not enforced. | `adapters/ultracrew` |

### Medium Gaps (required for scale)

| Gap | Status | Description | Implementing crate |
|-----|--------|-------------|-------------------|
| **Timeline filtering** | Open | The Scheduling Timeline cannot be filtered by type, date range, or crew member. | `adapters/ultracrew` |
| **Cycle review report** | ✅ Resolved (EP-001) | `CycleReviewReport` struct implemented in `decision_intelligence.rs`. | `adapters/ultracrew` |
| **Pattern accumulation** | Open | Workforce behaviour patterns are not yet accumulated across scheduling cycles in a queryable form. | `coralys-ecology` |

### Deferred

| Gap | Status | Description | Target |
|-----|--------|-------------|--------|
| **coralys-scheduler determinism test** | Deferred → P-002 | `coralys-scheduler` determinism test not wired. `deterministic_rng` field in `ScheduleOptimizer` declared but never wired into the engine. Pre-existing gap; not introduced by EP-001. | P-002 |

---

## ChronoSentiment Enterprise Gaps

**Overall status (v1.0):** The ChronoSentiment adapter (`adapters/chronosentiment`) was an empty stub. All Enterprise capabilities were documented in the blueprint but not yet implemented.

**EP-001 update:** The shared adapter foundation (evidence, hypothesis, timeline, workspace, learning) is now implemented and shared with the Personal product. Enterprise-specific wiring (committee review, organisational learning loop, institutional KG) remains pending.

### Critical Gaps (block MVP)

| Gap | Status | Description | Implementing crate |
|-----|--------|-------------|-------------------|
| **Decision Workspace** | Open | No Enterprise-specific implementation. The shared `InvestmentWorkspace` exists but Enterprise committee review and multi-stakeholder workflows are not wired. | `adapters/chronosentiment` |
| **Investment Thesis with versioning** | Open | Shared foundation implemented (EP-001). Enterprise-specific wiring (committee approval, version governance) pending. | `adapters/chronosentiment` |
| **Evidence management** | Open | Shared foundation implemented (EP-001). Enterprise-specific wiring (regulatory evidence tagging, AI conversation capture) pending. | `adapters/chronosentiment` |
| **Decision Timeline** | Open | Shared foundation implemented (EP-001). Enterprise-specific wiring pending. | `adapters/chronosentiment` |
| **Decision Outcome recording** | Open | Shared foundation implemented (EP-001). Enterprise-specific outcome types (committee decision, regulatory outcome) pending. | `adapters/chronosentiment` |

### High Gaps (required for commercial launch)

| Gap | Status | Description | Implementing crate |
|-----|--------|-------------|-------------------|
| **Committee Review workflow** | Open | No implementation. Structured committee review with attendees, discussion, decision, and conditions does not exist. | `adapters/chronosentiment` |
| **Organisational Decision Learning Loop** | Open | No implementation. The post-decision review process does not exist. | `adapters/chronosentiment` |
| **Institutional Decision Knowledge Graph** | Open | No implementation. The `MemoryModel` foundation exists in `coralys-ecology` but is not wired to the ChronoSentiment adapter. | `adapters/chronosentiment` |

### Medium Gaps (required for scale)

| Gap | Status | Description | Implementing crate |
|-----|--------|-------------|-------------------|
| **AI conversation documentation** | Open | No implementation. AI research conversations cannot be captured as evidence items. | `adapters/chronosentiment` |
| **Regulatory compliance reporting** | Open | No implementation. Automated AI documentation reports for FCA, SEC, EU AI Act do not exist. | `adapters/chronosentiment` |

---

## ChronoSentiment Personal Gaps

**Overall status (v1.0):** The ChronoSentiment adapter (`adapters/chronosentiment`) was an empty stub. All Personal capabilities were documented in the blueprint but not yet implemented.

**EP-001 update:** Personal adapter foundation fully implemented — evidence, hypothesis, timeline, workspace, and learning modules delivered with 22 tests. All Critical and High gaps resolved.

### Critical Gaps (block MVP)

| Gap | Status | Description | Implementing crate |
|-----|--------|-------------|-------------------|
| **Research Workspace** | ✅ Resolved (EP-001) | Implemented in `adapters/chronosentiment/src/workspace.rs`. | `adapters/chronosentiment` |
| **Research Dossier** | ✅ Resolved (EP-001) | Structured research records implemented in the workspace module. | `adapters/chronosentiment` |
| **Investment Thesis with versioning** | ✅ Resolved (EP-001) | Implemented in `adapters/chronosentiment/src/hypothesis.rs`. | `adapters/chronosentiment` |
| **Research Timeline** | ✅ Resolved (EP-001) | Implemented in `adapters/chronosentiment/src/timeline.rs`. | `adapters/chronosentiment` |
| **Investment Outcome recording** | ✅ Resolved (EP-001) | Outcome recording implemented in `adapters/chronosentiment/src/workspace.rs`. | `adapters/chronosentiment` |

### High Gaps (required for commercial launch)

| Gap | Status | Description | Implementing crate |
|-----|--------|-------------|-------------------|
| **Quarterly Research Review** | ✅ Resolved (EP-001) | Implemented in `adapters/chronosentiment/src/learning.rs`. | `adapters/chronosentiment` |
| **Personal Investment Learning Loop** | ✅ Resolved (EP-001) | End-to-end learning loop implemented in `adapters/chronosentiment/src/learning.rs`. | `adapters/chronosentiment` |
| **Personal Investment Knowledge Graph** | Open | `MemoryModel` foundation exists in `coralys-ecology`; adapter wiring not yet complete. | `adapters/chronosentiment` |

### Medium Gaps (required for scale)

| Gap | Status | Description | Implementing crate |
|-----|--------|-------------|-------------------|
| **AI conversation documentation** | Open | No implementation. AI research conversations cannot be captured as research sources. | `adapters/chronosentiment` |
| **Research quality scoring** | Open | No implementation. v2.0 candidate. | `adapters/chronosentiment` |

---

## Gap Summary by Product

### v1.0 Baseline (for reference)

| Product | Critical | High | Medium | Low | Total |
|---------|----------|------|--------|-----|-------|
| Platform (all products) | 0 | 4 | 8 | 0 | 12 |
| UltraCrew | 2 | 4 | 3 | 0 | 9 |
| ChronoSentiment Enterprise | 5 | 3 | 2 | 0 | 10 |
| ChronoSentiment Personal | 5 | 3 | 2 | 0 | 10 |
| **Total** | **12** | **14** | **15** | **0** | **41** |

### v2.0 Post EP-001

| Product | Critical Open | High Open | Medium Open | Deferred | Resolved (EP-001) | Total Open |
|---------|--------------|-----------|-------------|----------|-------------------|------------|
| Platform (all products) | 0 | 3 | 6 | 0 | 3 | 9 |
| UltraCrew | 0 | 2 | 2 | 1 | 5 | 4 |
| ChronoSentiment Enterprise | 5 | 3 | 2 | 0 | 0 | 10 |
| ChronoSentiment Personal | 0 | 1 | 2 | 0 | 7 | 3 |
| **Total** | **5** | **9** | **12** | **1** | **15** | **26** |

---

## Implementation Priority Order (Updated Post EP-001)

**Phase 1 — UltraCrew MVP completion** ✅ Substantially complete
- Disruption recovery workflow ✅
- Disruption evidence recording ✅
- Workforce Operations Learning Loop workflow ✅
- Recovery option ranking ✅
- Cycle review report ✅
- Remaining: KG persistence, Workspace status transitions, Timeline filtering, Pattern accumulation

**Phase 2 — Platform primitive formalisation** (Next priority)
1. Hypothesis versioning at platform level (High) — adapter implementation exists; promote to platform
2. Evidence immutability enforcement at platform level (High) — adapter implementation exists; promote to platform
3. Knowledge Graph traversal (High)
4. Intent primitive (High)
5. Knowledge Graph persistence (High)
6. Review primitive (Medium)
7. Context primitive (Medium)
8. Actor primitive (Medium)
9. Workspace lifecycle enforcement (Medium)

**Phase 3 — ChronoSentiment Personal adapter** ✅ Complete (EP-001)
- Research Workspace ✅
- Investment Thesis with versioning ✅
- Evidence management ✅
- Research Timeline ✅
- Outcome recording ✅
- Quarterly Research Review ✅
- Personal Investment Learning Loop ✅
- Remaining: KG wiring, AI conversation documentation

**Phase 4 — ChronoSentiment Enterprise adapter** (Next after Phase 2)
1. Decision Workspace Enterprise wiring (Critical)
2. Committee Review workflow (High)
3. Organisational Decision Learning Loop (High)
4. Institutional Decision Knowledge Graph (High)

**Phase 5 — Platform completeness (P-002)**
- Knowledge Graph persistence (`coralys-ecology`)
- coralys-scheduler determinism test (GAP-UC-007, deferred)

---

*Coralys Platform Implementation Gap Register v2.0 | July 2026 | Status: Updated — EP-001 resolutions applied*
*Previous version: v1.0 Baseline (2026-07-26)*
*Single consolidated gap register from all Baseline documents.*
*Review trigger: Sprint completion; implementation status change; new baseline document added.*