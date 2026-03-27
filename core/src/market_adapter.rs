use serde::{Deserialize, Serialize};
use crate::{SimEvent, MarketEventType};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Candle {
    pub timestamp: u64,
    pub open: u64,
    pub high: u64,
    pub low: u64,
    pub close: u64,
    pub volume: u64,
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
    add_event(candle.close, candle.volume - (vol_chunk * 3), candle.timestamp + time_step * 3);

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
