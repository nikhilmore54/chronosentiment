//! CLI wrapper around [`train_nse`](train_nse.rs): `clap` flags → env vars → same pipeline.
//!
//! Run from `core/` (or set `DATA_FOLDER` to your NSE CSV folder):
//! ```text
//! PATH="/opt/homebrew/bin:$PATH" cargo run --release --example training_nse -- \
//!   --pop 10 --gens 5 --slice 20 --deterministic --log-level info
//! ```

use clap::Parser;

#[path = "train_nse.rs"]
mod nse_train;

#[derive(Parser, Debug)]
#[command(name = "training_nse", about = "NSE multi-asset GA training (clap front-end)")]
struct Args {
    #[arg(long, default_value_t = 10)]
    pop: usize,
    #[arg(long, default_value_t = 5)]
    gens: usize,
    /// Train on at most this many assets (stable folder scan order). Use ≥15–20 for stable percentiles.
    #[arg(long, default_value_t = 20)]
    slice: usize,
    #[arg(long, action = clap::ArgAction::SetTrue)]
    deterministic: bool,
    #[arg(long, default_value = "info")]
    log_level: String,
    #[arg(long, help = "Override DATA_FOLDER (default: data/nse/5m or env)")]
    data_folder: Option<String>,
    /// If set, switches to validate mode (same as `train_nse` + VALIDATE_ON).
    #[arg(long)]
    validate_on: Option<String>,
}

fn main() {
    let args = Args::parse();
    std::env::set_var("GA_POPULATION_SIZE", args.pop.to_string());
    std::env::set_var("GA_GENERATIONS", args.gens.to_string());
    std::env::set_var("NSE_TRAIN_MAX_ASSETS", args.slice.to_string());
    if args.deterministic {
        std::env::set_var("NSE_DETERMINISTIC", "1");
    }
    std::env::set_var("RUST_LOG", args.log_level);
    if let Some(p) = args.data_folder {
        std::env::set_var("DATA_FOLDER", p);
    }
    if let Some(sym) = args.validate_on {
        std::env::set_var("RUN_MODE", "validate");
        std::env::set_var("VALIDATE_ON", sym);
    }
    nse_train::run_nse_training_cli();
}
