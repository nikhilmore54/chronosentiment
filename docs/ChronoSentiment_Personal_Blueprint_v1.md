# ChronoSentiment Personal — Product Blueprint v1.1

**Document type:** Product Blueprint
**Version:** 1.1
**Status:** Draft
**Date:** 2026-07-26
**Change from v1.0:** Research-first vocabulary throughout. Research Workspace introduced. Decision Journal renamed Research Timeline. Research Reviews added. Research Maturity model added. Research Graph added. Continuous Research Loop elevated to top of document.
**Owner:** Product

---

## Document Governance

| Field | Value |
|-------|-------|
| Document Status | Draft |
| Next Review | After first 30 days of founder self-use |
| Review Trigger | Founder usage evidence; first external user feedback; Phase 1B personal product validation |

**Relationship to other documents:**
- Sits within: ChronoSentiment Product Strategy v1.1 (Section 9 — Two-Product-Line Architecture)
- Shares philosophy with: ChronoSentiment Product Blueprint v1.0 (Enterprise)
- Informed by: CS-R-001 through CS-R-015A (research programme)

**Vocabulary symmetry with Enterprise:**

| Personal | Enterprise |
|----------|-----------|
| Research Workspace | Decision Workspace |
| Research Dossier | Decision Record |
| Research Timeline | Decision Timeline |
| Research Memory | Decision Memory |
| Research Intelligence | Decision Intelligence |
| Research Reviews | Committee Reviews |

---

## Purpose

ChronoSentiment Personal is an AI-powered personal investment research platform. It helps investors organise research, build structured research dossiers, track how investment theses evolve, conduct periodic research reviews, and learn from the outcomes of their decisions over time.

It is not a stock recommendation platform. It does not tell investors what to buy or sell. It does not produce target prices or expected returns. It does not make decisions for the investor.

**Positioning statement:**

> ChronoSentiment Personal is your personal investment research workspace. It helps you build better research, track how your thinking evolves, and learn from every investment you make.

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
Lessons captured (what did the research miss?)
        ↓
Research improves (better dossiers next time)
        ↓
Research Workspace (next investment)
```

The loop never ends. Every completed investment teaches the system — and the investor — something. Over time, the platform becomes a **personal investment learning system** whose value compounds with every research cycle.

---

## The Core Object — Research Dossier

The Research Dossier is the centre of ChronoSentiment Personal. Everything else revolves around it.

```
Research Dossier
        ↓
Research Reviews (quarterly updates)
        ↓
Research Timeline (full history of how the dossier evolved)
        ↓
Research Memory (the accumulated archive of all dossiers)
        ↓
Research Intelligence (patterns across the archive)
```

A Research Dossier is a structured, living research package for a specific company. It is not a static document — it evolves as new information arrives, as assumptions are tested, and as the investment thesis is revised.

### Research Dossier structure

```
RESEARCH DOSSIER

Company:          Reliance Industries
Research Maturity: Level 4 — Competitive Analysis
Last updated:     [Date]
Status:           Active — held since March 2026

─────────────────────────────────────────

PORTFOLIO CONTEXT

Current weight:       14%
Sector exposure:      Energy: 22% of portfolio
Original allocation:  10%
Holding period:       4 months

─────────────────────────────────────────

INVESTMENT THESIS (recorded at time of decision)

"The retail business is significantly undervalued relative
to peers. Jio's ARPU growth will accelerate. The energy
business provides a floor on valuation."

─────────────────────────────────────────

CURRENT OBSERVATIONS

• Valuation higher than at purchase (P/E 28x vs 22x)
• Retail margins improving — thesis tracking
• Jio ARPU growth: +8% YoY — below 10% expectation
• Energy business: revenue flat — thesis partially intact

─────────────────────────────────────────

ASSUMPTION TRACKER

Assumption 1: Retail margins will expand
Status: ✓ Tracking — margins up 120bps

Assumption 2: Jio ARPU will grow >10% YoY
Status: ⚠ Partial — growth at 8%, below expectation

Assumption 3: Energy business stable
Status: ✓ Tracking — revenue flat but not declining

─────────────────────────────────────────

PORTFOLIO OBSERVATIONS

• Position has grown from 10% to 14% due to price appreciation
• Energy sector now at 22% of portfolio
• Increasing the position would raise energy allocation to 29%

─────────────────────────────────────────

OPEN QUESTIONS

• Is retail growth sustainable at current margins?
• How much upside remains at current valuation?
• What are analysts missing about the Jio business?

─────────────────────────────────────────

RESEARCH SOURCES

Annual Report 2025–26 (read: June 2026)
Q4 Earnings Call Transcript (read: May 2026)
Analyst report — Kotak Securities (read: June 2026)
News: 12 articles since purchase

─────────────────────────────────────────

RESEARCH TIMELINE

March 2026: Dossier created. Thesis recorded. Position initiated.
May 2026:   Q4 results reviewed. Jio ARPU below expectation noted.
            Thesis v2 recorded.
June 2026:  Annual report reviewed. Retail margin thesis confirmed.
            Thesis v3 recorded.
July 2026:  Quarterly research review. Assumption 2 flagged partial.
```

Notice what is absent: no buy/sell recommendation, no target price, no expected return. The dossier helps the investor conduct a higher-quality review. The investor draws their own conclusion.

---

## The Research Workspace

The Research Workspace is the working environment where a Research Dossier is built and maintained. It is the personal equivalent of the Enterprise Decision Workspace.

A Research Workspace contains:

- The Research Dossier (structured research notes)
- Attached documents (annual reports, earnings transcripts, analyst reports, PDFs)
- AI conversation exports (research conversations with ChatGPT, Claude, or other tools)
- Assumptions log
- Risk register
- Open questions list
- Research review history
- Portfolio context snapshots

The Research Workspace is persistent — it does not close when the investor makes a decision. It continues to accumulate evidence throughout the investment lifecycle, from initial research through exit and post-mortem.

---

## Research Maturity

Every Research Dossier has a maturity level. Maturity measures how complete the research is — not whether the investor should buy or sell.

```
Level 0 — Idea
  Company identified. No research conducted yet.

Level 1 — Initial Reading
  Basic financials reviewed. Company understood at a high level.

Level 2 — Financial Review
  Detailed financial analysis completed. Valuation assessed.
  Key ratios reviewed. Historical performance understood.

Level 3 — Management Review
  Annual report read. Earnings calls reviewed.
  Management track record assessed. Capital allocation understood.

Level 4 — Competitive Analysis
  Competitive position assessed. Industry dynamics understood.
  Peer comparison completed. Moat evaluated.

Level 5 — Research Ready
  All major assumptions documented. Key risks identified.
  Open questions resolved or explicitly accepted.
  Investment thesis fully articulated.

Level 6 — Post-Investment Monitoring
  Position held. Quarterly research reviews active.
  Thesis being tested against reality.

Level 7 — Completed
  Position exited. Outcome recorded. Lessons captured.
  Dossier archived in Research Memory.
```

Research Maturity does not say "Buy" or "Sell." It says: how complete is your research? An investor who reaches Level 5 has done the work. What they decide to do with that research is their own choice.

---

## Research Reviews

Every quarter, ChronoSentiment Personal prompts the investor to review each active Research Dossier. The review is structured around research questions, not investment questions.

**Quarterly Research Review questions:**

- Has anything material changed since the last review?
- Which assumptions have been confirmed, partially confirmed, or broken?
- What new evidence has arrived?
- What questions remain open?
- Is the investment thesis stronger or weaker than at the last review?
- Has the portfolio context changed (position size, sector exposure)?

The review produces a Research Review record — a timestamped snapshot of the dossier's state at that point in time. The Research Timeline is built from these review records.

**What the review does not ask:**

- Should I buy more?
- Should I sell?
- What is the target price?

Those are investment decisions. The review is a research maintenance exercise.

---

## The Research Timeline

The Research Timeline is the complete history of how a Research Dossier evolved — every revision, every review, every new piece of evidence, every assumption update.

```
Research Timeline — Reliance Industries

March 2026
  Dossier created. Thesis v1 recorded.
  "Retail undervalued. Jio ARPU growth. Energy floor."
  Research Maturity: Level 2

May 2026
  Q4 results reviewed. Jio ARPU below expectation.
  Assumption 2 flagged partial.
  Thesis v2: "Monitoring Jio ARPU trajectory."
  Research Maturity: Level 3

June 2026
  Annual report reviewed. Retail margin thesis confirmed.
  Competitive analysis completed.
  Thesis v3: "Retail thesis intact. Jio uncertain."
  Research Maturity: Level 4

July 2026
  Quarterly research review.
  Assumption 2 remains partial. No new evidence on Jio.
  Open question added: "Is Jio ARPU guidance reliable?"
  Research Maturity: Level 4 (unchanged)
```

The Research Timeline is the investment equivalent of a Git commit history. The investor can see exactly how their thinking evolved, what information caused each revision, and whether the current thesis is materially different from the original.

---

## The AI Research Assistant

The AI in ChronoSentiment Personal functions as a research assistant, not a decision-maker. It answers research questions, not investment questions.

**Research questions the AI can answer:**

> "Summarise the latest quarterly report for Reliance."

> "Compare Infosys and TCS on revenue growth and margins over the last five years."

> "Explain why the P/E ratio has expanded since I initiated this position."

> "Compare the current state of the company with my original investment thesis."

> "Highlight assumptions in my thesis that have changed based on recent results."

> "Show all negative news about this company since my research start date."

> "Find companies in the healthcare sector with similar characteristics to my existing holdings."

> "What questions should I investigate before my next research review?"

**What the AI does not do:**

The AI does not say "Buy this." It does not say "Sell immediately." It does not produce target prices or expected returns. It does not allocate portfolio weights. It does not make decisions for the investor.

The investor owns the decision. The AI improves the research that informs it.

---

## Portfolio Observations

ChronoSentiment Personal provides portfolio-level observations — not recommendations.

The distinction is important. An observation surfaces information and lets the investor draw their own conclusion. A recommendation tells the investor what to do.

**Examples of observations (not advice):**

> "Infosys now represents 19% of your portfolio. You originally allocated 10%. You may wish to review whether this concentration still aligns with your investment objectives."

> "Adding ICICI Bank would increase financial-sector exposure from 21% to 29%. Compare this with your target allocation before making a decision."

> "Your portfolio currently has no healthcare exposure."

> "Three of your five largest positions are highly correlated (correlation > 0.75). This may reduce the diversification benefit of holding them separately."

These observations give the investor information they need to make a better decision. They do not make the decision for the investor.

---

## The Research Graph

Over time, Research Dossiers do not exist in isolation. They form a network — a personal investment knowledge graph.

```
Reliance Industries
        ↓
    Jio Platform
        ↓
    Telecom sector
        ↓
    5G infrastructure
        ↓
    Bharti Airtel
        ↓
    Tower companies
        ↓
    Semiconductor supply chain
        ↓
    TSMC
```

Every piece of research links to related research. An insight about Jio's ARPU growth informs the research on Bharti Airtel. An understanding of 5G infrastructure informs the research on tower companies.

Over years, the investor builds their own investment knowledge graph — a network of linked research that reflects how they understand the industries and companies they follow.

This is a moat that no competing platform can replicate. Another platform can import a user's portfolio and documents. It cannot reconstruct years of linked research, evolving theses, cross-company relationships, and knowledge connections.

---

## The Six-Level Feedback Loop

The feedback loop is what distinguishes ChronoSentiment Personal from a static research platform. Every completed investment cycle teaches the system — and the investor — something.

### Level 1 — Thesis feedback

Every Research Dossier is eventually revisited at Level 7 (Completed). The system compares the original thesis with the actual outcome.

> Original thesis: "Revenue growth will accelerate because EV sales are increasing."
>
> Outcome: Revenue +3%, EV sales +22%.
>
> Lesson: Revenue depended more on exports than domestic EV demand. The thesis was directionally correct but the mechanism was wrong.

The research itself becomes better.

### Level 2 — Portfolio feedback

Not just "Did the stock go up?" but "Did this investment improve the overall portfolio?"

- Did it reduce volatility?
- Did it increase concentration?
- Did it improve diversification?
- Did it improve dividend income?

The research learns portfolio effects, not just stock-level outcomes.

### Level 3 — Process feedback

The platform observes how the investor researches.

> "You typically read annual reports but rarely review cash flow statements."
>
> "You tend to initiate positions after news events rather than before."
>
> "80% of your research time is spent on technology companies."

These are observations about research process, not investment advice. They help the investor identify blind spots in their own research methodology.

### Level 4 — Thesis evolution

The platform tracks how theses evolve over time and identifies patterns.

> "Your theses tend to be revised significantly within the first three months. The initial thesis is often too optimistic about the timeline."

> "Your most successful investments had theses that remained largely unchanged for 12+ months."

### Level 5 — Research quality

Over time, the platform learns which research sources actually improve the investor's decisions.

> Research sources for a completed investment:
> - Annual report ★★★★★ (assumptions that proved correct came from here)
> - Management interview ★★★★★ (key insight came from here)
> - News articles ★☆☆☆☆ (no material impact on thesis)
> - Analyst report ★★☆☆☆ (directionally useful but not specific)

After years of use, the platform knows which sources actually improve this investor's decisions — and can prioritise them in future research.

### Level 6 — Investor behaviour

The platform learns the investor's behavioural patterns and surfaces them as observations.

> "Historically, when you closed a position within three months of a 20% gain, those companies subsequently outperformed your remaining portfolio in 68% of cases."

> "In past market corrections, your most successful research identified companies where the thesis remained intact despite price declines. The current situation resembles those historical cases more than the situations where the thesis had broken."

This is feedback, not advice. The system is not telling the investor what to do. It is helping them compare the current situation with their own research history before acting.

---

## The Moat — Personal Investment Knowledge Graph

The moat of ChronoSentiment Personal is not the Research Archive by itself. It is the **Personal Investment Knowledge Graph** — the accumulated network of linked research, evolving theses, source credibility scores, behavioural patterns, and knowledge connections that builds over years of use.

```
Year 1:   Research organised. Theses recorded. Basic patterns visible.
Year 2:   Behavioural patterns identified. Research quality improving.
          Research Graph beginning to form.
Year 3:   Deep investor model. Research highly personalised.
          Research Graph rich with cross-company connections.
          Switching cost: not just the archive — the graph.
```

The moat progression:

```
Research Archive (static)
        ↓
Research Timeline (temporal — how thinking evolved)
        ↓
Personal Learning Loop (behavioural — how the investor improves)
        ↓
Personal Investment Knowledge Graph (structural — how companies connect)
```

A competing platform can import a user's portfolio and documents. It cannot reconstruct years of linked research, evolving theses, cross-company relationships, source credibility, assumption tracking, review history, behavioural patterns, and knowledge connections.

---

## What ChronoSentiment Personal Avoids

The following outputs are deliberately excluded because they move the product toward regulated investment advice:

| Excluded output | Why excluded |
|----------------|-------------|
| "Buy this stock" | Personalised investment recommendation |
| "Sell immediately" | Personalised investment recommendation |
| "Allocate 20% of your portfolio" | Personalised portfolio advice |
| "Target price ₹X" | Personalised price target |
| "Expected return 22%" | Personalised return expectation |
| Automatic trade execution | Automated investment decision |

The product helps investors research better. It does not research for them, and it does not decide for them.

---

## Validation Approach

ChronoSentiment Personal can be validated by the founder using it daily. This is a significant advantage over the Enterprise product, which requires design partners.

**Founder as first user:** Every investment the founder researches becomes a test of the Research Workspace. Every Research Review becomes a test of the review format. Every completed investment becomes a test of the feedback loop.

**Evidence generated:** Each research cycle creates an OBS or DEM record in EL-001. After 30 days of use, the founder has a body of evidence about whether the Research Dossier format, portfolio observations, Research Reviews, and Research Maturity model produce meaningfully better investment research.

**Validation questions:**
- Does the Research Dossier format improve the quality of investment research?
- Does the Research Maturity model help the investor understand how complete their research is?
- Do the quarterly Research Reviews surface useful updates?
- Do the portfolio observations surface information the investor would not have noticed?
- Does the Research Timeline help the investor track how their thinking evolves?
- Does the feedback loop surface useful patterns after 10–20 completed investments?

---

## Relationship to Coralys

ChronoSentiment Personal is implemented as a domain adapter over the Coralys Knowledge Evolution Platform. It does not introduce a separate architecture. The underlying platform is Coralys. ChronoSentiment Personal configures and extends Coralys through an investment research adapter. All research workspaces, evidence management, hypothesis tracking, reviews, timelines, pattern extraction, and learning capabilities are realised through Coralys' Continuous Learning Engine — a first-class platform primitive that manages the complete knowledge lifecycle from hypothesis creation through evidence gathering, review, outcome recording, pattern extraction, and learning. ChronoSentiment Personal supplies the investment-specific semantics; the Continuous Learning Engine supplies the lifecycle.

**Platform hierarchy:**

```
                    Coralys Platform
         (Knowledge Evolution Platform)

           ┌────────────┼────────────┐
           │            │            │
           ▼            ▼            ▼

     UltraCrew    ChronoSentiment   Future Products
                  Enterprise        (Medical, M&A,
                                     Engineering...)
                        ▲
                        │
           ChronoSentiment Personal
           (Investment Research Adapter)
```

ChronoSentiment Personal is not a separate platform. It is a domain adapter that configures Coralys for individual investors.

**Concept mapping — Coralys to ChronoSentiment Personal:**

| Coralys Core | ChronoSentiment Personal |
|-------------|--------------------------|
| Workspace | Research Workspace |
| Subject | Company |
| Context | Portfolio |
| Evidence | Research Sources (annual reports, earnings calls, AI conversations) |
| Hypothesis | Investment Thesis |
| Review | Research Review |
| Timeline | Research Timeline |
| Outcome | Investment Outcome |
| Pattern | Behavioural Pattern |
| Learning | Personal Learning Loop |

Coralys does not know about stocks, portfolios, or investment theses. The adapter supplies the semantics. This separation ensures that the core platform remains domain-neutral while allowing multiple products — ChronoSentiment Personal, ChronoSentiment Enterprise, UltraCrew, and future adapters — to share the same underlying capabilities.

**Strategic implication:** Coralys is the enduring platform asset. ChronoSentiment Personal, ChronoSentiment Enterprise, and future products are domain-specific realisations built on top of it. The platform strategy is not "build ChronoSentiment and then generalise." It is "build Coralys and then specialise." ChronoSentiment is the first rich domain. The same platform can later serve medical research, corporate strategy, M&A, procurement, engineering design reviews, and scientific research without changing the Coralys core.

---

*ChronoSentiment Personal Product Blueprint v1.1 | July 2026*
*Research-first vocabulary throughout. Research Workspace, Research Dossier, Research Timeline, Research Reviews, Research Maturity, Research Graph.*
*Implemented as a domain adapter over the Coralys Knowledge Evolution Platform.*
*Positioning: personal investment research workspace, not recommendation engine.*
*Review trigger: After first 30 days of founder self-use; first external user feedback.*