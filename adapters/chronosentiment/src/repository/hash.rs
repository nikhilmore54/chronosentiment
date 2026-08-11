use serde::Serialize;
use sha2::{Sha256, Digest};

/// Generates a deterministic content hash for an artifact following ARCH-012.
pub fn generate_content_hash<T: Serialize>(payload: &T, metadata: &crate::repository::knowledge::ArtifactMetadata) -> String {
    #[derive(Serialize)]
    struct HashEnvelope<'a, P> {
        artifact_schema_version: &'a str,
        artifact_type: &'a crate::repository::knowledge::ArtifactType,
        evaluation_timestamp: &'a chrono::DateTime<chrono::Utc>,
        engine_versions: &'a serde_json::Value,
        lineage: &'a crate::repository::knowledge::ArtifactLineage,
        replay_context_hash: &'a str,
        knowledge_lake_version: &'a str,
        payload: &'a P,
    }

    let envelope = HashEnvelope {
        artifact_schema_version: &metadata.artifact_schema_version,
        artifact_type: &metadata.artifact_type,
        evaluation_timestamp: &metadata.evaluation_timestamp,
        engine_versions: &metadata.engine_versions,
        lineage: &metadata.lineage,
        replay_context_hash: &metadata.replay_context_hash,
        knowledge_lake_version: &metadata.knowledge_lake_version,
        payload,
    };

    // Note: serde_json::to_string automatically sorts BTreeMap keys but NOT struct fields.
    // However, HashEnvelope struct fields are fixed order, and payload is presumed deterministic.
    let json_bytes = serde_json::to_vec(&envelope).expect("Failed to serialize for hashing");
    let mut hasher = Sha256::new();
    hasher.update(&json_bytes);
    format!("{:x}", hasher.finalize())
}
