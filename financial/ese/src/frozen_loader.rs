use anyhow::{Context, Result};
use flate2::read::GzDecoder;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

pub const CANDLE_ROOT: &str = "state_archive/candles";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrozenBar {
    pub ts: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrozenManifest {
    pub batch_id: u32,
    pub cohort_file: Option<String>,
    pub interval: Option<String>,
    pub period: Option<String>,
    pub symbols_cohort: Option<u32>,
    pub symbols_frozen: Option<u32>,
    pub total_bars: Option<u64>,
    pub timeline_intervals: Option<u32>,
    pub timeline_fingerprint: Option<String>,
    pub timeline_first_ts: Option<i64>,
    pub timeline_last_ts: Option<i64>,
    pub substrate_hash: Option<String>,
    pub frozen_at_utc: Option<String>,
}

pub fn frozen_batch_dir(batch_id: u32, root: &Path) -> PathBuf {
    root.join(format!("batch_{batch_id:03}"))
}

fn symbol_file_path(batch_dir: &Path, symbol: &str) -> PathBuf {
    let safe = symbol.replace('/', "_");
    batch_dir.join("symbols").join(format!("{safe}.jsonl.gz"))
}

/// Load manifest + per-symbol bars from frozen gzip jsonl (Python-compatible layout).
pub fn load_frozen_cohort(
    batch_id: u32,
    cohort_symbols: &[String],
    root: &Path,
) -> Result<(HashMap<String, Vec<FrozenBar>>, FrozenManifest)> {
    let batch_dir = frozen_batch_dir(batch_id, root);
    let manifest_path = batch_dir.join("manifest.json");
    let manifest: FrozenManifest = serde_json::from_slice(
        &std::fs::read(&manifest_path)
            .with_context(|| format!("read {}", manifest_path.display()))?,
    )
    .context("parse frozen manifest")?;

    let mut data = HashMap::new();
    for sym in cohort_symbols {
        let path = symbol_file_path(&batch_dir, sym);
        if !path.exists() {
            continue;
        }
        let bars = read_symbol_bars(&path)?;
        if !bars.is_empty() {
            data.insert(sym.clone(), bars);
        }
    }
    Ok((data, manifest))
}

pub fn read_symbol_bars(path: &Path) -> Result<Vec<FrozenBar>> {
    let f = File::open(path).with_context(|| format!("open {}", path.display()))?;
    let decoder = GzDecoder::new(f);
    let reader = BufReader::new(decoder);
    let mut out = Vec::new();
    for line in reader.lines() {
        let line = line?;
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let bar: FrozenBar = serde_json::from_str(line).context("bar json")?;
        out.push(bar);
    }
    out.sort_by_key(|b| b.ts);
    Ok(out)
}
