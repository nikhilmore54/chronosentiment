//! Recommendation Engine — Coralys Decision Intelligence
//!
//! Converts a certified C3-002 decision into a versioned [`RecommendationRecord`]
//! by combining frozen HDV-001 historical evidence with the decision's own
//! geometric risk/reward parameters.
//!
//! # Architecture contract
//! - This module is the ONLY place recommendation logic lives.
//! - The UI receives a [`RecommendationRecord`] and displays it verbatim.
//! - No recommendation logic is duplicated in Next.js.
//! - Evidence classification thresholds are documented in [`evidence`] and versioned.
//! - Historical outcome rates are NOT presented as forward probabilities.

pub mod engine;
pub mod evidence;

pub use engine::{
    RECOMMENDATION_POLICY_VERSION_V1, RecommendationEngineV1, RecommendationRecordV1,
};
pub use engine::{RecommendationAction, RecommendationEngine, RecommendationRecord};
pub use evidence::{AnalogueKey, EvidenceClass, EvidenceStore, HistoricalEvidence};
pub use evidence::{
    DegradationLevel, MIN_V1_SAMPLE, Rec001hStore, V1Evidence, VolatilityRegime, VolumeRegime,
};
