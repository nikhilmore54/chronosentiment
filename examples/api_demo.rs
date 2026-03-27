// examples/api_demo.rs

use chronosentiment_api::*;
use chronosentiment_core::*;

fn main() {
    println!("--- ChronoSentiment API Demo ---");
    println!();

    // 1. POST /simulate
    println!(">>> POST /simulate {{ \"mode\": \"real\", \"seed\": 42 }}");
    let sim_input = SimulateInput {
        mode: "real".to_string(),
        dataset: None,
        seed: 42,
    };
    match handle_simulate(sim_input) {
        Ok(res) => {
            println!("Response: {{");
            println!("  \"pnl\": {},", res.pnl);
            println!("  \"trade_count\": {},", res.trade_count);
            println!("  \"state_hash\": \"{}\",", res.state_hash);
            println!("  \"events\": [{} events]", res.events.len());
            println!("}}");

            // Store result for subsequent calls
            let sim_res = res.original_result;

            // 2. GET /events?from=2&to=5
            println!("\n>>> GET /events?from=2&to=5");
            match handle_events(&sim_res, Some(2), Some(5)) {
                Ok(res) => {
                    println!("Response: [{} raw events in range]", res.events.len());
                }
                Err(e) => println!("Error: {:?}", e),
            }

            // 3. GET /certify
            println!("\n>>> GET /certify");
            match handle_certify(&sim_res) {
                Ok(report) => {
                    println!("Response: {{");
                    println!("  \"hash_1\": \"{}\",", report.hash_1);
                    println!("  \"hash_2\": \"{}\",", report.hash_2);
                    println!("  \"status\": \"{}\"", report.status);
                    println!("}}");
                }
                Err(e) => println!("Error: {:?}", e),
            }

            // 4. GET /trade/O1/inspect
            println!("\n>>> GET /trade/O1/inspect");
            match handle_inspect("O1".to_string(), &sim_res) {
                Ok(inspection) => {
                    println!("Response: {{");
                    println!("  \"order_id\": \"{}\",", inspection.decision.order_id);
                    println!("  \"filled\": {},", inspection.outcome.filled_quantity);
                    println!("  \"causal_chain\": [{} steps]", inspection.execution.causal_chain.len());
                    println!("}}");
                }
                Err(e) => println!("Error: {:?}", e),
            }

            // 5. GET /timeline
            println!("\n>>> GET /timeline");
            match handle_timeline(&sim_res) {
                Ok(timeline) => {
                    println!("Response: [{} events]", timeline.len());
                }
                Err(e) => println!("Error: {:?}", e),
            }

            // 6. POST /simulate_with_market_data
            println!("\n>>> POST /simulate_with_market_data");
            let market_input = MarketDataSimulateInput {
                mode: "real".to_string(),
                market_data_jsonl: vec![
                    r#"{"timestamp": 1, "type": "add", "price": 100, "qty": 500, "side": "SELL"}"#.to_string(),
                    r#"{"timestamp": 2, "type": "trade", "price": 100, "qty": 200}"#.to_string(),
                ],
                order_intents: vec![
                    CreateOrder {
                        order_id: "T1".to_string(),
                        side: Side::Buy,
                        price: 100,
                        quantity: 100,
                        timestamp: 1,
                    }
                ],
            };
            match handle_simulate_with_market_data(market_input) {
                Ok(res) => {
                    println!("Response: {{");
                    println!("  \"pnl\": {},", res.pnl);
                    println!("  \"state_hash\": \"{}\",", res.state_hash);
                    println!("  \"events\": [{} events]", res.events.len());
                    println!("}}");
                }
                Err(e) => println!("Error: {:?}", e),
            }
        }
        Err(e) => println!("Error: {:?}", e),
    }

    println!("\n--- API Execution Complete ---");
}
