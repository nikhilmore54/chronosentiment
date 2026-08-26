//! Phase 4 — Timestamp-Locked Chronology Recovery Engine
//!
//! Invariant: T_chronology ⊥ S_ecology
//!
//! The recovery layer is NEVER allowed to:
//!   - interpolate
//!   - forward-fill
//!   - infer future candles
//!   - substitute adjacent timestamps
//!
//! Only exact timestamp reconstruction is permitted.
//!
//! Recovery State Machine:
//!   [ PENDING ] → [ FETCHED ] → [ VERIFIED_TS_MATCH ] → [ RECOVERED ]
//!
//! Failure states:
//!   [ EXPIRED ] [ PERMANENT_GAP ] [ INVALID ]

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

// ── Constants ─────────────────────────────────────────────────────────────────

pub const BAR_SEC: i64 = 300;
pub const MAX_FETCH_ATTEMPTS: u32 = 3;
pub const EXPIRY_HOURS: f64 = 48.0;

// ── Repair Request Schema ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RepairStatus {
    Pending,
    Fetched,
    VerifiedTsMatch,
    Recovered,
    Expired,
    PermanentGap,
    Invalid,
}

impl std::fmt::Display for RepairStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            RepairStatus::Pending => "PENDING",
            RepairStatus::Fetched => "FETCHED",
            RepairStatus::VerifiedTsMatch => "VERIFIED_TS_MATCH",
            RepairStatus::Recovered => "RECOVERED",
            RepairStatus::Expired => "EXPIRED",
            RepairStatus::PermanentGap => "PERMANENT_GAP",
            RepairStatus::Invalid => "INVALID",
        };
        write!(f, "{s}")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepairRequest {
    pub batch_id: u32,
    pub symbol: String,
    pub target_ts: i64,
    pub bar_sec: i64,
    pub created_at_utc: String,
    pub reason: String,
    pub status: RepairStatus,
    pub attempts: u32,
    pub provider: String,
    #[serde(default)]
    pub history: Vec<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recovered_at_utc: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provenance_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recovery_meta: Option<Value>,
}

impl RepairRequest {
    pub fn new(
        batch_id: u32,
        symbol: &str,
        target_ts: i64,
        reason: &str,
        bar_sec: i64,
        provider: &str,
    ) -> Self {
        Self {
            batch_id,
            symbol: symbol.to_string(),
            target_ts,
            bar_sec,
            created_at_utc: Utc::now().to_rfc3339(),
            reason: reason.to_string(),
            status: RepairStatus::Pending,
            attempts: 0,
            provider: provider.to_string(),
            history: vec![],
            recovered_at_utc: None,
            provenance_path: None,
            recovery_meta: None,
        }
    }

    fn push_event(&mut self, event: Value) {
        self.history.push(event);
    }
}

// ── OHLCV ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ohlcv {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

// ── Repair Queue I/O ──────────────────────────────────────────────────────────

pub fn repair_queue_dir(archive_root: &Path, batch_id: u32) -> PathBuf {
    archive_root.join(format!("repair_queue/batch_{batch_id:03}"))
}

pub fn repair_request_path(
    archive_root: &Path,
    batch_id: u32,
    target_ts: i64,
    symbol: &str,
) -> PathBuf {
    let safe_sym = symbol.replace('.', "_").replace('-', "_");
    repair_queue_dir(archive_root, batch_id).join(format!("{target_ts}_{safe_sym}.json"))
}

pub fn load_repair_request(path: &Path) -> Result<RepairRequest> {
    let text = fs::read_to_string(path)
        .with_context(|| format!("read repair request {}", path.display()))?;
    serde_json::from_str(&text).with_context(|| format!("parse repair request {}", path.display()))
}

pub fn save_repair_request(req: &RepairRequest, path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let text = serde_json::to_string_pretty(req)?;
    fs::write(path, text)?;
    Ok(())
}

/// Queue a repair request. Idempotent — skips if already in terminal state.
pub fn queue_repair(
    archive_root: &Path,
    batch_id: u32,
    symbol: &str,
    target_ts: i64,
    reason: &str,
    bar_sec: i64,
    provider: &str,
) -> Result<PathBuf> {
    let path = repair_request_path(archive_root, batch_id, target_ts, symbol);
    if path.exists() {
        let existing = load_repair_request(&path)?;
        match existing.status {
            RepairStatus::Recovered | RepairStatus::PermanentGap | RepairStatus::Invalid => {
                println!(
                    "  ⚠️  {symbol}@{target_ts}: already terminal ({}), skipping",
                    existing.status
                );
                return Ok(path);
            }
            _ => {
                println!(
                    "  ↩️  {symbol}@{target_ts}: already queued ({})",
                    existing.status
                );
                return Ok(path);
            }
        }
    }
    let req = RepairRequest::new(batch_id, symbol, target_ts, reason, bar_sec, provider);
    save_repair_request(&req, &path)?;
    println!("  📋 Queued repair: {symbol}@{target_ts} reason={reason}");
    Ok(path)
}

// ── Timestamp-Locked Fetch via Yahoo Finance v8 API ───────────────────────────

/// Fetch the candle for exactly `target_ts` from Yahoo Finance.
///
/// Hard constraint: provider_ts MUST equal target_ts.
/// No nearest-neighbor substitution. No forward-fill. No synthetic continuity.
///
/// Returns Some(Ohlcv) if exact match found, None otherwise.
pub fn fetch_exact_ts(symbol: &str, target_ts: i64, bar_sec: i64) -> Result<Option<Ohlcv>> {
    // Fetch a small window: [target_ts - bar_sec, target_ts + bar_sec * 2]
    let period1 = target_ts - bar_sec;
    let period2 = target_ts + bar_sec * 2;

    // Yahoo Finance v8 chart API — same endpoint yfinance uses internally
    let url = format!(
        "https://query1.finance.yahoo.com/v8/finance/chart/{symbol}?period1={period1}&period2={period2}&interval=5m&includePrePost=false"
    );

    let client = reqwest::blocking::Client::builder()
        .user_agent("Mozilla/5.0 (compatible; cs-ingest/0.1)")
        .timeout(std::time::Duration::from_secs(15))
        .build()?;

    let resp = client
        .get(&url)
        .send()
        .with_context(|| format!("fetch {symbol}@{target_ts}"))?;

    if !resp.status().is_success() {
        return Ok(None);
    }

    let body: Value = resp.json()?;

    // Parse Yahoo v8 response structure
    let result = body
        .pointer("/chart/result/0")
        .ok_or_else(|| anyhow::anyhow!("no chart result for {symbol}"))?;

    let timestamps = result
        .pointer("/timestamp")
        .and_then(|v| v.as_array())
        .ok_or_else(|| anyhow::anyhow!("no timestamps in chart result"))?;

    let opens = result
        .pointer("/indicators/quote/0/open")
        .and_then(|v| v.as_array());
    let highs = result
        .pointer("/indicators/quote/0/high")
        .and_then(|v| v.as_array());
    let lows = result
        .pointer("/indicators/quote/0/low")
        .and_then(|v| v.as_array());
    let closes = result
        .pointer("/indicators/quote/0/close")
        .and_then(|v| v.as_array());
    let volumes = result
        .pointer("/indicators/quote/0/volume")
        .and_then(|v| v.as_array());

    // Timestamp-locked verification: exact match only
    for (i, ts_val) in timestamps.iter().enumerate() {
        let provider_ts = ts_val.as_i64().unwrap_or(0);
        if provider_ts == target_ts {
            let get_f64 = |arr: Option<&Vec<Value>>, idx: usize| -> f64 {
                arr.and_then(|a| a.get(idx))
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0)
            };
            let ohlcv = Ohlcv {
                open: get_f64(opens, i),
                high: get_f64(highs, i),
                low: get_f64(lows, i),
                close: get_f64(closes, i),
                volume: get_f64(volumes, i),
            };
            // Validate: reject zero/null candles
            if ohlcv.close <= 0.0 || ohlcv.open <= 0.0 {
                return Ok(None);
            }
            return Ok(Some(ohlcv));
        }
    }

    // No exact timestamp match found
    Ok(None)
}

// ── Provenance Ledger ─────────────────────────────────────────────────────────

pub fn provenance_dir(archive_root: &Path, symbol: &str, target_ts: i64) -> PathBuf {
    let safe_sym = symbol.replace('.', "_").replace('-', "_");
    archive_root
        .join("provenance")
        .join(&safe_sym)
        .join(target_ts.to_string())
}

pub fn next_revision_index(prov_dir: &Path) -> u32 {
    if !prov_dir.exists() {
        return 1;
    }
    let mut max_idx = 0u32;
    if let Ok(entries) = fs::read_dir(prov_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let s = name.to_string_lossy();
            if s.starts_with("rho_") && s.ends_with(".json") {
                if let Ok(idx) = s[4..s.len() - 5].parse::<u32>() {
                    max_idx = max_idx.max(idx);
                }
            }
        }
    }
    max_idx + 1
}

/// Append an immutable provenance revision. Never overwrites prior history.
pub fn append_provenance(
    archive_root: &Path,
    symbol: &str,
    target_ts: i64,
    ohlcv: &Ohlcv,
    provider: &str,
    reason: &str,
) -> Result<PathBuf> {
    let prov_dir = provenance_dir(archive_root, symbol, target_ts);
    fs::create_dir_all(&prov_dir)?;

    let mut rev_idx = next_revision_index(&prov_dir);
    let now = Utc::now();
    let ingest_ts = now.timestamp();

    let mut file;
    let mut rev_path;
    loop {
        rev_path = prov_dir.join(format!("rho_{rev_idx}.json"));
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&rev_path)
        {
            Ok(f) => {
                file = f;
                break;
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                rev_idx += 1;
            }
            Err(e) => {
                return Err(anyhow::anyhow!(e).context(format!(
                    "Failed to atomically create revision file at {:?}",
                    rev_path
                )));
            }
        }
    }

    let revision = serde_json::json!({
        "schema_version": 1,
        "symbol": symbol,
        "market_ts": target_ts,
        "ingest_ts": ingest_ts,
        "revision_index": rev_idx,
        "provider": provider,
        "ohlcv": {
            "open": ohlcv.open,
            "high": ohlcv.high,
            "low": ohlcv.low,
            "close": ohlcv.close,
            "volume": ohlcv.volume,
        },
        "revision_reason": reason,
        "retrieved_at_utc": now.to_rfc3339(),
    });

    let text = serde_json::to_string_pretty(&revision)?;
    file.write_all(text.as_bytes())?;
    file.flush()?;
    Ok(rev_path)
}

// ── Recovery State Machine ────────────────────────────────────────────────────

/// Run one repair request through the full state machine.
/// Returns the final status.
pub fn process_repair_request(
    req: &mut RepairRequest,
    path: &Path,
    archive_root: &Path,
) -> Result<RepairStatus> {
    let symbol = req.symbol.clone();
    let target_ts = req.target_ts;
    let bar_sec = req.bar_sec;
    let provider = req.provider.clone();

    // Check expiry
    let created: DateTime<Utc> = DateTime::parse_from_rfc3339(&req.created_at_utc)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now());
    let age_hours = (Utc::now() - created).num_seconds() as f64 / 3600.0;
    if age_hours > EXPIRY_HOURS {
        req.status = RepairStatus::Expired;
        req.push_event(serde_json::json!({
            "ts_utc": Utc::now().to_rfc3339(),
            "event": "EXPIRED",
            "age_hours": age_hours,
        }));
        save_repair_request(req, path)?;
        return Ok(RepairStatus::Expired);
    }

    req.attempts += 1;
    req.push_event(serde_json::json!({
        "ts_utc": Utc::now().to_rfc3339(),
        "event": "FETCH_ATTEMPT",
        "attempt": req.attempts,
    }));

    // ── FETCHED ───────────────────────────────────────────────────────────────
    let ohlcv_opt = fetch_exact_ts(&symbol, target_ts, bar_sec).unwrap_or(None);

    if ohlcv_opt.is_none() {
        req.push_event(serde_json::json!({
            "ts_utc": Utc::now().to_rfc3339(),
            "event": "FETCH_FAILED",
            "reason": "no_exact_ts_match",
        }));
        if req.attempts >= MAX_FETCH_ATTEMPTS {
            req.status = RepairStatus::PermanentGap;
            req.push_event(serde_json::json!({
                "ts_utc": Utc::now().to_rfc3339(),
                "event": "PERMANENT_GAP",
                "reason": format!("exhausted {} attempts", MAX_FETCH_ATTEMPTS),
            }));
        }
        save_repair_request(req, path)?;
        return Ok(req.status.clone());
    }

    let ohlcv = ohlcv_opt.unwrap();

    req.push_event(serde_json::json!({
        "ts_utc": Utc::now().to_rfc3339(),
        "event": "FETCHED",
        "ohlcv_preview": {
            "open": (ohlcv.open * 10000.0).round() / 10000.0,
            "high": (ohlcv.high * 10000.0).round() / 10000.0,
            "low": (ohlcv.low * 10000.0).round() / 10000.0,
            "close": (ohlcv.close * 10000.0).round() / 10000.0,
        },
    }));

    // ── VERIFIED_TS_MATCH ─────────────────────────────────────────────────────
    // fetch_exact_ts already enforces exact match. Record the verification event.
    req.push_event(serde_json::json!({
        "ts_utc": Utc::now().to_rfc3339(),
        "event": "VERIFIED_TS_MATCH",
        "provider_ts": target_ts,
        "barrier_ts": target_ts,
    }));

    // ── RECOVERED ─────────────────────────────────────────────────────────────
    let rev_path = append_provenance(
        archive_root,
        &symbol,
        target_ts,
        &ohlcv,
        &provider,
        &req.reason,
    )?;

    let rev_idx = next_revision_index(&rev_path.parent().unwrap()) - 1;

    let recovery_meta = serde_json::json!({
        "symbol": symbol,
        "ts": target_ts,
        "retrieved_at_utc": Utc::now().to_rfc3339(),
        "revision_index": rev_idx,
        "provider": provider,
        "recovered": true,
        "recovery_reason": "timestamp_locked_repair",
        "ohlcv": {
            "open": ohlcv.open,
            "high": ohlcv.high,
            "low": ohlcv.low,
            "close": ohlcv.close,
            "volume": ohlcv.volume,
        },
        "batch_id": req.batch_id,
    });

    req.status = RepairStatus::Recovered;
    req.recovered_at_utc = Some(Utc::now().to_rfc3339());
    req.provenance_path = Some(rev_path.to_string_lossy().to_string());
    req.recovery_meta = Some(recovery_meta);
    req.push_event(serde_json::json!({
        "ts_utc": Utc::now().to_rfc3339(),
        "event": "RECOVERED",
        "provenance_path": rev_path.to_string_lossy(),
    }));

    save_repair_request(req, path)?;
    println!(
        "  ✅ RECOVERED {symbol}@{target_ts} → {}",
        rev_path.display()
    );
    Ok(RepairStatus::Recovered)
}

// ── Gap Detection from live_session_steps.jsonl ───────────────────────────────

#[derive(Debug)]
pub struct GapDescriptor {
    pub symbol: String,
    pub target_ts: i64,
    pub reason: String,
}

pub fn detect_gaps_from_steps(steps_path: &Path) -> Result<Vec<GapDescriptor>> {
    let text = fs::read_to_string(steps_path)
        .with_context(|| format!("read steps log {}", steps_path.display()))?;

    let mut gaps: Vec<GapDescriptor> = vec![];
    let mut seen = std::collections::HashSet::new();

    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let step: Value = match serde_json::from_str(line) {
            Ok(v) => v,
            Err(_) => continue,
        };

        let target_ts = step
            .get("target_ts")
            .or_else(|| step.get("ts"))
            .and_then(|v| v.as_i64());
        let target_ts = match target_ts {
            Some(ts) => ts,
            None => continue,
        };

        let barrier_committed = step
            .get("barrier_committed")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let reason = if !barrier_committed {
            step.get("skip_reason")
                .and_then(|v| v.as_str())
                .unwrap_or("quorum_not_met")
                .to_string()
        } else {
            "partial_quorum_gap".to_string()
        };

        // Scan symbol_close_lags for no_data symbols
        if let Some(lags) = step.get("symbol_close_lags").and_then(|v| v.as_array()) {
            for lag in lags {
                if lag.get("status").and_then(|v| v.as_str()) == Some("no_data") {
                    if let Some(sym) = lag.get("symbol").and_then(|v| v.as_str()) {
                        let key = (sym.to_string(), target_ts);
                        if seen.insert(key) {
                            gaps.push(GapDescriptor {
                                symbol: sym.to_string(),
                                target_ts,
                                reason: reason.clone(),
                            });
                        }
                    }
                }
            }
        }
    }

    Ok(gaps)
}

// ── Repair Config ─────────────────────────────────────────────────────────────

pub struct RepairConfig {
    pub archive_root: PathBuf,
    pub batch_id: u32,
}

// ── Public API ────────────────────────────────────────────────────────────────

pub fn cmd_queue(
    cfg: &RepairConfig,
    symbol: &str,
    target_ts: i64,
    reason: &str,
    bar_sec: i64,
    provider: &str,
) -> Result<()> {
    let path = queue_repair(
        &cfg.archive_root,
        cfg.batch_id,
        symbol,
        target_ts,
        reason,
        bar_sec,
        provider,
    )?;
    println!("  Repair request: {}", path.display());
    Ok(())
}

pub fn cmd_detect(cfg: &RepairConfig, run_label: &str) -> Result<()> {
    // Resolve steps log path
    let steps_path = if run_label.is_empty() {
        cfg.archive_root.join(format!(
            "batches/batch_{:03}/metadata/live_session_steps.jsonl",
            cfg.batch_id
        ))
    } else {
        cfg.archive_root.join(format!(
            "batches/batch_{:03}/runs/{run_label}/metadata/live_session_steps.jsonl",
            cfg.batch_id
        ))
    };

    println!("🔍 Detecting gaps from {}", steps_path.display());
    let gaps = detect_gaps_from_steps(&steps_path)?;

    if gaps.is_empty() {
        println!("  ✅ No gaps detected");
        return Ok(());
    }

    println!("  Found {} gap(s):", gaps.len());
    for g in &gaps {
        queue_repair(
            &cfg.archive_root,
            cfg.batch_id,
            &g.symbol,
            g.target_ts,
            &g.reason,
            BAR_SEC,
            "yfinance",
        )?;
    }
    println!("  📋 Queued {} repair request(s)", gaps.len());
    Ok(())
}

pub fn cmd_process(cfg: &RepairConfig) -> Result<()> {
    let q_dir = repair_queue_dir(&cfg.archive_root, cfg.batch_id);
    if !q_dir.exists() {
        println!("  ℹ️  No repair queue for batch_{:03}", cfg.batch_id);
        return Ok(());
    }

    let mut entries: Vec<PathBuf> = fs::read_dir(&q_dir)?
        .flatten()
        .filter(|e| e.path().extension().map(|x| x == "json").unwrap_or(false))
        .map(|e| e.path())
        .collect();
    entries.sort();

    let pending: Vec<PathBuf> = entries
        .into_iter()
        .filter(|p| {
            load_repair_request(p)
                .map(|r| r.status == RepairStatus::Pending)
                .unwrap_or(false)
        })
        .collect();

    if pending.is_empty() {
        println!("  ✅ No pending repairs for batch_{:03}", cfg.batch_id);
        return Ok(());
    }

    println!(
        "🔧 Processing {} pending repair(s) for batch_{:03}...",
        pending.len(),
        cfg.batch_id
    );

    let mut counts: std::collections::HashMap<String, u32> = std::collections::HashMap::new();
    for rp in &pending {
        let mut req = load_repair_request(rp)?;
        println!(
            "  → {}@{} (attempt {}/{})",
            req.symbol,
            req.target_ts,
            req.attempts + 1,
            MAX_FETCH_ATTEMPTS
        );
        let status = process_repair_request(&mut req, rp, &cfg.archive_root)?;
        *counts.entry(status.to_string()).or_insert(0) += 1;
    }

    println!("\n  Results: {counts:?}");
    Ok(())
}

pub fn cmd_status(cfg: &RepairConfig) -> Result<()> {
    let q_dir = repair_queue_dir(&cfg.archive_root, cfg.batch_id);
    if !q_dir.exists() {
        println!("  ℹ️  No repair queue for batch_{:03}", cfg.batch_id);
        return Ok(());
    }

    let mut entries: Vec<PathBuf> = fs::read_dir(&q_dir)?
        .flatten()
        .filter(|e| e.path().extension().map(|x| x == "json").unwrap_or(false))
        .map(|e| e.path())
        .collect();
    entries.sort();

    if entries.is_empty() {
        println!("  ✅ Empty repair queue for batch_{:03}", cfg.batch_id);
        return Ok(());
    }

    let mut by_status: std::collections::HashMap<String, Vec<RepairRequest>> =
        std::collections::HashMap::new();
    for rp in &entries {
        if let Ok(req) = load_repair_request(rp) {
            by_status
                .entry(req.status.to_string())
                .or_default()
                .push(req);
        }
    }

    println!("\n📊 Repair Queue — batch_{:03}", cfg.batch_id);
    println!("{}", "=".repeat(60));
    let mut status_keys: Vec<String> = by_status.keys().cloned().collect();
    status_keys.sort();
    for status in &status_keys {
        let reqs = &by_status[status];
        println!("  {status}: {}", reqs.len());
        for req in reqs.iter().take(5) {
            println!(
                "    {}@{} reason={} attempts={}",
                req.symbol, req.target_ts, req.reason, req.attempts
            );
        }
        if reqs.len() > 5 {
            println!("    ... and {} more", reqs.len() - 5);
        }
    }
    println!("  Total: {}", entries.len());
    Ok(())
}
