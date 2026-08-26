//! Pluggable decision policy.
//!
//! ChronoSentiment evaluates a caller-supplied `DecisionPolicy`. It does not
//! invent or silently select one. Coralys may later supply a discovered
//! artifact that implements this trait.
//!
//! `BaselineTrendMappingPolicy` is a historical fixture, not a promoted strategy.

use chrono::{DateTime, Utc};

use crate::metrics::concepts::Concept;
use crate::reasoning::assessment::{
    AssessmentProfile, Direction, FactorAvailability, FactorStatus,
};

pub const BASELINE_TREND_MAPPING_POLICY_NAME: &str = "baseline.trend_mapping.v0";
pub const TREND_MAPPING_RULE: &str =
    "Trend.Bullish→LONG; Trend.Bearish→SHORT; Trend.other→NO_TRADE; Trend.absent→NO_TRADE";

use super::{DecisionAction, EvidenceFactor};

pub struct PolicyDecision {
    pub action: DecisionAction,
    pub mapping_rule: String,
    /// Stable action reason used for decision identity. Must not mention unused factors.
    pub action_reason: String,
    pub diagnostics: String,
    pub evidence_refs: Vec<String>,
    pub factors: Vec<EvidenceFactor>,
    pub consumed_concepts: Vec<String>,
}

pub trait DecisionPolicy: Send + Sync {
    fn name(&self) -> &str;
    fn decide(&self, assessment: &AssessmentProfile, as_of: DateTime<Utc>) -> PolicyDecision;
}

/// Historical/product baseline fixture — not a promoted trading policy.
///
/// Reproducible only when the caller selects it explicitly. Not Decision Engine
/// v1.0. Not a Coralys-discovered candidate. Assessment numeric scores are not
/// treated as calibrated decision confidence.
pub struct BaselineTrendMappingPolicy;

impl DecisionPolicy for BaselineTrendMappingPolicy {
    fn name(&self) -> &str {
        BASELINE_TREND_MAPPING_POLICY_NAME
    }

    fn decide(&self, assessment: &AssessmentProfile, _as_of: DateTime<Utc>) -> PolicyDecision {
        let mut factors = factors_from_profile(assessment);
        let mut trend_action = None;
        let mut trend_direction = None;

        for a in &assessment.assessments {
            if a.concept == Concept::Trend {
                trend_direction = Some(format!("{:?}", a.direction));
                trend_action = Some(match a.direction {
                    Direction::Bullish => DecisionAction::Long,
                    Direction::Bearish => DecisionAction::Short,
                    _ => DecisionAction::NoTrade,
                });
            }
        }

        ensure_factor(&mut factors, "Trend");
        ensure_factor(&mut factors, "Momentum");
        ensure_factor(&mut factors, "Volatility");

        let action = trend_action.unwrap_or(DecisionAction::NoTrade);
        let action_reason = match (&trend_direction, action) {
            (Some(dir), DecisionAction::Long) => format!("Trend {dir} → LONG by mapping_rule"),
            (Some(dir), DecisionAction::Short) => format!("Trend {dir} → SHORT by mapping_rule"),
            (Some(dir), DecisionAction::NoTrade) => {
                format!("Trend {dir} → NO_TRADE by mapping_rule")
            }
            (None, _) => "Trend absent → NO_TRADE by mapping_rule".to_string(),
        };
        let consumed: Vec<String> = factors
            .iter()
            .filter(|f| f.present)
            .map(|f| {
                if let Some(dir) = &f.direction {
                    format!("{}={dir}", f.concept)
                } else {
                    format!("{}=available", f.concept)
                }
            })
            .collect();
        let missing: Vec<&str> = factors
            .iter()
            .filter(|f| !f.present)
            .map(|f| f.concept.as_str())
            .collect();
        let diagnostics = format!(
            "{action_reason}. Policy={}. Observed: {}. Unavailable: {}. Decision confidence UNAVAILABLE. Consumed concepts: Trend only.",
            self.name(),
            if consumed.is_empty() {
                "none".to_string()
            } else {
                consumed.join(", ")
            },
            if missing.is_empty() {
                "none".to_string()
            } else {
                missing.join(", ")
            }
        );
        let identity_refs = vec![
            self.name().to_string(),
            trend_direction
                .clone()
                .unwrap_or_else(|| "Trend=absent".to_string()),
        ];

        PolicyDecision {
            action,
            mapping_rule: TREND_MAPPING_RULE.to_string(),
            action_reason,
            diagnostics,
            evidence_refs: identity_refs,
            factors,
            consumed_concepts: vec!["Trend".to_string()],
        }
    }
}

pub(super) fn factors_from_profile(profile: &AssessmentProfile) -> Vec<EvidenceFactor> {
    let mut out: Vec<EvidenceFactor> = profile.factor_status.iter().map(status_to_factor).collect();
    for a in &profile.assessments {
        let name = format!("{:?}", a.concept);
        if let Some(existing) = out.iter_mut().find(|f| f.concept == name) {
            existing.present = true;
            existing.direction = Some(format!("{:?}", a.direction));
            existing.strength = a.strength.as_ref().map(|s| format!("{s:?}"));
            // Fabricated assessment scores (e.g. 0.82) are not product evidence.
            existing.assessment_confidence = None;
        } else {
            out.push(EvidenceFactor {
                concept: name,
                present: true,
                direction: Some(format!("{:?}", a.direction)),
                strength: a.strength.as_ref().map(|s| format!("{s:?}")),
                assessment_confidence: None,
            });
        }
    }
    out.sort_by(|a, b| a.concept.cmp(&b.concept));
    out
}

fn status_to_factor(status: &FactorStatus) -> EvidenceFactor {
    EvidenceFactor {
        concept: format!("{:?}", status.concept),
        present: status.availability == FactorAvailability::Available,
        direction: None,
        strength: None,
        assessment_confidence: None,
    }
}

pub(super) fn ensure_factor(factors: &mut Vec<EvidenceFactor>, concept: &str) {
    if factors.iter().any(|f| f.concept == concept) {
        return;
    }
    factors.push(EvidenceFactor {
        concept: concept.to_string(),
        present: false,
        direction: None,
        strength: None,
        assessment_confidence: None,
    });
}
