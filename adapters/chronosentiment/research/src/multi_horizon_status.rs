//! Aggregate RP-608/609/610 horizon recommendation surfaces for the live dashboard.
//!
//! Observation-only. `live_order_authority = 0.0`.
//! Rust port of `scripts/rp608_multi_horizon_status.py`.

use chrono::Utc;
use serde_json::{json, Map, Value};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use super::rp609_multi_horizon::{
    default_artifact_journals as rp609_journals, default_horizons as rp609_horizons,
    write_horizon_registry as write_rp609_registry, MultiHorizonEngine, LIVE_ORDER_AUTHORITY,
    PROTOCOL_ID as RP609_PROTOCOL,
};
use super::rp610_crypto_multi_horizon::{
    default_artifact_journals as rp610_journals, default_horizons as rp610_horizons,
    open_holds_from_book, summarize_books, Rp610Store, PROTOCOL_ID as RP610_PROTOCOL,
};

const TERM_NSE: &[(&str, &str)] = &[
    (
        "H0_INTRADAY",
        "15m bars · 60m target / 180m max · session square-off",
    ),
    ("H1_SWING", "2–20 sessions (CNC swing)"),
    ("H2_PORTFOLIO_BUILD", "5–60 sessions (staggered CNC build)"),
    ("H3_LONG_TERM", "≥60 sessions / ≥90d thesis hold"),
];

const TERM_CRYPTO: &[(&str, &str)] = &[
    ("H0_INTRADAY_60M", "≤60 minutes (trade-plan overlay)"),
    ("H1_MULTI_DAY", "2–14 calendar days"),
    ("H2_MULTI_WEEK", "14–56 calendar days"),
    ("H3_LONG_TERM", "≥90 calendar days (thesis hold)"),
];

fn class_nse(hid: &str) -> &'static str {
    match hid {
        "H0_INTRADAY" => "INTRADAY",
        "H1_SWING" => "MULTI_DAY",
        "H2_PORTFOLIO_BUILD" => "MULTI_WEEK",
        "H3_LONG_TERM" => "LONG_TERM",
        _ => "INTRADAY",
    }
}

fn class_crypto(hid: &str) -> &'static str {
    match hid {
        "H0_INTRADAY_60M" => "INTRADAY",
        "H1_MULTI_DAY" => "MULTI_DAY",
        "H2_MULTI_WEEK" => "MULTI_WEEK",
        "H3_LONG_TERM" => "LONG_TERM",
        _ => "INTRADAY",
    }
}

fn label(cls: &str) -> String {
    match cls {
        "INTRADAY" => "Intraday".into(),
        "MULTI_DAY" => "Multi-day".into(),
        "MULTI_WEEK" => "Multi-week".into(),
        "LONG_TERM" => "Long-term".into(),
        other => other.to_string(),
    }
}

fn term_lookup(table: &[(&str, &str)], hid: &str) -> String {
    table
        .iter()
        .find(|(k, _)| *k == hid)
        .map(|(_, v)| (*v).to_string())
        .unwrap_or_else(|| hid.to_string())
}

fn iso_now() -> String {
    Utc::now().to_rfc3339().replace("+00:00", "Z")
}

fn count_by_class(recs: &[Value]) -> BTreeMap<String, i64> {
    let mut out = BTreeMap::from([
        ("INTRADAY".into(), 0),
        ("MULTI_DAY".into(), 0),
        ("MULTI_WEEK".into(), 0),
        ("LONG_TERM".into(), 0),
    ]);
    for r in recs {
        let cls = r
            .get("horizon_class")
            .and_then(|v| v.as_str())
            .unwrap_or("INTRADAY");
        *out.entry(cls.to_string()).or_insert(0) += 1;
    }
    out
}

fn rec_card(
    market: &str,
    horizon_id: &str,
    horizon_class: &str,
    horizon_term: &str,
    symbol: &str,
    side: &str,
    decision: &str,
    entry_price: Option<f64>,
    stop: Option<Value>,
    target: Option<Value>,
    status: &str,
    thesis: &str,
    source: &str,
    extra: Map<String, Value>,
) -> Value {
    let mut card = json!({
        "market": market,
        "horizon_id": horizon_id,
        "horizon_class": horizon_class,
        "horizon_label": label(horizon_class),
        "horizon_term": horizon_term,
        "symbol": symbol,
        "side": side,
        "decision": decision,
        "entry_price": entry_price,
        "stop_loss_price": stop,
        "target_price": target,
        "status": status,
        "thesis": thesis,
        "source": source,
        "live_order_authority": LIVE_ORDER_AUTHORITY,
    });
    if let Some(obj) = card.as_object_mut() {
        for (k, v) in extra {
            obj.insert(k, v);
        }
    }
    card
}

fn safe_load_json(path: &Path) -> Option<Value> {
    fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
}

fn nse_status(workspace: &Path, nse_dir: &Path) -> Value {
    let journals = rp609_journals(workspace);
    if let Err(e) = write_rp609_registry(&journals) {
        return json!({
            "available": false,
            "error": e,
            "recommendations": [],
            "catalog": [],
        });
    }
    let eng = match MultiHorizonEngine::new(journals.clone(), nse_dir.to_path_buf()) {
        Ok(e) => e,
        Err(e) => {
            return json!({
                "available": false,
                "error": e,
                "recommendations": [],
                "catalog": [],
            })
        }
    };
    let report = eng.report();
    let mut recommendations = Vec::new();

    if let Some(open) = eng.swing.get("open").and_then(|v| v.as_object()) {
        for t in open.values() {
            recommendations.push(rec_card(
                "NSE",
                "H1_SWING",
                "MULTI_DAY",
                &term_lookup(TERM_NSE, "H1_SWING"),
                t.get("symbol").and_then(|v| v.as_str()).unwrap_or(""),
                t.get("side").and_then(|v| v.as_str()).unwrap_or("LONG"),
                "RECOMMEND_SWING",
                t.get("entry_price")
                    .or_else(|| t.get("reference_price"))
                    .and_then(|v| v.as_f64()),
                t.get("invalidation_price")
                    .or_else(|| t.get("stop_loss_price"))
                    .cloned(),
                t.get("target_price").cloned(),
                t.get("status").and_then(|v| v.as_str()).unwrap_or("OPEN"),
                t.get("thesis_tag")
                    .or_else(|| t.get("thesis"))
                    .and_then(|v| v.as_str())
                    .unwrap_or(""),
                "RP-609",
                Map::from_iter([
                    ("trade_id".into(), t.get("trade_id").cloned().unwrap_or(Value::Null)),
                    ("product_type".into(), json!("CNC")),
                ]),
            ));
        }
    }

    if let Some(plans) = eng.portfolio.get("plans").and_then(|v| v.as_object()) {
        for plan in plans.values() {
            let mut leg_syms = Vec::new();
            let mut legs_out = Vec::new();
            if let Some(legs) = plan.get("legs").and_then(|v| v.as_array()) {
                for leg in legs {
                    let Some(s) = leg.get("symbol").and_then(|v| v.as_str()) else {
                        continue;
                    };
                    if s.is_empty() {
                        continue;
                    }
                    if !leg_syms.iter().any(|x| x == s) {
                        leg_syms.push(s.to_string());
                    }
                    legs_out.push(json!({
                        "symbol": s,
                        "sector": leg.get("sector").and_then(|v| v.as_str()).unwrap_or("—"),
                        "target_weight_pct": leg.get("target_weight_pct").and_then(|v| v.as_f64()).unwrap_or(0.0),
                        "qty_filled": leg.get("qty_filled").and_then(|v| v.as_f64()).unwrap_or(0.0),
                        "notional_filled": leg.get("notional_filled").and_then(|v| v.as_f64()).unwrap_or(0.0),
                        "avg_price": leg.get("avg_price").cloned().unwrap_or(Value::Null),
                    }));
                }
            }
            let plan_id = plan
                .get("plan_id")
                .and_then(|v| v.as_str())
                .unwrap_or("PORTFOLIO");
            let filled = plan
                .get("tranches_filled")
                .and_then(|v| v.as_i64())
                .unwrap_or(0);
            let n_tr = plan.get("n_tranches").and_then(|v| v.as_i64()).unwrap_or(0);
            let tranche_progress = if n_tr > 0 {
                format!("{filled}/{n_tr}")
            } else {
                "—".to_string()
            };
            recommendations.push(rec_card(
                "NSE",
                "H2_PORTFOLIO_BUILD",
                "MULTI_WEEK",
                &term_lookup(TERM_NSE, "H2_PORTFOLIO_BUILD"),
                plan_id,
                "LONG",
                "PORTFOLIO_BUILD",
                None,
                None,
                None,
                plan.get("status").and_then(|v| v.as_str()).unwrap_or("OPEN"),
                plan_id,
                "RP-609",
                Map::from_iter([
                    ("book_kind".into(), json!("PORTFOLIO_BUILD")),
                    ("tranches_filled".into(), json!(filled)),
                    ("n_tranches".into(), json!(n_tr)),
                    ("tranche_progress".into(), json!(tranche_progress)),
                    (
                        "total_capital_inr".into(),
                        plan.get("total_capital_inr").cloned().unwrap_or(Value::Null),
                    ),
                    ("product_type".into(), json!("CNC")),
                    ("plan_id".into(), json!(plan_id)),
                    ("symbols".into(), json!(leg_syms)),
                    ("legs".into(), json!(legs_out)),
                    ("leg_count".into(), json!(legs_out.len())),
                ]),
            ));
        }
    }

    if let Some(open) = eng
        .long_term
        .get("open")
        .or_else(|| eng.long_term.get("holds"))
        .and_then(|v| v.as_object())
    {
        for h in open.values() {
            let st = h.get("status").and_then(|v| v.as_str()).unwrap_or("HOLDING");
            if matches!(st, "CLOSED" | "THESIS_INVALIDATED") {
                continue;
            }
            recommendations.push(rec_card(
                "NSE",
                "H3_LONG_TERM",
                "LONG_TERM",
                &term_lookup(TERM_NSE, "H3_LONG_TERM"),
                h.get("symbol").and_then(|v| v.as_str()).unwrap_or(""),
                h.get("side").and_then(|v| v.as_str()).unwrap_or("LONG"),
                "HOLD_LONG_TERM",
                h.get("entry_price").and_then(|v| v.as_f64()),
                None,
                None,
                st,
                h.get("thesis").and_then(|v| v.as_str()).unwrap_or(""),
                "RP-609",
                Map::from_iter([
                    ("hold_id".into(), h.get("hold_id").cloned().unwrap_or(Value::Null)),
                    (
                        "thesis_version".into(),
                        h.get("thesis_version").cloned().unwrap_or(Value::Null),
                    ),
                    ("product_type".into(), json!("CNC")),
                ]),
            ));
        }
    }

    let mut catalog = Vec::new();
    for (hid, meta) in rp609_horizons() {
        let cls = class_nse(&hid);
        catalog.push(json!({
            "horizon_id": hid,
            "horizon_class": cls,
            "horizon_label": label(cls),
            "horizon_term": term_lookup(TERM_NSE, &hid),
            "label": meta.get("label"),
            "product_type": meta.get("product_type"),
            "owner_protocol": meta.get("owner_protocol"),
            "observational_link_only": meta.get("observational_link_only").and_then(|v| v.as_bool()).unwrap_or(false),
        }));
    }
    let gates = safe_load_json(&journals.join("RP609_promotion_gates.json")).unwrap_or(json!({}));
    json!({
        "available": true,
        "protocol_id": RP609_PROTOCOL,
        "live_order_authority": LIVE_ORDER_AUTHORITY,
        "updated_utc": iso_now(),
        "journals": journals.display().to_string(),
        "catalog": catalog,
        "report": report,
        "recommendations": recommendations,
        "promotion_gates": gates.get("gates").cloned().unwrap_or(gates),
        "counts_by_class": count_by_class(
            &recommendations
        ),
    })
}

fn crypto_status(workspace: &Path) -> Value {
    let journals = rp610_journals(workspace);
    let store = Rp610Store::new(journals.clone());
    if let Err(e) = store.write_horizon_registry() {
        return json!({
            "available": false,
            "error": e,
            "recommendations": [],
            "catalog": [],
        });
    }
    let mut recommendations = Vec::new();
    for (hid, cls) in [
        ("H1_MULTI_DAY", "MULTI_DAY"),
        ("H2_MULTI_WEEK", "MULTI_WEEK"),
        ("H3_LONG_TERM", "LONG_TERM"),
    ] {
        let opens = open_holds_from_book(&store, hid).unwrap_or_default();
        for h in opens {
            recommendations.push(rec_card(
                "CRYPTO",
                hid,
                cls,
                &term_lookup(TERM_CRYPTO, hid),
                h.get("symbol").and_then(|v| v.as_str()).unwrap_or("BTCUSDT"),
                h.get("side").and_then(|v| v.as_str()).unwrap_or("LONG"),
                &format!("RECOMMEND_{cls}"),
                h.get("entry_price").and_then(|v| v.as_f64()),
                h.get("stop_pct").cloned(),
                h.get("target_pct").cloned(),
                h.get("status").and_then(|v| v.as_str()).unwrap_or("OPEN"),
                h.get("thesis").and_then(|v| v.as_str()).unwrap_or(""),
                "RP-610",
                Map::from_iter([
                    (
                        "candidate_id".into(),
                        h.get("candidate_id").cloned().unwrap_or(Value::Null),
                    ),
                    ("product_type".into(), json!("SPOT_SIM")),
                    (
                        "hold_days_min".into(),
                        h.get("hold_days_min").cloned().unwrap_or(Value::Null),
                    ),
                    (
                        "hold_days_max".into(),
                        h.get("hold_days_max").cloned().unwrap_or(Value::Null),
                    ),
                ]),
            ));
        }
    }
    let mut catalog = Vec::new();
    for (hid, meta) in rp610_horizons() {
        let cls = class_crypto(&hid);
        catalog.push(json!({
            "horizon_id": hid,
            "horizon_class": cls,
            "horizon_label": label(cls),
            "horizon_term": term_lookup(TERM_CRYPTO, &hid),
            "label": meta.get("label"),
            "product_type": meta.get("product_type"),
            "owner_protocol": meta.get("owner_protocol"),
            "observational_link_only": meta.get("observational_link_only").and_then(|v| v.as_bool()).unwrap_or(false),
        }));
    }
    let gates = safe_load_json(&journals.join("RP610_promotion_gates.json")).unwrap_or(json!({}));
    let summary = summarize_books(&store).unwrap_or_else(|e| json!({"error": e}));
    json!({
        "available": true,
        "protocol_id": RP610_PROTOCOL,
        "live_order_authority": LIVE_ORDER_AUTHORITY,
        "updated_utc": iso_now(),
        "journals": journals.display().to_string(),
        "catalog": catalog,
        "summary": summary,
        "recommendations": recommendations,
        "promotion_gates": gates.get("gates").cloned().unwrap_or(gates),
        "counts_by_class": count_by_class(&recommendations),
    })
}

pub fn get_multi_horizon_dashboard_state(workspace: &Path) -> Value {
    let nse_dir = workspace
        .join("live_capture")
        .join("shadow_validation")
        .join("nse");
    let nse = nse_status(workspace, &nse_dir);
    let crypto = crypto_status(workspace);
    let mut all_recs = Vec::new();
    if let Some(arr) = nse.get("recommendations").and_then(|v| v.as_array()) {
        all_recs.extend(arr.clone());
    }
    if let Some(arr) = crypto.get("recommendations").and_then(|v| v.as_array()) {
        all_recs.extend(arr.clone());
    }
    json!({
        "live_order_authority": LIVE_ORDER_AUTHORITY,
        "updated_utc": iso_now(),
        "classes": ["INTRADAY", "MULTI_DAY", "MULTI_WEEK", "LONG_TERM"],
        "nse": nse,
        "crypto": crypto,
        "recommendations": all_recs.clone(),
        "counts_by_class": count_by_class(&all_recs),
        "note": "H0/INTRADAY live books remain owned by RP-608 daemons. H1–H3 are RP-609/610 observation-only paper books (gates CLOSED).",
    })
}

pub fn workspace_from_manifest() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."))
}
