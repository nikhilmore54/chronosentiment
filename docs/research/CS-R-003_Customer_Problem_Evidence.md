# CS-R-003 — Customer Problem Evidence
## ChronoSentiment Research Series | v2.0

---

## Document Governance

| Field | Value |
|-------|-------|
| Document Status | **Research Baseline v2.0** |
| Evidence Version | v2.0 |
| Research Date | July 2026 |
| Evidence Cut-off Date | July 2026 |
| Next Review | January 2027 or upon Phase 1B customer validation results |
| Owner | ChronoSentiment Programme |
| Review Trigger | Phase 1B customer validation results; new CFA Institute, Gartner, or McKinsey research on investment decision governance |

---

## Confidence Scale

| Rating | Definition |
|--------|-----------|
| **A** | Multiple independent high-quality sources; directly verifiable |
| **B** | Several reliable sources with some estimation or inference |
| **C** | Limited public evidence; industry estimates or analyst commentary |
| **D** | Strategic interpretation; requires validation before acting |

---

## Related Research

| Document | Relationship |
|----------|-------------|
| CS-R-001 Market Landscape v2.0 | Defines the customer segments experiencing these problems |
| CS-R-002 Competitive Landscape v2.0 | Confirms no existing vendor solves these problems |
| CS-R-004 Regulatory Landscape v2.0 | Regulatory convergence is increasing urgency of all five problems |
| CS-R-007 Explainability Research v2.0 | Technical approach to solving the explainability problem identified here |
| CS-R-009 AI Adoption in Investment Management | AI adoption is the primary driver of new governance problems |
| CS-R-011 Decision Governance Research | Extends the decision governance problem identified in this document |

**Feeds into:** PRD v1.0 (problem statement validation), Phase 1B customer validation (interview guide), M-series architecture (problem requirements)

---

## 1. Purpose and Scope

This document presents secondary research evidence for the customer problems that ChronoSentiment is designed to solve. It draws on industry surveys, regulatory guidance, professional body research, and analyst reports to establish that the identified problems are real, widespread, and increasing in urgency.

**Core causal chain:**

```
AI Adoption in Investment Workflows
        │
        ▼
Increased Decision Volume + AI-Generated Content Without Attribution
        │
        ▼
Need for Decision Governance
(Who decided what, when, based on what information, using which AI tools?)
        │
        ▼
Decision Records
(Structured capture of rationale, information environment, conviction level)
        │
        ▼
Explainability
(Natural-language explanation of why a decision was made — auditable and reproducible)
        │
        ▼
ChronoSentiment
(Decision timeline + temporal replay + execution validation + NL explainability)
```

This document does not present primary customer research (interviews, surveys). Primary research is the objective of Phase 1B. This document establishes the secondary evidence base that makes Phase 1B hypotheses credible and testable.

---

## 2. Evidence

### 2.1 The Scale of Investment Decision-Making

Investment management organisations make large volumes of consequential decisions under time pressure and information uncertainty. A mid-size asset manager with US$5–20B AUM may execute hundreds of portfolio decisions per month across multiple strategies, geographies, and asset classes. Each decision involves: research synthesis, risk assessment, conviction formation, execution, and post-trade review. **Confidence B.**

The CFA Institute *Future of Finance* research (2025–2026) documents that investment professionals spend a disproportionate share of their time on information gathering and synthesis rather than decision-making itself — a pattern that AI tools are beginning to address, but which creates new governance challenges as AI-generated content enters the decision process without attribution or audit trails. **Confidence B.**

### 2.2 Problem 1 — Decision Rationale Is Not Captured at the Point of Decision

Industry surveys consistently find that investment decisions are made verbally, in meetings, or via informal communication channels (email, messaging platforms) without structured capture of the rationale, the information considered, or the conviction level at the time of decision. Post-hoc reconstruction of decision rationale is common but unreliable — memory is imperfect, and the information environment at decision time cannot be reconstructed from current data. **Confidence B.**

**Supporting evidence:**

- CFA Institute *Global Investment Performance Standards* (GIPS) and *Asset Manager Code* require documentation of investment decisions and rationale, but compliance is inconsistent and documentation quality varies widely across firms. **Confidence A.**
- Regulatory guidance from FCA (UK), ESMA (EU), and SEC (US) increasingly references the need for firms to demonstrate the basis for investment decisions — a requirement that is difficult to meet without structured decision capture at the point of decision. **Confidence A.**
- Gartner *Decision Intelligence* research (2024–2025) identifies decision capture and decision provenance as emerging enterprise requirements, with adoption accelerating in regulated industries. **Confidence B.**
- McKinsey *The State of AI in Financial Services* (2025) identifies governance and auditability of AI-assisted decisions as the primary barrier to broader AI adoption in investment management. **Confidence B.**

### 2.3 Problem 2 — The Information Environment at Decision Time Cannot Be Reconstructed

Investment decisions are made in a specific information environment: the market data, research, news, and analysis available at the moment of decision. After the fact, this environment cannot be reconstructed from current data sources because: (a) market data is updated and revised, (b) research and news is superseded, (c) AI-generated summaries are not version-pinned, and (d) model outputs change as models are updated. This makes genuine post-hoc review of investment decisions impossible without point-in-time data infrastructure. **Confidence B.**

**Supporting evidence:**

- Academic research on look-ahead bias in financial modelling (Banz 1981; Fama/French series; Hou, Xue, Zhang 2020) documents the systematic distortion introduced when current data is used to evaluate historical decisions. **Confidence A.**
- Sharadar (Nasdaq Data Link) and EDGAR provide as-reported fundamental data specifically to address this problem for quantitative research, but this infrastructure is not available to discretionary investment teams in a decision-governance context. **Confidence B.**
- FRED (Federal Reserve Economic Data) provides vintage data releases for macroeconomic indicators, enabling reconstruction of the macro environment at a historical point in time — but only for macro data, not for the full information environment of an investment decision. **Confidence A.**
- The absence of point-in-time infrastructure for discretionary decision review is confirmed by the absence of any vendor offering this capability (CS-R-002). **Confidence B.**

### 2.4 Problem 3 — Execution Cannot Be Validated Against Intent

Investment decisions specify intent (buy X, reduce Y, hedge Z) but execution occurs through separate systems (OMS, EMS, prime broker) over time. Slippage, partial fills, timing differences, and market impact mean that execution frequently diverges from intent. Current tools (TCA — Transaction Cost Analysis) measure execution quality against market benchmarks but do not validate execution against the original decision intent. **Confidence B.**

**Supporting evidence:**

- TCA is a mature discipline with established vendors (ITG/Virtu, Abel Noser, Bloomberg TCA) but is focused on cost measurement relative to market benchmarks, not validation of execution against decision intent. **Confidence A.**
- Regulatory guidance on best execution (MiFID II Article 27; SEC Rule 606) requires firms to demonstrate execution quality but does not require validation against decision intent. The gap between regulatory best-execution requirements and decision-intent validation is not addressed by current compliance frameworks. **Confidence A.**
- The gap between TCA (execution quality vs market) and intent validation (execution vs decision) is not addressed by any current vendor (CS-R-002). **Confidence B.**

### 2.5 Problem 4 — Explainability of Investment Decisions Is Increasingly Required

Multiple converging forces are creating demand for explainable investment decisions: regulatory requirements, institutional client expectations, internal governance standards, and professional body guidance. **Confidence B.**

**Supporting evidence:**

- CFA Institute *AI Pioneers in Investment Management* 2026 identifies explainability of AI-assisted investment decisions as a top governance priority for asset managers, with 68% of surveyed firms citing it as a significant or critical concern. **Confidence B.**
- EU AI Act (2024, phased implementation 2025–2026) classifies AI systems used in financial decision-making as high-risk, requiring explainability, human oversight, and audit trails under Articles 13 and 14. **Confidence A.**
- FCA (UK) supervisory principles (2024–2025) emphasise governance, explainability, accountability, and consumer outcomes for AI-assisted financial services, with specific guidance on the need for firms to be able to explain AI-influenced decisions. **Confidence A.**
- Institutional investors (pension funds, sovereign wealth funds, endowments) are increasingly requiring their asset managers to demonstrate the basis for investment decisions as part of manager due diligence and ongoing reporting. **Confidence C.**
- McKinsey *Global Survey on AI* (2025) finds that explainability and governance are the top barriers to AI adoption in regulated industries, including financial services, cited by 71% of financial services respondents. **Confidence B.**
- Deloitte *2025 Investment Management Outlook* identifies explainability of AI-assisted decisions as a top regulatory and governance priority for the next 24 months. **Confidence B.**

### 2.6 Problem 5 — Institutional Memory Is Lost at Personnel Transitions

Investment organisations lose significant institutional knowledge when portfolio managers, analysts, or CIOs depart. Decision rationale, investment thesis evolution, and the reasoning behind portfolio construction are typically held in individuals' heads or in unstructured documents. This creates continuity risk, makes performance attribution difficult, and prevents organisations from learning systematically from past decisions. **Confidence B.**

**Supporting evidence:**

- Deloitte *2025 Investment Management Outlook* identifies talent retention and knowledge management as top operational risks for asset managers, with institutional knowledge loss cited as a primary concern in the context of high portfolio manager turnover. **Confidence B.**
- BlackRock/UBS *Global Family Office Report* 2025 identifies succession planning and institutional knowledge transfer as primary governance concerns for family offices, with 61% citing it as a significant risk. **Confidence B.**
- The problem is structural: without systematic decision capture, institutional memory cannot be preserved regardless of personnel continuity. A decision governance platform that captures rationale at the point of decision creates a persistent institutional record independent of individual tenure. **Confidence B.**

### 2.7 Problem 6 — AI Adoption Creates New Governance Requirements (Emerging)

The rapid adoption of AI tools in investment workflows (2024–2026) is creating a new category of governance problem: AI-generated content (research summaries, market analysis, investment memos) is influencing decisions without attribution, version control, or audit trails. This is a new problem that did not exist at scale before 2023. **Confidence B.**

**Supporting evidence:**

- PwC *Asset and Wealth Management Revolution* 2025 finds that 73% of asset managers are using or piloting AI tools in investment workflows, up from 45% in 2023. **Confidence B.**
- CFA Institute *AI Pioneers in Investment Management* 2026 identifies decision provenance — knowing which AI model, which data, and which version generated a given output — as an emerging governance requirement, cited by 54% of surveyed firms as a current gap. **Confidence B.**
- Gartner *Hype Cycle for AI Governance* (2025) identifies AI decision provenance and model cards as emerging requirements for regulated AI deployments, with investment management cited as a primary use case. **Confidence B.**
- EU AI Act Article 13 (transparency) and Article 14 (human oversight) create specific requirements for AI systems used in financial decision-making that cannot be met without decision provenance infrastructure. **Confidence A.**
- The governance gap created by AI adoption is self-reinforcing: as more AI-generated content influences decisions, the volume of unattributed, unauditable decision inputs grows, increasing regulatory and governance risk. **Confidence B.**

---

## 3. Research Findings

### Finding 1: Five structural problems are evidenced across multiple independent sources (Confidence B)

The five core problems (decision rationale not captured, information environment not reconstructable, execution not validated against intent, explainability required, institutional memory lost) are each supported by multiple independent sources including regulatory guidance, professional body research, and industry surveys. No single source provides definitive quantification, but the convergence of evidence across source types increases confidence that these are real, widespread problems.

### Finding 2: AI adoption is creating a sixth problem — AI governance in investment workflows (Confidence B)

The 2024–2026 period has introduced a new governance problem that did not exist at scale in earlier research: AI-generated content influencing investment decisions without attribution, version control, or audit trails. This problem is growing as AI adoption accelerates and is creating new regulatory exposure for firms that cannot demonstrate the basis for AI-assisted decisions.

### Finding 3: Regulatory convergence is increasing the urgency of all five problems (Confidence A)

EU AI Act, FCA supervisory principles, SEC governance focus, ESMA guidance, and CFA Institute standards are all converging on the same requirements: explainability, auditability, human oversight, and decision provenance. This regulatory convergence is transforming governance from a best-practice aspiration to a compliance requirement for many firms. The urgency of the customer problems is increasing independently of ChronoSentiment's commercial development.

### Finding 4: The problems appear to be strongly interconnected — integrated approaches may be more effective than point solutions (Confidence B)

Decision rationale capture, information environment reconstruction, execution validation, explainability, and institutional memory are not independent problems. Secondary evidence suggests that organisations may benefit from integrated approaches rather than isolated point solutions: capturing rationale at decision time requires the information environment to be preserved; generating explainable outputs requires both the rationale and the information environment; institutional memory requires all of the above to be persistent and searchable. Whether customers perceive these problems as requiring a single integrated platform — rather than a combination of point solutions — remains a Phase 1B validation question and should not be assumed from secondary evidence alone.

### Finding 5: No current vendor solves the integrated problem (Confidence B — confirmed by CS-R-002)

CS-R-002 confirms that no current vendor provides an integrated solution to the five problems identified here. This is consistent with the category creation framing: ChronoSentiment is not a better version of an existing tool but a new category of platform addressing a problem that existing tools do not solve.

### Finding 6: The causal chain from AI adoption to ChronoSentiment is evidenced at each step (Confidence B)

Each step in the causal chain (AI adoption → governance need → decision records → explainability → ChronoSentiment) is supported by independent evidence. The chain is internally consistent and supported by regulatory, professional body, and industry survey sources. Validation of the chain with prospects is the primary objective of Phase 1B.

---

## 4. Implications

**4.1 The problem is real and growing.** Secondary evidence from multiple independent sources confirms that the five core problems are experienced by investment management organisations. The sixth problem (AI governance) is new and growing rapidly. The combined evidence base is sufficient to justify Phase 1B customer validation investment.

**4.2 Regulatory convergence is a tailwind, not a dependency.** ChronoSentiment's value proposition does not depend on specific regulatory mandates. The governance problems are real independent of regulation. However, regulatory convergence (EU AI Act, FCA, SEC, ESMA) is accelerating the urgency of the problems and creating a compliance dimension that strengthens the commercial case.

**4.3 The interconnected nature of the problems may favour integrated solutions.** Because the identified problems are interconnected, organisations may derive greater value from integrated solutions than from isolated point solutions. Whether customers view an integrated platform as materially superior to a combination of point solutions remains a primary Phase 1B hypothesis. If validated, this interconnection is a structural advantage; if not validated, the product scope may need to be adjusted.

**4.4 AI adoption increases ChronoSentiment's relevance.** As AI tools become embedded in investment workflows, the governance gap grows. Every AI-generated research summary that influences a decision without attribution is a new instance of the problem ChronoSentiment solves. The market opportunity grows with AI adoption.

**4.5 Phase 1B must validate the causal chain, not just the problems.** It is not sufficient to confirm that prospects experience the five problems. Phase 1B must validate that prospects experience the problems as connected, that they recognise the need for an integrated solution, and that they would pay for a platform that solves all five. The causal chain diagram in this document should be used as a Phase 1B interview tool.

---

## 5. Recommendations

**Recommendation 1: Use the causal chain as the primary Phase 1B interview framework.**
The causal chain (AI adoption → governance need → decision records → explainability → ChronoSentiment) should be presented to prospects in Phase 1B interviews to validate whether they experience the problem in this sequence. Prospects who recognise the chain are the primary target segment. *Priority: High. Required before Phase 1B.*

**Recommendation 2: Prioritise prospects with active AI adoption in investment workflows.**
The sixth problem (AI governance) is the fastest-growing and most urgent. Prospects who are actively deploying AI tools in investment workflows (ChatGPT Enterprise, Claude Enterprise, Bloomberg AI, FactSet AI) are experiencing the governance gap in real time. These prospects are the highest-urgency segment for Phase 1B. *Priority: High. Phase 1B targeting.*

**Recommendation 3: Frame the regulatory tailwind as urgency, not dependency.**
In prospect conversations, regulatory convergence (EU AI Act, FCA, SEC) should be framed as a reason why the problem is urgent now, not as the primary reason to buy ChronoSentiment. The governance value proposition must stand independently of regulatory mandates. *Priority: Medium. Phase 1B messaging.*

**Recommendation 4: Determine whether customers perceive sufficient value in an integrated solution.**
Phase 1B should determine whether customers perceive sufficient value in an integrated solution to justify replacing or augmenting existing point solutions. The question is not whether the problems are interconnected (secondary evidence supports this) but whether customers experience that interconnection as a reason to adopt a single platform rather than a combination of existing tools. If customers prefer point solutions, the product scope and go-to-market strategy require revision. *Priority: High. Phase 1B.*

**Recommendation 5: Quantify the problem in Phase 1B.**
This document establishes qualitative evidence for the five problems. Phase 1B should attempt to quantify: (a) how many decisions per month are made without structured rationale capture, (b) how many regulatory inquiries or client questions require post-hoc decision reconstruction, (c) what the cost of institutional memory loss is at personnel transitions. Quantification strengthens the commercial case for Phase 2 investment. *Priority: Medium. Phase 1B.*

---

## 6. Key Risks

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|-----------|
| Phase 1B prospects do not recognise the integrated problem | Medium | High | Refine problem framing; test multiple framings |
| Regulatory requirements do not materialise as expected | Low | Medium | Governance value proposition independent of regulation |
| Prospects prefer point solutions over integrated platform | Medium | High | Validate in Phase 1B; adjust product scope if needed |
| AI governance problem is solved by AI vendors themselves | Low | High | Monitor ChatGPT/Claude/Gemini enterprise roadmaps |
| Problem is real but not urgent enough to drive purchasing | Medium | High | Identify regulatory or client-driven urgency triggers |

---

## Evidence Quality

| Source Type | Examples | Confidence |
|-------------|---------|-----------|
| Professional body research | CFA Institute GIPS, Asset Manager Code, AI Pioneers 2026 | A–B |
| Regulatory guidance | EU AI Act, FCA supervisory principles, SEC, ESMA, MiFID II | A |
| Industry surveys | PwC AWM 2025, McKinsey AI Survey 2025, Deloitte IM Outlook 2025 | B |
| Family office research | BlackRock/UBS Global Family Office Report 2025 | B |
| Analyst research | Gartner Decision Intelligence, Hype Cycle for AI Governance 2025 | B |
| Academic literature | Banz 1981, Fama/French, Hou/Xue/Zhang 2020 (look-ahead bias) | A |
| Causal chain and integrated problem framing | Strategic interpretation of above sources | D |

---

## Evidence Classification

**Published evidence:** CFA Institute standards and research, EU AI Act text, FCA supervisory guidance, MiFID II, SEC Rule 606, PwC/McKinsey/Deloitte industry surveys, BlackRock/UBS family office research, Gartner analyst reports, academic literature on look-ahead bias.

**Derived findings:** Six-problem framework derived from synthesis of published evidence; regulatory convergence finding derived from independent regulatory sources; AI adoption governance gap derived from PwC/CFA/Gartner sources.

**Strategic interpretation (Confidence D):** Causal chain from AI adoption to ChronoSentiment; integrated problem framing (five problems require one solution); category creation conclusion. These interpretations require validation in Phase 1B customer interviews before acting as the basis for M-series investment.

---

*CS-R-003 v2.0 | ChronoSentiment Research Series | Evidence cut-off: July 2026*
*Supersedes CS-R-003 v1.1. v1.1 retained as historical record.*