//! Canonical market-event types for NSE Kite full-mode ticks.

use serde::{Deserialize, Serialize};

/// Five-level book entry (displayed depth).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct DepthLevel {
    pub price: f64,
    pub quantity: f64,
}

/// Provenance of the market event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceProvenance {
    /// Decoded from Kite WebSocket full-mode binary.
    KiteWsFull,
    /// Synthetic / recorded fixture replay.
    FixtureReplay,
    /// Post-disconnect reconcile path (not steady-state polling).
    Reconcile,
}

impl SourceProvenance {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::KiteWsFull => "kite_ws_full",
            Self::FixtureReplay => "fixture_replay",
            Self::Reconcile => "reconcile",
        }
    }
}

/// Canonical decoded + annotated market event (LTP, volume, L1–L5).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MarketEvent {
    pub instrument_token: u32,
    pub last_price: f64,
    pub volume: f64,
    pub exchange_timestamp_sec: u32,
    pub exchange_timestamp_utc_ms: u64,
    pub receipt_utc_ms: u64,
    pub quote_age_ms: f64,
    pub best_bid: f64,
    pub best_ask: f64,
    pub bid_qty: f64,
    pub ask_qty: f64,
    pub depth_buy: [DepthLevel; 5],
    pub depth_sell: [DepthLevel; 5],
    pub source: SourceProvenance,
    /// Monotonic ingest sequence assigned by the engine (ordering / dedupe).
    pub ingest_seq: u64,
    /// Content fingerprint used for deduplication.
    pub content_fp: u64,
}

impl MarketEvent {
    pub fn exchange_timestamp_utc_ms_from_sec(sec: u32) -> u64 {
        u64::from(sec).saturating_mul(1000)
    }
}
