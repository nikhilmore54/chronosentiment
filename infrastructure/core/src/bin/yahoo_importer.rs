use clap::Parser;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use tokio;
use yahoo_finance_api as yahoo;

#[derive(Parser, Debug)]
#[command(author, version, about = "Phase 2A Yahoo OHLCV Importer")]
struct Args {
    #[arg(short, long, default_value = "BTC-USD")]
    symbol: String,

    #[arg(short, long, default_value = "1m")]
    interval: String,

    #[arg(short, long)]
    name: String, // e.g., "2024_etf_approval_yahoo"
}

#[derive(Debug, Serialize)]
struct NormalizedTick {
    symbol: String,
    timestamp: u64,
    price: f64,
    volume: f64,
    is_buyer_maker: bool, // mocked false for OHLCV
}

#[derive(Debug, Serialize)]
struct CaptureGap {
    gap_start: u64,
    gap_end: u64,
    reason: String,
}

#[derive(Debug, Serialize)]
struct CaptureManifest {
    substrate: String,
    capture_start: u64,
    capture_end: u64,
    total_ticks: usize,
    chronology_hash: String,
    gaps: Vec<CaptureGap>,
    provenance: String,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    println!(
        "Starting Yahoo Importer for {} ({})",
        args.symbol, args.name
    );

    let base_dir = PathBuf::from("chronology")
        .join("historical")
        .join(&args.name);
    std::fs::create_dir_all(&base_dir).unwrap();

    let provider = yahoo::YahooConnector::new().unwrap();

    // Yahoo max range for 1m interval is 7 days, we'll just fetch the last 7 days for now
    let response = match provider
        .get_latest_quotes(&args.symbol, &args.interval)
        .await
    {
        Ok(res) => res,
        Err(e) => {
            println!("Error fetching Yahoo data: {}", e);
            return;
        }
    };

    let quotes = match response.quotes() {
        Ok(q) => q,
        Err(e) => {
            println!("Error parsing Yahoo quotes: {}", e);
            return;
        }
    };

    if quotes.is_empty() {
        println!("No quotes returned by Yahoo.");
        return;
    }

    let start_time = quotes.first().unwrap().timestamp * 1000;
    let end_time = quotes.last().unwrap().timestamp * 1000;

    let file_path = base_dir.join(format!(
        "{}_{}.jsonl",
        args.symbol.to_lowercase().replace("-", ""),
        start_time
    ));
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&file_path)
        .unwrap();

    let mut hasher = Sha256::new();
    let mut tick_count = 0;
    let gaps: Vec<CaptureGap> = Vec::new(); // Assuming continuous array from Yahoo

    for quote in quotes {
        let ts = quote.timestamp * 1000;
        let tick = NormalizedTick {
            symbol: args.symbol.clone(),
            timestamp: ts,
            price: quote.close,
            volume: quote.volume as f64,
            is_buyer_maker: false,
        };

        let line = format!("{}\n", serde_json::to_string(&tick).unwrap());
        file.write_all(line.as_bytes()).unwrap();
        hasher.update(line.as_bytes());
        tick_count += 1;
    }

    let final_hash = hasher.finalize();
    let hash_hex = final_hash
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>();

    let manifest = CaptureManifest {
        substrate: args.symbol.clone(),
        capture_start: start_time,
        capture_end: end_time,
        total_ticks: tick_count,
        chronology_hash: hash_hex.clone(),
        gaps,
        provenance: "Yahoo Finance OHLCV".to_string(),
    };

    let meta_path = base_dir.join(format!("{}_{}_manifest.json", args.name, start_time));
    let mut meta_file = File::create(&meta_path).unwrap();
    meta_file
        .write_all(serde_json::to_string_pretty(&manifest).unwrap().as_bytes())
        .unwrap();

    println!("✅ Yahoo Historical Capture Complete: {} ticks", tick_count);
    println!("   Hash: {}", hash_hex);
    println!("   Directory: {:?}", base_dir);
}
