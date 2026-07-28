// ultracrew-cli.rs – production command‑line interface for UltraCrew
// ---------------------------------------------------------------
// This binary provides the canonical entry point for the UltraCrew product.

use std::path::PathBuf;
use clap::{Parser, ValueEnum};
use std::fs::File;
use std::io::BufReader;
use serde_json;
use ultracrew::public_contracts::ScheduleRequest;
use ultracrew::config::OptimizationProfile;
use ultracrew::health::health_check;

/// Command‑line arguments for the UltraCrew CLI.
#[derive(Parser, Debug)]
#[command(name = "ultracrew-cli", author, version, about = "Run the UltraCrew optimizer on a dataset")]
struct Cli {
    /// Run adapter health checks and print a JSON status report, then exit.
    /// Can be used without --input. Exit code 0 = ok, 1 = degraded or error.
    #[arg(long, action)]
    health: bool,

    /// Path to the input dataset (JSON format)
    #[arg(short, long, value_name = "FILE")]
    input: Option<PathBuf>,

    /// Path to write the output schedule (JSON)
    #[arg(short, long, value_name = "FILE", default_value = "schedule.json")]
    output: PathBuf,

    /// Select optimisation profile (fast, balanced, thorough, research)
    #[arg(long, value_enum, default_value = "balanced")]
    profile: OptimizationProfile,

    /// Optional override for population size of the GA (research mode)
    #[arg(long)]
    population: Option<usize>,

    /// Optional override for generation limit of the GA (research mode)
    #[arg(long)]
    generations: Option<usize>,

    /// Print a short list of available optimisation profiles and exit
    #[arg(long, action)]
    list_profiles: bool,

    /// Show the resolved EvolutionConfig for the selected profile (including any overrides) and exit
    #[arg(long, action)]
    show_config: bool,
}

fn main() -> anyhow::Result<()> {
    // Parse CLI arguments
    let args = Cli::parse();

    // Handle --health flag (standalone; does not require --input)
    if args.health {
        let resp = health_check();
        println!("{}", resp.to_json());
        if resp.is_ok() {
            std::process::exit(0);
        } else {
            std::process::exit(1);
        }
    }

    // Print banner (to stderr)
    eprintln!("====================================================");
    eprintln!("UltraCrew Optimizer – Demo Configuration Standard v1");
    eprintln!("====================================================");

    // Handle list_profiles flag
    if args.list_profiles {
        eprintln!("Available optimisation profiles:");
        // Iterate over all variants
        for profile in OptimizationProfile::value_variants() {
            // Get the CLI name (lowercase) and description
            let name = profile.to_possible_value().unwrap().get_name().to_owned();
        eprintln!("  {} - {}", name, profile.description());
        }
        return Ok(());
    }

    // 1. Load the dataset into ScheduleRequest
    let input_path = args.input.ok_or_else(|| {
        anyhow::anyhow!("--input is required unless --health, --list-profiles, or --show-config is used")
    })?;
    let file = File::open(&input_path)
        .map_err(|e| anyhow::anyhow!("Failed to open input file {}: {}", input_path.display(), e))?;
    let reader = BufReader::new(file);
    let request: ScheduleRequest = serde_json::from_reader(reader)
        .map_err(|e| anyhow::anyhow!("Failed to deserialize input JSON: {}", e))?;

    // 2. Convert to internal ScheduleContext and validate inputs
    let context = request.to_context();
    if let Err(e) = ultracrew::constraint_engine::validate_context(&context) {
        anyhow::bail!("Dataset validation failed: {}", e);
    }

    // 3. Build EvolutionConfig (including overrides)
    let mut config = args.profile.config();
    // Optional overrides for research experiments
    if let Some(pop) = args.population {
        config.population_size = pop;
    }
    if let Some(gen) = args.generations {
        config.generation_limit = gen;
    }
    // Handle show_config flag
    if args.show_config {
        eprintln!("Resolved EvolutionConfig: {}", serde_json::to_string_pretty(&config)?);
        return Ok(());
    }


    // 4. Run pipeline
    let start = std::time::Instant::now();
    let solution = ultracrew::pipeline::run_pipeline(context.clone(), config)
        .map_err(|e| anyhow::anyhow!("Optimization pipeline failed: {}", e))?;
    let elapsed = start.elapsed();

    // 5. Compute summary KPIs
    let total_shifts = context.shifts.len();
    let assigned_shifts = solution.assignments.len();
    let coverage_pct = if total_shifts > 0 {
        (assigned_shifts as f64 / total_shifts as f64) * 100.0
    } else {
        0.0
    };

    // Compute per-worker hours for workload balance
    let mut worker_hours: std::collections::HashMap<u64, u64> = std::collections::HashMap::new();
    for (&shift_id, &worker_id) in &solution.assignments {
        if let Some(shift) = context.shifts.iter().find(|s| s.id == shift_id) {
            *worker_hours.entry(worker_id).or_insert(0) += shift.duration_hours;
        }
    }
    let hours_values: Vec<f64> = worker_hours.values().map(|&h| h as f64).collect();
    let mean_hours = if hours_values.is_empty() { 0.0 } else {
        hours_values.iter().sum::<f64>() / hours_values.len() as f64
    };
    let max_hours = hours_values.iter().cloned().fold(0.0_f64, f64::max);
    let min_hours = hours_values.iter().cloned().fold(f64::MAX, f64::min);

    // 6. Print summary
    eprintln!();
    eprintln!("────────────────────────────────────────────────────");
    eprintln!("  UltraCrew Optimization Summary");
    eprintln!("────────────────────────────────────────────────────");
    eprintln!("  Scenario:          {}", input_path.display());
    eprintln!("  Workers:           {}", context.workers.len());
    eprintln!("  Shifts:            {}", total_shifts);
    eprintln!("  Profile:           {:?}", args.profile);
    eprintln!("────────────────────────────────────────────────────");
    eprintln!("  Coverage:          {}/{} shifts ({:.1}%)", assigned_shifts, total_shifts, coverage_pct);
    eprintln!("  Hard violations:   {}", solution.hard_violations);
    eprintln!("  Rest violations:   {}", solution.rest_violations);
    eprintln!("  Fitness score:     {:.4}", solution.fitness);
    eprintln!("  Fairness penalty:  {:.4}", solution.fairness_penalty);
    eprintln!("  Fatigue penalty:   {:.4}", solution.fatigue_penalty);
    eprintln!("────────────────────────────────────────────────────");
    eprintln!("  Workload balance:");
    eprintln!("    Mean hours/worker: {:.1}h", mean_hours);
    eprintln!("    Min hours/worker:  {:.1}h", if min_hours == f64::MAX { 0.0 } else { min_hours });
    eprintln!("    Max hours/worker:  {:.1}h", max_hours);
    eprintln!("────────────────────────────────────────────────────");
    eprintln!("  Runtime:           {:.2}s", elapsed.as_secs_f64());
    eprintln!("  Output:            {}", args.output.display());
    eprintln!("────────────────────────────────────────────────────");

    // 7. Output schedule solution to JSON
    let out_file = File::create(&args.output)
        .map_err(|e| anyhow::anyhow!("Failed to create output file {}: {}", args.output.display(), e))?;
    serde_json::to_writer_pretty(out_file, &solution)
        .map_err(|e| anyhow::anyhow!("Failed to write schedule JSON: {}", e))?;

    eprintln!("✅ Optimization complete – schedule written to {}", args.output.display());
    Ok(())
}
