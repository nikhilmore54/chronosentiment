pub trait Scenario: Send + Sync + 'static {}

pub trait Solution: Clone + Send + Sync + 'static {}

pub trait Outcome {
    type Sol: Solution;

    fn objectives(&self) -> &[f64];

    fn primary_objective(&self) -> f64 {
        self.objectives().first().copied().unwrap_or(0.0)
    }

    fn is_valid(&self) -> bool;

    fn solution(&self) -> &Self::Sol;
}

pub trait State: Send + Sync + 'static {}

pub trait Action: Send + Sync + 'static {}

pub mod analysis;
pub mod memory;
pub mod models;
pub mod operators;
pub mod pipeline;
pub mod telemetry;

pub use models::{
    DecisionLineage, DecisionProposal, EvaluationResult, MatchingResult, StateReference, Violation,
};
pub use pipeline::EvolutionaryPipeline;

pub type SimulationResult<S> = Result<S, String>;

pub trait DecisionPlugin {
    type State;
    type Evaluation;

    fn current_state(&self) -> Self::State;

    fn evaluate(&self, state: &Self::State) -> Self::Evaluation;

    fn simulate(
        &self,
        state: &Self::State,
        proposal: &DecisionProposal,
    ) -> SimulationResult<Self::State>;

    fn execute(&mut self, proposal: &DecisionProposal);
}

pub trait ReasoningEngine {
    type Plugin: DecisionPlugin;
    type Config;

    fn solve(
        &self,
        plugin: &mut Self::Plugin,
        config: &Self::Config,
    ) -> Result<Vec<DecisionProposal>, String>;
}
