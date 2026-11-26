# CS-R-004 — Regulatory Landscape
## ChronoSentiment Research Series | v2.0

---

## Document Governance

| Field | Value |
|-------|-------|
| Document Status | **Research Baseline v2.0** |
| Evidence Version | v2.0 |
| Research Date | July 2026 |
| Evidence Cut-off Date | July 2026 |
| Next Review | January 2027 or upon material regulatory development |
| Owner | ChronoSentiment Programme |
| Review Trigger | EU AI Act enforcement action; SEC or FCA formal AI guidance; IOSCO new AI framework; NIST AI RMF major update |

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
| CS-R-001 Market Landscape v2.0 | Regulatory requirements apply across all customer segments identified |
| CS-R-003 Customer Problem Evidence v2.0 | Regulatory convergence is the primary urgency driver for customer problems |
| CS-R-007 Explainability Research v2.0 | Explainability requirements are the technical expression of regulatory mandates |
| CS-R-011 Decision Governance Research | Decision governance requirements are the operational expression of regulatory mandates |
| CS-R-014 Product Category Creation Study | Regulatory tailwind supports category creation timing argument |

**Feeds into:** PRD v1.0 (regulatory context), Phase 1B customer validation (regulatory urgency testing), M-series architecture (compliance requirements)

---

## Research Limitations

This document analyses published regulatory material only. It does not establish:

- How investment firms currently interpret these regulations in practice
- Whether regulatory concerns actually influence software purchasing decisions
- Whether compliance budgets exist for decision governance platforms
- Which specific regulations are most salient to CIOs, portfolio managers, and compliance officers
- How firms are currently documenting AI-assisted investment decisions

These questions require Phase 1B customer validation. The regulatory analysis here provides the factual foundation for that research, not a substitute for it.

---

## 1. Purpose and Scope

This document maps the regulatory landscape relevant to ChronoSentiment as of July 2026. It covers: EU AI Act, FCA (UK), SEC (US), ESMA (EU securities markets), IOSCO (international standard-setter), NIST AI Risk Management Framework, OECD AI Principles, and ISO/IEC AI governance standards.

**Central finding (Evidence):** Global regulatory convergence is occurring across six themes — explainability, auditability, human oversight, governance, accountability, and decision lineage. No single jurisdiction has yet mandated a specific decision governance platform, but the convergence of requirements creates a structural backdrop for investment firms to invest in decision governance infrastructure.

**Interpretation (Confidence D):** ChronoSentiment's planned architecture appears well aligned with these obligations. Whether this alignment translates into purchasing urgency requires Phase 1B validation.

---

## 2. Evidence

### 2.1 EU AI Act (Regulation 2024/1689)

The EU AI Act entered into force on 1 August 2024 with a phased implementation schedule. It is the most comprehensive AI-specific regulation globally. **Confidence A.**

**Phased implementation timeline:**

| Phase | Date | Scope |
|-------|------|-------|
| Phase 1 — Prohibited practices | February 2025 | Bans on unacceptable-risk AI systems |
| Phase 2 — GPAI model obligations | August 2025 | General-purpose AI model providers |
| Phase 3 — High-risk AI systems | August 2026 | Full obligations for high-risk AI in regulated sectors |
| Phase 4 — Full enforcement | 2027 onwards | Complete framework including notified body assessments |

**Evidence:** AI systems used in financial decision-making are classified as high-risk under Annex III of the EU AI Act (Article 6). High-risk classification triggers obligations under Articles 9–15, including: risk management systems (Art. 9), data governance (Art. 10), technical documentation (Art. 11), automatic logging of events (Art. 12), transparency to enable users to interpret outputs (Art. 13), human oversight measures (Art. 14), and accuracy and robustness requirements (Art. 15). **Confidence A — directly verifiable from published regulation text.**

**Interpretation (Confidence B):** The precise scope of "investment decisions affecting natural persons" under Annex III will be clarified by implementing acts from the AI Office. The classification of specific investment management AI systems as high-risk is not yet definitively settled.

### 2.2 FCA (Financial Conduct Authority, UK)

The FCA has not issued AI-specific rules as of July 2026 but has articulated supervisory principles through multiple publications and speeches. **Confidence A.**

**Evidence:** FCA supervisory positions include: governance frameworks with clear accountability under SMCR; ability to explain AI-assisted decisions to customers and regulators; demonstration of good consumer outcomes under Consumer Duty (effective July 2023); and model risk management principles (SS1/23, published jointly with PRA) requiring validation, monitoring, and governance of model outputs. **Confidence A — from published FCA guidance and speeches.**

**Interpretation (Confidence B):** How these principles apply to specific investment management AI systems has not been tested through enforcement. The FCA's principles-based approach means that compliance is assessed against outcomes, not specific technical requirements.

### 2.3 SEC (Securities and Exchange Commission, US)

The SEC has taken a governance-focused approach, emphasising existing obligations rather than new AI-specific rules. **Confidence A.**

**Evidence:** Key SEC positions include: investment advisers cannot delegate fiduciary responsibility to AI systems; a proposed rule on conflicts of interest in predictive data analytics (2023, under review as of July 2026) would require evaluation and mitigation of AI-related conflicts; existing record-keeping rules (Rule 17a-4; Rule 204-2) require records of investment decisions and their basis, and the SEC has indicated these apply to AI-assisted decisions; AI governance and documentation of AI-assisted decisions are stated examination priorities for 2025–2026. **Confidence A for published guidance; Confidence B for the status of the proposed predictive analytics rule.**

**Interpretation (Confidence B):** The SEC's approach is principles-based and enforcement-driven. Specific technical requirements for AI documentation have not been formally codified. The examination priority signal is meaningful but not equivalent to a formal rule.

### 2.4 ESMA (European Securities and Markets Authority)

ESMA has published guidance on AI in investment management and is developing a more comprehensive framework in parallel with EU AI Act implementation. **Confidence A.**

**Evidence:** ESMA has clarified that MiFID II obligations (suitability, best execution, conflicts of interest) apply regardless of whether decisions are made by humans or AI systems. ESMA's algorithmic trading guidelines (MiFID II Article 17) require governance, testing, and monitoring of algorithmic systems, and these requirements are being extended to AI-assisted investment decision systems. ESMA published a report on AI in investment management (2024) identifying governance, explainability, and human oversight as key supervisory priorities. **Confidence A for published guidance; Confidence B for developing frameworks.**

### 2.5 IOSCO (International Organization of Securities Commissions)

IOSCO is the international standard-setter for securities regulators and its guidance influences regulatory approaches globally. **Confidence A.**

**Evidence:** IOSCO published a report on AI and machine learning in capital markets (2021, updated 2024) identifying governance, explainability, accountability, and auditability as core requirements for AI systems in regulated financial markets. IOSCO is working to promote consistent regulatory approaches across jurisdictions. **Confidence A for published reports.**

### 2.6 NIST AI Risk Management Framework (AI RMF 1.0)

The NIST AI RMF (published January 2023, with ongoing updates) provides a voluntary framework for managing AI risks. It is widely adopted by US financial services firms and referenced by SEC. **Confidence A.**

**Evidence:** The NIST AI RMF organises AI risk management around four functions: GOVERN (policies, accountability), MAP (risk identification and categorisation), MEASURE (quantitative and qualitative risk analysis including explainability assessment), and MANAGE (risk controls, incident response). The framework is voluntary but is referenced by SEC examiners as a best-practice standard. **Confidence A — from published NIST documentation and SEC examination guidance.**

### 2.7 OECD AI Principles (2019, updated 2024)

The OECD AI Principles are the international policy framework for AI governance, adopted by 46 countries including all G7 members. **Confidence A.**

**Evidence:** Key principles include: transparency and explainability (Principle 1.3), robustness, security, and safety including auditability (Principle 1.4), and accountability for the proper functioning of AI systems including maintaining records of AI decisions (Principle 1.5). The OECD AI Principles are not legally binding but are influential in shaping national regulatory approaches. **Confidence A — from published OECD documentation.**

### 2.8 ISO/IEC AI Governance Standards

The ISO/IEC JTC 1/SC 42 committee is developing a suite of AI governance standards. **Confidence B.**

**Evidence:** Key standards include ISO/IEC 42001:2023 (AI management system), ISO/IEC 23894:2023 (AI risk management guidance), and ISO/IEC 38507:2022 (governance implications of AI use). These standards are voluntary but are increasingly referenced by regulators and institutional investors as evidence of AI governance maturity. **Confidence B — standards are published but adoption in investment management is not yet systematically documented.**

---

## 3. Research Findings

### Finding 1: Global regulatory convergence is occurring across six themes (Confidence A)

Across all jurisdictions and standard-setting bodies assessed, regulatory requirements are converging on six themes:

| Theme | EU AI Act | FCA | SEC | ESMA | IOSCO | NIST | OECD |
|-------|-----------|-----|-----|------|-------|------|------|
| Explainability | Art. 13 | ✅ | ✅ | ✅ | ✅ | MEASURE | Principle 1.3 |
| Auditability | Art. 12 | ✅ | ✅ | ✅ | ✅ | MANAGE | Principle 1.4 |
| Human oversight | Art. 14 | ✅ | ✅ | ✅ | ✅ | GOVERN | Principle 1.4 |
| Governance | Art. 9 | SMCR | ✅ | ✅ | ✅ | GOVERN | Principle 1.5 |
| Accountability | Art. 9 | SMCR | ✅ | ✅ | ✅ | GOVERN | Principle 1.5 |
| Decision lineage | Art. 12 | ✅ | Rule 204-2 | ✅ | ✅ | MAP | Principle 1.3 |

### Finding 2: EU AI Act Phase 3 (August 2026) is the most significant near-term regulatory event (Confidence A)

The full obligations for high-risk AI systems under the EU AI Act take effect in August 2026. Investment management firms using AI in decision-making that affects EU persons must comply with Articles 9–15 by this date. This is an established regulatory fact. Whether it creates purchasing urgency for decision governance platforms is a separate question requiring Phase 1B validation.

### Finding 3: No regulator has mandated a specific decision governance platform (Confidence A)

As of July 2026, no regulator has mandated the use of a specific decision governance platform or technology. Regulatory requirements are expressed as principles and outcomes (explainability, auditability, human oversight) rather than specific technical solutions.

### Finding 4: Existing record-keeping rules already require decision documentation (Confidence A)

SEC Rule 204-2 (investment advisers) and MiFID II record-keeping requirements already require firms to maintain records of investment decisions and the basis for those decisions. These existing requirements predate AI-specific regulation and are not being consistently met by current technology.

### Finding 5: Regulatory requirements are increasing in specificity over time (Confidence B)

The trajectory of regulatory guidance is toward greater specificity: from general principles (OECD 2019) to detailed technical requirements (EU AI Act 2024). This trajectory suggests that future regulatory requirements will be more specific and more demanding. **Confidence B — this is a directional assessment, not a prediction.**

---

## 4. Evidence Sufficiency Assessment

| Area | Evidence Sufficiency | Notes |
|------|---------------------|-------|
| EU AI Act text and obligations | High | Published regulation; directly verifiable |
| FCA supervisory principles | High | Published guidance and speeches |
| SEC guidance and examination priorities | High | Published guidance; proposed rule status uncertain |
| ESMA and MiFID II obligations | High | Published guidance |
| IOSCO and international standards | High | Published reports |
| NIST AI RMF | High | Published framework |
| Customer regulatory awareness | Low | Not established — requires Phase 1B |
| Regulatory influence on purchasing decisions | Low | Not established — requires Phase 1B |
| Compliance budget availability | Low | Not established — requires Phase 1B |
| Regulatory urgency as a sales driver | Low | Not established — requires Phase 1B |

---

## 5. Outstanding Validation Questions

The following questions cannot be answered from published regulatory material and require Phase 1B primary research:

1. **Regulatory awareness:** Are CIOs, portfolio managers, and compliance officers aware of the EU AI Act's high-risk AI obligations? Which regulations are most salient to each stakeholder group?
2. **Purchasing influence:** Do regulatory requirements actually influence software purchasing decisions, or are they addressed through internal process changes?
3. **Budget ownership:** Who owns AI governance budgets? Is this a compliance function, a technology function, or an investment function?
4. **Current documentation practice:** How are firms currently documenting AI-assisted investment decisions? What tools are they using?
5. **Operational vs compliance framing:** Is AI governance considered an operational issue or a compliance issue? This affects which budget it competes for.
6. **Enforcement risk perception:** Do firms perceive meaningful enforcement risk for AI documentation failures, or is this seen as a future concern?

**Research method:** Phase 1B customer interviews with compliance officers, CIOs, and portfolio managers at target customer segments.

---

## 6. Strategic Interpretation: Governance Opportunity

*This section contains ChronoSentiment's interpretation of the regulatory landscape. Everything above is research. Everything below is interpretation. Confidence D throughout unless otherwise noted.*

**The governance opportunity framing:** The regulatory landscape creates a governance opportunity for ChronoSentiment, not a compliance mandate. The distinction is important:

- **Compliance mandate framing (avoid):** "Buy ChronoSentiment to comply with the EU AI Act." This framing is inaccurate (no specific platform is mandated), creates dependency on regulatory outcomes, and positions ChronoSentiment as a cost centre.
- **Governance opportunity framing (preferred):** "ChronoSentiment builds the decision governance infrastructure that regulators are increasingly expecting firms to demonstrate. Firms that invest in decision governance capabilities are likely to be better positioned as regulatory expectations continue to mature."

**The timing argument (Confidence D):** EU AI Act Phase 3 obligations (August 2026) create a near-term compliance event for EU-exposed investment management firms. This may create purchasing urgency in H1 2026 for firms seeking to implement solutions before the deadline. Whether this urgency translates into purchasing decisions for a decision governance platform requires Phase 1B validation.

**The differentiation argument (Confidence D):** No existing investment management platform addresses point-in-time documentation requirements for AI-assisted decisions (CS-R-002). The regulatory requirements are specific enough that a purpose-built solution has a defensible position against general-purpose compliance tools. This requires competitive validation.

---

## 7. PRD Traceability

| PRD Assumption | Regulatory Evidence | Evidence Status |
|----------------|--------------------|-----------------| 
| AI governance demand is increasing | EU AI Act, FCA, SEC, ESMA, IOSCO convergence | Supported (Confidence A) |
| Explainability is becoming expected | EU AI Act Art. 13, FCA principles, IOSCO, NIST MEASURE | Supported (Confidence A) |
| Decision provenance is becoming important | EU AI Act Art. 12, IOSCO, CFA Institute 2026 | Supported (Confidence B) |
| Regulatory requirements create purchasing urgency | Not established in published regulatory material | Not yet validated — Phase 1B |
| Customers will buy because of regulation | Not established | Not yet validated — Phase 1B |
| ChronoSentiment architecture meets regulatory requirements | Internal assessment only | Not yet validated — legal review required |

---

## 8. Emerging Regulatory Themes

| Theme | Current Status | Trajectory | ChronoSentiment Relevance |
|-------|---------------|-----------|--------------------------|
| AI decision provenance | Emerging guidance (IOSCO, ESMA) | Likely to become formal requirement | Core capability |
| Model cards / system cards | NIST AI RMF recommendation | Likely to become standard practice | Supports explainability output |
| AI incident reporting | EU AI Act Art. 73 (providers) | Likely to extend to deployers | Decision records enable incident reconstruction |
| Third-party AI governance | EU AI Act supply chain | Expanding | ChronoSentiment as governance layer for third-party AI |
| Cross-border AI governance | IOSCO coordination | Increasing harmonisation | Single platform for multi-jurisdiction compliance |

---

## 9. Recommendations

**Recommendation 1: Lead with governance opportunity, not compliance mandate.**
All customer-facing materials should frame ChronoSentiment as a governance investment, not a compliance tool for a specific regulation. This framing is more accurate, more durable, and more commercially attractive. *Priority: High. Required before Phase 1B.*

**Recommendation 2: Prioritise EU-exposed firms in Phase 1B.**
EU AI Act Phase 3 obligations (August 2026) create the most concrete near-term regulatory event. EU-exposed investment management firms are the highest-urgency segment for Phase 1B customer validation. *Priority: High. Phase 1B targeting.*

**Recommendation 3: Map ChronoSentiment capabilities to NIST AI RMF functions.**
Develop a capability mapping document showing how ChronoSentiment addresses each NIST AI RMF function (GOVERN, MAP, MEASURE, MANAGE). This provides a credible framework for US market positioning and aligns with SEC examination priorities. *Priority: Medium. Phase 1B or Phase 2.*

**Recommendation 4: Monitor EU AI Act implementing acts and guidance.**
The EU AI Office is developing implementing acts that will clarify which investment management AI systems fall under high-risk classification. Monitor and update CS-R-004 when guidance is published. *Priority: Medium. Ongoing.*

**Recommendation 5: Develop a regulatory alignment matrix for customer conversations.**
Create a one-page matrix mapping ChronoSentiment capabilities to specific regulatory requirements (EU AI Act articles, FCA principles, SEC rules, NIST functions). Validate this matrix with compliance officers in Phase 1B. *Priority: Medium. Phase 1B.*

---

## Evidence Quality

| Source Type | Examples | Confidence |
|-------------|---------|-----------|
| Primary regulatory text | EU AI Act (Regulation 2024/1689), MiFID II, SEC Rule 204-2 | A |
| Regulatory guidance and speeches | FCA supervisory principles, SEC examination priorities, ESMA reports | A |
| International standards | IOSCO reports, NIST AI RMF 1.0, OECD AI Principles | A |
| ISO/IEC standards | ISO/IEC 42001, 23894, 38507 | B |
| Regulatory trajectory interpretation | Increasing specificity finding | B |
| Product implications | ChronoSentiment alignment with regulatory requirements | D |

---

## Evidence Classification

**Published evidence:** EU AI Act text and implementation timeline, FCA supervisory publications, SEC guidance and examination priorities, ESMA reports, IOSCO reports, NIST AI RMF 1.0, OECD AI Principles 2024, ISO/IEC standards.

**Derived findings:** Six-theme convergence table derived from independent regulatory sources; EU AI Act Phase 3 urgency derived from published implementation timeline; existing record-keeping gap derived from regulatory text.

**Strategic interpretation (Confidence D):** Governance opportunity framing; regulatory trajectory toward greater specificity; ChronoSentiment positioning as governance infrastructure; PRD traceability assessments marked "Not yet validated." These require Phase 1B customer conversations with compliance officers and risk managers before acting as the basis for M-series investment.

---

*CS-R-004 v2.0 | ChronoSentiment Research Series | Evidence cut-off: July 2026*
*Supersedes CS-R-004 v1.1. v1.1 retained as historical record.*