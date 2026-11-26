use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionProposal {
    pub priority: f64,
    pub estimated_gain: f64,
    pub affected_resources: Vec<String>,
    pub violations_resolved: Vec<String>,
    pub confidence: f64,
    pub payload: serde_json::Value,
}
