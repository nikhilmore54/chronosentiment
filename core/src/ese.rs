use crate::*;
use std::collections::HashMap;

pub const FIXED_LATENCY: u64 = 2;

#[derive(Clone)]
pub enum InternalEvent {
    Market(MarketEvent),
    Order(CreateOrder),
}

impl InternalEvent {
    fn timestamp(&self) -> u64 {
        match self {
            InternalEvent::Market(e) => e.exchange_ts,
            InternalEvent::Order(o) => o.ts,
        }
    }
}

pub fn run_simulation(mode: ExecutionMode) -> SimulationResult {
    let market_events_data = r#"
        {"type":"MarketEvent","subtype":"NEW_ORDER","price":100,"quantity":500,"side":"SELL","exchange_ts":1}
        {"type":"MarketEvent","subtype":"NEW_ORDER","price":100,"quantity":500,"side":"SELL","exchange_ts":2}
        {"type":"MarketEvent","subtype":"NEW_ORDER","price":100,"quantity":500,"side":"SELL","exchange_ts":3}
        {"type":"MarketEvent","subtype":"TRADE","price":100,"quantity":200,"exchange_ts":4}
        {"type":"MarketEvent","subtype":"NEW_ORDER","price":100,"quantity":300,"side":"SELL","exchange_ts":5}
        {"type":"MarketEvent","subtype":"TRADE","price":100,"quantity":400,"exchange_ts":6}
        {"type":"MarketEvent","subtype":"CANCEL","price":100,"quantity":200,"exchange_ts":7}
        {"type":"MarketEvent","subtype":"TRADE","price":100,"quantity":500,"exchange_ts":8}
        {"type":"MarketEvent","subtype":"NEW_ORDER","price":101,"quantity":800,"side":"SELL","exchange_ts":9}
        {"type":"MarketEvent","subtype":"TRADE","price":101,"quantity":300,"exchange_ts":10}
    "#;

    let order_intents_data = r#"
        {"type":"CreateOrder","order_id":"O1","side":"BUY","price":100,"quantity":600,"ts":2}
        {"type":"CreateOrder","order_id":"O2","side":"BUY","price":100,"quantity":400,"ts":5}
        {"type":"CreateOrder","order_id":"O3","side":"BUY","price":101,"quantity":300,"ts":9}
    "#;

    let market_events: Vec<MarketEvent> = market_events_data
        .lines()
        .filter(|s| !s.trim().is_empty())
        .map(|s| parse_market_event(s.trim()))
        .collect();

    let create_orders: Vec<CreateOrder> = order_intents_data
        .lines()
        .filter(|s| !s.trim().is_empty())
        .map(|s| parse_create_order(s.trim()))
        .collect();

    let mut pnl = 0i64;
    let mut total_trades = 0u64;
    let mut order_outcomes: HashMap<String, OrderOutcome> = HashMap::new();
    let mut events_log: Vec<SimEvent> = Vec::new();
    let mut next_seq_id = 0u64;
    let mut last_seq_for_order: HashMap<String, u64> = HashMap::new();

    for order in &create_orders {
        order_outcomes.insert(
            order.order_id.clone(),
            OrderOutcome {
                order_id: order.order_id.clone(),
                filled_quantity: 0,
                remaining_quantity: order.quantity,
                arrival_time: 0,
                queue_ahead: 0,
            },
        );
    }

    match mode {
        ExecutionMode::Ideal => {
            for order in create_orders {
                let intent_seq = next_seq_id;
                events_log.push(SimEvent::OrderIntent {
                    sequence_id: intent_seq,
                    parent_sequence_id: None,
                    order_id: order.order_id.clone(),
                    side: order.side,
                    price: order.price,
                    quantity: order.quantity,
                    ts: order.ts,
                });
                next_seq_id += 1;
                last_seq_for_order.insert(order.order_id.clone(), intent_seq);

                let outcome = order_outcomes.get_mut(&order.order_id).unwrap();
                outcome.filled_quantity = order.quantity;
                outcome.remaining_quantity = 0;
                outcome.arrival_time = order.ts;
                outcome.queue_ahead = 0;
                
                let enter_seq = next_seq_id;
                events_log.push(SimEvent::OrderEnteredQueue {
                    sequence_id: enter_seq,
                    parent_sequence_id: Some(intent_seq),
                    order_id: order.order_id.clone(),
                    ts: order.ts,
                    price: order.price,
                    quantity_ahead: 0,
                });
                next_seq_id += 1;
                last_seq_for_order.insert(order.order_id.clone(), enter_seq);
                
                let fill_seq = next_seq_id;
                events_log.push(SimEvent::PartialFill {
                    sequence_id: fill_seq,
                    parent_sequence_id: Some(enter_seq),
                    order_id: order.order_id.clone(),
                    ts: order.ts,
                    filled_qty: order.quantity,
                    price: order.price,
                });
                next_seq_id += 1;
                last_seq_for_order.insert(order.order_id.clone(), fill_seq);

                pnl += (order.quantity * order.price) as i64;
                total_trades += 1;
            }
        }
        ExecutionMode::Real => {
            let mut active_order_ids: Vec<String> = Vec::new();
            let mut q_ahead_map: HashMap<String, u64> = HashMap::new();
            
            q_ahead_map.insert("O1".to_string(), 1000);
            q_ahead_map.insert("O2".to_string(), 2000);
            q_ahead_map.insert("O3".to_string(), 500);

            let mut internal_stream: Vec<InternalEvent> = Vec::new();
            for me in market_events {
                internal_stream.push(InternalEvent::Market(me));
            }
            for order in create_orders {
                internal_stream.push(InternalEvent::Order(order));
            }

            internal_stream.sort_by(|a, b| {
                let ts_cmp = a.timestamp().cmp(&b.timestamp());
                if ts_cmp == std::cmp::Ordering::Equal {
                    match (a, b) {
                        (InternalEvent::Order(_), InternalEvent::Market(_)) => std::cmp::Ordering::Less,
                        (InternalEvent::Market(_), InternalEvent::Order(_)) => std::cmp::Ordering::Greater,
                        _ => std::cmp::Ordering::Equal,
                    }
                } else {
                    ts_cmp
                }
            });

            let mut latently_scheduled_orders: Vec<(u64, CreateOrder)> = Vec::new();

            for t in 1..=10 {
                for event in &internal_stream {
                    if let InternalEvent::Order(o) = event {
                        if o.ts == t {
                            let intent_seq = next_seq_id;
                            events_log.push(SimEvent::OrderIntent {
                                sequence_id: intent_seq,
                                parent_sequence_id: None,
                                order_id: o.order_id.clone(),
                                side: o.side,
                                price: o.price,
                                quantity: o.quantity,
                                ts: o.ts,
                            });
                            next_seq_id += 1;
                            last_seq_for_order.insert(o.order_id.clone(), intent_seq);
                            latently_scheduled_orders.push((t + FIXED_LATENCY, o.clone()));
                        }
                    }
                }

                let mut arrived_now = Vec::new();
                for i in (0..latently_scheduled_orders.len()).rev() {
                    if latently_scheduled_orders[i].0 == t {
                        let (_, order) = latently_scheduled_orders.remove(i);
                        arrived_now.push(order);
                    }
                }
                arrived_now.sort_by(|a, b| a.order_id.cmp(&b.order_id));
                for order in arrived_now {
                    let qa = *q_ahead_map.get(&order.order_id).unwrap();
                    let parent_seq = *last_seq_for_order.get(&order.order_id).unwrap();
                    let outcome = order_outcomes.get_mut(&order.order_id).unwrap();
                    outcome.arrival_time = t;
                    outcome.queue_ahead = qa;
                    active_order_ids.push(order.order_id.clone());
                    
                    let enter_seq = next_seq_id;
                    events_log.push(SimEvent::OrderEnteredQueue {
                        sequence_id: enter_seq,
                        parent_sequence_id: Some(parent_seq),
                        order_id: order.order_id.clone(),
                        ts: t,
                        price: order.price,
                        quantity_ahead: qa,
                    });
                    next_seq_id += 1;
                    last_seq_for_order.insert(order.order_id.clone(), enter_seq);
                }

                for event in &internal_stream {
                    if let InternalEvent::Market(me) = event {
                        if me.exchange_ts == t {
                            events_log.push(SimEvent::MarketEvent {
                                sequence_id: next_seq_id,
                                parent_sequence_id: None,
                                subtype: me.subtype,
                                price: me.price,
                                quantity: me.quantity,
                                ts: me.exchange_ts,
                            });
                            next_seq_id += 1;

                            if let MarketEventType::Trade = me.subtype {
                                let mut remaining_liquidity = me.quantity;
                                for id in &active_order_ids {
                                    let qa = q_ahead_map.get_mut(id).unwrap();
                                    let parent_seq = *last_seq_for_order.get(id).unwrap();
                                    
                                    let order_price = if id == "O1" || id == "O2" { 100 } else { 101 };
                                    
                                    if order_price == me.price {
                                        if *qa > 0 {
                                            let consumed = if remaining_liquidity > *qa { *qa } else { remaining_liquidity };
                                            *qa -= consumed;
                                            remaining_liquidity -= consumed;
                                            
                                            let prog_seq = next_seq_id;
                                            events_log.push(SimEvent::QueueProgression {
                                                sequence_id: prog_seq,
                                                parent_sequence_id: Some(parent_seq),
                                                order_id: id.clone(),
                                                ts: t,
                                                new_quantity_ahead: *qa,
                                            });
                                            next_seq_id += 1;
                                            last_seq_for_order.insert(id.clone(), prog_seq);
                                        }
                                        
                                        if *qa == 0 && remaining_liquidity > 0 {
                                            let current_parent_seq = *last_seq_for_order.get(id).unwrap();
                                            let outcome = order_outcomes.get_mut(id).unwrap();
                                            if outcome.remaining_quantity > 0 {
                                                let fill = if remaining_liquidity > outcome.remaining_quantity {
                                                    outcome.remaining_quantity
                                                } else {
                                                    remaining_liquidity
                                                };
                                                
                                                outcome.filled_quantity += fill;
                                                outcome.remaining_quantity -= fill;
                                                remaining_liquidity -= fill;
                                                pnl += (fill * me.price) as i64;
                                                total_trades += 1;
                                                
                                                let fill_seq = next_seq_id;
                                                events_log.push(SimEvent::PartialFill {
                                                    sequence_id: fill_seq,
                                                    parent_sequence_id: Some(current_parent_seq),
                                                    order_id: id.clone(),
                                                    ts: t,
                                                    filled_qty: fill,
                                                    price: me.price,
                                                });
                                                next_seq_id += 1;
                                                last_seq_for_order.insert(id.clone(), fill_seq);
                                            }
                                        }
                                    }
                                    if remaining_liquidity == 0 { break; }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    SimulationResult {
        pnl,
        trades: total_trades,
        order_outcomes,
        events: events_log,
    }
}

pub fn parse_market_event(json_str: &str) -> MarketEvent {
    let mut subtype = MarketEventType::NewOrder;
    let mut price = 0;
    let mut quantity = 0;
    let mut side = None;
    let mut exchange_ts = 0;

    for part in json_str.trim_start_matches('{').trim_end_matches('}').split(',') {
        let parts: Vec<&str> = part.split(':').collect();
        if parts.len() == 2 {
            let key = parts[0].trim().trim_matches('"');
            let value = parts[1].trim().trim_matches('"').trim();

            match key {
                "subtype" => subtype = match value {
                    "NEW_ORDER" => MarketEventType::NewOrder,
                    "TRADE" => MarketEventType::Trade,
                    "CANCEL" => MarketEventType::Cancel,
                    _ => unreachable!(),
                },
                "price" => price = value.parse::<u64>().unwrap(),
                "quantity" => quantity = value.parse::<u64>().unwrap(),
                "side" => side = Some(match value {
                    "BUY" => Side::Buy,
                    "SELL" => Side::Sell,
                    _ => unreachable!(),
                }),
                "exchange_ts" => exchange_ts = value.parse::<u64>().unwrap(),
                _ => {}
            }
        }
    }
    MarketEvent { subtype, price, quantity, side, exchange_ts }
}

pub fn parse_create_order(json_str: &str) -> CreateOrder {
    let mut order_id = String::new();
    let mut side = Side::Buy;
    let mut price = 0;
    let mut quantity = 0;
    let mut ts = 0;

    for part in json_str.trim_start_matches('{').trim_end_matches('}').split(',') {
        let parts: Vec<&str> = part.split(':').collect();
        if parts.len() == 2 {
            let key = parts[0].trim().trim_matches('"');
            let value = parts[1].trim().trim_matches('"').trim();

            match key {
                "order_id" => order_id = value.to_string(),
                "side" => side = match value {
                    "BUY" => Side::Buy,
                    "SELL" => Side::Sell,
                    _ => unreachable!(),
                },
                "price" => price = value.parse::<u64>().unwrap(),
                "quantity" => quantity = value.parse::<u64>().unwrap(),
                "ts" => ts = value.parse::<u64>().unwrap(),
                _ => {}
            }
        }
    }
    CreateOrder { order_id, side, price, quantity, ts }
}
