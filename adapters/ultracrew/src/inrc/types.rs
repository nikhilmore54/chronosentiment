use serde::{Deserialize, Serialize};

/// A single constraint violation detail for a nurse on a given day.
#[derive(Serialize, Deserialize, Clone)]
pub struct ViolationDetail {
    pub nurse_id: String,
    pub day: usize,
    pub constraint: String,
    pub actual: usize,
    pub required: usize,
}

/// Full validation report for a schedule against INRC constraints.
#[derive(Serialize, Deserialize, Clone)]
pub struct ValidationReport {
    pub max_consecutive_work_violations: usize,
    pub min_consecutive_work_violations: usize,
    pub min_days_off_violations: usize,
    pub max_days_off_violations: usize,
    pub forbidden_successions: usize,
    pub coverage_achieved: f64,
    pub is_legal: bool,
    pub details: Vec<ViolationDetail>,
}