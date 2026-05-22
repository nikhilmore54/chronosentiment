use crate::archive::{telemetry_gz_path, GzipWriterPool};
use crate::dedupe::DedupeIndex;
use anyhow::{Context, Result};
use serde_json::Value;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::fs;

const STABLE_SAMPLE_EVERY: u64 = 8;

pub struct ArchivePersistor {
    pub archive_dir: std::path::PathBuf,
    pub dedupe: DedupeIndex,
    gzip: GzipWriterPool,
    stable_counters: HashMap<String, u64>,
    pub persisted: u64,
    pub corridors: u64,
}

impl ArchivePersistor {
    pub fn new(archive_dir: std::path::PathBuf) -> Self {
        let dedupe = DedupeIndex::new(archive_dir.join("metadata/dedupe_index.json"));
        Self {
            archive_dir,
            dedupe,
            gzip: GzipWriterPool::new(),
            stable_counters: HashMap::new(),
            persisted: 0,
            corridors: 0,
        }
    }

    pub fn init_dedupe(&mut self, resume: bool, rebuild: bool) -> Result<()> {
        if rebuild || (resume && !self.dedupe.index_path.exists()) {
            self.dedupe.rebuild_from_archive(&self.archive_dir, None)?;
            self.dedupe.save()?;
        } else if resume || self.dedupe.index_path.exists() {
            self.dedupe.load()?;
        }
        Ok(())
    }

    pub fn persist_record(&mut self, record: &Value) -> Result<()> {
        let symbol = record["symbol"].as_str().unwrap_or("").to_string();
        let ts = record["ts"].as_i64().unwrap_or(0);
        if !self.dedupe.check_and_add(&symbol, ts) {
            return Ok(());
        }

        // Add schema_version to a cloned record value
        let mut mutated_record = record.clone();
        if let Some(obj) = mutated_record.as_object_mut() {
            obj.insert("schema_version".to_string(), serde_json::json!(1));
        }
        let record = &mutated_record;

        let sym_dir = self.archive_dir.join("raw").join(&symbol);
        fs::create_dir_all(&sym_dir)?;

        // ── Layer 1 — Canonical Barrier Archive (unconditional, barrier-native) ─
        let barriers_dir = sym_dir.join("barriers");
        fs::create_dir_all(&barriers_dir)?;
        let barrier_path = barriers_dir.join(format!("{ts}.json"));
        if !barrier_path.exists() {
            let obj = record.as_object().context("record must be object")?;
            let sorted: BTreeMap<&str, &Value> = obj.iter().map(|(k, v)| (k.as_str(), v)).collect();
            let json_str = serde_json::to_string(&sorted)?;
            fs::write(&barrier_path, json_str)?;
        }

        let instability = record["instability_type"].as_str().unwrap_or("");
        let corridor = record["corridor"].as_bool().unwrap_or(false);
        let prec_ent = record["precursor_entropy_expansion"].as_f64().unwrap_or(0.0);
        let prec_curv = record["precursor_curvature_destabilization"].as_f64().unwrap_or(0.0);
        let is_event = instability != "STABLE"
            || corridor
            || prec_ent > 0.05
            || prec_curv > 15.0;

        if !is_event {
            let cnt = self.stable_counters.entry(symbol.clone()).or_insert(0);
            *cnt += 1;
            if *cnt % STABLE_SAMPLE_EVERY != 0 {
                fs::write(
                    sym_dir.join("latest.json"),
                    serde_json::to_string(record)?,
                )?;
                return Ok(());
            }
        } else {
            self.stable_counters.insert(symbol.clone(), 0);
        }

        let line = sorted_json_line(record)?;
        let gz_path = telemetry_gz_path(&self.archive_dir, &symbol, ts);
        self.gzip.writeln(&gz_path, &line)?;

        let sym_dir = self.archive_dir.join("raw").join(&symbol);
        fs::create_dir_all(&sym_dir)?;
        fs::write(sym_dir.join("latest.json"), &line)?;

        if corridor {
            self.corridors += 1;
            let corr_dir = self
                .archive_dir
                .join("transitions")
                .join("corridor_events");
            self.gzip
                .writeln(&corr_dir.join(format!("{symbol}_events.jsonl.gz")), &line)?;
            let entropy = record["entropy"].as_f64().unwrap_or(0.0);
            if entropy > 0.95 {
                let coll_dir = self
                    .archive_dir
                    .join("transitions")
                    .join("collapse_events");
                self.gzip
                    .writeln(&coll_dir.join(format!("{symbol}_collapses.jsonl.gz")), &line)?;
            }
        }

        self.persisted += 1;
        Ok(())
    }

    pub fn flush(&mut self) -> Result<()> {
        self.gzip.flush_all()?;
        Ok(())
    }

    pub fn finalize(&mut self) -> Result<()> {
        self.gzip.flush_all()?;
        self.gzip.finish_all()?;
        self.dedupe.save()?;
        Ok(())
    }
}

/// Python `json.dumps(record, sort_keys=True)` compatibility.
pub fn sorted_json_line(record: &Value) -> Result<String> {
    let obj = record.as_object().context("record must be object")?;
    let sorted: BTreeMap<&str, &Value> = obj.iter().map(|(k, v)| (k.as_str(), v)).collect();
    Ok(serde_json::to_string(&sorted)? + "\n")
}
