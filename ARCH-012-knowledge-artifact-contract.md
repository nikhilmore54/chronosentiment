# ARCH-012 — Knowledge Artifact Contract

## 1. Core Philosophy

The **Reasoning Knowledge Lake** is an append-only event store of immutable reasoning artifacts. It acts as the permanent record for the ChronoSentiment platform, ensuring that every step of the decision-making process is fully reproducible and explainable.

## 2. Artifact Metadata

Every artifact persisted in the Knowledge Lake must include a standardized metadata envelope:

```rust
pub struct ArtifactMetadata {
    pub artifact_id: Uuid,
    pub artifact_schema_version: String,
    pub artifact_type: ArtifactType,
    pub created_at: DateTime<Utc>,
    pub evaluation_timestamp: DateTime<Utc>,
    pub engine_versions: serde_json::Value, // e.g. EngineVersionSet
    pub lineage: ArtifactLineage,
    pub replay_context_hash: String,
    pub knowledge_lake_version: String,
    pub content_hash: String,
}
```

### Artifact Identity vs Content
*   `artifact_id`: Uniquely identifies this exact persisted object.
*   `evaluation_timestamp`: When was this decision state evaluated?
*   `replay_context_hash`: What exact information state produced it?
*   `knowledge_lake_version`: Which version of the historical knowledge universe was used?
*   `content_hash`: Uniquely identifies the reasoning result, enabling deduplication, integrity verification, and distributed synchronization.

### Artifact Types

Supported types include:
*   `Observation`
*   `MetricReport`
*   `Evidence`
*   `Assessment`
*   `Hypothesis`
*   `Decision`
*   `ScenarioProjection`
*   `Outcome`
*   `ObservatorySnapshot`
*   `ResearchReport`

## 3. Artifact Lineage

Every artifact must record its provenance. This makes the reasoning chain fully reconstructible.

```rust
pub struct ArtifactLineage {
    pub produced_by: String, 
    pub consumed_artifacts: Vec<Uuid>,
    pub parent_artifacts: Vec<Uuid>,
}
```

## 4. Lifecycle

```text
Created
  ↓
Persisted
  ↓
Immutable
  ↓
Queryable
  ↓
Referenced
  ↓
Archived
```

## 5. Repository Rules

All repositories implementing the Knowledge Artifact Contract must strictly adhere to the following rules:

1.  **Append Only:** Repositories must never update an artifact in place. Corrections produce new artifacts with new identities and lineage.
2.  **Never Mutate:** Once persisted, an artifact is immutable.
3.  **Lineage Required:** Every artifact must include a fully populated `ArtifactLineage`.
4.  **Metadata Mandatory:** Every artifact must include `ArtifactMetadata`.
5.  **Schema Version Mandatory:** Every artifact must include an `artifact_schema_version` to support future backwards-compatible deserialization and migrations.

## 6. Serialization Rules

1.  **Format:** JSON.
2.  **Hashing:** The `content_hash` is the SHA-256 hash of the canonical deterministic JSON containing the artifact payload plus all reproducibility-relevant metadata, excluding only `artifact_id` and `created_at`. This means identical reasoning results with different engine versions or contexts will produce different hashes.
3.  **Versioning:** `artifact_schema_version` must be bumped whenever the structure changes.
4.  **Integrity Verification:** Clients must be able to re-hash the artifact content and compare it to the `content_hash` to verify integrity.
