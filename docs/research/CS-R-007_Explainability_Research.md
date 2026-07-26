# CS-R-007 — Explainability Research
## ChronoSentiment Research Series | v2.0

---

## Document Governance

| Field | Value |
|-------|-------|
| Document Status | **Research Baseline v2.0** |
| Evidence Version | v2.0 |
| Research Date | July 2026 |
| Evidence Cut-off Date | July 2026 |
| Next Review | January 2027 or upon material advance in LLM explainability or RAG architecture |
| Owner | ChronoSentiment Programme |
| Review Trigger | Material advance in deterministic LLM inference; new regulatory guidance on AI explainability standards; significant RAG architecture development relevant to financial decision governance |

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
| CS-R-003 Customer Problem Evidence v2.0 | Explainability is Problem 4 — the technical approach is defined here |
| CS-R-004 Regulatory Landscape v2.0 | EU AI Act Art. 13, FCA principles, and NIST AI RMF require explainability |
| CS-R-006 Data Landscape v2.0 | Explainability requires access to the data that informed the original decision |
| CS-R-008 Point-in-Time Architecture v2.0 | Explainability output depends on PIT data reconstruction |
| CS-R-011 Decision Governance Research | Explainability is the output layer of the decision governance system |

**Feeds into:** PRD v1.0 (explainability architecture), M-series architecture (LLM and RAG design), Phase 1B (explainability output validation with prospects)

---

## 1. Purpose and Scope

This document surveys the technical landscape for AI explainability as applied to investment decision governance. It evaluates: post-hoc explanation methods (SHAP, LIME), structured template approaches, retrieval-augmented generation (RAG), decision provenance as a first-class concept, model cards and system cards, and AI observability platforms.

**Central finding:** For investment decision governance, the appropriate explainability architecture is structured decision templates combined with deterministic LLM generation (temperature=0, version-pinned model) over a RAG context built from point-in-time data. Post-hoc methods (SHAP, LIME) are not appropriate as the primary explainability mechanism for human investment decisions. Decision provenance — tracking which data, models, and versions contributed to a decision — is a first-class architectural requirement, not an afterthought.

---

## 2. Evidence

### 2.1 Post-Hoc Explanation Methods: SHAP and LIME

**SHAP (SHapley Additive exPlanations)** — A game-theoretic approach to explaining the output of any machine learning model. SHAP assigns each feature an importance value for a particular prediction, based on the Shapley value from cooperative game theory. Published by Lundberg and Lee (2017, NeurIPS). **Confidence A.**

**LIME (Local Interpretable Model-agnostic Explanations)** — An approach to explaining individual predictions of any classifier or regressor by approximating the model locally with an interpretable model. Published by Ribeiro, Singh, and Guestrin (2016, KDD). **Confidence A.**

**Assessment for investment decision governance:**

SHAP and LIME are designed to explain the outputs of machine learning models — they answer the question "why did this model produce this output?" They are not designed to explain human investment decisions, which are not the output of a single ML model. Investment decisions involve: research synthesis, qualitative judgment, risk assessment, conviction formation, and execution intent. None of these are captured by feature attribution methods.

Specific limitations for ChronoSentiment's use case:
- SHAP/LIME require a trained ML model as input. Human investment decisions are not made by a single ML model.
- Feature attribution explains model behaviour, not human reasoning. A SHAP explanation of a portfolio manager's decision would require modelling the portfolio manager as an ML model — which is not feasible or appropriate.
- SHAP/LIME explanations are not human-readable in the way that regulatory and client audiences require. They produce feature importance scores, not natural-language narratives.
- SHAP/LIME do not capture the temporal dimension of investment decisions — the information environment at the time of decision.

**Conclusion: SHAP and LIME are rejected as the primary explainability mechanism for ChronoSentiment.** They may have a role in explaining specific quantitative model outputs within a broader decision governance framework, but they are not the core explainability architecture. **Confidence A.**

### 2.2 Structured Template Approaches

Structured templates for investment decision documentation are a well-established practice in investment management. Investment committees, regulatory filings, and client reporting all use structured templates to capture decision rationale. **Confidence A.**

**Template components for investment decision governance:**

- **Decision identifier:** Unique ID, timestamp, decision-maker, strategy/portfolio
- **Decision type:** New position, increase, decrease, exit, hedge, rebalance
- **Investment thesis:** Structured narrative of the investment case (bull case, bear case, key assumptions)
- **Information basis:** List of research, data, and analysis considered at decision time
- **Risk assessment:** Key risks identified and their assessed probability/impact
- **Conviction level:** Structured rating (high/medium/low) with rationale
- **Execution intent:** Target position, timeline, execution constraints
- **Review trigger:** Conditions that would cause the decision to be revisited

**Advantages of structured templates:**
- Human-readable and auditable
- Consistent across decisions and decision-makers
- Directly addresses regulatory requirements for decision documentation
- Captures the qualitative reasoning that SHAP/LIME cannot

**Limitations of structured templates alone:**
- Require discipline to complete at the point of decision (not post-hoc)
- Do not automatically link to the data and research that informed the decision
- Do not generate natural-language explanations from structured data
- Do not support temporal replay or reconstruction of the information environment

**Conclusion: Structured templates are a necessary but not sufficient component of ChronoSentiment's explainability architecture.** They provide the structured input that LLM generation and RAG can enhance. **Confidence A.**

### 2.3 Retrieval-Augmented Generation (RAG)

RAG is an architecture that combines a retrieval system (which fetches relevant documents or data) with a generative language model (which produces natural-language output based on the retrieved context). Published by Lewis et al. (2020, NeurIPS). RAG has become the dominant architecture for knowledge-grounded LLM applications as of 2024–2026. **Confidence A.**

**RAG architecture for investment decision explainability:**

```
Decision Record (structured template)
        +
Point-in-Time Data Context (CS-R-006, CS-R-008)
        │
        ▼
Retrieval System
(fetches relevant research, data, news, filings as they existed at decision time)
        │
        ▼
RAG Context
(decision record + retrieved PIT documents and data)
        │
        ▼
Deterministic LLM (temperature=0, version-pinned)
        │
        ▼
Natural-Language Explanation
(auditable, reproducible, grounded in PIT evidence)
```

**Key design requirements for ChronoSentiment RAG:**

1. **Point-in-time retrieval:** The retrieval system must fetch documents and data as they existed at the time of the original decision, not as they exist today. This requires PIT data infrastructure (CS-R-006, CS-R-008).
2. **Deterministic generation:** The LLM must be run at temperature=0 with a version-pinned model to ensure that the same input always produces the same output. This is essential for audit-grade explainability.
3. **Source attribution:** Every claim in the generated explanation must be attributable to a specific retrieved document or data point. This is the basis for decision provenance.
4. **Explanation versioning:** Generated explanations must be versioned and stored immutably. If the explanation is regenerated (e.g., with a different model version), both versions must be retained.

**Confidence A** for RAG architecture fundamentals. **Confidence B** for the specific design requirements for ChronoSentiment's use case.

### 2.4 Decision Provenance as a First-Class Concept

Decision provenance is the complete record of the inputs, processes, and outputs that contributed to a decision. It is analogous to data lineage in data engineering — tracking where data came from and how it was transformed — but applied to decision-making. **Confidence B.**

**Components of decision provenance for investment decisions:**

- **Data provenance:** Which data sources, at which point in time, contributed to the decision. Includes market data, fundamental data, macro data, news, research.
- **Model provenance:** Which AI models, at which version, were used to process or summarise data. Includes LLM version, embedding model version, retrieval system version.
- **Research provenance:** Which research documents, analyst reports, or expert opinions were considered. Includes document identifiers, publication dates, and access timestamps.
- **Decision provenance:** The complete chain from data inputs through research synthesis to decision rationale to execution intent.

**Decision provenance as a regulatory requirement:**

EU AI Act Article 12 (record-keeping) requires AI systems to automatically log events throughout the AI system's lifecycle. For ChronoSentiment, this means logging: which data was retrieved, which model version generated the explanation, and what the complete input context was. Decision provenance is the technical implementation of this regulatory requirement. **Confidence A.**

**CFA Institute *AI Pioneers in Investment Management* 2026** identifies decision provenance — knowing which AI model, which data, and which version generated a given output — as an emerging governance requirement. **Confidence B.**

### 2.5 Model Cards and System Cards

**Model cards** (Mitchell et al., 2019, FAccT) are structured documents that describe the intended use, performance characteristics, limitations, and ethical considerations of a machine learning model. They are a transparency mechanism for AI systems. **Confidence A.**

**System cards** extend the model card concept to AI systems composed of multiple models and components. Meta introduced system cards for its AI systems in 2022. **Confidence B.**

**Relevance to ChronoSentiment:**

ChronoSentiment's explainability output depends on specific LLM versions and retrieval system configurations. Model cards for the LLM components used in ChronoSentiment's explainability pipeline provide:
- Documentation of the model's training data, capabilities, and limitations
- Version-specific performance characteristics relevant to financial explanation generation
- Transparency for regulatory and audit purposes

**NIST AI RMF** recommends model cards as a transparency mechanism for AI systems in regulated contexts. **Confidence A.**

**Gartner *Hype Cycle for AI Governance* (2025)** identifies model cards as an emerging standard for AI transparency in regulated industries, with investment management cited as a primary use case. **Confidence B.**

### 2.6 AI Observability Platforms

AI observability platforms (Fiddler AI, Arthur AI, Arize AI, WhyLabs) provide monitoring, drift detection, and explainability for ML models in production. **Confidence A.**

**Assessment for ChronoSentiment:**

As noted in CS-R-002, AI observability platforms address model governance (monitoring ML model behaviour) rather than decision governance (explaining human investment decisions). However, they provide relevant architectural patterns:

- **Logging and tracing:** Observability platforms log all model inputs, outputs, and intermediate states. ChronoSentiment's decision provenance system should adopt similar logging patterns.
- **Drift detection:** Observability platforms detect when model behaviour changes over time. ChronoSentiment should monitor for drift in LLM explanation quality as model versions change.
- **Explanation consistency:** Observability platforms test whether explanations are consistent across similar inputs. ChronoSentiment should implement similar consistency testing for its explanation pipeline.

**Conclusion:** AI observability platforms provide useful architectural patterns but are not directly applicable to ChronoSentiment's use case. ChronoSentiment should implement its own observability layer for the explanation pipeline. **Confidence B.**

### 2.7 Deterministic LLM Inference

Deterministic LLM inference — running a language model at temperature=0 with a version-pinned model — is essential for audit-grade explainability. **Confidence A.**

**Why determinism matters for ChronoSentiment:**

- **Reproducibility:** An explanation generated today must be reproducible tomorrow. If the LLM is updated or run at non-zero temperature, the same input may produce different outputs. This breaks the audit trail.
- **Regulatory compliance:** EU AI Act Article 12 requires logging of AI system outputs. If outputs are non-deterministic, the logged output may not match a regenerated output, creating compliance risk.
- **Trust:** Investment professionals and regulators must be able to trust that an explanation accurately represents the decision rationale. Non-deterministic explanations undermine this trust.

**Implementation requirements:**
- Temperature=0 for all explanation generation
- Version-pinned LLM (specific model version, not "latest")
- Immutable storage of generated explanations with input context hash
- Explanation versioning when model version changes

**Current LLM providers supporting deterministic inference:** OpenAI (GPT-4o, temperature=0), Anthropic (Claude, temperature=0), Google (Gemini, temperature=0). All major providers support temperature=0 inference. **Confidence A.**

---

## 3. Research Findings

### Finding 1: SHAP/LIME are not appropriate as the primary explainability mechanism for human investment decisions (Confidence A)

Post-hoc explanation methods are designed to explain ML model outputs, not human decisions. Investment decisions involve qualitative reasoning, temporal context, and multi-source information synthesis that cannot be captured by feature attribution methods. SHAP/LIME may have a role in explaining specific quantitative model outputs within a broader decision governance framework, but they are not the core explainability architecture for ChronoSentiment.

### Finding 2: The recommended architecture is structured templates + deterministic LLM + RAG over PIT data (Confidence B)

The combination of structured decision templates (capturing the human decision rationale), deterministic LLM generation (temperature=0, version-pinned), and RAG over point-in-time data (providing the information environment at decision time) is the appropriate explainability architecture for ChronoSentiment. This architecture is: human-readable, auditable, reproducible, grounded in evidence, and consistent with regulatory requirements.

### Finding 3: Decision provenance is a first-class architectural requirement (Confidence B)

Decision provenance — the complete record of data, models, and processes that contributed to a decision — must be designed into ChronoSentiment's architecture from the beginning, not added as an afterthought. It is the technical implementation of EU AI Act Article 12 record-keeping requirements and the CFA Institute's emerging governance standard for AI-assisted investment decisions.

### Finding 4: Deterministic LLM inference is non-negotiable for audit-grade explainability (Confidence A)

Temperature=0 inference with a version-pinned LLM is required for audit-grade explainability. Non-deterministic inference breaks the audit trail and creates regulatory compliance risk. This is a hard architectural constraint, not a preference.

### Finding 5: Model cards for LLM components are an emerging regulatory expectation (Confidence B)

NIST AI RMF and Gartner research identify model cards as an emerging standard for AI transparency in regulated industries. ChronoSentiment should maintain model cards for all LLM components used in its explainability pipeline, documenting version, training data, capabilities, and limitations.

---

## 4. Recommended Explainability Architecture

### 4.1 Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                    DECISION CAPTURE LAYER                    │
│  Structured template: thesis, basis, risk, conviction,      │
│  execution intent, review trigger                           │
└─────────────────────────┬───────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────┐
│                  POINT-IN-TIME DATA LAYER                   │
│  Market data (Polygon/Databento), fundamentals (Sharadar),  │
│  filings (EDGAR), macro (FRED), news (TBD)                  │
│  All data accessed as-of decision timestamp                 │
└─────────────────────────┬───────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────┐
│                    RETRIEVAL LAYER (RAG)                    │
│  Semantic search over PIT data corpus                       │
│  Returns: relevant research, data, filings, news            │
│  All retrieved documents timestamped and attributed         │
└─────────────────────────┬───────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────┐
│              DETERMINISTIC GENERATION LAYER                 │
│  LLM: temperature=0, version-pinned                         │
│  Input: decision template + retrieved PIT context           │
│  Output: natural-language explanation with source citations │
└─────────────────────────┬───────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────┐
│                  PROVENANCE AND STORAGE LAYER               │
│  Immutable storage of: explanation, input context hash,     │
│  model version, retrieval results, generation timestamp     │
│  Explanation versioning on model update                     │
└─────────────────────────────────────────────────────────────┘
```

### 4.2 Explanation Output Format

A ChronoSentiment explanation should include:

1. **Decision summary:** One-paragraph natural-language summary of the decision and its rationale.
2. **Information basis:** Structured list of data sources, research, and analysis that informed the decision, with PIT timestamps.
3. **Key assumptions:** The critical assumptions underlying the investment thesis, with confidence levels.
4. **Risk factors:** Key risks identified at decision time, with assessed probability and impact.
5. **Execution intent vs actual:** Comparison of intended execution with actual execution (from OMS/EMS data).
6. **Provenance block:** Model version, retrieval system version, data sources, generation timestamp.

### 4.3 Model Card Requirements

ChronoSentiment should maintain model cards for each LLM version used in the explanation pipeline, documenting:
- Model identifier and version
- Training data cut-off date
- Intended use and limitations for financial explanation generation
- Performance characteristics on financial explanation benchmarks
- Known failure modes and edge cases
- Version history and change log

---

## 5. Implications

**5.1 The explainability architecture is a core product differentiator.** The combination of structured templates + deterministic LLM + RAG over PIT data is not available in any current vendor (CS-R-002). This architecture is the technical basis for ChronoSentiment's category creation claim.

**5.2 PIT data infrastructure is a prerequisite for explainability.** The RAG layer requires access to data as it existed at the time of the original decision. Without PIT data infrastructure (CS-R-006, CS-R-008), the explainability architecture cannot function. PIT data is not an optional feature — it is a prerequisite.

**5.3 Determinism is a hard constraint that affects LLM provider selection.** Not all LLM providers support deterministic inference in production. Provider selection must prioritise deterministic inference support, version pinning, and long-term model availability. Model deprecation by LLM providers is a significant operational risk.

**5.4 Decision provenance must be designed in from the beginning.** Retrofitting decision provenance into an existing architecture is significantly more difficult than designing it in from the start. The M-series architecture must treat provenance as a first-class requirement.

**5.5 Explanation quality validation is a Phase 1B requirement.** The explainability architecture described here is a Confidence B–D design. Phase 1B customer validation must include testing of explanation output quality with actual investment professionals. If explanations are not trusted by the target audience, the architecture requires revision.

---

## 6. Recommendations

**Recommendation 1: Adopt structured templates + deterministic LLM + RAG as the core explainability architecture.**
This architecture is the recommended approach based on the evidence in this document. It should be the basis for M-series architecture design. *Priority: High. M-series architecture.*

**Recommendation 2: Reject SHAP/LIME as the primary explainability mechanism.**
SHAP and LIME are not appropriate for human investment decision explainability. They may be used for specific quantitative model outputs within the platform, but should not be the core explainability architecture. *Priority: High. Architecture decision.*

**Recommendation 3: Implement decision provenance as a first-class architectural requirement.**
Design the provenance layer into the M-series architecture from the beginning. Every explanation must include a complete provenance block: model version, retrieval results, data sources, generation timestamp, and input context hash. *Priority: High. M-series architecture.*

**Recommendation 4: Validate explanation output quality in Phase 1B.**
Develop prototype explanation outputs for 3–5 representative investment decisions and test them with Phase 1B prospects. Measure: comprehensibility, trust, regulatory adequacy, and willingness to use. Revise the architecture based on feedback. *Priority: High. Phase 1B.*

**Recommendation 5: Establish LLM provider selection criteria before M-series architecture.**
Select LLM providers based on: deterministic inference support, version pinning, long-term model availability, financial domain performance, and data privacy terms. Develop a provider evaluation framework before committing to a specific LLM in the M-series architecture. *Priority: Medium. M-series preparation.*

**Recommendation 6: Maintain model cards for all LLM components.**
Implement model card documentation for all LLM versions used in the explanation pipeline. Update model cards on each model version change. Store model cards in the same immutable storage as explanation outputs. *Priority: Medium. Phase 2.*

---

## 7. Key Risks

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|-----------|
| LLM provider deprecates version-pinned model | Medium | High | Multi-provider strategy; explanation versioning on model change |
| Explanation quality not trusted by investment professionals | Medium | High | Phase 1B validation; iterative refinement |
| PIT data gaps create incomplete RAG context | Medium | Medium | Canonical data stack (CS-R-006); gap documentation |
| Deterministic inference not available at required scale | Low | High | Provider evaluation; fallback to near-deterministic |
| Regulatory guidance requires specific explainability method | Low | Medium | Monitor EU AI Office guidance; architecture flexibility |

---

## Evidence Quality

| Source Type | Examples | Confidence |
|-------------|---------|-----------|
| Academic publications | SHAP (Lundberg/Lee 2017), LIME (Ribeiro et al. 2016), RAG (Lewis et al. 2020), Model Cards (Mitchell et al. 2019) | A |
| Professional body research | CFA Institute AI Pioneers 2026 | B |
| Regulatory text | EU AI Act Art. 12, 13, 14; NIST AI RMF | A |
| Analyst research | Gartner Hype Cycle for AI Governance 2025 | B |
| Architecture design | Structured templates + LLM + RAG recommendation | D |

---

## Evidence Classification

**Published evidence:** SHAP and LIME academic papers, RAG paper (Lewis et al. 2020), Model Cards paper (Mitchell et al. 2019), EU AI Act text, NIST AI RMF, CFA Institute AI Pioneers 2026, Gartner Hype Cycle for AI Governance 2025.

**Derived findings:** SHAP/LIME rejection for human decision explainability derived from analysis of method design vs use case requirements; determinism requirement derived from regulatory text and audit requirements; provenance requirement derived from EU AI Act Article 12 and CFA Institute guidance.

**Strategic interpretation (Confidence D):** Recommended explainability architecture (structured templates + deterministic LLM + RAG); decision provenance as first-class requirement; model card implementation approach. These require validation in Phase 1B and M-series architecture review before adoption as the basis for implementation.

---

*CS-R-007 v2.0 | ChronoSentiment Research Series | Evidence cut-off: July 2026*
*Supersedes CS-R-007 v1.0. v1.0 retained as historical record.*