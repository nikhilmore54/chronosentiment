use anyhow::{Context, Result};
use flate2::read::GzDecoder;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

/// Python-compatible dedupe index at `metadata/dedupe_index.json`.
#[derive(Debug, Default)]
pub struct DedupeIndex {
    pub index_path: PathBuf,
    pub seen: HashSet<(String, i64)>,
    pub skipped: u64,
}

#[derive(Serialize, Deserialize)]
struct IndexFile {
    version: u32,
    count: usize,
    keys: Vec<(String, i64)>,
}

impl DedupeIndex {
    pub fn new(index_path: PathBuf) -> Self {
        Self {
            index_path,
            seen: HashSet::new(),
            skipped: 0,
        }
    }

    pub fn load(&mut self) -> Result<usize> {
        if !self.index_path.exists() {
            return Ok(0);
        }
        let raw = std::fs::read_to_string(&self.index_path)?;
        let data: Value = serde_json::from_str(&raw)?;
        self.seen.clear();
        if let Some(keys) = data.get("keys").and_then(|k| k.as_array()) {
            for entry in keys {
                if let Some(pair) = entry.as_array() {
                    if pair.len() == 2 {
                        let sym = pair[0].as_str().unwrap_or("").to_string();
                        let ts = pair[1].as_i64().unwrap_or(0);
                        self.seen.insert((sym, ts));
                    }
                }
            }
        }
        Ok(self.seen.len())
    }

    pub fn rebuild_from_archive(
        &mut self,
        archive_dir: &Path,
        cohort: Option<&HashSet<String>>,
    ) -> Result<usize> {
        let raw = archive_dir.join("raw");
        if !raw.is_dir() {
            return Ok(0);
        }
        for entry in std::fs::read_dir(&raw)? {
            let entry = entry?;
            if !entry.file_type()?.is_dir() {
                continue;
            }
            let symbol = entry.file_name().to_string_lossy().to_string();
            if let Some(c) = cohort {
                if !c.contains(&symbol) {
                    continue;
                }
            }
            for gz in std::fs::read_dir(entry.path())? {
                let gz = gz?;
                let fname = gz.file_name();
                let name = fname.to_string_lossy();
                if !name.ends_with(".jsonl.gz") || !name.starts_with("telemetry_stream_") {
                    continue;
                }
                scan_gzip_records(&gz.path(), &symbol, &mut self.seen)?;
            }
        }
        Ok(self.seen.len())
    }

    pub fn check_and_add(&mut self, symbol: &str, ts: i64) -> bool {
        let key = (symbol.to_string(), ts);
        if self.seen.contains(&key) {
            self.skipped += 1;
            return false;
        }
        self.seen.insert(key);
        true
    }

    pub fn save(&self) -> Result<()> {
        if let Some(parent) = self.index_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut keys: Vec<(String, i64)> = self.seen.iter().cloned().collect();
        keys.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));
        let payload = IndexFile {
            version: 1,
            count: keys.len(),
            keys,
        };
        let tmp = self.index_path.with_extension("json.tmp");
        std::fs::write(&tmp, serde_json::to_string_pretty(&payload)?)?;
        std::fs::rename(&tmp, &self.index_path)?;
        Ok(())
    }
}

fn scan_gzip_records(path: &Path, symbol: &str, seen: &mut HashSet<(String, i64)>) -> Result<()> {
    let f = File::open(path).with_context(|| format!("open {}", path.display()))?;
    let reader = BufReader::new(GzDecoder::new(f));
    for line in reader.lines() {
        let line = line?;
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let v: Value = serde_json::from_str(line)?;
        if let Some(ts) = v.get("ts").and_then(|t| t.as_i64()) {
            seen.insert((symbol.to_string(), ts));
        }
    }
    Ok(())
}
