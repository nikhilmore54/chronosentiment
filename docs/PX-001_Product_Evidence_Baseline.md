# PX-001 — Product Evidence Baseline

**Document type:** Milestone
**Document ID:** PX-001
**Version:** 1.0
**Status:** Active
**Date:** 2026-07-27
**Owner:** Product / Engineering Leadership

---

## Document Governance

| Field | Value |
|-------|-------|
| Document Status | Active v1.0 |
| Review Trigger | Stream milestone reached; evidence record added; kill criterion met |

**Relationship to other documents:**
- Governed by: `strategy/CS-S-001_Product_First_Governance_Principle.md` (product-first principle)
- Preceded by: `EP-001_MILESTONE.md` (platform implementation baseline)
- Informs: `EL-001_Phase1B_Evidence_Ledger.md` (evidence records)
- Informs: `EP-002_ROADMAP.md` v2.0 (platform consolidation triggers)
- Informs: `CV-001_Commercial_Validation_Playbook.md` (ChronoSentiment Enterprise validation)
- Informs: `P001_PILOT_RUNBOOK.md` (UltraCrew SunAir pilot)

---

## Purpose

PX-001 marks the transition from platform implementation to product evidence generation.

Up to EP-001, success was measured by building a technically coherent platform. From PX-001 onward, success is measured by whether the products generate convincing operational and commercial evidence.

> **The governance programme is frozen.** No new governance documents, architectural redesigns, or speculative platform primitives are to be introduced during PX-001. Platform work is limited to consolidation justified by demonstrated product need.

---

## Milestone Definition

PX-001 is complete when all four streams have met their success criteria below. Streams run in parallel. Stream 4 (Coralys) has no active deliverables — its criterion is a constraint.

---

## Stream 1 — UltraCrew

**Objective:** Produce operational evidence.

**Sprint sequence:**

1. Freeze UltraCrew MVP baseline — no new features during pilot
2. Execute SunAir pilot scenarios (see `docs/P001_PILOT_RUNBOOK.md`)
3. Collect operational evidence across all measurement dimensions
4. Produce the first Operational Evidence Report

**Measurement dimensions:**

| Dimension | What is being measured | Evidence type |
|-----------|----------------------|---------------|
| Disruption recovery time | Time from disruption event to accepted recovery plan | Operational |
| Planner effort | Manual interventions required per scheduling cycle | Operational |
| Roster quality | Coverage rate; constraint violation rate | Operational |
| Recommendation acceptance | % of learning loop recommendations accepted by planners | Operational |
| Explanation usefulness | Dispatcher rating of explanation quality | Qualitative |

**Success criteria:**

| Criterion | Definition |
|-----------|-----------|
| Pilot executed | SunAir pilot scenarios completed per `P001_PILOT_RUNBOOK.md` |
| Operational evidence collected | At least one evidence record per measurement dimension |
| Dispatcher feedback documented | Structured feedback from at least one dispatcher |
| Initial ROI measured | Disruption recovery time delta documented |

**Evidence destination:** `EL-001_Phase1B_Evidence_Ledger.md` — new entries under UltraCrew operational evidence.

**Capability maturity target:** Move UltraCrew disruption recovery and operational learning loop from L1 (Implemented) to L2 (Demonstrated).

---

## Stream 2 — ChronoSentiment Enterprise

**Objective:** Validate H1–H7 through structured customer discovery.

**Approach:** Build the minimum workflow needed to let investment teams experience Evidence → Thesis → Timeline → Review → Learning. Then conduct structured discovery. Do not add features ahead of evidence.

**Workflow under test:**

```
Evidence capture
      ↓
Thesis formation
      ↓
Timeline construction
      ↓
Review session
      ↓
Learning extraction
```

**Evidence accumulation rule:** Every customer interaction must produce an evidence record that does one of:

- Strengthens a hypothesis (confidence increases)
- Weakens a hypothesis (confidence decreases)
- Refines a hypothesis (scope or framing changes)
- Invalidates a hypothesis (kill criterion triggered)

**Success criteria:**

| Criterion | Definition |
|-----------|-----------|
| Design partners engaged | At least one investment team in structured discovery |
| H1–H7 evidence ledger populated | At least one evidence record per hypothesis |
| Commercial validation underway | CV-001 playbook active; evidence acquisition in progress |

**Evidence destination:** `EL-001_Phase1B_Evidence_Ledger.md` — H1–H7 evidence records per `CV-001_Commercial_Validation_Playbook.md`.

**Kill criteria:** Defined in `CV-001_Commercial_Validation_Playbook.md`. If kill criteria are met, PX-001 Stream 2 closes with a go/no-go decision rather than a success record.

---

## Stream 3 — ChronoSentiment Personal

**Objective:** Validate the individual investor workflow.

**Workflow under test:**

```
Evidence
      ↓
Thesis
      ↓
Research
      ↓
Review
      ↓
Learning
```

**Questions to answer:**

| Question | What it reveals |
|----------|----------------|
| Where do users abandon the workflow? | Drop-off points and friction |
| Which actions become habitual? | Retention drivers |
| Which capabilities motivate an upgrade to Enterprise? | Upgrade path validation |

**Success criteria:**

| Criterion | Definition |
|-----------|-----------|
| Prototype workflow exercised | At least one user has completed the full Evidence → Learning loop |
| User journey observed | Drop-off points documented |
| Adoption friction documented | At least three friction points identified and categorised |

**Evidence destination:** `EL-001_Phase1B_Evidence_Ledger.md` — ChronoSentiment Personal workflow evidence.

---

## Stream 4 — Coralys Platform

**Objective:** Maintain the constraint. No speculative work.

**This stream has no active deliverables.** Its success criterion is a constraint on what does not happen.

**Success criteria:**

| Criterion | Definition |
|-----------|-----------|
| Zero speculative abstractions | No new platform primitive introduced without two-product evidence trigger |
| No large redesign | No architectural redesign initiated during PX-001 |
| Consolidation justified | Any platform consolidation performed is traceable to a demonstrated product need |

**Permitted platform work during PX-001:**

- Bug fixes and performance improvements in existing crates
- Consolidation of a primitive where two products have independently demonstrated the same semantics (per CS-S-001 two-product rule)
- Developer experience improvements that reduce friction for product teams

**Not permitted during PX-001:**

- New platform primitives introduced speculatively
- EP-002 consolidation work that is not triggered by product evidence
- Architectural redesign of existing stable crates

---

## Milestone Completion

PX-001 is complete when all three active streams (1, 2, 3) have met their success criteria and Stream 4 constraint has been maintained throughout.

**Completion produces:**

| Output | Destination |
|--------|-------------|
| UltraCrew Operational Evidence Report | `EL-001_Phase1B_Evidence_Ledger.md` |
| ChronoSentiment Enterprise H1–H7 evidence records | `EL-001_Phase1B_Evidence_Ledger.md` |
| ChronoSentiment Personal workflow friction report | `EL-001_Phase1B_Evidence_Ledger.md` |
| Platform consolidation log (if any) | `EP-002_ROADMAP.md` scope updates |

**What PX-001 completion enables:**

- EP-002 consolidation decisions based on real product evidence
- P-002 (Knowledge Graph) investment decision — justified or deferred
- Commercial go/no-go decisions for each product
- Next milestone definition based on evidence rather than plan

---

## North Star Outcome

Beyond the stream success criteria, there is one outcome that marks the moment Coralys stops being a software platform and becomes a decision-making platform with demonstrated value.

> **First Independent Customer Decision Changed Because of Coralys**

This is not a PX-001 exit criterion — it may arrive during PX-001 or after it. But it is the outcome that all three active streams are working toward.

**For UltraCrew:**
> A dispatcher accepted a recovery recommendation that they otherwise would not have made — and the outcome was better than their default decision would have been.

**For ChronoSentiment Enterprise:**
> An investment team changed or rejected an investment thesis because the evidence workflow exposed a weakness they had not previously seen.

**For ChronoSentiment Personal:**
> An individual investor revised a position or avoided a decision because the Evidence → Thesis → Review loop surfaced a contradiction in their own reasoning.

When any of these occurs, record it as an evidence entry in `EL-001_Phase1B_Evidence_Ledger.md` with type `OPS` (operational) or `INT` (customer interview). It is the strongest possible signal that the platform is creating value.

---

## Weekly Review Practice

During PX-001, the Evidence Ledger (`EL-001_Phase1B_Evidence_Ledger.md`) is the centre of weekly reviews. The review is not a feature status meeting.

**Review questions:**

| Question | Purpose |
|----------|---------|
| What evidence did we collect this week? | Confirm evidence accumulation is active |
| Which hypotheses changed? | Track confidence movement across H1–H7 and operational claims |
| Which product risks were reduced? | Measure PX-001 progress against stream success criteria |
| Which assumptions were invalidated? | Surface kill criteria early |
| Did we discover a genuine cross-product abstraction? | The only valid trigger for platform consolidation work |

If a week passes without a new evidence record, that is a signal — either execution is blocked or effort has drifted back toward non-evidence-generating work.

---

## Governance Note

If product execution exposes a genuine deficiency in the governance framework, that deficiency should be documented as a gap and addressed in a targeted update to the relevant document. It does not justify reopening the governance programme as a whole.

The governance programme is frozen. Product execution is the programme.

---

*PX-001 Product Evidence Baseline v1.0 | July 2026 | Status: Active*
*Review trigger: Stream milestone reached; evidence record added; kill criterion met.*