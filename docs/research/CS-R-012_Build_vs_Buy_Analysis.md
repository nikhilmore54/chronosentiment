# CS-R-012 — Build vs Buy Analysis
## ChronoSentiment Research Series | v1.0

---

## Document Governance

| Field | Value |
|-------|-------|
| Document Status | **Research Baseline v1.0** |
| Evidence Version | v1.0 |
| Research Date | July 2026 |
| Evidence Cut-off Date | July 2026 |
| Next Review | January 2027 or upon material change in technology landscape or customer feedback |
| Owner | ChronoSentiment Programme |
| Review Trigger | Phase 1B customer validation results; material change in open-source tooling for decision governance; new vendor entering the decision governance category |

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
| CS-R-002 Competitive Landscape v2.0 | No vendor currently provides the integrated capability — confirms build requirement |
| CS-R-006 Data Landscape v2.0 | Data vendor selection is a buy decision within the build strategy |
| CS-R-007 Explainability Research v2.0 | Explainability layer — build vs buy options assessed |
| CS-R-008 Point-in-Time Architecture v2.0 | PIT architecture — build vs buy options assessed |
| CS-R-013 Technology Readiness Assessment | Technology maturity informs build vs buy risk assessment |

**Feeds into:** M-series architecture decisions, engineering roadmap, PRD v1.0 (technical strategy)

---

## Research Limitations

This document analyses the build vs buy decision for ChronoSentiment's core technical components based on publicly available information about technology options and vendor capabilities. It does not establish:

- The actual engineering effort required to build each component
- The total cost of ownership for build vs buy options at ChronoSentiment's specific scale
- Integration complexity with ChronoSentiment's existing codebase
- Vendor pricing for enterprise contracts (where not publicly available)

These questions require proof-of-concept implementation and vendor conversations. This document provides the framework for the decision, not the final answer.

---

## 1. Purpose and Scope

This document analyses the build vs buy decision for each major technical component of ChronoSentiment. It covers: point-in-time data infrastructure, explainability and NLP layer, decision capture and storage, execution validation, and the application layer.

**Framework:** For each component, the analysis assesses: (1) whether a buy option exists that meets ChronoSentiment's requirements, (2) the cost and risk of building vs buying, and (3) the strategic implications of each choice (differentiation, lock-in, maintenance burden).

**Central finding:** ChronoSentiment's core differentiation — the integration of decision capture, temporal replay, and natural-language explainability — cannot be bought. It must be built. However, the underlying infrastructure components (data storage, query engine, LLM API, data vendors) should be bought or assembled from open-source components rather than built from scratch.

---

## 2. Component Analysis

### 2.1 Point-in-Time Data Infrastructure

**Requirement:** Store and query financial data with time-travel capability — reconstruct the data state at any historical moment.

**Buy options:**
- **Snowflake Time Travel:** Snowflake provides time-travel queries up to 90 days (Enterprise tier). Not suitable for ChronoSentiment's requirement of indefinite historical reconstruction. **Confidence A.**
- **Databricks Delta Lake:** Delta Lake provides time-travel queries with configurable retention. Suitable for production use but requires Databricks platform dependency. **Confidence A.**
- **Commercial PIT data vendors:** Sharadar (Nasdaq Data Link), EDGAR, FRED provide as-reported data for specific data types but do not provide a general-purpose PIT query infrastructure. **Confidence A.**

**Build options:**
- **Apache Iceberg + DuckDB (recommended):** Open-source, no vendor lock-in, indefinite snapshot retention, broad ecosystem support. Build cost: engineering effort to implement and operate. See CS-R-008 for detailed analysis. **Confidence B.**
- **Append-only Parquet + DuckDB (MVP):** Simpler, lower-risk MVP approach. Sufficient for Phase 1B validation. **Confidence B.**

**Decision: Build (using open-source components).** No buy option provides indefinite PIT reconstruction without vendor lock-in. The open-source stack (Iceberg + DuckDB) is mature and well-supported. **Confidence B.**

**Strategic implication:** PIT infrastructure is a core differentiator. Building it on open-source components avoids vendor lock-in while maintaining control over the capability.

### 2.2 Explainability and NLP Layer

**Requirement:** Generate natural-language explanations of investment decisions that are: accurate, auditable, reproducible, and appropriate for regulatory and client audiences.

**Buy options:**
- **OpenAI GPT-4o API:** High-quality NL generation. Suitable for explanation drafting. Limitations: model updates change outputs (not reproducible), data sent to OpenAI (data privacy risk), cost at scale. **Confidence A.**
- **Anthropic Claude API:** High-quality NL generation with strong instruction-following. Similar limitations to GPT-4o. **Confidence A.**
- **Google Gemini API:** High-quality NL generation. Similar limitations. **Confidence A.**
- **Self-hosted open-source LLMs (Llama 3, Mistral, Qwen):** Reproducible (version-pinned), data privacy preserved, no per-token cost at scale. Limitations: requires GPU infrastructure, lower quality than frontier models for complex reasoning. **Confidence B.**

**Build options:**
- **Structured template + LLM (recommended):** Build a structured explanation template that captures the decision inputs, then use a version-pinned LLM (temperature=0) to generate the natural-language explanation. The template ensures reproducibility; the LLM provides fluency. See CS-R-007 for detailed analysis. **Confidence B.**
- **Pure template (no LLM):** Build structured templates that generate explanations from structured data without LLM. Reproducible and auditable but less fluent. Suitable for MVP. **Confidence B.**

**Decision: Build (structured template) + Buy (LLM API for generation).** The explanation architecture must be built — no vendor provides investment-decision-specific explanation generation. The LLM API is a commodity component that should be bought. For production, a version-pinned self-hosted LLM is preferred for reproducibility and data privacy. **Confidence B.**

**Strategic implication:** The explanation architecture (template + structured inputs) is a core differentiator. The LLM is a commodity. ChronoSentiment should not be dependent on a specific LLM provider.

### 2.3 Decision Capture and Storage

**Requirement:** Capture structured investment decision records at the point of decision, with: decision rationale, information environment snapshot, AI tool attribution, conviction level, and execution intent.

**Buy options:**
- **Research management systems (AlphaSense, FactSet):** Capture research notes and investment memos but do not provide structured decision records with AI attribution or execution intent. **Confidence A.**
- **Note-taking tools (Notion, Confluence):** Flexible but unstructured. Cannot enforce decision record schema or link to PIT data. **Confidence A.**
- **CRM/workflow tools (Salesforce, HubSpot):** Designed for sales workflows, not investment decisions. Significant customisation required. **Confidence B.**
- **GRC platforms (ServiceNow, MetricStream):** Designed for enterprise governance, not investment decisions. Significant customisation required and not designed for investment workflow integration. **Confidence B.**

**Build options:**
- **Custom decision capture application:** Build a lightweight application (web + API) that presents a structured decision template, captures the decision record, and stores it in the PIT database. **Confidence B.**
- **Integration layer:** Build integrations with existing tools (email, Slack, research platforms) to capture decision signals and prompt structured capture. **Confidence B.**

**Decision: Build.** No buy option provides investment-decision-specific structured capture with AI attribution and PIT data linkage. This is a core product capability that must be built. **Confidence A.**

**Strategic implication:** Decision capture is the primary user-facing product. It must be designed for investment workflow integration and minimal friction. This is where ChronoSentiment's UX investment should be concentrated.

### 2.4 Execution Validation

**Requirement:** Compare actual execution (from OMS/EMS/prime broker) against original decision intent, identifying divergences and generating explanations.

**Buy options:**
- **TCA vendors (ITG/Virtu, Abel Noser, Bloomberg TCA):** Measure execution quality against market benchmarks. Do not validate execution against decision intent. **Confidence A.**
- **OMS/EMS vendors (Enfusion, Allvue, Charles River):** Provide execution records but do not link to decision intent records. **Confidence A.**
- **No buy option exists** for execution-vs-intent validation. **Confidence A.**

**Build options:**
- **Execution validation engine:** Build a component that ingests execution records (from OMS/EMS via API or file export), compares against decision intent records, and generates divergence reports. **Confidence B.**
- **Integration with existing TCA:** Build a layer on top of existing TCA that adds decision-intent context to TCA analysis. **Confidence B.**

**Decision: Build.** No buy option exists. Execution validation is a core differentiator. **Confidence A.**

**Strategic implication:** Execution validation requires OMS/EMS integration. This is a Phase 2+ capability — the integration complexity is significant and should not be in the MVP scope.

### 2.5 Market and Financial Data

**Requirement:** Historical market data, fundamental data, and alternative data with point-in-time availability for decision replay.

**Buy options (recommended):**
- **Polygon.io:** Real-time and historical market data (equities, options, forex, crypto). REST and WebSocket APIs. Reasonable pricing for mid-market use. **Confidence A.**
- **Sharadar (Nasdaq Data Link):** As-reported fundamental data for US equities. Point-in-time available. Essential for fundamental decision replay. **Confidence A.**
- **EDGAR (SEC):** Free, authoritative source for US company filings. As-filed data available. **Confidence A.**
- **FRED (Federal Reserve):** Free, authoritative source for US macroeconomic data with vintage releases. **Confidence A.**
- **Databento:** High-quality historical market microstructure data. Suitable for execution analysis. **Confidence B.**

**Build options:** Building proprietary market data infrastructure is not appropriate for ChronoSentiment at any stage. Data acquisition and normalisation is a commodity function. **Confidence A.**

**Decision: Buy.** All market and financial data should be sourced from established vendors. See CS-R-006 for detailed data landscape analysis. **Confidence A.**

**Strategic implication:** Data vendor selection affects cost, coverage, and PIT capability. The canonical stack (Polygon + Sharadar + EDGAR + FRED) provides adequate coverage for MVP at approximately US$3,000–5,000/year.

### 2.6 Application Layer and Infrastructure

**Requirement:** Web application, API, authentication, deployment infrastructure, and monitoring.

**Buy/use options (recommended):**
- **Cloud infrastructure:** AWS, GCP, or Azure. Standard choice; no build required. **Confidence A.**
- **Authentication:** Auth0, Clerk, or AWS Cognito. Standard choice; no build required. **Confidence A.**
- **Application framework:** FastAPI (Python) or Axum (Rust) for API; React or Next.js for frontend. Open-source; no cost. **Confidence A.**
- **Monitoring:** Datadog, Grafana Cloud, or AWS CloudWatch. Standard choice. **Confidence B.**
- **Database:** PostgreSQL for structured data; S3 + Iceberg for PIT data. Standard choices. **Confidence A.**

**Decision: Buy/use standard components.** Application infrastructure is a commodity. No build required beyond standard software engineering. **Confidence A.**

---

## 3. Build vs Buy Summary

| Component | Decision | Rationale | Phase |
|-----------|---------|-----------|-------|
| PIT data infrastructure | **Build** (open-source) | No buy option with indefinite retention and no lock-in | MVP |
| Explainability architecture | **Build** (template + LLM API) | No buy option for investment-specific explanation | MVP |
| Decision capture application | **Build** | No buy option with investment-specific schema and AI attribution | MVP |
| Execution validation engine | **Build** | No buy option exists | Phase 2+ |
| Market and financial data | **Buy** | Commodity; established vendors with PIT capability | MVP |
| LLM API (for NL generation) | **Buy** | Commodity; multiple providers; version-pin for reproducibility | MVP |
| Application infrastructure | **Buy/use** | Commodity; standard cloud and open-source components | MVP |
| OMS/EMS integration | **Buy** (integration) | Standard API integration with existing systems | Phase 2+ |

---

## 4. Research Findings

### Finding 1: ChronoSentiment's core differentiation cannot be bought (Confidence A)

The integrated capability — decision capture + temporal replay + natural-language explainability — does not exist as a buyable product. It must be built. This is confirmed by CS-R-002's competitive analysis. The build requirement is not a risk; it is the basis for ChronoSentiment's competitive moat.

### Finding 2: The underlying infrastructure components should be bought or assembled from open-source (Confidence B)

PIT data infrastructure (Iceberg + DuckDB), LLM APIs, market data, and application infrastructure are all available as commodity components. Building these from scratch would be wasteful and would not create differentiation. The build effort should be concentrated on the integration and application layer.

### Finding 3: Execution validation is a Phase 2+ capability due to OMS/EMS integration complexity (Confidence B)

Execution validation requires integration with OMS/EMS systems, which have complex, proprietary APIs and significant integration effort. This capability should not be in the MVP scope. The MVP should focus on decision capture and temporal replay; execution validation should be a Phase 2 feature.

### Finding 4: LLM provider independence is a strategic requirement (Confidence B)

ChronoSentiment's explainability layer should not be dependent on a single LLM provider. The structured template architecture (CS-R-007) enables LLM provider independence: the template captures the structured inputs, and any version-pinned LLM can generate the natural-language explanation. This reduces vendor risk and enables self-hosted deployment for data-sensitive customers.

### Finding 5: The canonical data stack provides adequate MVP coverage at low cost (Confidence B)

The combination of Polygon.io + Sharadar + EDGAR + FRED provides adequate market and fundamental data coverage for MVP at approximately US$3,000–5,000/year. This is a manageable cost for a pre-revenue product and does not require enterprise data contracts.

---

## 5. Implications

**5.1 Engineering effort should be concentrated on the integration layer.** The highest-value engineering work is building the integration between: decision capture → PIT data → explainability generation → audit trail. The underlying components (Iceberg, DuckDB, LLM API, Polygon) are commodity. The integration is the product.

**5.2 MVP scope should be tightly constrained.** The MVP should include: decision capture, PIT data storage (Parquet + DuckDB), and basic explainability (structured template). Execution validation, OMS/EMS integration, and advanced analytics are Phase 2+ features.

**5.3 Open-source infrastructure reduces vendor risk.** Building on Apache Iceberg, DuckDB, and open-source LLMs (for production) reduces vendor lock-in and enables self-hosted deployment for data-sensitive customers (a likely requirement for institutional asset managers).

**5.4 Data vendor selection is a near-term decision.** The canonical data stack should be selected and contracted before MVP development begins. Data vendor APIs and data quality will affect the MVP architecture.

---

## 6. Recommendations

**Recommendation 1: Adopt the canonical data stack (Polygon + Sharadar + EDGAR + FRED) for MVP.**
This stack provides adequate coverage at low cost and is well-documented. Contract these vendors before MVP development begins. *Priority: High. Pre-MVP.*

**Recommendation 2: Build the MVP on Parquet + DuckDB, migrate to Iceberg for production.**
The MVP PIT infrastructure should use append-only Parquet + DuckDB for simplicity. Migrate to Apache Iceberg for production when data volumes and query complexity require it. *Priority: High. MVP architecture.*

**Recommendation 3: Design the explainability layer for LLM provider independence.**
The structured template architecture should be implemented from the beginning. Do not build tight coupling to a specific LLM provider. Use version-pinned API calls with temperature=0 for reproducibility. *Priority: High. MVP architecture.*

**Recommendation 4: Defer execution validation to Phase 2.**
OMS/EMS integration is complex and not required for Phase 1B validation. Focus MVP engineering effort on decision capture and temporal replay. *Priority: High. Scope management.*

**Recommendation 5: Evaluate self-hosted LLM options before Phase 2.**
For production deployment with institutional asset managers, data privacy requirements may preclude sending decision data to external LLM APIs. Evaluate self-hosted options (Llama 3, Mistral) before Phase 2 to understand the quality/cost/privacy trade-off. *Priority: Medium. Phase 2 planning.*

---

## Evidence Quality

| Source Type | Examples | Confidence |
|-------------|---------|-----------|
| Technology documentation | Iceberg, DuckDB, Snowflake, Delta Lake docs | A |
| Vendor capabilities | AlphaSense, FactSet, TCA vendors, OMS/EMS vendors | A |
| Data vendor capabilities | Polygon, Sharadar, EDGAR, FRED, Databento | A |
| Build vs buy framework | Standard software engineering practice | B |
| Strategic implications | Interpretation of technology and competitive analysis | D |

---

## Evidence Classification

**Published evidence:** Technology documentation for all assessed components; vendor product documentation; data vendor capabilities and pricing (where public).

**Derived findings:** Build vs buy decisions derived from capability gap analysis (CS-R-002) and technology assessment (CS-R-008); MVP scope derived from integration complexity assessment.

**Strategic interpretation (Confidence D):** LLM provider independence as strategic requirement; execution validation as Phase 2+ scope; open-source infrastructure preference; canonical data stack recommendation. These require engineering validation and Phase 1B customer feedback before adoption as final architecture decisions.

---

*CS-R-012 v1.0 | ChronoSentiment Research Series | Evidence cut-off: July 2026*