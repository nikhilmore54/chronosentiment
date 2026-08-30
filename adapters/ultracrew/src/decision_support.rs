use serde::{Serialize, Deserialize};
use crate::optimization::ScheduleEvaluation;
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionMetrics {
    pub coverage: f64, // percentage of shifts covered (1.0 if feasible)
    pub fairness_penalty: f64, // SC1 penalty
    pub utilization: f64, // Not currently calculated in standard evaluation, we can approximate or use cost
    pub cost: f64, // Fitness
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RosterAlternative {
    pub id: String,
    pub metrics: DecisionMetrics,
    pub assignments: std::collections::HashMap<u64, u64>, // ShiftID -> WorkerID
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    pub recommended_id: String,
    pub why: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionResult {
    pub alternatives: Vec<RosterAlternative>,
    pub recommendation: Option<Recommendation>,
}

pub struct DecisionSupportEngine;

impl DecisionSupportEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn generate_decision_matrix(&self, candidates: Vec<ScheduleEvaluation>) -> DecisionResult {
        // 1. Feasibility Filter
        let feasible_candidates: Vec<ScheduleEvaluation> = candidates.into_iter()
            .filter(|eval| eval.is_valid && (eval.hc1_violations + eval.hc2_violations + eval.hc3_violations + eval.rest_violations) == 0)
            .collect();

        // 2. Structural Differentiation (Diversity filter)
        // We want 3-5 distinct options. We'll measure distance by differing assignments.
        let mut selected = Vec::new();
        
        for candidate in feasible_candidates {
            if selected.len() >= 5 {
                break;
            }
            if selected.is_empty() {
                selected.push(candidate);
                continue;
            }
            
            // Check distance against already selected
            let is_distinct = selected.iter().all(|s| {
                let diff_count = candidate.schedule.assignments.iter().filter(|(k, v)| {
                    s.schedule.assignments.get(k) != Some(*v)
                }).count();
                // Simple deterministic threshold: must differ by at least 5% of assignments
                let threshold = (candidate.schedule.assignments.len() as f64 * 0.05).max(1.0) as usize;
                diff_count >= threshold
            });

            if is_distinct {
                selected.push(candidate);
            }
        }

        let mut alternatives = Vec::new();
        let mut opt_label = 'A';
        for eval in selected {
            let id = format!("Option {}", opt_label);
            opt_label = (opt_label as u8 + 1) as char;

            let metrics = DecisionMetrics {
                coverage: 1.0, // Because they are feasible
                fairness_penalty: eval.fairness_penalty,
                utilization: 0.8, // Placeholder
                cost: eval.fitness,
            };

            alternatives.push(RosterAlternative {
                id,
                metrics,
                assignments: eval.schedule.assignments,
            });
        }

        // 3. Recommendation Engine
        let mut recommendation = None;
        if !alternatives.is_empty() {
            // Find the option with the lowest fairness penalty
            let mut best_idx = 0;
            let mut min_fairness = f64::MAX;
            for (i, alt) in alternatives.iter().enumerate() {
                if alt.metrics.fairness_penalty < min_fairness {
                    min_fairness = alt.metrics.fairness_penalty;
                    best_idx = i;
                }
            }

            let best_alt = &alternatives[best_idx];
            let mut why = vec![
                "Strictly meets all hard feasibility constraints (100% coverage).".to_string()
            ];

            // Compare against others
            for (i, alt) in alternatives.iter().enumerate() {
                if i != best_idx {
                    let fairness_diff = alt.metrics.fairness_penalty - best_alt.metrics.fairness_penalty;
                    if fairness_diff > 0.0 {
                        // Assuming baseline of ~1000 for fairness to get a %
                        let pct = if alt.metrics.fairness_penalty > 0.0 {
                            (fairness_diff / alt.metrics.fairness_penalty * 100.0) as i32
                        } else { 0 };
                        if pct > 0 {
                            why.push(format!("{}% better workload fairness balance than {}.", pct, alt.id));
                        }
                    }
                }
            }

            recommendation = Some(Recommendation {
                recommended_id: best_alt.id.clone(),
                why,
            });
        }

        DecisionResult {
            alternatives,
            recommendation,
        }
    }
}
