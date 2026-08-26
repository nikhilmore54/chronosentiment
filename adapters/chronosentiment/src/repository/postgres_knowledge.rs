use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};
use std::error::Error;
use uuid::Uuid;

use crate::reasoning::assessment::AssessmentProfile;
use crate::reasoning::decision::Decision;
use crate::reasoning::strategy::OpportunityStrategy;
use crate::repository::knowledge::{
    ArtifactMetadata, ArtifactRepository, ArtifactType, AssessmentQueries, KnowledgeArtifact,
};
use crate::validation::outcome::OutcomeRecord;

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
            "#,
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
            "#,
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
impl ArtifactRepository<OutcomeRecord> for PostgresKnowledgeRepository {
    async fn store(&self, artifact: &OutcomeRecord) -> Result<(), Box<dyn Error>> {
        let meta = artifact.metadata();
        let metadata_json = serde_json::to_value(meta)?;
        let outcome_json = serde_json::to_value(artifact)?;

        sqlx::query(
            r#"
            INSERT INTO knowledge_outcomes (
                id, decision_id, strategy_id, instrument_id,
                evaluation_timestamp, horizon, horizon_expiry_timestamp, observation_end_timestamp,
                entry_reached, target_hit, stop_hit, exit_reason,
                outcome_return, mfe, mae, drawdown,
                metadata_json, outcome_json
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18)
            "#,
        )
        .bind(meta.artifact_id)
        .bind(artifact.decision_id)
        .bind(artifact.strategy_id)
        .bind(artifact.instrument_id)
        .bind(artifact.evaluation_timestamp)
        .bind(&artifact.horizon)
        .bind(artifact.horizon_expiry_timestamp)
        .bind(artifact.observation_end_timestamp)
        .bind(artifact.entry_reached)
        .bind(artifact.target_hit)
        .bind(artifact.stop_hit)
        .bind(&artifact.exit_reason)
        .bind(artifact.outcome_return)
        .bind(artifact.mfe)
        .bind(artifact.mae)
        .bind(artifact.maximum_drawdown)
        .bind(metadata_json)
        .bind(outcome_json)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn get(&self, id: Uuid) -> Result<Option<OutcomeRecord>, Box<dyn Error>> {
        let record = sqlx::query(
            r#"
            SELECT outcome_json
            FROM knowledge_outcomes
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        match record {
            Some(row) => {
                let outcome_json: serde_json::Value = row.try_get("outcome_json")?;
                let outcome: OutcomeRecord = serde_json::from_value(outcome_json)?;
                Ok(Some(outcome))
            }
            None => Ok(None),
        }
    }
}

#[async_trait]
impl ArtifactRepository<Decision> for PostgresKnowledgeRepository {
    async fn store(&self, artifact: &Decision) -> Result<(), Box<dyn Error>> {
        let meta = artifact.metadata();
        let metadata_json = serde_json::to_value(meta)?;
        let decision_json = serde_json::to_value(artifact)?;
        let opp_str = format!("{:?}", artifact.opportunity);

        sqlx::query(
            r#"
            INSERT INTO knowledge_decisions (
                id, instrument_id, evaluation_timestamp, assessment_id,
                opportunity, metadata_json, decision_json
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
        )
        .bind(meta.artifact_id)
        .bind(artifact.instrument_id)
        .bind(artifact.evaluation_timestamp)
        .bind(artifact.assessment_id)
        .bind(opp_str)
        .bind(metadata_json)
        .bind(decision_json)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn get(&self, id: Uuid) -> Result<Option<Decision>, Box<dyn Error>> {
        let record = sqlx::query(
            r#"
            SELECT decision_json
            FROM knowledge_decisions
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        match record {
            Some(row) => {
                let decision_json: serde_json::Value = row.try_get("decision_json")?;
                let decision: Decision = serde_json::from_value(decision_json)?;
                Ok(Some(decision))
            }
            None => Ok(None),
        }
    }
}

#[async_trait]
impl ArtifactRepository<OpportunityStrategy> for PostgresKnowledgeRepository {
    async fn store(&self, artifact: &OpportunityStrategy) -> Result<(), Box<dyn Error>> {
        let meta = artifact.metadata();
        let metadata_json = serde_json::to_value(meta)?;
        let strategy_json = serde_json::to_value(artifact)?;
        let horizon_str = format!("{:?}", artifact.expected_horizon);

        sqlx::query(
            r#"
            INSERT INTO knowledge_strategies (
                id, decision_id, expected_horizon,
                metadata_json, strategy_json
            )
            VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(meta.artifact_id)
        .bind(artifact.decision_id)
        .bind(horizon_str)
        .bind(metadata_json)
        .bind(strategy_json)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn get(&self, id: Uuid) -> Result<Option<OpportunityStrategy>, Box<dyn Error>> {
        let record = sqlx::query(
            r#"
            SELECT strategy_json
            FROM knowledge_strategies
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        match record {
            Some(row) => {
                let strategy_json: serde_json::Value = row.try_get("strategy_json")?;
                let strategy: OpportunityStrategy = serde_json::from_value(strategy_json)?;
                Ok(Some(strategy))
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
