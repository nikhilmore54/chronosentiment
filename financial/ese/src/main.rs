use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use cs_ingest::dedupe::DedupeIndex;
use cs_ingest::frozen_loader::{load_frozen_cohort, CANDLE_ROOT};
use cs_ingest::repair::{cmd_detect, cmd_process, cmd_queue, cmd_status, RepairConfig, BAR_SEC};
use cs_ingest::replay::{run_replay_step, ReplayStepConfig};
use cs_ingest::timeline::align_timeline;
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "cs-ingest",
    about = "Deterministic replay-bound ingest utilities"
)]
struct Cli {
    #[command(subcommand)]
    cmd: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Load frozen substrate and print timeline alignment (parity with Python freeze).
    Timeline {
        #[arg(long)]
        batch_id: u32,
        #[arg(long, default_value = "cohorts/batch_003.txt")]
        cohort: PathBuf,
        #[arg(long, default_value = CANDLE_ROOT)]
        candle_root: PathBuf,
    },
    /// Verify dedupe index against archive raw telemetry (integrity check).
    DedupeVerify {
        #[arg(long)]
        archive: PathBuf,
    },
    /// End-to-end frozen replay: barrier → observatory → dedupe → archive.
    ReplayStep {
        #[arg(long)]
        batch_id: u32,
        #[arg(long, default_value = "cohorts/batch_003.txt")]
        cohort: PathBuf,
        #[arg(long)]
        archive: PathBuf,
        #[arg(long, default_value = CANDLE_ROOT)]
        candle_root: PathBuf,
        #[arg(
            long,
            default_value = "observatory/provider_clustering_pca_weights.json"
        )]
        pca_weights: PathBuf,
        #[arg(long, default_value = "./target/release/examples/live_observatory")]
        observatory: PathBuf,
        #[arg(long, default_value = "0")]
        start_interval: usize,
        #[arg(long)]
        max_intervals: Option<usize>,
        #[arg(long)]
        resume: bool,
        #[arg(long)]
        rebuild_dedupe: bool,
        #[arg(
            long,
            help = "Wipe archive telemetry layers before ingest (parity / replay verify)"
        )]
        fresh: bool,
    },
    /// Phase 4: Timestamp-locked chronology gap recovery.
    /// State machine: PENDING → FETCHED → VERIFIED_TS_MATCH → RECOVERED
    /// Hard invariant: T_provider MUST equal T_barrier exactly.
    Repair {
        #[arg(long, default_value = "state_archive")]
        archive_root: PathBuf,
        #[arg(long)]
        batch_id: u32,
        #[command(subcommand)]
        action: RepairAction,
    },
}

#[derive(Subcommand)]
enum RepairAction {
    /// Queue a single repair request for a (symbol, target_ts) pair.
    Queue {
        #[arg(long)]
        symbol: String,
        #[arg(long)]
        target_ts: i64,
        #[arg(long, default_value = "manual")]
        reason: String,
        #[arg(long, default_value_t = BAR_SEC)]
        bar_sec: i64,
        #[arg(long, default_value = "yfinance")]
        provider: String,
    },
    /// Auto-detect gaps from live_session_steps.jsonl and queue them.
    Detect {
        #[arg(long, default_value = "")]
        run_label: String,
    },
    /// Process all PENDING repair requests (timestamp-locked fetch + provenance write).
    Process,
    /// Show repair queue status summary.
    Status,
}

fn read_cohort(path: &PathBuf) -> Result<Vec<String>> {
    let text = fs::read_to_string(path)?;
    Ok(text
        .lines()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect())
}

fn cmd_timeline(batch_id: u32, cohort: PathBuf, candle_root: PathBuf) -> Result<()> {
    let symbols = read_cohort(&cohort)?;
    let (data, manifest) = load_frozen_cohort(batch_id, &symbols, &candle_root)?;
    let aligned = align_timeline(&data);

    println!("{}", "=".repeat(60));
    println!("CS-INGEST — TIMELINE ALIGNMENT");
    println!("{}", "=".repeat(60));
    println!("  batch_id            : {batch_id:03}");
    println!("  symbols loaded      : {}", aligned.symbol_count);
    println!("  total bars          : {}", aligned.total_bars);
    println!("  timeline intervals  : {}", aligned.timestamps.len());
    println!("  fingerprint (rust)  : {}", aligned.fingerprint);
    if let Some(fp) = &manifest.timeline_fingerprint {
        println!("  fingerprint (manifest): {fp}");
        println!("  fingerprint match   : {}", fp == &aligned.fingerprint);
    }
    if let (Some(a), Some(b)) = (aligned.timestamps.first(), aligned.timestamps.last()) {
        println!("  ts range            : {a} → {b}");
    }
    println!("{}", "=".repeat(60));
    Ok(())
}

fn cmd_dedupe_verify(archive: PathBuf) -> Result<()> {
    let index_path = archive.join("metadata/dedupe_index.json");
    let mut idx = DedupeIndex::new(index_path.clone());
    let loaded = idx.load().context("load dedupe index")?;
    let mut rebuilt = DedupeIndex::new(index_path);
    let count = rebuilt
        .rebuild_from_archive(&archive, None)
        .context("rebuild from archive")?;

    println!("{}", "=".repeat(60));
    println!("CS-INGEST — DEDUPE VERIFY");
    println!("{}", "=".repeat(60));
    println!("  archive       : {}", archive.display());
    println!("  index keys    : {loaded}");
    println!("  archive keys  : {count}");
    println!(
        "  parity        : {}",
        if loaded == count { "OK" } else { "MISMATCH" }
    );
    println!("{}", "=".repeat(60));
    Ok(())
}

fn cmd_replay_step(
    batch_id: u32,
    cohort: PathBuf,
    archive: PathBuf,
    candle_root: PathBuf,
    pca_weights: PathBuf,
    observatory: PathBuf,
    start_interval: usize,
    max_intervals: Option<usize>,
    resume: bool,
    rebuild_dedupe: bool,
    fresh: bool,
) -> Result<()> {
    let symbols = read_cohort(&cohort)?;
    fs::create_dir_all(&archive)?;
    let result = run_replay_step(&ReplayStepConfig {
        batch_id,
        cohort_symbols: symbols,
        candle_root,
        archive_dir: archive.clone(),
        observatory_path: observatory,
        pca_weights_path: pca_weights,
        start_interval,
        max_intervals,
        resume,
        rebuild_dedupe,
        fresh,
    })?;

    println!("{}", "=".repeat(60));
    println!("CS-INGEST — REPLAY-STEP COMPLETE");
    println!("{}", "=".repeat(60));
    println!("  archive             : {}", archive.display());
    println!("  intervals run       : {}", result.intervals_run);
    println!("  persisted ticks     : {}", result.processed_ticks);
    println!("  corridors           : {}", result.corridors);
    println!("  dedupe skipped      : {}", result.dedupe_skipped);
    println!("  timeline fingerprint: {}", result.timeline_fingerprint);
    println!("  duration_sec        : {:.2}", result.duration_sec);
    println!("{}", "=".repeat(60));
    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.cmd {
        Command::Timeline {
            batch_id,
            cohort,
            candle_root,
        } => cmd_timeline(batch_id, cohort, candle_root),
        Command::DedupeVerify { archive } => cmd_dedupe_verify(archive),
        Command::ReplayStep {
            batch_id,
            cohort,
            archive,
            candle_root,
            pca_weights,
            observatory,
            start_interval,
            max_intervals,
            resume,
            rebuild_dedupe,
            fresh,
        } => cmd_replay_step(
            batch_id,
            cohort,
            archive,
            candle_root,
            pca_weights,
            observatory,
            start_interval,
            max_intervals,
            resume,
            rebuild_dedupe,
            fresh,
        ),
        Command::Repair {
            archive_root,
            batch_id,
            action,
        } => {
            let cfg = RepairConfig {
                archive_root,
                batch_id,
            };
            match action {
                RepairAction::Queue {
                    symbol,
                    target_ts,
                    reason,
                    bar_sec,
                    provider,
                } => cmd_queue(&cfg, &symbol, target_ts, &reason, bar_sec, &provider),
                RepairAction::Detect { run_label } => cmd_detect(&cfg, &run_label),
                RepairAction::Process => cmd_process(&cfg),
                RepairAction::Status => cmd_status(&cfg),
            }
        }
    }
}
