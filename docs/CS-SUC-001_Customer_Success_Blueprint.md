# CS-SUC-001 — Customer Success Blueprint
## ChronoSentiment | v1.0

---

## Document Governance

| Field | Value |
|-------|-------|
| Document Status | Draft v1.0 |
| Date | 2026-07-26 |
| Owner | Product / Commercial |
| Review Trigger | After first design partner completes Month 3; after first full-year customer |

**Relationship to other documents:**
- Informed by: Product Blueprint v1.0 (product definition), CS-CAT-001 (category vocabulary)
- Feeds into: COM-001 (design partner conversations), CV-001 (design partner process)
- Tested by: EL-001 (DEM and POC evidence records)

---

## Purpose

This document describes how customers adopt ChronoSentiment — not what the product does, but what changes inside an investment organisation as the product becomes embedded in the workflow.

This matters for two reasons.

First, ChronoSentiment's primary moat is the Decision Archive. The archive only becomes valuable if customers use the product consistently over time. The adoption journey is therefore part of the product — it determines whether the moat is built.

Second, the adoption journey is the design partner conversation. When a prospective design partner asks "what would this look like for us?", this document provides the answer.

The adoption journey has five stages: Activation, Habit Formation, Governance Integration, Archive Value, and Decision Intelligence. Each stage has a distinct value proposition, a distinct set of habits that need to form, and a distinct set of success indicators.

---

## The Adoption Journey Overview

```
Week 1–2        Activation
                First Decision Workspace created
                First Decision Record generated
                Team sees the product work

Month 1         Habit Formation
                Decision creation becomes routine
                Evidence capture becomes automatic
                Team stops asking "should I use this?"

Month 3         Governance Integration
                Committee workflow runs through ChronoSentiment
                Approvals are recorded in the system
                LP reporting uses Decision Records

Month 6         Archive Value
                Archive contains 50–100 decisions
                Post-mortem analysis becomes possible
                New team members onboard using the archive

Year 1          Decision Intelligence
                Patterns emerge across the archive
                Assumption failures become visible
                The archive is a strategic asset
```

---

## Stage 1 — Activation (Week 1–2)

### What happens

The first two weeks are about demonstrating that the product works with real decisions from the firm. The goal is not to change behaviour — it is to show the team what the product produces.

The activation sequence:

1. **Day 1–2:** Import 3–5 historical decisions. These are decisions the team already made, reconstructed in ChronoSentiment using existing research, notes, and market data. The team sees their own decisions in the Decision Workspace for the first time.

2. **Day 3–5:** Create one new Decision Workspace for a live decision currently in progress. The portfolio manager enters the thesis, attaches the research, and records the AI tools used. The committee reviews it in the system.

3. **Day 7–10:** Generate the first Decision Record. Export it as a PDF. Show the team what the LP-ready document looks like.

4. **Day 10–14:** Run the first post-mortem on one of the historical decisions. Show the divergence analysis — what the thesis predicted versus what actually happened.

### What the team experiences

The activation stage is designed to produce one specific reaction: "This is what we've been missing."

The most common activation moment is the LP reporting demonstration. When a portfolio manager sees a Decision Record generated in two minutes — a document that would normally take two hours to write — the value proposition becomes concrete.

The second most common activation moment is the post-mortem. When the team sees a historical decision reconstructed with the information that was available at the time — not filtered through hindsight — the intellectual honesty of the tool becomes apparent.

### Success indicators

- At least one Decision Workspace created for a live decision
- At least one Decision Record exported and reviewed by the team
- At least one team member describes the product as "useful" unprompted
- No significant friction in the onboarding process

### What can go wrong

**The team treats it as a compliance tool.** If the first use case is regulatory documentation rather than investment workflow, the product feels like overhead rather than value. Start with LP reporting or post-mortem analysis, not compliance.

**The historical import is too difficult.** If reconstructing historical decisions requires too much manual effort, the activation stage stalls. The import process must be fast — if it takes more than 30 minutes per decision, it will not happen.

**The portfolio manager is not involved.** If the product is introduced by the technology or compliance team without the portfolio manager's involvement, it will not be adopted. The portfolio manager must be the primary user from day one.

---

## Stage 2 — Habit Formation (Month 1)

### What happens

Month 1 is about making Decision Workspace creation a routine part of the investment process. The goal is to reach the point where the team creates a Decision Workspace for every new investment decision without being asked.

The habit formation sequence:

1. **Week 3:** The portfolio manager creates a Decision Workspace for every new investment idea that reaches the "worth investigating" stage. This is the first habit — creating the workspace early, before the thesis is fully formed.

2. **Week 4:** The analyst begins attaching evidence to the Decision Workspace as research is completed — not at the end of the process, but as each piece of research is done. This is the second habit — continuous evidence capture.

3. **Week 5–6:** The AI conversation export becomes automatic. Every time the portfolio manager uses ChatGPT, Claude, or AlphaSense for investment research, the relevant conversation is exported and attached to the Decision Workspace. This is the third habit — AI provenance capture.

4. **Week 6–8:** The team stops asking "should I use ChronoSentiment for this?" and starts asking "where is the Decision Workspace for this?"

### What the team experiences

The habit formation stage is the hardest part of the adoption journey. It requires changing behaviour, not just demonstrating value. The product must be easier to use than the alternative (email, shared drives, memory) for the habit to form.

The key insight is that the habit forms around the Decision Workspace, not around the Decision Record. The Decision Record is the output — it is what the team shows to LPs and regulators. But the habit is about creating and maintaining the workspace throughout the decision lifecycle.

The most common friction point in Month 1 is evidence capture. Portfolio managers are not accustomed to attaching research to a specific decision as they work. The product must make this as frictionless as possible — ideally, a single click to attach a document or export an AI conversation.

### Success indicators

- Decision Workspace created for every new investment decision (not just selected ones)
- Evidence attached to workspaces within 24 hours of being produced (not at the end of the process)
- AI conversation exports happening routinely (not just when reminded)
- The team uses ChronoSentiment vocabulary ("Decision Workspace", "Decision Record") in internal conversations

### What can go wrong

**The product is used selectively.** If the team only creates Decision Workspaces for "important" decisions, the archive will be incomplete and the habit will not form. Every investment decision needs a workspace — the value of the archive depends on completeness.

**Evidence capture is too manual.** If attaching evidence requires more than two clicks, it will not happen consistently. The product must make evidence capture automatic or near-automatic.

**The portfolio manager delegates to the analyst.** If the portfolio manager treats ChronoSentiment as an administrative task and delegates it to the analyst, the thesis capture will be incomplete. The portfolio manager must own the Decision Workspace.

---

## Stage 3 — Governance Integration (Month 3)

### What happens

By Month 3, the team has established the habit of creating Decision Workspaces. The next stage is integrating the committee governance process into ChronoSentiment — making the investment committee review happen in the system, not alongside it.

The governance integration sequence:

1. **Month 2, Week 1:** The first committee review happens in ChronoSentiment. The portfolio manager presents the Decision Workspace to the committee. The CIO reviews the thesis, evidence, and AI provenance in the system. The discussion is recorded.

2. **Month 2, Week 2–4:** The committee approval process moves into ChronoSentiment. Approvals are recorded with conditions. Dissenting views are captured. The approval timestamp is immutable.

3. **Month 3:** The first LP query is answered using a Decision Record. The portfolio manager generates the record in two minutes. The LP receives a document that is accurate, complete, and grounded in the information available at the time.

4. **Month 3:** The first regulatory documentation request is fulfilled using Decision Records. The compliance team sees the value of the system for the first time.

### What the team experiences

The governance integration stage is where ChronoSentiment becomes embedded in the firm's operating model. Once the committee review process runs through the system, the product is no longer optional — it is part of how the firm makes decisions.

The most significant moment in this stage is the first LP query answered using a Decision Record. This is typically the moment when the CIO becomes a champion of the product — not because of the technology, but because of the time saved and the quality of the response.

The second significant moment is the first regulatory documentation request. When the compliance team sees that the Decision Records already contain everything the regulator needs — AI tool usage, committee approval, conditions, timestamps — the compliance value proposition becomes concrete.

### Success indicators

- Committee review process runs through ChronoSentiment for all new decisions
- At least one LP query answered using a Decision Record
- At least one regulatory documentation request fulfilled using Decision Records
- CIO actively uses the system (not just the portfolio manager and analyst)
- Compliance team has reviewed at least one Decision Record

### What can go wrong

**The committee resists the change.** Investment committees have established processes. Introducing a new system into the committee workflow requires buy-in from the CIO. If the CIO is not a champion, the governance integration will stall.

**The LP query is handled the old way.** If the first LP query is answered using the traditional process (email, manual document assembly) rather than ChronoSentiment, the team will not experience the time-saving value proposition. The first LP query must be handled in the system.

**The compliance team is not involved.** If the compliance team is not introduced to ChronoSentiment by Month 3, they will not understand its value for regulatory documentation. The compliance team should be shown the system by Month 2.

---

## Stage 4 — Archive Value (Month 6)

### What happens

By Month 6, the Decision Archive contains 50–100 decisions. This is the point at which the archive begins to generate value beyond individual decision documentation.

The archive value sequence:

1. **Month 4–5:** The archive reaches 30–50 decisions. The team begins to use the search function — finding past decisions relevant to current investment ideas.

2. **Month 5–6:** A new team member joins. Instead of spending weeks asking colleagues about the firm's investment philosophy and past decisions, they spend two days reading the Decision Archive. The onboarding value proposition becomes concrete.

3. **Month 6:** The first cohort post-mortem analysis. The team reviews a cohort of decisions from the past six months — not individual decisions, but patterns across decisions. Which assumptions failed most often? Which sectors generated the most accurate theses? Which AI tools contributed to the best decisions?

4. **Month 6:** The first time a portfolio manager says "let me check what we did last time" and finds the answer in the archive rather than asking a colleague.

### What the team experiences

The archive value stage is where the product's long-term value proposition becomes tangible. The team begins to experience the Decision Archive as an asset — something that makes them better at their jobs, not just something that documents what they did.

The most significant moment in this stage is the new team member onboarding. When a new analyst or portfolio manager can understand the firm's investment philosophy and past decisions by reading the archive — without asking anyone — the institutional memory value proposition becomes undeniable.

The second significant moment is the first cohort post-mortem. When the team can review patterns across 50 decisions — not just individual decisions — the learning value of the archive becomes apparent.

### Success indicators

- Archive contains 50+ decisions
- Team uses the search function at least weekly
- At least one new team member onboarded using the archive
- At least one cohort post-mortem completed
- At least one instance of "let me check the archive" replacing "let me ask a colleague"

### What can go wrong

**The archive is incomplete.** If the team did not create Decision Workspaces for all decisions in Months 1–3, the archive will have gaps. Gaps reduce the value of the archive and undermine the institutional memory proposition. Completeness is essential.

**The search function is not used.** If the team does not discover the search function, the archive remains a passive record rather than an active resource. The product must surface relevant past decisions proactively — not just when searched.

**The post-mortem is skipped.** If the team does not conduct a cohort post-mortem by Month 6, they will not experience the learning value of the archive. The post-mortem should be scheduled as part of the onboarding process.

---

## Stage 5 — Decision Intelligence (Year 1)

### What happens

By Year 1, the Decision Archive contains 100–200 decisions. This is the point at which the archive begins to generate Decision Intelligence — patterns across decisions that improve future decision-making.

The Decision Intelligence sequence:

1. **Month 9–10:** The first pattern emerges. The team identifies a recurring assumption failure — a type of assumption that has been wrong more often than right. The pattern is documented and shared with the investment committee.

2. **Month 10–11:** The first AI tool performance analysis. Which AI tools contributed to decisions that outperformed? Which contributed to decisions that underperformed? The analysis is not definitive — correlation, not causation — but it is informative.

3. **Month 12:** The annual investment process review. The CIO uses the Decision Archive to evaluate the firm's decision-making process — not just the outcomes, but the quality of the reasoning. Which assumptions were well-supported by evidence? Which were not? Where did the committee add value? Where did it not?

4. **Year 1 milestone:** The Decision Archive is recognised as a strategic asset. The CIO mentions it in LP communications. The firm's investment process is described in terms of the Decision Archive.

### What the team experiences

The Decision Intelligence stage is where ChronoSentiment becomes part of the firm's identity — not just a tool they use, but a capability that defines how they make decisions.

The most significant moment in this stage is the annual investment process review. When the CIO can evaluate the firm's decision-making process using the Decision Archive — not just the outcomes, but the quality of the reasoning — the product has become genuinely strategic.

The second significant moment is the first time the Decision Archive is mentioned in LP communications. When the CIO describes the firm's investment process in terms of the Decision Archive — "we document every decision, capture all AI tool usage, and conduct quarterly post-mortem analysis" — the archive has become a competitive differentiator.

### Success indicators

- Archive contains 100+ decisions
- At least one recurring pattern identified and documented
- Annual investment process review conducted using the archive
- Decision Archive mentioned in LP communications
- CIO describes ChronoSentiment as a competitive differentiator

### What can go wrong

**The team stops using the product.** If the team's usage drops after Month 6 — if the habit does not persist — the archive will stop growing and the Decision Intelligence value proposition will not be reached. Consistent usage is essential.

**The patterns are not acted on.** If the team identifies patterns in the archive but does not change their decision-making process in response, the learning value is lost. The post-mortem process must include a mechanism for translating patterns into process changes.

**The archive is not mentioned externally.** If the Decision Archive is not mentioned in LP communications or design partner conversations, the competitive differentiation value is not realised. The CIO must be willing to describe the archive as a capability.

---

## The Adoption Journey and the Moat

The adoption journey is not just a customer success framework. It is the mechanism by which ChronoSentiment's primary moat — the Decision Archive — is built.

Each stage of the adoption journey adds to the archive:

```
Week 1–2    3–5 historical decisions imported
Month 1     10–20 new decisions created
Month 3     30–50 decisions in archive
Month 6     50–100 decisions in archive
Year 1      100–200 decisions in archive
```

The switching cost increases at each stage. By Year 1, a firm with 100–200 decisions in the archive cannot easily move to a different system — the archive is not portable in any meaningful sense. The institutional memory built in ChronoSentiment cannot be reconstructed from scratch.

This is why the adoption journey is part of the product. The moat is not built by the technology alone — it is built by the habits that form in Months 1–3 and the archive that accumulates in Months 3–12.

---

## Design Partner Implications

For design partners, the adoption journey has specific implications.

**The design partner commitment is Month 1–3.** The most important period for a design partner is the first three months — Activation, Habit Formation, and Governance Integration. If the design partner reaches Month 3 with the committee workflow integrated and the first LP query answered using a Decision Record, the design partnership has succeeded.

**The design partner feedback is most valuable in Month 1.** The friction points in Habit Formation — evidence capture, AI conversation export, committee workflow — are the most important product feedback. Design partners should be asked specifically about these friction points in Month 1.

**The design partner reference is most credible at Month 6.** A design partner who has reached the Archive Value stage — with 50+ decisions in the archive, a new team member onboarded using the archive, and a cohort post-mortem completed — is the most credible reference for prospective customers.

---

*CS-SUC-001 Customer Success Blueprint v1.0 | July 2026 | ChronoSentiment*
*Describes the adoption journey from Week 1 through Year 1.*
*Review trigger: After first design partner completes Month 3; after first full-year customer.*