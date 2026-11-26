# ChronoSentiment Enterprise — Product Blueprint

**Document type:** Product Blueprint
**Version:** 1.0
**Status:** Baseline
**Date:** 2026-07-26
**Owner:** Product

---

## Document Governance

| Field | Value |
|-------|-------|
| Document Status | Baseline v1.0 |
| Review Trigger | Material change in product features, user experience, or platform integration |

**Relationship to other documents:**
- Informed by: `CS-S-003_ChronoSentiment_Enterprise_Product_Strategy_v1.0.md` (product strategy)
- Informed by: `CORALYS_PLATFORM_ARCHITECTURE.md` (platform architecture)
- Informed by: `ChronoSentiment_Product_Blueprint_v1.md` (predecessor combined blueprint)
- Informed by: `CHRONOSENTIMENT_PRD_V1.md` (product requirements)
- Informs: Engineering implementation

---

## Purpose

This document defines the product blueprint for ChronoSentiment Enterprise — the detailed product specification, user experience, and platform integration for the Financial Decision Intelligence Platform.

---

## Product Identity

**Product name:** ChronoSentiment Enterprise
**Commercial positioning:** Financial Decision Intelligence Platform
**Platform:** Coralys Knowledge Evolution Platform
**Target audience:** Institutional investment teams — asset managers, hedge funds, family offices

---

## Core User Journey

The core user journey for ChronoSentiment Enterprise follows the Knowledge Evolution Lifecycle adapted for institutional investment decisions:

```
Evidence → Thesis → Committee Review → Outcome → Organisational Learning
```

In investment management terms:

| Stage | User Action | Platform Response |
|-------|-------------|------------------|
| Evidence | Gather research — annual reports, earnings calls, AI conversations, market data | Structured, time-stamped evidence record created in Decision Workspace |
| Thesis | Articulate investment thesis with assumptions and risks | Versioned Investment Thesis created; linked to evidence |
| Committee Review | Present thesis to investment committee; record review outcome | Committee Review Record created; thesis revised or confirmed |
| Outcome | Record investment outcome — what actually happened vs. what was predicted | Decision Outcome recorded; learning loop triggered |
| Organisational Learning | Review completed decision cycles; validate patterns | Organisational Decision Patterns surfaced; Institutional Decision Knowledge Graph updated |

---

## Primary User Personas

### Persona 1 — The Portfolio Manager

**Name:** James
**Role:** Portfolio Manager, mid-size asset manager (£2B AUM)
**Problem:** James makes 4–6 investment decisions per week. His research is scattered across email, Bloomberg, and a shared drive. When a position is reviewed six months later, he cannot reconstruct the original reasoning. When a junior analyst asks why the firm holds a position, James has to reconstruct the thesis from memory.

**What ChronoSentiment Enterprise does for James:** Every investment decision has a Decision Workspace. James captures research as evidence, articulates his thesis, records the committee review, and tracks the outcome. Six months later, the full decision history is searchable and auditable.

---

### Persona 2 — The CIO

**Name:** Priya
**Role:** Chief Investment Officer, independent asset manager (£1.5B AUM)
**Problem:** Priya is accountable to LPs and regulators for the firm's investment decisions. She cannot demonstrate a structured, auditable decision process. When a regulator asks about AI usage in investment decisions, she has no documentation. When a senior analyst leaves, years of institutional knowledge leave with them.

**What ChronoSentiment Enterprise does for Priya:** The Institutional Decision Knowledge Graph accumulates across all decisions. Priya can demonstrate a structured, auditable decision process to LPs and regulators. When a senior analyst leaves, their decision history remains in the system. The firm's institutional knowledge is preserved.

---

### Persona 3 — The Investment Analyst

**Name:** Marcus
**Role:** Investment Analyst, hedge fund (£800M AUM)
**Problem:** Marcus spends 60% of his time on research that has already been done by colleagues. The firm has no searchable record of past research. When Marcus starts a new thesis, he cannot find out whether the firm has covered this company before, what the previous thesis was, or what happened to the position.

**What ChronoSentiment Enterprise does for Marcus:** The Institutional Decision Knowledge Graph links companies, sectors, and investment theses. Marcus can search the firm's decision history before starting new research. He can see what the firm believed about a company two years ago, what changed, and what the outcome was.

---

## Feature Specifications

### Decision Workspace

**Purpose:** Structured environment for a single investment decision — from thesis formation through committee review to outcome recording.

**Core behaviour:**
- Each Decision Workspace is linked to a specific investment opportunity (the Subject)
- Research evidence is captured within the Workspace with timestamps and source records
- The Workspace persists from thesis creation through execution and outcome recording
- Workspaces accumulate into the Institutional Decision Knowledge Graph over time

**Fields:**
| Field | Type | Description |
|-------|------|-------------|
| Investment Opportunity | Text | The company, asset, or opportunity being evaluated |
| Fund / Mandate | Text | The fund or mandate context for this decision |
| Investment Thesis | Object | The active thesis for this Workspace |
| Thesis Versions | List | All thesis versions created in this Workspace |
| Evidence Items | List | Time-stamped evidence items (research, data, AI conversations) |
| Committee Reviews | List | All committee reviews conducted for this Workspace |
| Outcome | Object | The recorded investment outcome |
| Created | Date | Date the Workspace was opened |
| Last Updated | Date | Date the Workspace was last updated |
| Status | Enum | Active / Under Review / Decided / Monitoring / Closed |

---

### Investment Thesis

**Purpose:** Structured hypothesis management — what the team believes, why, and what would change their mind.

**Core behaviour:**
- Each Investment Thesis is a structured hypothesis about an investment opportunity
- Theses are versioned — each revision is timestamped and linked to the evidence that caused it
- Assumptions and risks are explicitly documented
- The thesis is linked to the evidence that supports it
- Thesis versions feed the Organisational Decision Learning Loop

**Fields:**
| Field | Type | Description |
|-------|------|-------------|
| Title | Text | Brief title of the investment thesis |
| Thesis Statement | Text | The core investment belief (e.g. "HDFC Bank is undervalued relative to its long-term earnings power") |
| Assumptions | List | Explicit assumptions the thesis depends on |
| Risks | List | Risks that could invalidate the thesis |
| Evidence | List | Linked evidence items that support the thesis |
| Version | Integer | Thesis version number (v1, v2, v3...) |
| Version Notes | Text | What changed in this version and why |
| Status | Enum | Draft / Under Review / Approved / Rejected / Monitoring / Closed |
| Created | Date | Date this version was created |
| Created By | Actor | The portfolio manager or analyst who created this version |

---

### Evidence Management

**Purpose:** Structured, immutable record of all research that informs an investment decision.

**Core behaviour:**
- Evidence is attached to a Decision Workspace with a timestamp and source record
- Evidence is immutable once recorded — it cannot be revised to match a later thesis
- Evidence types are configurable by the firm (annual reports, earnings calls, AI conversations, market data, expert calls, news)
- Evidence items are linked to the thesis versions they informed
- Evidence accumulates in the Institutional Decision Knowledge Graph

**Evidence types:**
| Type | Description |
|------|-------------|
| Annual Report | Company annual report or 10-K |
| Earnings Call | Earnings call transcript or notes |
| AI Conversation | Documented AI research conversation |
| Market Data | Price, volume, or financial data |
| Expert Call | Expert network call notes |
| News | News article or press release |
| Internal Research | Internal analyst note or model |
| Regulatory Filing | SEC, FCA, or other regulatory filing |

---

### Committee Review

**Purpose:** Structured, documented investment committee review process.

**Core behaviour:**
- A Committee Review is triggered when a thesis is ready for committee consideration
- The review records who attended, what was discussed, what was decided, and what conditions were set
- The review outcome is linked to the thesis version that was reviewed
- Reviews are timestamped and immutable once recorded
- Reviews feed the Decision Timeline and the Organisational Decision Learning Loop

**Fields:**
| Field | Type | Description |
|-------|------|-------------|
| Review Date | Date | Date of the committee review |
| Attendees | List | Investment committee members who attended |
| Thesis Version | Integer | The thesis version reviewed |
| Discussion Summary | Text | Summary of the committee discussion |
| Decision | Enum | Approved / Rejected / Deferred / Conditional |
| Conditions | List | Conditions attached to an approval or deferral |
| Next Review | Date | Date of the next scheduled review (if applicable) |
| Recorded By | Actor | The person who recorded the review |

---

### Decision Timeline

**Purpose:** Chronological view of how an investment decision evolved.

**Core behaviour:**
- The Decision Timeline is a chronological feed of all evidence items, thesis versions, committee reviews, and outcomes for a Decision Workspace
- Items are filterable by type, date range, and actor
- The timeline provides a complete audit trail of every decision
- Timeline items link back to their source evidence or thesis version

**Timeline item types:**
| Type | Description |
|------|-------------|
| Evidence Captured | New evidence added to the Workspace |
| Thesis Created | A new thesis version was created |
| Thesis Revised | An existing thesis was revised |
| Review Scheduled | A committee review was scheduled |
| Review Completed | A committee review was completed |
| Decision Made | An investment decision was made |
| Position Opened | A position was opened |
| Position Monitored | A monitoring note was added |
| Outcome Recorded | An investment outcome was recorded |
| Pattern Identified | An organisational decision pattern was identified |

---

### Organisational Decision Learning Loop

**Purpose:** Post-decision review process — what worked, what didn't, what to change.

**Core behaviour:**
- The Learning Loop is triggered at the end of each decision cycle (or manually)
- It surfaces completed decisions, outcomes, and patterns from the cycle
- The portfolio manager reviews each completed decision: what drove the outcome, what would they do differently
- Patterns are identified across multiple decisions and portfolio managers
- Validated insights are added to the Institutional Decision Knowledge Graph
- The Learning Loop produces a decision review report

**Learning Loop stages:**
| Stage | Description |
|-------|-------------|
| Review | Surface completed decisions and outcomes from the cycle |
| Reflect | Portfolio manager reflects on outcomes — what drove them, what they'd do differently |
| Pattern | Patterns identified across multiple decisions and portfolio managers |
| Insight | Validated insights added to the Institutional Decision Knowledge Graph |
| Report | Decision review report generated |

---

### Institutional Decision Knowledge Graph

**Purpose:** Accumulated organisational investment knowledge — decision patterns, sector insights, company histories.

**Core behaviour:**
- The Institutional Decision Knowledge Graph is the persistent, structured knowledge asset for the firm
- It accumulates entities (companies, sectors, portfolio managers, investment theses) and relationships (decision patterns, outcome patterns, sector correlations)
- Patterns are surfaced from the graph by the Continuous Learning Engine
- The graph evolves over time as new decisions, reviews, and outcomes are added
- The graph is queryable — portfolio managers can explore the firm's decision history directly

**Entity types:**
| Type | Description |
|------|-------------|
| Company | A specific company with investment history |
| Sector | A sector or industry with accumulated insights |
| Portfolio Manager | A specific portfolio manager with decision history |
| Investment Thesis | A structured investment hypothesis |
| Evidence | A structured research evidence item |
| Committee Review | A documented committee review |
| Outcome | A recorded investment outcome |
| Pattern | A recurring structure in decisions, assumptions, or outcomes |
| Knowledge | A validated, durable investment insight |

---

## Platform Integration

ChronoSentiment Enterprise is built on the Coralys Knowledge Evolution Platform. The platform provides:

| Platform Capability | ChronoSentiment Enterprise Usage |
|--------------------|----------------------------------|
| Temporal Sentiment Engine | Captures and structures investment signals from evidence items |
| Continuous Learning Engine | Evolves knowledge from investment evidence to decision pattern to organisational knowledge |
| Knowledge Graph | The persistent, structured institutional investment knowledge asset |
| Lifecycle Governance | Governs state transitions for Investment Theses (Draft → Under Review → Approved → Monitoring → Closed) |

---

## User Experience Principles

1. **Decisions first** — the primary output is always a better investment decision. The knowledge layer is the engine behind it, not the interface.
2. **Provenance by default** — every piece of information in the system has a traceable source. No black boxes.
3. **Governance as infrastructure** — committee review workflows and audit trails are built in from the start, not retrofitted.
4. **Organisational learning as the moat** — the Institutional Decision Knowledge Graph becomes more valuable with every decision cycle.
5. **AI documentation as a first-class feature** — the AI's contribution to every decision is documented, not hidden.

---

## Technical Constraints

| Constraint | Detail |
|------------|--------|
| Platform | Coralys Knowledge Evolution Platform |
| Domain adapter | `adapters/chronosentiment` (stub — to be implemented) |
| Deployment | Enterprise (cloud-hosted, single-tenant option available) |
| Data residency | EU and UK data residency options required |
| Security | SOC 2 Type II; encryption at rest and in transit |
| Availability | 99.9% uptime SLA |

---

## MVP Scope

The following capabilities are in scope for the ChronoSentiment Enterprise MVP:

| Capability | Status |
|------------|--------|
| Decision Workspace | Documented; implementation in progress |
| Investment Thesis with versioning | Documented; implementation in progress |
| Evidence management | Documented; implementation in progress |
| Decision Timeline | Documented; implementation in progress |
| Committee Review workflow | Documented; implementation in progress |
| Decision Outcome recording | Documented; implementation in progress |
| Organisational Decision Learning Loop | Planned |
| Institutional Decision Knowledge Graph | Planned |
| AI conversation documentation | Planned |
| Regulatory compliance reporting | v2.0 |

---

## v2.0 Candidates

| Feature | Description | Rationale |
|---------|-------------|-----------|
| Regulatory Compliance Reporting | Automated AI documentation reports for FCA, SEC, EU AI Act | Growing regulatory requirement |
| Cross-Firm Benchmarking | Anonymised decision quality benchmarking across firms | High-value for CIOs; requires network scale |
| Predictive Decision Support | AI-assisted thesis formation based on historical patterns | High-value; requires mature Knowledge Graph |
| Knowledge Graph Services | Semantic retrieval, contextual enrichment, provenance traversal | Makes the Institutional Decision Knowledge Graph queryable and actionable |
| Mobile Decision Capture | Mobile interface for capturing evidence and thesis notes on the go | Reduces friction for portfolio managers |

---

## Document History

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2026-07-26 | Initial version — derived from `ChronoSentiment_Product_Blueprint_v1.md` and refined for the Enterprise product |

---

*ChronoSentiment Enterprise Product Blueprint v1.0 | July 2026*
*Defines the product blueprint for ChronoSentiment Enterprise — Financial Decision Intelligence Platform.*
*Review trigger: Material change in product features, user experience, or platform integration.*

---

*End of document.*