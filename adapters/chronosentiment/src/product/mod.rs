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
//! - [`intraday_decision`]  — IC v1 DecisionBrief assembly (adapter-owned; HTTP serializes it)
//! - [`paper_lifecycle`]    — shared TARGET/STOP/HORIZON evaluator (v0.2 walk, one observation at a time)
//! - [`paper_replay`]       — historical driver: frozen CSV blotter (does not re-walk)
//! - [`deferred_live`]      — live driver: MarketObservation → paper_lifecycle → live ledger
//! - [`deferred_live_loop`] — event-driven observation ingest / arming (no Yahoo fetch)
//! - [`deferred_live_decision_loop`] — Increment 1 sidecar: controlled-clock ingest → DecisionSurface (no new recommendation)
//! - [`deferred_live_asof_ic`] — Increment 2 sidecar: as-of IC assembler + shared live/cached ingest coordinator (no driver mutation)
//! - [`live_observation`]   — CACHED_1M / YAHOO_1M controlled clock (not a broker feed)
//! - [`live_reassess_experiment`] — opt-in sidecar: v0.2 path-shape + adverse-mark gate (does not mutate Stage C)
//!
//! ## Invariant
//!
//! No module in this layer may alter C3-002 direction or Coralys execution parameters.
//! The allocation engine is a portfolio constraint layer, not a market intelligence layer.

pub mod allocation_engine;
pub mod coralys_state_bridge;
pub mod deferred_live;
pub mod deferred_live_asof_ic;
pub mod deferred_live_decision_loop;
pub mod deferred_live_loop;
pub mod deferred_live_performance;
pub mod deferred_live_session;
pub mod e4_shadow_validator;
pub mod live_observation;
pub mod live_update_capture;
pub mod intraday_decision;
pub mod paper_lifecycle;
pub mod paper_replay;
pub mod portfolio_context;
pub mod recommendation;
pub mod recommendation_engine;
pub mod user_profile;

pub use allocation_engine::{AllocationEngine, ALLOCATION_ENGINE_VERSION};
pub use coralys_state_bridge::*;
pub use intraday_decision::{DecisionBrief, ExecutionFacts, load_intraday_briefs};
pub use deferred_live::{
    overlay_live_quote, DeferredLiveConfig, DeferredLiveDriver, LivePaperAction,
    LivePaperLedger, LivePaperPosition, LivePaperStatus, MarketObservation, OpenError,
    EXECUTION_MODE,
};
pub use deferred_live_loop::{
    cached_1m_tape, cached_1m_through_unix, cached_session_tape, ingest_tape, lifecycle_scenario_tape,
    observation_producer_from_env, observations_from_cached_bars, yahoo_1m_tape, CachedOhlcBar,
    DeferredLiveRuntime, IngestOutcome, LifecycleScenario, load_cached_1m_observations,
};
pub use deferred_live_decision_loop::{DecisionLoopRuntime, DecisionSurface};
pub use deferred_live_asof_ic::{
    offer_asof_brief, offer_asof_from_prefix, run_cached_session, session_tape_tickers,
    AsOfIcAssembler, AsOfOffer, AsOfPathFeatures, AsOfSessionDriver, AsOfSessionEvent,
    CachedSessionRun, WatchFixture, classify_h60, compute_time_safe_oqs, H60_BAR_COUNT,
};
pub use deferred_live_session::{
    eligible_for_auto_arm, select_session_briefs, SessionState,
};
pub use deferred_live_performance::{
    score_deferred_live, DeferredLivePerformance, DeferredLivePerformanceRow,
};
pub use live_observation::{
    ControlledObservationTape, ObservationFeedSnapshot, ObservationFeedStatus, ObservationProducer,
    ObservationSourceKind, SourcedObservation, fetch_yahoo_1m_bars, inter_bar_wait_millis,
    observations_from_yahoo_chart, parse_speed, today_ist_date,
};
pub use paper_lifecycle::{
    apply_observation, signed_return, walk_bars, HorizonPolicy, PaperObservation, PaperWalkState,
    V02_HORIZON_BAR, V02_START_BAR,
};
pub use paper_replay::{
    PaperBlotter, PaperPosition, assemble_paper_blotter, attach_decision_links, load_paper_positions,
};
pub use portfolio_context::{PortfolioContext, PortfolioContextError, PortfolioPosition};
pub use recommendation::{
    PortfolioAllocationRequest, PortfolioRecommendation, RecommendationAction,
};
pub use recommendation_engine::{
    PortfolioRecommendationEngine, RecommendationEngineError, RECOMMENDATION_ENGINE_VERSION,
};
pub use user_profile::{InvestmentHorizon, RiskTolerance, UserProfile, UserProfileError};
pub use e4_shadow_validator::{E4ShadowValidator, E4ShadowPosition};


