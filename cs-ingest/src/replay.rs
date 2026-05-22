use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use crate::frozen_loader::{load_frozen_cohort, FrozenBar};
use crate::observatory::ObservatoryProcess;
use crate::pca::PcaWeights;
use crate::persist::ArchivePersistor;
use crate::telemetry::TelemetryProcessor;
use crate::timeline::{align_timeline, timeline_fingerprint};

pub struct ReplayStepConfig {
    pub batch_id: u32,
    pub cohort_symbols: Vec<String>,
    pub candle_root: PathBuf,
    pub archive_dir: PathBuf,
    pub observatory_path: PathBuf,
    pub pca_weights_path: PathBuf,
    pub start_interval: usize,
    pub max_intervals: Option<usize>,
    pub resume: bool,
    pub rebuild_dedupe: bool,
    pub fresh: bool,
}

/// Wipe hot/warm telemetry layers (matches Python `fresh_wipe_archive`).
pub fn fresh_wipe_archive(archive_dir: &Path) -> Result<()> {
    for sub in ["raw", "transitions", "trajectories", "topology", "metadata"] {
        let path = archive_dir.join(sub);
        if path.is_dir() {
            fs::remove_dir_all(&path)?;
        }
    }
    Ok(())
}

pub struct ReplayStepResult {
    pub intervals_run: usize,
    pub processed_ticks: u64,
    pub corridors: u64,
    pub dedupe_skipped: u64,
    pub timeline_fingerprint: String,
    pub duration_sec: f64,
}

fn bar_at_ts(bars: &[FrozenBar], ts: i64) -> Option<&FrozenBar> {
    bars.binary_search_by_key(&ts, |b| b.ts).ok().map(|i| &bars[i])
}

pub fn run_replay_step(cfg: &ReplayStepConfig) -> Result<ReplayStepResult> {
    if cfg.fresh {
        if cfg.archive_dir.exists() {
            fresh_wipe_archive(&cfg.archive_dir)?;
        }
        fs::create_dir_all(&cfg.archive_dir)?;
    }
    let (data, _manifest) =
        load_frozen_cohort(cfg.batch_id, &cfg.cohort_symbols, &cfg.candle_root)?;
    let aligned = align_timeline(&data);
    let fp = timeline_fingerprint(&aligned.timestamps);

    let mut persistor = ArchivePersistor::new(cfg.archive_dir.clone());
    persistor.init_dedupe(cfg.resume, cfg.rebuild_dedupe)?;

    let pca = PcaWeights::load(&cfg.pca_weights_path)?;
    let mut processor = TelemetryProcessor::new(pca);
    let mut observatory = ObservatoryProcess::spawn(&cfg.observatory_path)?;

    let t0 = Instant::now();
    let end = cfg
        .max_intervals
        .map(|m| (cfg.start_interval + m).min(aligned.timestamps.len()))
        .unwrap_or(aligned.timestamps.len());

    let mut intervals_run = 0usize;
    for (idx, ts) in aligned.timestamps[0..end].iter().enumerate() {
        let interval_idx = idx + 1;
        let mut batch: Vec<(String, &FrozenBar)> = Vec::new();
        for (sym, bars) in &data {
            if let Some(b) = bar_at_ts(bars, *ts) {
                batch.push((sym.clone(), b));
            }
        }
        if batch.is_empty() {
            continue;
        }
        
        let lines = observatory.run_barrier(*ts, &batch)?;
        
        // Only parse and persist if we've reached the novel frontier.
        // The preceding bars were silently processed to perfectly rebuild mathematical state.
        if idx >= cfg.start_interval {
            for line in lines {
                if let Some(rec) = processor.process_line(&line) {
                    persistor.persist_record(&rec)?;
                }
            }
            intervals_run += 1;
            
            if interval_idx % 50 == 0 || interval_idx == end {
                persistor.flush()?;
                if interval_idx % 50 == 0 {
                    persistor.dedupe.save()?;
                }
                eprintln!(
                    "   interval {interval_idx}/{end} | persisted {} | corridors {} | dedupe_skip {}",
                    persistor.persisted, persistor.corridors, persistor.dedupe.skipped
                );
            }
        }
    }

    persistor.finalize()?;
    let duration_sec = t0.elapsed().as_secs_f64();

    Ok(ReplayStepResult {
        intervals_run,
        processed_ticks: persistor.persisted,
        corridors: persistor.corridors,
        dedupe_skipped: persistor.dedupe.skipped,
        timeline_fingerprint: fp,
        duration_sec,
    })
}
