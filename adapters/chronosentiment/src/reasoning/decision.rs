use crate::reasoning::historical_reasoning::HistoricalReasoningReport;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Opportunity {
    Positive,
    Neutral,
    Negative,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExpectedHorizon {
    Short,
    Medium,
    Long,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceDecomposition {
    pub evidence_quality: f64,
    pub evidence_agreement: f64,
    pub historical_reliability: f64,
    pub data_completeness: f64,
    pub model_stability: f64,
}

impl ConfidenceDecomposition {
    pub fn derived_overall(&self) -> f64 {
        (self.evidence_quality
            + self.evidence_agreement
            + self.historical_reliability
            + self.data_completeness
            + self.model_stability)
            / 5.0
    }
}

/// Knowledge Lake decision artifact (B3/B4 persist).
///
/// This is **not** `decision_support::TradingDecision`. Do not merge the two.
/// Product paths must emit `TradingDecision` via `DecisionPolicy`.
/// This type remains so historical dumps can be deserialized.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Decision {
    pub metadata: crate::repository::knowledge::ArtifactMetadata,
    pub decision_id: Uuid,
    pub evaluation_timestamp: DateTime<Utc>,
    pub instrument_id: Uuid,
    pub assessment_id: Uuid,
    pub universe: String,

    pub market_context_id: Option<Uuid>,
    pub evidence_ids: Vec<Uuid>,
    pub hypothesis_ids: Vec<Uuid>,
    pub scenario_ids: Vec<Uuid>,

    pub opportunity: Opportunity,
    pub confidence: ConfidenceDecomposition,

    pub opportunity_score: f64,
    pub quality_score: f64,
    pub expected_horizon: ExpectedHorizon,

    pub replay_context_hash: String,
    pub knowledge_lake_version: String,

    pub evaluation_profile_version: String,
    pub concept_model_version: String,
    pub metric_model_version: String,
    pub evidence_rule_version: String,
    pub assessment_engine_version: String,
    pub hypothesis_engine_version: String,
    pub validation_engine_version: String,
    pub decision_engine_version: String,
    pub scenario_projection_version: String,
}

#[cfg(feature = "legacy-lake")]
use crate::metrics::concepts::Concept;
#[cfg(feature = "legacy-lake")]
use crate::reasoning::assessment::{AssessmentProfile, Direction};

/// B3/B4 lake generator. Not a `DecisionPolicy`. Compiled only with `legacy-lake`.
/// Preserve behaviour (including fabricated 0.5 confidence). Do not repair it
/// to improve historical SHORT coverage.
#[cfg(feature = "legacy-lake")]
pub struct DecisionEngine;

#[cfg(feature = "legacy-lake")]
impl DecisionEngine {
    pub fn evaluate(
        &self,
        profile: &AssessmentProfile,
        eval_dt: DateTime<Utc>,
        instrument_id: Uuid,
    ) -> Decision {
        // Baseline Decision Policy v1.0
        // A simple rule: if Trend is Bullish, opportunity is Positive.

        let mut opp = Opportunity::Neutral;
        for assessment in &profile.assessments {
            if assessment.concept == Concept::Trend && assessment.direction == Direction::Bullish {
                opp = Opportunity::Positive;
                break;
            } else if assessment.concept == Concept::Trend
                && assessment.direction == Direction::Bearish
            {
                opp = Opportunity::Negative;
                break;
            }
        }

        let decision_id = Uuid::new_v4();
        let mut metadata = crate::repository::knowledge::ArtifactMetadata::mock();
        metadata.artifact_type = crate::repository::knowledge::ArtifactType::Decision;
        metadata.evaluation_timestamp = eval_dt;
        metadata.artifact_id = decision_id;

        Decision {
            metadata,
            decision_id,
            evaluation_timestamp: eval_dt,
            instrument_id,
            assessment_id: profile.metadata.artifact_id,
            universe: "Nifty50".to_string(),
            market_context_id: None,
            evidence_ids: vec![],
            hypothesis_ids: vec![],
            scenario_ids: vec![],
            opportunity: opp,
            confidence: ConfidenceDecomposition {
                evidence_quality: 0.5,
                evidence_agreement: 0.5,
                historical_reliability: 0.5,
                data_completeness: 0.5,
                model_stability: 0.5,
            },
            opportunity_score: 0.5,
            quality_score: 0.5,
            expected_horizon: ExpectedHorizon::Medium,
            replay_context_hash: "".to_string(),
            knowledge_lake_version: "baseline-v1.0".to_string(),
            evaluation_profile_version: "baseline-v1.0".to_string(),
            concept_model_version: "baseline-v1.0".to_string(),
            metric_model_version: "baseline-v1.0".to_string(),
            evidence_rule_version: "baseline-v1.0".to_string(),
            assessment_engine_version: "baseline-v1.0".to_string(),
            hypothesis_engine_version: "baseline-v1.0".to_string(),
            validation_engine_version: "baseline-v1.0".to_string(),
            decision_engine_version: "baseline-v1.0".to_string(),
            scenario_projection_version: "baseline-v1.0".to_string(),
        }
    }
}

impl crate::repository::knowledge::KnowledgeArtifact for Decision {
    fn metadata(&self) -> &crate::repository::knowledge::ArtifactMetadata {
        &self.metadata
    }

    fn instrument_id(&self) -> Option<Uuid> {
        Some(self.instrument_id)
    }
}
