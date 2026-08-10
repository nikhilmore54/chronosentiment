use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};
use std::error::Error;
use uuid::Uuid;

use crate::instrument::Instrument;
use crate::observation::Observation;

/// The central Persistence Service for the Knowledge Lake.
/// Enforces the invariants: Time Travel is first class, and raw observations never hit the DB.
#[async_trait]
pub trait ObservationRepository: Send + Sync {
    /// Stores an enriched, validated, canonical observation envelope in the Knowledge Lake.
    async fn store_observation(&self, observation: &Observation) -> Result<(), Box<dyn Error>>;

    /// Time-Travel Query: Retrieves all observations for a specific instrument 
    /// that were known precisely AT or BEFORE the `evaluation_timestamp`.
    /// Observations whose `effective_from` is after the `evaluation_timestamp` are strictly excluded.
    async fn get_observations_as_of(
        &self,
        instrument_id: Uuid,
        evaluation_timestamp: DateTime<Utc>,
    ) -> Result<Vec<Observation>, Box<dyn Error>>;

    /// Retrieves the full, unredacted historical observation timeline for an instrument.
    async fn get_complete_history(&self, instrument_id: Uuid) -> Result<Vec<Observation>, Box<dyn Error>>;
}

/// The Instrument Master Repository.
#[async_trait]
pub trait InstrumentRepository: Send + Sync {
    /// Stores a new instrument in the master table.
    async fn store_instrument(&self, instrument: &Instrument) -> Result<(), Box<dyn Error>>;

    /// Looks up an instrument by its exchange and display symbol.
    async fn get_by_symbol(&self, exchange: &str, display_symbol: &str) -> Result<Option<Instrument>, Box<dyn Error>>;
    
    /// Retrieves an instrument by its unique ID.
    async fn get_by_id(&self, id: Uuid) -> Result<Option<Instrument>, Box<dyn Error>>;
}

pub struct PostgresRepository {
    pool: PgPool,
}

impl PostgresRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ObservationRepository for PostgresRepository {
    async fn store_observation(&self, observation: &Observation) -> Result<(), Box<dyn Error>> {
        let raw_payload = sqlx::types::Json(&observation.raw_payload);
        let normalized_payload = sqlx::types::Json(&observation.normalized_payload);

        sqlx::query(
            r#"
            INSERT INTO observations (
                id, observation_type, observed_at, effective_from, effective_to,
                recorded_at, instrument_id, raw_payload, normalized_payload, confidence_score,
                freshness_at, quality_score, source_name, coverage, consistency_score,
                provenance_hash, schema_version
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17
            )
            "#,
        )
        .bind(observation.id)
        .bind(&observation.observation_type)
        .bind(observation.observed_at)
        .bind(observation.effective_from)
        .bind(observation.effective_to)
        .bind(observation.recorded_at)
        .bind(observation.instrument_id)
        .bind(raw_payload)
        .bind(normalized_payload)
        .bind(observation.confidence)
        .bind(observation.freshness)
        .bind(observation.quality_score)
        .bind(&observation.source)
        .bind(&observation.coverage)
        .bind(observation.consistency)
        .bind(&observation.provenance_hash)
        .bind(observation.schema_version as i32)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn get_observations_as_of(
        &self,
        instrument_id: Uuid,
        evaluation_timestamp: DateTime<Utc>,
    ) -> Result<Vec<Observation>, Box<dyn Error>> {
        let records = sqlx::query(
            r#"
            SELECT 
                id, observation_type, observed_at, effective_from, effective_to,
                recorded_at, instrument_id, raw_payload, normalized_payload, confidence_score,
                freshness_at, quality_score, source_name, coverage, consistency_score,
                provenance_hash, schema_version
            FROM observations
            WHERE instrument_id = $1
              AND effective_from <= $2
            ORDER BY effective_from ASC
            "#
        )
        .bind(instrument_id)
        .bind(evaluation_timestamp)
        .fetch_all(&self.pool)
        .await?;

        let observations = records.into_iter().map(|rec| {
            Observation {
                id: rec.get("id"),
                research_session_id: None,
                instrument_id: rec.get("instrument_id"),
                observation_type: rec.get("observation_type"),
                source: rec.get("source_name"),
                source_identifier: None,
                observed_at: rec.get("observed_at"),
                effective_from: rec.get("effective_from"),
                effective_to: rec.get("effective_to"),
                recorded_at: rec.get("recorded_at"),
                raw_payload: rec.get::<sqlx::types::Json<serde_json::Value>, _>("raw_payload").0,
                normalized_payload: rec.get::<sqlx::types::Json<serde_json::Value>, _>("normalized_payload").0,
                confidence: rec.get("confidence_score"),
                freshness: rec.get("freshness_at"),
                coverage: rec.get("coverage"),
                consistency: rec.get("consistency_score"),
                quality_score: rec.get("quality_score"),
                provenance_hash: rec.get("provenance_hash"),
                schema_version: rec.get::<i32, _>("schema_version") as u32,
            }
        }).collect();

        Ok(observations)
    }

    async fn get_complete_history(&self, instrument_id: Uuid) -> Result<Vec<Observation>, Box<dyn Error>> {
        let records = sqlx::query(
            r#"
            SELECT 
                id, observation_type, observed_at, effective_from, effective_to,
                recorded_at, instrument_id, raw_payload, normalized_payload, confidence_score,
                freshness_at, quality_score, source_name, coverage, consistency_score,
                provenance_hash, schema_version
            FROM observations
            WHERE instrument_id = $1
            ORDER BY effective_from ASC
            "#
        )
        .bind(instrument_id)
        .fetch_all(&self.pool)
        .await?;

        let observations = records.into_iter().map(|rec| {
            Observation {
                id: rec.get("id"),
                research_session_id: None, 
                instrument_id: rec.get("instrument_id"),
                observation_type: rec.get("observation_type"),
                source: rec.get("source_name"),
                source_identifier: None,
                observed_at: rec.get("observed_at"),
                effective_from: rec.get("effective_from"),
                effective_to: rec.get("effective_to"),
                recorded_at: rec.get("recorded_at"),
                raw_payload: rec.get::<sqlx::types::Json<serde_json::Value>, _>("raw_payload").0,
                normalized_payload: rec.get::<sqlx::types::Json<serde_json::Value>, _>("normalized_payload").0,
                confidence: rec.get("confidence_score"),
                freshness: rec.get("freshness_at"),
                coverage: rec.get("coverage"),
                consistency: rec.get("consistency_score"),
                quality_score: rec.get("quality_score"),
                provenance_hash: rec.get("provenance_hash"),
                schema_version: rec.get::<i32, _>("schema_version") as u32,
            }
        }).collect();

        Ok(observations)
    }
}

#[async_trait]
impl InstrumentRepository for PostgresRepository {
    async fn store_instrument(&self, instrument: &Instrument) -> Result<(), Box<dyn Error>> {
        let provider_ids = sqlx::types::Json(&instrument.provider_ids);

        sqlx::query(
            r#"
            INSERT INTO instruments (id, exchange, display_symbol, provider_ids, created_at)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (exchange, display_symbol) DO UPDATE 
            SET provider_ids = EXCLUDED.provider_ids
            "#
        )
        .bind(instrument.id)
        .bind(&instrument.exchange)
        .bind(&instrument.display_symbol)
        .bind(provider_ids)
        .bind(instrument.created_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn get_by_symbol(&self, exchange: &str, display_symbol: &str) -> Result<Option<Instrument>, Box<dyn Error>> {
        let record = sqlx::query(
            r#"
            SELECT id, exchange, display_symbol, provider_ids, created_at
            FROM instruments
            WHERE exchange = $1 AND display_symbol = $2
            "#
        )
        .bind(exchange)
        .bind(display_symbol)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(rec) = record {
            Ok(Some(Instrument {
                id: rec.get("id"),
                exchange: rec.get("exchange"),
                display_symbol: rec.get("display_symbol"),
                provider_ids: rec.get::<sqlx::types::Json<std::collections::HashMap<String, String>>, _>("provider_ids").0,
                created_at: rec.get("created_at"),
            }))
        } else {
            Ok(None)
        }
    }

    async fn get_by_id(&self, id: Uuid) -> Result<Option<Instrument>, Box<dyn Error>> {
        let record = sqlx::query(
            r#"
            SELECT id, exchange, display_symbol, provider_ids, created_at
            FROM instruments
            WHERE id = $1
            "#
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(rec) = record {
            Ok(Some(Instrument {
                id: rec.get("id"),
                exchange: rec.get("exchange"),
                display_symbol: rec.get("display_symbol"),
                provider_ids: rec.get::<sqlx::types::Json<std::collections::HashMap<String, String>>, _>("provider_ids").0,
                created_at: rec.get("created_at"),
            }))
        } else {
            Ok(None)
        }
    }
}
