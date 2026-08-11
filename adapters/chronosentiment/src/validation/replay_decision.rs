use chrono::{DateTime, Utc};
use std::collections::HashSet;
use uuid::Uuid;
use crate::reasoning::assessment::AssessmentProfile;
use crate::reasoning::evidence::EvidenceSet;
use crate::reasoning::historical_reasoning::HistoricalReasoningReport;
use crate::reasoning::hypothesis::CompetingHypotheses;
use crate::reasoning::decision::Decision;
use crate::reasoning::strategy::OpportunityStrategy;

/// The central artifact of Phase 4.1 to 4.3: Decision Replay
/// This captures exactly what the system knew and decided at timestamp `T`.
#[derive(Debug, Clone)]
pub struct DecisionReplay {
    pub decision_id: Uuid,
    pub evaluation_timestamp: DateTime<Utc>,
    pub replay_context_hash: String,
    
    // The frozen reasoning chain at time T
    pub assessment_profile: AssessmentProfile,
    pub evidence_set: EvidenceSet,
    pub historical_reasoning_report: HistoricalReasoningReport,
    pub hypotheses: CompetingHypotheses,
    pub decision: Decision,
    pub strategy: OpportunityStrategy,
}

/// The Temporal Firewall ensures no future data can leak into Knowledge(T)
pub struct TemporalFirewall {
    pub evaluation_timestamp: DateTime<Utc>,
}

impl TemporalFirewall {
    pub fn new(evaluation_timestamp: DateTime<Utc>) -> Self {
        Self { evaluation_timestamp }
    }

    /// Verifies that a given observation or historical case occurred BEFORE the evaluation timestamp.
    /// Panics if a temporal violation is detected, preserving the integrity of the Time Machine.
    pub fn assert_historical(&self, timestamp: DateTime<Utc>, source: &str) {
        if timestamp > self.evaluation_timestamp {
            panic!(
                "TEMPORAL VIOLATION: Attempted to access future knowledge from '{}' (Data time: {}, Firewall time: {})",
                source, timestamp, self.evaluation_timestamp
            );
        }
    }
}
