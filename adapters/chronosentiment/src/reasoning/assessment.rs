use crate::metrics::concepts::Concept;
use coralys_moga::runtime::optimization::metric::MetricReport;
use sha2::{Sha256, Digest};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FactorAvailability {
    Available,
    Unavailable,
}

/// Independent factor status at T. Missing metrics are UNAVAILABLE, not invented.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FactorStatus {
    pub concept: Concept,
    pub availability: FactorAvailability,
    pub supporting_metrics: Vec<String>,
    pub missing_metrics: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssessmentProfile {
    pub metadata: ArtifactMetadata,
    pub instrument_id: Option<Uuid>,
    pub assessments: Vec<DomainAssessment>,
    /// Always one row per requested concept. Empty on pre-enrichment B4 profiles (`serde` default).
    #[serde(default)]
    pub factor_status: Vec<FactorStatus>,
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
        let mut statuses = self.factor_status.clone();
        statuses.sort_by_key(|s| format!("{:?}", s.concept));
        for s in &statuses {
            parts.push(format!("{:?}:{:?}", s.concept, s.availability));
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

pub const ENRICHMENT_CONCEPTS: [Concept; 3] = [Concept::Trend, Concept::Momentum, Concept::Volatility];

pub struct AssessmentEngine;

impl AssessmentEngine {
    pub fn required_metrics(concept: &Concept) -> &'static [&'static str] {
        match concept {
            Concept::Trend => &["ma_20", "ma_50"],
            Concept::Momentum => &["roc_20"],
            Concept::Volatility => &["atr_14"],
            Concept::Liquidity => &["volume_20d"],
            _ => &[],
        }
    }

    /// Demo/test helper. Do not persist: `ArtifactMetadata::mock()` stamps wall-clock time.
    pub fn assess(&self, metrics: &MetricReport, active_concepts: &[Concept]) -> AssessmentProfile {
        self.assess_with_metadata(metrics, active_concepts, ArtifactMetadata::mock(), None)
    }

    /// Population path: `evaluation_timestamp` is replay as-of `T`.
    /// `created_at` remains persist wall-clock from `mock()` (recorded_at). Do not use `Utc::now()` as T.
    pub fn assess_at(
        &self,
        metrics: &MetricReport,
        active_concepts: &[Concept],
        evaluation_timestamp: DateTime<Utc>,
        instrument_id: Option<Uuid>,
    ) -> AssessmentProfile {
        let mut metadata = ArtifactMetadata::mock();
        metadata.evaluation_timestamp = evaluation_timestamp;
        self.assess_with_metadata(metrics, active_concepts, metadata, instrument_id)
    }

    pub fn assess_with_metadata(&self, metrics: &MetricReport, active_concepts: &[Concept], mut metadata: ArtifactMetadata, instrument_id: Option<Uuid>) -> AssessmentProfile {
        let mut assessments = Vec::new();
        let mut factor_status = Vec::new();

        for concept in active_concepts {
            let required = Self::required_metrics(concept);
            let missing: Vec<String> = required
                .iter()
                .filter(|m| metrics.get_float(*m).is_none())
                .map(|m| (*m).to_string())
                .collect();
            let supporting: Vec<String> = required
                .iter()
                .filter(|m| metrics.get_float(*m).is_some())
                .map(|m| (*m).to_string())
                .collect();

            if missing.is_empty() && !required.is_empty() {
                factor_status.push(FactorStatus {
                    concept: concept.clone(),
                    availability: FactorAvailability::Available,
                    supporting_metrics: supporting,
                    missing_metrics: vec![],
                });
                if let Some(assessment) = self.assess_concept(concept, metrics) {
                    assessments.push(assessment);
                }
            } else if required.is_empty() {
                factor_status.push(FactorStatus {
                    concept: concept.clone(),
                    availability: FactorAvailability::Unavailable,
                    supporting_metrics: vec![],
                    missing_metrics: vec!["no_certified_metrics".to_string()],
                });
            } else {
                factor_status.push(FactorStatus {
                    concept: concept.clone(),
                    availability: FactorAvailability::Unavailable,
                    supporting_metrics: supporting,
                    missing_metrics: missing,
                });
            }
        }
        factor_status.sort_by_key(|s| format!("{:?}", s.concept));
        assessments.sort_by_key(|a| format!("{:?}", a.concept));

        #[derive(Serialize)]
        struct Payload<'a> {
            assessments: &'a [DomainAssessment],
            factor_status: &'a [FactorStatus],
        }
        metadata.content_hash = crate::repository::hash::generate_content_hash(
            &Payload {
                assessments: &assessments,
                factor_status: &factor_status,
            },
            &metadata,
        );

        AssessmentProfile {
            metadata,
            instrument_id,
            assessments,
            factor_status,
        }
    }

    /// Semantic direction only when it does not require a new trading threshold.
    /// Volatility ATR is magnitude-only: AVAILABLE in `factor_status`, no invented High/Low.
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
            }
            Concept::Momentum => {
                let roc_20 = metrics.get_float("roc_20");
                roc_20.map(|roc| {
                    let dir = if roc > 0.0 {
                        Direction::Positive
                    } else {
                        Direction::Negative
                    };
                    DomainAssessment {
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
                    }
                })
            }
            Concept::Volatility => None,
            _ => None,
        }
    }
}
