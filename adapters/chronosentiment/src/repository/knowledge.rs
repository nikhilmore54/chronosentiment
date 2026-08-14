use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::error::Error;
use uuid::Uuid;
use crate::reasoning::assessment::AssessmentProfile;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ArtifactType {
    Observation,
    MetricReport,
    Evidence,
    Assessment,
    Hypothesis,
    Decision,
    ScenarioProjection,
    ObservatorySnapshot,
    ResearchReport,
    Outcome,
    Strategy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactLineage {
    pub produced_by: String,
    pub consumed_artifacts: Vec<Uuid>,
    pub parent_artifacts: Vec<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactMetadata {
    pub artifact_id: Uuid,
    pub artifact_schema_version: String,
    pub artifact_type: ArtifactType,
    pub created_at: DateTime<Utc>,
    pub evaluation_timestamp: DateTime<Utc>,
    pub engine_versions: serde_json::Value,
    pub lineage: ArtifactLineage,
    pub replay_context_hash: String,
    pub knowledge_lake_version: String,
    pub content_hash: String,
}

impl ArtifactMetadata {
    /// Placeholder metadata. `evaluation_timestamp` is wall-clock and MUST be
    /// overwritten with replay `dt` before a Knowledge Lake persist.
    pub fn mock() -> Self {
        Self {
            artifact_id: Uuid::new_v4(),
            artifact_schema_version: "1.0".to_string(),
            artifact_type: ArtifactType::Assessment,
            created_at: Utc::now(),
            evaluation_timestamp: Utc::now(),
            engine_versions: serde_json::json!({}),
            lineage: ArtifactLineage {
                produced_by: "mock_engine".to_string(),
                consumed_artifacts: vec![],
                parent_artifacts: vec![],
            },
            replay_context_hash: "mock".to_string(),
            knowledge_lake_version: "mock".to_string(),
            content_hash: "mock".to_string(),
        }
    }
}

pub trait KnowledgeArtifact {
    fn metadata(&self) -> &ArtifactMetadata;
    fn instrument_id(&self) -> Option<Uuid>;
}

#[async_trait]
pub trait ArtifactRepository<T: KnowledgeArtifact + Send + Sync> {
    async fn store(&self, artifact: &T) -> Result<(), Box<dyn Error>>;
    async fn get(&self, id: Uuid) -> Result<Option<T>, Box<dyn Error>>;
}

#[async_trait]
pub trait AssessmentQueries {
    async fn find_by_signature(
        &self,
        signature: &str,
    ) -> Result<Vec<AssessmentProfile>, Box<dyn Error>>;
    
    async fn find_by_market_state(
        &self,
        state: &str,
    ) -> Result<Vec<AssessmentProfile>, Box<dyn Error>>;
}
