# ChronoSentiment Product Blueprint v1.0

**Document type:** Product Blueprint
**Version:** 1.0
**Status:** Draft
**Date:** 2026-07-26
**Owner:** Product

---

## Document Governance

| Field | Value |
|-------|-------|
| Document Status | Draft |
| Next Review | After Phase 1B customer validation |
| Review Trigger | Phase 1B results; design partner feedback; material change in product direction |

**Relationship to other documents:**
- Informed by: CS-R-001 through CS-R-015A (research programme)
- Complements: CHRONOSENTIMENT_PRD_V1.md (product requirements)
- Describes evolution of: existing application documented in `docs/ui/uiux.md`
- Feeds into: Engineering contracts, MVP scope, Phase 1B validation design

---

## Purpose

This document defines what ChronoSentiment is as a product — not as a market opportunity or an investment case, but as an experience that investment professionals use every day.

The research programme (CS-R-001 through CS-R-015) has answered: *Can we build it, and should we?* This document answers: *What exactly are we building, and why will people use it?*

**Critical framing:** ChronoSentiment is not a greenfield product. A substantial application already exists. The MVP is not a build-from-scratch project — it is a repositioning and extension of a mature technical platform. This document describes that evolution.

The organising principle is a shift in narrative:

```
Research narrative:   Market → Problem → Investment Thesis → Validation
Product narrative:    User → Workflow → Product → Evidence → Business
```

---

## Why Investment Organisations Forget

Investment management is a knowledge-intensive profession. Every consequential decision is the product of months of research, dozens of conversations, multiple analytical frameworks, and the accumulated judgement of experienced professionals.

And then it disappears.

Not the trade. Not the position. Not the P&L. Those are recorded everywhere.

What disappears is the **reasoning** — the why behind the decision. The context that made the thesis compelling. The assumptions that were made. The risks that were identified and accepted. The AI tools that contributed to the analysis. The discussion that happened in the committee room. The conditions that were attached to the approval.

Six months later, when an LP asks why a position was initiated, the portfolio manager reconstructs the answer from memory — filtered through hindsight, coloured by the outcome, and missing the information that was available at the time but has since been superseded.

This is not a failure of intelligence or diligence. It is a structural problem with how investment organisations manage knowledge.

### The five ways investment organisations forget

**1. Staff turnover.** When a senior analyst or portfolio manager leaves, they take years of thesis context with them. The positions remain on the book. The reasoning does not. The new team member inherits a portfolio they cannot fully explain.

**2. Hindsight bias.** When a decision is reviewed after the outcome is known, the original reasoning is unconsciously reconstructed to fit the result. A decision that was genuinely uncertain at the time appears obvious in retrospect — or obviously wrong. The review teaches the wrong lessons.

**3. Undocumented AI usage.** Investment teams now use AI tools — ChatGPT, Claude, Bloomberg AI, AlphaSense — in their research and decision process. These conversations are not recorded. The AI's contribution to the thesis is invisible. When a regulator asks which AI tools influenced which decisions, there is no answer.

**4. Fragmented research.** The evidence behind a decision lives in email threads, Bloomberg chat, shared drives, analyst notes, and the portfolio manager's memory. No single place holds the complete picture. Reconstructing it takes days.

**5. Committee memory.** Investment committee discussions are rarely recorded in full. The formal minutes capture the decision. They do not capture the debate, the objections that were raised and addressed, the conditions that were attached, or the dissenting views that were overruled. The institutional memory of how the committee thinks is lost.

### The consequence

The consequence is not just operational inconvenience. It is a systematic degradation of institutional learning.

If an organisation cannot accurately reconstruct why a decision was made, it cannot learn from that decision. It cannot distinguish between a good decision with a bad outcome and a bad decision with a good outcome. It cannot identify recurring patterns in its own reasoning. It cannot improve.

**ChronoSentiment preserves institutional decision memory.** Not as an archive. As a living record that makes every past decision accessible, explainable, and useful for future decisions.

---

## Why Now?

The problem of institutional forgetting is not new. What is new is the combination of forces that makes it both more acute and more solvable than at any previous point.

### Why the problem is more acute now

**AI adoption has accelerated the documentation gap.** Investment teams are now using AI tools in their research and decision process at scale. These tools generate analysis, surface insights, and contribute to investment theses — but their contributions are invisible in the decision record. The gap between what actually happened and what can be documented has widened significantly in the last 24 months.

**Regulatory requirements are catching up.** The EU AI Act, FCA guidance on AI in financial services, and SEC commentary on AI in investment management are creating explicit requirements to document AI usage in consequential decisions. Firms that cannot demonstrate what AI tools contributed to which decisions face regulatory exposure. The urgency is real and growing.

**LP expectations have risen.** Institutional LPs — pension funds, endowments, sovereign wealth funds — are increasingly asking for decision-level transparency, not just portfolio-level reporting. The question "why did you make this decision?" is becoming a standard part of LP due diligence and ongoing reporting.

**Staff turnover has increased.** The post-2020 labour market in financial services has seen elevated turnover at the analyst and portfolio manager level. The knowledge loss problem has become more frequent and more visible.

### Why the problem is more solvable now

**Point-in-time reconstruction is technically feasible.** The ChronoSentiment platform already implements deterministic replay — the ability to reconstruct the exact information environment at any historical point in time. This is the hardest technical problem in decision documentation, and it is already solved.

**Large language models make explainability tractable.** Generating natural-language explanations of complex technical processes — what happened, why it happened, what it caused — is now a solved problem. The narrative block system in the existing platform demonstrates this at the execution layer. The same capability applies at the decision layer.

**Decision archives are becoming valuable.** As AI systems become more capable of learning from historical decisions, the decision archive itself becomes a strategic asset. Firms that have documented their decision history will be able to train AI systems on their own institutional knowledge. Firms that have not will be starting from scratch.

**The category is being created now.** No vendor currently makes investment decision governance its primary product. The category is being defined. The firm that defines it will have a significant first-mover advantage in category language, customer relationships, and data network effects.

---

## A Day with ChronoSentiment

The following narrative illustrates how ChronoSentiment integrates into the investment workflow. It is not a feature list — it is a story about how the product changes the experience of making, explaining, and learning from investment decisions.

---

**Monday morning.** A portfolio manager at a mid-size asset manager has been following a European industrial company for three months. The thesis has crystallised: the market is underpricing the company's energy transition exposure, and a catalyst is approaching in the form of a Q3 earnings call. She opens ChronoSentiment and creates a new Decision Workspace.

She types the thesis in three sentences. She attaches the AlphaSense research summary she used, exports the ChatGPT conversation where she stress-tested the assumptions, and records the key risks: execution risk on the energy transition programme, and currency exposure if the euro weakens. She sets conviction at 4/5 and submits for committee review.

The replay engine automatically snapshots the market data state at the moment of submission. The information environment at the time of the decision is preserved.

**Tuesday.** The investment committee reviews the decision. The CIO challenges one assumption — the timeline for the energy transition catalyst. The discussion is recorded. The committee approves with one condition: position size capped at 2% until the Q3 earnings call confirms the thesis. The approval record is timestamped and immutable.

**Wednesday.** The position is initiated. The execution is linked to the Decision Workspace. The certification state (CERTIFIED) is recorded.

**Three months later.** An LP sends a query: "Can you explain the rationale for your position in [company]?" The portfolio manager opens the Decision Workspace. ChronoSentiment reconstructs the information environment at the time of the decision — the market data, the research, the AI conversations, the committee discussion. She generates a natural-language explanation in two minutes. The LP receives a PDF that reads as if a thoughtful analyst wrote it, grounded in the information that was available at the time.

**Six months later.** The Q3 earnings call has passed. The thesis was partially right — the energy transition exposure was correctly identified, but the timeline was longer than expected. The portfolio manager opens the Review tab. ChronoSentiment presents the original thesis alongside the actual outcome. The divergence analysis shows where the execution diverged from the plan. She records the lesson: the timeline assumption was too aggressive; future theses in this sector should apply a 1.5x timeline multiplier. The lesson is tagged and searchable.

**One year later.** A new analyst is building a thesis on a similar company in the same sector. ChronoSentiment surfaces the previous decision as a relevant reference. The analyst can see the original thesis, the committee discussion, the outcome, and the lesson — without asking anyone. The institutional memory is intact.

---

## The Decision Workspace

The Decision Workspace is ChronoSentiment's primary innovation. It is the persistent home of an investment decision throughout its entire lifecycle — before the decision is made, during execution, and after the outcome is known.

Think of it as **GitHub for investment decisions**. Just as GitHub gives every piece of code a persistent, versioned, collaborative home, the Decision Workspace gives every investment decision a persistent, timestamped, auditable home.

The analogy is precise in several ways:

- **Versioned.** Every change to the Decision Workspace is timestamped and attributed. The history of how the thesis evolved is preserved.
- **Collaborative.** Multiple team members can contribute to a Decision Workspace — the analyst who built the model, the portfolio manager who formed the thesis, the CIO who approved it.
- **Linked.** The Decision Workspace links to the evidence (research, AI conversations, market data), the execution (trades, strategies), and the outcome (actual result, divergence analysis).
- **Searchable.** Past decisions are searchable by sector, thesis type, outcome, AI tool used, or any other dimension.
- **Auditable.** Every action in the Decision Workspace is logged. Who viewed it, who edited it, when, and what changed.

### What lives in the Decision Workspace

```
Decision Workspace
│
├── Thesis
│   The investment case in the portfolio manager's own words.
│   What we believe, why we believe it, what we expect to happen.
│
├── Evidence
│   Everything that informed the thesis at the time of the decision.
│   Research documents, data snapshots, AI conversation exports,
│   market data (auto-captured by the replay engine at the moment
│   of decision creation).
│
├── Assumptions & Risks
│   What must be true for the thesis to work.
│   What could make it wrong.
│
├── Committee
│   The discussion record. Who said what.
│   The approval (or rejection) with conditions.
│   The dissenting views.
│
├── Execution
│   The linked strategy and trades.
│   The execution vs plan comparison (via the existing comparison engine).
│   The certification state.
│
├── Outcome
│   What actually happened.
│   Thesis vs outcome (via the existing divergence analysis).
│   Lessons captured.
│
└── Provenance
    AI tools used and their contributions.
    Information sources.
    Causal trace (via the existing ancestry/propagation system).
    Audit trail.
```

### The Decision Record

The Decision Workspace produces a **Decision Record** — the persistent, exportable artefact that captures the complete lifecycle of a decision. The Decision Record is what gets shared with LPs, presented to regulators, and used for post-mortem analysis.

The Decision Record is not a report generated after the fact. It is a live document that accumulates evidence throughout the decision lifecycle and can be exported at any point as a timestamped, audit-grade document.

---

## Current Product Assessment

### What Already Exists

ChronoSentiment today is a **schema-bound causal replay instrument** — a technically sophisticated execution analysis and replay platform. The following capabilities are already implemented and production-grade:

| Existing Capability | Technical Description | Status |
|--------------------|-----------------------|--------|
| **Deterministic replay** | Reconstruct any historical execution state from a strategy ID and seed; schema-bound, backend-certified | ✅ Complete |
| **Execution simulation** | Run genetic algorithm (GA) strategies against historical data with configurable signal filters | ✅ Complete |
| **Timeline** | Chronological narrative block sequence with timestamped events, key event markers, group transitions | ✅ Complete |
| **Observability** | Real-time telemetry: queue depth, fill latency, sync ratio, events/second, snapshot sequence ID, throttle state | ✅ Complete |
| **Explainability** | Narrative blocks with natural-language execution traces; causal ancestry and forward propagation panels | ✅ Complete |
| **Scenario comparison** | Dual-strategy comparison with divergence analysis, comparison summary, confidence level, final verdict | ✅ Complete |
| **Certification** | Backend-certified `certification_state` (CERTIFIED / DEGRADED / PARTIAL / INVALID) surfaced per replay | ✅ Complete |
| **Provenance** | Causal ancestry graph (lineage), forward propagation (downstream topology), divergence type taxonomy | ✅ Complete |
| **Replay inspection** | Divergence accumulation at replay position, causal chain depth counter, ancestry path breadcrumb | ✅ Complete |
| **Global ranking** | Strategy ranking across the cohort | ✅ Complete |

### Current Application Identity

Today the application behaves primarily as an **execution analysis and replay platform**. It is technically exceptional — deterministic, schema-bound, causally traceable, and certification-aware.

However, the language of the current application is the language of execution engineering:

- Replay, simulation, ecology, survivability surfaces
- Certification, topology, orchestration
- Divergence, propagation, causal ancestry
- Cohort, seed, sequence ID, kernel state

A portfolio manager does not think in these terms. They think in terms of:

- Ideas, decisions, positions, thesis
- Investment committee, approval, conviction
- Review, outcome, learning
- LP reporting, regulatory documentation

**The gap is not technical capability. The gap is abstraction level.** The existing platform operates at the execution layer. The product needs to operate at the decision layer — and expose the execution layer as evidence for decisions, not as the primary interface.

### What the Existing Platform Proves

The existing application demonstrates that the team can build the hardest parts of ChronoSentiment:

- **Temporal reconstruction is solved.** Deterministic replay from any historical state is implemented and production-grade.
- **Explainability is solved.** Narrative blocks with causal traces are implemented.
- **Provenance is solved.** Causal ancestry, forward propagation, and divergence attribution are implemented.
- **Certification is solved.** Backend-certified state with audit-grade confidence levels is implemented.
- **Scenario comparison is solved.** Dual-strategy comparison with divergence analysis is implemented.

The MVP is not a build-from-scratch project. It is a **repositioning** of these capabilities at the decision layer, plus the addition of the Decision Workspace as the missing product abstraction.

---

## Chapter 3 — Evolution from the Existing Platform

### The Abstraction Shift

Nothing in the existing platform is discarded. Every capability is repositioned one abstraction level higher — from execution concepts to decision concepts.

| Existing Capability | Existing Name | New Product Role | New Name |
|--------------------|---------------|-----------------|----------|
| Replay timeline | Replay Timeline | Chronological view of how a decision evolved | Decision Timeline |
| Strategy inspector | Strategy Inspector | Inspect the evidence behind a decision | Decision Inspector |
| Deterministic replay | Replay / Reconstruct | Reconstruct the information environment at the time of a decision | Decision Reconstruction |
| Narrative blocks | Execution Trace | Natural-language explanation of why a decision was made | Decision Explanation |
| Causal ancestry | Lineage / Ancestry | What information and events led to this decision | Decision Provenance |
| Forward propagation | Downstream topology | What this decision caused or influenced | Decision Impact |
| Scenario comparison | Compare Strategies | Compare alternative decision paths | Decision Alternatives |
| Certification state | Certification | Audit-grade confidence in the decision record | Decision Evidence Quality |
| Divergence analysis | Divergence accumulation | Where the actual outcome diverged from the thesis | Thesis vs Outcome |
| Observability telemetry | Observatory telemetry | Operational context at the time of the decision | Decision Context |
| Global ranking | Global Ranking | How this decision compares to alternatives considered | Decision Ranking |

### The Evolution Path

```
Today — Execution Analysis Platform
│
│  Replay engine (deterministic, schema-bound)
│  Execution simulation (GA, signal filters)
│  Causal trace (ancestry, propagation)
│  Certification (CERTIFIED/DEGRADED/PARTIAL/INVALID)
│  Scenario comparison (dual-strategy, divergence)
│  Observability (telemetry, topology)
│
↓  + Decision Workspace (new)
↓  + Decision capture (new)
↓  + Committee workflow (new)
↓  + Approval workflow (new)
↓  + Reporting layer (new)
↓  + Investment-domain UX (new)
│
MVP — Financial Decision Management Platform
│
│  Decision Workspace (new)
│  Decision Timeline (repositioned from Replay Timeline)
│  Decision Inspector (repositioned from Strategy Inspector)
│  Decision Reconstruction (repositioned from Replay)
│  Decision Explanation (repositioned from Narrative Blocks)
│  Decision Provenance (repositioned from Causal Ancestry)
│  Decision Alternatives (repositioned from Compare Strategies)
│  Decision Evidence Quality (repositioned from Certification)
│
↓  + Cross-decision pattern recognition (Phase 2)
↓  + AI-assisted decision improvement (Phase 3)
│
Long-term — Institutional Decision Intelligence Platform
```

### Engineering Implication

The MVP engineering effort is substantially smaller than a greenfield build because the hardest technical problems are already solved. The remaining work is:

**Already implemented (no rebuild required):**
- Temporal reconstruction engine
- Deterministic replay
- Causal trace and provenance
- Certification and evidence quality
- Scenario comparison and divergence analysis
- Observability and telemetry

**Needs building for MVP:**
- Decision Workspace (the missing product abstraction — where decisions live)
- Decision capture (structured input: thesis, evidence, assumptions, risks, conviction)
- Committee and approval workflow
- Investment-domain UX (language, navigation, personas)
- Reporting layer (LP report, regulatory documentation, committee summary export)
- Data integration layer (connect existing replay engine to investment-domain data sources)

---

## 1. Product Philosophy

**The Decision is the product.**

Every feature, screen, and workflow in ChronoSentiment exists to serve one core object: the **Decision**. Not the trade. Not the strategy. Not the simulation. The Decision.

A decision is not a moment. It is a lifecycle:

```
Evidence gathered
        ↓
Thesis formed
        ↓
Discussion and challenge
        ↓
AI assistance applied
        ↓
Approval granted
        ↓
Execution linked
        ↓
Outcome observed
        ↓
Review conducted
        ↓
Learning captured
```

ChronoSentiment holds this lifecycle together — before, during, and after the decision is made. The existing platform already handles the execution and outcome layers with exceptional technical depth. The MVP adds the thesis, capture, committee, and review layers that connect the platform to the investment decision workflow.

**The product is not governance.** Governance is one outcome of using ChronoSentiment well. The product is **decision management**: treating every consequential investment decision as a managed asset rather than a transient event.

**The first question a new user should be able to answer after five minutes:**

> What can I do today that I couldn't do yesterday?

The answer should be concrete and immediate: *I can create a decision record that captures why I made this call, what information I had, and what I expected to happen — and I can come back to it in six months and understand it completely, including a reconstruction of the exact market conditions at the time.*

---

## 2. Core User Personas

### Persona A — The Portfolio Manager

**Name:** Alex, Senior Portfolio Manager, mid-size asset manager ($3B AUM)
**Team:** 8 investment professionals
**AI tools in use:** ChatGPT Enterprise for research synthesis, Bloomberg for data, internal models for screening

**Daily reality:**
- Makes 3–8 investment decisions per week, ranging from position sizing to new position initiation
- Spends 2–3 hours per week in investment committee
- Receives LP queries about specific positions 4–6 times per year
- Has lost institutional knowledge twice when senior analysts left

**Jobs to be done:**
- Record why a decision was made before the context fades
- Explain a position to an LP without reconstructing the original thesis from memory
- Review a decision that went wrong without hindsight bias distorting the analysis
- Understand which AI tools contributed to which conclusions

**What Alex will pay for:** Not governance. Not compliance. The ability to answer "why did we do this?" six months later without spending a day reconstructing it.

---

### Persona B — The CIO

**Name:** Sarah, Chief Investment Officer, family office ($800M AUM)
**Team:** 5 investment professionals + 2 analysts
**Pressure:** LP board meets quarterly; increasing regulatory scrutiny; two senior departures in 18 months

**Daily reality:**
- Chairs investment committee weekly
- Responsible for LP reporting and regulatory compliance
- Has experienced the knowledge loss problem directly — a key analyst left and took 3 years of thesis context with them
- Is using AI tools but has no systematic way to track which AI outputs influenced which decisions

**Jobs to be done:**
- Run a better investment committee — structured pre-decision documentation, clear approval records
- Produce LP reports that explain decisions without reconstructing them from scratch
- Demonstrate to regulators that AI use in investment decisions is documented and auditable
- Retain institutional knowledge independent of individual team members

**What Sarah will pay for:** A system that makes the investment committee more effective and makes LP reporting faster. Governance is a consequence she will value, not the reason she buys.

---

### Persona C — The Compliance Officer

**Name:** Marcus, Head of Compliance, mid-size asset manager ($5B AUM)
**Pressure:** EU AI Act enforcement, SEC guidance on AI in investment management, board-level AI governance requirements

**Daily reality:**
- Responsible for demonstrating that AI use in investment decisions is documented
- Currently has no systematic way to audit which AI tools influenced which investment decisions
- Relies on portfolio managers to self-report AI usage — inconsistently

**Jobs to be done:**
- Audit AI usage in investment decisions without disrupting the investment team's workflow
- Produce documentation for regulatory review without manual reconstruction
- Demonstrate to the board that AI governance is in place

**What Marcus will pay for:** Audit-ready documentation that does not require the investment team to change their workflow significantly. He is a buyer, not a primary user.

---

## 4. Product Principles

**1. The Decision Workspace is the centre of gravity.**
Every feature either creates, enriches, or uses a Decision Workspace. Features that do not serve the decision lifecycle do not belong in the MVP.

**2. Capture must be frictionless.**
If capturing a decision takes more than 2 minutes, portfolio managers will not do it. The capture experience must be fast, structured, and integrated into the existing workflow — not a separate administrative task.

**3. Temporal honesty is non-negotiable.**
Every piece of information in a Decision Workspace must be timestamped and immutable. The system must be able to reconstruct exactly what was known at the time of the decision, using only information that was available then. This is already implemented in the replay engine — the MVP exposes it at the decision layer.

**4. Explainability must be human-readable.**
The natural-language explanation of a decision must be readable by a non-technical stakeholder — an LP, a board member, a regulator. The existing narrative block system provides the technical foundation; the MVP wraps it in investment-domain language.

**5. Governance is a consequence, not a feature.**
The product should not feel like a compliance tool. It should feel like a decision management tool that happens to produce governance-grade documentation as a side effect of normal use.

**6. The existing platform is evidence, not the interface.**
The replay engine, certification system, causal trace, and divergence analysis are the evidence layer — they prove what happened and why. The Decision Workspace is the interface layer — where investment professionals interact with that evidence in their own language.

---

## 5. Hero Workflows

These are the five workflows that define the product. Each is a customer action, not an engineering component.

### Hero Workflow 1 — Capture a Decision

**The job:** Never lose why a decision was made.

**The trigger:** A portfolio manager has formed a view and wants to record it before the context fades.

**The workflow:**
1. Open ChronoSentiment → New Decision
2. Enter thesis statement (1–3 sentences: what we believe, why, and what we expect to happen)
3. Attach supporting evidence (documents, data snapshots, AI conversation exports)
4. Record assumptions and risks
5. Set conviction level
6. Submit for committee review or mark as personal record

**The outcome:** A structured, timestamped decision record that captures the information environment at the moment of decision. The replay engine automatically snapshots the market data state at the time of capture. Retrievable in full six months later.

**Time to complete:** Target < 5 minutes for a standard decision.

---

### Hero Workflow 2 — Explain a Decision

**The job:** Understand the reasoning months later — and explain it to someone else.

**The trigger:** An LP asks why a position was initiated. A regulator requests documentation. A new team member needs to understand the thesis.

**The workflow:**
1. Open the Decision Workspace for the relevant decision
2. ChronoSentiment reconstructs the information environment at the time of decision (point-in-time data via the existing replay engine, attached evidence, AI conversations)
3. Generate natural-language explanation using the existing narrative block system, wrapped in investment-domain language: "On [date], we initiated this position because [thesis], based on [evidence], with [conviction level]. The key assumptions were [X, Y, Z]. The risks we identified were [A, B]."
4. Export as PDF or share link for LP/regulatory use

**The outcome:** A human-readable explanation of the decision, grounded in the information available at the time, produced in minutes rather than days. The certification state (CERTIFIED / DEGRADED / PARTIAL / INVALID) is included in the export as evidence quality indicator.

**Time to complete:** Target < 10 minutes from request to exportable document.

---

### Hero Workflow 3 — Review a Decision

**The job:** Compare thesis with outcome — without hindsight bias.

**The trigger:** A position has been closed, or a significant outcome has occurred. The team wants to understand whether the decision was good, independent of whether the outcome was good.

**The workflow:**
1. Open the Decision Workspace → Review tab
2. ChronoSentiment presents the original thesis, assumptions, and risks alongside the actual outcome
3. The divergence analysis (from the existing comparison engine) shows where the actual execution diverged from the thesis
4. The team records: Was the thesis right? Was the reasoning right? Were the risks correctly identified?
5. Lessons are captured and tagged

**The outcome:** A structured post-mortem that separates process quality from outcome quality. The existing divergence analysis provides the technical evidence; the review workflow provides the investment-domain interpretation layer.

**Time to complete:** Target 30–60 minutes for a thorough review.

---

### Hero Workflow 4 — Learn from Decisions

**The job:** Find recurring behavioural patterns across the decision history.

**The trigger:** Quarterly or annual review. A pattern of losses in a specific sector. A CIO wanting to understand the team's decision-making tendencies.

**The workflow:**
1. Open Insights → Decision Patterns
2. ChronoSentiment surfaces: most common assumption failures, sectors where conviction was systematically miscalibrated, AI tools that contributed to decisions that underperformed
3. The team discusses and records implications for future decision-making

**The outcome:** Systematic learning from the decision archive. The product becomes more valuable over time as the archive grows.

**Note:** This workflow is Phase 2 capability. It requires a sufficient decision archive to be meaningful.

---

### Hero Workflow 5 — Improve Decisions

**The job:** Recommend better future decisions based on past patterns.

**The trigger:** A portfolio manager is forming a new thesis in a sector where the team has a documented history.

**The workflow:**
1. Open New Decision → ChronoSentiment surfaces relevant past decisions in the same sector or with similar characteristics
2. The system highlights: assumptions that failed in similar past decisions, risks that were underweighted, AI tools that were used and their track record
3. The portfolio manager incorporates this context into the new decision

**The outcome:** The decision archive actively improves future decisions. The product closes the loop between past learning and future action.

**Note:** This workflow is Phase 3 capability. It requires a sufficient decision archive and an ML layer built on top of the existing replay and comparison infrastructure.

---

## 6. Information Architecture

### The Decision Workspace

The Decision Workspace is the primary screen. It contains everything relevant to a single decision. The existing replay engine, certification system, and causal trace are accessible from within the workspace as the evidence layer.

```
Decision Workspace
│
├── Header
│   ├── Decision title
│   ├── Status (Draft / In Review / Approved / Executed / Under Review / Closed)
│   ├── Created by / Approved by
│   └── Key dates (created, approved, executed, reviewed)
│
├── Thesis
│   ├── Thesis statement
│   ├── Conviction level
│   └── Expected outcome
│
├── Evidence
│   ├── Attached documents
│   ├── Data snapshots (point-in-time, via replay engine)
│   ├── AI conversation exports
│   └── Market data at time of decision (auto-captured by replay engine)
│
├── Assumptions & Risks
│   ├── Key assumptions (what must be true)
│   └── Key risks (what could make this wrong)
│
├── Committee
│   ├── Discussion record
│   ├── Approval history
│   └── Conditions attached to approval
│
├── Execution
│   ├── Linked strategy / trades
│   ├── Execution vs plan (via existing comparison engine)
│   └── Certification state (CERTIFIED / DEGRADED / PARTIAL / INVALID)
│
├── Outcome
│   ├── Actual outcome
│   ├── Thesis vs outcome (via existing divergence analysis)
│   └── Lessons captured
│
└── Provenance
    ├── AI tools used
    ├── Information sources
    ├── Causal trace (via existing ancestry/propagation system)
    └── Audit trail (who viewed, who edited, when)
```

### Supporting Screens

| Screen | Job | Primary User | Existing Capability Used |
|--------|-----|-------------|--------------------------|
| **Dashboard** | Review today's decisions and pending actions | Portfolio Manager, CIO | — (new) |
| **Decision Timeline** | Chronological view of all decisions | Portfolio Manager, CIO | Replay Timeline (repositioned) |
| **Decision Inspector** | Inspect evidence behind a specific decision | Portfolio Manager | Strategy Inspector (repositioned) |
| **Decision Alternatives** | Compare alternative decision paths | Portfolio Manager, CIO | Compare Strategies (repositioned) |
| **Reporting** | Generate LP reports, regulatory documentation | CIO, Compliance | — (new, uses existing data) |
| **Provenance** | Audit AI usage across all decisions | Compliance, CIO | Causal ancestry + divergence (repositioned) |
| **Insights** | Decision patterns, assumption failure analysis | CIO, Portfolio Manager | Global Ranking + comparison (Phase 2) |

---

## 7. MVP Scope

The MVP must be the simplest product experience that helps an investment team make, remember, explain, and improve consequential decisions. Because the technical foundation already exists, the MVP is primarily a **product layer** built on top of the existing execution platform.

### Already Implemented (no rebuild required)

| Capability | Existing Component | MVP Role |
|-----------|-------------------|---------|
| Temporal reconstruction | Replay engine (deterministic, schema-bound) | Powers Decision Reconstruction and point-in-time evidence |
| Execution simulation | GA runner with signal filters | Powers execution validation and Decision Alternatives |
| Causal trace | Ancestry + forward propagation panels | Powers Decision Provenance |
| Certification | `certification_state` (CERTIFIED/DEGRADED/PARTIAL/INVALID) | Powers Decision Evidence Quality indicator |
| Scenario comparison | Compare Strategies + ComparisonPanels | Powers Decision Alternatives |
| Narrative explainability | Narrative blocks with divergence types | Powers Decision Explanation (with investment-domain wrapper) |
| Observability telemetry | Observatory state, telemetry strip | Powers Decision Context (market conditions at time of decision) |
| Divergence analysis | Divergence accumulation, type taxonomy | Powers Thesis vs Outcome comparison |

### Needs Building for MVP

| Capability | Description | Effort Estimate |
|-----------|-------------|----------------|
| **Decision Workspace** | The core new abstraction — where decisions live; full lifecycle UI | High |
| **Decision capture** | Structured input: thesis, evidence attachments, assumptions, risks, conviction level | Medium |
| **Committee workflow** | Submit → review → approve/reject with conditions; approval record | Medium |
| **Investment-domain UX** | Navigation, language, visual design for investment professionals (not engineers) | High |
| **Reporting layer** | LP report, regulatory audit, committee summary export (PDF) | Medium |
| **Data integration** | Connect replay engine to investment-domain data sources (market data, AI conversation import) | Medium |
| **AI conversation import** | Paste or upload ChatGPT/Claude conversation as evidence in a Decision Workspace | Low |

### MVP Success Criteria

The MVP is successful if:

1. A portfolio manager can create a complete decision record in under 5 minutes.
2. A CIO can generate an LP-ready explanation of a past decision in under 10 minutes.
3. A compliance officer can produce an AI usage audit for a specific decision without asking the portfolio manager.
4. At least 1 design partner uses the product for real investment decisions over a 90-day period and reports that it changed their workflow.

---

## 8. Product Roadmap

### Phase 1B (Now — 90 days)
*Objective: Validate the product concept with target customers before building*

- Customer interviews (20–30 firms): validate problem urgency, willingness to pay, category language
- Proof-of-concept: build the Decision Workspace core (capture + explain) using the existing replay engine as the evidence layer
- Design partner identification: secure at least 1 firm willing to use the PoC for real decisions
- Category language validation: determine what customers call this product in their own words

### MVP (Months 4–9, contingent on Phase 1B go decision)
*Objective: Build the decision layer on top of the existing execution platform*

- Decision Workspace (full lifecycle: Before / During / After)
- Decision capture with evidence attachment and AI conversation import
- Committee and approval workflow
- Investment-domain UX (repositioned navigation, language, visual design)
- Reporting layer (LP report, regulatory audit, committee summary)
- Data integration (connect replay engine to investment-domain data sources)

*Existing capabilities repositioned (no rebuild):*
- Decision Timeline (from Replay Timeline)
- Decision Inspector (from Strategy Inspector)
- Decision Alternatives (from Compare Strategies)
- Decision Provenance (from Causal Ancestry)
- Decision Explanation (from Narrative Blocks)
- Decision Evidence Quality (from Certification State)

### Phase 2 (Months 10–18, contingent on MVP validation)
*Objective: Deepen workflow integration and expand to 5–20 paying customers*

- Bloomberg / FactSet / AlphaSense data integration
- OMS/PMS trade linkage
- Advanced provenance (AI tool performance tracking across decisions)
- Multi-user enterprise administration
- Decision Patterns / Insights (cross-decision pattern recognition)
- API for custom integrations

### Phase 3 (Months 19–36, contingent on Phase 2 commercial validation)
*Objective: Build the decision intelligence layer on top of the decision archive*

- AI-assisted decision improvement (surface relevant past decisions when creating new ones)
- Assumption failure analysis
- Counterfactual analysis ("what would the recommendation have been if we had weighted this factor differently?")
- Benchmarking against anonymised peer decisions (opt-in)

---

## 9. Success Metrics

### Phase 1B metrics (validation)
- Number of firms interviewed: target 20–30
- Problem urgency confirmation rate: target ≥ 5/20 firms confirm problem is real and active
- Willingness to pay: target ≥ 3 firms at ≥ US$30,000/yr
- Design partners secured: target ≥ 1

### MVP metrics (product)
- Time to create a decision record: target < 5 minutes
- Time to generate LP explanation: target < 10 minutes
- Decision records created per user per week: target ≥ 3
- Design partner retention at 90 days: target 100%
- Design partner NPS: target ≥ 50

### Commercial metrics (business)
- Annual Recurring Revenue (ARR): target US$300K–US$600K at end of MVP phase (3–5 paying customers)
- Net Revenue Retention (NRR): target ≥ 110%
- Sales cycle length: target < 6 months for initial contract

---

## 10. UX Principles

**1. Decisions first, features second.**
Every screen opens on a decision or a list of decisions. The product never opens on a settings page, a dashboard of metrics, or a feature menu. The Decision is always the entry point.

**2. Capture is a habit, not a task.**
The capture experience must feel like taking a note, not filling out a form. Structured fields guide the user, but the product should not feel bureaucratic.

**3. Time is always visible.**
Every piece of information in the product displays when it was created. The user should always know whether they are looking at information from the time of the decision or information added later. The existing `timestamp_ns` field and temporal epistemic integrity of the replay engine make this technically straightforward.

**4. Explanations are written for the reader, not the system.**
When ChronoSentiment generates a natural-language explanation, it should read as if a thoughtful analyst wrote it. The existing narrative block system provides the technical foundation; the investment-domain wrapper makes it readable by LPs and regulators. The explanation should be editable by the user before export.

**5. The product should feel calm.**
Investment management is a high-pressure environment. ChronoSentiment should feel like a place where decisions are managed carefully, not a tool that adds urgency or complexity.

**6. Governance is invisible.**
The compliance and audit features should be present but not prominent. A portfolio manager should be able to use ChronoSentiment for six months without thinking about governance. The governance documentation should be a natural output of normal use, not a separate workflow.

**7. The execution layer is evidence, not the interface.**
The replay engine, certification system, causal trace, and divergence analysis are powerful and technically impressive. They should be accessible from within the Decision Workspace as the evidence layer — not as the primary navigation or the first thing a portfolio manager sees.

---

*ChronoSentiment Product Blueprint v1.0 | July 2026*
*Describes the evolution of the existing ChronoSentiment execution platform into a Financial Decision Management Platform.*
*Review trigger: Phase 1B customer validation results; design partner feedback; material change in product direction.*