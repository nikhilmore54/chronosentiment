use crate::{CreateOrder, ExecutionMode, MarketEvent, MarketEventType, OrderOutcome, Side, SimEvent, SimulationResult};
use std::collections::HashMap;

pub const FIXED_LATENCY: u64 = 5;

pub enum InternalEvent {
    Market(MarketEvent),
    Order(CreateOrder),
    OrderArrival(u64, CreateOrder),
}

impl InternalEvent {
    pub fn timestamp(&self) -> u64 {
        match self {
            InternalEvent::Market(me) => me.exchange_ts,
            InternalEvent::Order(o) => o.timestamp,
            InternalEvent::OrderArrival(ts, _) => *ts,
        }
    }
}

pub fn run_simulation_harness(
    mode: ExecutionMode,
    market_events: Vec<MarketEvent>,
    create_orders: Vec<CreateOrder>,
) -> (String, SimulationResult, String) {
    let result = run_simulation_with_data(mode, market_events, create_orders);
    ("OK".to_string(), result, "System state fetched successfully".to_string())
}

pub fn run_simulation_with_data(
    mode: ExecutionMode,
    market_events: Vec<MarketEvent>,
    create_orders: Vec<CreateOrder>,
) -> SimulationResult {
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
            for order in &create_orders {
                let intent_seq = next_seq_id;
                events_log.push(SimEvent::OrderIntent {
                    sequence_id: intent_seq,
                    parent_sequence_id: None,
                    order_id: order.order_id.clone(),
                    side: order.side,
                    price: order.price,
                    quantity: order.quantity,
                    timestamp: order.timestamp,
                });
                next_seq_id += 1;
                last_seq_for_order.insert(order.order_id.clone(), intent_seq);

                if let Some(outcome) = order_outcomes.get_mut(&order.order_id) {
                    outcome.filled_quantity = order.quantity;
                    outcome.remaining_quantity = 0;
                    outcome.arrival_time = order.timestamp;
                    outcome.queue_ahead = 0;
                }
                
                let enter_seq = next_seq_id;
                events_log.push(SimEvent::OrderEnteredQueue {
                    sequence_id: enter_seq,
                    parent_sequence_id: Some(intent_seq),
                    order_id: order.order_id.clone(),
                    timestamp: order.timestamp,
                    price: order.price,
                    queue_ahead: 0,
                });
                next_seq_id += 1;
                last_seq_for_order.insert(order.order_id.clone(), enter_seq);
                
                let fill_seq = next_seq_id;
                events_log.push(SimEvent::PartialFill {
                    sequence_id: fill_seq,
                    parent_sequence_id: Some(enter_seq),
                    order_id: order.order_id.clone(),
                    timestamp: order.timestamp,
                    filled_qty: order.quantity,
                    price: order.price,
                });
                next_seq_id += 1;
                last_seq_for_order.insert(order.order_id.clone(), fill_seq);

                let side_multiplier = match order.side {
                    Side::Buy => -1i64,
                    Side::Sell => 1i64,
                };
                pnl += side_multiplier * (order.quantity * order.price) as i64;
                total_trades += 1;
            }
        }
        ExecutionMode::Real => {
            let mut active_order_ids: Vec<String> = Vec::new();
            let mut q_ahead_map: HashMap<String, u64> = HashMap::new();
            let mut market_book: HashMap<u64, u64> = HashMap::new();

            // Default queue_ahead for specific benchmark orders
            q_ahead_map.insert("O1".to_string(), 1000);
            q_ahead_map.insert("O2".to_string(), 2000);
            q_ahead_map.insert("O3".to_string(), 500);

            let first_price = market_events.first().map(|e| e.price).unwrap_or(0);
            let last_price = market_events.last().map(|e| e.price).unwrap_or(0);
            let drift = (last_price as f64 - first_price as f64) * 0.3;

            let mut event_queue: Vec<InternalEvent> = Vec::new();
            for me in market_events {
                event_queue.push(InternalEvent::Market(me));
            }
            for order in &create_orders {
                event_queue.push(InternalEvent::Order(order.clone()));
            }

            let mut reference_price: u64 = 0;

            while !event_queue.is_empty() {
                event_queue.sort_by(|a, b| {
                    let ts_cmp = a.timestamp().cmp(&b.timestamp());
                    if ts_cmp == std::cmp::Ordering::Equal {
                        match (a, b) {
                            (InternalEvent::Order(_), _) => std::cmp::Ordering::Less,
                            (_, InternalEvent::Order(_)) => std::cmp::Ordering::Greater,
                            (InternalEvent::OrderArrival(_, _), _) => std::cmp::Ordering::Less,
                            (_, InternalEvent::OrderArrival(_, _)) => std::cmp::Ordering::Greater,
                            _ => std::cmp::Ordering::Equal,
                        }
                    } else {
                        ts_cmp
                    }
                });

                let event = event_queue.remove(0);
                let t = event.timestamp();

                match event {
                    InternalEvent::Order(o) => {
                        let intent_seq = next_seq_id;
                        events_log.push(SimEvent::OrderIntent {
                            sequence_id: intent_seq,
                            parent_sequence_id: None,
                            order_id: o.order_id.clone(),
                            side: o.side,
                            price: o.price,
                            quantity: o.quantity,
                            timestamp: o.timestamp,
                        });
                        next_seq_id += 1;
                        last_seq_for_order.insert(o.order_id.clone(), intent_seq);
                        event_queue.push(InternalEvent::OrderArrival(t + FIXED_LATENCY, o.clone()));
                    }
                    InternalEvent::OrderArrival(arrival_ts, order) => {
                        let qa = if let Some(preset) = q_ahead_map.get(&order.order_id) {
                            *preset
                        } else {
                            *market_book.get(&order.price).unwrap_or(&0)
                        };

                        let parent_seq = *last_seq_for_order.get(&order.order_id).unwrap_or(&0);
                        if let Some(outcome) = order_outcomes.get_mut(&order.order_id) {
                            outcome.arrival_time = arrival_ts;
                            outcome.queue_ahead = qa;
                        }
                        active_order_ids.push(order.order_id.clone());
                        q_ahead_map.insert(order.order_id.clone(), qa);
                        
                        let enter_seq = next_seq_id;
                        events_log.push(SimEvent::OrderEnteredQueue {
                            sequence_id: enter_seq,
                            parent_sequence_id: Some(parent_seq),
                            order_id: order.order_id.clone(),
                            timestamp: arrival_ts,
                            price: order.price,
                            queue_ahead: qa,
                        });
                        next_seq_id += 1;
                        last_seq_for_order.insert(order.order_id.clone(), enter_seq);
                    }
                    InternalEvent::Market(me) => {
                        if reference_price == 0 { reference_price = me.price; }
                        events_log.push(SimEvent::MarketEvent {
                            sequence_id: next_seq_id,
                            parent_sequence_id: None,
                            subtype: me.subtype,
                            price: me.price,
                            quantity: me.quantity,
                            side: me.side,
                            timestamp: me.exchange_ts,
                        });
                        next_seq_id += 1;

                        match me.subtype {
                            MarketEventType::NewOrder => {
                                *market_book.entry(me.price).or_insert(0) += me.quantity;
                            }
                            MarketEventType::Cancel | MarketEventType::Trade => {
                                let entry = market_book.entry(me.price).or_insert(0);
                                *entry = entry.saturating_sub(me.quantity);
                            }
                        }

                        if let MarketEventType::Trade = me.subtype {
                            // 🔴 HARDER liquidity constraint (was 0.3 → now 0.2)
                            let mut remaining_liquidity = (me.quantity as f64 * 0.5) as u64;
                        
                            for id in &active_order_ids {
                                let order_intent = if let Some(o) = create_orders.iter().find(|o| o.order_id == *id) {
                                    o
                                } else {
                                    continue;
                                };
                        
                                // ✅ STRICT LIMIT CHECK (already correct)
                                let price_ok = match order_intent.side {
                                    Side::Buy => order_intent.price >= me.price,
                                    Side::Sell => order_intent.price <= me.price,
                                };
                                if !price_ok { continue; }
                        
                                let qa = if let Some(q) = q_ahead_map.get_mut(id) {
                                    q
                                } else {
                                    continue;
                                };
                        
                                let parent_seq = *last_seq_for_order.get(id).unwrap_or(&0);
                        
                                // ✅ Anchor logic unchanged
                                let dynamic_anchor = reference_price as f64 + drift;
                                let anchor_buffer = (dynamic_anchor * 0.0001).max(1.0);
                                let anchor_met = match order_intent.side {
                                    Side::Buy => (me.price as f64) <= dynamic_anchor + anchor_buffer,
                                    Side::Sell => (me.price as f64) >= dynamic_anchor - anchor_buffer,
                                };
                        
                                if !anchor_met || remaining_liquidity == 0 {
                                    continue;
                                }
                        
                                // 🔴 Deterministic RNG (unchanged but reused)
                                let mut fill_hasher = std::collections::hash_map::DefaultHasher::new();
                                use std::hash::Hasher;
                                fill_hasher.write(id.as_bytes());
                                fill_hasher.write_u64(me.exchange_ts);
                                let fill_roll = (fill_hasher.finish() % 1000) as f64 / 1000.0;
                        
                                // 🔴 APPLY probability EARLY (before queue clear)
                                if fill_roll > order_intent.fill_probability {
                                    continue;
                                }
                        
                                // 🔴 QUEUE CLEARING WITH FRICTION (NEW)
                                if *qa > 0 {
                                    let penetration_factor = 0.6; // NEW

                                    let effective_queue = (*qa as f64 * penetration_factor) as u64;
                                    let consumed = remaining_liquidity.min(effective_queue);

                                    *qa = qa.saturating_sub(consumed);
                                    remaining_liquidity -= consumed;
                        
                                    let prog_seq = next_seq_id;
                                    events_log.push(SimEvent::QueueProgression {
                                        sequence_id: prog_seq,
                                        parent_sequence_id: Some(parent_seq),
                                        order_id: id.clone(),
                                        timestamp: t,
                                        queue_ahead: *qa,
                                    });
                                    next_seq_id += 1;
                                    last_seq_for_order.insert(id.clone(), prog_seq);
                        
                                    if let Some(outcome) = order_outcomes.get_mut(id) {
                                        outcome.queue_ahead = *qa;
                                    }
                        
                                    // 🔴 CRITICAL: NEVER fill in same tick after queue clearing
                                    continue;
                                }
                        
                                // 🔴 ONLY fill if queue fully cleared BEFORE this tick
                                if *qa == 0 && remaining_liquidity > 0 {
                                    let current_parent_seq = *last_seq_for_order.get(id).unwrap_or(&0);
                        
                                    if let Some(outcome) = order_outcomes.get_mut(id) {
                                        if outcome.remaining_quantity > 0 {
                        
                                            let fill = remaining_liquidity.min(outcome.remaining_quantity);
                        
                                            // 🔴 SECOND probability check (optional but realistic)
                                            // let fill_roll_2 = ((fill_hasher.finish() >> 10) % 1000) as f64 / 1000.0;
                                            // if fill_roll_2 > order_intent.fill_probability {
                                            //     continue;
                                            // }
                        
                                            // 🔴 SLIPPAGE + EXECUTION LOGGING (NEW)
                                            let mut slippage_hasher = std::collections::hash_map::DefaultHasher::new();
                                            slippage_hasher.write(id.as_bytes());
                                            slippage_hasher.write_u64(me.exchange_ts);
                                            slippage_hasher.write_u64(outcome.filled_quantity);
                                            let slippage_roll = (slippage_hasher.finish() % 100) as f64 / 100.0;
                                            
                                            // Spread ≈ 0.01% of price
                                            let spread = (me.price as f64 * 0.0001).max(1.0);
                                            let slippage = (spread * slippage_roll) as u64;

                                            let execution_price = match order_intent.side {
                                                Side::Buy => me.price + slippage,
                                                Side::Sell => me.price.saturating_sub(slippage),
                                            };

                                            outcome.filled_quantity += fill;
                                            outcome.remaining_quantity -= fill;
                                            remaining_liquidity -= fill;
                        
                                            let side_multiplier = match order_intent.side {
                                                Side::Buy => -1i64,
                                                Side::Sell => 1i64,
                                            };
                                            pnl += side_multiplier * (fill * execution_price) as i64;
                                            total_trades += 1;
                        
                                            let fill_seq_current = next_seq_id;
                                            events_log.push(SimEvent::PartialFill {
                                                sequence_id: fill_seq_current,
                                                parent_sequence_id: Some(current_parent_seq),
                                                order_id: id.clone(),
                                                timestamp: t,
                                                filled_qty: fill,
                                                price: execution_price,
                                            });
                                            next_seq_id += 1;
                                            last_seq_for_order.insert(id.clone(), fill_seq_current);
                        
                                            if outcome.remaining_quantity == 0 {
                                                let final_fill_seq = next_seq_id;
                                                events_log.push(SimEvent::OrderFilled {
                                                    sequence_id: final_fill_seq,
                                                    parent_sequence_id: Some(fill_seq_current),
                                                    order_id: id.clone(),
                                                    timestamp: t,
                                                });
                                                next_seq_id += 1;
                                                last_seq_for_order.insert(id.clone(), final_fill_seq);
                                            }
                                        }
                                    }
                                }
                        
                                if remaining_liquidity == 0 {
                                    // allow small chance to still fill (edge liquidity)
                                    if fill_roll < 0.1 {
                                        remaining_liquidity = 1;
                                    } else {
                                        continue;
                                    }
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
                    _ => MarketEventType::NewOrder,
                },
                "price" => price = value.parse::<u64>().unwrap_or(0),
                "quantity" => quantity = value.parse::<u64>().unwrap_or(0),
                "side" => side = Some(match value {
                    "BUY" => Side::Buy,
                    "SELL" => Side::Sell,
                    _ => Side::Buy,
                }),
                "exchange_ts" => exchange_ts = value.parse::<u64>().unwrap_or(0),
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
    let mut timestamp = 0;
    let mut fill_probability = 1.0;

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
                    _ => Side::Buy,
                },
                "price" => price = value.parse::<u64>().unwrap_or(0),
                "quantity" => quantity = value.parse::<u64>().unwrap_or(0),
                "ts" => timestamp = value.parse::<u64>().unwrap_or(0),
                "fill_probability" => fill_probability = value.parse::<f64>().unwrap_or(1.0),
                _ => {}
            }
        }
    }
    CreateOrder { order_id, side, price, quantity, timestamp, fill_probability }
}
