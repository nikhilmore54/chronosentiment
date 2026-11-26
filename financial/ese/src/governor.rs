//! Minimal production governor — deterministic threshold logic.
//!
//! Parity with former `scripts/governor_refresher.py`:
//!   if instability_rate > HALT_THRESHOLD   → multiplier=0.0, gate=closed
//!   elif corridor_rate > THROTTLE_THRESHOLD → multiplier=0.65, gate=open
//!   else                                    → multiplier=1.0,  gate=open

use anyhow::{Context, Result};
use flate2::read::GzDecoder;
use serde::Serialize;
use serde_json::Value;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub const HALT_THRESHOLD: f64 = 0.40;
pub const THROTTLE_THRESHOLD: f64 = 0.20;
pub const DEFAULT_WINDOW: usize = 50;
pub const DEFAULT_INTERVAL_SEC: f64 = 0.5;
pub const DEFAULT_BRIDGE: &str = "analysis/real_live/governor_state.json";

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct GovernorState {
    pub multiplier: f64,
    pub gate_open: bool,
    pub reason: String,
    pub instability_rate: f64,
    pub corridor_rate: f64,
    pub window_size: usize,
    pub ts: i64,
}

fn now_ts() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn record_ts(rec: &Value) -> i64 {
    rec.get("ts").and_then(|v| v.as_i64()).unwrap_or(0)
}

/// Collect the most recent `window` telemetry records from the raw archive.
pub fn iter_recent_records(archive_dir: &Path, window: usize) -> Result<Vec<Value>> {
    let mut records: Vec<Value> = Vec::new();
    let raw_dir = archive_dir.join("raw");
    if !raw_dir.is_dir() {
        return Ok(records);
    }

    for entry in fs::read_dir(&raw_dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let latest = entry.path().join("latest.json");
        if latest.exists() {
            if let Ok(text) = fs::read_to_string(&latest) {
                if let Ok(rec) = serde_json::from_str::<Value>(&text) {
                    records.push(rec);
                }
            }
        }
    }

    if records.len() >= window {
        records.sort_by_key(|r| std::cmp::Reverse(record_ts(r)));
        records.truncate(window);
        return Ok(records);
    }

    for entry in fs::read_dir(&raw_dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let mut gz_files: Vec<PathBuf> = fs::read_dir(entry.path())?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| {
                p.extension()
                    .and_then(|e| e.to_str())
                    .map(|e| e == "gz")
                    .unwrap_or(false)
                    && p.file_name()
                        .and_then(|n| n.to_str())
                        .map(|n| n.contains(".jsonl."))
                        .unwrap_or(false)
            })
            .collect();
        gz_files.sort();
        gz_files.reverse();
        for gz_path in gz_files.into_iter().take(2) {
            if let Ok(file) = File::open(&gz_path) {
                let decoder = GzDecoder::new(file);
                let reader = BufReader::new(decoder);
                for line in reader.lines() {
                    let Ok(line) = line else { continue };
                    let line = line.trim();
                    if line.is_empty() {
                        continue;
                    }
                    if let Ok(rec) = serde_json::from_str::<Value>(line) {
                        records.push(rec);
                    }
                }
            }
            if records.len() >= window * 3 {
                break;
            }
        }
    }

    records.sort_by_key(|r| std::cmp::Reverse(record_ts(r)));
    records.truncate(window);
    Ok(records)
}

pub fn compute_governor_state(records: &[Value]) -> GovernorState {
    let n = records.len();
    if n == 0 {
        return GovernorState {
            multiplier: 1.0,
            gate_open: true,
            reason: "NO_DATA".into(),
            instability_rate: 0.0,
            corridor_rate: 0.0,
            window_size: 0,
            ts: now_ts(),
        };
    }

    let instability_count = records
        .iter()
        .filter(|r| {
            r.get("instability_type")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                != "STABLE"
        })
        .count();
    let corridor_count = records
        .iter()
        .filter(|r| r.get("corridor").and_then(|v| v.as_bool()).unwrap_or(false))
        .count();

    let instability_rate = instability_count as f64 / n as f64;
    let corridor_rate = corridor_count as f64 / n as f64;

    let (multiplier, gate_open, reason) = if instability_rate > HALT_THRESHOLD {
        (
            0.0,
            false,
            format!("HALT (instability_rate={instability_rate:.3} > {HALT_THRESHOLD})"),
        )
    } else if corridor_rate > THROTTLE_THRESHOLD {
        (
            0.65,
            true,
            format!("THROTTLE (corridor_rate={corridor_rate:.3} > {THROTTLE_THRESHOLD})"),
        )
    } else {
        (
            1.0,
            true,
            format!("NOMINAL (instability={instability_rate:.3}, corridor={corridor_rate:.3})"),
        )
    };

    GovernorState {
        multiplier,
        gate_open,
        reason,
        instability_rate: (instability_rate * 10000.0).round() / 10000.0,
        corridor_rate: (corridor_rate * 10000.0).round() / 10000.0,
        window_size: n,
        ts: now_ts(),
    }
}

pub fn write_governor_state(state: &GovernorState, bridge_path: &Path) -> Result<()> {
    if let Some(parent) = bridge_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let tmp = bridge_path.with_extension("json.tmp");
    {
        let mut f = File::create(&tmp).with_context(|| format!("create {}", tmp.display()))?;
        let text = serde_json::to_string_pretty(state)?;
        f.write_all(text.as_bytes())?;
        f.write_all(b"\n")?;
    }
    fs::rename(&tmp, bridge_path).with_context(|| {
        format!(
            "atomic replace {} → {}",
            tmp.display(),
            bridge_path.display()
        )
    })?;
    Ok(())
}

pub fn run_once(archive_dir: &Path, window: usize, bridge_path: &Path) -> Result<GovernorState> {
    let records = iter_recent_records(archive_dir, window)?;
    let state = compute_governor_state(&records);
    write_governor_state(&state, bridge_path)?;
    Ok(state)
}

pub fn run_loop(
    archive_dir: &Path,
    window: usize,
    interval_sec: f64,
    bridge_path: &Path,
) -> Result<()> {
    eprintln!(
        "[governor] starting — archive={} window={window} interval={interval_sec}s",
        archive_dir.display()
    );
    eprintln!("[governor] thresholds — halt>{HALT_THRESHOLD} throttle>{THROTTLE_THRESHOLD}");
    eprintln!("[governor] bridge → {}", bridge_path.display());

    let mut prev_reason: Option<String> = None;
    loop {
        match run_once(archive_dir, window, bridge_path) {
            Ok(state) => {
                if prev_reason.as_deref() != Some(&state.reason) {
                    let gate = if state.gate_open { "OPEN" } else { "CLOSED" };
                    eprintln!(
                        "[governor] → mult={:.2} gate={gate} | {}",
                        state.multiplier, state.reason
                    );
                    prev_reason = Some(state.reason.clone());
                }
            }
            Err(e) => eprintln!("[governor] error: {e:#}"),
        }
        thread::sleep(Duration::from_secs_f64(interval_sec.max(0.05)));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn nominal_when_stable() {
        let records = vec![
            json!({"ts": 1, "instability_type": "STABLE", "corridor": false}),
            json!({"ts": 2, "instability_type": "STABLE", "corridor": false}),
        ];
        let s = compute_governor_state(&records);
        assert_eq!(s.multiplier, 1.0);
        assert!(s.gate_open);
        assert!(s.reason.starts_with("NOMINAL"));
    }

    #[test]
    fn halt_on_instability() {
        let records = vec![
            json!({"ts": 1, "instability_type": "ENTROPIC_COLLAPSE", "corridor": false}),
            json!({"ts": 2, "instability_type": "EXPANSION_STRESS", "corridor": false}),
        ];
        let s = compute_governor_state(&records);
        assert_eq!(s.multiplier, 0.0);
        assert!(!s.gate_open);
    }

    #[test]
    fn throttle_on_corridor() {
        let records = vec![
            json!({"ts": 1, "instability_type": "STABLE", "corridor": true}),
            json!({"ts": 2, "instability_type": "STABLE", "corridor": true}),
            json!({"ts": 3, "instability_type": "STABLE", "corridor": true}),
            json!({"ts": 4, "instability_type": "STABLE", "corridor": false}),
            json!({"ts": 5, "instability_type": "STABLE", "corridor": false}),
        ];
        let s = compute_governor_state(&records);
        assert_eq!(s.multiplier, 0.65);
        assert!(s.gate_open);
    }

    #[test]
    fn no_data_default() {
        let s = compute_governor_state(&[]);
        assert_eq!(s.reason, "NO_DATA");
        assert_eq!(s.multiplier, 1.0);
    }
}
