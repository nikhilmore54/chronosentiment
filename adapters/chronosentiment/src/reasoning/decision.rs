use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use crate::reasoning::historical_reasoning::HistoricalReasoningReport;

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
        (self.evidence_quality + 
         self.evidence_agreement + 
         self.historical_reliability + 
         self.data_completeness + 
         self.model_stability) / 5.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Decision {
    pub decision_id: Uuid,
    pub evaluation_timestamp: DateTime<Utc>,
    pub instrument_id: Uuid,
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

use crate::reasoning::assessment::{AssessmentProfile, Direction};
use crate::metrics::concepts::Concept;

pub struct DecisionEngine;

impl DecisionEngine {
    pub fn evaluate(&self, profile: &AssessmentProfile, eval_dt: DateTime<Utc>, instrument_id: Uuid) -> Decision {
        // Baseline Decision Policy v1.0
        // A simple rule: if Trend is Bullish, opportunity is Positive.
        
        let mut opp = Opportunity::Neutral;
        for assessment in &profile.assessments {
            if assessment.concept == Concept::Trend && assessment.direction == Direction::Bullish {
                opp = Opportunity::Positive;
                break;
            } else if assessment.concept == Concept::Trend && assessment.direction == Direction::Bearish {
                opp = Opportunity::Negative;
                break;
            }
        }
        
        Decision {
            decision_id: Uuid::new_v4(),
            evaluation_timestamp: eval_dt,
            instrument_id,
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
