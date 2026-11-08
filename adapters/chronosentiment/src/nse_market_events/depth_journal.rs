//! Durable timestamped L1–L5 depth snapshots (not latest-value-only).
//!
//! Field shapes align with `scripts/rp608_nse_of_book_depth_history.snapshot_row_from_tick`
//! for differential comparison (protocol_id preserved).

use serde::{Deserialize, Serialize};

use super::event::{DepthLevel, MarketEvent};

pub const PROTOCOL_ID: &str = "RP-608-NSE-OF-BOOK-INTELLIGENCE-001";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DepthSnapshotRecord {
    pub protocol_id: String,
    pub record_type: String,
    pub symbol: String,
    pub instrument_token: u32,
    pub sample_utc_ms: u64,
    pub exchange_timestamp_sec: u32,
    pub exchange_timestamp_utc_ms: u64,
    pub receipt_utc_ms: u64,
    pub quote_age_ms: f64,
    pub stale: bool,
    pub missing_level_qty: bool,
    pub bid_levels_qty_nonzero: u32,
    pub ask_levels_qty_nonzero: u32,
    pub last_price: f64,
    pub best_bid: f64,
    pub best_ask: f64,
    pub bid_qty: f64,
    pub ask_qty: f64,
    pub volume: f64,
    pub depth_buy: Vec<DepthLevel>,
    pub depth_sell: Vec<DepthLevel>,
    pub quote_source: String,
    pub ingest_seq: u64,
    pub content_fp: u64,
}

fn round4(x: f64) -> f64 {
    (x * 10_000.0).round() / 10_000.0
}

fn levels_rounded(levels: &[DepthLevel; 5]) -> Vec<DepthLevel> {
    levels
        .iter()
        .map(|l| DepthLevel {
            price: round4(l.price),
            quantity: round4(l.quantity),
        })
        .collect()
}

pub fn depth_snapshot_from_event(
    ev: &MarketEvent,
    symbol: &str,
    sample_utc_ms: u64,
) -> DepthSnapshotRecord {
    let depth_buy = levels_rounded(&ev.depth_buy);
    let depth_sell = levels_rounded(&ev.depth_sell);
    let bid_nz = depth_buy.iter().filter(|l| l.quantity > 0.0).count() as u32;
    let ask_nz = depth_sell.iter().filter(|l| l.quantity > 0.0).count() as u32;
    let stale = ev.quote_age_ms > 60_000.0;
    DepthSnapshotRecord {
        protocol_id: PROTOCOL_ID.to_string(),
        record_type: "depth_snapshot_l1_l5".to_string(),
        symbol: symbol.to_string(),
        instrument_token: ev.instrument_token,
        sample_utc_ms,
        exchange_timestamp_sec: ev.exchange_timestamp_sec,
        exchange_timestamp_utc_ms: ev.exchange_timestamp_utc_ms,
        receipt_utc_ms: ev.receipt_utc_ms,
        quote_age_ms: ev.quote_age_ms,
        stale,
        missing_level_qty: bid_nz < 5 || ask_nz < 5,
        bid_levels_qty_nonzero: bid_nz,
        ask_levels_qty_nonzero: ask_nz,
        last_price: ev.last_price,
        best_bid: ev.best_bid,
        best_ask: ev.best_ask,
        bid_qty: ev.bid_qty,
        ask_qty: ev.ask_qty,
        volume: ev.volume,
        depth_buy,
        depth_sell,
        quote_source: ev.source.as_str().to_string(),
        ingest_seq: ev.ingest_seq,
        content_fp: ev.content_fp,
    }
}
