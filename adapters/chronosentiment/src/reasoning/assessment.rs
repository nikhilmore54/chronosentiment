use crate::metrics::concepts::Concept;
use coralys_moga::runtime::optimization::metric::MetricReport;
use sha2::{Sha256, Digest};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::repository::knowledge::{ArtifactMetadata, KnowledgeArtifact};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Direction {
    Bullish,
    Bearish,
    Neutral,
    Positive,
    Negative,
    Supportive,
    Expensive,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Strength {
    Strong,
    Moderate,
    Weak,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Maturity {
    Early,
    Mid,
    Late,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Persistence {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainAssessment {
    pub concept: Concept,
    pub direction: Direction,
    pub strength: Option<Strength>,
    pub maturity: Option<Maturity>,
    pub persistence: Option<Persistence>,
    pub confidence: f64,
    pub uncertainty: f64,
    pub uncertainty_reason: Option<String>,
    pub supporting_metrics: Vec<String>,
    pub contradicting_metrics: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssessmentProfile {
    pub metadata: ArtifactMetadata,
    pub instrument_id: Option<Uuid>,
    pub assessments: Vec<DomainAssessment>,
}

impl KnowledgeArtifact for AssessmentProfile {
    fn metadata(&self) -> &ArtifactMetadata {
        &self.metadata
    }
    
    fn instrument_id(&self) -> Option<Uuid> {
        self.instrument_id
    }
}

impl AssessmentProfile {
    pub fn to_signature(&self) -> String {
        let mut parts = Vec::new();
        let mut assessments = self.assessments.clone();
        assessments.sort_by_key(|a| format!("{:?}", a.concept));
        
        for a in &assessments {
            let mut s = format!("{:?}: {:?}", a.concept, a.direction);
            if let Some(strength) = &a.strength {
                s.push_str(&format!(" / {:?}", strength));
            }
            if let Some(maturity) = &a.maturity {
                s.push_str(&format!(" / {:?}", maturity));
            }
            if let Some(persistence) = &a.persistence {
                s.push_str(&format!(" / {:?}", persistence));
            }
            parts.push(s);
        }
        
        if parts.is_empty() {
            "Neutral / Weak".to_string()
        } else {
            parts.join(" | ")
        }
    }
    
    pub fn to_hash(&self) -> String {
        let sig = self.to_signature();
        let mut hasher = Sha256::new();
        hasher.update(sig.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}

pub struct AssessmentEngine;

impl AssessmentEngine {
    pub fn assess(&self, metrics: &MetricReport, active_concepts: &[Concept]) -> AssessmentProfile {
        self.assess_with_metadata(metrics, active_concepts, ArtifactMetadata::mock(), None)
    }

    pub fn assess_with_metadata(&self, metrics: &MetricReport, active_concepts: &[Concept], mut metadata: ArtifactMetadata, instrument_id: Option<Uuid>) -> AssessmentProfile {
        let mut assessments = Vec::new();

        for concept in active_concepts {
            if let Some(assessment) = self.assess_concept(concept, metrics) {
                assessments.push(assessment);
            }
        }
        
        metadata.content_hash = crate::repository::hash::generate_content_hash(&assessments, &metadata);

        AssessmentProfile { metadata, instrument_id, assessments }
    }

    fn assess_concept(&self, concept: &Concept, metrics: &MetricReport) -> Option<DomainAssessment> {
        match concept {
            Concept::Trend => {
                let ma_20 = metrics.get_float("ma_20");
                let ma_50 = metrics.get_float("ma_50");
                
                if let (Some(m20), Some(m50)) = (ma_20, ma_50) {
                    let (dir, conf, uncert) = if m20 > m50 {
                        (Direction::Bullish, 0.82, 0.18)
                    } else {
                        (Direction::Bearish, 0.82, 0.18)
                    };
                    
                    Some(DomainAssessment {
                        concept: Concept::Trend,
                        direction: dir,
                        strength: Some(Strength::Strong),
                        maturity: Some(Maturity::Early),
                        persistence: Some(Persistence::Medium),
                        confidence: conf,
                        uncertainty: uncert,
                        uncertainty_reason: Some("Insufficient volume history".to_string()),
                        supporting_metrics: vec!["ma_20".to_string(), "ma_50".to_string()],
                        contradicting_metrics: vec![],
                    })
                } else {
                    None
                }
            },
            Concept::Momentum => {
                let roc_20 = metrics.get_float("roc_20");
                if let Some(roc) = roc_20 {
                    let dir = if roc > 0.0 { Direction::Positive } else { Direction::Negative };
                    Some(DomainAssessment {
                        concept: Concept::Momentum,
                        direction: dir,
                        strength: Some(Strength::Moderate),
                        maturity: None,
                        persistence: None,
                        confidence: 0.73,
                        uncertainty: 0.27,
                        uncertainty_reason: None,
                        supporting_metrics: vec!["roc_20".to_string()],
                        contradicting_metrics: vec![],
                    })
                } else {
                    None
                }
            },
            // ... (other concepts omitted for brevity in mock)
            _ => None,
        }
    }
}
