# CS-R-006 — Data Landscape
## ChronoSentiment Research Series | v2.0

---

## Document Governance

| Field | Value |
|-------|-------|
| Document Status | **Research Baseline v2.0** |
| Evidence Version | v2.0 |
| Research Date | July 2026 |
| Evidence Cut-off Date | July 2026 |
| Next Review | January 2027 or upon material data provider change |
| Owner | ChronoSentiment Programme |
| Review Trigger | Material pricing change by Polygon.io, Databento, or Nasdaq Data Link; new point-in-time data provider entering market; EDGAR API changes |

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
| CS-R-008 Point-in-Time Architecture v2.0 | Data providers feed the point-in-time architecture; provider selection affects architecture design |
| CS-R-003 Customer Problem Evidence v2.0 | Information environment reconstruction (Problem 2) requires point-in-time data |
| CS-R-007 Explainability Research v2.0 | Explainability requires access to the data that informed the original decision |
| CS-R-012 Build vs Buy Analysis | Data provider costs are a significant component of build economics |
| CS-R-013 Technology Readiness Assessment | Data provider API maturity affects implementation timeline |

**Feeds into:** PRD v1.0 (data architecture), M-series architecture (data layer design), Phase 1B (data cost validation)

---

## 1. Purpose and Scope

This document maps the data provider landscape relevant to ChronoSentiment's point-in-time data requirements. ChronoSentiment requires data that can be accessed as it existed at a specific historical point in time — not as it exists today. This is a more demanding requirement than standard financial data access and significantly constrains the provider landscape.

**Core data requirement:** Point-in-time (PIT) access to market data, fundamental data, macroeconomic data, and alternative data — meaning the ability to reconstruct the information environment as it existed at the moment of an investment decision.

**Central finding:** A canonical data stack of Polygon.io (market data) + Sharadar/Nasdaq Data Link (as-reported fundamentals) + EDGAR (SEC filings) + FRED (macro vintage data) provides adequate PIT coverage for most investment management use cases at approximately US$3,000–US$5,000/yr. Databento is an emerging alternative for institutional-grade market data with superior PIT semantics. The canonical stack is sufficient for MVP; Databento is the recommended upgrade path for production.

---

## 2. Evidence

### 2.1 Market Data Providers

**Polygon.io** — REST and WebSocket API for US equities, options, forex, and crypto market data. Provides historical OHLCV data, trades, quotes, and corporate actions. PIT semantics: historical data is stored as-of the original timestamp; corporate action adjustments are applied separately, allowing access to unadjusted historical prices. Pricing: US$29–US$199/month for individual plans; enterprise pricing available. **Confidence A.**

**Databento** — Institutional-grade market data API with explicit point-in-time semantics. Databento stores data with nanosecond-precision timestamps and provides access to historical data as it existed at any point in time, including order book snapshots, trade data, and reference data. Designed specifically for quantitative research and backtesting use cases where PIT accuracy is critical. Pricing: usage-based, approximately US$0.10–US$1.00 per GB of historical data depending on data type and resolution. **Confidence A.**

**Interactive Brokers Historical Data API** — Historical market data available through the IBKR API. Limited PIT semantics; data is adjusted for corporate actions by default. Not recommended for PIT use cases. **Confidence A.**

**Bloomberg Data License** — Institutional market data with comprehensive coverage. Supports PIT access for historical data. Pricing: US$10,000–US$50,000+/yr depending on data scope. Not appropriate for MVP due to cost; potential upgrade path for large institutional customers. **Confidence B.**

**Refinitiv/LSEG Tick History** — Institutional tick data with PIT semantics. Similar positioning to Bloomberg Data License. Not appropriate for MVP. **Confidence B.**

### 2.2 Fundamental Data Providers

**Sharadar (Nasdaq Data Link)** — As-reported fundamental data for US equities. Sharadar provides financial statement data as it was originally reported, before restatements and revisions — the gold standard for PIT fundamental data. Covers income statement, balance sheet, cash flow, and key ratios. Pricing: US$300–US$600/yr for individual access; enterprise pricing available. **Confidence A.**

**Compustat (S&P Global)** — Institutional fundamental data with as-reported and restated versions. The industry standard for academic and institutional quantitative research. Pricing: US$5,000–US$50,000+/yr depending on access tier. Not appropriate for MVP due to cost. **Confidence A.**

**Intrinio** — Financial data API with fundamental data, news, and alternative data. Limited as-reported PIT semantics. Pricing: US$50–US$500/month. **Confidence B.**

**Tiingo** — Financial data API with fundamental data and news. Limited PIT semantics. Pricing: US$10–US$50/month. **Confidence B.**

### 2.3 Macroeconomic Data Providers

**FRED (Federal Reserve Economic Data)** — Free API providing macroeconomic time series data from the Federal Reserve Bank of St. Louis. Critically, FRED provides vintage data releases — the ability to access macroeconomic data as it was released at a specific historical date, before subsequent revisions. This is essential for reconstructing the macro environment at the time of an investment decision. Pricing: Free. **Confidence A.**

**BLS (Bureau of Labor Statistics)** — Free API for US labour market data (CPI, employment, wages). Provides historical release data. Pricing: Free. **Confidence A.**

**World Bank Open Data** — Free API for global macroeconomic and development data. Limited PIT semantics for historical releases. Pricing: Free. **Confidence A.**

**Bloomberg Economics** — Institutional macroeconomic data with comprehensive global coverage. Not appropriate for MVP due to cost. **Confidence B.**

### 2.4 SEC Filing Data

**EDGAR (SEC Electronic Data Gathering, Analysis, and Retrieval)** — Free API providing access to all SEC filings (10-K, 10-Q, 8-K, proxy statements, etc.) with original filing timestamps. EDGAR is the authoritative source for as-filed SEC documents — the exact text of filings as they were submitted, with the original filing date. This is essential for reconstructing the information environment at the time of an investment decision. Pricing: Free. **Confidence A.**

**SEC XBRL API** — Structured financial data extracted from EDGAR XBRL filings. Provides machine-readable financial statement data with original filing timestamps. Pricing: Free. **Confidence A.**

**Calcbench** — XBRL data extraction and normalisation service. Provides structured financial data from SEC filings with as-reported and normalised versions. Pricing: US$100–US$500/month. **Confidence B.**

### 2.5 Alternative Data Providers

**News and sentiment data:** Multiple providers offer historical news archives with original publication timestamps (essential for PIT news reconstruction). Key providers: Refinitiv News Analytics, RavenPack, Benzinga, NewsAPI. Pricing varies widely. **Confidence B.**

**Earnings call transcripts:** Seeking Alpha, Motley Fool, and specialised providers offer historical earnings call transcripts with original publication timestamps. AlphaSense and Tegus provide transcript search and analysis. **Confidence B.**

**Analyst estimates:** Visible Alpha (S&P Global), FactSet Estimates, Bloomberg Consensus provide historical analyst estimate data. PIT semantics vary by provider. **Confidence B.**

---

## 3. Provider Decision Matrix

| Provider | Data Type | PIT Semantics | Cost (MVP) | API Quality | Licence Flexibility | Overall |
|----------|-----------|--------------|-----------|-------------|--------------------|---------| 
| **Polygon.io** | Market data | ✅ Good | US$29–199/mo | ✅ Excellent | ✅ Flexible | ⭐⭐⭐⭐ |
| **Databento** | Market data | ✅ Excellent | Usage-based | ✅ Excellent | ✅ Flexible | ⭐⭐⭐⭐⭐ |
| **Sharadar/NDL** | Fundamentals | ✅ Excellent (as-reported) | US$300–600/yr | ✅ Good | ✅ Flexible | ⭐⭐⭐⭐⭐ |
| **EDGAR** | SEC filings | ✅ Excellent (as-filed) | Free | ✅ Good | ✅ Open | ⭐⭐⭐⭐⭐ |
| **FRED** | Macro | ✅ Excellent (vintage) | Free | ✅ Good | ✅ Open | ⭐⭐⭐⭐⭐ |
| **BLS** | Labour/CPI | ✅ Good | Free | ⚠️ Adequate | ✅ Open | ⭐⭐⭐⭐ |
| **Bloomberg DL** | All types | ✅ Excellent | US$10K+/yr | ✅ Excellent | ⚠️ Restrictive | ⭐⭐⭐ (cost) |
| **Compustat** | Fundamentals | ✅ Excellent | US$5K+/yr | ✅ Excellent | ⚠️ Restrictive | ⭐⭐⭐ (cost) |
| **Intrinio** | Fundamentals | ⚠️ Limited | US$50–500/mo | ⚠️ Adequate | ✅ Flexible | ⭐⭐⭐ |
| **Tiingo** | Market/fundamentals | ⚠️ Limited | US$10–50/mo | ⚠️ Adequate | ✅ Flexible | ⭐⭐ |

**PIT Semantics rating:** Excellent = explicit as-reported/as-filed/vintage data; Good = historical data with original timestamps; Limited = adjusted/revised data only; None = current data only.

---

## 4. Canonical Data Stack

### 4.1 MVP Stack (Phase 1 / Phase 2)

| Layer | Provider | Purpose | Annual Cost |
|-------|----------|---------|------------|
| Market data | Polygon.io (Starter) | OHLCV, trades, corporate actions | ~US$350/yr |
| Fundamental data | Sharadar (Nasdaq Data Link) | As-reported financials | ~US$500/yr |
| SEC filings | EDGAR API | As-filed documents, XBRL data | Free |
| Macro data | FRED API | Vintage macro releases | Free |
| Labour/CPI | BLS API | Historical CPI, employment releases | Free |
| **Total MVP** | | | **~US$850/yr** |

### 4.2 Production Stack (Phase 3+)

| Layer | Provider | Purpose | Annual Cost |
|-------|----------|---------|------------|
| Market data | Databento | Institutional PIT market data | ~US$2,000–5,000/yr (usage) |
| Fundamental data | Sharadar (Nasdaq Data Link) | As-reported financials | ~US$500/yr |
| SEC filings | EDGAR API | As-filed documents, XBRL data | Free |
| Macro data | FRED API | Vintage macro releases | Free |
| News/sentiment | Provider TBD | Historical news with timestamps | ~US$500–2,000/yr |
| Earnings transcripts | Provider TBD | Historical transcripts | ~US$500–1,000/yr |
| **Total Production** | | | **~US$3,500–8,000/yr** |

*Note: Production stack costs are indicative. Actual costs depend on data volume, query frequency, and negotiated enterprise pricing.*

### 4.3 Databento as the Production Market Data Upgrade

Databento is the recommended upgrade from Polygon.io for production deployments because:

1. **Explicit PIT semantics:** Databento is designed from the ground up for PIT data access. Historical data is stored with nanosecond precision and can be accessed as it existed at any historical timestamp.
2. **Institutional data quality:** Databento sources data from primary exchanges and venues, not consolidated feeds. This provides higher accuracy for historical reconstruction.
3. **Usage-based pricing:** Databento's usage-based pricing model aligns cost with actual data consumption, which is appropriate for a platform where replay operations are the primary data-intensive workload.
4. **API design:** Databento's API is designed for programmatic access by quantitative researchers and engineers, with comprehensive documentation and client libraries.

**Confidence A** for Databento capabilities. **Confidence B** for cost estimates at production scale.

---

## 5. Research Findings

### Finding 1: A canonical stack of four free/low-cost providers covers MVP requirements (Confidence A)

Polygon.io + Sharadar + EDGAR + FRED provides adequate PIT coverage for US equity investment management use cases at approximately US$850/yr. This is sufficient for MVP development and Phase 1B customer validation. The low cost of the MVP stack reduces the financial risk of Phase 1 development.

### Finding 2: Databento is the superior production market data provider for PIT use cases (Confidence A)

Databento's explicit PIT semantics, institutional data quality, and usage-based pricing make it the recommended production market data provider. The upgrade from Polygon.io to Databento should be planned for Phase 3 (production deployment) rather than MVP.

### Finding 3: EDGAR and FRED are underappreciated PIT data assets (Confidence A)

EDGAR (as-filed SEC documents) and FRED (vintage macro releases) are free, high-quality PIT data sources that are not widely used in decision governance contexts. Their inclusion in the canonical stack provides significant PIT coverage at zero marginal cost.

### Finding 4: Institutional data providers (Bloomberg, Compustat) are not appropriate for MVP (Confidence A)

Bloomberg Data License and Compustat provide excellent PIT data but at costs (US$5,000–US$50,000+/yr) that are not appropriate for MVP development. These providers are the upgrade path for large institutional customers who require comprehensive global coverage.

### Finding 5: News and alternative data PIT coverage is the primary gap in the canonical stack (Confidence B)

The canonical MVP stack does not include news, sentiment, or earnings transcript data with reliable PIT semantics. This is a gap for investment decisions that were significantly influenced by news or analyst commentary. Addressing this gap requires either a dedicated news data provider or a custom news archiving solution. This is a Phase 2 or Phase 3 consideration.

---

## 6. Implications

**6.1 The MVP data stack is low-cost and low-risk.** The canonical MVP stack (Polygon.io + Sharadar + EDGAR + FRED) costs approximately US$850/yr and provides adequate PIT coverage for US equity investment management. This low cost reduces the financial risk of Phase 1 development and allows ChronoSentiment to validate the product concept before committing to higher-cost data infrastructure.

**6.2 Databento is a strategic data partner, not just a vendor.** Databento's explicit PIT semantics and institutional positioning make it a natural strategic partner for ChronoSentiment. An early partnership or integration agreement with Databento could provide competitive differentiation and preferential pricing.

**6.3 Data provider selection affects architecture design.** The choice of data providers has significant implications for the point-in-time architecture (CS-R-008). Databento's API design, data formats, and PIT semantics should be incorporated into the architecture design from Phase 2 onwards.

**6.4 News and alternative data is the primary coverage gap.** The canonical stack does not provide reliable PIT coverage for news and alternative data. This gap should be addressed in Phase 2 or Phase 3 based on customer feedback from Phase 1B about the importance of news data in their decision-making process.

**6.5 Data licensing terms must be reviewed before commercial deployment.** All data providers have licensing terms that restrict commercial use of their data. Before Phase 2 commercial deployment, legal review of data licensing terms is required to ensure that ChronoSentiment's use of provider data is compliant with licence terms.

---

## 7. Recommendations

**Recommendation 1: Adopt the canonical MVP stack for Phase 1 development.**
Use Polygon.io + Sharadar + EDGAR + FRED for Phase 1 development and Phase 1B customer validation. This stack provides adequate PIT coverage at minimal cost. *Priority: High. Phase 1.*

**Recommendation 2: Plan Databento integration for Phase 2.**
Design the Phase 2 data architecture to support Databento as the primary market data provider. Engage with Databento's enterprise team to understand pricing, API capabilities, and partnership opportunities. *Priority: Medium. Phase 2 preparation.*

**Recommendation 3: Validate news data requirements in Phase 1B.**
Phase 1B customer interviews should include questions about the role of news and alternative data in investment decisions. If news data is critical for PIT reconstruction, prioritise news data provider selection in Phase 2. *Priority: Medium. Phase 1B.*

**Recommendation 4: Conduct legal review of data licensing terms before Phase 2.**
Engage legal counsel to review data licensing terms for all providers in the canonical stack before Phase 2 commercial deployment. Identify any restrictions on commercial use, redistribution, or derived data products. *Priority: High. Before Phase 2.*

**Recommendation 5: Monitor Databento pricing as usage scales.**
Databento's usage-based pricing is cost-effective at low volumes but may become significant at production scale. Monitor data consumption and negotiate enterprise pricing before production deployment. *Priority: Medium. Phase 3.*

---

## 8. Key Risks

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|-----------|
| Polygon.io pricing increases materially | Low | Medium | Databento as fallback; multi-provider architecture |
| Sharadar data quality issues for specific use cases | Low | Medium | Validate against EDGAR XBRL for critical data points |
| EDGAR API rate limits affect production performance | Medium | Medium | Implement caching layer; request rate limit increase |
| Data licensing terms restrict commercial use | Medium | High | Legal review before Phase 2; negotiate commercial licences |
| News data PIT coverage gap affects product value | Medium | Medium | Phase 1B validation; Phase 2 news provider selection |

---

## Evidence Quality

| Source Type | Examples | Confidence |
|-------------|---------|-----------|
| Provider documentation | Polygon.io, Databento, EDGAR, FRED, Sharadar docs | A |
| Provider pricing pages | Polygon.io, Databento, Sharadar public pricing | A–B |
| Industry knowledge | Bloomberg, Compustat pricing estimates | B–C |
| PIT semantics assessment | Structured analysis of provider documentation | B |
| Canonical stack recommendation | Strategic interpretation of provider analysis | D |

---

## Evidence Classification

**Published evidence:** Provider API documentation, public pricing pages, EDGAR and FRED API specifications, Databento technical documentation, Sharadar data dictionary.

**Derived findings:** Provider decision matrix derived from published documentation and pricing; canonical stack cost estimates derived from public pricing; PIT semantics ratings derived from provider documentation analysis.

**Strategic interpretation (Confidence D):** Canonical stack recommendation; Databento as strategic partner framing; MVP vs production stack split. These require validation against actual development experience and Phase 1B customer data requirements before adoption as the basis for M-series architecture decisions.

---

*CS-R-006 v2.0 | ChronoSentiment Research Series | Evidence cut-off: July 2026*
*Supersedes CS-R-006 v1.0. v1.0 retained as historical record.*