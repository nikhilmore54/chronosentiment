use clap::Parser;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Parser, Debug)]
#[command(author, version, about = "Canonical Historical Chronology Importer")]
struct Args {
    #[arg(short, long, default_value = "BTCUSDT")]
    symbol: String,

    #[arg(short, long, default_value = "1m")]
    interval: String,

    #[arg(short, long)]
    start_time: u64, // ms timestamp

    #[arg(short, long)]
    end_time: u64, // ms timestamp

    #[arg(short, long)]
    name: String, // e.g., "2024_etf_approval"
}

#[derive(Debug, Serialize)]
struct NormalizedTick {
    symbol: String,
    timestamp: u64,
    price: f64,
    volume: f64,
    is_buyer_maker: bool, // mocked false for kline mapping
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
}

fn main() {
    let args = Args::parse();
    println!("Starting Historical Importer for {} ({})", args.symbol, args.name);

    let base_dir = PathBuf::from("chronology").join("historical").join(&args.name);
    std::fs::create_dir_all(&base_dir).unwrap();

    let file_path = base_dir.join(format!("{}_{}.jsonl", args.symbol.to_lowercase(), args.start_time));
    let mut file = OpenOptions::new().create(true).write(true).truncate(true).open(&file_path).unwrap();

    let client = Client::new();
    let mut current_start = args.start_time;
    let mut hasher = Sha256::new();
    let mut tick_count = 0;
    let mut gaps = Vec::new();

    loop {
        if current_start >= args.end_time {
            break;
        }

        let url = format!(
            "https://api.binance.com/api/v3/klines?symbol={}&interval={}&startTime={}&endTime={}&limit=1000",
            args.symbol.to_uppercase(), args.interval, current_start, args.end_time
        );

        let res = match client.get(&url).send() {
            Ok(r) => r,
            Err(e) => {
                println!("Request error: {}. Recording gap and retrying...", e);
                gaps.push(CaptureGap {
                    gap_start: current_start,
                    gap_end: current_start + 60000,
                    reason: "API Fetch Error".to_string(),
                });
                current_start += 60000;
                continue;
            }
        };

        if !res.status().is_success() {
            println!("API Error: {}. Recording gap and retrying...", res.status());
            gaps.push(CaptureGap {
                gap_start: current_start,
                gap_end: current_start + 60000,
                reason: "API Rate Limit / Error".to_string(),
            });
            std::thread::sleep(std::time::Duration::from_secs(5));
            current_start += 60000;
            continue;
        }

        let klines: Vec<Value> = res.json().unwrap_or_else(|_| vec![]);
        if klines.is_empty() {
            break; // No more data
        }

        for k in &klines {
            let ts = k[0].as_u64().unwrap();
            let close: f64 = k[4].as_str().unwrap().parse().unwrap();
            let vol: f64 = k[5].as_str().unwrap().parse().unwrap();

            let tick = NormalizedTick {
                symbol: args.symbol.clone(),
                timestamp: ts,
                price: close,
                volume: vol,
                is_buyer_maker: false,
            };

            let line = format!("{}\n", serde_json::to_string(&tick).unwrap());
            file.write_all(line.as_bytes()).unwrap();
            hasher.update(line.as_bytes());
            tick_count += 1;
            
            current_start = ts + 1;
        }

        println!("Fetched {} ticks. Current ts: {}", tick_count, current_start);
        std::thread::sleep(std::time::Duration::from_millis(100)); // Respect rate limits
    }

    let final_hash = hasher.finalize();
    let hash_hex = final_hash.iter().map(|b| format!("{:02x}", b)).collect::<String>();

    let manifest = CaptureManifest {
        substrate: args.symbol,
        capture_start: args.start_time,
        capture_end: args.end_time,
        total_ticks: tick_count,
        chronology_hash: hash_hex.clone(),
        gaps,
    };

    let meta_path = base_dir.join(format!("{}_{}_manifest.json", args.name, args.start_time));
    let mut meta_file = File::create(&meta_path).unwrap();
    meta_file.write_all(serde_json::to_string_pretty(&manifest).unwrap().as_bytes()).unwrap();

    println!("✅ Historical Capture Complete: {} ticks", tick_count);
    println!("   Hash: {}", hash_hex);
    println!("   Directory: {:?}", base_dir);
}
