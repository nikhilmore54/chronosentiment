# CS-R-008 — Point-in-Time Architecture
## ChronoSentiment Research Series | v2.0

---

## Document Governance

| Field | Value |
|-------|-------|
| Document Status | **Research Baseline v2.0** |
| Evidence Version | v2.0 |
| Research Date | July 2026 |
| Evidence Cut-off Date | July 2026 |
| Next Review | January 2027 or upon material technology development |
| Owner | ChronoSentiment Programme |
| Review Trigger | Apache Iceberg major release; Delta Lake major release; DuckDB major release; new point-in-time query standard emerges |

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
| CS-R-004 Regulatory Landscape v2.0 | Point-in-time architecture is the technical implementation of regulatory documentation requirements |
| CS-R-006 Data Landscape v2.0 | Data vendors and formats that point-in-time architecture must ingest |
| CS-R-007 Explainability Research v2.0 | Provenance tracking depends on point-in-time data architecture |
| CS-R-012 Build vs Buy Analysis | Architecture choices inform build vs buy decisions |
| CS-R-013 Technology Readiness Assessment | Technology maturity assessment for Iceberg, Delta Lake, DuckDB |

**Feeds into:** M-series architecture decisions, PRD v1.0 (technical requirements), engineering roadmap

---

## Research Limitations

This document analyses publicly available technical documentation, benchmarks, and case studies. It does not establish:

- Performance characteristics of these technologies at ChronoSentiment's specific data volumes and query patterns
- Integration complexity with ChronoSentiment's existing data pipeline
- Operational burden of maintaining these systems at production scale
- Cost at ChronoSentiment's specific usage patterns

These questions require proof-of-concept implementation and benchmarking. The analysis here provides the foundation for technology selection, not a substitute for empirical testing.

---

## 1. Purpose and Scope

This document surveys the technical landscape for point-in-time (PIT) data architecture as applied to ChronoSentiment's core requirement: reconstructing the information environment at any historical moment. It evaluates Apache Iceberg, Delta Lake, and DuckDB as the primary technology candidates, and provides an explicit MVP vs Production architecture split.

**Central finding:** The recommended architecture uses Apache Iceberg (or Delta Lake) for storage and versioning, with DuckDB as the query engine for analytical workloads. This combination provides time-travel capability, query performance, and operational simplicity appropriate for ChronoSentiment's requirements. For MVP, a simpler append-only Parquet + DuckDB stack is sufficient and lower-risk. The full Iceberg/Delta Lake architecture is the production target.

---

## 2. The Point-in-Time Problem

### 2.1 Why Point-in-Time Data Is Hard

Most data systems are designed to answer questions about the current state of the world. Investment analysis requires answering questions about the state of the world at a specific historical moment — without contamination from information that was not available at that time.

**Data mutation:** Financial data is frequently revised. Economic indicators are restated. Corporate earnings are restated. Analyst ratings change. A system that stores only the current value of a data point cannot reconstruct what was known at a historical moment. **Confidence A.**

**Survivorship bias:** Companies that went bankrupt, were acquired, or were delisted are often removed from databases. A point-in-time system must preserve the universe of securities as it existed at each historical moment. **Confidence A.**

**Look-ahead contamination:** If a model is trained or evaluated using data that was not available at the time of the decision, its performance will be overstated. This is a pervasive problem in financial ML research (Banz 1981; Hou, Xue, Zhang 2020). **Confidence A.**

**Sentiment data decay:** News sentiment, social media sentiment, and analyst sentiment are highly time-sensitive. The sentiment environment at 9:00 AM on a given day may be materially different from the sentiment environment at 3:00 PM. A point-in-time system must preserve intraday sentiment states. **Confidence B.**

### 2.2 Existing Approaches and Their Limitations

| Approach | Description | Limitation |
|----------|-------------|------------|
| Snapshot tables | Periodic full copies of data state | Storage-intensive; limited temporal resolution; no intraday granularity |
| Slowly changing dimensions (SCD Type 2) | Track changes with effective date ranges | Complex to query; does not handle high-frequency updates well |
| Event sourcing | Append-only log of all changes | Correct approach but requires specialised query infrastructure |
| Bitemporal databases | Separate valid time and transaction time | Correct approach; limited commercial support; complex to implement |
| Ad hoc timestamp columns | Add `as_of_date` to existing tables | Fragile; does not handle schema evolution; no ACID guarantees |

Modern open table formats (Iceberg, Delta Lake) address the gap between correctness and operational simplicity. **Confidence B.**

---

## 3. Apache Iceberg

### 3.1 Overview

Apache Iceberg is an open table format for large analytic datasets. Originally developed at Netflix, open-sourced in 2018, Apache top-level project since 2020. As of 2026, it is the dominant open table format for data lake architectures. **Confidence A.**

### 3.2 Core Capabilities Relevant to ChronoSentiment

**Time travel:** Iceberg maintains a snapshot history of every table write. Queries can be issued against any historical snapshot using `AS OF` syntax or snapshot ID. This is the foundational PIT capability. **Confidence A.**

**Schema evolution:** Iceberg supports adding, dropping, renaming, and reordering columns without rewriting existing data. This is essential for a platform that will evolve its data schema over time. **Confidence A.**

**ACID transactions:** Iceberg provides serialisable isolation for concurrent reads and writes. This ensures that PIT queries return consistent results even during concurrent data ingestion. **Confidence A.**

**Partition evolution:** Iceberg allows partition strategies to change over time without rewriting existing data. This enables query performance optimisation as data volumes grow. **Confidence A.**

**Hidden partitioning:** Iceberg automatically partitions data based on column values without requiring users to specify partition columns in queries. This simplifies query authoring. **Confidence A.**

### 3.3 Iceberg Ecosystem

Iceberg is supported by: Apache Spark, Apache Flink, Trino, Presto, DuckDB (via iceberg extension), Snowflake, AWS Athena, Google BigQuery, and Azure Synapse. This broad ecosystem support means that ChronoSentiment is not locked into a specific query engine. **Confidence A.**

### 3.4 Iceberg for ChronoSentiment

**PIT query pattern:** `SELECT * FROM decisions.market_data FOR SYSTEM_TIME AS OF TIMESTAMP '2025-03-15 09:30:00'` — returns the market data as it existed at 9:30 AM on 15 March 2025, before any subsequent revisions or additions.

**Snapshot retention:** Iceberg snapshots can be retained for configurable periods. For ChronoSentiment, snapshots should be retained indefinitely (or for the regulatory retention period) to support audit-grade PIT reconstruction.

**Confidence A** for Iceberg capabilities. **Confidence B** for ChronoSentiment-specific implementation patterns.

---

## 4. Delta Lake

### 4.1 Overview

Delta Lake is an open table format developed by Databricks, open-sourced in 2019. It provides similar capabilities to Apache Iceberg: ACID transactions, time travel, schema evolution, and scalable metadata management. Delta Lake has stronger integration with Apache Spark and is more widely adopted in enterprise data engineering contexts. **Confidence A.**

### 4.2 Delta Lake vs Apache Iceberg

| Capability | Apache Iceberg | Delta Lake |
|-----------|---------------|-----------|
| Time travel | ✅ Snapshot-based | ✅ Version-based |
| Schema evolution | ✅ Full | ✅ Full |
| ACID transactions | ✅ Serialisable | ✅ Serialisable |
| Spark integration | ✅ Good | ✅ Excellent |
| DuckDB integration | ✅ Via extension | ✅ Via extension |
| Ecosystem breadth | ✅ Broader | ⚠️ Narrower (Databricks-centric) |
| Metadata scalability | ✅ Better at scale | ⚠️ Metadata bottleneck at very large scale |
| Open governance | ✅ Apache Foundation | ⚠️ Linux Foundation (Databricks-led) |

**Recommendation:** Apache Iceberg is preferred for ChronoSentiment due to broader ecosystem support and open governance. Delta Lake is an acceptable alternative if the team has existing Spark/Databricks expertise. **Confidence D — strategic interpretation.**

### 4.3 Delta Lake for ChronoSentiment

**PIT query pattern:** `SELECT * FROM delta.`/path/to/market_data` VERSION AS OF 42` or `TIMESTAMP AS OF '2025-03-15 09:30:00'` — equivalent PIT capability to Iceberg.

**Confidence A** for Delta Lake capabilities. **Confidence B** for ChronoSentiment-specific implementation patterns.

---

## 5. DuckDB

### 5.1 Overview

DuckDB is an in-process analytical database management system. It is designed for fast analytical queries on local or remote data, without requiring a separate server process. As of 2026, DuckDB is the dominant in-process analytical database for data science and engineering workloads. **Confidence A.**

### 5.2 Core Capabilities Relevant to ChronoSentiment

**In-process execution:** DuckDB runs within the application process, eliminating network latency for query execution. This is critical for ChronoSentiment's interactive PIT query use cases. **Confidence A.**

**Parquet and Iceberg support:** DuckDB can query Parquet files directly (including remote S3/GCS/Azure Blob) and supports Apache Iceberg tables via extension. This enables PIT queries over Iceberg tables without a separate query engine. **Confidence A.**

**Columnar execution:** DuckDB uses a vectorised columnar execution engine, providing fast analytical query performance on large datasets. Benchmark performance is competitive with dedicated analytical databases for single-node workloads. **Confidence A.**

**SQL compatibility:** DuckDB supports standard SQL including window functions, CTEs, and complex aggregations. This enables sophisticated PIT query patterns without custom query languages. **Confidence A.**

**Embeddability:** DuckDB can be embedded in Python, Rust, Go, Java, and other languages. This enables ChronoSentiment to embed analytical query capability directly in its application layer. **Confidence A.**

### 5.3 DuckDB for ChronoSentiment

**PIT query pattern (Parquet):**
```sql
SELECT *
FROM read_parquet('s3://chronosentiment/market_data/date=2025-03-15/*.parquet')
WHERE timestamp <= '2025-03-15 09:30:00'
ORDER BY timestamp DESC
LIMIT 1
```

**PIT query pattern (Iceberg via extension):**
```sql
INSTALL iceberg;
LOAD iceberg;
SELECT * FROM iceberg_scan('s3://chronosentiment/market_data')
WHERE snapshot_id = (
  SELECT snapshot_id FROM iceberg_snapshots('s3://chronosentiment/market_data')
  WHERE committed_at <= '2025-03-15 09:30:00'
  ORDER BY committed_at DESC LIMIT 1
)
```

**Confidence A** for DuckDB capabilities. **Confidence B** for ChronoSentiment-specific query patterns.

---

## 6. Architecture Recommendations

### 6.1 MVP Architecture (Phase 1 / Phase 2)

**Objective:** Validate the PIT data concept with minimum infrastructure complexity. Prioritise correctness and simplicity over performance and scale.

```
Data Ingestion
    │
    ▼
Append-Only Parquet Files
(partitioned by date, stored in S3/GCS/local)
    │
    ▼
DuckDB (in-process)
(PIT queries via timestamp filtering on Parquet partitions)
    │
    ▼
ChronoSentiment Application Layer
(decision capture, explainability, audit trail)
```

**MVP data model:** Each data type (market data, fundamentals, news) stored as append-only Parquet files partitioned by ingestion date. PIT queries filter by ingestion timestamp. No schema evolution support; no ACID transactions. Sufficient for Phase 1B validation.

**MVP technology stack:**
- Storage: Local filesystem or S3-compatible object storage
- Format: Parquet (columnar, compressed)
- Query engine: DuckDB (embedded in Python application)
- Ingestion: Python scripts using pandas/polars + pyarrow

**MVP cost:** Minimal. DuckDB is free and open-source. Parquet storage on S3 costs approximately US$0.023/GB/month. For MVP data volumes (< 100GB), total storage cost < US$3/month.

**MVP limitations:** No time-travel queries (only timestamp filtering); no schema evolution; no concurrent write safety; not suitable for production data volumes.

### 6.2 Production Architecture (Phase 3+)

**Objective:** Production-grade PIT data infrastructure supporting: time-travel queries, schema evolution, concurrent access, large data volumes, and audit-grade provenance.

```
Data Ingestion (streaming + batch)
    │
    ▼
Apache Iceberg Tables
(on S3/GCS, with Iceberg catalog — AWS Glue or Nessie)
    │
    ├──────────────────────────────────┐
    ▼                                  ▼
DuckDB (in-process)              Apache Spark / Trino
(interactive PIT queries)        (batch processing, large-scale analytics)
    │
    ▼
ChronoSentiment Application Layer
(decision capture, explainability, audit trail, replay engine)
```

**Production data model:** Each data type stored as Apache Iceberg tables with: time-travel snapshots retained indefinitely, schema evolution support, ACID transaction guarantees, and partition evolution for query performance optimisation.

**Production technology stack:**
- Storage: S3-compatible object storage (AWS S3, GCS, or MinIO for self-hosted)
- Format: Apache Iceberg (with Parquet as the underlying file format)
- Catalog: AWS Glue Data Catalog or Project Nessie (open-source, Git-like catalog)
- Interactive query engine: DuckDB (embedded, via Iceberg extension)
- Batch query engine: Apache Spark or Trino (for large-scale analytics)
- Ingestion: Apache Flink (streaming) or Apache Spark (batch)

**Production cost (indicative):** S3 storage ~US$0.023/GB/month; compute for Spark/Flink jobs ~US$0.10–US$1.00/compute-hour; DuckDB is free. For production data volumes (1–10TB), total infrastructure cost ~US$500–US$2,000/month. **Confidence C — highly dependent on data volumes and query patterns.**

### 6.3 MVP to Production Migration Path

| Phase | Architecture | Key Milestone |
|-------|-------------|--------------|
| Phase 1 (MVP) | Append-only Parquet + DuckDB | Phase 1B customer validation |
| Phase 2 (Beta) | Iceberg tables + DuckDB + basic catalog | First paying customer |
| Phase 3 (Production) | Full Iceberg + DuckDB + Spark/Trino + Nessie | Production deployment |

The migration from MVP to production is designed to be incremental: Parquet files can be registered as Iceberg tables without rewriting data, and DuckDB queries work against both Parquet and Iceberg. This minimises migration risk.

---

## 7. Research Findings

### Finding 1: Apache Iceberg is the recommended production storage format (Confidence B)

Iceberg's combination of time-travel queries, schema evolution, ACID transactions, and broad ecosystem support makes it the appropriate production storage format for ChronoSentiment's PIT data requirements. Delta Lake is an acceptable alternative with equivalent capabilities but narrower ecosystem support.

### Finding 2: DuckDB is the recommended query engine for interactive PIT queries (Confidence A)

DuckDB's in-process execution, Parquet and Iceberg support, columnar performance, and embeddability make it the appropriate query engine for ChronoSentiment's interactive PIT query use cases. It eliminates the operational complexity of a separate query engine for the MVP and early production phases.

### Finding 3: The MVP architecture (Parquet + DuckDB) is sufficient for Phase 1B validation (Confidence B)

The MVP architecture does not require Iceberg or a distributed query engine. Append-only Parquet files with DuckDB timestamp filtering provide adequate PIT capability for Phase 1B customer validation at minimal infrastructure cost and complexity.

### Finding 4: The production architecture requires an Iceberg catalog (Confidence B)

Production deployment requires an Iceberg catalog to manage table metadata, snapshot history, and schema evolution. AWS Glue Data Catalog is the simplest option for AWS-hosted deployments. Project Nessie is the recommended open-source alternative for multi-cloud or self-hosted deployments.

### Finding 5: Snapshot retention policy is a regulatory requirement, not just a technical choice (Confidence B)

EU AI Act Article 12 requires logging of AI system events throughout the system's lifecycle. For ChronoSentiment, this means Iceberg snapshots must be retained for the regulatory retention period (typically 5–7 years for investment management records). Snapshot retention policy must be designed into the production architecture from the beginning.

---

## 8. Evidence Sufficiency Assessment

| Area | Evidence Sufficiency | Notes |
|------|---------------------|-------|
| Apache Iceberg capabilities | High | Extensive public documentation and benchmarks |
| Delta Lake capabilities | High | Extensive public documentation and benchmarks |
| DuckDB capabilities | High | Extensive public documentation and benchmarks |
| MVP architecture design | Medium | Based on published technology capabilities; not yet tested at ChronoSentiment scale |
| Production architecture design | Medium | Based on published case studies; not yet tested at ChronoSentiment scale |
| Performance at ChronoSentiment data volumes | Low | Requires proof-of-concept benchmarking |
| Operational complexity at production scale | Low | Requires production deployment experience |
| Cost at production scale | Low | Highly dependent on data volumes and query patterns |

---

## 9. Outstanding Validation Questions

1. **Data volume:** What are ChronoSentiment's expected data volumes at MVP, beta, and production? This determines whether DuckDB alone is sufficient or whether Spark/Trino is required.
2. **Query latency requirements:** What is the acceptable latency for PIT queries in the ChronoSentiment application? This determines whether in-process DuckDB is sufficient or whether a caching layer is required.
3. **Concurrent access:** How many concurrent users will issue PIT queries? This determines whether DuckDB's single-writer model is sufficient or whether a distributed query engine is required.
4. **Snapshot retention:** What is the regulatory retention period for ChronoSentiment's PIT data? This determines the Iceberg snapshot retention policy and storage cost.
5. **Catalog selection:** Is ChronoSentiment deploying on AWS (Glue catalog), multi-cloud (Nessie), or self-hosted? This determines the catalog technology selection.

**Research method:** Proof-of-concept implementation with representative data volumes and query patterns. Benchmark DuckDB performance against expected production query patterns before committing to the production architecture.

---

## 10. PRD Traceability

| PRD Requirement | Architecture Component | CS-R-008 Section | Confidence |
|-----------------|----------------------|-----------------|------------|
| Point-in-time data reconstruction | Iceberg time-travel / Parquet timestamp filtering | Sections 3, 4, 6 | B |
| Audit trail for AI decisions | Iceberg snapshot retention | Section 7, Finding 5 | B |
| Data provenance documentation | Iceberg catalog + snapshot metadata | Sections 3.2, 6.2 | B |
| Deterministic replay | Iceberg snapshot pinning + DuckDB query | Sections 3.2, 5.3 | B |
| Schema evolution support | Iceberg schema evolution | Section 3.2 | A |
| MVP feasibility | Parquet + DuckDB stack | Section 6.1 | B |

---

## Evidence Quality

| Source Type | Examples | Confidence |
|-------------|---------|-----------|
| Technology documentation | Apache Iceberg docs, Delta Lake docs, DuckDB docs | A |
| Academic / technical publications | Look-ahead bias literature (Banz 1981, Hou et al. 2020) | A |
| Industry benchmarks | DuckDB TPC-H benchmarks, Iceberg performance studies | B |
| Architecture design | MVP and production architecture recommendations | D |

---

## Evidence Classification

**Published evidence:** Apache Iceberg specification and documentation, Delta Lake documentation, DuckDB documentation and benchmarks, academic literature on look-ahead bias in financial modelling.

**Derived findings:** Technology comparison table derived from published documentation; MVP architecture design derived from technology capabilities and ChronoSentiment requirements; snapshot retention as regulatory requirement derived from EU AI Act Article 12.

**Strategic interpretation (Confidence D):** Iceberg preferred over Delta Lake; MVP Parquet + DuckDB stack; production Iceberg + DuckDB + Spark/Trino architecture; migration path design. These require proof-of-concept validation before adoption as the basis for M-series architecture decisions.

---

*CS-R-008 v2.0 | ChronoSentiment Research Series | Evidence cut-off: July 2026*
*Supersedes CS-R-008 v1.0. v1.0 retained as historical record.*