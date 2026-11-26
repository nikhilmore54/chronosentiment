//! Pure-Rust OR-Library Benchmark Harness and Data Harvester.
//!
//! Replaces legacy Python scripts with native, high-performance Rust execution:
//! - Parses and verifies raw OR-Library instances (.txt)
//! - Exports structured benchmark matrices (.json)
//! - Executes multi-seed evaluation suites across Northwest SPP and Aircraft Landing
//! - Generates CSV and JSON telemetry for publication and statistical audits

use coralys_airline::orlib::airland::{AircraftLandingEvaluator, AirlandInstance};
use coralys_airline::orlib::spp::{SetPartitioningEvaluator, SppInstance};
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Instant;

#[derive(Debug, Serialize, Deserialize)]
struct SppExportColumn {
    id: usize,
    cost: u64,
    rows: Vec<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SppExportFile {
    instance: String,
    #[serde(rename = "type")]
    inst_type: String,
    num_rows: usize,
    num_cols: usize,
    certified_optimum: Option<u64>,
    columns: Vec<SppExportColumn>,
}

#[derive(Debug, Serialize, Deserialize)]
struct AlpExportPlane {
    id: usize,
    appearance_time: f64,
    earliest_time: f64,
    target_time: f64,
    latest_time: f64,
    penalty_early: f64,
    penalty_late: f64,
    separation: Vec<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
struct AlpExportFile {
    instance: String,
    #[serde(rename = "type")]
    inst_type: String,
    num_planes: usize,
    freeze_time: f64,
    certified_optimum: Option<f64>,
    planes: Vec<AlpExportPlane>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BenchmarkRecord {
    pub category: String,
    pub instance: String,
    pub dimensions: String,
    pub seed: u64,
    pub cost_z: f64,
    pub cert_opt_z: f64,
    pub gap_pct: f64,
    pub time_ms: f64,
    pub is_feasible: bool,
}

fn get_repo_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

fn harvest_and_export_json(raw_dir: &Path, data_dir: &Path) -> std::io::Result<()> {
    let spp_raw = raw_dir.join("spp");
    let alp_raw = raw_dir.join("airland");
    let spp_data = data_dir.join("spp");
    let alp_data = data_dir.join("airland");

    fs::create_dir_all(&spp_data)?;
    fs::create_dir_all(&alp_data)?;

    // 1. Export SPP JSONs
    if spp_raw.exists() {
        println!("Exporting SPP benchmark instances to JSON...");
        for i in 1..=43 {
            let name = format!("sppnw{:02}", i);
            let raw_path = spp_raw.join(format!("{}.txt", name));
            if !raw_path.exists() {
                continue;
            }
            if let Ok(inst) = SppInstance::from_file(&raw_path) {
                let export = SppExportFile {
                    instance: name.clone(),
                    inst_type: "set_partitioning".to_string(),
                    num_rows: inst.num_rows,
                    num_cols: inst.num_cols,
                    certified_optimum: inst.certified_optimum(),
                    columns: inst
                        .columns
                        .into_iter()
                        .map(|c| SppExportColumn {
                            id: c.id,
                            cost: c.cost,
                            rows: c.rows,
                        })
                        .collect(),
                };
                let json_path = spp_data.join(format!("{}.json", name));
                let json_str = serde_json::to_string(&export).unwrap();
                let mut f = File::create(json_path)?;
                f.write_all(json_str.as_bytes())?;
            }
        }
    }

    // 2. Export ALP JSONs
    if alp_raw.exists() {
        println!("Exporting ALP benchmark instances to JSON...");
        for i in 1..=13 {
            let name = format!("airland{}", i);
            let raw_path = alp_raw.join(format!("{}.txt", name));
            if !raw_path.exists() {
                continue;
            }
            if let Ok(inst) = AirlandInstance::from_file(&raw_path) {
                let export = AlpExportFile {
                    instance: name.clone(),
                    inst_type: "aircraft_landing".to_string(),
                    num_planes: inst.num_planes,
                    freeze_time: inst.freeze_time,
                    certified_optimum: inst.certified_optimum(),
                    planes: inst
                        .planes
                        .into_iter()
                        .map(|p| AlpExportPlane {
                            id: p.id,
                            appearance_time: p.appearance_time,
                            earliest_time: p.earliest_time,
                            target_time: p.target_time,
                            latest_time: p.latest_time,
                            penalty_early: p.penalty_early_per_sec,
                            penalty_late: p.penalty_late_per_sec,
                            separation: p.separation_times,
                        })
                        .collect(),
                };
                let json_path = alp_data.join(format!("{}.json", name));
                let json_str = serde_json::to_string(&export).unwrap();
                let mut f = File::create(json_path)?;
                f.write_all(json_str.as_bytes())?;
            }
        }
    }

    Ok(())
}

fn run_benchmarks(raw_dir: &Path, out_dir: &Path) {
    let alp_eval = AircraftLandingEvaluator::default();
    let spp_eval = SetPartitioningEvaluator::default();

    let mut records: Vec<BenchmarkRecord> = Vec::new();

    println!("\n=========================================================================================");
    println!("CORALYS OR-LIBRARY BENCHMARK RUNNER (PURE RUST NATIVE ENGINE)");
    println!("=========================================================================================\n");

    // 1. Aircraft Landing Problem
    println!("--- [1/2] AIRCRAFT LANDING BENCHMARKS (airland1 - airland8) ---");
    println!(
        "{:<10} {:>6} | {:>9} {:>9} | {:>7} {:>6} | {:>8}",
        "Instance", "Planes", "Coralys_Z", "Cert_Z*", "Gap_%", "Feas", "Time_ms"
    );
    println!("{}", "-".repeat(68));

    let alp_dir = raw_dir.join("airland");
    for i in 1..=8 {
        let name = format!("airland{}", i);
        let path = alp_dir.join(format!("{}.txt", name));
        if !path.exists() {
            continue;
        }

        if let Ok(inst) = AirlandInstance::from_file(&path) {
            let cert = inst.certified_optimum().unwrap_or(0.0);
            let t0 = Instant::now();
            let sol = alp_eval.solve(&inst);
            let ms = t0.elapsed().as_secs_f64() * 1000.0;
            let gap = sol.optimality_gap(cert);

            println!(
                "{:<10} {:>6} | {:>9.1} {:>9.1} | {:>6.2}% {:>6} | {:>7.2}ms",
                name,
                inst.num_planes,
                sol.total_penalty,
                cert,
                gap,
                if sol.is_feasible { "YES" } else { "NO" },
                ms
            );

            records.push(BenchmarkRecord {
                category: "Aircraft_Landing".to_string(),
                instance: name,
                dimensions: format!("{} planes", inst.num_planes),
                seed: 42,
                cost_z: sol.total_penalty,
                cert_opt_z: cert,
                gap_pct: gap,
                time_ms: ms,
                is_feasible: sol.is_feasible,
            });
        }
    }

    // 2. Northwest Airlines Set Partitioning
    println!("\n--- [2/2] NORTHWEST AIRLINES SET PARTITIONING (sppnw01 - sppnw43) ---");
    println!(
        "{:<10} {:>5} {:>7} | {:>9} {:>9} | {:>7} {:>6} | {:>8}",
        "Instance", "Rows", "Cols", "Coralys_Z", "Cert_Z*", "Gap_%", "Feas", "Time_ms"
    );
    println!("{}", "-".repeat(78));

    let spp_dir = raw_dir.join("spp");
    for i in 1..=43 {
        let name = format!("sppnw{:02}", i);
        let path = spp_dir.join(format!("{}.txt", name));
        if !path.exists() {
            continue;
        }

        if let Ok(inst) = SppInstance::from_file(&path) {
            let cert = inst.certified_optimum().unwrap_or(0);
            let t0 = Instant::now();
            let sol = spp_eval.solve(&inst, 5, 42);
            let ms = t0.elapsed().as_secs_f64() * 1000.0;
            let gap = sol.optimality_gap(cert);

            println!(
                "{:<10} {:>5} {:>7} | {:>9} {:>9} | {:>6.2}% {:>6} | {:>7.2}ms",
                name,
                inst.num_rows,
                inst.num_cols,
                sol.total_cost,
                cert,
                gap,
                if sol.is_feasible() { "YES" } else { "NO" },
                ms
            );

            records.push(BenchmarkRecord {
                category: "Set_Partitioning".to_string(),
                instance: name,
                dimensions: format!("{}x{}", inst.num_rows, inst.num_cols),
                seed: 42,
                cost_z: sol.total_cost as f64,
                cert_opt_z: cert as f64,
                gap_pct: gap,
                time_ms: ms,
                is_feasible: sol.is_feasible(),
            });
        }
    }

    // Export Telemetry
    let json_telemetry = out_dir.join("benchmark_results.json");
    if let Ok(serialized) = serde_json::to_string_pretty(&records) {
        let _ = fs::write(&json_telemetry, serialized);
        println!("\nTelemetry exported -> {:?}", json_telemetry);
    }

    let csv_telemetry = out_dir.join("benchmark_results.csv");
    if let Ok(mut wtr) = csv::Writer::from_path(&csv_telemetry) {
        for r in &records {
            let _ = wtr.serialize(r);
        }
        let _ = wtr.flush();
        println!("CSV exported       -> {:?}", csv_telemetry);
    }
}

fn main() {
    let repo_root = get_repo_root();
    let raw_dir = repo_root.join("benchmarks").join("orlib");
    let data_dir = repo_root.join("data").join("benchmarks").join("orlib");

    let args: Vec<String> = std::env::args().collect();
    let cmd = args.get(1).map(|s| s.as_str()).unwrap_or("all");

    match cmd {
        "export" => {
            if let Err(e) = harvest_and_export_json(&raw_dir, &data_dir) {
                eprintln!("Error exporting JSON: {}", e);
            }
        }
        "run" => {
            run_benchmarks(&raw_dir, &data_dir);
        }
        "all" | _ => {
            let _ = harvest_and_export_json(&raw_dir, &data_dir);
            run_benchmarks(&raw_dir, &data_dir);
        }
    }
}
