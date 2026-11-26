# ChronoSentiment — Product Definition v1.0

**Document type:** Product Requirements Document
**Version:** 1.0
**Status:** Draft
**Date:** 2026-07-23
**Owner:** Product

---

## The One-Sentence Answer

> ChronoSentiment is a **Financial Decision Intelligence Platform** whose execution-validation engine enables investment professionals to evaluate trading strategies under realistic, deterministic execution conditions before capital is deployed.

If someone asks to buy ChronoSentiment tomorrow, they are buying a system that gives their investment team a structured, auditable, and explainable record of how financial decisions were made — and why.

---

## 1. Target Customer

**Primary:** Mid-size investment firms (AUM $500M–$10B) with 5–50 investment professionals.

These firms are large enough to have structured investment processes but small enough that those processes are not yet fully systematised. They face increasing pressure from regulators, LPs, and boards to demonstrate that investment decisions are disciplined, documented, and defensible.

**Secondary:** Family offices, endowments, and corporate treasury teams with similar governance pressures.

**Not the primary target (yet):** Retail investors, algorithmic trading desks, or large institutional asset managers with existing proprietary systems.

---

## 2. Customer Problem

Investment teams make decisions under uncertainty, time pressure, and information overload. The problems they face are not primarily about prediction — they are about **process**:

- "Why did we make this decision?" — no structured record exists.
- "What information did we have at the time?" — context is lost after the fact.
- "How did our thesis evolve?" — the reasoning trail is in emails and meeting notes.
- "Can we explain this to our LP?" — the answer is often "not easily."
- "What would we do differently?" — no systematic way to review past decisions.

These are governance, auditability, and learning problems. They are not solved by better prediction models.

---

## 3. Value Proposition

ChronoSentiment gives investment teams:

1. **A structured decision record.** Every investment recommendation is captured with its rationale, the information available at the time, and the confidence level.

2. **An explainable recommendation engine.** Every signal and recommendation comes with a natural-language explanation that a non-technical stakeholder can read and challenge.

3. **A decision timeline.** A chronological view of how a thesis evolved — from initial signal to final decision to outcome.

4. **Scenario replay.** The ability to reconstruct any past decision exactly as it appeared at the time, using only the information that was available then.

5. **Counterfactual analysis.** "What would the recommendation have been if we had weighted this factor differently?"

6. **Audit-ready documentation.** A complete, timestamped record suitable for LP reporting, regulatory review, or internal governance.

---

## 4. Product Scope

### In scope for v1.0

- **Research workspace** — a structured environment for capturing and organising investment research, linked to specific securities and time periods.
- **Decision timeline** — a chronological record of recommendations, rationale, and outcomes for a portfolio or watchlist.
- **Recommendation engine** — current signals with confidence scores, supporting evidence, and natural-language explanations.
- **Execution replay** — deterministic reconstruction of a past decision using only the information available at that time.
- **Explainability** — natural-language explanation of every recommendation, including the factors considered and their relative weights.
- **Scenario comparison** — side-by-side comparison of two analysis runs with different parameters or time periods.
- **Portfolio dashboard** — holdings, exposure, and decision history at a glance.

### Out of scope for v1.0

- Execution (order management, brokerage integration).
- Real-time market data feeds (v1.0 uses historical and delayed data).
- Automated trading or algorithmic execution.
- Multi-asset class support beyond equities (v1.0 is equities-first).
- Team collaboration features (v1.0 is single-user or small-team).

---

## 5. MVP

The minimum viable product answers one question for one customer:

> "Given a portfolio of 20–30 equities, can ChronoSentiment produce a structured, explainable recommendation for each position, and can it replay any past recommendation exactly as it appeared at the time?"

**MVP deliverables:**
- Portfolio dashboard with 20–30 equity positions
- Recommendation engine with NL explanations for each position
- Decision timeline for the past 12 months
- Execution validation — deterministic replay of how a past recommendation would have behaved under realistic execution conditions (simulated order book, latency, queue dynamics)
- Export to PDF for LP reporting

**MVP success criteria:**
1. A portfolio manager can walk an LP through a past investment decision using ChronoSentiment in under 10 minutes, without referring to emails or meeting notes.
2. A quantitative analyst can replay any past recommendation and observe how it would have executed under realistic market conditions — with the same result on every run.

---

## 6. Differentiators

| Dimension | ChronoSentiment | Typical alternatives |
|-----------|----------------|---------------------|
| Explainability | Every recommendation explained in natural language | Black-box signals or factor scores |
| Decision record | Structured, timestamped, auditable | Emails, spreadsheets, meeting notes |
| Replay | Exact reconstruction of past decisions | Not available |
| Governance | Built-in audit trail | Retrofitted or absent |
| Positioning | Decision intelligence, not prediction | Prediction accuracy, alpha generation |

ChronoSentiment does not compete on prediction accuracy. It competes on **decision quality, explainability, and governance**.

---

## 7. Pricing Model

**Proposed:** Annual subscription, per-seat.

| Tier | Seats | AUM target | Annual price |
|------|-------|-----------|-------------|
| Starter | 1–3 | < $500M | $24,000/yr |
| Professional | 4–10 | $500M–$2B | $60,000/yr |
| Enterprise | 11–50 | $2B–$10B | $120,000–$240,000/yr |

**Rationale:** Investment firms are accustomed to paying for research and data subscriptions. The pricing is positioned below Bloomberg Terminal ($24,000/yr per seat) but above generic SaaS tools. The value is in governance and explainability, not data.

**Alternative:** Usage-based pricing tied to number of positions or decisions analysed. To be validated with early customers.

---

## 8. Success Metrics

### Product metrics (12-month targets)
- 5 paying pilot customers
- Net Promoter Score ≥ 40
- Time-to-first-recommendation < 30 minutes from onboarding
- Explanation coverage ≥ 95% (every recommendation has a full NL explanation)
- Decision replay accuracy = 100% (same input → same output, always)

### Business metrics (12-month targets)
- ARR ≥ $300,000
- Customer retention ≥ 90%
- Average contract value ≥ $60,000

### Evidence metrics (pre-commercial)
- M-series architecture validation complete
- P-001 Atlas Capital demonstration browser-verified
- DS-001 canonical dataset deterministic replay confirmed

---

## 9. Open Questions

These questions must be answered before M-series begins:

1. **Data sourcing:** Which public data sources will form the canonical dataset? (OHLCV, earnings, SEC filings, macro releases — which providers, which licences?)
2. **Explainability model:** What is the underlying model for generating NL explanations? (Rule-based, LLM-assisted, or hybrid?)
3. **Replay fidelity:** How do we guarantee that a replay uses only information available at the decision time? (Point-in-time data architecture.)
4. **Regulatory positioning:** Is ChronoSentiment a research tool or an investment adviser? (Determines regulatory obligations in each jurisdiction.)
5. **First customer:** Who is the Atlas Capital equivalent — a real firm willing to be the canonical demonstration customer?

---

## 10. Next Steps

1. Answer the five open questions above.
2. Identify one pilot customer willing to co-design the MVP.
3. Begin M-series architecture validation (observatory model, API stability, decision lineage, explainability design).
4. Begin P-001 production readiness (Atlas Capital demonstration entity, seven deliverables).
5. Do not begin DS-001 until P-001 is frozen.

---

*This document is the foundation for all subsequent ChronoSentiment architecture, evidence, and commercial decisions. It should be reviewed and updated after the first pilot customer engagement.*