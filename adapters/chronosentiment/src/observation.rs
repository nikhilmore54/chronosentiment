use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

/// A raw observation straight from the translator, before any validation or UUID assignments.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawObservation {
    pub observation_type: String,
    pub source: String,
    pub source_identifier: Option<String>,
    pub observed_at: DateTime<Utc>,
    pub raw_payload: Value,
    pub normalized_payload: Value,
}

/// A canonical observation produced by the validation layer.
/// It represents a single, immutable, trusted fact observed in the world.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatedObservation {
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
    pub freshness: f64,   // e.g., age in hours when recorded
    pub coverage: String, // 'Complete', 'Partial'
    pub consistency: Option<f64>,
    pub quality_score: f64, // Aggregated quality metric

    /// Cryptographic hash of the payload and source to ensure immutability/integrity
    pub provenance_hash: String,

    /// Schema version for backward compatibility
    pub schema_version: u32,
}
