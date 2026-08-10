use chrono::Utc;
use serde_json::Value;
use uuid::Uuid;
use sha2::{Sha256, Digest};
use crate::observation::Observation;

pub mod context;
pub mod replay;

pub struct ValidationEngine;

impl ValidationEngine {
    /// Enriches a raw observation into a canonical, immutable Observation envelope.
    pub fn enrich_observation(
        instrument_id: Uuid,
        observation_type: &str,
        source: &str,
        raw_payload: Value,
        normalized_payload: Value,
        confidence: f64,
        coverage: &str,
    ) -> Observation {
        let now = Utc::now();
        
        let mut obs = Observation {
            id: Uuid::new_v4(),
            research_session_id: None,
            instrument_id: Some(instrument_id),
            observation_type: observation_type.to_string(),
            source: source.to_string(),
            source_identifier: None,
            observed_at: now, // In reality, this comes from the provider. Using 'now' as default fallback.
            effective_from: now,
            effective_to: None,
            recorded_at: now,
            raw_payload,
            normalized_payload,
            confidence,
            freshness: 0.0, // newly enriched
            coverage: coverage.to_string(),
            consistency: None,
            quality_score: confidence, // Naive aggregate for now
            provenance_hash: String::new(),
            schema_version: 1,
        };
        
        obs.provenance_hash = Self::compute_hash(&obs);
        obs
    }
    
    /// Computes a cryptographic hash to ensure immutability
    fn compute_hash(obs: &Observation) -> String {
        let mut hasher = Sha256::new();
        hasher.update(obs.source.as_bytes());
        hasher.update(obs.observation_type.as_bytes());
        hasher.update(obs.observed_at.timestamp().to_be_bytes());
        hasher.update(obs.raw_payload.to_string().as_bytes());
        
        format!("{:x}", hasher.finalize())
    }
}
