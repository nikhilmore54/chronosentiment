use std::fs::File;
use std::io::{Write, BufRead, BufReader};
use std::path::PathBuf;
use clap::Parser;
use sha2::{Sha256, Digest};
use serde::Deserialize;

use chronosentiment_core::topology::TopologyField;
use chronosentiment_core::cognition::CognitionGeometry;
use chronosentiment_core::morphology::{generate_occupancy_traces, TraceArtifactV1};
use chronosentiment_core::observatory::{ObservatoryManifestV1, ChronologyBounds};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Parser, Debug)]
#[command(author, version, about = "Canonical Observability Artifact Generator")]
struct Args {
    #[arg(short, long)]
    substrate: String,

    #[arg(long)]
    substrate_file: Option<String>,

    #[arg(short, long)]
    topology: String,

    #[arg(short, long)]
    cognition: String,
}

#[derive(Deserialize)]
struct ChronologyEvent {
    price: Option<f64>,
    close: Option<f64>,
}

fn parse_topology(ident: &str) -> TopologyField {
    match ident {
        "baseline" => TopologyField::Baseline,
        "plateau_low" => TopologyField::PlateauLow { occupancy: 0.2 },
        "impulse_shock" => TopologyField::ImpulseShock { at_tick: 2000, magnitude: 1.0 },
        "drift_field" => TopologyField::DriftField { min_acceptance: 0.1 },
        "fragmented_regime" => TopologyField::FragmentedRegime { switch_period: 10 },
        osc if osc.starts_with("osc_") => {
            let parts: Vec<&str> = osc.split('_').collect();
            let period = parts.get(1).map_or(50, |p| p.parse().unwrap_or(50));
            let amplitude = parts.get(2).map_or(1.0, |a| a.parse::<f64>().unwrap_or(1.0));
            TopologyField::Oscillatory { period, amplitude, noise: 0.0 }
        }
        _ => panic!("Unknown topology identifier: {}", ident),
    }
}

fn parse_cognition(ident: &str) -> CognitionGeometry {
    match ident {
        "rolling_50" => CognitionGeometry::RollingBounded { window: 50 },
        "rolling_100" => CognitionGeometry::RollingBounded { window: 100 },
        "event_reset" => CognitionGeometry::EventReset { drop_threshold_pct: 0.005 },
        "accumulator" => CognitionGeometry::Accumulator,
        _ => panic!("Unknown cognition identifier: {}", ident),
    }
}

fn main() {
    let args = Args::parse();

    // 1. Load the raw price substrate (mocked here for pure simulation, usually pulled from CSV/DB)
    // To maintain strict generic bounds, we'll simulate a 4320-tick random walk or read from existing substrate.
    // For pure architectural trace generation over synthetic topologies, we'll use a continuous mock series.
    let mut prices = Vec::new();
    
    if let Some(file_path) = &args.substrate_file {
        let file = File::open(file_path).expect("Failed to open substrate file");
        let reader = BufReader::new(file);
        for line in reader.lines() {
            let line = line.unwrap();
            let event: ChronologyEvent = serde_json::from_str(&line).expect("Failed to parse jsonl event");
            let p = event.price.unwrap_or_else(|| event.close.expect("Missing both price and close"));
            prices.push(p.max(1.0));
        }
    } else {
        let mut current_price = 40000.0;
        for i in 0..4320 {
            let hash_int = (i as u64).wrapping_mul(99887766554433).wrapping_add(12345);
            let step = ((hash_int % 100) as f64 / 100.0) * 10.0 - 5.0;
            current_price += step;
            prices.push(current_price.max(1.0));
        }
    }

    // Hash the substrate to bind the trace to the specific physical data
    let mut hasher = Sha256::new();
    for p in &prices {
        hasher.update(p.to_bits().to_le_bytes());
    }
    let substrate_hash = hasher.finalize().iter().map(|b| format!("{:02x}", b)).collect::<String>();

    if std::env::var("CHRONO_INJECT_FLOAT_COERCION").unwrap_or_else(|_| "0".to_string()) == "1" {
        println!("\nWARNING:\nExperimental divergence injection enabled:\nCHRONO_INJECT_FLOAT_COERCION=1\n");
        // Simulate internal float accumulation drift after tick 10
        for (i, p) in prices.iter_mut().enumerate() {
            if i >= 10 {
                // Add a microscopic rounding error that snowballs
                *p += 0.000001;
            }
        }
    }

    // 2. Parse Canonical Definitions
    let topology = parse_topology(&args.topology);
    let cognition = parse_cognition(&args.cognition);

    // 3. Generate Trace Artifact
    let traces = generate_occupancy_traces(&prices, topology, cognition);

    let artifact = TraceArtifactV1 {
        topology_identifier: args.topology.clone(),
        cognition_identifier: args.cognition.clone(),
        substrate_hash: substrate_hash.clone(),
        total_ticks: traces.len(),
        traces,
    };

    let out_json = serde_json::to_string_pretty(&artifact).unwrap();
    let artifact_hash = Sha256::digest(out_json.as_bytes()).iter().map(|b| format!("{:02x}", b)).collect::<String>();

    let manifest = ObservatoryManifestV1 {
        replay_version: "v1".to_string(),
        topology_version: "v1".to_string(),
        cognition_version: "v1".to_string(),
        commit_hash: "canonical".to_string(), // In production, inject via build script
        artifact_hash,
        generation_timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        chronology_bounds: ChronologyBounds {
            start_tick: 0,
            end_tick: artifact.total_ticks as u64 - 1,
            total_ticks: artifact.total_ticks,
        },
    };

    // 4. Output deterministic artifacts into Canonical Observatory Directory Structure
    let base_dir = PathBuf::from("artifacts")
        .join(&args.substrate)
        .join(&args.topology)
        .join(&args.cognition);
    
    std::fs::create_dir_all(&base_dir).unwrap();

    let trace_path = base_dir.join("trace_v1.json");
    if artifact.total_ticks < 500000 {
        let mut file = File::create(&trace_path).unwrap();
        file.write_all(out_json.as_bytes()).unwrap();
    }
    
    // Always compute and output a summary for fast parsing without disk explosion
    let sum_occupancy: f64 = artifact.traces.iter().map(|t| t.occupancy).sum();
    let mean_occupancy = if artifact.total_ticks > 0 { sum_occupancy / artifact.total_ticks as f64 } else { 0.0 };
    let max_occupancy = artifact.traces.iter().map(|t| t.occupancy).fold(0.0, f64::max);
    let persistence = artifact.traces.iter().filter(|t| t.occupancy > mean_occupancy).count();
    
    let summary = format!("{{\"max\": {}, \"persistence\": {}}}", max_occupancy, persistence);
    let summary_path = base_dir.join("trace_summary.json");
    let mut sum_file = File::create(&summary_path).unwrap();
    sum_file.write_all(summary.as_bytes()).unwrap();

    let meta_path = base_dir.join("metadata.json");
    let meta_json = serde_json::to_string_pretty(&manifest).unwrap();
    let mut meta_file = File::create(&meta_path).unwrap();
    meta_file.write_all(meta_json.as_bytes()).unwrap();

    let hash_path = base_dir.join("replay_hash.txt");
    let mut hash_file = File::create(&hash_path).unwrap();
    hash_file.write_all(substrate_hash.as_bytes()).unwrap();

    println!("✅ Generated Canonical Observability Artifact");
    println!("   Topology : {}", args.topology);
    println!("   Cognition: {}", args.cognition);
    println!("   Ticks    : {}", artifact.total_ticks);
    println!("   Directory: {:?}", base_dir);
}
