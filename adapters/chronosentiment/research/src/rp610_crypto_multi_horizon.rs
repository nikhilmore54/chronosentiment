//! RP-610 Crypto multi-horizon observation engine (paper spot-sim).
//!
//! H0 (≤60m) remains owned by RP-608. H1/H2/H3 are observation-only.
//! `live_order_authority = 0.0`.
//!
//! Rust port of `scripts/rp610_crypto_multi_horizon.py` +
//! `scripts/rp610_crypto_multi_horizon_engine.py`.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

pub const SCHEMA_VERSION: &str = "rp610_crypto_multi_horizon_v1";
pub const PROTOCOL_ID: &str = "RP-610-CRYPTO-MULTI-HORIZON-001";
pub const LIVE_ORDER_AUTHORITY: f64 = 0.0;
pub const FRICTION_MODEL_ID: &str = "CRYPTO_SPOT_RT_BPS_V1";
pub const DEFAULT_RT_FRICTION_BPS: f64 = 12.0;
pub const DEFAULT_NOTIONAL_USDT: f64 = 1000.0;

const AMENDABLE: &[&str] = &[
    "stop_pct",
    "target_pct",
    "hold_days_min",
    "hold_days_max",
    "max_exit_ts",
    "thesis",
    "thesis_version",
    "source_signal_id",
];

fn round_n(x: f64, n: i32) -> f64 {
    let p = 10f64.powi(n);
    (x * p).round() / p
}

fn iso(dt: DateTime<Utc>) -> String {
    dt.to_rfc3339().replace("+00:00", "Z")
}

fn parse_ts(value: &Value) -> Option<DateTime<Utc>> {
    let s = value.as_str()?;
    let s = if s.ends_with('Z') {
        format!("{}+00:00", &s[..s.len() - 1])
    } else {
        s.to_string()
    };
    DateTime::parse_from_rfc3339(&s)
        .ok()
        .map(|dt| dt.with_timezone(&Utc))
}

fn stable_id(parts: &[&str]) -> String {
    let raw = parts.join("|");
    let mut hasher = Sha256::new();
    hasher.update(raw.as_bytes());
    format!("{:x}", hasher.finalize())[..20].to_string()
}

fn append_jsonl(path: &Path, row: &Value) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let mut f = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|e| e.to_string())?;
    writeln!(f, "{}", serde_json::to_string(row).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    Ok(())
}

fn load_jsonl(path: &Path) -> Vec<Value> {
    let Ok(text) = fs::read_to_string(path) else {
        return Vec::new();
    };
    text.lines()
        .filter(|l| !l.trim().is_empty())
        .filter_map(|l| serde_json::from_str(l).ok())
        .collect()
}

fn write_json(path: &Path, payload: &Value) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::write(
        path,
        serde_json::to_string_pretty(payload).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptoFriction {
    pub round_trip_friction_bps: f64,
    pub total_friction_usdt: f64,
    pub friction_model_id: String,
}

pub fn estimate_crypto_friction_usdt(
    entry_price: f64,
    exit_price: f64,
    quantity: f64,
    rt_friction_bps: f64,
) -> CryptoFriction {
    let notional = (entry_price * quantity).abs() + (exit_price * quantity).abs();
    let mid = if notional > 0.0 { notional / 2.0 } else { 0.0 };
    let total = mid * (rt_friction_bps / 10_000.0);
    CryptoFriction {
        round_trip_friction_bps: rt_friction_bps,
        total_friction_usdt: round_n(total, 6),
        friction_model_id: FRICTION_MODEL_ID.to_string(),
    }
}

pub fn default_horizons() -> BTreeMap<String, Value> {
    let mut m = BTreeMap::new();
    m.insert(
        "H0_INTRADAY_60M".into(),
        json!({
            "horizon_id": "H0_INTRADAY_60M",
            "label": "Intraday / ≤60m trade-plan overlay (owned by RP-608)",
            "product_type": "SPOT_SIM",
            "owner_protocol": "RP-608-TRADE-PLAN-OVERLAY-001",
            "hold_days_min": 0,
            "hold_days_max": 0,
            "observational_link_only": true,
            "default_review": Value::Null,
        }),
    );
    m.insert(
        "H1_MULTI_DAY".into(),
        json!({
            "horizon_id": "H1_MULTI_DAY",
            "label": "Multi-day swing",
            "product_type": "SPOT_SIM",
            "owner_protocol": PROTOCOL_ID,
            "hold_days_min": 2,
            "hold_days_max": 14,
            "observational_link_only": false,
            "default_review": "ON_EXIT_OR_MAX_DAYS",
            "default_stop_pct": 0.04,
            "default_target_pct": 0.08,
        }),
    );
    m.insert(
        "H2_MULTI_WEEK".into(),
        json!({
            "horizon_id": "H2_MULTI_WEEK",
            "label": "Multi-week position",
            "product_type": "SPOT_SIM",
            "owner_protocol": PROTOCOL_ID,
            "hold_days_min": 14,
            "hold_days_max": 56,
            "observational_link_only": false,
            "default_review": "WEEKLY_MARK",
            "default_stop_pct": 0.08,
            "default_target_pct": 0.18,
        }),
    );
    m.insert(
        "H3_LONG_TERM".into(),
        json!({
            "horizon_id": "H3_LONG_TERM",
            "label": "Long-term thesis hold",
            "product_type": "SPOT_SIM",
            "owner_protocol": PROTOCOL_ID,
            "hold_days_min": 90,
            "hold_days_max": Value::Null,
            "observational_link_only": false,
            "default_review": "QUARTERLY",
            "review_cadence_days": 90,
        }),
    );
    m
}

/// Journal root for RP-610 books.
#[derive(Debug, Clone)]
pub struct Rp610Store {
    pub journals: PathBuf,
}

impl Rp610Store {
    pub fn new(journals: PathBuf) -> Self {
        Self { journals }
    }

    pub fn ensure(&self) -> Result<(), String> {
        fs::create_dir_all(&self.journals).map_err(|e| e.to_string())
    }

    pub fn candidates_path(&self) -> PathBuf {
        self.journals.join("RP610_candidates.jsonl")
    }
    pub fn day_path(&self) -> PathBuf {
        self.journals.join("RP610_day_trades.jsonl")
    }
    pub fn week_path(&self) -> PathBuf {
        self.journals.join("RP610_week_trades.jsonl")
    }
    pub fn long_path(&self) -> PathBuf {
        self.journals.join("RP610_long_term_holds.jsonl")
    }
    pub fn daily_log_path(&self) -> PathBuf {
        self.journals.join("RP610_daily_observation_log.jsonl")
    }

    pub fn book_path(&self, horizon_id: &str) -> Result<PathBuf, String> {
        match horizon_id {
            "H1_MULTI_DAY" => Ok(self.day_path()),
            "H2_MULTI_WEEK" => Ok(self.week_path()),
            "H3_LONG_TERM" => Ok(self.long_path()),
            _ => Err(format!("unsupported horizon for book: {horizon_id}")),
        }
    }

    pub fn write_horizon_registry(&self) -> Result<PathBuf, String> {
        self.ensure()?;
        let path = self.journals.join("RP610_horizon_registry.json");
        let now = iso(Utc::now());
        let gates = json!({
            "G-H1_MULTI_DAY": "CLOSED",
            "G-H2_MULTI_WEEK": "CLOSED",
            "G-H3_LONG_TERM": "CLOSED",
            "G-LIVE": "CLOSED",
        });
        let payload = json!({
            "schema_version": SCHEMA_VERSION,
            "protocol_id": PROTOCOL_ID,
            "market": "CRYPTO",
            "symbol_default": "BTCUSDT",
            "live_order_authority": LIVE_ORDER_AUTHORITY,
            "friction_model_id": FRICTION_MODEL_ID,
            "default_rt_friction_bps": DEFAULT_RT_FRICTION_BPS,
            "default_notional_usdt": DEFAULT_NOTIONAL_USDT,
            "updated_utc": now,
            "horizons": default_horizons(),
            "promotion_gates": gates,
            "notes": [
                "H0 owned by RP-608 trade-plan overlay; RP-610 must not mutate 60m exits.",
                "H1–H3 are observation-only spot-sim; no perpetual funding in v1.",
            ],
        });
        write_json(&path, &payload)?;
        write_json(
            &self.journals.join("RP610_promotion_gates.json"),
            &json!({
                "protocol_id": PROTOCOL_ID,
                "updated_utc": now,
                "gates": gates,
                "decision_status": "OBSERVE_ONLY_NO_CONCLUSION",
            }),
        )?;
        Ok(path)
    }
}

fn pnl_usdt(side: &str, entry: f64, exit_px: f64, qty: f64) -> f64 {
    if side == "LONG" {
        (exit_px - entry) * qty
    } else {
        (entry - exit_px) * qty
    }
}

pub fn build_candidate(
    symbol: &str,
    side: &str,
    entry_price: f64,
    entry_ts: DateTime<Utc>,
    horizon_id: &str,
    thesis: &str,
    source_signal_id: &str,
    notional_usdt: f64,
    stop_pct: Option<f64>,
    target_pct: Option<f64>,
) -> Result<Value, String> {
    let hz = default_horizons()
        .get(horizon_id)
        .cloned()
        .ok_or_else(|| format!("unknown horizon {horizon_id}"))?;
    if hz
        .get("observational_link_only")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
    {
        return Err(format!(
            "{horizon_id} is H0 link-only; use RP-608 for intraday"
        ));
    }
    let side_u = side.to_uppercase();
    if side_u != "LONG" && side_u != "SHORT" {
        return Err("side must be LONG or SHORT".into());
    }
    let qty = if entry_price != 0.0 {
        notional_usdt / entry_price
    } else {
        0.0
    };
    let stop = stop_pct.or_else(|| hz.get("default_stop_pct").and_then(|v| v.as_f64()));
    let target = target_pct.or_else(|| hz.get("default_target_pct").and_then(|v| v.as_f64()));
    let hold_max = hz.get("hold_days_max").and_then(|v| v.as_i64());
    let hold_min = hz.get("hold_days_min").and_then(|v| v.as_i64()).unwrap_or(0);
    let max_exit_ts = hold_max.map(|d| iso(entry_ts + Duration::days(d)));
    let entry_iso = iso(entry_ts);
    let cand_id = stable_id(&[
        PROTOCOL_ID,
        horizon_id,
        &symbol.to_uppercase(),
        &side_u,
        &entry_iso,
        &format!("{:.8}", entry_price),
    ]);
    Ok(json!({
        "schema_version": SCHEMA_VERSION,
        "protocol_id": PROTOCOL_ID,
        "candidate_id": cand_id,
        "horizon_id": horizon_id,
        "symbol": symbol.to_uppercase(),
        "side": side_u,
        "product_type": "SPOT_SIM",
        "live_order_authority": LIVE_ORDER_AUTHORITY,
        "entry_ts": entry_iso,
        "entry_price": entry_price,
        "quantity": qty,
        "notional_usdt": notional_usdt,
        "stop_pct": stop,
        "target_pct": target,
        "hold_days_min": hold_min,
        "hold_days_max": hold_max,
        "max_exit_ts": max_exit_ts,
        "thesis": thesis,
        "source_signal_id": source_signal_id,
        "status": "OPEN",
        "created_utc": iso(Utc::now()),
    }))
}

pub fn admit_candidate(store: &Rp610Store, candidate: &Value) -> Result<Value, String> {
    store.write_horizon_registry()?;
    let path = store.candidates_path();
    let cid = candidate
        .get("candidate_id")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let existing: std::collections::HashSet<String> = load_jsonl(&path)
        .iter()
        .filter_map(|r| {
            r.get("candidate_id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
        })
        .collect();
    if existing.contains(cid) {
        return Ok(candidate.clone());
    }
    append_jsonl(&path, candidate)?;
    let hid = candidate
        .get("horizon_id")
        .and_then(|v| v.as_str())
        .ok_or("missing horizon_id")?;
    let mut admit = candidate.clone();
    admit
        .as_object_mut()
        .unwrap()
        .insert("event".into(), json!("ADMIT"));
    append_jsonl(&store.book_path(hid)?, &admit)?;
    Ok(candidate.clone())
}

pub fn evaluate_open_hold(
    hold: &Value,
    mark_ts: DateTime<Utc>,
    mark_price: f64,
    force_close: bool,
    force_reason: &str,
) -> Option<Value> {
    let status = hold.get("status").and_then(|v| v.as_str()).unwrap_or("");
    let event = hold.get("event").and_then(|v| v.as_str()).unwrap_or("");
    if status == "CLOSED" {
        return None;
    }
    if !matches!(status, "" | "OPEN" | "ADMIT" | "HOLDING")
        && !matches!(event, "" | "ADMIT")
    {
        // keep evaluating OPEN-like rows only
    }
    let entry = hold.get("entry_price")?.as_f64()?;
    let qty = hold.get("quantity")?.as_f64()?;
    let side = hold.get("side")?.as_str()?.to_uppercase();
    let entry_ts = parse_ts(hold.get("entry_ts")?)?;

    let mut reason: Option<String> = None;
    let mut exit_px = mark_price;

    if force_close {
        reason = Some(if force_reason.is_empty() {
            "FORCE_CLOSE".into()
        } else {
            force_reason.into()
        });
    } else {
        if let Some(stop_pct) = hold.get("stop_pct").and_then(|v| v.as_f64()) {
            if side == "LONG" && mark_price <= entry * (1.0 - stop_pct) {
                reason = Some("INVALIDATION".into());
                exit_px = entry * (1.0 - stop_pct);
            } else if side == "SHORT" && mark_price >= entry * (1.0 + stop_pct) {
                reason = Some("INVALIDATION".into());
                exit_px = entry * (1.0 + stop_pct);
            }
        }
        if reason.is_none() {
            if let Some(target_pct) = hold.get("target_pct").and_then(|v| v.as_f64()) {
                if side == "LONG" && mark_price >= entry * (1.0 + target_pct) {
                    reason = Some("TARGET".into());
                    exit_px = entry * (1.0 + target_pct);
                } else if side == "SHORT" && mark_price <= entry * (1.0 - target_pct) {
                    reason = Some("TARGET".into());
                    exit_px = entry * (1.0 - target_pct);
                }
            }
        }
        if reason.is_none() {
            if let Some(max_exit) = hold.get("max_exit_ts").and_then(parse_ts) {
                if mark_ts >= max_exit {
                    reason = Some("MAX_HOLD_DAYS".into());
                    exit_px = mark_price;
                }
            }
        }
    }

    let reason = reason?;
    let gross = pnl_usdt(&side, entry, exit_px, qty);
    let fr = estimate_crypto_friction_usdt(entry, exit_px, qty, DEFAULT_RT_FRICTION_BPS);
    let net = gross - fr.total_friction_usdt;
    let hold_days = ((mark_ts - entry_ts).num_seconds() as f64 / 86_400.0).max(0.0);
    Some(json!({
        "schema_version": SCHEMA_VERSION,
        "protocol_id": PROTOCOL_ID,
        "event": "CLOSE",
        "status": "CLOSED",
        "candidate_id": hold.get("candidate_id"),
        "horizon_id": hold.get("horizon_id"),
        "symbol": hold.get("symbol"),
        "side": side,
        "product_type": "SPOT_SIM",
        "live_order_authority": LIVE_ORDER_AUTHORITY,
        "entry_ts": hold.get("entry_ts"),
        "entry_price": entry,
        "exit_ts": iso(mark_ts),
        "exit_price": exit_px,
        "quantity": qty,
        "exit_reason": reason,
        "hold_days": round_n(hold_days, 4),
        "gross_pnl_usdt": round_n(gross, 6),
        "net_pnl_usdt": round_n(net, 6),
        "friction_usdt": fr.total_friction_usdt,
        "friction_model_id": fr.friction_model_id,
        "round_trip_friction_bps": fr.round_trip_friction_bps,
        "closed_utc": iso(Utc::now()),
    }))
}

fn reconstruct_opens(rows: &[Value]) -> BTreeMap<String, Value> {
    let mut by_id: BTreeMap<String, Value> = BTreeMap::new();
    for r in rows {
        let Some(cid) = r.get("candidate_id").and_then(|v| v.as_str()) else {
            continue;
        };
        let event = r.get("event").and_then(|v| v.as_str()).unwrap_or("");
        let status = r.get("status").and_then(|v| v.as_str()).unwrap_or("");
        if matches!(event, "CLOSE" | "SUPERSEDE") || matches!(status, "CLOSED" | "SUPERSEDED") {
            by_id.remove(cid);
            continue;
        }
        if event == "MARK" {
            if let Some(base) = by_id.get_mut(cid) {
                let map = base.as_object_mut().unwrap();
                map.insert("last_mark_price".into(), r.get("mark_price").cloned().unwrap_or(Value::Null));
                map.insert("last_mark_ts".into(), r.get("mark_ts").cloned().unwrap_or(Value::Null));
                map.insert(
                    "last_mtm_gross_usdt".into(),
                    r.get("mtm_gross_usdt").cloned().unwrap_or(Value::Null),
                );
            }
            continue;
        }
        if event == "AMEND" {
            if let Some(base) = by_id.get_mut(cid) {
                let mut next = base.clone();
                let map = next.as_object_mut().unwrap();
                if let Some(amended) = r.get("amended").and_then(|v| v.as_object()) {
                    for (k, v) in amended {
                        if AMENDABLE.contains(&k.as_str()) || k == "horizon_id" {
                            map.insert(k.clone(), v.clone());
                        }
                    }
                }
                if map.get("status").and_then(|v| v.as_str()).unwrap_or("").is_empty() {
                    map.insert("status".into(), json!("OPEN"));
                }
                map.insert(
                    "last_amendment_id".into(),
                    r.get("amendment_id").cloned().unwrap_or(Value::Null),
                );
                map.insert(
                    "last_amend_utc".into(),
                    r.get("amended_utc").cloned().unwrap_or(Value::Null),
                );
                let count = map.get("amend_count").and_then(|v| v.as_i64()).unwrap_or(0) + 1;
                map.insert("amend_count".into(), json!(count));
                *base = next;
            }
            continue;
        }
        if matches!(event, "ADMIT" | "")
            || matches!(status, "OPEN" | "ADMIT" | "HOLDING" | "")
        {
            if !by_id.contains_key(cid) || event == "ADMIT" {
                let mut row = r.clone();
                row.as_object_mut()
                    .unwrap()
                    .entry("amend_count".to_string())
                    .or_insert(json!(0));
                by_id.insert(cid.to_string(), row);
            }
        }
    }
    by_id
}

pub fn open_holds_from_book(store: &Rp610Store, horizon_id: &str) -> Result<Vec<Value>, String> {
    let rows = load_jsonl(&store.book_path(horizon_id)?);
    Ok(reconstruct_opens(&rows).into_values().collect())
}

pub fn find_open_hold(store: &Rp610Store, candidate_id: &str) -> Result<Option<Value>, String> {
    for hid in ["H1_MULTI_DAY", "H2_MULTI_WEEK", "H3_LONG_TERM"] {
        for hold in open_holds_from_book(store, hid)? {
            if hold.get("candidate_id").and_then(|v| v.as_str()) == Some(candidate_id) {
                return Ok(Some(hold));
            }
        }
    }
    Ok(None)
}

fn roll_horizon(
    store: &Rp610Store,
    hold: &Value,
    new_horizon_id: &str,
    reason: &str,
    stop_pct: Option<f64>,
    target_pct: Option<f64>,
    hold_days_max: Option<i64>,
    hold_days_min: Option<i64>,
    thesis: Option<&str>,
    thesis_version: Option<&str>,
    source_signal_id: Option<&str>,
    mark_price: Option<f64>,
    mark_ts: DateTime<Utc>,
) -> Result<Value, String> {
    let hz = default_horizons()
        .get(new_horizon_id)
        .cloned()
        .ok_or_else(|| format!("cannot roll to {new_horizon_id}"))?;
    if hz
        .get("observational_link_only")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
    {
        return Err(format!("cannot roll to {new_horizon_id}"));
    }
    let old_hid = hold
        .get("horizon_id")
        .and_then(|v| v.as_str())
        .ok_or("missing horizon")?;
    let entry_ts = parse_ts(hold.get("entry_ts").unwrap_or(&Value::Null))
        .ok_or("hold missing entry_ts")?;
    let mut child = build_candidate(
        hold.get("symbol").and_then(|v| v.as_str()).unwrap_or("BTCUSDT"),
        hold.get("side").and_then(|v| v.as_str()).unwrap_or("LONG"),
        hold.get("entry_price").and_then(|v| v.as_f64()).unwrap_or(0.0),
        entry_ts,
        new_horizon_id,
        thesis.unwrap_or_else(|| hold.get("thesis").and_then(|v| v.as_str()).unwrap_or("")),
        source_signal_id.unwrap_or_else(|| {
            hold.get("source_signal_id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
        }),
        hold.get("notional_usdt")
            .and_then(|v| v.as_f64())
            .unwrap_or(DEFAULT_NOTIONAL_USDT),
        stop_pct.or_else(|| hold.get("stop_pct").and_then(|v| v.as_f64())),
        target_pct.or_else(|| hold.get("target_pct").and_then(|v| v.as_f64())),
    )?;
    let parent_cid = hold
        .get("candidate_id")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let mark_iso = iso(mark_ts);
    let child_id = stable_id(&[PROTOCOL_ID, "ROLL", parent_cid, new_horizon_id, &mark_iso]);
    {
        let map = child.as_object_mut().unwrap();
        map.insert("candidate_id".into(), json!(child_id));
        if let Some(v) = hold_days_min {
            map.insert("hold_days_min".into(), json!(v));
        }
        if let Some(v) = hold_days_max {
            map.insert("hold_days_max".into(), json!(v));
            map.insert(
                "max_exit_ts".into(),
                json!(iso(entry_ts + Duration::days(v))),
            );
        }
        if let Some(tv) = thesis_version {
            map.insert("thesis_version".into(), json!(tv));
        }
        map.insert("parent_candidate_id".into(), json!(parent_cid));
        map.insert("roll_reason".into(), json!(reason));
        if new_horizon_id == "H3_LONG_TERM" {
            map.insert("status".into(), json!("HOLDING"));
        }
    }
    let supersede = json!({
        "schema_version": SCHEMA_VERSION,
        "protocol_id": PROTOCOL_ID,
        "event": "SUPERSEDE",
        "status": "SUPERSEDED",
        "candidate_id": parent_cid,
        "horizon_id": old_hid,
        "symbol": hold.get("symbol"),
        "supersede_reason": "HORIZON_CHANGED",
        "reason": reason,
        "child_candidate_id": child.get("candidate_id"),
        "new_horizon_id": new_horizon_id,
        "mark_ts": iso(mark_ts),
        "mark_price": mark_price,
        "live_order_authority": LIVE_ORDER_AUTHORITY,
        "superseded_utc": iso(Utc::now()),
        "friction_usdt": 0.0,
        "net_pnl_usdt": Value::Null,
    });
    append_jsonl(&store.book_path(old_hid)?, &supersede)?;
    admit_candidate(store, &child)?;
    append_jsonl(
        &store.candidates_path(),
        &json!({
            "event": "HORIZON_ROLL",
            "parent_candidate_id": parent_cid,
            "child_candidate_id": child.get("candidate_id"),
            "from_horizon": old_hid,
            "to_horizon": new_horizon_id,
            "reason": reason,
            "rolled_utc": iso(Utc::now()),
        }),
    )?;
    Ok(json!({"supersede": supersede, "child": child}))
}

pub struct AmendArgs<'a> {
    pub reason: &'a str,
    pub stop_pct: Option<f64>,
    pub target_pct: Option<f64>,
    pub hold_days_max: Option<i64>,
    pub hold_days_min: Option<i64>,
    pub max_exit_ts: Option<&'a str>,
    pub thesis: Option<&'a str>,
    pub thesis_version: Option<&'a str>,
    pub source_signal_id: Option<&'a str>,
    pub new_horizon_id: Option<&'a str>,
    pub mark_price: Option<f64>,
    pub mark_ts: Option<DateTime<Utc>>,
}

pub fn amend_hold(
    store: &Rp610Store,
    candidate_id: &str,
    args: AmendArgs<'_>,
) -> Result<Value, String> {
    let hold = find_open_hold(store, candidate_id)?
        .ok_or_else(|| format!("no open hold for candidate_id={candidate_id}"))?;
    if let Some(new_hid) = args.new_horizon_id {
        if new_hid != hold.get("horizon_id").and_then(|v| v.as_str()).unwrap_or("") {
            return roll_horizon(
                store,
                &hold,
                new_hid,
                args.reason,
                args.stop_pct,
                args.target_pct,
                args.hold_days_max,
                args.hold_days_min,
                args.thesis,
                args.thesis_version,
                args.source_signal_id,
                args.mark_price,
                args.mark_ts.unwrap_or_else(Utc::now),
            );
        }
    }

    let mut prior = Map::new();
    let mut amended = Map::new();
    let mut set = |field: &str, value: Option<Value>| {
        if let Some(v) = value {
            prior.insert(field.to_string(), hold.get(field).cloned().unwrap_or(Value::Null));
            amended.insert(field.to_string(), v);
        }
    };
    set(
        "stop_pct",
        args.stop_pct.map(|v| json!(v)),
    );
    set(
        "target_pct",
        args.target_pct.map(|v| json!(v)),
    );
    set(
        "hold_days_min",
        args.hold_days_min.map(|v| json!(v)),
    );
    set(
        "hold_days_max",
        args.hold_days_max.map(|v| json!(v)),
    );
    set("thesis", args.thesis.map(|v| json!(v)));
    set(
        "thesis_version",
        args.thesis_version.map(|v| json!(v)),
    );
    set(
        "source_signal_id",
        args.source_signal_id.map(|v| json!(v)),
    );
    if args.hold_days_max.is_some() || args.max_exit_ts.is_some() {
        if let Some(ts) = args.max_exit_ts {
            set("max_exit_ts", Some(json!(ts)));
        } else if let (Some(entry_ts), Some(d)) =
            (parse_ts(hold.get("entry_ts").unwrap_or(&Value::Null)), args.hold_days_max)
        {
            set(
                "max_exit_ts",
                Some(json!(iso(entry_ts + Duration::days(d)))),
            );
        }
    }
    if amended.is_empty() {
        return Err("amend requires at least one changed field".into());
    }
    let now = iso(Utc::now());
    let mut keys: Vec<&str> = amended.keys().map(|s| s.as_str()).collect();
    keys.sort();
    let key_join = keys.join(",");
    let amendment_id = stable_id(&["AMEND", candidate_id, args.reason, &now, &key_join]);
    let hid = hold
        .get("horizon_id")
        .and_then(|v| v.as_str())
        .unwrap_or("H1_MULTI_DAY");
    let row = json!({
        "schema_version": SCHEMA_VERSION,
        "protocol_id": PROTOCOL_ID,
        "event": "AMEND",
        "status": hold.get("status").cloned().unwrap_or(json!("OPEN")),
        "amendment_id": amendment_id,
        "candidate_id": candidate_id,
        "horizon_id": hid,
        "symbol": hold.get("symbol"),
        "reason": args.reason,
        "prior": prior,
        "amended": amended,
        "live_order_authority": LIVE_ORDER_AUTHORITY,
        "amended_utc": now,
    });
    append_jsonl(&store.book_path(hid)?, &row)?;
    let mut cand_evt = row.clone();
    cand_evt
        .as_object_mut()
        .unwrap()
        .insert("event".into(), json!("AMEND_CANDIDATE"));
    append_jsonl(&store.candidates_path(), &cand_evt)?;
    let mut effective = hold.clone();
    if let Some(obj) = row.get("amended").and_then(|v| v.as_object()) {
        let map = effective.as_object_mut().unwrap();
        for (k, v) in obj {
            map.insert(k.clone(), v.clone());
        }
        map.insert("last_amendment_id".into(), json!(amendment_id));
    }
    Ok(json!({"amendment": row, "effective": effective}))
}

pub fn write_daily_observation(
    store: &Rp610Store,
    mark_ts: DateTime<Utc>,
    marks: &BTreeMap<String, f64>,
    notes: &str,
) -> Result<Value, String> {
    store.write_horizon_registry()?;
    let mut open_counts = Map::new();
    let mut closes_today = Map::new();
    for hid in ["H1_MULTI_DAY", "H2_MULTI_WEEK", "H3_LONG_TERM"] {
        let opens = open_holds_from_book(store, hid)?;
        open_counts.insert(hid.to_string(), json!(opens.len()));
        let mut closes = 0;
        for (sym, px) in marks {
            for hold in &opens {
                if hold
                    .get("symbol")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .eq_ignore_ascii_case(sym)
                {
                    if let Some(result) =
                        evaluate_open_hold(hold, mark_ts, *px, false, "")
                    {
                        append_jsonl(&store.book_path(hid)?, &result)?;
                        closes += 1;
                    }
                }
            }
        }
        closes_today.insert(hid.to_string(), json!(closes));
        for hold in open_holds_from_book(store, hid)? {
            let sym = hold
                .get("symbol")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_uppercase();
            let px = marks
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case(&sym))
                .map(|(_, v)| *v)
                .unwrap_or(0.0);
            if px <= 0.0 {
                continue;
            }
            let side = hold.get("side").and_then(|v| v.as_str()).unwrap_or("LONG");
            let entry = hold.get("entry_price").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let qty = hold.get("quantity").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let mtm = pnl_usdt(side, entry, px, qty);
            append_jsonl(
                &store.book_path(hid)?,
                &json!({
                    "event": "MARK",
                    "status": "OPEN",
                    "candidate_id": hold.get("candidate_id"),
                    "horizon_id": hid,
                    "symbol": sym,
                    "mark_ts": iso(mark_ts),
                    "mark_price": px,
                    "mtm_gross_usdt": round_n(mtm, 6),
                    "live_order_authority": LIVE_ORDER_AUTHORITY,
                }),
            )?;
        }
    }
    let summary = json!({
        "schema_version": SCHEMA_VERSION,
        "protocol_id": PROTOCOL_ID,
        "event": "DAILY_OBSERVATION",
        "observation_ts": iso(mark_ts),
        "live_order_authority": LIVE_ORDER_AUTHORITY,
        "marks": marks,
        "open_counts": open_counts,
        "closes_today": closes_today,
        "notes": notes,
        "decision_status": "OBSERVE_ONLY_NO_CONCLUSION",
    });
    append_jsonl(&store.daily_log_path(), &summary)?;
    Ok(summary)
}

pub fn bootstrap_demo_candidates(
    store: &Rp610Store,
    symbol: &str,
    entry_price: f64,
    entry_ts: DateTime<Utc>,
) -> Result<Vec<Value>, String> {
    let mut out = Vec::new();
    for (hid, thesis) in [
        ("H1_MULTI_DAY", "demo multi-day swing"),
        ("H2_MULTI_WEEK", "demo multi-week position"),
        ("H3_LONG_TERM", "demo long-term thesis"),
    ] {
        let c = build_candidate(
            symbol,
            "LONG",
            entry_price,
            entry_ts,
            hid,
            thesis,
            "BOOTSTRAP_DEMO",
            DEFAULT_NOTIONAL_USDT,
            None,
            None,
        )?;
        admit_candidate(store, &c)?;
        out.push(c);
    }
    Ok(out)
}

pub fn summarize_books(store: &Rp610Store) -> Result<Value, String> {
    store.write_horizon_registry()?;
    let mut horizons = Map::new();
    for hid in ["H1_MULTI_DAY", "H2_MULTI_WEEK", "H3_LONG_TERM"] {
        let rows = load_jsonl(&store.book_path(hid)?);
        let closes: Vec<&Value> = rows
            .iter()
            .filter(|r| r.get("event").and_then(|v| v.as_str()) == Some("CLOSE"))
            .collect();
        let net: f64 = closes
            .iter()
            .map(|r| r.get("net_pnl_usdt").and_then(|v| v.as_f64()).unwrap_or(0.0))
            .sum();
        horizons.insert(
            hid.to_string(),
            json!({
                "open": open_holds_from_book(store, hid)?.len(),
                "closed": closes.len(),
                "net_pnl_usdt_closed": round_n(net, 6),
            }),
        );
    }
    Ok(json!({"protocol_id": PROTOCOL_ID, "horizons": horizons}))
}

pub fn default_artifact_journals(workspace: &Path) -> PathBuf {
    workspace
        .join("artifacts")
        .join("rp610_crypto_multi_horizon")
        .join("journals")
}
