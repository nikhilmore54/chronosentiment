use chronosentiment_core::ese::ExecutionEngine;
use chronosentiment_core::{MarketEvent, MarketEventType, Side};
use coralys_ecology::models::{CognitionGeometry, MemoryState};
use coralys_ecology::traits::MemoryModel;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct CryptoStateRecord {
    timestamp: i64,
    price: f64,
    volume_acceleration_60m: f64,
    persistence_240m: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct Phase3TraceRecord {
    timestamp: i64,
    price: f64,
    volume_acceleration_60m: f64,
    persistence_240m: f64,
    overlap_ratio: f64,
    trajectory_state: String,
    mfe: f64,
    mae: f64,
}

fn process_phase3(records: &[CryptoStateRecord], market: &[MarketEvent]) -> Vec<Phase3TraceRecord> {
    let mut baseline_memory = MemoryState::new(CognitionGeometry::RollingBounded { window: 1440 });
    let mut experimental_memory = MemoryState::new(CognitionGeometry::RollingBounded { window: 1440 });
    let mut engine = ExecutionEngine::default();
    
    let mut joined_trace = Vec::with_capacity(records.len());
    
    for (i, record) in records.iter().enumerate() {
        baseline_memory.observe(record.price);
        experimental_memory.observe(record.price);
        let overlap = experimental_memory.overlap_ratio(&baseline_memory);
        
        let entry_price = record.price.round() as u64;
        let tp_target = (record.price * 1.05).round() as u64;
        let sl_target = (record.price * 0.95).round() as u64;
        
        // Explicitly defined trajectory: Enter Long at every tick, hold max 60 bars.
        let exec_result = engine.simulate_round_trip(
            entry_price,
            tp_target,
            sl_target,
            Side::Buy,
            1,
            market,
            i,
            60,
        );
        
        joined_trace.push(Phase3TraceRecord {
            timestamp: record.timestamp,
            price: record.price,
            volume_acceleration_60m: record.volume_acceleration_60m,
            persistence_240m: record.persistence_240m,
            overlap_ratio: overlap,
            trajectory_state: format!("{:?}", exec_result.exit_reason),
            mfe: exec_result.mfe,
            mae: exec_result.mae,
        });
    }
    
    joined_trace
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let workspace = Path::new("/Users/nikhil/ChronoSentiment_MEGA_FINAL");
    let dataset = workspace.join("datasets/crypto_integration_trace_v0.jsonl");

    let file = File::open(&dataset)?;
    let reader = BufReader::new(file);

    let mut original_records = Vec::new();
    let mut market_events = Vec::new();
    
    println!("Phase 3: Behaviour (Explicit Trajectory)");
    println!("Reading JSONL from: {}", dataset.display());

    for line in reader.lines() {
        let line = line?;
        let record: CryptoStateRecord = serde_json::from_str(&line)?;
        
        market_events.push(MarketEvent {
            subtype: MarketEventType::Trade,
            price: record.price.round() as u64,
            quantity: 1,
            side: Some(Side::Buy),
            exchange_ts: record.timestamp as u64,
        });
        
        original_records.push(record);
    }

    let trace_run_1 = process_phase3(&original_records, &market_events);
    let trace_run_2 = process_phase3(&original_records, &market_events);

    // Hashing helper
    use sha2::{Digest, Sha256};
    fn hash_trace(trace: &[Phase3TraceRecord]) -> String {
        let mut hasher = Sha256::new();
        for record in trace {
            let json_str = serde_json::to_string(record).unwrap();
            hasher.update(json_str.as_bytes());
            hasher.update(b"\n");
        }
        let hash_bytes = hasher.finalize();
        hash_bytes.iter().map(|b| format!("{:02x}", b)).collect::<String>()
    }

    let hash1 = hash_trace(&trace_run_1);
    let hash2 = hash_trace(&trace_run_2);

    println!("\nVerification Results:");
    println!(" - Deterministic Joined Trace (Run 1 == Run 2): {}", hash1 == hash2);
    println!(" - Hash Run 1: {}", hash1);
    
    println!("\nFirst 3 Joined Trace Records:");
    for r in trace_run_1.iter().take(3) {
        println!(" - t={}, px={:.2}, VA={:.4}, persistence={:.4}, overlap={:.4}, traj={}, MFE={:.4}, MAE={:.4}", 
                 r.timestamp, r.price, r.volume_acceleration_60m, r.persistence_240m, r.overlap_ratio, r.trajectory_state, r.mfe, r.mae);
    }
    
    println!("\nLast 3 Joined Trace Records:");
    for r in trace_run_1.iter().rev().take(3).rev() {
        println!(" - t={}, px={:.2}, VA={:.4}, persistence={:.4}, overlap={:.4}, traj={}, MFE={:.4}, MAE={:.4}", 
                 r.timestamp, r.price, r.volume_acceleration_60m, r.persistence_240m, r.overlap_ratio, r.trajectory_state, r.mfe, r.mae);
    }

    Ok(())
}
