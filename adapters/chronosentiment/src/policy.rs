use serde::{Deserialize, Serialize};

/// Represents a point-in-time snapshot of the user's trading or risk policies.
/// This allows the Replay Engine to answer: "Would this decision have been legal under my policy at the time?"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicySnapshot {
    pub snapshot_id: String,
    pub timestamp: u64,
    pub rules: Vec<PolicyRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRule {
    pub rule_id: String,
    pub description: String,
    pub parameters: std::collections::HashMap<String, String>,
}
