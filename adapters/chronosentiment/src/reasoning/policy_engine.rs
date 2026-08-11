use crate::policy::PolicySnapshot;
use crate::reasoning::scenario::{Scenario, ScenarioStatus};

pub struct ChronoPolicyEngine;

impl ChronoPolicyEngine {
    /// Evaluates scenarios against the active policies.
    /// E.g. "Maximum drawdown = 15%". If a scenario expects 20%, it is rejected.
    pub fn evaluate_scenarios(&self, mut scenarios: Vec<Scenario>, policy: &Option<PolicySnapshot>) -> Vec<Scenario> {
        let max_drawdown_allowed = if let Some(p) = policy {
            p.rules.iter()
                .find(|r| r.rule_id == "max_drawdown")
                .and_then(|r| r.parameters.get("limit"))
                .and_then(|v| v.parse::<f64>().ok())
                .unwrap_or(0.15) // default 15%
        } else {
            0.15 // default
        };

        for scenario in &mut scenarios {
            if scenario.expected_drawdown > max_drawdown_allowed {
                scenario.status = ScenarioStatus::PolicyRejected;
                // Prepend reason to description
                scenario.description = format!("[REJECTED BY POLICY (Drawdown {:.1}% > {:.1}%)] {}", 
                    scenario.expected_drawdown * 100.0, 
                    max_drawdown_allowed * 100.0,
                    scenario.description
                );
            }
        }

        scenarios
    }
}
