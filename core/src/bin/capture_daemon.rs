use futures_util::StreamExt;
use sha2::{Digest, Sha256};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
struct BinanceAggTrade {
    #[serde(rename = "e")]
    event_type: String,
    #[serde(rename = "E")]
    event_time: u64,
    #[serde(rename = "s")]
    symbol: String,
    #[serde(rename = "p")]
    price: String,
    #[serde(rename = "q")]
    quantity: String,
    #[serde(rename = "m")]
    is_buyer_maker: bool,
}

#[derive(Debug, Serialize)]
struct NormalizedTick {
    symbol: String,
    timestamp: u64,
    price: f64,
    volume: f64,
    is_buyer_maker: bool,
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

#[tokio::main]
async fn main() {
    let symbol = "btcusdt";
    let ws_url = format!("wss://stream.binance.com:9443/ws/{}@aggTrade", symbol);

    println!("Starting Stage 1: Live Chronology Capture for {}", symbol.to_uppercase());
    println!("Connecting to {}...", ws_url);

    let base_dir = PathBuf::from("chronology").join("live_capture");
    std::fs::create_dir_all(&base_dir).unwrap();

    let rotation_interval_secs = 3600; 
    let mut current_rotation_start = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

    let mut file_path = base_dir.join(format!("{}_{}.jsonl", symbol, current_rotation_start));
    let mut file = OpenOptions::new().create(true).append(true).open(&file_path).unwrap();

    let mut hasher = Sha256::new();
    let mut tick_count = 0;
    let mut gaps: Vec<CaptureGap> = Vec::new();
    let mut last_tick_time = current_rotation_start;

    loop {
        let (ws_stream, _) = match connect_async(&ws_url).await {
            Ok(stream) => stream,
            Err(e) => {
                println!("Connection error: {}. Retrying in 5s...", e);
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                continue;
            }
        };
        println!("Connected. Capturing dumb chronology ticks...");
        let (_, mut read) = ws_stream.split();

        while let Some(msg_result) = read.next().await {
            match msg_result {
                Ok(Message::Text(text)) => {
                    if let Ok(raw_trade) = serde_json::from_str::<BinanceAggTrade>(&text) {
                        let tick = NormalizedTick {
                            symbol: raw_trade.symbol.clone(),
                            timestamp: raw_trade.event_time,
                            price: raw_trade.price.parse().unwrap_or(0.0),
                            volume: raw_trade.quantity.parse().unwrap_or(0.0),
                            is_buyer_maker: raw_trade.is_buyer_maker,
                        };

                        let line = format!("{}\n", serde_json::to_string(&tick).unwrap());
                        file.write_all(line.as_bytes()).unwrap();
                        hasher.update(line.as_bytes());
                        tick_count += 1;
                        last_tick_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

                        let now = last_tick_time;
                        if now - current_rotation_start >= rotation_interval_secs {
                            // Finalize current rotation
                            let final_hash = std::mem::replace(&mut hasher, Sha256::new()).finalize();
                            let hash_hex = final_hash.iter().map(|b| format!("{:02x}", b)).collect::<String>();
                            let manifest = CaptureManifest {
                                substrate: raw_trade.symbol.clone(),
                                capture_start: current_rotation_start,
                                capture_end: now,
                                total_ticks: tick_count,
                                chronology_hash: hash_hex.clone(),
                                gaps: std::mem::take(&mut gaps),
                            };

                            let meta_path = base_dir.join(format!("{}_{}_manifest.json", symbol, current_rotation_start));
                            let mut meta_file = File::create(&meta_path).unwrap();
                            meta_file.write_all(serde_json::to_string_pretty(&manifest).unwrap().as_bytes()).unwrap();

                            println!("Rotated Archive: {} ({} ticks) [Hash: {}]", file_path.display(), tick_count, hash_hex);

                            // Start new rotation
                            current_rotation_start = now;
                            file_path = base_dir.join(format!("{}_{}.jsonl", symbol, current_rotation_start));
                            file = OpenOptions::new().create(true).append(true).open(&file_path).unwrap();
                            tick_count = 0;
                        }
                    }
                }
                Ok(Message::Close(_)) | Err(_) => {
                    println!("Websocket disconnected. Recording gap and reconnecting...");
                    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
                    gaps.push(CaptureGap {
                        gap_start: last_tick_time,
                        gap_end: now,
                        reason: "Websocket disconnect or error".to_string(),
                    });
                    break;
                }
                _ => {}
            }
        }
    }
}
