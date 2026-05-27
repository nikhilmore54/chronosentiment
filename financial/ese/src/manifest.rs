use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct IngestManifest {
    pub batch_id: u32,
    pub run_label: String,
    pub symbols_downloaded: usize,
    pub processed_ticks: u64,
    pub corridors_classified: u64,
    pub timeline_fingerprint: String,
    pub timeline_intervals: usize,
    pub corridor_rate: f64,
    pub duration_sec: f64,
    pub dedupe_keys: usize,
    pub frozen_substrate_hash: Option<String>,
    pub frozen_timeline_fingerprint: Option<String>,
}

pub fn write_ingest_manifest(path: &Path, manifest: &IngestManifest) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(manifest)?;
    std::fs::write(path, json)
}
