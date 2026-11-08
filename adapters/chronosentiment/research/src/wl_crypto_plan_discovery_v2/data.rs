//! Load + enrich crypto plan outcomes (decision-time only).

use super::types::{RawPlan, TradeRow, WlStats};
use chrono::{FixedOffset, TimeZone};
use std::collections::{BTreeMap, HashSet};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub const DISCOVERY_DAYS: &[&str] = &["2026-11-03", "2026-11-04", "2026-11-05"];
pub const HOLDOUT_PRE: &[&str] = &[
    "2026-10-26",
    "2026-10-27",
    "2026-10-28",
    "2026-10-29",
    "2026-10-30",
    "2026-10-31",
    "2026-11-01",
    "2026-11-02",
];
pub const HOLDOUT_EX_ANOMALY: &[&str] = &[
    "2026-10-26",
    "2026-10-27",
    "2026-10-28",
    "2026-10-30",
    "2026-10-31",
    "2026-11-01",
    "2026-11-02",
];
pub const HOLDOUT_FWD: &[&str] = &["2026-11-06"];
pub const OF_ERA_START: &str = "2026-11-01";
pub const ANOMALY_DAY: &str = "2026-10-29";

fn ist() -> FixedOffset {
    FixedOffset::east_opt(5 * 3600 + 30 * 60).expect("IST offset")
}

fn day_ist_from_ms(ms: i64) -> Option<String> {
    let secs = if ms > 10_000_000_000 { ms / 1000 } else { ms };
    let dt = chrono::Utc.timestamp_opt(secs, 0).single()?;
    Some(dt.with_timezone(&ist()).format("%Y-%m-%d").to_string())
}

fn hour_ist_from_ms(ms: i64) -> String {
    let secs = if ms > 10_000_000_000 { ms / 1000 } else { ms };
    match chrono::Utc.timestamp_opt(secs, 0).single() {
        Some(dt) => dt.with_timezone(&ist()).format("%H").to_string(),
        None => "??".into(),
    }
}

pub fn atr_bucket(atr: Option<f64>, entry: Option<f64>) -> String {
    match (atr, entry) {
        (Some(a), Some(e)) if e > 0.0 => {
            let rel = a / e;
            if rel < 0.002 {
                "ATR_REL_<20bps".into()
            } else if rel < 0.004 {
                "ATR_REL_20_40bps".into()
            } else if rel < 0.008 {
                "ATR_REL_40_80bps".into()
            } else {
                "ATR_REL_≥80bps".into()
            }
        }
        _ => "ATR_REL_UNAVAILABLE".into(),
    }
}

pub fn imb_bucket(imb: Option<f64>) -> String {
    match imb {
        None => "IMB_UNAVAILABLE".into(),
        Some(v) => {
            let a = v.abs();
            if a < 0.30 {
                "ABS_IMB_<0.30".into()
            } else if a < 0.50 {
                "ABS_IMB_0.30_0.50".into()
            } else if a < 0.70 {
                "ABS_IMB_0.50_0.70".into()
            } else {
                "ABS_IMB_≥0.70".into()
            }
        }
    }
}

pub fn rr_bucket(rr: Option<f64>) -> String {
    match rr {
        None => "NET_RR_UNAVAILABLE".into(),
        Some(v) if v < 1.0 => "NET_RR_<1".into(),
        Some(v) if v < 1.5 => "NET_RR_1_1.5".into(),
        Some(v) if v < 2.0 => "NET_RR_1.5_2".into(),
        Some(_) => "NET_RR_≥2".into(),
    }
}

fn window_for(day: &str) -> String {
    if DISCOVERY_DAYS.contains(&day) {
        "discovery".into()
    } else if HOLDOUT_FWD.contains(&day) {
        "holdout_fwd".into()
    } else if day == ANOMALY_DAY {
        "holdout_pre_ANOMALY".into()
    } else if HOLDOUT_PRE.contains(&day) {
        "holdout_pre".into()
    } else {
        "other".into()
    }
}

pub fn wilson_low(wins: usize, n: usize) -> Option<f64> {
    if n == 0 {
        return None;
    }
    let z = 1.96_f64;
    let p = wins as f64 / n as f64;
    let denom = 1.0 + z * z / n as f64;
    let centre = p + z * z / (2.0 * n as f64);
    let adj = z * ((p * (1.0 - p) / n as f64) + z * z / (4.0 * n as f64 * n as f64)).sqrt();
    Some(((centre - adj) / denom).max(0.0) * 100.0)
}

pub fn wl_stats(rows: &[&TradeRow], book_n: Option<usize>) -> WlStats {
    let n = rows.len();
    let wins = rows.iter().filter(|r| r.win).count();
    let net_sum: f64 = rows.iter().map(|r| r.net).sum();
    WlStats {
        n,
        wins,
        losses: n.saturating_sub(wins),
        win_rate_pct: if n == 0 {
            None
        } else {
            Some(100.0 * wins as f64 / n as f64)
        },
        wilson_low_95_pct: wilson_low(wins, n),
        net_sum,
        coverage_pct: book_n.map(|b| {
            if b == 0 {
                0.0
            } else {
                100.0 * n as f64 / b as f64
            }
        }),
    }
}

pub fn load_trades(path: &Path) -> Result<Vec<TradeRow>, String> {
    let f = File::open(path).map_err(|e| format!("open {}: {e}", path.display()))?;
    let reader = BufReader::new(f);
    let mut out = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();

    for (lineno, line) in reader.lines().enumerate() {
        let line = line.map_err(|e| format!("read line {}: {e}", lineno + 1))?;
        if line.trim().is_empty() {
            continue;
        }
        let raw: RawPlan = serde_json::from_str(&line)
            .map_err(|e| format!("json line {}: {e}", lineno + 1))?;
        let Some(net) = raw.net_pnl_usdt else {
            continue;
        };
        let Some(exit_ms) = raw.exit_time_ms else {
            continue;
        };
        let Some(day) = day_ist_from_ms(exit_ms) else {
            continue;
        };
        let plan_id = raw
            .plan_id
            .clone()
            .unwrap_or_else(|| format!("anon:{}:{}", day, lineno));
        if !seen.insert(plan_id.clone()) {
            continue;
        }

        let direction = raw
            .direction
            .clone()
            .unwrap_or_else(|| "?".into())
            .to_uppercase();
        let of_supported = match raw.order_flow_supported {
            Some(true) => "OF_SUP=True".to_string(),
            Some(false) => "OF_SUP=False".to_string(),
            None => "OF_SUP_UNAVAILABLE".to_string(),
        };

        let entry = raw.entry_price.or(raw.reference_price);
        let hour = raw
            .entry_time_ms
            .map(hour_ist_from_ms)
            .unwrap_or_else(|| "??".into());

        out.push(TradeRow {
            plan_id,
            day_ist: day.clone(),
            window: window_for(&day),
            win: net > 0.0,
            net,
            direction: direction.clone(),
            of_supported: of_supported.clone(),
            dir_x_of: format!("{direction}|{of_supported}"),
            imb_bucket: imb_bucket(raw.order_flow_imbalance),
            atr_bucket: atr_bucket(raw.atr_14, entry),
            net_rr_bucket: rr_bucket(raw.net_rr_ratio),
            hour_ist: hour,
            of_era: day.as_str() >= OF_ERA_START,
            symbol: raw.symbol.unwrap_or_else(|| "BTCUSDT".into()),
        });
    }
    Ok(out)
}

pub fn filter_days<'a>(rows: &'a [TradeRow], days: &[&str]) -> Vec<&'a TradeRow> {
    rows.iter()
        .filter(|r| days.contains(&r.day_ist.as_str()))
        .collect()
}

pub fn per_day_stats(rows: &[&TradeRow]) -> BTreeMap<String, WlStats> {
    let mut map: BTreeMap<String, Vec<&TradeRow>> = BTreeMap::new();
    for r in rows {
        map.entry(r.day_ist.clone()).or_default().push(*r);
    }
    map.into_iter()
        .map(|(d, rs)| {
            let n = rs.len();
            (d, wl_stats(&rs, Some(n)))
        })
        .collect()
}
