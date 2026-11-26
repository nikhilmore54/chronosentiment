use uuid::Uuid;
use crate::hypothesis::InvestmentThesis;
use crate::validation::context::MarketEvaluationContext as EvaluationContext;
use crate::reasoning::scenario::{Scenario, ScenarioStatus};

pub struct ChronoScenarioEngine;

impl ChronoScenarioEngine {
    /// Generates competing scenarios based on the generated hypotheses and the current context.
    /// It always generates baseline scenarios (e.g., Hold Cash) to compare against.
    pub fn generate_scenarios(&self, context: &EvaluationContext, hypotheses: &[InvestmentThesis]) -> Vec<Scenario> {
        let mut scenarios = Vec::new();

        // 1. Primary Scenarios from Hypotheses
        for thesis in hypotheses {
            let is_bullish = thesis.summary().contains("outperform");
            
            let scenario = Scenario {
                scenario_id: Uuid::new_v4().to_string(),
                research_session_id: context.research_session_id.clone(),
                hypothesis_ids: vec![thesis.thesis_id.clone()],
                expected_return: if is_bullish { 0.15 } else { -0.10 }, // Dummy projections
                expected_risk: 0.12,
                expected_drawdown: 0.05,
                expected_probability: 0.65,
                status: ScenarioStatus::Proposed,
                description: if is_bullish {
                    "Execute primary bullish thesis (Buy)".to_string()
                } else {
                    "Execute primary bearish thesis (Sell/Short)".to_string()
                },
            };
            scenarios.push(scenario);
        }

        // 2. Baseline Scenario: Hold Cash (Wait)
        scenarios.push(Scenario {
            scenario_id: Uuid::new_v4().to_string(),
            research_session_id: context.research_session_id.clone(),
            hypothesis_ids: vec![],
            expected_return: 0.0,
            expected_risk: 0.0,
            expected_drawdown: 0.0,
            expected_probability: 1.0,
            status: ScenarioStatus::Proposed,
            description: "Baseline: Hold Cash / Wait".to_string(),
        });

        // 3. Baseline Scenario: Nifty ETF (Passive)
        scenarios.push(Scenario {
            scenario_id: Uuid::new_v4().to_string(),
            research_session_id: context.research_session_id.clone(),
            hypothesis_ids: vec![],
            expected_return: 0.10,
            expected_risk: 0.15,
            expected_drawdown: 0.20,
            expected_probability: 0.5,
            status: ScenarioStatus::Proposed,
            description: "Baseline: Buy Nifty ETF".to_string(),
        });

        scenarios
    }
}
