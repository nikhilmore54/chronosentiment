use crate::NormalizedMarketEvent;
use crate::Side;
use serde::Deserialize;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug, Deserialize)]
struct BinanceRawEvent {
    #[serde(rename = "s")]
    symbol: String,
    #[serde(rename = "E")]
    event_time: u64,
    #[serde(rename = "p")]
    price: String,
    #[serde(rename = "q")]
    quantity: String,
    #[serde(rename = "m")]
    is_buyer_maker: bool,
}

pub fn load_binance_events_from_jsonl(
    path: &str,
    _depth: usize,
) -> Result<Vec<NormalizedMarketEvent>, String> {
    let file = File::open(path).map_err(|e| e.to_string())?;
    let reader = BufReader::new(file);
    let mut events = Vec::new();

    for line in reader.lines() {
        let line = line.map_err(|e| e.to_string())?;
        if let Ok(raw) = serde_json::from_str::<BinanceRawEvent>(&line) {
            events.push(NormalizedMarketEvent {
                asset: raw.symbol,
                exchange_ts: raw.event_time,
                price: raw.price.parse().unwrap_or(0.0),
                volume: raw.quantity.parse().unwrap_or(0.0),
                side: Some(if raw.is_buyer_maker {
                    Side::Sell
                } else {
                    Side::Buy
                }),
                best_bid: None,
                best_ask: None,
                bids: None,
                asks: None,
            });
        }
    }

    Ok(events)
}
