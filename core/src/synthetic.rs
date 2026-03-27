use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use crate::{MarketEvent, MarketEventType, Side};
use std::collections::HashMap;

fn next_normal(rng: &mut ChaCha8Rng) -> f64 {
    let u1: f64 = rng.gen();
    let u2: f64 = rng.gen();
    (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos()
}

pub fn generate_deterministic_scenarios(asset: &str, base_seed: u64, base_price: u64) -> HashMap<String, Vec<MarketEvent>> {
    let mut scenarios = HashMap::new();
    // drift_range, vol_range, q_range, ts_range
    let regimes = vec![
        ("trending_up", (0.0005, 0.002), (0.001, 0.003), (90, 110), (1, 5)),
        ("trending_down", (-0.002, -0.0005), (0.001, 0.003), (90, 110), (1, 5)),
        ("sideways", (-0.0002, 0.0002), (0.0005, 0.001), (90, 110), (1, 5)),
        ("volatile", (-0.0005, 0.0005), (0.003, 0.006), (40, 60), (1, 3)),
        ("mixed", (-0.001, 0.001), (0.002, 0.004), (70, 90), (1, 5)),
    ];

    let mut master_rng = ChaCha8Rng::seed_from_u64(base_seed);

    for (regime_name, drift_range, vol_range, q_range, ts_range) in regimes {
        for i in 0..10 {
            let scenario_name = format!("{}_{}_{}", asset, regime_name, i);
            let mut scenario_rng = ChaCha8Rng::seed_from_u64(master_rng.next_u64());

            let drift_pct = scenario_rng.gen_range(drift_range.0..=drift_range.1);
            let vol_pct = scenario_rng.gen_range(vol_range.0..=vol_range.1);
            let current_q_base = scenario_rng.gen_range(q_range.0..=q_range.1);
            let current_ts_range = (ts_range.0, ts_range.1);

            let mut events = Vec::with_capacity(200);
            let mut current_price = base_price as f64;
            let mut current_ts = 1000u64;

            for _ in 0..200 {
                current_ts += scenario_rng.gen_range(current_ts_range.0..=current_ts_range.1);

                // Price move: price += price * (drift + volatility * noise)
                let noise = next_normal(&mut scenario_rng);
                let move_pct = drift_pct + vol_pct * noise;
                current_price *= 1.0 + move_pct;
                
                let price = current_price.max(1.0).round() as u64;

                // 3. Quantity: spikes + low periods
                let is_spike = scenario_rng.gen_bool(0.1);
                let quantity = if is_spike {
                    current_q_base * scenario_rng.gen_range(5..15)
                } else {
                    current_q_base + scenario_rng.gen_range(0..20)
                };

                // 4. Subtype distribution
                let subtype_roll = scenario_rng.gen_range(0..100);
                let subtype = if subtype_roll < 40 {
                    MarketEventType::Trade
                } else if subtype_roll < 70 {
                    MarketEventType::NewOrder
                } else {
                    MarketEventType::Cancel
                };

                // 5. Side (biased by drift for Trades)
                let side = if let MarketEventType::Trade = subtype {
                    if drift_pct > 0.0 && scenario_rng.gen_bool(0.65) {
                        Some(Side::Buy)
                    } else if drift_pct < 0.0 && scenario_rng.gen_bool(0.65) {
                        Some(Side::Sell)
                    } else if scenario_rng.gen_bool(0.5) {
                        Some(Side::Buy)
                    } else {
                        Some(Side::Sell)
                    }
                } else {
                    if scenario_rng.gen_bool(0.5) { Some(Side::Buy) } else { Some(Side::Sell) }
                };

                events.push(MarketEvent {
                    subtype,
                    price,
                    quantity,
                    side,
                    exchange_ts: current_ts,
                });
            }
            scenarios.insert(scenario_name, events);
        }
    }
    scenarios
}
