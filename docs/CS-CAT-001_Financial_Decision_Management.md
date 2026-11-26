# Financial Decision Management
## Defining a New Category in Investment Technology

**Document type:** Category Definition
**Version:** 1.0
**Status:** Draft
**Date:** 2026-07-26
**Owner:** Product / Commercial

---

## Document Governance

| Field | Value |
|-------|-------|
| Document Status | Draft |
| Intended audience | Investors, design partners, prospective customers, industry analysts |
| Review Trigger | Phase 1B completion; material change in category language based on customer evidence |

**Relationship to other documents:**
- Informed by: CS-R-001 through CS-R-015A (research programme)
- Sits alongside: ChronoSentiment Product Strategy v1.0
- Tested by: EL-001 Phase 1B Evidence Ledger (H3 — category language hypothesis)

---

## Introduction

Investment management has a documentation problem.

Not a compliance problem. Not a technology problem. Not a data problem.

A documentation problem.

Every consequential investment decision is the product of months of research, dozens of conversations, multiple analytical frameworks, and the accumulated judgement of experienced professionals. And then — almost immediately — the reasoning behind that decision begins to disappear.

The trade is recorded. The position is tracked. The P&L is measured. But the *why* — the thesis, the evidence, the assumptions, the AI tools that contributed, the committee discussion, the conditions attached to the approval — is not systematically captured anywhere.

This is not a new problem. But it has become significantly more acute in the last three years, for reasons we will examine. And it has become solvable, for the first time, because of a specific combination of technical capabilities that now exist.

The category that addresses this problem is **Financial Decision Management**.

This paper defines that category: what it is, what it is not, why existing systems do not address it, and what the vocabulary of the category looks like.

---

## Part I — Why Existing Systems Do Not Solve This Problem

The most important thing to understand about Financial Decision Management is that it is genuinely new. It is not a rebranding of an existing category. It is not a feature that can be added to an existing system. It addresses a problem that existing systems were not designed to solve.

To understand why, it is necessary to examine what existing systems actually do.

---

### 1. Order Management Systems (OMS) are not decision management systems

An OMS records what was traded: the instrument, the quantity, the price, the time, the counterparty. It is a transaction record.

It does not record why the trade was made. It does not capture the thesis that motivated the position. It does not record the evidence that was available at the time. It does not capture the committee discussion. It does not record the AI tools that contributed to the analysis.

An OMS answers: *What did we do?*

Financial Decision Management answers: *Why did we do it, and what did we know when we decided?*

These are different questions. They require different systems.

---

### 2. Portfolio Management Systems (PMS) are not decision management systems

A PMS tracks portfolio positions, exposures, risk metrics, and performance attribution. It is a portfolio record.

It does not record the investment thesis behind each position. It does not capture the reasoning that led to a position being initiated, sized, or closed. It does not record the committee governance process. It does not capture the AI tools that contributed to the analysis.

A PMS answers: *What do we own, and how is it performing?*

Financial Decision Management answers: *Why do we own it, and what did we believe when we decided to buy it?*

These are different questions. They require different systems.

---

### 3. Research Management Systems (RMS) are not decision management systems

An RMS organises research documents, analyst notes, and investment ideas. It is a research library.

It does not link research to specific investment decisions. It does not capture the committee governance process. It does not record the AI tools that contributed to the analysis. It does not create a point-in-time record of the information environment at the moment of decision.

An RMS answers: *What research do we have?*

Financial Decision Management answers: *What research informed this specific decision, and what was the state of that research at the moment the decision was made?*

These are different questions. They require different systems.

---

### 4. Knowledge Management Systems are not decision management systems

A knowledge management system (Confluence, Notion, SharePoint) organises documents, notes, and institutional knowledge. It is a document repository.

It does not have a structured decision workflow. It does not capture the committee governance process. It does not create point-in-time snapshots of the information environment. It does not link documents to specific investment decisions with timestamps and provenance. It does not generate audit-grade decision records.

A knowledge management system answers: *Where is the document?*

Financial Decision Management answers: *What information was available at the time of this specific decision, who approved it, under what conditions, and what happened?*

These are different questions. They require different systems.

---

### 5. Compliance systems are not decision management systems

A compliance system monitors regulatory requirements, flags violations, and generates compliance reports. It is a risk and control system.

It does not capture the investment thesis. It does not record the committee discussion. It does not create a narrative explanation of why a decision was made. It does not link AI tool usage to specific decisions in a way that is useful for investment review, not just regulatory audit.

A compliance system answers: *Did we follow the rules?*

Financial Decision Management answers: *Why did we make this decision, and can we explain it to anyone who asks — LP, regulator, or new team member — in a way that is accurate, complete, and grounded in the information available at the time?*

These are different questions. They require different systems.

---

### 6. AI tools are not decision management systems

Investment teams now use AI tools — ChatGPT, Claude, Bloomberg AI, AlphaSense — in their research and decision process. These tools generate analysis, surface insights, and contribute to investment theses.

But AI tools do not record their own contributions to decisions. The conversation that led to a key insight is not linked to the decision it informed. The AI's contribution is invisible in the decision record.

An AI tool answers: *What can I help you analyse?*

Financial Decision Management answers: *What AI tools contributed to this decision, what did they contribute, and how can that contribution be explained to a regulator or LP?*

These are different questions. They require different systems.

---

### The gap

The gap between what existing systems record and what investment organisations actually need to document is large and growing.

```
What existing systems record:
  OMS:        What was traded
  PMS:        What is owned and how it performs
  RMS:        What research exists
  KMS:        Where documents are stored
  Compliance: Whether rules were followed
  AI tools:   What analysis was generated

What investment organisations need to document:
  Why the decision was made
  What information was available at the time
  What AI tools contributed and how
  What the committee discussed and decided
  What conditions were attached to the approval
  What the outcome was and what was learned
  How to explain all of this to an LP or regulator
```

Financial Decision Management fills this gap.

---

## Part II — The Category Vocabulary

Financial Decision Management has its own vocabulary. Defining these terms precisely is important because the category is new and the language is not yet standardised. The following definitions reflect the product philosophy of ChronoSentiment and are offered as a contribution to the emerging category vocabulary.

---

### Decision Workspace

The persistent, collaborative environment where an investment decision lives throughout its entire lifecycle — from thesis formation through execution, review, and post-mortem analysis.

The Decision Workspace is not a document. It is not a form. It is not a report. It is a structured environment that accumulates evidence, records governance, links to execution, and produces a Decision Record.

The Decision Workspace is to investment decisions what a GitHub repository is to software: a persistent, versioned, collaborative home for everything related to a specific piece of work.

---

### Decision Record

The persistent, exportable artefact that captures the complete lifecycle of an investment decision. The Decision Record is produced by the Decision Workspace and can be exported at any point as a timestamped, audit-grade document.

The Decision Record is not a report generated after the fact. It is a live document that accumulates evidence throughout the decision lifecycle. It is the document that gets shared with LPs, presented to regulators, and used for post-mortem analysis.

A Decision Record contains: the thesis, the evidence available at the time of the decision, the AI tools that contributed, the committee discussion and approval, the execution record, the outcome, and the lessons captured.

---

### Decision Memory

The accumulated record of an organisation's investment decisions over time. Decision Memory is the institutional asset that ChronoSentiment builds as the product is used.

Decision Memory is not a database of trades. It is a searchable, queryable record of why decisions were made — the reasoning, the evidence, the assumptions, the outcomes, and the lessons. It is the institutional knowledge that survives staff turnover, resists hindsight bias, and enables genuine learning from experience.

Decision Memory is the primary long-term value proposition of Financial Decision Management. The longer an organisation uses the system, the more valuable its Decision Memory becomes.

---

### Decision Provenance

The complete record of where a decision came from — the research that informed it, the AI tools that contributed, the people who were involved, the information that was available at the time, and the causal chain that led from evidence to conclusion.

Decision Provenance is what makes a decision explainable. It is the answer to the question: "How did you arrive at this conclusion?" It is not a narrative constructed after the fact — it is a record built at the time of the decision, capturing the actual information environment.

Decision Provenance is what regulators are beginning to require for AI-assisted decisions. It is what LPs are beginning to ask for in due diligence. It is what investment committees need to evaluate the quality of a decision, not just its outcome.

---

### Decision Reconstruction

The ability to recreate the exact information environment that existed at the moment a decision was made — the market data, the research, the AI conversations, the portfolio state — as it was at that specific point in time, not as it appears in retrospect.

Decision Reconstruction is the hardest technical problem in Financial Decision Management. It requires point-in-time data capture, deterministic replay, and a system architecture designed from the ground up to preserve temporal state.

Decision Reconstruction is what makes it possible to review a decision fairly — to evaluate it against the information that was available at the time, not against information that became available later. It is what prevents hindsight bias from corrupting the post-mortem process.

---

### Decision Intelligence

The capability to surface patterns across the Decision Memory — recurring assumption failures, sectors where conviction is systematically miscalibrated, AI tools that contributed to decisions that underperformed, committee dynamics that correlate with better or worse outcomes.

Decision Intelligence is the Phase 2 value proposition of Financial Decision Management. It transforms the Decision Memory from a passive archive into an active learning system. It is what makes the product more valuable over time, not just more complete.

---

### Decision Archive

The complete, searchable collection of Decision Records accumulated by an organisation over time. The Decision Archive is the physical manifestation of Decision Memory.

The Decision Archive is a strategic asset. It is the record of how the organisation thinks — its investment philosophy expressed in actual decisions, not just stated principles. It is the training data for future AI systems that will learn from the organisation's own decision history. It is the onboarding resource for new team members who need to understand how the organisation makes decisions.

The Decision Archive is also the primary source of switching costs. An organisation that has built a three-year Decision Archive cannot easily move to a different system — the archive is not portable in any meaningful sense.

---

### Decision Governance

The structured process by which investment decisions are reviewed, approved, and recorded — the committee workflow, the approval conditions, the dissenting views, the escalation paths.

Decision Governance is not compliance. Compliance is about following rules. Decision Governance is about making good decisions and being able to explain them. The two overlap — good governance produces compliance-ready documentation — but they are not the same thing.

Decision Governance is what investment committees do. Financial Decision Management is the system that makes Decision Governance systematic, consistent, and auditable.

---

### Decision Evidence

The structured record of the information that was available at the time of a decision — the research documents, the data snapshots, the AI conversations, the market data — captured at the moment of decision creation, not reconstructed later.

Decision Evidence is what distinguishes a Decision Record from a narrative. A narrative can be constructed after the fact, filtered through hindsight, and shaped by the outcome. Decision Evidence is captured at the time, preserved in its original form, and linked to the decision with a timestamp.

Decision Evidence is what makes a Decision Record audit-grade. It is what a regulator can examine to verify that the decision was made on the basis of the information that was available at the time.

---

### Decision Lifecycle

The complete arc of an investment decision from thesis formation through execution, review, and post-mortem analysis. The Decision Lifecycle has five stages:

```
1. Formation    Thesis created; evidence gathered; AI tools used
2. Governance   Committee review; approval or rejection; conditions recorded
3. Execution    Trade initiated; execution linked to decision
4. Review       Outcome assessed; thesis vs reality compared
5. Learning     Lessons captured; patterns identified; archive updated
```

Financial Decision Management supports the entire Decision Lifecycle. Existing systems support individual stages — OMS supports Execution, RMS supports Formation, compliance systems support Governance — but no existing system supports the complete lifecycle as a unified record.

---

## Part III — Why This Category Is Emerging Now

Financial Decision Management is not a new idea. The problem it addresses has existed for as long as investment management has existed. What is new is the combination of forces that makes the category both necessary and possible.

### Necessity

**AI adoption has created a documentation gap.** Investment teams are now using AI tools in their research and decision process at scale. These tools generate analysis that influences decisions, but their contributions are not recorded. The gap between what actually happened and what can be documented has widened significantly.

**Regulatory requirements are creating explicit demand.** The EU AI Act, FCA guidance on AI in financial services, and SEC commentary on AI in investment management are creating explicit requirements to document AI usage in consequential decisions. The regulatory pressure is real, immediate, and growing.

**LP expectations have risen.** Institutional LPs are increasingly asking for decision-level transparency. The question "why did you make this decision?" is becoming a standard part of LP due diligence and ongoing reporting.

**Staff turnover has increased.** The post-2020 labour market in financial services has seen elevated turnover at the analyst and portfolio manager level. The knowledge loss problem has become more frequent and more visible.

### Possibility

**Point-in-time reconstruction is now technically feasible.** The ability to reconstruct the exact information environment at any historical point in time — what data was available, what the portfolio state was, what the market was doing — is now achievable with modern data infrastructure and deterministic replay systems.

**Large language models make explainability tractable.** Generating natural-language explanations of complex analytical processes is now a solved problem. The narrative block system in ChronoSentiment's existing platform demonstrates this at the execution layer.

**Decision archives are becoming valuable.** As AI systems become more capable of learning from historical decisions, the decision archive itself becomes a strategic asset. Firms that have documented their decision history will be able to train AI systems on their own institutional knowledge.

**The category is being created now.** No vendor currently makes investment decision governance its primary product. The category is being defined. The firm that defines it will have a significant first-mover advantage.

---

## Part IV — The Category Positioning

Financial Decision Management sits at the intersection of four existing categories, drawing from each without belonging to any of them.

```
Research Management          Portfolio Management
(what we know)               (what we own)
         \                   /
          \                 /
           Financial Decision
              Management
           (why we decided)
          /                 \
         /                   \
Compliance & Governance      AI Governance
(did we follow the rules)    (what AI contributed)
```

Financial Decision Management is not a replacement for any of these categories. It is a new layer that connects them — the system of record for the reasoning that links research to portfolio decisions, and the governance layer that makes that reasoning auditable.

---

## Conclusion

Financial Decision Management is a new category because it addresses a problem that existing systems were not designed to solve: the systematic capture, preservation, and explanation of investment decision reasoning.

The problem is not new. The urgency is new. The technical capability to solve it is new. The regulatory pressure to address it is new.

The category is being created now. The vocabulary is being established. The first products are being built.

ChronoSentiment is building the platform that defines this category — starting with the Decision Workspace, the Decision Record, and the Decision Archive, and expanding toward Decision Intelligence as the archive grows.

The central claim of Financial Decision Management is simple:

> Investment organisations should be able to explain every consequential decision they make — not as a narrative constructed after the fact, but as a record built at the time, grounded in the information that was available, and preserved in a form that survives staff turnover, resists hindsight bias, and satisfies the expectations of LPs, regulators, and future team members.

That is what Financial Decision Management delivers.

---

*CS-CAT-001 Financial Decision Management v1.0 | July 2026 | ChronoSentiment*
*Category definition paper. Intended for investors, design partners, prospective customers, and industry analysts.*
*Review trigger: Phase 1B completion; material change in category language based on customer evidence (H3).*