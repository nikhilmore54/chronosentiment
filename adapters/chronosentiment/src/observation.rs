use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

/// A canonical observation produced by any provider.
/// It represents a single, immutable fact observed in the world.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Observation {
    /// Globally unique identifier for this observation
    pub id: Uuid,
    
    /// The research session this observation was collected for (optional)
    pub research_session_id: Option<Uuid>,

    /// The instrument this observation relates to (links to Instrument Master)
    pub instrument_id: Option<Uuid>,
    
    /// The type of observation (e.g., 'MarketPrice', 'MacroRelease', 'EarningsReport')
    pub observation_type: String,

    /// The gateway/provider that sourced this observation (e.g., 'Kite', 'FRED', 'EDGAR')
    pub source: String,
    
    /// The provider's internal identifier for this observation (if any)
    pub source_identifier: Option<String>,

    /// When the event actually occurred in the real world
    pub observed_at: DateTime<Utc>,
    
    /// When this observation becomes valid/effective for reasoning (prevents look-ahead bias)
    pub effective_from: DateTime<Utc>,
    
    /// When this observation is superseded or expires (optional)
    pub effective_to: Option<DateTime<Utc>>,
    
    /// When ChronoSentiment permanently recorded this observation
    pub recorded_at: DateTime<Utc>,

    /// The exact untouched payload from the provider
    pub raw_payload: Value,

    /// The transformed, standardized payload for downstream reasoning
    pub normalized_payload: Value,

    /// Knowledge Confidence scoring
    pub confidence: f64,
    pub freshness: f64, // e.g., age in hours when recorded
    pub coverage: String, // 'Complete', 'Partial'
    pub consistency: Option<f64>,
    pub quality_score: f64, // Aggregated quality metric

    /// Cryptographic hash of the payload and source to ensure immutability/integrity
    pub provenance_hash: String,

    /// Schema version for backward compatibility
    pub schema_version: u32,
}

impl Observation {
    /// Creates a new observation envelope with default validation states.
    /// This should be called by the Validation layer after a normalizer produces the raw payload.
    pub fn new(
        observation_type: String,
        source: String,
        observed_at: DateTime<Utc>,
        effective_from: DateTime<Utc>,
        raw_payload: Value,
        normalized_payload: Value,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            research_session_id: None,
            instrument_id: None,
            observation_type,
            source,
            source_identifier: None,
            observed_at,
            effective_from,
            effective_to: None,
            recorded_at: Utc::now(),
            raw_payload,
            normalized_payload,
            confidence: 1.0,
            freshness: 0.0,
            coverage: "Complete".to_string(),
            consistency: None,
            quality_score: 1.0,
            provenance_hash: String::new(), // To be filled by Validation layer
            schema_version: 1,
        }
    }
}
