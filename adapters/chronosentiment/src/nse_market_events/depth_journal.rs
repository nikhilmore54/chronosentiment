//! Durable timestamped L1–L5 depth snapshots (not latest-value-only).
//!
//! Field shapes align with `scripts/rp608_nse_of_book_depth_history.snapshot_row_from_tick`
//! for differential comparison (protocol_id preserved).

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

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

fn level_tuple_from_json(levels: &Value) -> Vec<Value> {
    let mut out = Vec::with_capacity(5);
    for i in 0..5 {
        let row = levels
            .as_array()
            .and_then(|a| a.get(i))
            .cloned()
            .unwrap_or(json!({}));
        let px = row
            .get("price")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        let qty = row
            .get("quantity")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        out.push(json!({"price": round4(px), "quantity": round4(qty)}));
    }
    out
}

fn f64_field(tick: &Value, key: &str) -> f64 {
    tick.get(key).and_then(|v| v.as_f64()).unwrap_or(0.0)
}

/// Python-parity depth row from a retained ticker-cache tick (Track B side-path).
/// Does not mutate OF ±0.30 or paper ledgers. Authority 0%.
pub fn depth_snapshot_row_from_cache_tick(
    symbol: &str,
    tick: &Value,
    sample_utc_ms: u64,
) -> Value {
    let depth_buy = level_tuple_from_json(tick.get("depth_buy").unwrap_or(&Value::Null));
    let depth_sell = level_tuple_from_json(tick.get("depth_sell").unwrap_or(&Value::Null));
    let bid_nz = depth_buy
        .iter()
        .filter(|lv| lv.get("quantity").and_then(|q| q.as_f64()).unwrap_or(0.0) > 0.0)
        .count() as u32;
    let ask_nz = depth_sell
        .iter()
        .filter(|lv| lv.get("quantity").and_then(|q| q.as_f64()).unwrap_or(0.0) > 0.0)
        .count() as u32;
    let exch_sec = tick
        .get("exchange_timestamp_sec")
        .and_then(|v| v.as_u64().or_else(|| v.as_i64().map(|i| i as u64)))
        .unwrap_or(0);
    let exch_ms = if exch_sec > 0 {
        Value::from(exch_sec * 1000)
    } else {
        Value::Null
    };
    let receipt_ms = tick
        .get("receipt_utc_ms")
        .and_then(|v| v.as_u64().or_else(|| v.as_i64().map(|i| i as u64)))
        .filter(|v| *v > 0)
        .map(Value::from)
        .unwrap_or(Value::Null);
    let age_ms = tick.get("quote_age_ms").and_then(|v| v.as_f64());
    let stale = age_ms.map(|a| a > 60_000.0).unwrap_or(false);
    let token = tick
        .get("instrument_token")
        .and_then(|v| v.as_u64().or_else(|| v.as_i64().map(|i| i as u64)))
        .filter(|v| *v > 0)
        .map(Value::from)
        .unwrap_or(Value::Null);
    let last = f64_field(tick, "last_price");
    let best_bid = f64_field(tick, "best_bid");
    let best_ask = f64_field(tick, "best_ask");
    let volume = f64_field(tick, "volume");
    let sample_iso = chrono::DateTime::<chrono::Utc>::from_timestamp_millis(sample_utc_ms as i64)
        .map(|t| t.to_rfc3339_opts(chrono::SecondsFormat::Millis, true))
        .unwrap_or_default();
    json!({
        "protocol_id": PROTOCOL_ID,
        "record_type": "depth_snapshot_l1_l5",
        "symbol": symbol,
        "instrument_token": token,
        "sample_utc_ms": sample_utc_ms,
        "sample_iso": sample_iso,
        "exchange_timestamp_sec": if exch_sec > 0 { Value::from(exch_sec) } else { Value::Null },
        "exchange_timestamp_utc_ms": exch_ms,
        "receipt_utc_ms": receipt_ms,
        "quote_age_ms": age_ms.map(Value::from).unwrap_or(Value::Null),
        "stale": stale,
        "missing_level_qty": bid_nz < 5 || ask_nz < 5,
        "bid_levels_qty_nonzero": bid_nz,
        "ask_levels_qty_nonzero": ask_nz,
        "last_price": if last > 0.0 { Value::from(last) } else { Value::Null },
        "best_bid": if best_bid > 0.0 { Value::from(best_bid) } else { Value::Null },
        "best_ask": if best_ask > 0.0 { Value::from(best_ask) } else { Value::Null },
        "bid_qty": f64_field(tick, "bid_qty"),
        "ask_qty": f64_field(tick, "ask_qty"),
        "volume": if volume > 0.0 { Value::from(volume) } else { Value::Null },
        "depth_buy": depth_buy,
        "depth_sell": depth_sell,
        "quote_source": tick.get("quote_source").and_then(|v| v.as_str()).unwrap_or("kite_ws_full"),
        "authority": "rust_nse_depth_journal",
    })
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_tick_row_rounds_levels_and_flags_missing() {
        let tick = json!({
            "instrument_token": 738561,
            "exchange_timestamp_sec": 1_700_000_000u64,
            "receipt_utc_ms": 1_700_000_000_100u64,
            "quote_age_ms": 50.0,
            "last_price": 100.0,
            "best_bid": 99.9,
            "best_ask": 100.1,
            "bid_qty": 10.0,
            "ask_qty": 12.0,
            "volume": 1000.0,
            "depth_buy": [{"price": 99.9, "quantity": 10.0}],
            "depth_sell": [{"price": 100.1, "quantity": 12.0}],
            "quote_source": "kite_ws_full",
        });
        let row = depth_snapshot_row_from_cache_tick("RELIANCE", &tick, 1_700_000_000_200);
        assert_eq!(row["protocol_id"], PROTOCOL_ID);
        assert_eq!(row["symbol"], "RELIANCE");
        assert_eq!(row["missing_level_qty"], true);
        assert_eq!(row["bid_levels_qty_nonzero"], 1);
        assert_eq!(row["stale"], false);
        assert_eq!(row["authority"], "rust_nse_depth_journal");
        assert_eq!(row["depth_buy"].as_array().unwrap().len(), 5);
    }
}
