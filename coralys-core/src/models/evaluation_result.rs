use super::violation::Violation;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationResult {
    pub objectives: Vec<f64>,
    pub hard_constraint_violations: Vec<Violation>,
    pub soft_constraint_violations: Vec<Violation>,
    pub metrics: HashMap<String, f64>,
}
