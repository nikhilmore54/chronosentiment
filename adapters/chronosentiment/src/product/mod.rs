//! ChronoSentiment Product Domain — v0.1
//!
//! This module defines the product layer: the contracts that translate
//! market intelligence (C3-002) and execution intelligence (Coralys v0)
//! into personalised portfolio recommendations for a specific user.
//!
//! ## Architecture
//!
//! ```text
//! C3-002 Direction
//!        │
//!        ▼
//! Coralys Execution Intent
//!        │
//!        ▼
//! PortfolioAllocationRequest  ←  UserProfile
//!        │                   ←  PortfolioContext
//!        ▼
//! AllocationEngine v0  (deterministic, transparent)
//!        │
//!        ▼
//! PortfolioRecommendation[]
//!        │
//!        ▼
//! API Adapter / UI / Notification
//! ```
//!
//! ## Separation of concerns
//!
//! - [`user_profile`]       — user preferences (budget, risk tolerance, horizon)
//! - [`portfolio_context`]  — point-in-time portfolio state (holdings, cash, exposure)
//! - [`recommendation`]     — product contracts (PortfolioAllocationRequest, PortfolioRecommendation)
//! - [`allocation_engine`]  — deterministic sizing logic (no market intelligence)
//! - [`recommendation_engine`] — orchestrates all contracts into Vec<PortfolioRecommendation>
//!
//! ## Invariant
//!
//! No module in this layer may alter C3-002 direction or Coralys execution parameters.
//! The allocation engine is a portfolio constraint layer, not a market intelligence layer.

pub mod allocation_engine;
pub mod portfolio_context;
pub mod recommendation;
pub mod recommendation_engine;
pub mod user_profile;

pub use allocation_engine::{AllocationEngine, ALLOCATION_ENGINE_VERSION};
pub use portfolio_context::{PortfolioContext, PortfolioContextError, PortfolioPosition};
pub use recommendation::{
    PortfolioAllocationRequest, PortfolioRecommendation, RecommendationAction,
};
pub use recommendation_engine::{
    PortfolioRecommendationEngine, RecommendationEngineError, RECOMMENDATION_ENGINE_VERSION,
};
pub use user_profile::{InvestmentHorizon, RiskTolerance, UserProfile, UserProfileError};
