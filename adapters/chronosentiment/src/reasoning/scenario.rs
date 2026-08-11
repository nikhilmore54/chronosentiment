use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScenarioStatus {
    Proposed,
    PolicyRejected,
    Selected,
    Rejected,
}

/// The core aggregate for decision making. The Decision Engine never compares hypotheses, it compares scenarios.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scenario {
    pub scenario_id: String,
    pub research_session_id: String,
    
    // Core attributes mapped from hypothesis
    pub hypothesis_ids: Vec<String>,
    pub expected_return: f64,
    pub expected_risk: f64,
    pub expected_drawdown: f64,
    pub expected_probability: f64,
    
    // Status
    pub status: ScenarioStatus,
    pub description: String,
}
