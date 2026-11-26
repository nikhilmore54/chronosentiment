# CS-R-013 — Technology Readiness Assessment
## ChronoSentiment Research Series | v1.0

---

## Document Governance

| Field | Value |
|-------|-------|
| Document Status | **Research Baseline v1.0** |
| Evidence Version | v1.0 |
| Research Date | July 2026 |
| Evidence Cut-off Date | July 2026 |
| Next Review | January 2027 or upon material technology development |
| Owner | ChronoSentiment Programme |
| Review Trigger | Major release of Apache Iceberg, DuckDB, or primary LLM APIs; material change in open-source LLM quality; new PIT data technology emerges |

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
| CS-R-006 Data Landscape v2.0 | Data vendor readiness assessed here |
| CS-R-007 Explainability Research v2.0 | LLM and explainability technology readiness assessed here |
| CS-R-008 Point-in-Time Architecture v2.0 | PIT technology readiness assessed here |
| CS-R-012 Build vs Buy Analysis | Technology readiness informs build vs buy risk |

**Feeds into:** M-series architecture decisions, engineering roadmap, risk register

---

## Research Limitations

This document assesses technology readiness based on publicly available documentation, benchmarks, and community adoption data. It does not establish:

- Performance characteristics at ChronoSentiment's specific data volumes and query patterns
- Integration complexity with ChronoSentiment's specific codebase and infrastructure
- Operational burden at production scale
- Actual cost at ChronoSentiment's usage patterns

These questions require proof-of-concept implementation. This document provides the technology selection framework, not a substitute for empirical testing.

---

## 1. Purpose and Scope

This document assesses the readiness of the key technologies required to build ChronoSentiment. It uses a Technology Readiness Level (TRL) framework adapted from NASA/ESA standards, applied to software technology maturity. It covers: point-in-time data infrastructure, query engines, LLM APIs, open-source LLMs, data vendor APIs, and application infrastructure.

**TRL Scale (adapted for software):**

| TRL | Definition |
|-----|-----------|
| TRL 1–3 | Research / proof of concept — not production-ready |
| TRL 4–6 | Validated in relevant environment — early production use |
| TRL 7–8 | Production-proven — widely deployed in similar contexts |
| TRL 9 | Mission-proven — dominant standard in its category |

---

## 2. Technology Assessments

### 2.1 Apache Iceberg

**TRL: 8–9** | **Confidence A**

Apache Iceberg is production-proven at scale. It is deployed by Netflix, Apple, LinkedIn, Airbnb, and hundreds of other organisations for large-scale data lake management. It is the dominant open table format for data lake architectures as of 2026, with broad ecosystem support (Spark, Flink, Trino, DuckDB, Snowflake, AWS Athena, Google BigQuery).

**Readiness for ChronoSentiment:**
- Time-travel queries: Production-ready. Widely used in financial services for historical data analysis.
- Schema evolution: Production-ready. Well-documented and tested.
- ACID transactions: Production-ready. Serialisable isolation.
- Snapshot retention: Production-ready. Configurable retention policies.
- DuckDB integration: Production-ready via iceberg extension (DuckDB 0.10+).

**Risks:** Iceberg catalog management (AWS Glue or Nessie) adds operational complexity. Iceberg metadata can become a bottleneck at very large scale (billions of files), but this is not a concern at ChronoSentiment's expected data volumes.

**Assessment: Ready for production use. No significant technology risk.**

### 2.2 DuckDB

**TRL: 8** | **Confidence A**

DuckDB is production-proven for analytical workloads. It is widely used in data science, analytics engineering, and embedded analytics applications. As of 2026, DuckDB 1.x is stable and actively maintained by DuckDB Labs with strong community support.

**Readiness for ChronoSentiment:**
- In-process execution: Production-ready. Eliminates network latency for query execution.
- Parquet support: Production-ready. Native Parquet reader with predicate pushdown.
- Iceberg support: Production-ready via extension (DuckDB 0.10+).
- Python embedding: Production-ready. Well-documented Python API.
- Rust embedding: Production-ready. Official Rust bindings available.
- Concurrent reads: Production-ready. Multiple readers supported.
- Concurrent writes: Limited. DuckDB uses a single-writer model. For ChronoSentiment's read-heavy workload, this is not a significant constraint.

**Risks:** DuckDB is not designed for high-concurrency write workloads. If ChronoSentiment requires many concurrent writers (e.g., real-time decision capture from many users simultaneously), a separate write path (PostgreSQL or Kafka) may be required.

**Assessment: Ready for production use. Single-writer limitation requires architectural consideration for high-concurrency scenarios.**

### 2.3 OpenAI GPT-4o API

**TRL: 9** | **Confidence A**

GPT-4o is the most widely deployed LLM API in enterprise applications as of 2026. It is production-proven across a wide range of NL generation tasks including document summarisation, explanation generation, and structured output.

**Readiness for ChronoSentiment:**
- Natural-language explanation generation: Production-ready. High quality for structured-to-text generation.
- Structured output (JSON mode): Production-ready. Reliable structured output with schema enforcement.
- Temperature=0 reproducibility: Production-ready. Deterministic output at temperature=0 for a given model version.
- Model versioning: Production-ready. Specific model versions (e.g., gpt-4o-2024-11-20) can be pinned.
- Data privacy: **Risk.** Decision data sent to OpenAI API. Not suitable for customers with strict data residency requirements without Azure OpenAI Service deployment.
- Cost at scale: Moderate. GPT-4o pricing is approximately US$2.50/1M input tokens, US$10/1M output tokens (as of July 2026). For ChronoSentiment's expected usage, cost is manageable at MVP scale.

**Risks:** Data privacy is the primary risk for institutional asset managers. Model deprecation requires version management. Cost at scale requires monitoring.

**Assessment: Ready for MVP use. Data privacy risk requires mitigation strategy for production deployment with institutional customers.**

### 2.4 Anthropic Claude API

**TRL: 8–9** | **Confidence A**

Claude (Claude 3.5/4 series) is production-proven for document analysis, structured output, and explanation generation. Strong instruction-following and low hallucination rate make it suitable for ChronoSentiment's explanation generation use case.

**Readiness for ChronoSentiment:**
- Natural-language explanation generation: Production-ready. Particularly strong for structured-to-text generation with complex instructions.
- Structured output: Production-ready. XML and JSON output modes available.
- Temperature=0 reproducibility: Production-ready.
- Model versioning: Production-ready. Specific model versions can be pinned.
- Data privacy: **Risk.** Same data privacy considerations as GPT-4o. AWS Bedrock deployment available for data residency requirements.

**Assessment: Ready for MVP use. Equivalent to GPT-4o for ChronoSentiment's use case. Data privacy risk same as GPT-4o.**

### 2.5 Self-Hosted Open-Source LLMs

**TRL: 6–7** | **Confidence B**

Open-source LLMs (Llama 3, Mistral, Qwen, Gemma) have improved significantly in 2024–2026. For structured-to-text generation tasks (generating NL explanations from structured decision records), open-source models at the 7B–70B parameter range are approaching frontier model quality.

**Readiness for ChronoSentiment:**
- Natural-language explanation generation: Adequate for structured-to-text tasks. Quality gap vs frontier models is narrowing.
- Structured output: Adequate with instruction tuning. Less reliable than frontier models for complex schemas.
- Reproducibility: Excellent. Self-hosted models are fully version-pinned and deterministic.
- Data privacy: Excellent. No data leaves the deployment environment.
- Infrastructure requirement: GPU infrastructure required. Minimum 1x A100 (80GB) for 70B models; 1x RTX 4090 for 7B models.
- Cost: GPU infrastructure cost ~US$2–4/hour (cloud GPU) or capital cost for on-premises.

**Risks:** Quality gap vs frontier models for complex reasoning tasks. Infrastructure management overhead. Model update cadence requires evaluation.

**Assessment: Not ready for MVP. Suitable for Phase 2+ when data privacy requirements from institutional customers justify the infrastructure investment.**

### 2.6 Polygon.io API

**TRL: 8** | **Confidence A**

Polygon.io is a production-proven market data API widely used by fintech applications, quantitative researchers, and data engineers. It provides real-time and historical data for US equities, options, forex, and crypto.

**Readiness for ChronoSentiment:**
- Historical OHLCV data: Production-ready. Extensive history (20+ years for US equities).
- Real-time data: Production-ready. WebSocket and REST APIs.
- Point-in-time availability: Adequate. Historical data is available as-of the date it was published, but Polygon does not provide as-reported data for corporate actions or fundamental data.
- API reliability: Production-ready. 99.9%+ uptime SLA on paid tiers.
- Pricing: Starter tier ~US$29/month; Stocks Starter ~US$79/month; higher tiers for options and real-time.

**Assessment: Ready for MVP use. Adequate for market data; Sharadar required for as-reported fundamental data.**

### 2.7 Sharadar (Nasdaq Data Link)

**TRL: 8** | **Confidence A**

Sharadar is the standard source for as-reported fundamental data for US equities. It is widely used by quantitative researchers and financial data engineers for point-in-time fundamental analysis.

**Readiness for ChronoSentiment:**
- As-reported fundamental data: Production-ready. Quarterly and annual data as originally reported, with restatement history.
- Point-in-time availability: Excellent. As-reported data is the primary use case.
- API: Production-ready. REST API and bulk download available.
- Pricing: ~US$150–300/month for full fundamental dataset.

**Assessment: Ready for MVP use. Essential for fundamental decision replay.**

### 2.8 PostgreSQL

**TRL: 9** | **Confidence A**

PostgreSQL is the dominant open-source relational database. Production-proven across all scales and use cases. No technology risk.

**Readiness for ChronoSentiment:**
- Decision record storage: Production-ready. Structured decision records, user data, audit logs.
- JSONB support: Production-ready. Flexible schema for decision record metadata.
- Full-text search: Production-ready. Useful for decision record search.
- Managed deployment: Production-ready. AWS RDS, GCP Cloud SQL, Supabase all provide managed PostgreSQL.

**Assessment: Ready for production use. No technology risk.**

### 2.9 React / Next.js

**TRL: 9** | **Confidence A**

React and Next.js are the dominant frontend frameworks for web applications. Production-proven across all scales. No technology risk.

**Assessment: Ready for production use. No technology risk.**

### 2.10 FastAPI (Python) / Axum (Rust)

**TRL: 8–9** | **Confidence A**

FastAPI is the dominant Python API framework for data-intensive applications. Axum is the dominant Rust web framework. Both are production-proven.

**Readiness for ChronoSentiment:**
- FastAPI: Excellent for Python-based data processing pipelines. Strong integration with pandas, polars, DuckDB.
- Axum: Excellent for high-performance API endpoints. Strong integration with Rust ecosystem (relevant given ChronoSentiment's existing Rust codebase).

**Assessment: Both ready for production use. Choice depends on team expertise and performance requirements.**

---

## 3. Technology Readiness Summary

| Technology | TRL | Readiness for MVP | Primary Risk | Phase |
|-----------|-----|------------------|-------------|-------|
| Apache Iceberg | 8–9 | ✅ Ready | Catalog management complexity | Production |
| DuckDB | 8 | ✅ Ready | Single-writer limitation | MVP + Production |
| Parquet (MVP PIT) | 9 | ✅ Ready | None | MVP |
| OpenAI GPT-4o API | 9 | ✅ Ready | Data privacy for institutional customers | MVP |
| Anthropic Claude API | 8–9 | ✅ Ready | Data privacy for institutional customers | MVP |
| Self-hosted LLMs | 6–7 | ⚠️ Not MVP-ready | Quality gap; infrastructure overhead | Phase 2+ |
| Polygon.io | 8 | ✅ Ready | No as-reported fundamental data | MVP |
| Sharadar | 8 | ✅ Ready | None | MVP |
| EDGAR | 9 | ✅ Ready | None | MVP |
| FRED | 9 | ✅ Ready | None | MVP |
| PostgreSQL | 9 | ✅ Ready | None | MVP |
| React / Next.js | 9 | ✅ Ready | None | MVP |
| FastAPI / Axum | 8–9 | ✅ Ready | None | MVP |

---

## 4. Research Findings

### Finding 1: The MVP technology stack is mature and production-ready (Confidence A)

All technologies required for the ChronoSentiment MVP (Parquet + DuckDB, GPT-4o or Claude API, Polygon + Sharadar + EDGAR + FRED, PostgreSQL, React/Next.js, FastAPI/Axum) are at TRL 8–9. There is no significant technology risk in the MVP stack.

### Finding 2: The production PIT infrastructure (Apache Iceberg) is mature and production-ready (Confidence A)

Apache Iceberg is at TRL 8–9 and is production-proven at scale in financial services contexts. The migration from MVP (Parquet) to production (Iceberg) is well-understood and low-risk.

### Finding 3: Data privacy is the primary technology risk for institutional customers (Confidence B)

The use of external LLM APIs (GPT-4o, Claude) requires sending decision data to third-party providers. Institutional asset managers may have data residency requirements that preclude this. Self-hosted LLMs are not MVP-ready but should be evaluated for Phase 2.

### Finding 4: Self-hosted LLMs are not MVP-ready but are approaching adequacy for structured-to-text tasks (Confidence B)

Open-source LLMs at the 7B–70B parameter range are approaching frontier model quality for structured-to-text generation tasks. They are not recommended for MVP due to infrastructure overhead and quality gap, but should be evaluated for Phase 2 when institutional customer data privacy requirements become a constraint.

### Finding 5: No technology risk is a significant barrier to ChronoSentiment's development (Confidence A)

The technology stack required to build ChronoSentiment is mature, well-documented, and production-proven. The primary risks are operational (data privacy, infrastructure management) rather than technological (unproven technology). This is a positive finding for the M-series investment case.

---

## 5. Implications

**5.1 Technology risk is not a significant barrier.** The MVP can be built with mature, production-proven technology. This reduces the engineering risk in the M-series investment case.

**5.2 Data privacy is the primary operational risk for institutional customers.** The LLM API data privacy issue should be addressed in the product architecture before Phase 2. Options include: Azure OpenAI Service (data residency), AWS Bedrock (data residency), or self-hosted LLMs (full data isolation).

**5.3 The MVP-to-production migration path is low-risk.** The migration from Parquet + DuckDB (MVP) to Iceberg + DuckDB (production) is well-understood and does not require rewriting the application layer.

**5.4 The existing Rust codebase is an asset.** ChronoSentiment's existing Rust codebase (coralys-scheduling, coralys-core, etc.) is compatible with the recommended technology stack. DuckDB has official Rust bindings; Axum is the recommended Rust web framework.

---

## 6. Recommendations

**Recommendation 1: Proceed with the MVP technology stack as specified.**
The MVP stack (Parquet + DuckDB + GPT-4o/Claude API + Polygon + Sharadar + PostgreSQL + React/Next.js) is mature and production-ready. No technology substitutions are required. *Priority: High. Pre-MVP.*

**Recommendation 2: Implement a data privacy mitigation strategy before Phase 2.**
Before deploying to institutional asset managers, implement one of: Azure OpenAI Service, AWS Bedrock, or self-hosted LLM. Evaluate the quality/cost/privacy trade-off in Phase 2 planning. *Priority: High. Phase 2 planning.*

**Recommendation 3: Evaluate DuckDB concurrency limits in proof-of-concept.**
DuckDB's single-writer model may be a constraint for high-concurrency decision capture scenarios. Test concurrent write performance in the proof-of-concept phase before committing to the production architecture. *Priority: Medium. Proof-of-concept.*

**Recommendation 4: Leverage the existing Rust codebase for performance-critical components.**
The existing Rust codebase provides a foundation for performance-critical components (data ingestion, PIT query engine, execution validation). Use Axum for the API layer and DuckDB Rust bindings for the query layer. *Priority: Medium. Architecture.*

---

## Evidence Quality

| Source Type | Examples | Confidence |
|-------------|---------|-----------|
| Technology documentation | Apache Iceberg, DuckDB, OpenAI, Anthropic, Polygon, Sharadar docs | A |
| Community adoption data | GitHub stars, download statistics, production deployment case studies | A–B |
| Benchmark data | DuckDB TPC-H benchmarks, LLM quality benchmarks | B |
| TRL assessments | Structured assessment of published technology maturity evidence | B |
| Strategic implications | Interpretation of technology assessment | D |

---

## Evidence Classification

**Published evidence:** Technology documentation for all assessed components; production deployment case studies; benchmark data.

**Derived findings:** TRL assessments derived from published documentation and community adoption data; data privacy risk derived from LLM API architecture analysis.

**Strategic interpretation (Confidence D):** MVP stack recommendation; Phase 2 data privacy mitigation strategy; Rust codebase leverage recommendation. These require engineering validation before adoption as final architecture decisions.

---

*CS-R-013 v1.0 | ChronoSentiment Research Series | Evidence cut-off: July 2026*