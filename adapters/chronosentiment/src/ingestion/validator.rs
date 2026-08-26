use crate::instrument::Instrument;
use crate::observation::{RawObservation, ValidatedObservation};
use chrono::Utc;
use uuid::Uuid;

pub struct ValidationEngine;

impl ValidationEngine {
    pub fn validate(&self, raw: RawObservation, instrument: &Instrument) -> ValidatedObservation {
        let now = Utc::now();
        // In a real system, compute cryptographic hash of raw_payload here
        let provenance_hash = "mock_hash_5f4dcc3b5aa765d61d8327deb882cf99".to_string();

        ValidatedObservation {
            id: Uuid::new_v4(),
            research_session_id: None,
            instrument_id: Some(instrument.id),
            observation_type: raw.observation_type,
            source: raw.source,
            source_identifier: raw.source_identifier,
            observed_at: raw.observed_at,
            effective_from: raw.observed_at, // For market price, it is effective as soon as it happens
            effective_to: None,
            recorded_at: now,
            raw_payload: raw.raw_payload,
            normalized_payload: raw.normalized_payload,
            confidence: 1.0,
            freshness: 0.0,
            coverage: "Complete".to_string(),
            consistency: None,
            quality_score: 1.0,
            provenance_hash,
            schema_version: 1,
        }
    }
}
