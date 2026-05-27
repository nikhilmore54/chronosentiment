pub mod ensemble;
pub mod paper;
pub mod pipeline;
pub mod reco;
pub mod strategy_id;
pub mod strategy_ranking;

pub use edge_decay::*;
pub use edge_half_life_estimator::*;
pub use ensemble::*;
pub use paper::*;
pub use pipeline::*;
pub mod market_regime;
pub use reco::*;
pub use strategy_id::*;
pub use strategy_ranking::*;

// ==========================================
// SEMANTIC TRANSLATION BOUNDARY
// ==========================================
// We map the financially-loaded domain vocabulary back to the 
// neutral optimization substrate vocabulary.

pub type Strategy = chronosentiment_optimization::Candidate;
pub type StrategyEvaluation = chronosentiment_optimization::CandidateEvaluation;
pub type RankStats = chronosentiment_optimization::EvaluationMetrics;
pub type AlphaConsensus = chronosentiment_optimization::ConsensusMetric;
pub type MarketRegime = chronosentiment_optimization::ScenarioContext;
pub type DirectionArchetype = chronosentiment_optimization::BehavioralArchetype;
pub type OrderIntent = chronosentiment_optimization::ExecutionDirective;
pub type TradeRecommendation = chronosentiment_optimization::ExecutionProposal;

