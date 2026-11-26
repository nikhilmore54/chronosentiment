# UltraCrew Commercial Execution Charter

> **Date**: 2026-07-28
> **Status**: Active
> **Type**: Commercial Execution Charter — Governing Document
> **Owner**: Programme Leadership
> **Predecessors**: Engineering P-001 (closed 2026-07-23), Architecture Programme (closed 2026-07-22)
> **Governed by**: [`docs/ARCHITECTURE_EVOLUTION.md`](../ARCHITECTURE_EVOLUTION.md) (constitutional baseline, frozen)

---

## Purpose

This document is the governing charter for UltraCrew commercial execution. It coordinates the existing workstreams — SunAir Operational Demonstration, CV-001 Commercial Validation, and pilot activities — under a single programme question and exit criterion.

It does not replace or rename any existing programme document. Each existing document retains its own scope and responsibility.

The architecture stream is closed. The engineering milestone is complete. The question is no longer "Can Coralys support this architecture?" It is:

> **Can UltraCrew create measurable operational value that customers are willing to adopt and pay for?**

---

## Programme Hierarchy

```
Architecture Programme (Closed 2026-07-22)
        │
        ▼
Engineering P-001 (Closed 2026-07-23)
        │
        ▼
Commercial Execution Charter ← THIS DOCUMENT
        │
        ├──────────────────────┐
        │                      │
        ▼                      ▼
SunAir Operational         CV-001 Commercial
Demonstration              Validation Playbook
(EP-002 roadmap)           (docs/CV-001_Commercial_Validation_Playbook.md)
        │                      │
        └──────────┬───────────┘
                   ▼
         Customer Pilot Evidence
                   │
                   ▼
         EP-002 Platform Consolidation
         (conditional on product evidence)
                   │
                   ▼
                 P-002
```

**Document responsibilities:**

| Document | Responsibility |
|---|---|
| [`docs/EP-002_ROADMAP.md`](../EP-002_ROADMAP.md) | Sequence of programmes; platform consolidation roadmap |
| [`docs/P001_MILESTONE.md`](../P001_MILESTONE.md) | Engineering P-001 completion record (closed) |
| This document | Governing charter for commercial execution |
| SunAir Operational Demonstration (EP-002 roadmap §P-001) | Operational validation — moves capabilities from L1 to L2 |
| [`docs/CV-001_Commercial_Validation_Playbook.md`](../CV-001_Commercial_Validation_Playbook.md) | Commercial validation methodology |
| [`docs/governance/CR-001_Constitutional_Operationalisation_Review.md`](CR-001_Constitutional_Operationalisation_Review.md) | Constitutional review (Accepted — Pending Ratification) |

---

## Exit Criterion

At least one customer agrees that the pilot delivered sufficient value to justify progressing toward commercial deployment.

The charter is complete when all of the following are satisfied:

1. A customer has participated in a structured Workforce Operations Assessment.
2. UltraCrew has optimised one or more representative customer scenarios.
3. The customer agrees the optimisation produced measurable operational improvements.
4. Decision rationale is understood and accepted by planners.
5. The pilot has generated reusable commercial evidence (KPIs, testimonials, or documented outcomes).
6. The findings identify either no architectural limitations (reinforcing the constitutional baseline) or recurring product-driven needs justifying future platform evolution.

---

## Five Streams

### Stream 1 — Customer Problem Validation (Highest Priority)

**Objective:** Ensure UltraCrew is solving the right problem before demonstrating the product.

**Deliverables:**

- Identify 5–10 target organisations (regional airlines, charter operators, helicopter operators, ground handling companies with crew scheduling needs)
- Conduct structured discovery meetings
- Document current scheduling processes
- Quantify operational pain points:
  - planning time per scheduling cycle
  - overtime cost and frequency
  - crew utilisation rate
  - disruption handling time and cost
  - compliance effort (fatigue rules, union agreements)
  - planner workload and manual intervention rate
- Produce a Workforce Operations Assessment (WOA) report for each prospect

**Success metric:** Customers recognise their own operational problems in the assessment and agree they are material.

**Governing principle:** Lead with the problem, not the product. The WOA is a diagnostic tool, not a sales pitch.

---

### Stream 2 — WDX Demonstration

**Objective:** Show that UltraCrew produces better decisions and that planners can understand and trust them.

**For each prospect, demonstrate:**

- Current schedule (their data or a representative proxy)
- Optimised schedule (UltraCrew output)
- KPI comparison (coverage, overtime, fairness, fatigue, utilisation)
- Explainability (why this assignment? what constraints were binding?)
- Trade-offs (what was sacrificed to achieve this outcome?)
- Scenario analysis (what if we change this constraint?)

**Frame demonstrations around operational outcomes, not algorithms.** Planners do not need to understand MOGA. They need to trust the recommendation and understand the reasoning.

**Existing artefacts:**
- [`fixtures/demo/sunair_explanation_engine.html`](../../fixtures/demo/sunair_explanation_engine.html) — decision factor cards, candidate ranking
- [`fixtures/demo/sunair_scenario_comparison.html`](../../fixtures/demo/sunair_scenario_comparison.html) — KPI comparison, convergence chart
- [`fixtures/demo/sunair_planner_workspace.html`](../../fixtures/demo/sunair_planner_workspace.html) — Gantt, overrides, export
- [`docs/sunair_sales_playbook.md`](../sunair_sales_playbook.md) — buyer profiles, demo flow, objection handling

**Success metric:** Planners trust both the recommendations and the reasoning behind them.

---

### Stream 3 — Pilot Readiness

**Objective:** Make deployment straightforward enough to lower the adoption barrier.

**Deliverables:**

- Pilot deployment guide (customer-facing, non-technical)
- Onboarding checklist
- Required customer data specification (format, fields, volume)
- Integration checklist (systems, APIs)
- Success metrics and acceptance criteria (agreed with customer before pilot begins)
- Rollback plan
- Support process (contact, response times, escalation)

**Existing artefacts:**
- [`docs/sunair_pilot_guide.md`](../sunair_pilot_guide.md) — operator-facing install and run guide
- [`docs/P001_PILOT_RUNBOOK.md`](../P001_PILOT_RUNBOOK.md) — 9-step operator guide with sign-off checklist
- [`adapters/ultracrew/src/strict_validator.rs`](../../adapters/ultracrew/src/strict_validator.rs) — data validation (18 tests)

**Success metric:** A non-technical operations manager can follow the onboarding checklist without engineering support.

---

### Stream 4 — Commercial Evidence

**Objective:** Build a reusable evidence repository from every pilot engagement.

**For every pilot, produce:**

- Before/after KPIs (coverage, overtime, fairness, utilisation, planning time)
- Planning effort reduction (hours saved per scheduling cycle)
- Overtime reduction (cost and frequency)
- Fairness improvement (variance in credited hours across crew)
- Utilisation improvement (crew hours used vs available)
- Customer feedback (structured debrief)
- Planner quotations (direct quotes from planners about the experience)
- Lessons learned (what worked, what didn't, what to change)

**This evidence serves multiple downstream consumers** — see Evidence Governance below.

**Success metric:** Each pilot produces a documented evidence package that can be shared (with customer permission) with future prospects.

---

### Stream 5 — Product Hardening (Engineering, Pilot-Driven Only)

**Objective:** Fix what pilots expose. Do not build what pilots have not yet requested.

**Permitted work:**

- Usability issues identified by real planners during pilot
- Performance issues that affect pilot viability
- Robustness issues (crashes, data corruption, incorrect output)
- Reporting gaps (KPIs or outputs that pilots need but the product doesn't produce)
- Explainability gaps (decisions that planners cannot understand or trust)
- Deployment issues (installation, configuration, data import)
- Observability gaps (inability to diagnose problems during pilot)

**Prohibited work:**

- Speculative platform enhancements
- New Coralys architectural abstractions
- Constitutional amendments (CR-001 ratification waits for a product trigger)
- New domain capabilities without a concrete pilot need
- Broad generalisation beyond demonstrated requirements
- Additional domain extraction

**Governing rule:** If a pilot has not exposed the need, the work does not happen.

---

## Evidence Governance

Pilot evidence has multiple downstream consumers. Each piece of evidence should be routed to the appropriate consumer rather than remaining confined to individual pilot reports.

| Evidence type | Primary consumer |
|---|---|
| Operational KPIs (before/after) | Sales, WOA, future pilots |
| Planner feedback | Product (Stream 5 backlog) |
| Deployment issues | Engineering (Stream 5) |
| Repeated capability requests | EP-002 consolidation trigger |
| Architectural constraints or limitations | CR-002 (if recurring) |
| Customer testimonials | Marketing, sales collateral |
| ROI measurements | Commercial, investor narrative |
| Implementation guidance | Pilot Readiness (Stream 3) |

**Governing principle:** Commercial evidence is a governed repository, not a collection of individual reports. Every pilot engagement should leave behind artefacts that improve the next engagement.

---

## What Waits Until After This Charter Closes

Until the exit criterion is met, the following are deferred:

- Additional Coralys architectural work
- CR-001 ratification (waits for a product-driven trigger)
- CR-002 (only if pilots expose recurring constitutional gaps)
- P-002 (only after exit criterion is met)
- EP-002 consolidation (conditional on product evidence — see [`docs/EP-002_ROADMAP.md`](../EP-002_ROADMAP.md))
- "Constitutional Operationalisation" governance category formalisation (waits for a second constitutional review exhibiting the same pattern — consistent with Principle 4)

The Constitution provides the governing rule:

> *Future architectural changes shall originate from implementation evidence, benchmark evidence, pilot evidence, or repeated product evidence.*

This charter is the programme that generates pilot evidence. Until it does, the architecture remains frozen.

---

## Success Metrics

| Area | Measure |
|---|---|
| Customer engagement | Discovery meetings completed with 5–10 target organisations |
| Problem validation | WOA accepted by customer as an accurate description of their operational situation |
| Decision quality | WDX recommendations judged credible by planners |
| Product value | Measurable operational improvement demonstrated on customer scenarios |
| Commercial traction | Pilot agreement or formal next-step commitment from at least one customer |
| Platform validation | Findings identify no architectural limitations, or recurring needs that justify future evolution |

---

## Evidence Chain Position

```
Architecture Baseline (ARCHITECTURE_EVOLUTION.md)
        │
        ▼
Repository Convergence (M-001 → M-004)
        │
        ▼
UC-ARCH-001 Credit Framework
        │
        ▼
CR-001 Constitutional Operationalisation Review
        │
        ▼
Commercial Execution Charter ← YOU ARE HERE
        │
        ▼
Pilot Evidence (customer KPIs, testimonials, documented outcomes)
        │
        ▼
CR-002 (only if pilots expose recurring constitutional gaps)
EP-002 Platform Consolidation (conditional on product evidence)
P-002 (only after exit criterion is met)
```

---

*This document is active. It is updated as pilot engagements progress and evidence accumulates.*