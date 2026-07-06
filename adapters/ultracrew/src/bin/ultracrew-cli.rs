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

/// Command‑line arguments for the UltraCrew CLI.
#[derive(Parser, Debug)]
#[command(name = "ultracrew-cli", author, version, about = "Run the UltraCrew optimizer on a dataset")]
struct Cli {
    /// Path to the input dataset (JSON format)
    #[arg(short, long, value_name = "FILE")]
    input: PathBuf,

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
    let file = File::open(&args.input)
        .map_err(|e| anyhow::anyhow!("Failed to open input file {}: {}", args.input.display(), e))?;
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
    let solution = ultracrew::pipeline::run_pipeline(context, config)
        .map_err(|e| anyhow::anyhow!("Optimization pipeline failed: {}", e))?;

    // 5. Output schedule solution to JSON
    let out_file = File::create(&args.output)
        .map_err(|e| anyhow::anyhow!("Failed to create output file {}: {}", args.output.display(), e))?;
    serde_json::to_writer_pretty(out_file, &solution)
        .map_err(|e| anyhow::anyhow!("Failed to write schedule JSON: {}", e))?;

    eprintln!("✅ Optimization complete – schedule written to {}", args.output.display());
    Ok(())
}
