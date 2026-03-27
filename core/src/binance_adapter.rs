use crate::{MarketEvent, MarketEventType, Side};
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct NormalizedMarketEvent {
    pub asset: String,
    pub exchange_ts: u64,
    pub price: f64,
    pub volume: f64,
    pub side: Option<Side>,
    pub best_bid: Option<f64>,
    pub best_ask: Option<f64>,
    pub bids: Option<Vec<(f64, f64)>>,
    pub asks: Option<Vec<(f64, f64)>>,
}

impl NormalizedMarketEvent {
    /// Convert normalized event into legacy integer event format used by current GA/execution.
    /// Uses deterministic rounding to preserve reproducibility.
    pub fn to_legacy_market_event(&self) -> Option<MarketEvent> {
        if !self.price.is_finite() || self.price <= 0.0 || !self.volume.is_finite() || self.volume <= 0.0 {
            return None;
        }
        Some(MarketEvent {
            subtype: MarketEventType::Trade,
            price: (self.price * crate::PRICE_SCALE as f64).round() as u64,
            quantity: self.volume.round().max(1.0) as u64,
            side: self.side,
            exchange_ts: self.exchange_ts,
        })
    }
}

#[derive(Debug, Deserialize)]
struct BinanceTradeMsg {
    #[serde(rename = "T")]
    trade_time_ms: u64,
    #[serde(rename = "p")]
    price: String,
    #[serde(rename = "q")]
    qty: String,
    #[serde(rename = "m")]
    is_buyer_maker: bool,
}

#[derive(Debug, Deserialize)]
struct BinanceDepthMsg {
    #[serde(rename = "E")]
    event_time_ms: u64,
    #[serde(rename = "b")]
    bids: Vec<[String; 2]>,
    #[serde(rename = "a")]
    asks: Vec<[String; 2]>,
}

fn parse_f64(v: &str) -> Option<f64> {
    v.parse::<f64>().ok().filter(|x| x.is_finite() && *x >= 0.0)
}

fn parse_levels(levels: Vec<[String; 2]>) -> Vec<(f64, f64)> {
    levels
        .into_iter()
        .filter_map(|lvl| {
            let px = parse_f64(&lvl[0])?;
            let qty = parse_f64(&lvl[1])?;
            if px > 0.0 && qty > 0.0 {
                Some((px, qty))
            } else {
                None
            }
        })
        .collect()
}

/// Parse Binance trade stream payload (`<symbol>@trade`) into normalized event.
pub fn parse_binance_trade_event(payload: &str) -> Option<NormalizedMarketEvent> {
    let msg: BinanceTradeMsg = serde_json::from_str(payload).ok()?;
    let price = parse_f64(&msg.price)?;
    let volume = parse_f64(&msg.qty)?;
    if price <= 0.0 || volume <= 0.0 {
        return None;
    }

    // Binance `m=true` means buyer is maker => aggressive side is sell.
    let side = if msg.is_buyer_maker { Side::Sell } else { Side::Buy };
    Some(NormalizedMarketEvent {
        asset: "UNKNOWN".to_string(), // Tagged later by loader
        exchange_ts: msg.trade_time_ms,
        price,
        volume,
        side: Some(side),
        best_bid: None,
        best_ask: None,
        bids: None,
        asks: None,
    })
}

/// Parse Binance depth stream payload (`<symbol>@depth`) into normalized event.
/// `top_k=1` gives top-of-book; larger values keep deterministic depth slices.
pub fn parse_binance_depth_event(payload: &str, top_k: usize) -> Option<NormalizedMarketEvent> {
    let msg: BinanceDepthMsg = serde_json::from_str(payload).ok()?;
    let mut bids = parse_levels(msg.bids);
    let mut asks = parse_levels(msg.asks);
    if bids.is_empty() || asks.is_empty() {
        return None;
    }

    bids.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    asks.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

    let k = top_k.max(1);
    let bids_k: Vec<(f64, f64)> = bids.into_iter().take(k).collect();
    let asks_k: Vec<(f64, f64)> = asks.into_iter().take(k).collect();
    let best_bid = bids_k.first().map(|x| x.0)?;
    let best_ask = asks_k.first().map(|x| x.0)?;
    let mid = (best_bid + best_ask) * 0.5;

    Some(NormalizedMarketEvent {
        asset: "UNKNOWN".to_string(), // Tagged later by loader
        exchange_ts: msg.event_time_ms,
        price: mid,
        volume: 0.0,
        side: None,
        best_bid: Some(best_bid),
        best_ask: Some(best_ask),
        bids: Some(bids_k),
        asks: Some(asks_k),
    })
}

#[derive(Debug, Deserialize)]
struct BinanceJsonlRow {
    pub asset: String,
    pub payload: String,
    #[serde(rename = "type")]
    pub event_type: String,
}

/// Load and parse Binance events from a JSONL file.
/// Each line must be a `BinanceJsonlRow` with `asset` and `payload` (raw Binance JSON).
pub fn load_binance_events_from_jsonl(path: &str, top_k: usize) -> Result<Vec<NormalizedMarketEvent>, String> {
    let raw = std::fs::read_to_string(path)
        .map_err(|e| format!("failed to read jsonl {}: {}", path, e))?;
    let mut events = Vec::new();
    for line in raw.lines() {
        if line.trim().is_empty() { continue; }
        let row: BinanceJsonlRow = serde_json::from_str(line)
            .map_err(|e| format!("failed to parse jsonl line: {} (line: {})", e, line))?;
        
        let mut ev = match row.event_type.as_str() {
            "trade" => parse_binance_trade_event(&row.payload),
            "depth" => parse_binance_depth_event(&row.payload, top_k),
            _ => None,
        };

        if let Some(ref mut e) = ev {
            e.asset = row.asset.clone();
            events.push(e.clone());
        }
    }
    Ok(events)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_trade_event_and_maps_side() {
        let payload = r#"{"e":"trade","E":1,"s":"BTCUSDT","t":1,"p":"67500.10","q":"0.005","T":1710000000123,"m":true}"#;
        let ev = parse_binance_trade_event(payload).expect("trade parse");
        assert_eq!(ev.exchange_ts, 1710000000123);
        assert_eq!(ev.side, Some(Side::Sell));
        assert!(ev.price > 0.0);
        assert!(ev.volume > 0.0);
        let legacy = ev.to_legacy_market_event().expect("legacy conversion");
        assert_eq!(legacy.subtype, MarketEventType::Trade);
        assert!(legacy.quantity >= 1);
    }

    #[test]
    fn parses_depth_event_top1() {
        let payload = r#"{"e":"depthUpdate","E":1710000001000,"s":"BTCUSDT","U":1,"u":2,"b":[["67499.90","1.2"],["67499.80","0.8"]],"a":[["67500.10","0.6"],["67500.20","0.9"]]}"#;
        let ev = parse_binance_depth_event(payload, 1).expect("depth parse");
        assert_eq!(ev.best_bid, Some(67499.90));
        assert_eq!(ev.best_ask, Some(67500.10));
        assert!(ev.price > 0.0);
        assert_eq!(ev.bids.as_ref().map(|v| v.len()), Some(1));
        assert_eq!(ev.asks.as_ref().map(|v| v.len()), Some(1));
    }
}
