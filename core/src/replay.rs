use crate::*;

pub fn print_timeline(simulation: &SimulationResult) {
    println!("--- Timeline ---");
    println!();
    for event in &simulation.events {
        match event {
            SimEvent::MarketEvent { subtype, price, quantity, ts, .. } => {
                let subtype_str = match subtype {
                    MarketEventType::NewOrder => "NEW_ORDER",
                    MarketEventType::Trade => "TRADE",
                    MarketEventType::Cancel => "CANCEL",
                };
                if *subtype == MarketEventType::Trade {
                    println!("t={:<2} | MarketTrade qty={} price={}", ts, quantity, price);
                } else {
                    println!("t={:<2} | MarketEvent {:<9} price={} qty={}", ts, subtype_str, price, quantity);
                }
            }
            SimEvent::OrderIntent { order_id, ts, .. } => {
                println!("t={:<2} | OrderIntent {} created", ts, order_id);
            }
            SimEvent::OrderEnteredQueue { order_id, ts, .. } => {
                println!("t={:<2} | Order {} entered queue", ts, order_id);
            }
            SimEvent::PartialFill { order_id, ts, filled_qty, .. } => {
                println!("t={:<2} | PartialFill {} qty={}", ts, order_id, filled_qty);
            }
            SimEvent::QueueProgression { .. } => {}
        }
    }
}

pub fn replay_step(simulation: &SimulationResult, step: usize) {
    if let Some(event) = simulation.events.get(step) {
        println!("Step {}:", step);
        format_event(event);
    }
}

pub fn replay_range(simulation: &SimulationResult, start_ts: u64, end_ts: u64) {
    println!("--- Replay (t={} → t={}) ---", start_ts, end_ts);
    println!();
    for event in &simulation.events {
        let ts = event.timestamp();
        if ts >= start_ts && ts <= end_ts {
            format_event(event);
        }
    }
}

fn format_event(event: &SimEvent) {
    let ts = event.timestamp();
    match event {
        SimEvent::MarketEvent { subtype, quantity, .. } => {
            if *subtype == MarketEventType::Trade {
                println!("t={:<2} MarketTrade qty={}", ts, quantity);
            }
        }
        SimEvent::OrderEnteredQueue { order_id, .. } => {
            println!("t={:<2} Order {} enters queue", ts, order_id);
        }
        SimEvent::PartialFill { order_id, filled_qty, .. } => {
            println!("t={:<2} PartialFill {} qty={}", ts, order_id, filled_qty);
        }
        _ => {}
    }
}
