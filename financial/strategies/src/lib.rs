pub mod domain;
pub use domain::*;
pub mod exit;
pub mod ensemble;
pub mod evaluation;
pub mod orchestration;
pub mod pipeline;
pub mod reco;
pub mod strategy_id;
pub mod strategy_ranking;
pub mod pnl_overlay;
pub mod replay_evaluator;

pub mod market_regime;
pub use strategy_id::*;

pub mod compatibility {
    pub use crate::pipeline::*;
    pub use crate::reco::*;
    pub use crate::strategy_ranking::*;
    pub use crate::evaluation::*;
}

// ==========================================
// SEMANTIC TRANSLATION BOUNDARY
// ==========================================
// We map the financially-loaded domain vocabulary back to the 
// neutral optimization substrate vocabulary.










