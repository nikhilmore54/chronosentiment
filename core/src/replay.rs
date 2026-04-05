use crate::*;

pub fn print_timeline(simulation: &SimulationResult) {
    println!("-- - Timeline ---");
    println!();
    for event in &simulation.events {
        match event {
            SimEvent::MarketEvent {
                subtype,
                price,
                quantity,
                timestamp,
                ..
            } => {
                let subtype_str = match subtype {
                    MarketEventType::NewOrder => "NEW_ORDER",
                    MarketEventType::Trade => "TRADE",
                    MarketEventType::Cancel => "CANCEL",
                };
                if *subtype == MarketEventType::Trade {
                    println!(
                        "t={:<2} | MarketTrade qty={} price={}",
                        timestamp, quantity, price
                    );
                } else {
                    println!(
                        "t={:<2} | MarketEvent {:<9} price={} qty={}",
                        timestamp, subtype_str, price, quantity
                    );
                }
            }
            SimEvent::OrderIntent {
                order_id,
                timestamp,
                ..
            } => {
                println!("t={:<2} | OrderIntent {} created", timestamp, order_id);
            }
            SimEvent::OrderEnteredQueue {
                order_id,
                timestamp,
                ..
            } => {
                println!("t={:<2} | Order {} entered queue", timestamp, order_id);
            }
            SimEvent::PartialFill {
                order_id,
                timestamp,
                filled_qty,
                ..
            } => {
                println!(
                    "t={:<2} | PartialFill {} qty={}",
                    timestamp, order_id, filled_qty
                );
            }
            SimEvent::QueueProgression { .. } => {}
            SimEvent::OrderFilled {
                order_id,
                timestamp,
                ..
            } => {
                println!("t={:<2} | OrderFilled {}", timestamp, order_id);
            }
        }
    }
}

pub fn replay_step(simulation: &SimulationResult, step: usize) {
    if let Some(event) = simulation.events.get(step) {
        println!("Step {}:", step);
        format_event(event);
    }
}

pub fn replay_range(simulation: &SimulationResult, start_timestamp: u64, end_timestamp: u64) {
    println!(
        "-- - Replay (t={} → t={}) ---",
        start_timestamp, end_timestamp
    );
    println!();
    for event in &simulation.events {
        let timestamp = event.timestamp();
        if timestamp >= start_timestamp && timestamp <= end_timestamp {
            format_event(event);
        }
    }
}

fn format_event(event: &SimEvent) {
    let timestamp = event.timestamp();
    match event {
        SimEvent::MarketEvent {
            subtype, quantity, ..
        } => {
            if *subtype == MarketEventType::Trade {
                println!("t={:<2} MarketTrade qty={}", timestamp, quantity);
            }
        }
        SimEvent::OrderEnteredQueue { order_id, .. } => {
            println!("t={:<2} Order {} enters queue", timestamp, order_id);
        }
        SimEvent::PartialFill {
            order_id,
            filled_qty,
            ..
        } => {
            println!(
                "t={:<2} PartialFill {} qty={}",
                timestamp, order_id, filled_qty
            );
        }
        _ => {}
    }
}
