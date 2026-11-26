# Coralys Commercial Evidence Register v1.0

**Coralys Workforce Solutions — Commercial Evidence Register**

*Applies to all Solution Engines: UltraCrew · UltraRoster · UltraShift · UltraRail · UltraField*

---

## Purpose

This register is the single source of truth for all external commercial interactions across the Coralys platform.

Every session — whether a WOA diagnostic, a WDX demonstration, or a pilot conversation — is logged here, regardless of which Solution Engine was demonstrated. The register drives the roadmap for both the platform and individual solutions. Nothing enters any backlog unless it appears in this register.

---

## Evidence Categories

All feedback and observations are classified into one of three categories before being recorded.

| Category | Definition | Examples |
|----------|------------|---------|
| **Product Evidence** | Observations about the software itself — what worked, what failed, what confused | UI friction, missing features, performance issues, data accuracy questions |
| **Commercial Evidence** | Observations about the sales and engagement process | Objections raised, questions not answered, pricing reactions, procurement concerns |
| **Strategic Evidence** | Observations about market fit, positioning, and competitive context | Comparisons to alternatives, regulatory requirements, organisational readiness signals |

**Classification rule:** If you are unsure which category applies, ask: *"Would fixing this require a code change?"* If yes → Product. If no, ask: *"Would fixing this require a different conversation?"* If yes → Commercial. If neither → Strategic.

---

## Evidence Hierarchy

When interpreting session data, weight evidence according to this hierarchy. Prioritise what customers actually did and said over internal assumptions.

| Evidence type | Weight |
|---------------|--------|
| Behaviour observed during a demo | High |
| Direct quote from a customer | High |
| Requested capability | Medium |
| Facilitator interpretation | Low |
| Internal opinion | Lowest |

**Principle:** If a roadmap decision rests primarily on facilitator interpretation or internal opinion, it requires corroboration from at least one High-weight evidence item before it is actioned.

---

## Evidence Scope Classification

Every evidence item is classified by scope before it enters the roadmap. This prevents domain-specific requests from polluting the platform backlog, and platform gaps from being buried in solution-specific feedback.

| Scope | Description | Examples |
|-------|-------------|---------|
| **Platform** | Applies to Coralys itself — optimisation engine, explainability, reporting, APIs, performance, security | "Need faster schedule generation", "Need audit trail for all decisions", "Need REST API for integration" |
| **Solution** | Applies only to a specific Solution Engine | "Need ICU nurse certification rules" (UltraRoster), "Need DGCA duty-time rules" (UltraCrew) |

**Classification rule:** Ask — *"Would this change affect every Solution Engine, or only one?"* If every engine → Platform. If one engine → Solution (note which).

---

## v1.1 Decision Rules

These rules govern how evidence translates into roadmap decisions.

1. **2+ organisations before it becomes a v1.1 candidate.** A capability requested by a single organisation is recorded but not prioritised. A capability raised by two or more organisations is a v1.1 candidate.

2. **Exception — critical compliance or safety requirements.** A regulatory or safety capability that is mandatory to operate in a target market can bypass the 2+ organisations rule. Record the requirement, the market, and the regulatory basis before actioning.

3. **Fix workflow friction before adding features.** If a step in the demo or pilot process causes repeated confusion, that is addressed before new capabilities are added.

4. **Refine messaging before expanding product.** If business value is questioned, the response is to improve how value is communicated — not to add more features.

5. **Roadmap is driven by this register, not by internal assumptions.** No v1.1 item is added without a corresponding entry in this register.

---

## Session Template

Copy this template for every external session. One entry per session.

---

### Session [N] — [Organisation Name]

**Date:** YYYY-MM-DD
**Facilitator:** [Name]
**Attendees:** [Names and roles]
**Session type:** WOA Diagnostic / WDX Demonstration / Pilot Conversation / Combined
**Solution:** UltraCrew / UltraRoster / UltraShift / UltraRail / UltraField / Other
**Domain:** Airline / Healthcare / Manufacturing / Rail / Logistics / Other

---

#### WOA Assessment

| Dimension | Observed tier | Notes |
|-----------|--------------|-------|
| Data availability | Reactive / Managed / Optimised / Adaptive | |
| Decision process | Reactive / Managed / Optimised / Adaptive | |
| Tooling maturity | Reactive / Managed / Optimised / Adaptive | |
| Organisational readiness | Reactive / Managed / Optimised / Adaptive | |

**Overall WOA tier:** [Reactive / Managed / Optimised / Adaptive]

---

#### WDX Demonstration

| Step | Completed | Notes |
|------|-----------|-------|
| 1. Baseline establishment | Yes / No | |
| 2. Divergence identification | Yes / No | |
| 3. Decision proposal | Yes / No | |
| 4. Outcome simulation | Yes / No | |
| 5. Confidence scoring | Yes / No | |
| 6. Commercial evidence capture | Yes / No | |

**Demo completion:** Full / Partial / Not reached
**Reason if partial:** [Note]

---

#### Customer Confidence

Rate each dimension 1–5 after the session. A customer may not be ready for a pilot but still rate operational credibility highly — record both.

| Dimension | Rating (1–5) | Notes |
|-----------|-------------|-------|
| Operational credibility | | |
| Decision transparency | | |
| Overall confidence | | |

**Rating guide:** 1 = No confidence / 2 = Sceptical / 3 = Neutral / 4 = Confident / 5 = Strong confidence

---

#### Objection Status

Record every objection raised during the session and its resolution status.

| Objection | Addressed during meeting | Requires follow-up | Product change required |
|-----------|------------------------|-------------------|------------------------|
| [Objection 1] | Yes / No | Yes / No | Yes / No |
| [Objection 2] | Yes / No | Yes / No | Yes / No |
| [Objection 3] | Yes / No | Yes / No | Yes / No |

**Review unresolved objections before the next engagement with this organisation.**

---

#### Feedback Classification

| Item | Category | Scope | Severity | v1.1 candidate? |
|------|----------|-------|----------|----------------|
| [Observation 1] | Product / Commercial / Strategic | Platform / Solution | High / Medium / Low | Yes / No |
| [Observation 2] | Product / Commercial / Strategic | Platform / Solution | High / Medium / Low | Yes / No |
| [Observation 3] | Product / Commercial / Strategic | Platform / Solution | High / Medium / Low | Yes / No |

---

#### Pilot Readiness

| Question | Answer |
|----------|--------|
| Decision maker identified? | Yes / No / Unknown |
| Budget cycle known? | Yes / No / Unknown |
| Procurement process understood? | Yes / No / Unknown |
| Pilot scope agreed? | Yes / No / Not yet |
| Timeline discussed? | Yes / No / Not yet |

**Pilot readiness assessment:** Ready / Promising / Early / Not suitable

---

#### Agreed Next Steps

| Action | Owner | Due date |
|--------|-------|----------|
| [Action 1] | [Name] | YYYY-MM-DD |
| [Action 2] | [Name] | YYYY-MM-DD |

---

#### Facilitator Notes

*Free text. Record anything that does not fit the structured fields above — tone of the meeting, unexpected questions, moments of genuine interest, concerns not raised directly.*

---

## Aggregate KPI Dashboard

Update this section after every session.

| KPI | Target | Actual |
|-----|--------|--------|
| WOA assessments completed | 10 | 0 |
| WDX demonstrations completed | 10 | 0 |
| Pilot opportunities identified | 3 | 0 |
| Demo completion rate | >90% | — |
| Repeat issues (2+ sessions) | Tracked | 0 |

### Commercial Funnel

Track every opportunity from first contact to signed subscription. If many prospects complete WOA but few accept a paid assessment, the commercial offer — not the technology — needs work.

| Stage | Count |
|-------|------:|
| Initial contact | |
| Discovery completed | |
| WOA completed | |
| WDX demonstrated | |
| Assessment proposal sent | |
| Assessment won | |
| Pilot proposal sent | |
| Pilot won | |
| Subscription signed | |

**Conversion to watch:** WOA completed → Assessment won. A drop here indicates a pricing or value-communication problem, not a product problem.

---

## Roadmap Summary

Update after every session. After ten demonstrations, this gives an immediate overview of where momentum is.

| Status | Count |
|--------|------:|
| Product evidence items | |
| Commercial evidence items | |
| Strategic evidence items | |
| v1.1 candidates | |
| Deferred ideas | |

---

## Recurring Observations

When the same issue appears in two or more sessions, it is recorded here and becomes a v1.1 candidate.

| Observation | Sessions | Category | v1.1 candidate |
|-------------|----------|----------|---------------|
| [Issue] | [Session numbers] | Product / Commercial / Strategic | Yes |

---

## Deferred Ideas

Requests raised by a single organisation that are recorded but not prioritised.

| Idea | Organisation | Session | Category | Notes |
|------|-------------|---------|----------|-------|
| [Idea] | [Org] | [N] | Product / Commercial / Strategic | [Note] |

---

*Register opened: [Date]*
*Last updated: [Date]*
*Maintained by: [Name]*
