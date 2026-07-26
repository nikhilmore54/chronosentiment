# ChronoSentiment Personal — Product Blueprint

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
- Informed by: `CS-S-004_ChronoSentiment_Personal_Product_Strategy_v1.0.md` (product strategy)
- Informed by: `CORALYS_PLATFORM_ARCHITECTURE.md` (platform architecture)
- Informed by: `ChronoSentiment_Personal_Blueprint_v1.md` (predecessor draft blueprint v1.1)
- Informs: Engineering implementation

---

## Purpose

This document defines the product blueprint for ChronoSentiment Personal — the detailed product specification, user experience, and platform integration for the Personal Investment Knowledge Platform.

---

## Product Identity

**Product name:** ChronoSentiment Personal
**Commercial positioning:** Personal Investment Knowledge Platform
**Platform:** Coralys Knowledge Evolution Platform
**Target audience:** Individual investors who take their investment research seriously

---

## The Continuous Research Loop

This is the product. Everything else supports it.

Most research platforms stop at the decision:

```
Research → Decision → End
```

No learning occurs. The next decision starts from scratch.

ChronoSentiment Personal runs a continuous loop:

```
Research Workspace
        ↓
Research Dossier (structured research)
        ↓
Investment Thesis (recorded at time of decision)
        ↓
Portfolio (decision executed)
        ↓
Market evolves
        ↓
Research Review (quarterly — has anything changed?)
        ↓
Research Timeline updated (thesis revised if needed)
        ↓
Outcome recorded (what actually happened)
        ↓
Personal Investment Learning Loop (what did I learn?)
        ↓
Personal Investment Knowledge Graph updated
        ↓
Next Research Workspace (starts with accumulated knowledge)
```

The loop is the moat. Every cycle makes the next one better.

---

## Core User Journey

The core user journey for ChronoSentiment Personal follows the Knowledge Evolution Lifecycle adapted for individual investment research:

| Stage | User Action | Platform Response |
|-------|-------------|------------------|
| Research | Gather research — annual reports, earnings calls, AI conversations, news | Structured, time-stamped research sources added to Research Workspace |
| Thesis | Articulate investment thesis with assumptions and risks | Versioned Investment Thesis created; linked to research sources |
| Decision | Execute investment decision | Investment Thesis recorded at time of decision; Research Timeline updated |
| Review | Quarterly review — has anything changed? | Research Review created; thesis revised if needed; Research Timeline updated |
| Outcome | Record investment outcome — what actually happened | Investment Outcome recorded; Personal Investment Learning Loop triggered |
| Learning | Reflect on what worked, what didn't, what to change | Personal Investment Learning Loop completed; Personal Investment Knowledge Graph updated |

---

## Primary User Persona

**Name:** Alex
**Role:** Self-directed individual investor
**Portfolio:** £150,000 in self-managed ISA and SIPP
**Research time:** 8–12 hours per week
**Problem:** Alex reads annual reports, follows earnings calls, and forms their own investment theses. But their research is scattered across a notebook, a spreadsheet, and browser bookmarks. When they revisit a company six months later, they cannot see how their thinking has evolved. When a position goes wrong, they cannot reconstruct the original thesis to understand what they missed. They are not learning systematically from their investment experience.

**What ChronoSentiment Personal does for Alex:** Every investment has a Research Workspace. Alex captures research as structured sources, articulates their thesis, records quarterly reviews, and tracks outcomes. Over time, the Personal Investment Knowledge Graph reflects Alex's accumulated investment knowledge — companies they've researched, sectors they understand, patterns in their own decision-making.

---

## Feature Specifications

### Research Workspace

**Purpose:** Structured environment for a single investment research cycle — from initial research through thesis formation, quarterly reviews, and outcome recording.

**Core behaviour:**
- Each Research Workspace is linked to a specific company (the Subject)
- Research sources are captured within the Workspace with timestamps and source records
- The Workspace persists from initial research through execution, quarterly reviews, and outcome recording
- Workspaces accumulate into the Personal Investment Knowledge Graph over time

**Fields:**
| Field | Type | Description |
|-------|------|-------------|
| Company | Text | The company being researched |
| Portfolio | Text | The portfolio context (e.g. "ISA — Long-term growth") |
| Investment Thesis | Object | The active thesis for this Workspace |
| Thesis Versions | List | All thesis versions created in this Workspace |
| Research Sources | List | Time-stamped research sources (annual reports, earnings calls, AI conversations, news) |
| Research Reviews | List | All quarterly research reviews conducted for this Workspace |
| Outcome | Object | The recorded investment outcome |
| Created | Date | Date the Workspace was opened |
| Last Updated | Date | Date the Workspace was last updated |
| Status | Enum | Researching / Thesis Formed / Invested / Monitoring / Closed |

---

### Research Dossier

**Purpose:** Accumulated, structured record of all research on a company — the investor's knowledge base for a specific investment.

**Core behaviour:**
- The Research Dossier is the structured view of all research sources in a Research Workspace
- Sources are organised by type (annual reports, earnings calls, AI conversations, news, personal notes)
- Sources are immutable once recorded — they cannot be revised to match a later thesis
- The dossier provides a chronological view of how the investor's research accumulated
- Dossiers accumulate in the Personal Investment Knowledge Graph

**Research source types:**
| Type | Description |
|------|-------------|
| Annual Report | Company annual report or 10-K |
| Earnings Call | Earnings call transcript or notes |
| AI Conversation | Documented AI research conversation |
| News | News article or press release |
| Personal Note | Investor's own research note or observation |
| Financial Data | Financial metrics or ratios |
| Sector Research | Industry or sector research |

---

### Investment Thesis

**Purpose:** Structured, versioned statement of what the investor believes about a company and why.

**Core behaviour:**
- Each Investment Thesis is a structured hypothesis about an investment opportunity
- Theses are versioned — each revision is timestamped and linked to the research that caused it
- Assumptions and risks are explicitly documented
- The thesis is linked to the research sources that support it
- Thesis versions feed the Personal Investment Learning Loop

**Fields:**
| Field | Type | Description |
|-------|------|-------------|
| Title | Text | Brief title of the investment thesis |
| Thesis Statement | Text | The core investment belief (e.g. "Reliance Industries is undervalued relative to its long-term earnings power") |
| Assumptions | List | Explicit assumptions the thesis depends on |
| Risks | List | Risks that could invalidate the thesis |
| Research Sources | List | Linked research sources that support the thesis |
| Version | Integer | Thesis version number (v1, v2, v3...) |
| Version Notes | Text | What changed in this version and why |
| Status | Enum | Draft / Active / Under Review / Closed |
| Created | Date | Date this version was created |

---

### Research Timeline

**Purpose:** Chronological view of how the investor's thinking about a company evolved.

**Core behaviour:**
- The Research Timeline is a chronological feed of all research sources, thesis versions, research reviews, and outcomes for a Research Workspace
- Items are filterable by type and date range
- The timeline provides a complete record of the investor's research journey for each company
- Timeline items link back to their source research or thesis version

**Timeline item types:**
| Type | Description |
|------|-------------|
| Research Added | New research source added to the Workspace |
| Thesis Created | A new investment thesis was created |
| Thesis Revised | An existing thesis was revised |
| Review Completed | A quarterly research review was completed |
| Decision Made | An investment decision was made |
| Position Opened | A position was opened |
| Position Monitored | A monitoring note was added |
| Outcome Recorded | An investment outcome was recorded |
| Learning Captured | A personal learning was captured |

---

### Quarterly Research Review

**Purpose:** Structured periodic review of an active investment thesis against new evidence.

**Core behaviour:**
- A Research Review is triggered quarterly (or manually) for each active Research Workspace
- The review surfaces new research since the last review and asks: has anything changed?
- The investor reviews each assumption in the thesis against new evidence
- If assumptions have changed, the thesis is revised (creating a new version)
- The review is recorded in the Research Timeline
- Reviews feed the Personal Investment Learning Loop

**Review structure:**
| Section | Description |
|---------|-------------|
| New Research | Research sources added since the last review |
| Assumption Check | Review each thesis assumption against new evidence |
| Thesis Status | Has the thesis changed? (Confirmed / Revised / Invalidated) |
| Revised Thesis | New thesis version (if revised) |
| Next Review | Date of the next scheduled review |

---

### Personal Investment Learning Loop

**Purpose:** Post-outcome review process — what worked, what didn't, what to change.

**Core behaviour:**
- The Learning Loop is triggered when an investment outcome is recorded (or manually)
- It surfaces the original thesis, all thesis versions, all research reviews, and the outcome
- The investor reflects on each stage: what drove the outcome, what they'd do differently
- Patterns are identified across multiple investments
- Validated insights are added to the Personal Investment Knowledge Graph
- The Learning Loop produces a personal investment review report

**Learning Loop stages:**
| Stage | Description |
|-------|-------------|
| Outcome | Surface the investment outcome and the original thesis |
| Thesis Review | Compare the original thesis to what actually happened |
| Assumption Review | Which assumptions were correct? Which were wrong? |
| Research Review | Was the research sufficient? What was missing? |
| Process Review | Was the research process sound? What would you do differently? |
| Learning | Capture the key lessons from this investment |
| Pattern | Identify patterns across multiple investments |
| Knowledge | Add validated insights to the Personal Investment Knowledge Graph |

---

### Personal Investment Knowledge Graph

**Purpose:** Accumulated personal investment knowledge — companies researched, sectors understood, patterns in the investor's own decision-making.

**Core behaviour:**
- The Personal Investment Knowledge Graph is the persistent, structured knowledge asset for the individual investor
- It accumulates entities (companies, sectors, investment theses) and relationships (research patterns, outcome patterns, sector correlations)
- Patterns are surfaced from the graph by the Continuous Learning Engine
- The graph evolves over time as new research cycles, reviews, and outcomes are added
- The graph is queryable — investors can explore their own investment history directly

**Entity types:**
| Type | Description |
|------|-------------|
| Company | A specific company with research history |
| Sector | A sector or industry with accumulated insights |
| Investment Thesis | A structured investment hypothesis |
| Research Source | A structured research source |
| Research Review | A documented quarterly review |
| Outcome | A recorded investment outcome |
| Pattern | A recurring structure in research, assumptions, or outcomes |
| Knowledge | A validated, durable personal investment insight |

---

## Platform Integration

ChronoSentiment Personal is built on the Coralys Knowledge Evolution Platform. Unlike other Coralys products, the platform's knowledge-centric nature is front and centre — because the customer's goal is knowledge evolution, not just decision quality.

| Platform Capability | ChronoSentiment Personal Usage |
|--------------------|-------------------------------|
| Temporal Sentiment Engine | Captures and structures investment signals from research sources |
| Continuous Learning Engine | Evolves knowledge from research evidence to investment pattern to personal knowledge |
| Knowledge Graph | The persistent, structured personal investment knowledge asset |
| Lifecycle Governance | Governs state transitions for Investment Theses (Draft → Active → Under Review → Closed) |

---

## Vocabulary Symmetry with Enterprise

ChronoSentiment Personal and ChronoSentiment Enterprise share the same underlying platform lifecycle. The vocabulary is adapted for each product's context:

| Personal | Enterprise |
|----------|-----------|
| Research Workspace | Decision Workspace |
| Research Dossier | Decision Record |
| Research Timeline | Decision Timeline |
| Research Memory | Decision Memory |
| Research Intelligence | Decision Intelligence |
| Research Reviews | Committee Reviews |
| Personal Investment Learning Loop | Organisational Decision Learning Loop |
| Personal Investment Knowledge Graph | Institutional Decision Knowledge Graph |

---

## User Experience Principles

1. **Research first** — the primary output is always better investment research. The knowledge layer is the product, not the infrastructure.
2. **AI as research assistant, not decision-maker** — AI helps the investor research better; it does not make decisions for them. The investor's reasoning is always front and centre.
3. **Provenance by default** — every piece of information in the system has a traceable source. No black boxes.
4. **Learning loop as the differentiator** — the Personal Investment Learning Loop is what makes ChronoSentiment Personal distinctive. It must be excellent.
5. **Knowledge compounds** — every research cycle should make the next one better. The investor should feel the compounding effect over time.

---

## Technical Constraints

| Constraint | Detail |
|------------|--------|
| Platform | Coralys Knowledge Evolution Platform |
| Domain adapter | `adapters/chronosentiment` (stub — to be implemented) |
| Deployment | Cloud-hosted (multi-tenant) |
| Data residency | UK and EU data residency |
| Security | Encryption at rest and in transit |
| Availability | 99.9% uptime SLA |

---

## MVP Scope

The following capabilities are in scope for the ChronoSentiment Personal MVP:

| Capability | Status |
|------------|--------|
| Research Workspace | Documented in Blueprint v1.1; implementation in progress |
| Research Dossier | Documented in Blueprint v1.1; implementation in progress |
| Investment Thesis with versioning | Documented in Blueprint v1.1; implementation in progress |
| Research Timeline | Documented in Blueprint v1.1; implementation in progress |
| Quarterly Research Review | Documented in Blueprint v1.1; implementation in progress |
| Investment Outcome recording | Documented in Blueprint v1.1; implementation in progress |
| Personal Investment Learning Loop | Planned |
| Personal Investment Knowledge Graph | Planned |
| AI conversation documentation | Planned |

---

## v2.0 Candidates

| Feature | Description | Rationale |
|---------|-------------|-----------|
| Research Quality Scoring | AI-assisted scoring of research quality against a structured framework | Helps investors improve their research process |
| Investor Behaviour Patterns | Patterns in the investor's own decision-making surfaced from the Knowledge Graph | High-value for self-improvement |
| Cross-Investor Benchmarking | Anonymised research quality benchmarking across investors | Requires network scale |
| Knowledge Graph Services | Semantic retrieval, contextual enrichment, provenance traversal | Makes the Personal Investment Knowledge Graph queryable and actionable |
| Mobile Research Capture | Mobile interface for capturing research notes and observations on the go | Reduces friction for investors |

---

## Document History

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2026-07-26 | Initial Baseline version — derived from `ChronoSentiment_Personal_Blueprint_v1.md` (Draft v1.1) and refined as a standalone Baseline document |

---

*ChronoSentiment Personal Product Blueprint v1.0 | July 2026*
*Defines the product blueprint for ChronoSentiment Personal — Personal Investment Knowledge Platform.*
*Review trigger: Material change in product features, user experience, or platform integration.*

---

*End of document.*