// Internal representation of a generated schedule solution.
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleSolution {
    /// Mapping from shift ID to assigned worker ID.
    pub assignments: HashMap<u64, u64>,
    /// Overall fitness score calculated by the optimizer.
    pub fitness: f64,
    /// Number of constraint violations (hard constraints).
    pub hard_violations: usize,
    /// Fairness penalty (variance of hours).
    pub fairness_penalty: f64,
    /// Fatigue penalty based on historical ecology data.
    pub fatigue_penalty: f64,
    /// Rest period violations count.
    pub rest_violations: usize,
    /// Optional generated recommendations for resolving violations.
    pub recommendations: Option<Vec<crate::recommendation::SchedulingRecommendation>>,
    /// Optional optimization/observatory convergence telemetry.
    pub telemetry: Option<crate::optimization::OptimizationReport>,
}

impl ScheduleSolution {
    /// Construct a solution from a `ScheduleEvaluation`.
    pub fn from_evaluation(eval: &crate::optimization::ScheduleEvaluation) -> Self {
        Self {
            assignments: eval.schedule.assignments.clone(),
            fitness: eval.fitness,
            hard_violations: if eval.is_valid { 0 } else { 1 },
            fairness_penalty: eval.fairness_penalty,
            fatigue_penalty: eval.fatigue_penalty,
            rest_violations: eval.rest_violations,
            recommendations: None,
            telemetry: None,
        }
    }
}
