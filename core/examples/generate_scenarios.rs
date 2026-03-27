use chronosentiment_core::{generate_deterministic_scenarios, MarketEventType, Side};
use std::fs::File;
use std::io::Write;
use std::path::Path;

fn main() {
    let assets = vec!["BTC", "ETH", "SOL"];
    let base_seed = 42u64;
    let base_price = 40000u64;

    let data_dir = "scenarios";
    if !Path::new(data_dir).exists() {
        std::fs::create_dir(data_dir).expect("Failed to create scenarios directory");
    }

    println!("Generating 50 deterministic scenarios per asset...");

    for asset in assets {
        let asset_scenarios = generate_deterministic_scenarios(asset, base_seed, base_price);
        println!("Asset {}: generated {} scenarios", asset, asset_scenarios.len());

        for (name, events) in asset_scenarios {
            let filename = format!("{}/{}.csv", data_dir, name);
            let mut file = File::create(filename).expect("Failed to create CSV file");
            
            // CSV Header
            writeln!(file, "timestamp,price,quantity,subtype,side").expect("Failed to write header");

            for ev in events {
                let subtype_str = match ev.subtype {
                    MarketEventType::NewOrder => "NEW_ORDER",
                    MarketEventType::Trade => "TRADE",
                    MarketEventType::Cancel => "CANCEL",
                };
                let side_str = match ev.side {
                    Some(Side::Buy) => "BUY",
                    Some(Side::Sell) => "SELL",
                    None => "NONE",
                };
                writeln!(file, "{},{},{},{},{}", 
                    ev.exchange_ts, 
                    ev.price, 
                    ev.quantity, 
                    subtype_str, 
                    side_str
                ).expect("Failed to write event");
            }
        }
    }

    println!("Generation complete. Scenarios saved in '{}' folder.", data_dir);
}
