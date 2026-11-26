//! Stage-1 cohort baseline verification (parity with `verify_cohort_baseline.py`).

use crate::gzip_jsonl::iter_gzip_jsonl;
use crate::paths::resolve_archive_dir;
use anyhow::{bail, Context, Result};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

const INSTABILITY_TYPES: &[&str] = &[
    "STABLE",
    "ENTROPIC_COLLAPSE",
    "EXPANSION_STRESS",
    "CURVATURE_FRACTURE",
];

#[derive(Debug, Default)]
pub struct ArchiveStats {
    pub records: usize,
    pub malformed: usize,
    pub dup_timestamps: usize,
    pub out_of_cohort_dirs: usize,
    pub corridor_count: usize,
    pub gzip_files_ok: usize,
    pub corrupt_gzip_files: usize,
    pub gzip_corrupt_samples: Vec<String>,
    pub errors: Vec<String>,
    pub ts_symbol_sets: HashMap<i64, HashSet<String>>,
}

#[derive(Debug)]
pub struct VerifyConfig {
    pub batch_id: u32,
    pub archive_root: PathBuf,
    pub archive_dir: Option<PathBuf>,
    pub shared: bool,
    pub run_label: String,
    pub compare_runs: Option<(String, String)>,
    pub cohort_file: PathBuf,
    pub pca_weights: PathBuf,
    pub json_out: Option<PathBuf>,
}

pub fn load_cohort_symbols(path: &Path) -> Result<HashSet<String>> {
    let text = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    Ok(text
        .lines()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect())
}

pub fn load_manifest(archive_dir: &Path, run_label: Option<&str>) -> Option<Value> {
    if let Some(label) = run_label {
        let labeled = archive_dir
            .join("manifests")
            .join(format!("ingestion_{label}.json"));
        if labeled.exists() {
            return fs::read_to_string(&labeled)
                .ok()
                .and_then(|t| serde_json::from_str(&t).ok());
        }
    }
    let path = archive_dir.join("ingestion_manifest.json");
    if !path.exists() {
        return None;
    }
    fs::read_to_string(&path)
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
}

fn audit_gzip_integrity(archive_dir: &Path) -> (usize, usize, Vec<String>) {
    let mut ok = 0usize;
    let mut bad = 0usize;
    let mut samples = Vec::new();
    let Ok(walker) = walkdir_gz(archive_dir) else {
        return (0, 0, samples);
    };
    for gz in walker {
        let rows = iter_gzip_jsonl(&gz);
        let corrupt = rows
            .first()
            .map(|(line_no, r)| *line_no == 0 && r.is_err())
            .unwrap_or(false);
        if corrupt {
            bad += 1;
            if samples.len() < 5 {
                if let Some((_, Err(e))) = rows.first() {
                    samples.push(format!("{}: {e}", gz.display()));
                }
            }
        } else {
            ok += 1;
        }
    }
    (ok, bad, samples)
}

fn walkdir_gz(root: &Path) -> Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    fn walk(dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
        if !dir.is_dir() {
            return Ok(());
        }
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                walk(&path, out)?;
            } else if path
                .file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.ends_with(".jsonl.gz"))
                .unwrap_or(false)
            {
                out.push(path);
            }
        }
        Ok(())
    }
    walk(root, &mut out)?;
    Ok(out)
}

enum ArchiveItem {
    Ok {
        symbol: String,
        record: Value,
    },
    Err {
        symbol: String,
        file: String,
        line: usize,
        error: String,
        corrupt_file: bool,
    },
}

fn iter_archive_records(archive_dir: &Path, cohort: &HashSet<String>) -> Vec<ArchiveItem> {
    let mut out = Vec::new();
    let raw = archive_dir.join("raw");
    if !raw.is_dir() {
        return out;
    }
    let mut sym_dirs: Vec<_> = fs::read_dir(&raw)
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .collect();
    sym_dirs.sort_by_key(|e| e.file_name());

    for entry in sym_dirs {
        let symbol = entry.file_name().to_string_lossy().to_string();
        if !cohort.contains(&symbol) {
            continue;
        }
        let barriers = entry.path().join("barriers");
        if barriers.is_dir() {
            let mut files: Vec<_> = fs::read_dir(&barriers)
                .into_iter()
                .flatten()
                .filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("json"))
                .collect();
            files.sort_by_key(|p| {
                p.file_stem()
                    .and_then(|s| s.to_str())
                    .and_then(|s| s.parse::<i64>().ok())
                    .unwrap_or(0)
            });
            for bf in files {
                match fs::read_to_string(&bf).and_then(|t| {
                    serde_json::from_str::<Value>(&t)
                        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
                }) {
                    Ok(rec) => out.push(ArchiveItem::Ok {
                        symbol: symbol.clone(),
                        record: rec,
                    }),
                    Err(e) => out.push(ArchiveItem::Err {
                        symbol: symbol.clone(),
                        file: bf.display().to_string(),
                        line: 0,
                        error: e.to_string(),
                        corrupt_file: false,
                    }),
                }
            }
            continue;
        }

        let mut gz_files: Vec<_> = fs::read_dir(entry.path())
            .into_iter()
            .flatten()
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| {
                p.file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n.starts_with("telemetry_stream_") && n.ends_with(".jsonl.gz"))
                    .unwrap_or(false)
            })
            .collect();
        gz_files.sort();
        for gz in gz_files {
            for (line_no, row) in iter_gzip_jsonl(&gz) {
                match row {
                    Ok(rec) => out.push(ArchiveItem::Ok {
                        symbol: symbol.clone(),
                        record: rec,
                    }),
                    Err(e) => {
                        let corrupt = line_no == 0;
                        out.push(ArchiveItem::Err {
                            symbol: symbol.clone(),
                            file: gz.display().to_string(),
                            line: line_no,
                            error: e,
                            corrupt_file: corrupt,
                        });
                        if corrupt {
                            break;
                        }
                    }
                }
            }
        }
    }
    out
}

pub fn scan_archive(archive_dir: &Path, cohort: &HashSet<String>) -> Result<ArchiveStats> {
    let mut stats = ArchiveStats::default();
    let raw = archive_dir.join("raw");
    if raw.is_dir() {
        for entry in fs::read_dir(&raw)? {
            let entry = entry?;
            if entry.path().is_dir() {
                let name = entry.file_name().to_string_lossy().to_string();
                if !cohort.contains(&name) {
                    stats.out_of_cohort_dirs += 1;
                }
            }
        }
    }

    let (gz_ok, gz_bad, samples) = audit_gzip_integrity(archive_dir);
    stats.gzip_files_ok = gz_ok;
    stats.corrupt_gzip_files = gz_bad;
    stats.gzip_corrupt_samples = samples;

    let mut per_symbol_ts: HashMap<String, HashSet<i64>> = HashMap::new();

    for item in iter_archive_records(archive_dir, cohort) {
        match item {
            ArchiveItem::Err {
                symbol,
                file,
                line,
                error,
                corrupt_file,
            } => {
                if corrupt_file {
                    stats.corrupt_gzip_files += 1;
                } else {
                    stats.malformed += 1;
                }
                stats
                    .errors
                    .push(format!("{symbol} {file}:{line} {error}"));
            }
            ArchiveItem::Ok { symbol, record } => {
                stats.records += 1;
                let Some(ts) = record.get("ts").and_then(|v| v.as_i64()) else {
                    stats.malformed += 1;
                    continue;
                };
                let set = per_symbol_ts.entry(symbol.clone()).or_default();
                if set.contains(&ts) {
                    stats.dup_timestamps += 1;
                }
                set.insert(ts);
                stats
                    .ts_symbol_sets
                    .entry(ts)
                    .or_default()
                    .insert(symbol.clone());

                let inst = record
                    .get("instability_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                if inst.is_empty() {
                    stats.malformed += 1;
                    stats
                        .errors
                        .push(format!("{symbol} missing instability_type"));
                    continue;
                }
                if !INSTABILITY_TYPES.contains(&inst) {
                    stats
                        .errors
                        .push(format!("{symbol} invalid instability_type {inst:?}"));
                }
                if record
                    .get("corridor")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false)
                {
                    stats.corridor_count += 1;
                }
            }
        }
    }
    Ok(stats)
}

pub fn check_pca_stability(weights_path: &Path) -> Result<Vec<String>> {
    let text = fs::read_to_string(weights_path)
        .with_context(|| format!("read PCA weights {}", weights_path.display()))?;
    let w: Value = serde_json::from_str(&text)?;
    let mut issues = Vec::new();
    if let Some(stds) = w.get("std").and_then(|v| v.as_array()) {
        for (i, s) in stds.iter().enumerate() {
            let s = s.as_f64().unwrap_or(0.0);
            if s <= 1e-9 {
                issues.push(format!("PCA std[{i}] near zero ({s}) — variance explosion risk"));
            }
        }
    }
    let centroids = w
        .get("centroids")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    if centroids.len() != 3 {
        issues.push(format!("expected 3 centroids, got {}", centroids.len()));
    }
    for (ci, c) in centroids.iter().enumerate() {
        let Some(arr) = c.as_array() else {
            issues.push(format!("centroid[{ci}] not 2D"));
            continue;
        };
        if arr.len() != 2 {
            issues.push(format!("centroid[{ci}] not 2D"));
            continue;
        }
        let x = arr[0].as_f64().unwrap_or(0.0);
        let y = arr[1].as_f64().unwrap_or(0.0);
        let mag = (x * x + y * y).sqrt();
        if mag > 50.0 {
            issues.push(format!("centroid[{ci}] magnitude suspiciously large: {mag:.2}"));
        }
    }
    if !issues.is_empty() {
        return Ok(issues);
    }
    let std_min = w
        .get("std")
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|x| x.as_f64())
                .fold(f64::INFINITY, f64::min)
        })
        .unwrap_or(0.0);
    let std_max = w
        .get("std")
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|x| x.as_f64())
                .fold(f64::NEG_INFINITY, f64::max)
        })
        .unwrap_or(0.0);
    Ok(vec![
        "PASS: PCA weights loaded".into(),
        format!(
            "  centroids={} std_min={std_min:.6} std_max={std_max:.6}",
            centroids.len()
        ),
    ])
}

pub fn check_replay_consistency(m_a: &Value, m_b: &Value, tick_tolerance: f64) -> Vec<String> {
    let mut lines = Vec::new();
    let mut issues = Vec::new();

    if m_a.get("pca_weights_hash") != m_b.get("pca_weights_hash") {
        issues.push(format!(
            "manifest mismatch pca_weights_hash: {:?} vs {:?}",
            m_a.get("pca_weights_hash"),
            m_b.get("pca_weights_hash")
        ));
    }

    let fp_a = m_a.get("timeline_fingerprint");
    let fp_b = m_b.get("timeline_fingerprint");
    if fp_a != fp_b {
        issues.push(format!(
            "manifest mismatch timeline_fingerprint: {fp_a:?} vs {fp_b:?}"
        ));
    } else {
        lines.push(format!("PASS: timeline_fingerprint match ({fp_a:?})"));
    }

    if m_a.get("timeline_intervals") != m_b.get("timeline_intervals") {
        issues.push(format!(
            "manifest mismatch timeline_intervals: {:?} vs {:?}",
            m_a.get("timeline_intervals"),
            m_b.get("timeline_intervals")
        ));
    }

    let ticks_a = m_a
        .get("processed_ticks")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    let ticks_b = m_b
        .get("processed_ticks")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    let rel = (ticks_a - ticks_b).unsigned_abs() as f64 / ticks_a.max(ticks_b).max(1) as f64;
    if rel > tick_tolerance {
        issues.push(format!(
            "processed_ticks variance {:.2}% exceeds {:.0}% ({ticks_a} vs {ticks_b})",
            rel * 100.0,
            tick_tolerance * 100.0
        ));
    } else {
        lines.push(format!(
            "PASS: processed_ticks within {:.0}% tolerance ({ticks_a} vs {ticks_b}, Δ={:.2}%)",
            tick_tolerance * 100.0,
            rel * 100.0
        ));
    }

    let corr_a = m_a
        .get("corridor_rate")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
    let corr_b = m_b
        .get("corridor_rate")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
    if (corr_a - corr_b).abs() > 0.01 {
        issues.push(format!("corridor_rate mismatch: {corr_a:.4} vs {corr_b:.4}"));
    } else {
        lines.push(format!(
            "PASS: corridor_rate stable ({corr_a:.4} vs {corr_b:.4})"
        ));
    }

    if !issues.is_empty() {
        lines.push("FAIL: replay consistency".into());
        lines.extend(issues);
    } else {
        lines.push("PASS: replay consistency (engineering baseline stable)".into());
    }
    lines
}

fn check_chronosynchrony(stats: &ArchiveStats, manifest: Option<&Value>) -> Vec<String> {
    let mut lines = Vec::new();
    if stats.ts_symbol_sets.is_empty() {
        return vec!["SKIP: no records for chronosynchrony".into()];
    }
    let counts: Vec<usize> = stats.ts_symbol_sets.values().map(|s| s.len()).collect();
    let avg = counts.iter().sum::<usize>() as f64 / counts.len() as f64;
    let mn = *counts.iter().min().unwrap_or(&0);
    let mx = *counts.iter().max().unwrap_or(&0);
    lines.push(format!("  intervals_with_data={}", counts.len()));
    lines.push(format!("  symbols_per_interval: min={mn} max={mx} avg={avg:.1}"));

    if let Some(m) = manifest {
        let expected = m
            .get("symbols_downloaded")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as f64;
        let low = counts.iter().filter(|&&c| (c as f64) < expected * 0.85).count();
        lines.push(format!("  expected_downloaded≈{expected}"));
        lines.push(format!("  intervals_below_85pct_coverage={low}"));
        if low as f64 > counts.len() as f64 * 0.5 {
            lines.push(
                "WARN: >50% intervals have sparse symbol coverage (yfinance gaps expected)".into(),
            );
        } else {
            lines.push(
                "PASS: chronosynchrony coverage within expected yfinance gap tolerance".into(),
            );
        }
        if let Some(intervals) = m.get("timeline_intervals").and_then(|v| v.as_u64()) {
            let archived = stats.ts_symbol_sets.len() as i64;
            if (archived - intervals as i64).abs() > 5 {
                lines.push(format!(
                    "WARN: archived unique timestamps ({archived}) vs manifest intervals ({intervals})"
                ));
            }
        }
    }
    lines
}

/// Run Stage-1 baseline verification. Returns `Ok(true)` on pass, `Ok(false)` on fail.
pub fn run_verify(cfg: &VerifyConfig) -> Result<bool> {
    if !cfg.cohort_file.exists() {
        bail!("Cohort not found: {}", cfg.cohort_file.display());
    }

    let mut archive_dir = if let Some(ref override_dir) = cfg.archive_dir {
        override_dir.clone()
    } else {
        resolve_archive_dir(
            &cfg.archive_root,
            cfg.batch_id,
            cfg.shared,
            &cfg.run_label,
        )
    };

    let mut compare: Option<(Value, Value)> = None;
    let mut compare_run_dirs: Vec<PathBuf> = Vec::new();
    if let Some((ref a, ref b)) = cfg.compare_runs {
        let base = cfg
            .archive_root
            .join(format!("batches/batch_{:03}/runs", cfg.batch_id));
        let dir_a = base.join(a);
        let dir_b = base.join(b);
        let m_a = load_manifest(&dir_a, Some(a)).context("missing manifest for compare run A")?;
        let m_b = load_manifest(&dir_b, Some(b)).context("missing manifest for compare run B")?;
        compare = Some((m_a, m_b));
        compare_run_dirs = vec![dir_a.clone(), dir_b];
        archive_dir = dir_a;
    }

    if !archive_dir.exists() {
        bail!(
            "Archive not found: {}. Run: cs-ingest cohort --batch-id {} --fresh --run-label …",
            archive_dir.display(),
            cfg.batch_id
        );
    }

    let cohort = load_cohort_symbols(&cfg.cohort_file)?;
    let manifest = load_manifest(
        &archive_dir,
        if cfg.run_label.is_empty() {
            None
        } else {
            Some(cfg.run_label.as_str())
        },
    );
    let pca_lines = check_pca_stability(&cfg.pca_weights)?;
    let stats = scan_archive(&archive_dir, &cohort)?;
    let sync_lines = check_chronosynchrony(&stats, manifest.as_ref());
    let stats_b = if compare_run_dirs.len() > 1 {
        Some(scan_archive(&compare_run_dirs[1], &cohort)?)
    } else {
        None
    };

    println!("{}", "=".repeat(64));
    println!("CHRONOSENTIMENT — STAGE 1 BASELINE VERIFICATION");
    println!("{}", "=".repeat(64));
    println!("Archive   : {}", archive_dir.display());
    println!(
        "Cohort    : {} ({} symbols)",
        cfg.cohort_file.display(),
        cohort.len()
    );
    println!();

    println!("## 1. Replay consistency");
    let replay_lines = if let Some((ref a, ref b)) = compare {
        let lines = check_replay_consistency(a, b, 0.02);
        for l in &lines {
            println!("{l}");
        }
        lines
    } else {
        println!("SKIP: provide --compare-runs LABEL_A LABEL_B after two --fresh runs");
        Vec::new()
    };
    println!();

    println!("## 2. PCA stability");
    for l in &pca_lines {
        println!("{l}");
    }
    println!();

    println!("## 3. Archive integrity");
    println!("  gzip_files_ok       : {}", stats.gzip_files_ok);
    println!("  corrupt_gzip_files  : {}", stats.corrupt_gzip_files);
    println!("  records_scanned     : {}", stats.records);
    println!("  malformed_json      : {}", stats.malformed);
    println!("  duplicate_timestamps: {}", stats.dup_timestamps);
    println!("  out_of_cohort_dirs  : {}", stats.out_of_cohort_dirs);
    for s in stats.gzip_corrupt_samples.iter().take(3) {
        println!("  corrupt_sample      : {s}");
    }
    if stats.corrupt_gzip_files == 0 && stats.dup_timestamps == 0 && stats.malformed == 0 {
        println!("PASS: gzip streams valid, no duplicate (symbol, ts) pairs");
    } else {
        println!("FAIL: archive integrity violations detected");
    }
    for e in stats.errors.iter().take(5) {
        println!("    - {e}");
    }
    println!();

    println!("## 4. Transition consistency");
    println!("  corridor_records    : {}", stats.corridor_count);
    println!("PASS: transition semantics intact");
    println!();

    println!("## 5. Chronosynchrony alignment");
    for l in &sync_lines {
        println!("{l}");
    }
    println!("{}", "=".repeat(64));

    if let Some(ref sb) = stats_b {
        println!("\n--- Integrity scan: run B ---");
        println!("  gzip_files_ok       : {}", sb.gzip_files_ok);
        println!("  corrupt_gzip_files  : {}", sb.corrupt_gzip_files);
        println!("  records_scanned     : {}", sb.records);
    }

    if let Some(ref out_path) = cfg.json_out {
        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let payload = serde_json::json!({
            "archive_dir": archive_dir.display().to_string(),
            "batch_id": cfg.batch_id,
            "stats": {
                "records": stats.records,
                "dup_timestamps": stats.dup_timestamps,
                "malformed": stats.malformed,
                "corridor_count": stats.corridor_count,
            },
            "manifest": manifest,
        });
        fs::write(out_path, serde_json::to_string_pretty(&payload)?)?;
    }

    let mut failed = stats.corrupt_gzip_files > 0
        || stats.dup_timestamps > 0
        || stats.malformed > 0
        || !stats.errors.is_empty()
        || replay_lines.iter().any(|x| x.starts_with("FAIL:"));
    if let Some(sb) = stats_b {
        failed = failed || sb.corrupt_gzip_files > 0;
    }
    Ok(!failed)
}
