use crate::{MarketEventType, SimEvent};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Candle {
    pub timestamp: u64,
    pub open: u64,
    pub high: u64,
    pub low: u64,
    pub close: u64,
    pub volume: u64,
}

#[derive(Debug, Deserialize)]
pub struct CandleRow {
    #[serde(rename = "Timestamp")]
    timestamp: String,

    #[serde(rename = "Open")]
    open: f64,

    #[serde(rename = "High")]
    high: f64,

    #[serde(rename = "Low")]
    low: f64,

    #[serde(rename = "Close")]
    close: f64,

    #[serde(rename = "Volume")]
    volume: f64,

    #[serde(default)]
    #[serde(rename = "Spread")]
    _spread: Option<f64>,

    #[serde(default)]
    #[serde(rename = "Slippage")]
    _slippage: Option<f64>,

    #[serde(default)]
    #[serde(rename = "Execution_Buy")]
    _exec_buy: Option<f64>,

    #[serde(default)]
    #[serde(rename = "Execution_Sell")]
    _exec_sell: Option<f64>,
}

fn parse_timestamp(ts: &str) -> i64 {
    use chrono::{DateTime, NaiveDateTime};

    NaiveDateTime::parse_from_str(ts, "%Y-%m-%d %H:%M:%S")
        .unwrap_or_else(|_| {
            println!("BAD TIMESTAMP: {}", ts);
            DateTime::from_timestamp(0, 0).unwrap().naive_utc()
        })
        .and_utc()
        .timestamp()
}

pub fn rows_to_candles(rows: Vec<CandleRow>) -> Vec<Candle> {
    rows.into_iter()
        .map(|row| Candle {
            timestamp: parse_timestamp(&row.timestamp) as u64,
            open: (row.open * crate::PRICE_SCALE as f64) as u64,
            high: (row.high * crate::PRICE_SCALE as f64) as u64,
            low: (row.low * crate::PRICE_SCALE as f64) as u64,
            close: (row.close * crate::PRICE_SCALE as f64) as u64,
            volume: row.volume as u64,
        })
        .collect()
}

pub fn convert_candle_to_events(candle: &Candle, mut seq_id: u64) -> (Vec<SimEvent>, u64) {
    let mut events = Vec::new();
    let time_step = 1; // Simple deterministic time increment within candle
    let vol_chunk = candle.volume / 4;

    let mut add_event = |price: u64, qty: u64, ts: u64| {
        events.push(SimEvent::MarketEvent {
            sequence_id: seq_id,
            parent_sequence_id: None,
            subtype: MarketEventType::Trade,
            price,
            quantity: qty,
            side: None,
            timestamp: ts,
        });
        seq_id += 1;
    };

    // Open
    add_event(candle.open, vol_chunk, candle.timestamp);
    // High
    add_event(candle.high, vol_chunk, candle.timestamp + time_step);
    // Low
    add_event(candle.low, vol_chunk, candle.timestamp + time_step * 2);
    // Close
    add_event(
        candle.close,
        candle.volume - (vol_chunk * 3),
        candle.timestamp + time_step * 3,
    );

    (events, seq_id)
}

pub fn convert_series_to_events(candles: &[Candle], start_seq_id: u64) -> Vec<SimEvent> {
    let mut all_events = Vec::new();
    let mut current_seq_id = start_seq_id;

    for candle in candles {
        let (mut candle_events, next_seq_id) = convert_candle_to_events(candle, current_seq_id);
        all_events.append(&mut candle_events);
        current_seq_id = next_seq_id;
    }

    all_events
}

pub fn convert_events_to_candles(events: &[crate::MarketEvent]) -> Vec<Candle> {
    if events.is_empty() {
        return Vec::new();
    }
    let open = events[0].price as u64;
    let mut high = open;
    let mut low = open;
    let mut close = open;
    let mut volume = 0;
    let timestamp = events[0].exchange_ts;

    for ev in events {
        let px = ev.price as u64;
        high = high.max(px);
        low = low.min(px);
        close = px;
        volume += ev.quantity as u64;
    }

    vec![Candle {
        timestamp,
        open,
        high,
        low,
        close,
        volume,
    }]
}
