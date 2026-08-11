use crate::reasoning::assessment::{AssessmentProfile, Direction};
use crate::metrics::concepts::Concept;
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq)]
pub enum EvidenceType {
    Observation,
    Assessment,
    Historical,
    Macro,
    Fundamental,
    Technical,
    Policy,
}

#[derive(Debug, Clone)]
pub struct EvidenceStatement {
    pub evidence_id: Uuid,
    pub assessment_id: Option<Uuid>,
    pub timestamp: DateTime<Utc>,
    pub concept: Concept,
    pub evidence_type: EvidenceType,
    pub description: String,
    pub confidence: f64,
    pub supports_continuation: bool,
}

#[derive(Debug, Clone)]
pub struct EvidenceSet {
    pub evidence: Vec<EvidenceStatement>,
}

pub struct EvidenceEngine;

impl EvidenceEngine {
    pub fn evaluate(&self, profile: &AssessmentProfile) -> EvidenceSet {
        let mut evidence_vec = Vec::new();
        let timestamp = Utc::now();

        for assessment in &profile.assessments {
            let desc = format!("{:?} is {:?}.", assessment.concept, assessment.direction);
            let supports = assessment.direction == Direction::Bullish || assessment.direction == Direction::Positive;
            
            evidence_vec.push(EvidenceStatement {
                evidence_id: Uuid::new_v4(),
                assessment_id: Some(Uuid::new_v4()), // Assuming Assessment would have an ID in reality
                timestamp,
                concept: assessment.concept.clone(),
                evidence_type: EvidenceType::Assessment,
                description: desc,
                confidence: assessment.confidence,
                supports_continuation: supports,
            });
        }

        EvidenceSet { evidence: evidence_vec }
    }
}
