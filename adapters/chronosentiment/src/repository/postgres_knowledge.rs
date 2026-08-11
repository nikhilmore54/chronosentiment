use async_trait::async_trait;
use sqlx::{PgPool, Row};
use std::error::Error;
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::repository::knowledge::{
    ArtifactMetadata, ArtifactRepository, ArtifactType, AssessmentQueries, KnowledgeArtifact,
};
use crate::reasoning::assessment::AssessmentProfile;

pub struct PostgresKnowledgeRepository {
    pool: PgPool,
}

impl PostgresKnowledgeRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ArtifactRepository<AssessmentProfile> for PostgresKnowledgeRepository {
    async fn store(&self, artifact: &AssessmentProfile) -> Result<(), Box<dyn Error>> {
        let meta = artifact.metadata();
        let metadata_json = serde_json::to_value(meta)?;
        let profile_json = serde_json::to_value(artifact)?;
        
        let signature = artifact.to_signature();
        let signature_json = serde_json::Value::String(signature.clone());
        let signature_hash = artifact.to_hash();

        sqlx::query(
            r#"
            INSERT INTO knowledge_assessments (
                id, instrument_id, evaluation_timestamp,
                signature, signature_hash, metadata_json, profile_json
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#
        )
        .bind(meta.artifact_id)
        .bind(artifact.instrument_id())
        .bind(meta.evaluation_timestamp)
        .bind(signature_json)
        .bind(signature_hash)
        .bind(metadata_json)
        .bind(profile_json)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn get(&self, id: Uuid) -> Result<Option<AssessmentProfile>, Box<dyn Error>> {
        let record = sqlx::query(
            r#"
            SELECT profile_json
            FROM knowledge_assessments
            WHERE id = $1
            "#
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        match record {
            Some(row) => {
                let profile_json: serde_json::Value = row.try_get("profile_json")?;
                let profile: AssessmentProfile = serde_json::from_value(profile_json)?;
                Ok(Some(profile))
            }
            None => Ok(None),
        }
    }
}

#[async_trait]
impl AssessmentQueries for PostgresKnowledgeRepository {
    async fn find_by_signature(
        &self,
        _signature: &str,
    ) -> Result<Vec<AssessmentProfile>, Box<dyn Error>> {
        // AssessmentSignature isn't fully defined as a type in assessment.rs, we use signature string hash
        // In a real implementation this would serialize signature to hash
        Err("Not implemented".into())
    }

    async fn find_by_market_state(
        &self,
        _state: &str,
    ) -> Result<Vec<AssessmentProfile>, Box<dyn Error>> {
        Err("Not implemented".into())
    }
}
