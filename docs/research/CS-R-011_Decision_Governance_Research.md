# CS-R-011 — Decision Governance Research
## ChronoSentiment Research Series | v1.0

---

## Document Governance

| Field | Value |
|-------|-------|
| Document Status | **Research Baseline v1.0** |
| Evidence Version | v1.0 |
| Research Date | July 2026 |
| Evidence Cut-off Date | July 2026 |
| Next Review | January 2027 or upon material new decision governance research |
| Owner | ChronoSentiment Programme |
| Review Trigger | New academic or practitioner research on decision governance in financial services; material regulatory development on AI decision documentation |

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
| CS-R-003 Customer Problem Evidence v2.0 | Decision governance is the integrated solution to the five customer problems |
| CS-R-004 Regulatory Landscape v2.0 | Regulatory requirements are the external driver of decision governance demand |
| CS-R-007 Explainability Research v2.0 | Explainability is the output layer of the decision governance system |
| CS-R-008 Point-in-Time Architecture v2.0 | PIT architecture is the data layer of the decision governance system |
| CS-R-010 Investment Workflow Evolution | Decision governance must integrate with evolved investment workflows |
| CS-R-014 Product Category Creation Study | Decision governance is the category ChronoSentiment is creating |

**Feeds into:** PRD v1.0 (product definition), M-series architecture (governance system design), Phase 1B customer validation (governance framing validation)

---

## Research Limitations

This document synthesises academic, practitioner, and regulatory research on decision governance. The field of "investment decision governance" as a distinct discipline is nascent — most relevant research comes from adjacent fields (corporate governance, AI governance, decision science, financial regulation). It does not establish:

- Whether investment management firms use the term "decision governance" or recognise it as a category
- Whether decision governance is a budget category in target firms
- How firms currently approach decision governance informally
- Whether ChronoSentiment's decision governance framing resonates with target customers

These questions require Phase 1B primary research.

---

## 1. Purpose and Scope

This document surveys the academic and practitioner literature on decision governance as it applies to investment management. It covers: decision science foundations, corporate governance frameworks, AI governance requirements, financial services regulatory guidance, and the emerging concept of investment decision governance as a distinct discipline.

**Central finding:** Decision governance — the systematic capture, documentation, and accountability of consequential decisions — is a well-established concept in corporate governance and risk management but has not been applied systematically to investment management decisions. The combination of AI adoption, regulatory convergence, and increasing accountability requirements is creating the conditions for investment decision governance to emerge as a distinct discipline and product category.

---

## 2. Evidence

### 2.1 Decision Science Foundations

**Decision quality frameworks:** The decision quality framework (Howard and Abbas, *Foundations of Decision Analysis*, 2015) defines a high-quality decision as one that is: well-framed, based on the right information, with clear alternatives, sound reasoning, commitment to action, and appropriate values. This framework is widely used in corporate decision-making but has not been systematically applied to investment management. **Confidence A.**

**Decision documentation:** Research on decision documentation in organisations (Nutt, *Why Decisions Fail*, 2002; Klein, *Sources of Power*, 1998) consistently finds that decisions made without explicit documentation of rationale are more likely to be reversed, misimplemented, or forgotten. Documentation is not just a governance requirement — it improves decision quality. **Confidence B.**

**Hindsight bias:** Psychological research on hindsight bias (Fischhoff, 1975; Roese and Vohs, 2012) demonstrates that people systematically overestimate how predictable past events were. In investment management, hindsight bias distorts post-hoc evaluation of investment decisions — decisions that turned out badly are judged as obviously wrong in retrospect, even when they were reasonable given the information available at the time. Point-in-time documentation is the only reliable defence against hindsight bias in investment decision review. **Confidence A.**

**Outcome bias:** Related to hindsight bias, outcome bias (Baron and Hershey, 1988) is the tendency to evaluate the quality of a decision based on its outcome rather than the quality of the decision process. Investment management is particularly susceptible to outcome bias — a good decision that produces a bad outcome is judged as a bad decision. Systematic decision documentation enables process-based evaluation independent of outcomes. **Confidence A.**

### 2.2 Corporate Governance Frameworks

**Board-level decision governance:** Corporate governance frameworks (OECD Principles of Corporate Governance, 2023; UK Corporate Governance Code, 2024) require boards to maintain records of significant decisions, including the information considered, the alternatives evaluated, and the rationale for the chosen course of action. These requirements are well-established for board-level decisions but have not been extended to investment decisions within asset management firms. **Confidence A.**

**Risk governance:** The Basel Committee on Banking Supervision's *Principles for the Sound Management of Operational Risk* (2011, updated 2021) and the Financial Stability Board's *Principles for Sound Compensation Practices* (2009) both emphasise the importance of decision documentation in risk governance. These principles apply to banks and broker-dealers but are increasingly referenced by asset managers as governance best practices. **Confidence A.**

**Three lines of defence:** The three lines of defence model (IIA, 2020) assigns governance responsibilities across: (1) business functions (first line — own and manage risk), (2) risk and compliance functions (second line — oversee and challenge), and (3) internal audit (third line — provide independent assurance). Investment decision governance sits primarily in the first line but requires documentation that enables second and third line oversight. **Confidence A.**

### 2.3 AI Governance Frameworks

**NIST AI RMF:** The NIST AI Risk Management Framework (2023) defines AI governance as the policies, processes, and accountability structures that ensure AI systems are developed and deployed responsibly. For investment management, AI governance includes: documentation of AI system purpose and design, monitoring of AI system performance, explainability of AI-assisted decisions, and accountability for AI governance failures. **Confidence A.**

**EU AI Act governance requirements:** EU AI Act Articles 9–15 establish specific governance requirements for high-risk AI systems, including: risk management systems, data governance, technical documentation, automatic logging, transparency, human oversight, and accuracy requirements. These requirements are the most specific regulatory expression of AI governance requirements for investment management. **Confidence A.**

**Model risk management:** The OCC/Federal Reserve *Supervisory Guidance on Model Risk Management* (SR 11-7, 2011) established model risk management as a governance discipline for banks. This guidance has been widely adopted by asset managers as a best practice framework for managing quantitative models. The extension of model risk management principles to AI models and LLMs is an emerging governance requirement. **Confidence B.**

### 2.4 Investment Decision Governance: Emerging Discipline

**CFA Institute standards:** The CFA Institute *Asset Manager Code* (2009, updated 2022) requires asset managers to: maintain records of investment decisions and the basis for those decisions, document the investment process, and be able to explain investment decisions to clients and regulators. These requirements establish investment decision documentation as a professional standard, but compliance is inconsistent and the standards do not specify technical implementation. **Confidence A.**

**GIPS (Global Investment Performance Standards):** GIPS requires firms to maintain records supporting the calculation and presentation of performance. This includes records of portfolio decisions, but GIPS focuses on performance calculation rather than decision governance. **Confidence A.**

**Emerging practitioner frameworks:** A small number of asset managers and consultants have begun developing investment decision governance frameworks. These frameworks typically include: decision capture templates, investment committee governance protocols, decision review processes, and performance attribution linked to decision records. These frameworks are not yet standardised or widely adopted. **Confidence C.**

**Academic research gap:** A systematic review of academic literature on investment decision governance reveals a significant gap: while there is extensive research on investment decision-making (behavioural finance, portfolio theory, factor investing), there is very limited research on the governance of investment decisions as a distinct discipline. This gap is consistent with the category creation opportunity identified in CS-R-002 and CS-R-014. **Confidence B.**

### 2.5 Decision Governance Technology Landscape

**Current technology for decision governance:** As documented in CS-R-002, no current vendor provides an integrated investment decision governance platform. Current approaches include: research management systems (AlphaSense, FactSet) for information capture; note-taking tools (Notion, Confluence) for decision documentation; email and messaging platforms for decision communication; and TCA tools for execution analysis. None of these tools provide integrated decision governance. **Confidence B.**

**Adjacent technology categories:** Several adjacent technology categories address parts of the decision governance problem: GRC (Governance, Risk, and Compliance) platforms (ServiceNow, MetricStream) address enterprise governance but are not designed for investment decisions; AI governance platforms (Fiddler AI, Arthur AI) address model governance but not investment decision governance; workflow automation platforms (Salesforce, Microsoft Power Platform) could be configured for decision capture but require significant customisation. **Confidence B.**

---

## 3. Research Findings

### Finding 1: Decision governance is a well-established concept in adjacent disciplines but nascent in investment management (Confidence B)

Corporate governance, risk management, and AI governance all have well-developed decision governance frameworks. Investment management has professional standards (CFA Institute, GIPS) that require decision documentation, but these standards are not systematically implemented and no technology platform has been built specifically to support them.

### Finding 2: Psychological research provides a strong evidence base for the value of decision documentation (Confidence A)

Hindsight bias and outcome bias are well-documented psychological phenomena that distort post-hoc evaluation of investment decisions. Point-in-time decision documentation is the only reliable defence against these biases. This provides a strong, evidence-based rationale for decision governance that is independent of regulatory requirements.

### Finding 3: AI adoption is creating new decision governance requirements that existing frameworks do not address (Confidence B)

Existing decision governance frameworks (CFA Institute, GIPS, model risk management) were designed for human decisions and quantitative models. The introduction of LLM-generated content into investment workflows creates new governance requirements — AI attribution, version control, prompt documentation — that existing frameworks do not address.

### Finding 4: The three lines of defence model creates a structural demand for decision governance infrastructure (Confidence B)

The three lines of defence model requires that investment decisions be documented in a way that enables second-line (risk and compliance) and third-line (internal audit) oversight. Current documentation practices (email, meeting notes, research memos) do not provide the structured, searchable, auditable record that second and third line functions require. This creates a structural demand for decision governance infrastructure.

### Finding 5: Investment decision governance is an emerging discipline without an established technology category (Confidence B)

The combination of AI adoption, regulatory convergence, and increasing accountability requirements is creating the conditions for investment decision governance to emerge as a distinct discipline. No technology category currently serves this discipline. This is consistent with the category creation opportunity identified in CS-R-002 and CS-R-014.

---

## 4. Implications

**4.1 The psychological evidence base strengthens ChronoSentiment's value proposition beyond compliance.** Hindsight bias and outcome bias are well-documented problems in investment management. ChronoSentiment's point-in-time decision documentation directly addresses these biases, providing a value proposition that is independent of regulatory requirements and resonates with investment professionals who care about decision quality.

**4.2 The three lines of defence model provides a structural demand driver.** Risk and compliance functions (second line) and internal audit (third line) have a structural need for auditable decision records. ChronoSentiment can be positioned as the infrastructure that enables second and third line oversight of investment decisions — a governance infrastructure sale, not just a portfolio manager tool.

**4.3 The academic research gap is a category creation signal.** The absence of academic research on investment decision governance as a distinct discipline is consistent with the category creation opportunity. ChronoSentiment has the opportunity to define the discipline, not just serve an existing market.

**4.4 AI attribution is a new governance requirement with no current solution.** The requirement to document which AI model, version, and data contributed to an investment decision is new, growing, and not addressed by any current tool. This is ChronoSentiment's most defensible near-term differentiator.

---

## 5. Recommendations

**Recommendation 1: Use the psychological evidence base in customer conversations.**
Hindsight bias and outcome bias are concepts that investment professionals recognise and care about. Framing ChronoSentiment as a defence against these biases — "know what you knew when you decided, not what you know now" — is a compelling value proposition that does not depend on regulatory compliance. *Priority: High. Phase 1B messaging.*

**Recommendation 2: Target second and third line functions as co-buyers.**
Risk and compliance officers (second line) and internal audit (third line) have a structural need for auditable decision records. Including these stakeholders in Phase 1B interviews will reveal whether they are co-buyers or influencers in the ChronoSentiment purchasing decision. *Priority: High. Phase 1B.*

**Recommendation 3: Develop a decision governance framework document.**
ChronoSentiment should develop and publish a decision governance framework for investment management — defining the discipline, establishing best practices, and positioning ChronoSentiment as the technology that implements the framework. This is a category creation investment that builds credibility and generates inbound interest. *Priority: Medium. Phase 2.*

**Recommendation 4: Validate the "AI attribution" framing as the most urgent governance requirement.**
Phase 1B should test whether AI attribution — documenting which AI tools contributed to which decisions — is the most urgent governance requirement for target customers. If validated, this is the MVP feature that creates the most immediate value. *Priority: High. Phase 1B.*

---

## Evidence Quality

| Source Type | Examples | Confidence |
|-------------|---------|-----------|
| Academic research | Howard/Abbas decision quality, Fischhoff hindsight bias, Baron/Hershey outcome bias | A |
| Corporate governance standards | OECD Principles, UK Corporate Governance Code, IIA three lines of defence | A |
| Regulatory frameworks | NIST AI RMF, EU AI Act, OCC SR 11-7, Basel operational risk principles | A |
| Professional standards | CFA Institute Asset Manager Code, GIPS | A |
| Practitioner frameworks | Emerging investment decision governance frameworks | C |
| Technology landscape | CS-R-002 competitive analysis | B |
| Category creation conclusion | Strategic interpretation | D |

---

## Evidence Classification

**Published evidence:** Academic research on decision quality, hindsight bias, and outcome bias; corporate governance standards; regulatory frameworks (NIST AI RMF, EU AI Act, SR 11-7); CFA Institute standards and GIPS.

**Derived findings:** Investment decision governance as nascent discipline derived from academic literature review; three lines of defence demand driver derived from governance framework analysis; AI attribution as new requirement derived from AI adoption evidence.

**Strategic interpretation (Confidence D):** Category creation opportunity; psychological evidence base as commercial framing; second/third line as co-buyers; decision governance framework as category creation investment. These require Phase 1B validation before adoption as the basis for commercial strategy.

---

*CS-R-011 v1.0 | ChronoSentiment Research Series | Evidence cut-off: July 2026*