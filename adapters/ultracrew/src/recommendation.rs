use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SchedulingRecommendation {
    pub constraint_id: String,
    pub severity: String, // "Hard" or "Soft" or "Warning"
    pub explanation: String,
    pub recommended_action: String,
}

pub struct RecommendationEngine;

impl RecommendationEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn generate_recommendations(&self, report: &crate::constraint_engine::ConstraintReport) -> Vec<SchedulingRecommendation> {
        let mut recs = Vec::new();

        // HC1: Skills
        if report.hc1_violations > 0 {
            recs.push(SchedulingRecommendation {
                constraint_id: "HC1".to_string(),
                severity: "Hard".to_string(),
                explanation: format!("There are {} shift assignments where the worker does not possess the required skill.", report.hc1_violations),
                recommended_action: "Reassign the shifts to workers who possess the required skill, or update the worker's qualified skills.".to_string(),
            });
        }

        // HC2: Double Booking
        if report.hc2_violations > 0 {
            recs.push(SchedulingRecommendation {
                constraint_id: "HC2".to_string(),
                severity: "Hard".to_string(),
                explanation: format!("There are {} instances of overlapping shifts assigned to the same worker (double booking).", report.hc2_violations),
                recommended_action: "Move one of the overlapping shifts to another available worker to resolve the schedule conflict.".to_string(),
            });
        }

        // HC3: Max Hours
        if report.hc3_violations > 0 {
            recs.push(SchedulingRecommendation {
                constraint_id: "HC3".to_string(),
                severity: "Hard".to_string(),
                explanation: format!("There are {} instances where a worker exceeds the maximum weekly hours limit (40 hours).", report.hc3_violations),
                recommended_action: "Reassign one or more shifts from the overloaded worker to under-utilized workers.".to_string(),
            });
        }

        // Rest: Rest Period Violations
        if report.rest_violations > 0 {
            recs.push(SchedulingRecommendation {
                constraint_id: "Rest".to_string(),
                severity: "Hard".to_string(),
                explanation: format!("There are {} instances where a worker has less than the minimum required 8 hours of rest between consecutive shifts.", report.rest_violations),
                recommended_action: "Adjust shift assignments to ensure there is a gap of at least 8 hours between consecutive shifts for each worker.".to_string(),
            });
        }

        // SC1: Fairness
        if report.fairness_penalty > 100.0 {
            recs.push(SchedulingRecommendation {
                constraint_id: "SC1".to_string(),
                severity: "Soft".to_string(),
                explanation: format!("High workload imbalance detected (Fairness penalty: {:.2}).", report.fairness_penalty),
                recommended_action: "Redistribute shifts to balance working hours more equitably across the workforce.".to_string(),
            });
        }

        // SC2: Fatigue
        if report.fatigue_penalty > 100.0 {
            recs.push(SchedulingRecommendation {
                constraint_id: "SC2".to_string(),
                severity: "Soft".to_string(),
                explanation: format!("High cumulative fatigue penalty detected (Fatigue penalty: {:.2}).", report.fatigue_penalty),
                recommended_action: "Assign shifts with high workload impact to workers with lower historical workload accumulation.".to_string(),
            });
        }

        // General warnings
        for warning in &report.warnings {
            recs.push(SchedulingRecommendation {
                constraint_id: "Warning".to_string(),
                severity: "Warning".to_string(),
                explanation: warning.clone(),
                recommended_action: "Review worker assignment density and evaluate if shifts can be shifted or split.".to_string(),
            });
        }

        recs
    }
}
