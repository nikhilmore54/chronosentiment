# Coralys — Product-First Governance Principle

**Document type:** Strategic Governance
**Document ID:** CS-S-001
**Version:** 1.0
**Status:** Operational
**Date:** 2026-07-27
**Owner:** Product / Engineering Leadership

---

## Document Governance

| Field | Value |
|-------|-------|
| Document Status | Operational v1.0 |
| Review Trigger | New product milestone; platform promotion decision; effort allocation review |

**Relationship to other documents:**
- Supersedes: Platform-centric framing in `EP-002_ROADMAP.md` v1.0 (see EP-002 v2.0 for updated scope)
- Informs: All future sprint planning, platform promotion decisions, EP-002 scope
- Informed by: `EP-001_MILESTONE.md` (Phase 1 completion state)
- Informs: `CS-S-002_UltraCrew_Product_Strategy_v1.0.md`, `CS-S-003_ChronoSentiment_Enterprise_Product_Strategy_v1.0.md`, `CS-S-004_ChronoSentiment_Personal_Product_Strategy_v1.0.md`

---

## Strategic Inflection Point

As of 2026-07-27, Coralys has crossed a phase boundary.

**Phase 1 (largely complete): Reduce architecture risk.**
**Phase 2 (primary effort from this point): Reduce product risk.**

These are different problems requiring different disciplines. Phase 1 is complete enough. Phase 2 begins now.

---

## Phase 1 — What Was Built (Architecture Risk Reduction)

The following platform capabilities are now established and stable:

| Capability | Status |
|------------|--------|
| Core optimisation engine (MOGA) | Stable — L2 Demonstrated |
| Domain adapter pattern | Stable |
| Traceability system | Stable |
| Governance framework | Stable |
| Evidence model | Stable — L1 Implemented |
| Workspace concepts | Stable — L1 Implemented |
| Learning model | Stable — L1 Implemented |
| Architectural boundaries | Stable |

**Stability criterion:** The platform does not need to be perfect. It needs to be stable enough that products can be built without constantly changing the underlying platform. That criterion is now met.

---

## Phase 2 — What Must Be Done (Product Risk Reduction)

Every piece of work from this point forward should answer a product question, not an architecture question.

### UltraCrew — Open Product Questions

| Question | What it validates |
|----------|-------------------|
| Does disruption recovery actually save dispatcher time? | Core value proposition |
| Does the learning loop produce insights that planners use? | Operational learning value |
| Which scheduling capabilities are customers willing to pay for? | Revenue model |
| Which explainability features build trust? | Adoption barrier |

### ChronoSentiment Enterprise — Open Product Questions

| Question | What it validates |
|----------|-------------------|
| Do organisations actually maintain evidence dossiers? | Core workflow adoption |
| Is hypothesis versioning valuable in practice? | Differentiation claim |
| Does the workspace model fit investment teams? | UX fit |
| Which workflows create measurable value? | ROI evidence |

### ChronoSentiment Personal — Open Product Questions

| Question | What it validates |
|----------|-------------------|
| Do individual investors naturally use the Evidence → Thesis → Review workflow? | Core loop adoption |
| Where do they stop using it? | Drop-off and friction points |
| Which parts become habit-forming? | Retention drivers |
| Which capabilities justify upgrading to Enterprise? | Upgrade path |

---

## The Governing Principle: Products Discover Abstractions

> **Products discover abstractions; architects consolidate them.**

This is the primary rule governing all platform evolution from this point forward.

### What this means in practice

A platform primitive should be promoted to `coralys-core` only when:

1. At least one product uses it in production or demonstrated operation, **and**
2. A second independent product demonstrably needs the same semantics.

### Applied to current candidates

| Primitive | Current state | Promotion criterion |
|-----------|--------------|---------------------|
| `Evidence` | ChronoSentiment uses it | Promote when UltraCrew operational evidence also requires it |
| `Hypothesis` | ChronoSentiment uses it | Promote when a second product demonstrates the same versioning need |
| `Intent` | Implicit in ChronoSentiment workspace | Promote when two products expose the same one-Intent-per-Workspace invariant |
| `Workspace` | Both adapters use it | Candidate — verify semantics are genuinely shared, not coincidentally similar |
| `Learning` | UltraCrew uses it | Promote when ChronoSentiment or a third product needs cross-session learning |
| `Timeline` | ChronoSentiment uses it | Promote when a second product needs temporal evidence ordering |

**The test is not "does this exist?" The test is "have two products independently discovered they need this?"**

---

## Effort Allocation (6–12 Month Horizon)

| Area | Allocation | Rationale |
|------|-----------|-----------|
| Product development and validation | 60–70% | UltraCrew, ChronoSentiment Enterprise, ChronoSentiment Personal |
| Platform consolidation | 20–30% | Extract proven abstractions; improve performance; improve developer experience |
| Research | 10–20% | Knowledge Graph, cross-workspace learning, new optimisation research, future Coralys capabilities |

**Explicitly excluded from this allocation:** Large-scale architecture redesign. The architecture's quality will now be measured by how little it needs to change as products mature.

---

## Product Portfolio Hierarchy

Two dimensions govern how the portfolio is managed. They are distinct and must not be conflated.

### Execution Priority

Execution priority determines where engineering effort is directed first. The sequence is:

```
1. UltraCrew                    — Revenue engine; pilot and commercial validation
2. ChronoSentiment Enterprise   — Second product; commercial validation
3. ChronoSentiment Personal     — Product-led growth; upgrade path to Enterprise
4. Coralys Platform             — Consolidation only; triggered by product evidence
```

This ordering reflects where product risk is highest and where customer evidence is most urgently needed. It is not a statement about strategic importance.

### Strategic Value

Strategically, Coralys is the compounding asset. Every abstraction that earns promotion to `coralys-core` reduces the cost of building the next product. The platform does not appear first in execution priority because it does not need validation — it needs products to generate the evidence that justifies its evolution.

The correct framing is:

> **The platform exists to accelerate and differentiate the products. Its strategic value compounds as more products are built on it. Its execution priority is subordinate to the products it serves.**

These two dimensions must be held simultaneously. Treating execution priority as strategic ranking would underinvest in the platform over time. Treating strategic value as execution priority would reproduce the platform drift that CS-S-001 is designed to prevent.

### Consequence for decision-making

The first question in any planning discussion is:

> **What product milestone gets us closer to a paying customer?**

If a proposed piece of work cannot be traced to a product milestone, it is not the highest execution priority. It may still have strategic value — but that value is realised through the products, not independently of them.

---

## What Success Looks Like

Success is no longer measured by platform completeness. It is measured by:

| Metric | Product |
|--------|---------|
| Dispatcher time saved per disruption event | UltraCrew |
| Planner adoption rate of learning loop recommendations | UltraCrew |
| Number of paying airline customers | UltraCrew |
| Number of design partners running evidence dossiers | ChronoSentiment Enterprise |
| H1–H7 hypothesis confidence movement | ChronoSentiment Enterprise |
| Evidence → Thesis → Review workflow completion rate | ChronoSentiment Personal |
| Personal-to-Enterprise upgrade conversion | ChronoSentiment Personal |

---

## Architectural Stability Corollary

The platform has reached the point where its quality is demonstrated by **resistance to unnecessary change**, not by continued expansion.

If repeated product development exposes awkwardness or duplication, that is the signal to refine the platform.

If it does not, resisting the temptation to redesign is a strength, not a missed opportunity.

> The platform should emerge from solving real operational problems across multiple products — not products being built to justify the platform.

---

## Relationship to EP-002

EP-002 is renamed and narrowed under this principle.

| Before (v1.0) | After (v2.0) |
|--------|-------|
| Platform Primitive Formalisation | Platform Consolidation |
| Promote Evidence, Hypothesis, Intent, Workspace-Outcome into `coralys-core` | Remove duplication that products have already exposed |
| Driven by architectural completeness | Driven by demonstrated product need |

EP-002 v2.0 reflects this narrowed scope. See [`EP-002_ROADMAP.md`](../EP-002_ROADMAP.md).

---

*Coralys Product-First Governance Principle CS-S-001 v1.0 | July 2026 | Status: Operational*
*Review trigger: New product milestone; platform promotion decision; effort allocation review.*