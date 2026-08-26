pub mod domain;
pub use domain::*;
pub mod ensemble;
pub mod evaluation;
pub mod exit;
pub mod orchestration;
pub mod pipeline;
pub mod pnl_overlay;
pub mod reco;
pub mod replay_evaluator;
pub mod strategy_id;
pub mod strategy_ranking;

pub mod market_regime;
pub use strategy_id::*;

pub mod compatibility {
    pub use crate::evaluation::*;
    pub use crate::pipeline::*;
    pub use crate::reco::*;
    pub use crate::strategy_ranking::*;
}

// ==========================================
// SEMANTIC TRANSLATION BOUNDARY
// ==========================================
// We map the financially-loaded domain vocabulary back to the
// neutral optimization substrate vocabulary.
