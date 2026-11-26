use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Violation {
    pub constraint_id: String,
    pub severity: String, // e.g. "Hard", "Soft"
    pub value: Option<f64>,
    pub expected: String,
    pub actual: String,
    pub description: String,
    pub penalty: i32,
}
