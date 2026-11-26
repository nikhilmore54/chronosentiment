use chronosentiment_core::{MarketEvent, MarketEventType, Side};
use std::collections::HashMap;

fn make_events(base_price: u64, scenario_idx: usize, seed: u64, regime: &str) -> Vec<MarketEvent> {
    let mut events = Vec::with_capacity(120);
    let mut price = base_price as i64 + (scenario_idx as i64 * 13) + (seed as i64 % 17);

    for step in 0..120usize {
        let drift = match regime {
            "trending_up" => 4 + (step as i64 % 3),
            "trending_down" => -(4 + (step as i64 % 3)),
            "sideways" => {
                if step % 2 == 0 {
                    1
                } else {
                    -1
                }
            }
            // "volatile"
            _ => {
                let wave = (step as i64 % 11) - 5;
                if step % 2 == 0 {
                    wave.abs() + 2
                } else {
                    -(wave.abs() + 2)
                }
            }
        };
        price = (price + drift).max(1);

        let subtype = if step % 5 == 0 {
            MarketEventType::NewOrder
        } else if step % 11 == 0 {
            MarketEventType::Cancel
        } else {
            MarketEventType::Trade
        };
        let side = if step % 2 == 0 {
            Some(Side::Buy)
        } else {
            Some(Side::Sell)
        };
        events.push(MarketEvent {
            subtype,
            price: price as u64,
            quantity: 100 + ((step as u64 + scenario_idx as u64) % 25),
            side,
            exchange_ts: (step as u64) + 1,
        });
    }

    events
}

pub fn generate_deterministic_scenarios(
    asset: &str,
    seed: u64,
    base_price: u64,
) -> HashMap<String, Vec<MarketEvent>> {
    let mut scenarios: HashMap<String, Vec<MarketEvent>> = HashMap::new();

    for i in 0..10usize {
        scenarios.insert(
            format!("{}_mixed_{}", asset, i),
            make_events(base_price, i, seed, "volatile"),
        );
        scenarios.insert(
            format!("{}_trending_up_{}", asset, i),
            make_events(base_price + 200, i, seed + 31, "trending_up"),
        );
        scenarios.insert(
            format!("{}_trending_down_{}", asset, i),
            make_events(base_price + 400, i, seed + 61, "trending_down"),
        );
        scenarios.insert(
            format!("{}_sideways_{}", asset, i),
            make_events(base_price + 600, i, seed + 91, "sideways"),
        );
        scenarios.insert(
            format!("{}_volatile_{}", asset, i),
            make_events(base_price + 800, i, seed + 121, "volatile"),
        );
    }

    scenarios
}
