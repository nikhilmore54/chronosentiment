//! RP-608 NSE market-event path (P0 ingest + P1 OF/depth linkage).
//!
//! Behavioural reference:
//! - `scripts/rp608_kite_ticker_cache.decode_full_mode_packet`
//! - `scripts/rp608_nse_market_event_ref.py`
//! - `scripts/rp608_order_flow_mark.py` (±0.30 frozen)
//!
//! Live order authority: 0%. No production cutover. No OF threshold changes.

pub mod decode;
pub mod depth_journal;
pub mod engine;
pub mod event;
pub mod of_mark;
pub mod paper_obs;
pub mod push_eval;
pub mod stale;

pub use decode::{decode_full_mode_frame, decode_full_mode_packet, DecodeError};
pub use depth_journal::{depth_snapshot_from_event, DepthSnapshotRecord};
pub use engine::{AcceptedEvent, EngineConfig, LinkedObservation, MarketEventEngine};
pub use event::{DepthLevel, MarketEvent, SourceProvenance};
pub use of_mark::{
    classify_order_flow_support, inspect_from_market_event, inspect_order_flow_book,
    is_order_flow_exit_signal, l1_imbalance, micro_vs_mid_bps, OfClassifyResult, OfInspection,
    OfMarkRecord, OfSide, ORDER_FLOW_EXIT_LABEL, ORDER_FLOW_IMBALANCE_THRESHOLD, ORDER_FLOW_LABEL,
};
pub use paper_obs::{
    close_allowed, OpenPaperObs, PaperLifecycleObsEngine, PaperLifecycleObservation,
    PaperObsAction, OF_PAPER_MIN_HOLD_MS, OF_PAPER_OPPOSITE_PERSIST_MS,
};
pub use push_eval::{
    evaluate_recommendation, rank_watchlist, session_return_bps, traj15_live, traj15_watch_eod,
    watch_score, PushEvalEngine, QuoteFlags, RecommendationEval, WatchCandidateInput,
    WatchlistEntry, MAX_NEW_ENTRY_BAR, MANDATORY_SQUARE_OFF_BAR, MIN_BARS_FOR_ENTRY,
    TRAJ15_ENTRY_THRESHOLD, WATCHLIST_TOP_N,
};
pub use stale::{FeedHealth, FeedStatus, StaleReason};
