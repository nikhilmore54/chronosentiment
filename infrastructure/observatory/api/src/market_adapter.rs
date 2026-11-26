use chronosentiment_core::{MarketEvent, MarketEventType, Side};
use serde::Deserialize;

#[derive(Deserialize, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
enum RawEventType {
    Trade,
    Add,
    Cancel,
    #[serde(other)]
    Unknown,
}

#[derive(Deserialize, Debug)]
struct RawMarketData {
    timestamp: u64,
    #[serde(rename = "type")]
    event_type: RawEventType,
    price: u64,
    qty: u64,
    side: Option<String>,
}

pub fn parse_market_data(lines: Vec<String>) -> Vec<MarketEvent> {
    lines
        .into_iter()
        .filter_map(|line| {
            let data: RawMarketData = serde_json::from_str(&line).ok()?;

            let subtype = match data.event_type {
                RawEventType::Trade => MarketEventType::Trade,
                RawEventType::Add => MarketEventType::NewOrder,
                RawEventType::Cancel => MarketEventType::Cancel,
                RawEventType::Unknown => return None,
            };

            let side = data.side.map(|s| {
                match s.to_uppercase().as_str() {
                    "BUY" => Side::Buy,
                    "SELL" => Side::Sell,
                    _ => Side::Buy, // Default for now
                }
            });

            Some(MarketEvent {
                subtype,
                price: data.price,
                quantity: data.qty,
                side,
                exchange_ts: data.timestamp,
            })
        })
        .collect()
}
