pub mod models;
pub mod evaluator;
pub mod loader;
pub mod graph;
pub mod ecmp;
pub mod path;
pub mod moga_impl;
pub mod telemetry;

use coralys_core::{
    DecisionPlugin, DecisionProposal, EvaluationResult, Violation,
    StateReference, SimulationResult,
};
use uuid::Uuid;
use chrono::Utc;
use std::sync::Mutex;
use crate::models::Solution;
use crate::evaluator::RoadefEvaluator;

/// Simple state wrapper for the ROADEF adapter.
#[derive(Clone, Debug)]
pub struct RoadefState {
    pub solution: Solution,
    pub reference: StateReference,
}

/// Decision plugin implementing the frozen `DecisionPlugin` contract.
pub struct RoadefDecisionPlugin {
    pub evaluator: RoadefEvaluator,
    pub current_state: Mutex<RoadefState>,
}

impl RoadefDecisionPlugin {
    /// Construct a new plugin given a prepared evaluator.
    pub fn new(evaluator: RoadefEvaluator) -> Self {
        let empty_solution = Solution { srpaths: vec![] };
        let reference = StateReference {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            plugin: "roadef".to_string(),
            metadata: std::collections::HashMap::new(),
        };
        Self {
            evaluator,
            current_state: Mutex::new(RoadefState {
                solution: empty_solution,
                reference,
            }),
        }
    }
}

impl DecisionPlugin for RoadefDecisionPlugin {
    type State = RoadefState;
    type Evaluation = EvaluationResult;

    fn current_state(&self) -> Self::State {
        self.current_state.lock().unwrap().clone()
    }

    fn evaluate(&self, state: &Self::State) -> Self::Evaluation {
        let eval = self.evaluator.evaluate_solution(&state.solution);
        translate_evaluation(eval)
    }

    fn simulate(&self, _state: &Self::State, proposal: &DecisionProposal) -> SimulationResult<Self::State> {
        // Deserialize proposal payload into a ROADEF `Solution`
        let new_solution: Solution = serde_json::from_value(proposal.payload.clone())
            .map_err(|e| format!("Failed to deserialize ROADEF solution: {}", e))?;
        let new_reference = StateReference {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            plugin: "roadef".to_string(),
            metadata: std::collections::HashMap::new(),
        };
        Ok(RoadefState {
            solution: new_solution,
            reference: new_reference,
        })
    }

    fn execute(&mut self, proposal: &DecisionProposal) {
        if let Ok(new_state) = self.simulate(&self.current_state(), proposal) {
            *self.current_state.lock().unwrap() = new_state;
        }
    }
}

/// Convert the raw evaluator result into the platform `EvaluationResult`.
fn translate_evaluation(raw: crate::evaluator::EvaluationResult) -> EvaluationResult {
    let mut hard_violations = Vec::new();
    if !raw.valid {
        hard_violations.push(Violation {
            constraint_id: "roadef_valid".to_string(),
            severity: "Hard".to_string(),
            value: Some(0.0),
            expected: "true".to_string(),
            actual: "false".to_string(),
            description: "Solution failed validation".to_string(),
            penalty: 1_000_000,
        });
    }

    // The platform treats higher numbers as better; we negate the objective (which is a cost).
    let objectives = vec![-raw.obj];

    EvaluationResult {
        objectives,
        hard_constraint_violations: hard_violations,
        soft_constraint_violations: Vec::new(),
        metrics: {
            let mut map = std::collections::HashMap::new();
            map.insert("obj".to_string(), raw.obj);
            map
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::loader::{load_network, load_traffic_matrix, load_scenario};

    #[test]
    fn test_roadef_plugin_lifecycle() {
        // Load a tiny instance (set A-01 from the repository) for a quick sanity check.
        let net = load_network("repo/challenge-roadef-2026-main/setA/setA-01-net.json").unwrap();
        let tm = load_traffic_matrix("repo/challenge-roadef-2026-main/setA/setA-01-tm.json").unwrap();
        let scenario = load_scenario("repo/challenge-roadef-2026-main/setA/setA-01-scenario.json").unwrap();

        let evaluator = RoadefEvaluator::new(&net, tm, scenario);
        let mut plugin = RoadefDecisionPlugin::new(evaluator);

        let initial_state = plugin.current_state();
        assert_eq!(initial_state.reference.plugin, "roadef");

        let eval = plugin.evaluate(&initial_state);
        assert!(!eval.objectives.is_empty());

        // Create a trivial proposal using the current (empty) solution.
        let proposal = DecisionProposal {
            priority: 1.0,
            estimated_gain: 0.0,
            affected_resources: vec![],
            violations_resolved: vec![],
            confidence: 1.0,
            payload: serde_json::to_value(&initial_state.solution).unwrap(),
        };

        let simulated = plugin.simulate(&initial_state, &proposal).unwrap();
        assert_ne!(simulated.reference.id, initial_state.reference.id);
        assert!(simulated.solution.srpaths.is_empty());

        plugin.execute(&proposal);
        let post_state = plugin.current_state();
        assert_ne!(post_state.reference.id, initial_state.reference.id);
    }
}
