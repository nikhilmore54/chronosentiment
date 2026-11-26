# ChronoSentiment Personal — Product Strategy

**Document type:** Product Strategy
**Version:** 1.0
**Status:** Baseline
**Date:** 2026-07-26
**Owner:** Strategy / Product

---

## Document Governance

| Field | Value |
|-------|-------|
| Document Status | Baseline v1.0 |
| Review Trigger | Material change in product positioning, target market, or competitive landscape |

**Relationship to other documents:**
- Informed by: `CORALYS_PLATFORM_ARCHITECTURE.md` (platform architecture)
- Informed by: `CORALYS_PLATFORM_STRATEGY.md` (platform portfolio positioning)
- Informed by: `ChronoSentiment_Product_Strategy_v1.md` (combined strategy — predecessor document)
- Informed by: `ChronoSentiment_Personal_Blueprint_v1.md` (product blueprint v1.1)
- Informed by: CS-R-001 through CS-R-015A (research programme)
- Informs: ChronoSentiment Personal Product Blueprint (to be written as a standalone document)

---

## Purpose

This document defines the product strategy for ChronoSentiment Personal — what it is, who it is for, what problem it solves, and how it is positioned commercially. It is a product-specific strategy document, derived from the combined ChronoSentiment Product Strategy and refined to reflect the Personal product's distinct identity, buyer, and value proposition.

---

## Product Identity

**Product name:** ChronoSentiment Personal
**Commercial positioning:** Personal Investment Knowledge Platform
**Platform:** Coralys Knowledge Evolution Platform
**Target audience:** Individual investors who take their investment research seriously

---

## What Makes ChronoSentiment Personal Different

ChronoSentiment Personal is the exception in the Coralys product portfolio.

In every other product — UltraCrew, ChronoSentiment Enterprise — the Coralys platform is hidden. The customer sees better decisions, better schedules, better outcomes. The knowledge layer is the engine behind the product, not the interface.

In ChronoSentiment Personal, the knowledge layer is front and centre.

This is because the customer's goal is different. An individual investor is not trying to make a single better decision. They are trying to become a better investor over time. The Research Workspace, the Investment Thesis, the Research Timeline, and the Personal Investment Learning Loop are not infrastructure — they are the product.

The customer's goal is knowledge evolution. The platform's capabilities are the product's features.

---

## The Problem ChronoSentiment Personal Solves

Individual investors face a structural knowledge problem that compounds over time:

**Research is scattered.** Notes are in a notebook, a spreadsheet, a browser tab, an email thread. There is no structured place to accumulate research on a company over time.

**Thinking is not tracked.** When an investor revisits a company six months later, they cannot see how their thinking has evolved. They cannot see what they believed before, what changed, and why.

**Decisions are not reviewed.** Most investors make a decision, execute it, and move on. They do not systematically review their decisions against their original thesis. They do not capture what they learned.

**AI conversations are lost.** Investors are increasingly using AI tools in their research. The AI's contribution to the research is invisible — there is no record of what the AI said, what the investor accepted, what they rejected, and why.

**The learning loop is broken.** Investors make decisions, experience outcomes, but rarely capture the lessons in a form that improves future decisions. The same mistakes recur. The same research is repeated.

ChronoSentiment Personal solves this by providing a structured environment for personal investment research — from initial research through thesis formation to outcome recording and personal learning.

---

## What ChronoSentiment Personal Is

ChronoSentiment Personal is a **Personal Investment Knowledge Platform**. It provides individual investors with:

- **Research Workspace** — a structured environment for each investment, from initial research through thesis formation and outcome
- **Research Dossier** — a structured, accumulated record of all research on a company
- **Investment Thesis** — a versioned, structured statement of what the investor believes and why
- **Research Timeline** — a chronological record of how the investor's thinking evolved
- **Research Reviews** — structured quarterly reviews of active theses against new evidence
- **Personal Investment Learning Loop** — a post-outcome review process that captures lessons and improves future research
- **Personal Investment Knowledge Graph** — a network of companies, sectors, and investment insights that accumulates over time

---

## Why ChronoSentiment Personal Is Distinctive

Most investment platforms for individual investors are data platforms. They provide market data, news, financial statements, and analyst reports. They help investors find information. They do not help investors build knowledge.

ChronoSentiment Personal is not a data platform. It is a knowledge platform. The distinction matters:

| Data platform | Knowledge platform |
|--------------|-------------------|
| Provides information | Helps build understanding |
| Stops at the decision | Runs a continuous loop |
| No memory of past research | Accumulates research over time |
| No tracking of how thinking evolved | Tracks thesis evolution |
| No learning from outcomes | Captures lessons from outcomes |
| No personal knowledge graph | Builds a personal investment knowledge graph |

The combination of Research Workspace, Investment Thesis, Research Timeline, Learning Loop, and Personal Knowledge Graph is distinctive and difficult to replicate — because it is built around the accumulation of reasoning, not just the presentation of data.

---

## Target Market

**Primary segment:** Serious individual investors who actively research their own investments — not passive index investors, not day traders. These are investors who read annual reports, follow earnings calls, and form their own investment theses. They are typically:

- Self-directed investors with £50,000–£5,000,000 in investable assets
- Investors who spend 5–20 hours per week on investment research
- Investors who have been investing for 3+ years and have developed a research process
- Investors who are frustrated by the lack of structure in their current research workflow

**Secondary segment:** Investment professionals who want a personal research tool separate from their firm's systems — analysts who want to track their own investment ideas, portfolio managers who want a personal research journal.

**Out of scope (v1.0):** Passive investors, day traders, and investors who do not conduct their own research.

---

## Commercial Positioning

**Personal Investment Knowledge Platform.**

The customer buys a better way to build, organise, and improve their own investment knowledge over time:

- Better research — structured research dossiers that accumulate over time
- Better thinking — versioned investment theses that track how their thinking evolves
- Better reviews — structured quarterly reviews that keep theses current
- Better learning — a personal learning loop that captures lessons from every investment
- Better knowledge — a personal investment knowledge graph that grows with every research cycle

The Coralys platform is front and centre — because the customer's goal is knowledge evolution, not just decision quality.

---

## Why ChronoSentiment Personal Is Different

| Competitor | What they do | Why ChronoSentiment Personal is different |
|------------|-------------|------------------------------------------|
| Bloomberg Terminal | Market data and news | No personal research workspace; no thesis management; no learning loop |
| Seeking Alpha | Investment ideas and analysis | No personal research workspace; no thesis versioning; no learning loop |
| Substack / newsletters | Investment commentary | No personal research workspace; no structured research |
| Notion / Obsidian | General note-taking | No investment-specific structure; no learning loop; no knowledge graph |
| Roam Research | Knowledge management | No investment-specific structure; no learning loop |
| Generic AI tools | AI assistance | No research workspace; no provenance; no learning loop |
| Spreadsheets | Portfolio tracking | No research structure; no thesis management; no learning loop |

ChronoSentiment Personal is the only product that treats the individual investor's research as a structured, evolving knowledge asset — not just a collection of notes or a portfolio tracker.

---

## Coralys Platform Realisation

ChronoSentiment Personal is the one product in the Coralys portfolio where the platform's knowledge-centric nature is front and centre. The platform provides:

- **Lifecycle governance** — every investment research cycle is a Research Workspace with a structured lifecycle
- **Continuous Learning Engine** — every completed research cycle contributes to personal investment knowledge
- **Knowledge Graph** — companies, sectors, and investment insights accumulate across all research cycles
- **Domain Adapter Model** — ChronoSentiment Personal configures the platform with personal investment vocabulary; the platform provides the lifecycle

The Coralys adapter vocabulary for ChronoSentiment Personal:

| Coralys Primitive | ChronoSentiment Personal |
|------------------|--------------------------|
| Workspace | Research Workspace |
| Actor | Individual investor |
| Intent | Research objective (e.g. "Evaluate Reliance Industries as a long-term holding") |
| Subject | Company |
| Context | Portfolio |
| Evidence | Research Sources (annual reports, earnings calls, AI conversations, news) |
| Hypothesis | Investment Thesis |
| Hypothesis version | Thesis version (v1, v2, v3...) |
| Review | Quarterly Research Review |
| Timeline | Research Timeline |
| Outcome | Investment Outcome |
| Pattern | Investor Behaviour Pattern / Research Quality Score |
| Learning | Personal Investment Learning Loop |
| Knowledge Graph | Personal Investment Knowledge Graph |

**Continuous Learning Engine realisation:** Personal Investment Learning Loop

---

## Go-to-Market

**Primary channel:** Product-led growth (PLG) — free tier with upgrade to paid
**Secondary channel:** Content marketing (investment research methodology, knowledge management for investors)
**Pricing model:** Freemium (free tier with limited workspaces; paid tier with unlimited workspaces, full knowledge graph, and learning loop)
**Target price point:** £10–£30 per month (individual)

**Entry motion:** Free tier — Research Workspace and Investment Thesis (fastest time-to-value for individual investors). Upgrade to paid for unlimited workspaces, Research Timeline, Learning Loop, and Personal Investment Knowledge Graph.

**Retention motion:** The Personal Investment Knowledge Graph creates switching costs that increase over time. The longer an investor uses ChronoSentiment Personal, the more their personal investment knowledge is embedded in the product.

---

## Roadmap Principles

1. **Research first** — every feature is evaluated by its contribution to the structured investment research lifecycle.
2. **Knowledge accumulation as the moat** — features that make the Personal Investment Knowledge Graph richer over time are more valuable than features that provide one-time utility.
3. **AI as research assistant, not decision-maker** — AI helps the investor research better; it does not make decisions for them.
4. **Provenance by default** — every piece of information in the system has a traceable source.
5. **Learning loop as the differentiator** — the Personal Investment Learning Loop is what makes ChronoSentiment Personal distinctive. It must be excellent.

---

## Roadmap

| Phase | Features | Status |
|-------|----------|--------|
| v1.0 | Research Workspace, Research Dossier, Investment Thesis with versioning, Research Timeline, Quarterly Research Review | Documented in Blueprint v1.1; implementation in progress |
| v1.1 | Personal Investment Learning Loop, Personal Investment Knowledge Graph, AI conversation documentation | Planned |
| v2.0 | Research quality scoring, investor behaviour patterns, cross-investor benchmarking (anonymised) | Planned |

---

*ChronoSentiment Personal Product Strategy v1.0 | July 2026 | Status: Baseline*
*Defines product strategy for ChronoSentiment Personal — Personal Investment Knowledge Platform.*
*Derived from `ChronoSentiment_Product_Strategy_v1.md` (combined strategy) and refined for the Personal product.*
*Review trigger: Material change in product positioning, target market, or competitive landscape.*