use anyhow::Result;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};

/// One gzip handle per file for the ingest run (matches Python GzipWriterPool).
pub struct GzipWriterPool {
    writers: HashMap<String, GzEncoder<std::fs::File>>,
}

impl GzipWriterPool {
    pub fn new() -> Self {
        Self {
            writers: HashMap::new(),
        }
    }

    fn path_key(path: &Path) -> Result<String> {
        // Do not use canonicalize() — it requires the file to exist and caused
        // duplicate gzip headers on the same path (relative key vs absolute key).
        let abs = if path.is_absolute() {
            path.to_path_buf()
        } else {
            std::env::current_dir()?.join(path)
        };
        Ok(abs.to_string_lossy().to_string())
    }

    pub fn writeln(&mut self, path: &Path, line: &str) -> Result<()> {
        let key = Self::path_key(path)?;
        if !self.writers.contains_key(&key) {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let file = if path.exists() && path.metadata()?.len() > 0 {
                OpenOptions::new().create(true).append(true).open(path)?
            } else {
                OpenOptions::new()
                    .create(true)
                    .write(true)
                    .truncate(true)
                    .open(path)?
            };
            let enc = GzEncoder::new(file, Compression::default());
            self.writers.insert(key.clone(), enc);
        }
        let enc = self.writers.get_mut(&key).unwrap();
        enc.write_all(line.as_bytes())?;
        if !line.ends_with('\n') {
            enc.write_all(b"\n")?;
        }
        Ok(())
    }

    pub fn flush_all(&mut self) -> Result<()> {
        for enc in self.writers.values_mut() {
            enc.flush()?;
        }
        Ok(())
    }

    /// Finalize gzip footers so archives are readable by dedupe-verify and Python.
    pub fn finish_all(&mut self) -> Result<()> {
        for (_, enc) in self.writers.drain() {
            let _file = enc.finish()?;
        }
        Ok(())
    }
}

pub fn telemetry_gz_path(archive_dir: &Path, symbol: &str, ts: i64) -> PathBuf {
    let day = chrono::DateTime::from_timestamp(ts, 0)
        .map(|dt| dt.format("%Y_%m_%d").to_string())
        .unwrap_or_else(|| "unknown".into());
    archive_dir
        .join("raw")
        .join(symbol)
        .join(format!("telemetry_stream_{day}.jsonl.gz"))
}
