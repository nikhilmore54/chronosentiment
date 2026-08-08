/// RP-406C: Benchmark Characterisation
///
/// Loads the RP-406B solution for every setA instance, computes the full
/// sorted load vector (utilisation per link, descending), and exports:
///   1. Per-instance CSV: setA-{nn}-loadvec-rp406b.csv
///   2. Combined CSV:     rp406c_all_loadvecs.csv
///   3. MLU summary table to stdout
///
/// Usage:
///   rp406c_characterise [--set-dir <path>] [--out-dir <path>] [--top <n>]
///
/// The comparison against published best vectors is done in a separate
/// Python/R script that reads the combined CSV.

use std::collections::HashMap;
use std::fs::File;
use std::io::Write;

use roadef::loader::{load_network, load_traffic_matrix, load_scenario};
use roadef::models::{Solution, SrPath};
use roadef::evaluator::RoadefEvaluator;

// ── Config ────────────────────────────────────────────────────────────────────

struct Config {
    set_dir: String,
    out_dir: String,
    top_n:   usize,
}

impl Config {
    fn from_args() -> Self {
        let args: Vec<String> = std::env::args().collect();
        let mut set_dir = "adapters/roadef/repo/challenge-roadef-2026-main/setA".to_string();
        let mut out_dir = set_dir.clone();
        let mut top_n   = 30usize;
        let mut i = 1;
        while i < args.len() {
            match args[i].as_str() {
                "--set-dir" => { i += 1; if i < args.len() { set_dir = args[i].clone(); } }
                "--out-dir" => { i += 1; if i < args.len() { out_dir = args[i].clone(); } }
                "--top"     => { i += 1; if i < args.len() { top_n = args[i].parse().unwrap_or(30); } }
                _ => {}
            }
            i += 1;
        }
        Config { set_dir, out_dir, top_n }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn load_srpaths(path: &str) -> anyhow::Result<Vec<SrPath>> {
    let raw = std::fs::read_to_string(path)?;
    let v: serde_json::Value = serde_json::from_str(&raw)?;
    let mut srpaths = Vec::new();
    if let Some(arr) = v["srpaths"].as_array() {
        for item in arr {
            let d = item["d"].as_u64().unwrap_or(0) as usize;
            let t = item["t"].as_u64().unwrap_or(0) as usize;
            let w: Vec<u64> = item["w"].as_array()
                .map(|a| a.iter().filter_map(|x| x.as_u64()).collect())
                .unwrap_or_default();
            srpaths.push(SrPath { d, t, w });
        }
    }
    Ok(srpaths)
}

fn compute_sorted_util(
    ev: &RoadefEvaluator,
    srpaths: &[SrPath],
    ns: usize,
    cap: &HashMap<u64, f64>,
) -> Vec<f64> {
    let sol = Solution { srpaths: srpaths.to_vec() };
    let mut combined: HashMap<u64, f64> = HashMap::new();
    for t in 0..ns {
        if let Some(loads) = ev.compute_loads(t, &sol) {
            for (id, flow) in &loads.arc_flows {
                let c = cap.get(id).copied().unwrap_or(1.0);
                let s = if c > 0.0 { flow / c } else { f64::INFINITY };
                let e = combined.entry(*id).or_insert(0.0);
                if s > *e { *e = s; }
            }
        }
    }
    let mut v: Vec<f64> = combined.values().copied().collect();
    v.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
    v
}

// ── Main ──────────────────────────────────────────────────────────────────────

fn main() -> anyhow::Result<()> {
    let cfg = Config::from_args();

    std::fs::create_dir_all(&cfg.out_dir)?;

    // Combined CSV header
    let combined_path = format!("{}/rp406c_all_loadvecs.csv", cfg.out_dir);
    let mut combined_csv = File::create(&combined_path)?;
    writeln!(combined_csv, "instance,rank,load")?;

    // MLU summary header
    println!("{:<12}  {:>16}  {:>10}  {:>6}  {:>10}  {:>8}",
        "Instance", "Objective", "MLU", "Valid", "Overloaded", "Links");
    println!("{}", "-".repeat(72));

    for n in 1..=20usize {
        let inst = format!("{:02}", n);

        // Load instance files
        let net = match load_network(&format!("{}/setA-{}-net.json", cfg.set_dir, inst)) {
            Ok(x) => x,
            Err(e) => { eprintln!("setA-{}: failed to load network: {}", inst, e); continue; }
        };
        let tm = match load_traffic_matrix(&format!("{}/setA-{}-tm.json", cfg.set_dir, inst)) {
            Ok(x) => x,
            Err(e) => { eprintln!("setA-{}: failed to load TM: {}", inst, e); continue; }
        };
        let sc = match load_scenario(&format!("{}/setA-{}-scenario.json", cfg.set_dir, inst)) {
            Ok(x) => x,
            Err(e) => { eprintln!("setA-{}: failed to load scenario: {}", inst, e); continue; }
        };

        let ns = tm.num_time_slots;
        let ev = RoadefEvaluator::new(&net, tm, sc);

        // Build capacity map
        let cap: HashMap<u64, f64> = net.links.iter()
            .map(|l| (l.id, l.capacity as f64))
            .collect();

        // Load RP-406B solution
        let sol_path = format!("{}/setA-{}-srpaths-rp406b.json", cfg.set_dir, inst);
        let srpaths = match load_srpaths(&sol_path) {
            Ok(x) => x,
            Err(e) => { eprintln!("setA-{}: failed to load solution: {}", inst, e); continue; }
        };

        // Evaluate
        let sol = Solution { srpaths: srpaths.clone() };
        let result = ev.evaluate_solution(&sol);

        // Compute sorted utilisation vector
        let sorted_util = compute_sorted_util(&ev, &srpaths, ns, &cap);
        let mlu = sorted_util.first().copied().unwrap_or(0.0);
        let overloaded = sorted_util.iter().filter(|&&v| v >= 1.0).count();
        let n_links = sorted_util.len();

        // Print MLU summary line
        let obj_str = if result.obj.is_finite() {
            format!("{:.6}", result.obj)
        } else {
            "inf".to_string()
        };
        println!("setA-{:<8}  {:>16}  {:>10.6}  {:>6}  {:>10}  {:>8}",
            inst, obj_str, mlu, result.valid, overloaded, n_links);

        // Per-instance CSV
        let csv_path = format!("{}/setA-{}-loadvec-rp406b.csv", cfg.out_dir, inst);
        let mut csv = File::create(&csv_path)?;
        writeln!(csv, "instance,rank,load")?;
        for (i, &v) in sorted_util.iter().enumerate() {
            writeln!(csv, "setA-{},{},{:.9}", inst, i + 1, v)?;
            writeln!(combined_csv, "setA-{},{},{:.9}", inst, i + 1, v)?;
        }

        // Top-N to stderr for inspection
        let top = cfg.top_n.min(sorted_util.len());
        eprintln!("  setA-{} top-{} load vector:", inst, top);
        for (i, &v) in sorted_util.iter().take(top).enumerate() {
            eprintln!("    {:3}  {:.6}", i + 1, v);
        }
    }

    println!("{}", "-".repeat(72));
    println!("Combined load vector CSV: {}", combined_path);

    Ok(())
}