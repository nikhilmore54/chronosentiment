//! RP-609 Indian retail multi-horizon observation engine (CNC paper book).
//!
//! Deterministic lifecycle for H1 swing, H2 portfolio build, H3 long-term hold.
//! `live_order_authority = 0.0` always. Does not mutate RP-608.
//!
//! Rust port of `scripts/rp609_indian_retail_horizons.py` +
//! `scripts/rp609_multi_horizon_engine.py`.

use chrono::{FixedOffset, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use uuid::Uuid;

pub const SCHEMA_VERSION: &str = "rp609_multi_horizon_v1";
pub const PROTOCOL_ID: &str = "RP-609-INDIAN-RETAIL-MULTI-HORIZON-001";
pub const LIVE_ORDER_AUTHORITY: f64 = 0.0;
pub const FRICTION_MODEL_ID: &str = "IN_EQ_CNC_FRICTION_V1";

fn round_n(x: f64, n: i32) -> f64 {
    let p = 10f64.powi(n);
    (x * p).round() / p
}

fn ist_offset() -> FixedOffset {
    FixedOffset::east_opt(5 * 3600 + 30 * 60).expect("IST offset")
}

fn utc_now() -> String {
    Utc::now().to_rfc3339()
}

fn ist_today() -> String {
    Utc::now()
        .with_timezone(&ist_offset())
        .format("%Y-%m-%d")
        .to_string()
}

fn short_id(prefix: &str) -> String {
    format!("{}_{}", prefix, &Uuid::new_v4().simple().to_string()[..10])
}

pub fn normalize_symbol(symbol: &str) -> String {
    let s = symbol.trim().to_uppercase();
    if s.ends_with(".NS") {
        s
    } else {
        format!("{s}.NS")
    }
}

pub fn tax_lot_hint_from_hold_sessions(hold_sessions: Option<i64>) -> &'static str {
    match hold_sessions {
        None => "UNKNOWN",
        Some(s) if s >= 250 => "LTCG_GT_12M_HINT",
        _ => "STCG_LTE_12M_HINT",
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CncFriction {
    pub brokerage: f64,
    pub stt: f64,
    pub exchange_charges: f64,
    pub sebi_charges: f64,
    pub stamp_duty: f64,
    pub gst: f64,
    pub spread_cost: f64,
    pub total_friction: f64,
    pub effective_friction_bps: f64,
    pub friction_model_id: String,
}

pub fn estimate_cnc_friction_inr(
    buy_price: f64,
    sell_price: f64,
    quantity: f64,
    half_spread_bps: f64,
) -> CncFriction {
    let buy_notional = (buy_price * quantity).max(0.0);
    let sell_notional = (sell_price * quantity).max(0.0);
    let turnover = buy_notional + sell_notional;
    let brokerage = if turnover > 0.0 {
        (0.0003 * turnover).min(40.0)
    } else {
        0.0
    };
    let stt = 0.001 * sell_notional;
    let exchange = 0.0000297 * turnover;
    let sebi = 0.000001 * turnover;
    let stamp = 0.00015 * buy_notional;
    let gst = 0.18 * (brokerage + exchange);
    let spread_cost = turnover * (half_spread_bps / 10_000.0);
    let total = brokerage + stt + exchange + sebi + stamp + gst + spread_cost;
    let mid = if turnover > 0.0 {
        (buy_notional + sell_notional) / 2.0
    } else {
        0.0
    };
    let eff_bps = if mid > 0.0 {
        total / mid * 10_000.0
    } else {
        0.0
    };
    CncFriction {
        brokerage: round_n(brokerage, 4),
        stt: round_n(stt, 4),
        exchange_charges: round_n(exchange, 4),
        sebi_charges: round_n(sebi, 4),
        stamp_duty: round_n(stamp, 4),
        gst: round_n(gst, 4),
        spread_cost: round_n(spread_cost, 4),
        total_friction: round_n(total, 4),
        effective_friction_bps: round_n(eff_bps, 4),
        friction_model_id: FRICTION_MODEL_ID.to_string(),
    }
}

fn pnl_long(entry: f64, exit_px: f64, qty: f64) -> Value {
    let gross = (exit_px - entry) * qty;
    let fr = estimate_cnc_friction_inr(entry, exit_px, qty, 2.5);
    let net = gross - fr.total_friction;
    let ret_bps = if entry != 0.0 {
        (exit_px - entry) / entry * 10_000.0
    } else {
        0.0
    };
    json!({
        "gross_pnl_inr": round_n(gross, 2),
        "net_pnl_inr": round_n(net, 2),
        "gross_return_bps": round_n(ret_bps, 2),
        "friction": fr,
    })
}

pub fn default_horizons() -> BTreeMap<String, Value> {
    let mut m = BTreeMap::new();
    m.insert(
        "H0_INTRADAY".into(),
        json!({
            "horizon_id": "H0_INTRADAY",
            "label": "Intraday (MIS shadow — owned by RP-608)",
            "product_type": "MIS",
            "owner_protocol": "RP-608-NSE-SHADOW-001",
            "hold_sessions_min": 0,
            "hold_sessions_max": 0,
            "same_day_only": true,
            "observational_link_only": true,
            "default_review": Value::Null,
        }),
    );
    m.insert(
        "H1_SWING".into(),
        json!({
            "horizon_id": "H1_SWING",
            "label": "Swing / positional delivery",
            "product_type": "CNC",
            "owner_protocol": PROTOCOL_ID,
            "hold_sessions_min": 2,
            "hold_sessions_max": 20,
            "same_day_only": false,
            "observational_link_only": false,
            "default_review": "ON_EXIT_OR_MAX_SESSIONS",
        }),
    );
    m.insert(
        "H2_PORTFOLIO_BUILD".into(),
        json!({
            "horizon_id": "H2_PORTFOLIO_BUILD",
            "label": "Staggered portfolio build (CNC tranches)",
            "product_type": "CNC",
            "owner_protocol": PROTOCOL_ID,
            "hold_sessions_min": 5,
            "hold_sessions_max": 60,
            "same_day_only": false,
            "observational_link_only": false,
            "default_review": "ON_BUILD_COMPLETE",
            "default_caps": {
                "max_name_pct": 10.0,
                "max_sector_pct": 30.0,
                "n_tranches_default": 4
            },
        }),
    );
    m.insert(
        "H3_LONG_TERM".into(),
        json!({
            "horizon_id": "H3_LONG_TERM",
            "label": "Long-term holding (thesis-linked CNC)",
            "product_type": "CNC",
            "owner_protocol": PROTOCOL_ID,
            "hold_sessions_min": 60,
            "hold_sessions_max": Value::Null,
            "same_day_only": false,
            "observational_link_only": false,
            "default_review": "QUARTERLY",
        }),
    );
    m
}

fn write_json(path: &Path, payload: &Value) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let tmp = path.with_extension(format!(
        "{}.tmp",
        path.extension().and_then(|s| s.to_str()).unwrap_or("json")
    ));
    let text = serde_json::to_string_pretty(payload).map_err(|e| e.to_string())?;
    fs::write(&tmp, text).map_err(|e| e.to_string())?;
    fs::rename(&tmp, path).map_err(|e| e.to_string())?;
    Ok(())
}

fn read_json(path: &Path, default: Value) -> Value {
    match fs::read_to_string(path) {
        Ok(s) => serde_json::from_str(&s).unwrap_or(default),
        Err(_) => default,
    }
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
    let line = serde_json::to_string(row).map_err(|e| e.to_string())?;
    writeln!(f, "{line}").map_err(|e| e.to_string())?;
    Ok(())
}

pub fn write_horizon_registry(journals: &Path) -> Result<PathBuf, String> {
    fs::create_dir_all(journals).map_err(|e| e.to_string())?;
    let path = journals.join("RP609_horizon_registry.json");
    let now = utc_now();
    let gates = json!({
        "G-H1_SWING": "CLOSED",
        "G-H2_PORTFOLIO_BUILD": "CLOSED",
        "G-H3_LONG_TERM": "CLOSED",
        "G-LIVE": "CLOSED",
    });
    let payload = json!({
        "schema_version": SCHEMA_VERSION,
        "protocol_id": PROTOCOL_ID,
        "live_order_authority": LIVE_ORDER_AUTHORITY,
        "friction_model_id": FRICTION_MODEL_ID,
        "updated_utc": now,
        "horizons": default_horizons(),
        "promotion_gates": gates,
        "notes": [
            "H0 owned by RP-608; RP-609 must not mutate intraday daemon.",
            "All H1–H3 activity is observation-only until gates open.",
        ],
    });
    write_json(&path, &payload)?;
    write_json(
        &journals.join("RP609_promotion_gates.json"),
        &json!({
            "protocol_id": PROTOCOL_ID,
            "updated_utc": now,
            "gates": gates,
            "decision_status": "OBSERVE_ONLY_NO_CONCLUSION",
        }),
    )?;
    Ok(path)
}

fn empty_swing_book() -> Value {
    json!({
        "schema_version": SCHEMA_VERSION,
        "protocol_id": PROTOCOL_ID,
        "live_order_authority": LIVE_ORDER_AUTHORITY,
        "open": {},
        "closed": [],
    })
}

fn empty_portfolio_book() -> Value {
    json!({
        "schema_version": SCHEMA_VERSION,
        "protocol_id": PROTOCOL_ID,
        "live_order_authority": LIVE_ORDER_AUTHORITY,
        "plans": {},
    })
}

fn empty_lt_book() -> Value {
    json!({
        "schema_version": SCHEMA_VERSION,
        "protocol_id": PROTOCOL_ID,
        "live_order_authority": LIVE_ORDER_AUTHORITY,
        "open": {},
        "closed": [],
    })
}

/// Multi-horizon CNC observation engine (RP-609).
pub struct MultiHorizonEngine {
    pub journals: PathBuf,
    pub nse_dir: PathBuf,
    pub swing: Value,
    pub portfolio: Value,
    pub long_term: Value,
}

impl MultiHorizonEngine {
    pub fn new(journals: PathBuf, nse_dir: PathBuf) -> Result<Self, String> {
        write_horizon_registry(&journals)?;
        fs::create_dir_all(&journals).map_err(|e| e.to_string())?;
        let swing = read_json(&journals.join("RP609_swing_book.json"), empty_swing_book());
        let portfolio = read_json(
            &journals.join("RP609_portfolio_book.json"),
            empty_portfolio_book(),
        );
        let long_term = read_json(&journals.join("RP609_long_term_book.json"), empty_lt_book());
        Ok(Self {
            journals,
            nse_dir,
            swing,
            portfolio,
            long_term,
        })
    }

    pub fn persist(&self) -> Result<(), String> {
        write_json(&self.journals.join("RP609_swing_book.json"), &self.swing)?;
        write_json(
            &self.journals.join("RP609_portfolio_book.json"),
            &self.portfolio,
        )?;
        write_json(
            &self.journals.join("RP609_long_term_book.json"),
            &self.long_term,
        )?;
        Ok(())
    }

    fn candidates_path(&self) -> PathBuf {
        self.journals.join("RP609_candidates.jsonl")
    }
    fn swing_path(&self) -> PathBuf {
        self.journals.join("RP609_swing_trades.jsonl")
    }
    fn portfolio_path(&self) -> PathBuf {
        self.journals.join("RP609_portfolio_builds.jsonl")
    }
    fn long_term_path(&self) -> PathBuf {
        self.journals.join("RP609_long_term_holds.jsonl")
    }
    fn daily_log_path(&self) -> PathBuf {
        self.journals.join("RP609_daily_observation_log.jsonl")
    }

    pub fn register_swing_candidate(
        &mut self,
        symbol: &str,
        thesis_tag: &str,
        reference_price: f64,
        invalidation_price: f64,
        target_price: f64,
        quantity: f64,
        max_sessions: i64,
        sector: Option<&str>,
        notes: Option<&str>,
        auto_open: bool,
        session_date: Option<&str>,
    ) -> Result<Value, String> {
        let sym = normalize_symbol(symbol);
        let h = default_horizons()
            .get("H1_SWING")
            .cloned()
            .unwrap_or(json!({}));
        let hmin = h.get("hold_sessions_min").and_then(|v| v.as_i64()).unwrap_or(2);
        let hmax = h.get("hold_sessions_max").and_then(|v| v.as_i64()).unwrap_or(20);
        let max_sessions = max_sessions.max(hmin).min(hmax);
        let cand_id = short_id("h1cand");
        let trade_id = short_id("h1");
        let sess = session_date.map(|s| s.to_string()).unwrap_or_else(ist_today);
        let candidate = json!({
            "schema_version": SCHEMA_VERSION,
            "protocol_id": PROTOCOL_ID,
            "event_type": "CANDIDATE_REGISTERED",
            "candidate_id": cand_id,
            "trade_id": trade_id,
            "horizon_id": "H1_SWING",
            "product_type": "CNC",
            "live_order_authority": LIVE_ORDER_AUTHORITY,
            "symbol": sym,
            "tradingsymbol": sym.trim_end_matches(".NS"),
            "sector": sector,
            "thesis_tag": thesis_tag,
            "reference_price": reference_price,
            "invalidation_price": invalidation_price,
            "target_price": target_price,
            "quantity": quantity,
            "max_sessions": max_sessions,
            "notes": notes,
            "status": "CANDIDATE",
            "recorded_utc": utc_now(),
            "session_date": sess,
        });
        append_jsonl(&self.candidates_path(), &candidate)?;
        if !auto_open {
            self.persist()?;
            return Ok(candidate);
        }
        self.open_swing_from_candidate(&candidate, Some(sess.as_str()))
    }

    pub fn open_swing_from_candidate(
        &mut self,
        candidate: &Value,
        session_date: Option<&str>,
    ) -> Result<Value, String> {
        let sess = session_date
            .map(|s| s.to_string())
            .or_else(|| {
                candidate
                    .get("session_date")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
            })
            .unwrap_or_else(ist_today);
        let entry = candidate["reference_price"].as_f64().unwrap_or(0.0);
        let qty = candidate["quantity"].as_f64().unwrap_or(0.0);
        let trade_id = candidate
            .get("trade_id")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| short_id("h1"));
        let trade = json!({
            "schema_version": SCHEMA_VERSION,
            "protocol_id": PROTOCOL_ID,
            "event_type": "SWING_OPEN",
            "live_order_authority": LIVE_ORDER_AUTHORITY,
            "horizon_id": "H1_SWING",
            "product_type": "CNC",
            "trade_id": trade_id,
            "candidate_id": candidate.get("candidate_id"),
            "symbol": candidate["symbol"],
            "tradingsymbol": candidate.get("tradingsymbol"),
            "sector": candidate.get("sector"),
            "thesis_tag": candidate.get("thesis_tag"),
            "side": "LONG",
            "entry_price": entry,
            "quantity": qty,
            "invalidation_price": candidate["invalidation_price"],
            "target_price": candidate["target_price"],
            "max_sessions": candidate["max_sessions"],
            "sessions_held": 0,
            "entry_session_date": sess,
            "last_mark_session_date": sess,
            "last_mark_price": entry,
            "mfe_price": entry,
            "mae_price": entry,
            "mfe_bps": 0.0,
            "mae_bps": 0.0,
            "status": "OPEN",
            "marks": [{"session_date": sess, "price": entry, "event": "ENTRY"}],
            "friction_model_id": FRICTION_MODEL_ID,
            "recorded_utc": utc_now(),
        });
        let tid = trade["trade_id"].as_str().unwrap().to_string();
        self.swing
            .as_object_mut()
            .unwrap()
            .entry("open")
            .or_insert_with(|| json!({}))
            .as_object_mut()
            .unwrap()
            .insert(tid.clone(), trade.clone());
        let mut open_evt = trade.clone();
        open_evt
            .as_object_mut()
            .unwrap()
            .insert("event_type".into(), json!("SWING_OPEN"));
        append_jsonl(&self.swing_path(), &open_evt)?;
        self.persist()?;
        Ok(trade)
    }

    fn update_excursions(pos: &mut Map<String, Value>, px: f64) {
        let ep = pos.get("entry_price").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let mfe_p = pos
            .get("mfe_price")
            .and_then(|v| v.as_f64())
            .unwrap_or(ep)
            .max(px);
        let mae_p = pos
            .get("mae_price")
            .and_then(|v| v.as_f64())
            .unwrap_or(ep)
            .min(px);
        pos.insert("mfe_price".into(), json!(mfe_p));
        pos.insert("mae_price".into(), json!(mae_p));
        pos.insert(
            "mfe_bps".into(),
            json!(if ep != 0.0 {
                round_n((mfe_p - ep) / ep * 10_000.0, 2)
            } else {
                0.0
            }),
        );
        pos.insert(
            "mae_bps".into(),
            json!(if ep != 0.0 {
                round_n((ep - mae_p) / ep * 10_000.0, 2)
            } else {
                0.0
            }),
        );
    }

    pub fn mark_swing(
        &mut self,
        trade_id: &str,
        price: f64,
        session_date: Option<&str>,
    ) -> Result<Value, String> {
        let sess = session_date.map(|s| s.to_string()).unwrap_or_else(ist_today);
        let (exit_reason, held, mfe_bps, mae_bps, out) = {
            let open = self
                .swing
                .get_mut("open")
                .and_then(|v| v.as_object_mut())
                .ok_or_else(|| format!("swing not open: {trade_id}"))?;
            let pos_val = open
                .get_mut(trade_id)
                .ok_or_else(|| format!("swing not open: {trade_id}"))?;
            let pos = pos_val
                .as_object_mut()
                .ok_or_else(|| format!("swing not open: {trade_id}"))?;

            if let Some(last) = pos.get("last_mark_session_date").and_then(|v| v.as_str()) {
                if sess.as_str() > last {
                    let held = pos.get("sessions_held").and_then(|v| v.as_i64()).unwrap_or(0) + 1;
                    pos.insert("sessions_held".into(), json!(held));
                }
            }
            pos.insert("last_mark_session_date".into(), json!(sess.clone()));
            pos.insert("last_mark_price".into(), json!(price));
            Self::update_excursions(pos, price);
            let marks = pos
                .entry("marks".to_string())
                .or_insert_with(|| json!([]))
                .as_array_mut()
                .unwrap();
            marks.push(json!({"session_date": sess, "price": price, "event": "MARK"}));

            let inv = pos
                .get("invalidation_price")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            let tgt = pos
                .get("target_price")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            let held = pos.get("sessions_held").and_then(|v| v.as_i64()).unwrap_or(0);
            let max_s = pos.get("max_sessions").and_then(|v| v.as_i64()).unwrap_or(0);
            let exit_reason = if price <= inv {
                Some("INVALIDATION")
            } else if price >= tgt {
                Some("TARGET")
            } else if held >= max_s {
                Some("MAX_SESSIONS")
            } else {
                None
            };
            (
                exit_reason,
                held,
                pos.get("mfe_bps").cloned(),
                pos.get("mae_bps").cloned(),
                pos_val.clone(),
            )
        };

        if let Some(reason) = exit_reason {
            return self.close_swing(trade_id, price, reason, Some(sess.as_str()));
        }

        let mark_evt = json!({
            "event_type": "SWING_MARK",
            "trade_id": trade_id,
            "session_date": sess,
            "price": price,
            "sessions_held": held,
            "mfe_bps": mfe_bps,
            "mae_bps": mae_bps,
            "live_order_authority": LIVE_ORDER_AUTHORITY,
            "recorded_utc": utc_now(),
        });
        append_jsonl(&self.swing_path(), &mark_evt)?;
        self.persist()?;
        Ok(out)
    }

    pub fn close_swing(
        &mut self,
        trade_id: &str,
        price: f64,
        reason: &str,
        session_date: Option<&str>,
    ) -> Result<Value, String> {
        let sess = session_date.map(|s| s.to_string()).unwrap_or_else(ist_today);
        let open = self
            .swing
            .get_mut("open")
            .and_then(|v| v.as_object_mut())
            .ok_or_else(|| format!("swing not open: {trade_id}"))?;
        let mut pos = open
            .remove(trade_id)
            .ok_or_else(|| format!("swing not open: {trade_id}"))?;
        {
            let map = pos.as_object_mut().unwrap();
            Self::update_excursions(map, price);
            let entry = map.get("entry_price").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let qty = map.get("quantity").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let held = map.get("sessions_held").and_then(|v| v.as_i64()).unwrap_or(0);
            let pnl = pnl_long(entry, price, qty);
            if let Some(obj) = pnl.as_object() {
                for (k, v) in obj {
                    map.insert(k.clone(), v.clone());
                }
            }
            map.insert("event_type".into(), json!("SWING_CLOSE"));
            map.insert("status".into(), json!("CLOSED"));
            map.insert("exit_price".into(), json!(price));
            map.insert("exit_session_date".into(), json!(sess));
            map.insert("exit_reason".into(), json!(reason));
            map.insert(
                "tax_lot_hint".into(),
                json!(tax_lot_hint_from_hold_sessions(Some(held))),
            );
            map.insert("closed_utc".into(), json!(utc_now()));
        }
        self.swing
            .as_object_mut()
            .unwrap()
            .entry("closed")
            .or_insert_with(|| json!([]))
            .as_array_mut()
            .unwrap()
            .push(pos.clone());
        append_jsonl(&self.swing_path(), &pos)?;
        self.persist()?;
        Ok(pos)
    }

    pub fn register_portfolio_plan(
        &mut self,
        plan_id: &str,
        total_capital_inr: f64,
        legs: &[Value],
        n_tranches: i64,
        max_name_pct: f64,
        max_sector_pct: f64,
        session_date: Option<&str>,
    ) -> Result<Value, String> {
        if self
            .portfolio
            .get("plans")
            .and_then(|p| p.get(plan_id))
            .is_some()
        {
            return Err(format!("plan already exists: {plan_id}"));
        }
        let sess = session_date.map(|s| s.to_string()).unwrap_or_else(ist_today);
        for leg in legs {
            let w = leg
                .get("target_weight_pct")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            if w > max_name_pct + 1e-9 {
                return Err(format!(
                    "{} weight {}% exceeds max_name_pct {}%",
                    leg.get("symbol").and_then(|v| v.as_str()).unwrap_or("?"),
                    w,
                    max_name_pct
                ));
            }
        }
        let mut by_sector: BTreeMap<String, f64> = BTreeMap::new();
        for leg in legs {
            let sec = leg
                .get("sector")
                .and_then(|v| v.as_str())
                .unwrap_or("UNKNOWN")
                .to_string();
            let w = leg
                .get("target_weight_pct")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            *by_sector.entry(sec).or_insert(0.0) += w;
        }
        for (sec, w) in &by_sector {
            if *w > max_sector_pct + 1e-9 {
                return Err(format!(
                    "sector {sec} weight {w}% exceeds max_sector_pct {max_sector_pct}%"
                ));
            }
        }
        let norm_legs: Vec<Value> = legs
            .iter()
            .map(|leg| {
                json!({
                    "symbol": normalize_symbol(leg.get("symbol").and_then(|v| v.as_str()).unwrap_or("")),
                    "sector": leg.get("sector"),
                    "target_weight_pct": leg.get("target_weight_pct").and_then(|v| v.as_f64()).unwrap_or(0.0),
                    "qty_filled": 0.0,
                    "notional_filled": 0.0,
                    "avg_price": Value::Null,
                })
            })
            .collect();
        let plan = json!({
            "schema_version": SCHEMA_VERSION,
            "protocol_id": PROTOCOL_ID,
            "event_type": "PORTFOLIO_BUILD_PLAN",
            "live_order_authority": LIVE_ORDER_AUTHORITY,
            "horizon_id": "H2_PORTFOLIO_BUILD",
            "product_type": "CNC",
            "plan_id": plan_id,
            "status": "ACTIVE",
            "total_capital_inr": total_capital_inr,
            "n_tranches": n_tranches,
            "tranches_filled": 0,
            "caps": {"max_name_pct": max_name_pct, "max_sector_pct": max_sector_pct},
            "legs": norm_legs,
            "fills": [],
            "created_session_date": sess,
            "last_tranche_session_date": Value::Null,
            "recorded_utc": utc_now(),
        });
        self.portfolio
            .as_object_mut()
            .unwrap()
            .entry("plans")
            .or_insert_with(|| json!({}))
            .as_object_mut()
            .unwrap()
            .insert(plan_id.to_string(), plan.clone());
        append_jsonl(&self.portfolio_path(), &plan)?;
        self.persist()?;
        Ok(plan)
    }

    pub fn advance_portfolio_tranche(
        &mut self,
        plan_id: &str,
        prices: &BTreeMap<String, f64>,
        session_date: Option<&str>,
    ) -> Result<Value, String> {
        let sess = session_date.map(|s| s.to_string()).unwrap_or_else(ist_today);
        let portfolio_path = self.portfolio_path();
        let mut plan = self
            .portfolio
            .get("plans")
            .and_then(|p| p.get(plan_id))
            .cloned()
            .ok_or_else(|| format!("unknown plan: {plan_id}"))?;
        if plan.get("status").and_then(|v| v.as_str()) != Some("ACTIVE") {
            return Ok(plan);
        }
        let filled = plan
            .get("tranches_filled")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);
        let n = plan.get("n_tranches").and_then(|v| v.as_i64()).unwrap_or(0);
        if filled >= n {
            plan.as_object_mut()
                .unwrap()
                .insert("status".into(), json!("BUILT"));
            self.portfolio
                .as_object_mut()
                .unwrap()
                .get_mut("plans")
                .unwrap()
                .as_object_mut()
                .unwrap()
                .insert(plan_id.to_string(), plan.clone());
            self.persist()?;
            return Ok(plan);
        }
        if plan
            .get("last_tranche_session_date")
            .and_then(|v| v.as_str())
            == Some(sess.as_str())
        {
            return Ok(plan);
        }

        let tranche_idx = filled + 1;
        let capital = plan
            .get("total_capital_inr")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0)
            / n as f64;
        let mut fills = Vec::new();
        let map = plan.as_object_mut().unwrap();
        let mut updated_legs = Vec::new();
        {
            let legs = map.get("legs").and_then(|v| v.as_array()).cloned().unwrap_or_default();
            for mut leg in legs {
                let sym = leg
                    .get("symbol")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let Some(&px) = prices.get(&sym) else {
                    updated_legs.push(leg);
                    continue;
                };
                if px <= 0.0 {
                    updated_legs.push(leg);
                    continue;
                }
                let w = leg
                    .get("target_weight_pct")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0);
                let notional = capital * (w / 100.0);
                let qty = if notional >= px {
                    (notional / px).floor().max(1.0)
                } else {
                    0.0
                };
                if qty <= 0.0 {
                    updated_legs.push(leg);
                    continue;
                }
                let spent = qty * px;
                let prev_n = leg.get("qty_filled").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let prev_notional = leg
                    .get("notional_filled")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0);
                let new_n = prev_n + qty;
                let new_notional = prev_notional + spent;
                let leg_map = leg.as_object_mut().unwrap();
                leg_map.insert("qty_filled".into(), json!(new_n));
                leg_map.insert("notional_filled".into(), json!(round_n(new_notional, 2)));
                leg_map.insert(
                    "avg_price".into(),
                    if new_n > 0.0 {
                        json!(round_n(new_notional / new_n, 4))
                    } else {
                        Value::Null
                    },
                );
                let fill = json!({
                    "tranche": tranche_idx,
                    "session_date": sess,
                    "symbol": sym,
                    "price": px,
                    "quantity": qty,
                    "notional": round_n(spent, 2),
                });
                fills.push(fill.clone());
                updated_legs.push(leg);
                map.entry("fills".to_string())
                    .or_insert_with(|| json!([]))
                    .as_array_mut()
                    .unwrap()
                    .push(fill);
            }
        }
        map.insert("legs".into(), json!(updated_legs));
        map.insert("tranches_filled".into(), json!(tranche_idx));
        map.insert("last_tranche_session_date".into(), json!(sess));
        if tranche_idx >= n {
            map.insert("status".into(), json!("BUILT"));
        }
        let status = plan
            .get("status")
            .cloned()
            .unwrap_or(json!("ACTIVE"));
        let event = json!({
            "event_type": "PORTFOLIO_TRANCHE_FILL",
            "plan_id": plan_id,
            "tranche": tranche_idx,
            "session_date": sess,
            "fills": fills,
            "status": status,
            "live_order_authority": LIVE_ORDER_AUTHORITY,
            "recorded_utc": utc_now(),
        });
        append_jsonl(&portfolio_path, &event)?;
        self.portfolio
            .as_object_mut()
            .unwrap()
            .get_mut("plans")
            .unwrap()
            .as_object_mut()
            .unwrap()
            .insert(plan_id.to_string(), plan.clone());
        self.persist()?;
        Ok(plan)
    }

    pub fn open_long_term(
        &mut self,
        symbol: &str,
        thesis: &str,
        thesis_version: &str,
        entry_price: f64,
        quantity: f64,
        sector: Option<&str>,
        review_cadence_sessions: i64,
        session_date: Option<&str>,
    ) -> Result<Value, String> {
        let sym = normalize_symbol(symbol);
        let sess = session_date.map(|s| s.to_string()).unwrap_or_else(ist_today);
        let hold_id = short_id("h3");
        let row = json!({
            "schema_version": SCHEMA_VERSION,
            "protocol_id": PROTOCOL_ID,
            "event_type": "LONG_TERM_HOLD_OPEN",
            "live_order_authority": LIVE_ORDER_AUTHORITY,
            "horizon_id": "H3_LONG_TERM",
            "product_type": "CNC",
            "hold_id": hold_id,
            "symbol": sym,
            "tradingsymbol": sym.trim_end_matches(".NS"),
            "sector": sector,
            "thesis": thesis,
            "thesis_version": thesis_version,
            "side": "LONG",
            "entry_price": entry_price,
            "quantity": quantity,
            "entry_session_date": sess,
            "sessions_held": 0,
            "last_mark_session_date": sess,
            "last_mark_price": entry_price,
            "mfe_bps": 0.0,
            "mae_bps": 0.0,
            "review_cadence_sessions": review_cadence_sessions,
            "next_review_session_count": review_cadence_sessions,
            "status": "HOLDING",
            "tax_lot_hint": "STCG_LTE_12M_HINT",
            "recorded_utc": utc_now(),
        });
        let hid = row["hold_id"].as_str().unwrap().to_string();
        self.long_term
            .as_object_mut()
            .unwrap()
            .entry("open")
            .or_insert_with(|| json!({}))
            .as_object_mut()
            .unwrap()
            .insert(hid, row.clone());
        append_jsonl(&self.long_term_path(), &row)?;
        self.persist()?;
        Ok(row)
    }

    pub fn mark_long_term(
        &mut self,
        hold_id: &str,
        price: f64,
        session_date: Option<&str>,
    ) -> Result<Value, String> {
        let sess = session_date.map(|s| s.to_string()).unwrap_or_else(ist_today);
        let log_path = self.long_term_path();
        let mut pos = self
            .long_term
            .get("open")
            .and_then(|o| o.get(hold_id))
            .cloned()
            .ok_or_else(|| format!("long-term not open: {hold_id}"))?;
        {
            let map = pos.as_object_mut().unwrap();
            if let Some(last) = map.get("last_mark_session_date").and_then(|v| v.as_str()) {
                if sess.as_str() > last {
                    let held = map.get("sessions_held").and_then(|v| v.as_i64()).unwrap_or(0) + 1;
                    map.insert("sessions_held".into(), json!(held));
                }
            }
            map.insert("last_mark_session_date".into(), json!(sess.clone()));
            map.insert("last_mark_price".into(), json!(price));
            let ep = map.get("entry_price").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let fav = if ep != 0.0 {
                (price - ep) / ep * 10_000.0
            } else {
                0.0
            };
            let adv = if ep != 0.0 {
                (ep - price) / ep * 10_000.0
            } else {
                0.0
            };
            let mfe = map.get("mfe_bps").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let mae = map.get("mae_bps").and_then(|v| v.as_f64()).unwrap_or(0.0);
            map.insert("mfe_bps".into(), json!(round_n(mfe.max(fav), 2)));
            map.insert("mae_bps".into(), json!(round_n(mae.max(adv), 2)));
            let held = map.get("sessions_held").and_then(|v| v.as_i64()).unwrap_or(0);
            map.insert(
                "tax_lot_hint".into(),
                json!(tax_lot_hint_from_hold_sessions(Some(held))),
            );
            let qty = map.get("quantity").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let unreal = pnl_long(ep, price, qty);
            map.insert(
                "unrealized_net_pnl_inr".into(),
                unreal.get("net_pnl_inr").cloned().unwrap_or(json!(0.0)),
            );
            map.insert(
                "unrealized_gross_pnl_inr".into(),
                unreal.get("gross_pnl_inr").cloned().unwrap_or(json!(0.0)),
            );
            let next_rev = map
                .get("next_review_session_count")
                .and_then(|v| v.as_i64())
                .unwrap_or(60);
            if held >= next_rev {
                map.insert("status".into(), json!("REVIEW_DUE"));
            }
        }
        let held = pos.get("sessions_held").and_then(|v| v.as_i64()).unwrap_or(0);
        let evt = json!({
            "event_type": "LONG_TERM_MARK",
            "hold_id": hold_id,
            "session_date": sess,
            "price": price,
            "sessions_held": held,
            "status": pos.get("status"),
            "unrealized_net_pnl_inr": pos.get("unrealized_net_pnl_inr"),
            "live_order_authority": LIVE_ORDER_AUTHORITY,
            "recorded_utc": utc_now(),
        });
        append_jsonl(&log_path, &evt)?;
        self.long_term
            .as_object_mut()
            .unwrap()
            .get_mut("open")
            .unwrap()
            .as_object_mut()
            .unwrap()
            .insert(hold_id.to_string(), pos.clone());
        self.persist()?;
        Ok(pos)
    }

    pub fn complete_long_term_review(
        &mut self,
        hold_id: &str,
        decision: &str,
    ) -> Result<Value, String> {
        if decision == "THESIS_INVALIDATED" {
            let open = self
                .long_term
                .get_mut("open")
                .and_then(|v| v.as_object_mut())
                .ok_or_else(|| format!("long-term not open: {hold_id}"))?;
            let mut pos = open
                .remove(hold_id)
                .ok_or_else(|| format!("long-term not open: {hold_id}"))?;
            {
                let map = pos.as_object_mut().unwrap();
                let px = map
                    .get("last_mark_price")
                    .and_then(|v| v.as_f64())
                    .or_else(|| map.get("entry_price").and_then(|v| v.as_f64()))
                    .unwrap_or(0.0);
                let entry = map.get("entry_price").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let qty = map.get("quantity").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let pnl = pnl_long(entry, px, qty);
                if let Some(obj) = pnl.as_object() {
                    for (k, v) in obj {
                        map.insert(k.clone(), v.clone());
                    }
                }
                map.insert("event_type".into(), json!("LONG_TERM_CLOSE"));
                map.insert("status".into(), json!("CLOSED"));
                map.insert("exit_reason".into(), json!("THESIS_INVALIDATED"));
                map.insert("exit_price".into(), json!(px));
                map.insert("closed_utc".into(), json!(utc_now()));
            }
            self.long_term
                .as_object_mut()
                .unwrap()
                .entry("closed")
                .or_insert_with(|| json!([]))
                .as_array_mut()
                .unwrap()
                .push(pos.clone());
            append_jsonl(&self.long_term_path(), &pos)?;
            self.persist()?;
            return Ok(pos);
        }
        let log_path = self.long_term_path();
        let mut pos = self
            .long_term
            .get("open")
            .and_then(|o| o.get(hold_id))
            .cloned()
            .ok_or_else(|| format!("long-term not open: {hold_id}"))?;
        {
            let map = pos.as_object_mut().unwrap();
            map.insert("status".into(), json!("HOLDING"));
            let held = map.get("sessions_held").and_then(|v| v.as_i64()).unwrap_or(0);
            let cadence = map
                .get("review_cadence_sessions")
                .and_then(|v| v.as_i64())
                .unwrap_or(60);
            map.insert(
                "next_review_session_count".into(),
                json!(held + cadence),
            );
        }
        let evt = json!({
            "event_type": "LONG_TERM_REVIEW",
            "hold_id": hold_id,
            "decision": decision,
            "next_review_session_count": pos.get("next_review_session_count"),
            "live_order_authority": LIVE_ORDER_AUTHORITY,
            "recorded_utc": utc_now(),
        });
        append_jsonl(&log_path, &evt)?;
        self.long_term
            .as_object_mut()
            .unwrap()
            .get_mut("open")
            .unwrap()
            .as_object_mut()
            .unwrap()
            .insert(hold_id.to_string(), pos.clone());
        self.persist()?;
        Ok(pos)
    }

    pub fn run_eod(
        &mut self,
        session_date: Option<&str>,
        prices: Option<&BTreeMap<String, f64>>,
    ) -> Result<Value, String> {
        let sess = session_date.map(|s| s.to_string()).unwrap_or_else(ist_today);
        let pxmap = match prices {
            Some(p) => p.clone(),
            None => load_price_map(&self.nse_dir),
        };
        let mut swing_marked = 0;
        let open_ids: Vec<String> = self
            .swing
            .get("open")
            .and_then(|v| v.as_object())
            .map(|o| o.keys().cloned().collect())
            .unwrap_or_default();
        for tid in open_ids {
            let sym = self.swing["open"][&tid]["symbol"]
                .as_str()
                .unwrap_or("")
                .to_string();
            if let Some(&px) = pxmap.get(&sym) {
                self.mark_swing(&tid, px, Some(sess.as_str()))?;
                swing_marked += 1;
            }
        }
        let plan_ids: Vec<String> = self
            .portfolio
            .get("plans")
            .and_then(|v| v.as_object())
            .map(|o| o.keys().cloned().collect())
            .unwrap_or_default();
        for pid in plan_ids {
            let status = self.portfolio["plans"][&pid]["status"]
                .as_str()
                .unwrap_or("")
                .to_string();
            if status == "ACTIVE" {
                self.advance_portfolio_tranche(&pid, &pxmap, Some(sess.as_str()))?;
            }
        }
        let hold_ids: Vec<String> = self
            .long_term
            .get("open")
            .and_then(|v| v.as_object())
            .map(|o| o.keys().cloned().collect())
            .unwrap_or_default();
        for hid in hold_ids {
            let sym = self.long_term["open"][&hid]["symbol"]
                .as_str()
                .unwrap_or("")
                .to_string();
            if let Some(&px) = pxmap.get(&sym) {
                self.mark_long_term(&hid, px, Some(sess.as_str()))?;
            }
        }
        let summary = json!({
            "schema_version": SCHEMA_VERSION,
            "protocol_id": PROTOCOL_ID,
            "event_type": "EOD_SNAPSHOT",
            "live_order_authority": LIVE_ORDER_AUTHORITY,
            "decision_status": "OBSERVE_ONLY_NO_CONCLUSION",
            "session_date": sess,
            "recorded_utc": utc_now(),
            "n_prices": pxmap.len(),
            "swing_open": self.swing.get("open").and_then(|v| v.as_object()).map(|o| o.len()).unwrap_or(0),
            "swing_closed_total": self.swing.get("closed").and_then(|v| v.as_array()).map(|a| a.len()).unwrap_or(0),
            "swing_marked": swing_marked,
            "portfolio_active": self.portfolio.get("plans").and_then(|v| v.as_object()).map(|o| o.values().filter(|p| p.get("status").and_then(|s| s.as_str()) == Some("ACTIVE")).count()).unwrap_or(0),
            "portfolio_built": self.portfolio.get("plans").and_then(|v| v.as_object()).map(|o| o.values().filter(|p| p.get("status").and_then(|s| s.as_str()) == Some("BUILT")).count()).unwrap_or(0),
            "long_term_open": self.long_term.get("open").and_then(|v| v.as_object()).map(|o| o.len()).unwrap_or(0),
            "long_term_review_due": self.long_term.get("open").and_then(|v| v.as_object()).map(|o| o.values().filter(|p| p.get("status").and_then(|s| s.as_str()) == Some("REVIEW_DUE")).count()).unwrap_or(0),
            "source_nse_dir": self.nse_dir.display().to_string(),
        });
        // rewrite daily log excluding same session
        let log_path = self.daily_log_path();
        let mut existing = Vec::new();
        if let Ok(text) = fs::read_to_string(&log_path) {
            for line in text.lines() {
                if line.trim().is_empty() {
                    continue;
                }
                if let Ok(row) = serde_json::from_str::<Value>(line) {
                    let sd = row
                        .get("session_date")
                        .or_else(|| row.get("observation_date_ist"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    if sd != sess {
                        existing.push(row);
                    }
                }
            }
        }
        existing.push(summary.clone());
        if let Some(parent) = log_path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let mut f = fs::File::create(&log_path).map_err(|e| e.to_string())?;
        for row in existing {
            writeln!(f, "{}", serde_json::to_string(&row).unwrap()).map_err(|e| e.to_string())?;
        }
        self.persist()?;
        Ok(summary)
    }

    pub fn report(&self) -> Value {
        let closed = self
            .swing
            .get("closed")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        let nets: f64 = closed
            .iter()
            .map(|t| t.get("net_pnl_inr").and_then(|v| v.as_f64()).unwrap_or(0.0))
            .sum();
        let open = self
            .swing
            .get("open")
            .and_then(|v| v.as_object())
            .cloned()
            .unwrap_or_default();
        let plans = self
            .portfolio
            .get("plans")
            .and_then(|v| v.as_object())
            .cloned()
            .unwrap_or_default();
        let mut plan_summary = Map::new();
        for (pid, p) in &plans {
            plan_summary.insert(
                pid.clone(),
                json!({
                    "status": p.get("status"),
                    "tranches_filled": p.get("tranches_filled"),
                    "n_tranches": p.get("n_tranches"),
                }),
            );
        }
        let lt_open = self
            .long_term
            .get("open")
            .and_then(|v| v.as_object())
            .cloned()
            .unwrap_or_default();
        let review_due: Vec<String> = lt_open
            .iter()
            .filter(|(_, p)| p.get("status").and_then(|s| s.as_str()) == Some("REVIEW_DUE"))
            .map(|(k, _)| k.clone())
            .collect();
        json!({
            "protocol_id": PROTOCOL_ID,
            "live_order_authority": LIVE_ORDER_AUTHORITY,
            "swing": {
                "open": open.len(),
                "closed": closed.len(),
                "closed_net_pnl_inr": round_n(nets, 2),
                "open_ids": open.keys().cloned().collect::<Vec<_>>(),
            },
            "portfolio": {"plans": plan_summary},
            "long_term": {
                "open": lt_open.len(),
                "review_due": review_due,
            },
        })
    }
}

pub fn load_price_map(nse_dir: &Path) -> BTreeMap<String, f64> {
    let mut prices = BTreeMap::new();
    let bh = nse_dir.join("rp608_nse_bar_history.json");
    if let Ok(text) = fs::read_to_string(&bh) {
        if let Ok(raw) = serde_json::from_str::<Value>(&text) {
            if let Some(obj) = raw.as_object() {
                for (sym, bars) in obj {
                    if let Some(arr) = bars.as_array() {
                        if let Some(last) = arr.last() {
                            if let Some(c) = last.get("close").and_then(|v| v.as_f64()) {
                                prices.insert(normalize_symbol(sym), c);
                            }
                        }
                    }
                }
            }
        }
    }
    let ev = nse_dir.join("rp608_nse_latest_evaluations.json");
    if let Ok(text) = fs::read_to_string(&ev) {
        if let Ok(payload) = serde_json::from_str::<Value>(&text) {
            let evaluations = payload
                .get("evaluations")
                .cloned()
                .unwrap_or(payload);
            if let Some(obj) = evaluations.as_object() {
                for (sym, row) in obj {
                    if let Some(px) = row
                        .get("reference_price")
                        .or_else(|| row.get("entry_price"))
                        .and_then(|v| v.as_f64())
                    {
                        prices
                            .entry(normalize_symbol(sym))
                            .or_insert(px);
                    }
                }
            }
        }
    }
    prices
}

pub fn default_artifact_journals(workspace: &Path) -> PathBuf {
    workspace
        .join("artifacts")
        .join("rp609_indian_retail_multi_horizon")
        .join("journals")
}
