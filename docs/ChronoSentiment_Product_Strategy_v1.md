# ChronoSentiment Product Strategy v1.0

**Document type:** Product Strategy
**Version:** 1.0
**Status:** Draft
**Date:** 2026-07-26
**Owner:** Product / Commercial

---

## Document Governance

| Field | Value |
|-------|-------|
| Document Status | Draft |
| Next Review | After Phase 1B completion |
| Review Trigger | Phase 1B go/no-go decision; material change in competitive landscape; design partner feedback that challenges strategic assumptions |

**Relationship to other documents:**
- Sits above: ChronoSentiment Product Blueprint v1.0 (product definition)
- Informed by: CS-R-001 through CS-R-015A (research programme)
- Connects to: CS-R-015 Investment Thesis (commercial rationale)

---

## Purpose

This document answers the strategic questions that the Product Blueprint does not. The Blueprint answers *what we are building and how it works*. This document answers *why this market, why now, why us, and why this will be hard to replicate*.

The central strategic claim is:

> ChronoSentiment is building the institutional memory layer for investment decisions — a category that does not yet exist, at a moment when the forces creating demand for it are converging for the first time.

---

## 1. Why Investment Management First

ChronoSentiment could, in principle, be built for any knowledge-intensive profession where consequential decisions are made, documented poorly, and reviewed later. Law firms, medical institutions, corporate boards, government agencies — all have the same structural problem.

The choice to start with investment management is deliberate and strategic.

### The problem is most acute here

Investment management has a unique combination of characteristics that makes the decision documentation problem both more painful and more urgent than in other domains:

**High decision frequency.** A portfolio manager makes 3–8 consequential investment decisions per week. A law firm partner makes 3–8 consequential decisions per year. The volume of decisions that need to be documented is an order of magnitude higher in investment management.

**High stakes per decision.** A single investment decision can represent tens or hundreds of millions of pounds. The cost of a poorly documented decision — in LP relations, regulatory exposure, or institutional learning — is correspondingly high.

**Explicit accountability.** Investment managers are explicitly accountable for their decisions to LPs, regulators, and boards. The accountability structure creates demand for documentation that does not exist in the same form in other professions.

**AI adoption is already happening.** Investment teams are already using AI tools in their research and decision process. The documentation gap is already open. In other professions, AI adoption is earlier-stage and the documentation gap has not yet become visible.

**Regulatory pressure is immediate.** The EU AI Act, FCA guidance, and SEC commentary are creating explicit requirements for AI documentation in investment decisions now. In other professions, the regulatory pressure is less immediate.

### The beachhead is well-defined

The target segment — independent asset managers with £500M–£5B AUM — is large enough to build a sustainable business, small enough to be reachable without enterprise sales infrastructure, and homogeneous enough that a single product can serve the segment well.

This is not the entire addressable market. It is the beachhead from which the market can be expanded.

### The reference value is high

A reference customer in investment management carries significant weight. CIOs talk to each other. A single credible reference at a respected mid-market asset manager is worth more than ten references in a less networked industry.

---

## 2. Why Not Other Knowledge-Work Domains

The following domains have the same structural problem but are not the right starting point.

| Domain | Why not first |
|--------|--------------|
| **Corporate boards** | Low decision frequency; long sales cycles; decisions are already documented (board minutes); regulatory pressure is lower |
| **Law firms** | Decisions are already documented (case files, legal opinions); professional obligation to document; different AI adoption pattern |
| **Medical institutions** | Decisions are already documented (clinical records); heavy regulatory environment creates compliance infrastructure; different buyer |
| **Government agencies** | Procurement cycles are 18–36 months; political complexity; not a commercial market |
| **Management consulting** | Decisions are client-owned, not firm-owned; different accountability structure; lower AI adoption urgency |

Investment management is the right starting point because the problem is most acute, the buyer is most accessible, and the reference value is highest.

**Adjacent markets after investment management:** Private equity (similar decision structure, higher ACV potential), hedge funds (higher AI adoption, faster sales cycles), corporate treasury (similar decision frequency, lower stakes), family offices (similar to asset management, higher ACV potential). These are Phase 2 and Phase 3 expansion markets, not Phase 1 targets.

---

## 3. Defensibility

The most important strategic question for any software product is: why won't a larger, better-resourced competitor simply build this?

The answer is not that it is technically difficult. The answer is that the defensibility comes from five reinforcing barriers that compound over time.

### Barrier 1 — The Decision Archive

The longer a firm uses ChronoSentiment, the more valuable the product becomes. After one year, the firm has a searchable archive of every investment decision, with full provenance, outcome data, and lessons captured. After three years, the archive is a strategic asset — a record of the firm's institutional knowledge that cannot be reconstructed from scratch.

This is a data network effect at the firm level. The product becomes more valuable with use, and switching costs increase as the archive grows.

A new entrant cannot replicate a three-year decision archive. They can only offer a blank slate.

### Barrier 2 — Workflow Integration

ChronoSentiment integrates into the investment workflow at the point of decision creation — the moment when the portfolio manager forms a thesis. This is the highest-value integration point in the workflow, and it is the hardest to displace once established.

Once a firm's investment committee process runs through ChronoSentiment, replacing it requires changing the committee workflow — a high-friction, politically sensitive change that firms are reluctant to make.

### Barrier 3 — The Knowledge Graph

As the decision archive grows, ChronoSentiment can surface patterns across decisions — recurring assumption failures, sectors where conviction is systematically miscalibrated, AI tools that contributed to decisions that underperformed. This cross-decision intelligence becomes more valuable as the archive grows and is impossible to replicate without the archive.

### Barrier 4 — Integration Depth

ChronoSentiment integrates with the data sources that investment teams already use — Bloomberg, FactSet, AlphaSense, OMS/PMS systems. Each integration increases switching costs and reduces the likelihood that a firm will replace the product.

### Barrier 5 — Evidence Standards

The evidential discipline built into ChronoSentiment — point-in-time reconstruction, certification state, causal provenance — creates audit-grade documentation that meets regulatory requirements. Once a firm has built its regulatory compliance posture around ChronoSentiment's documentation, replacing it requires rebuilding that posture from scratch.

---

## 4. Why Won't Microsoft, Bloomberg, or OpenAI Build This?

This is the most common investor objection to enterprise software startups. It deserves a direct answer.

### Microsoft

Microsoft has Copilot for Finance and a broad enterprise AI strategy. They could, in principle, add decision documentation to their financial services products.

**Why they are unlikely to prioritise this:** Microsoft's financial services products are horizontal — they serve all financial services firms, not investment management specifically. The investment management workflow (thesis formation, committee governance, LP reporting, regulatory documentation) is sufficiently specialised that a horizontal product cannot serve it well. Microsoft's primary strategic incentive is to sell Azure and Microsoft 365 seats; building deep workflow integration for a specialised segment is unlikely to rank highly against those priorities.

More importantly, Microsoft does not have the temporal reconstruction capability — the ability to reconstruct the exact information environment at the time of a decision. This is the hardest technical problem in decision documentation, and it requires a purpose-built architecture. ChronoSentiment already has it.

### Bloomberg

Bloomberg has deep penetration in investment management and could add decision documentation to the Bloomberg Terminal.

**Why they are unlikely to prioritise this:** Bloomberg's primary strategic focus has historically been market data and analytics rather than end-to-end investment decision management workflows. Their incentive is to sell data subscriptions, not to build decision management workflows. Purpose-built workflow tools — portfolio management, research management — have generally been more successful when built by specialist vendors rather than data platforms, and investment decision management is a workflow problem, not a data problem.

More importantly, Bloomberg does not have the AI conversation import capability, the committee workflow, or the post-mortem analysis layer. These are not data problems — they are workflow problems that require a different product philosophy.

### OpenAI / Anthropic

AI labs could build decision documentation tools on top of their models.

**Why they won't:** AI labs are infrastructure companies, not application companies. Their incentive is to sell API access, not to build vertical applications. The investment management workflow requires deep domain knowledge, regulatory expertise, and integration with financial data sources that AI labs do not have and are not building.

More importantly, the hardest problem in investment decision documentation is not generating natural language — it is temporal reconstruction (what information was available at the time of the decision) and causal provenance (what caused what). These are engineering problems, not AI problems. ChronoSentiment already has them solved.

### The real competitive threat

The real competitive threat is not from large incumbents. It is from a well-funded startup that identifies the same opportunity and executes faster. The defence against this threat is speed to design partners, depth of workflow integration, and the decision archive moat.

---

## 5. Long-Term Platform Vision

ChronoSentiment's long-term vision is to become the institutional memory layer for investment decisions — the system of record for how investment organisations think, decide, and learn.

### Phase 1 — Decision Management (MVP)

The product captures, explains, and reviews individual investment decisions. The value proposition is operational: faster LP reporting, better committee governance, regulatory compliance.

### Phase 2 — Decision Intelligence

The product surfaces patterns across the decision archive — recurring assumption failures, sectors where conviction is miscalibrated, AI tools that contributed to underperforming decisions. The value proposition shifts from operational to strategic: the product actively improves future decisions.

### Phase 3 — Institutional Memory

The product becomes the firm's institutional memory — a searchable, queryable record of how the firm thinks. New team members onboard by reading the decision archive. The CIO uses it to understand the firm's decision-making tendencies. The board uses it to evaluate the investment process.

### Phase 4 — Network Intelligence (long-term, opt-in)

With sufficient adoption, ChronoSentiment can offer anonymised benchmarking — how does a firm's decision-making compare to peers? Which assumption types fail most commonly across the industry? This requires a critical mass of firms and is a Phase 4 capability, not a near-term priority.

---

## 6. Adjacent Markets

After establishing the investment management beachhead, the following adjacent markets are natural expansions.

| Market | Timing | Rationale |
|--------|--------|-----------|
| **Private equity** | Phase 2 | Similar decision structure (investment committee, LP reporting, regulatory documentation); higher ACV potential (larger firms, higher stakes per decision) |
| **Hedge funds** | Phase 2 | Higher AI adoption; faster sales cycles; higher willingness to pay; different regulatory environment |
| **Family offices** | Phase 2 | Similar to asset management; higher ACV potential; less price-sensitive |
| **Corporate treasury** | Phase 3 | Similar decision frequency; lower stakes per decision; different buyer (CFO, not CIO) |
| **Pension funds / endowments** | Phase 3 | Large AUM; high governance requirements; long sales cycles; high ACV potential |
| **Sovereign wealth funds** | Phase 4 | Very large AUM; very high governance requirements; very long sales cycles; very high ACV potential |

**Deliberate out-of-scope (Phase 1):** Retail investment platforms, robo-advisors, trading desks, corporate M&A, legal, medical, government. These markets have different buyers, different workflows, and different regulatory environments. Pursuing them in Phase 1 would dilute focus without proportionate return.

---

## 7. What Is Deliberately Out of Scope

The following capabilities are deliberately excluded from the MVP and near-term roadmap. They are excluded not because they are unimportant, but because including them would compromise the focus required to build a product that investment teams actually use.

| Out of scope | Rationale |
|-------------|-----------|
| **Trade execution** | ChronoSentiment documents decisions; it does not execute them. Integration with OMS/PMS is in scope; replacing them is not. |
| **Portfolio analytics** | Bloomberg, FactSet, and purpose-built portfolio analytics tools already do this well. ChronoSentiment is not a data platform. |
| **Research generation** | AI research tools (AlphaSense, Sentieo, ChatGPT) already generate research. ChronoSentiment documents how that research was used in decisions. |
| **Automated investment recommendations** | ChronoSentiment surfaces relevant past decisions; it does not make investment recommendations. The product is a decision management tool, not an investment advisor. |
| **Retail investor features** | The product is designed for professional investment teams, not retail investors. |
| **Real-time market data** | ChronoSentiment captures point-in-time market data snapshots at the moment of decision creation. It is not a real-time data platform. |

---

## 8. The Strategic Bet

ChronoSentiment is making one central strategic bet:

> Investment teams will pay for a product that preserves institutional decision memory — and the combination of AI adoption, regulatory pressure, and staff turnover has made this problem urgent enough to drive purchasing decisions in 2026–2027.

This bet is falsifiable. Phase 1B is designed to test it. If the bet is wrong — if investment teams do not confirm the problem is urgent, or if willingness to pay is insufficient — the programme will reposition or terminate rather than proceed to MVP.

If the bet is right, ChronoSentiment has a significant first-mover advantage in a category that is being created now, with defensibility that compounds over time as the decision archive grows.

---

## 9. Two-Product-Line Architecture

ChronoSentiment is not one product. It is two products built on one platform philosophy.

The enterprise documents written to date — the Product Blueprint, the Category Definition, the Customer Success Blueprint — describe ChronoSentiment Enterprise: institutional Financial Decision Management for asset managers, hedge funds, and family offices.

A second product line, ChronoSentiment Personal, addresses a different market with a different value proposition but the same underlying philosophy: **the portfolio is the context, and the decision is the unit of value.**

### ChronoSentiment Personal

**Target:** Individual long-term investors

**Core proposition:** ChronoSentiment Personal is an AI-powered personal investment research and decision journal. It helps investors organise research, evaluate investment theses, track assumptions, and learn from past decisions. It does not tell investors what to buy or sell. It helps them think better.

**Positioning:** Your personal investment research workspace — the investment equivalent of Notion for research, Obsidian for knowledge management, GitHub for versioning investment theses.

**What makes it different from existing apps:**

Most research platforms stop at the decision. ChronoSentiment Personal continues the loop — from research through thesis, decision, outcome, and lesson, back to better research. Every completed investment teaches the system something. Over time, the platform becomes a personal investment learning system whose value compounds with every decision and every review cycle.

**Core capabilities:**

Research dossier builder (structured research notes organised by thesis, evidence, assumptions, risks, and questions), AI-assisted document summarisation, company comparison, thesis versioning (the thesis as a living document, revised as new information arrives), portfolio observations (not recommendations — surfacing information the investor needs to draw their own conclusions), and a Decision Journal that records every investment decision with its thesis, evidence, and outcome.

**The six-level feedback loop:**

```
Level 1 — Thesis feedback:    Did the thesis hold up?
Level 2 — Portfolio feedback: Did the investment improve the portfolio?
Level 3 — Process feedback:   How does this investor research?
Level 4 — Thesis evolution:   How do theses change over time?
Level 5 — Research quality:   Which sources actually improve decisions?
Level 6 — Investor behaviour: What patterns recur in this investor's decisions?
```

**The moat — Personal Investment Learning Loop:**

After years of use, another platform can import a user's portfolio and documents — but it cannot recreate years of accumulated learning about how that specific investor thinks and improves. The moat is not the archive. It is the learning.

**Validation advantage:** ChronoSentiment Personal can be validated by the founder using it daily. Every investment decision becomes a test of the research platform. Every outcome becomes training data. The founder is the first power user — a significant advantage over enterprise validation, which requires design partners.

---

### ChronoSentiment Enterprise

**Target:** Asset managers, hedge funds, family offices, private equity

**Core proposition:** Institutional Financial Decision Management — the system of record for investment decision reasoning, governance, and institutional memory.

This is the product described in the existing strategy documents: Decision Workspace, Decision Record, Decision Memory, Decision Governance, Decision Archive, Decision Intelligence, LP reporting, AI governance, regulatory evidence, committee workflows.

---

### The Shared Platform — Coralys

Both products are built on the same underlying platform: **Coralys**, the Knowledge Evolution Platform.

ChronoSentiment Personal and ChronoSentiment Enterprise are domain adapters over Coralys. Neither product introduces a separate architecture. Coralys provides the generic capabilities — workspaces, evidence management, hypothesis tracking, reviews, timelines, pattern extraction, and learning. Each product supplies the domain-specific semantics.

**Platform hierarchy:**

```
                    Coralys Platform
         (Knowledge Evolution Platform)

           ┌────────────┼────────────┐
           │            │            │
           ▼            ▼            ▼

     UltraCrew    ChronoSentiment   Future Products
                  Enterprise

                        ▲
                        │
           ChronoSentiment Personal
           (Investment Research Adapter)
```

**Concept mapping across products:**

| Coralys Core | ChronoSentiment Personal | ChronoSentiment Enterprise |
|-------------|--------------------------|---------------------------|
| Workspace | Research Workspace | Decision Workspace |
| Subject | Company | Investment opportunity |
| Context | Portfolio | Fund / mandate |
| Evidence | Research sources | Decision evidence |
| Hypothesis | Investment thesis | Investment thesis |
| Review | Research Review | Committee review |
| Timeline | Research Timeline | Decision Timeline |
| Outcome | Investment outcome | Decision outcome |
| Memory | Research Memory | Decision Memory |
| Intelligence | Research Intelligence | Decision Intelligence |

Coralys does not know about stocks, portfolios, or investment committees. The adapters supply the semantics. This separation ensures that the core platform remains domain-neutral while allowing both products — and future products in other domains — to share the same underlying capabilities.

**Strategic implication:** Coralys is the enduring platform asset. ChronoSentiment is the first rich domain. The same platform can later serve medical research, corporate strategy, M&A, procurement, engineering design reviews, and scientific research without changing the Coralys core.

---

### Sequencing

The recommended build sequence is:

**Stage 1 — ChronoSentiment Personal (now)**
Build the portfolio-aware decision engine for individual investors. Validate the decision framework with real users. Accumulate evidence from actual investment decisions. The founder is the first user.

**Stage 2 — ChronoSentiment Professional (Phase 2)**
Extend Personal with collaboration features for investment clubs, advisors, and small family offices. The same decision engine, with shared portfolios and multi-user governance.

**Stage 3 — ChronoSentiment Enterprise (Phase 3)**
The full institutional platform: Decision Workspaces, committee governance, LP reporting, regulatory documentation, AI governance, institutional Decision Memory. The enterprise product is built on the same decision engine, scaled to institutional requirements.

This sequencing gives ChronoSentiment a practical path to product-market learning (Personal), a bridge to institutional markets (Professional), and a clear long-term destination (Enterprise) — without requiring enterprise sales infrastructure or design partners to begin.

---

*ChronoSentiment Product Strategy v1.3 | July 2026*
*Updated: Section 9 — Coralys platform identity revised to Knowledge Evolution Platform; platform hierarchy diagram updated.*
*Sits above the Product Blueprint. Answers why this market, why now, why us, and why this will be hard to replicate.*
*Review trigger: Phase 1B go/no-go decision; material change in competitive landscape.*