use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use cs_ingest::dedupe::DedupeIndex;
use cs_ingest::fixtures::verify_chronology_byte_fixtures;
use cs_ingest::frozen_loader::{load_frozen_cohort, CANDLE_ROOT};
use cs_ingest::governor::{self, DEFAULT_BRIDGE, DEFAULT_INTERVAL_SEC, DEFAULT_WINDOW};
use cs_ingest::paths::{cohort_file, resolve_archive_dir};
use cs_ingest::repair::{cmd_detect, cmd_process, cmd_queue, cmd_status, RepairConfig, BAR_SEC};
use cs_ingest::replay::{run_replay_step, ReplayStepConfig};
use cs_ingest::timeline::align_timeline;
use cs_ingest::verify::{run_verify, VerifyConfig};
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Instant;

#[derive(Parser)]
#[command(name = "cs-ingest", about = "Deterministic replay-bound ingest utilities")]
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
        #[arg(long, default_value = "observatory/provider_clustering_pca_weights.json")]
        pca_weights: PathBuf,
        #[arg(
            long,
            default_value = "./target/release/examples/live_observatory"
        )]
        observatory: PathBuf,
        #[arg(long, default_value = "0")]
        start_interval: usize,
        #[arg(long)]
        max_intervals: Option<usize>,
        #[arg(long)]
        resume: bool,
        #[arg(long)]
        rebuild_dedupe: bool,
        #[arg(long, help = "Wipe archive telemetry layers before ingest (parity / replay verify)")]
        fresh: bool,
    },
    /// Cohort ingest convenience (replaces scripts/run_nse_cohort.py).
    /// Resolves batch archive paths then runs replay-step.
    Cohort {
        #[arg(long, default_value_t = 1)]
        batch_id: u32,
        #[arg(long, default_value = "state_archive")]
        archive_root: PathBuf,
        #[arg(long, default_value = "")]
        run_label: String,
        #[arg(long)]
        shared_archive: bool,
        #[arg(long, default_value = "0")]
        start_interval: usize,
        #[arg(long)]
        max_intervals: Option<usize>,
        #[arg(long)]
        fresh: bool,
        #[arg(long)]
        resume: bool,
        #[arg(long)]
        rebuild_dedupe: bool,
        #[arg(long, default_value = CANDLE_ROOT)]
        candle_root: PathBuf,
        #[arg(long, default_value = "observatory/provider_clustering_pca_weights.json")]
        pca_weights: PathBuf,
        #[arg(
            long,
            default_value = "./target/release/examples/live_observatory"
        )]
        observatory: PathBuf,
    },
    /// Phase 4: Timestamp-locked chronology gap recovery.
    Repair {
        #[arg(long, default_value = "state_archive")]
        archive_root: PathBuf,
        #[arg(long)]
        batch_id: u32,
        #[command(subcommand)]
        action: RepairAction,
    },
    /// Minimal production governor (replaces scripts/governor_refresher.py).
    Governor {
        #[arg(long, default_value = "state_archive")]
        archive_dir: PathBuf,
        #[arg(long, default_value_t = DEFAULT_WINDOW)]
        window: usize,
        #[arg(long, default_value_t = DEFAULT_INTERVAL_SEC)]
        interval: f64,
        #[arg(long, default_value = DEFAULT_BRIDGE)]
        bridge: PathBuf,
        #[arg(long, help = "Evaluate once and exit")]
        once: bool,
    },
    /// Stage-1 cohort baseline verification (replaces scripts/verify_cohort_baseline.py).
    Verify {
        #[arg(long, default_value_t = 1)]
        batch_id: u32,
        #[arg(long, default_value = "state_archive")]
        archive_root: PathBuf,
        #[arg(long, default_value = "")]
        archive_dir: String,
        #[arg(long)]
        shared_archive: bool,
        #[arg(long, default_value = "")]
        run_label: String,
        #[arg(long, num_args = 2, value_names = ["RUN_A", "RUN_B"])]
        compare_runs: Option<Vec<String>>,
        #[arg(long, default_value = "")]
        json_out: String,
        #[arg(long, default_value = "observatory/provider_clustering_pca_weights.json")]
        pca_weights: PathBuf,
    },
    /// V-006 chronology byte fixture verification (replaces verify_chronology_byte_fixtures.py).
    VerifyChronologyFixtures {
        #[arg(long, default_value = ".")]
        repo_root: PathBuf,
    },
}

#[derive(Subcommand)]
enum RepairAction {
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
    Detect {
        #[arg(long, default_value = "")]
        run_label: String,
    },
    Process,
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
        println!(
            "  fingerprint match   : {}",
            fp == &aligned.fingerprint
        );
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

fn cmd_cohort(
    batch_id: u32,
    archive_root: PathBuf,
    run_label: String,
    shared_archive: bool,
    start_interval: usize,
    max_intervals: Option<usize>,
    fresh: bool,
    resume: bool,
    rebuild_dedupe: bool,
    candle_root: PathBuf,
    pca_weights: PathBuf,
    observatory: PathBuf,
) -> Result<()> {
    if fresh && resume {
        bail!("Cannot use --fresh and --resume together");
    }
    let cohort = cohort_file(batch_id);
    if !cohort.exists() {
        bail!("Cohort not found: {}", cohort.display());
    }
    let archive = resolve_archive_dir(&archive_root, batch_id, shared_archive, &run_label);

    println!("{}", "=".repeat(60));
    println!("CHRONOSENTIMENT — FROZEN REPLAY (cs-ingest cohort)");
    println!("{}", "=".repeat(60));
    println!("  Backend            : cs-ingest replay-step");
    println!("  Archive            : {}", archive.display());
    if fresh {
        println!("  Certification      : --fresh (isolated archive)");
    }
    println!("{}", "=".repeat(60));

    let t0 = Instant::now();
    cmd_replay_step(
        batch_id,
        cohort,
        archive.clone(),
        candle_root,
        pca_weights,
        observatory,
        start_interval,
        max_intervals,
        resume,
        rebuild_dedupe,
        fresh,
    )?;
    let duration = t0.elapsed().as_secs_f64();

    println!();
    println!("{}", "=".repeat(60));
    println!("BATCH INGESTION COMPLETE");
    println!("{}", "=".repeat(60));
    println!("  Execution Time      : {duration:.2} seconds");
    println!("  Substrate Location  : {}/raw/", archive.display());
    println!("{}", "=".repeat(60));
    Ok(())
}

fn main() -> Result<ExitCode> {
    let cli = Cli::parse();
    match cli.cmd {
        Command::Timeline {
            batch_id,
            cohort,
            candle_root,
        } => {
            cmd_timeline(batch_id, cohort, candle_root)?;
            Ok(ExitCode::SUCCESS)
        }
        Command::DedupeVerify { archive } => {
            cmd_dedupe_verify(archive)?;
            Ok(ExitCode::SUCCESS)
        }
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
        } => {
            cmd_replay_step(
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
            )?;
            Ok(ExitCode::SUCCESS)
        }
        Command::Cohort {
            batch_id,
            archive_root,
            run_label,
            shared_archive,
            start_interval,
            max_intervals,
            fresh,
            resume,
            rebuild_dedupe,
            candle_root,
            pca_weights,
            observatory,
        } => {
            cmd_cohort(
                batch_id,
                archive_root,
                run_label,
                shared_archive,
                start_interval,
                max_intervals,
                fresh,
                resume,
                rebuild_dedupe,
                candle_root,
                pca_weights,
                observatory,
            )?;
            Ok(ExitCode::SUCCESS)
        }
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
                } => cmd_queue(&cfg, &symbol, target_ts, &reason, bar_sec, &provider)?,
                RepairAction::Detect { run_label } => cmd_detect(&cfg, &run_label)?,
                RepairAction::Process => cmd_process(&cfg)?,
                RepairAction::Status => cmd_status(&cfg)?,
            }
            Ok(ExitCode::SUCCESS)
        }
        Command::Governor {
            archive_dir,
            window,
            interval,
            bridge,
            once,
        } => {
            if once {
                let state = governor::run_once(&archive_dir, window, &bridge)?;
                println!("{}", serde_json::to_string_pretty(&state)?);
            } else {
                governor::run_loop(&archive_dir, window, interval, &bridge)?;
            }
            Ok(ExitCode::SUCCESS)
        }
        Command::Verify {
            batch_id,
            archive_root,
            archive_dir,
            shared_archive,
            run_label,
            compare_runs,
            json_out,
            pca_weights,
        } => {
            let compare = compare_runs.map(|v| (v[0].clone(), v[1].clone()));
            let cfg = VerifyConfig {
                batch_id,
                archive_root,
                archive_dir: if archive_dir.is_empty() {
                    None
                } else {
                    Some(PathBuf::from(archive_dir))
                },
                shared: shared_archive,
                run_label,
                compare_runs: compare,
                cohort_file: cohort_file(batch_id),
                pca_weights,
                json_out: if json_out.is_empty() {
                    None
                } else {
                    Some(PathBuf::from(json_out))
                },
            };
            let passed = run_verify(&cfg)?;
            Ok(if passed {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(1)
            })
        }
        Command::VerifyChronologyFixtures { repo_root } => {
            let root = if repo_root == PathBuf::from(".") {
                env::current_dir()?
            } else {
                repo_root
            };
            let code = verify_chronology_byte_fixtures(&root)?;
            Ok(ExitCode::from(code as u8))
        }
    }
}
