use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use super::violation::Violation;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationResult {
    pub objectives: Vec<f64>,
    pub hard_constraint_violations: Vec<Violation>,
    pub soft_constraint_violations: Vec<Violation>,
    pub metrics: HashMap<String, f64>,
}
