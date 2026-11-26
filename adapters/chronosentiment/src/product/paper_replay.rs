//! Frozen Paper Trader v0.2 replay — adapter-owned paper blotter.
//!
//! Reads `datasets/paper_trader_v2_YYYYMMDD.csv` produced by
//! `scripts/run_paper_trader_v2.py`. The walk is not re-run here.
//! Shared TARGET/STOP/HORIZON semantics live in [`super::paper_lifecycle`].
//!
//! CSV `entry_price` is the paper fill used by that engine (ledger reference
//! at bar 13). It is **not** `DecisionBrief.entry_price` and must not replace it.
//!
//! See `docs/CS-P-001_DECISION_SUPPORT_PRODUCT_MODE.md`.

use serde::{Deserialize, Serialize};
use std::path::Path;

use super::intraday_decision::DecisionBrief;

/// One frozen paper-trader v0.2 position. Field names are prefixed so they
/// cannot be confused with DecisionBrief prices.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
pub struct PaperPosition {
    pub ticker: String,
    pub direction: String,
    /// YYYY-MM-DD (normalized from CSV YYYYMMDD).
    pub date: String,
    /// Paper-trader fill. Not DecisionBrief.entry_price.
    pub paper_entry_price: f64,
    /// Target the paper walk used (LIVE-005 adaptive_target).
    pub paper_target: f64,
    /// Risk boundary the paper walk recorded.
    pub paper_risk: f64,
    pub candidate_stop_pct: Option<f64>,
    pub candidate_stop_price: Option<f64>,
    pub stop_confidence: String,
    pub coverage_quality: String,
    pub n_obs: i32,
    pub path_context: String,
    pub final_tier: String,
    pub paper_exit_price: Option<f64>,
    pub exit_time_ist: Option<String>,
    pub exit_reason: String,
    pub realized_return: Option<f64>,
    pub max_adverse_excursion: Option<f64>,
    pub max_favourable_excursion: Option<f64>,
    pub bars_held: i32,
    pub h300_counterfactual_ret: Option<f64>,
    pub stop_consequence_pp: Option<f64>,
    /// Joined from DecisionBrief when ticker+date+direction match. Never invented.
    pub decision_id: Option<String>,
    pub entry_action: Option<String>,
    pub oqs: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct PaperBlotter {
    pub total: usize,
    pub n_stop: usize,
    pub n_target: usize,
    pub n_horizon: usize,
    pub n_win: usize,
    pub mean_realized_return: Option<f64>,
    pub total_realized_return: Option<f64>,
    pub positions: Vec<PaperPosition>,
}

fn normalize_date(raw: &str) -> String {
    if raw.len() == 8 && raw.chars().all(|c| c.is_ascii_digit()) {
        format!("{}-{}-{}", &raw[0..4], &raw[4..6], &raw[6..8])
    } else {
        raw.to_string()
    }
}

fn parse_f64(s: &str) -> Option<f64> {
    let t = s.trim();
    if t.is_empty() {
        None
    } else {
        t.parse().ok()
    }
}

fn parse_i32(s: &str) -> i32 {
    s.trim().parse().unwrap_or(0)
}

fn parse_paper_row(header: &[&str], fields: &[&str]) -> Option<PaperPosition> {
    let idx = |name: &str| header.iter().position(|c| *c == name);
    let get = |name: &str| {
        idx(name)
            .and_then(|n| fields.get(n).copied())
            .unwrap_or("")
            .trim()
            .to_string()
    };

    let ticker = get("ticker");
    let date = normalize_date(&get("date"));
    let paper_entry_price = parse_f64(&get("entry_price"))?;
    let paper_target = parse_f64(&get("adaptive_target"))?;
    let paper_risk = parse_f64(&get("adaptive_risk"))?;
    if ticker.is_empty() || date.is_empty() {
        return None;
    }

    Some(PaperPosition {
        ticker,
        direction: get("direction"),
        date,
        paper_entry_price,
        paper_target,
        paper_risk,
        candidate_stop_pct: parse_f64(&get("candidate_stop_pct")),
        candidate_stop_price: parse_f64(&get("candidate_stop_price")),
        stop_confidence: get("stop_confidence"),
        coverage_quality: get("coverage_quality"),
        n_obs: parse_i32(&get("n_obs")),
        path_context: get("path_context"),
        final_tier: get("final_tier"),
        paper_exit_price: parse_f64(&get("exit_price")),
        exit_time_ist: {
            let v = get("exit_time_ist");
            if v.is_empty() || v == "None" {
                None
            } else {
                Some(v)
            }
        },
        exit_reason: get("exit_reason"),
        realized_return: parse_f64(&get("realized_return")),
        max_adverse_excursion: parse_f64(&get("max_adverse_excursion")),
        max_favourable_excursion: parse_f64(&get("max_favourable_excursion")),
        bars_held: parse_i32(&get("bars_held")),
        h300_counterfactual_ret: parse_f64(&get("h300_counterfactual_ret")),
        stop_consequence_pp: parse_f64(&get("stop_consequence_pp")),
        decision_id: None,
        entry_action: None,
        oqs: None,
    })
}

/// Load frozen paper_trader_v2_*.csv files from `datasets_dir`.
pub fn load_paper_positions(datasets_dir: &str) -> Vec<PaperPosition> {
    let dir = Path::new(datasets_dir);
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut positions = Vec::new();
    let mut files: Vec<_> = entries.flatten().map(|e| e.path()).collect();
    files.sort();
    for path in files {
        let fname = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if !fname.starts_with("paper_trader_v2_") || !fname.ends_with(".csv") {
            continue;
        }
        let Ok(content) = std::fs::read_to_string(&path) else {
            continue;
        };
        let mut lines = content.lines();
        let Some(header_line) = lines.next() else {
            continue;
        };
        let header: Vec<&str> = header_line.split(',').collect();
        for line in lines {
            if line.trim().is_empty() {
                continue;
            }
            let fields: Vec<&str> = line.split(',').collect();
            if let Some(pos) = parse_paper_row(&header, &fields) {
                positions.push(pos);
            }
        }
    }
    positions.sort_by(|a, b| b.date.cmp(&a.date).then(a.ticker.cmp(&b.ticker)));
    positions
}

/// Attach DecisionBrief identity when ticker, date, and direction match.
/// Does not copy brief prices onto the paper fill.
pub fn attach_decision_links(positions: &mut [PaperPosition], briefs: &[DecisionBrief]) {
    for pos in positions.iter_mut() {
        let mut matches: Vec<&DecisionBrief> = briefs
            .iter()
            .filter(|b| {
                b.ticker == pos.ticker && b.date == pos.date && b.direction == pos.direction
            })
            .collect();
        matches.sort_by(|a, b| b.oqs.cmp(&a.oqs));
        let chosen = matches
            .iter()
            .find(|b| b.entry_action == "ACT")
            .copied()
            .or_else(|| matches.first().copied());
        if let Some(b) = chosen {
            pos.decision_id = Some(b.id.clone());
            pos.entry_action = Some(b.entry_action.clone());
            pos.oqs = Some(b.oqs);
        }
    }
}

pub fn assemble_paper_blotter(mut positions: Vec<PaperPosition>) -> PaperBlotter {
    let n_stop = positions.iter().filter(|p| p.exit_reason == "STOP").count();
    let n_target = positions.iter().filter(|p| p.exit_reason == "TARGET").count();
    let n_horizon = positions
        .iter()
        .filter(|p| p.exit_reason == "HORIZON")
        .count();
    let rets: Vec<f64> = positions.iter().filter_map(|p| p.realized_return).collect();
    let n_win = rets.iter().filter(|r| **r > 0.0).count();
    let total_realized_return = if rets.is_empty() {
        None
    } else {
        Some(rets.iter().sum())
    };
    let mean_realized_return = if rets.is_empty() {
        None
    } else {
        Some(rets.iter().sum::<f64>() / rets.len() as f64)
    };
    let total = positions.len();
    positions.sort_by(|a, b| b.date.cmp(&a.date).then(a.ticker.cmp(&b.ticker)));
    PaperBlotter {
        total,
        n_stop,
        n_target,
        n_horizon,
        n_win,
        mean_realized_return,
        total_realized_return,
        positions,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn csv_date_normalizes_to_iso() {
        assert_eq!(normalize_date("20260907"), "2026-09-07");
        assert_eq!(normalize_date("2026-09-07"), "2026-09-07");
    }

    #[test]
    fn paper_entry_is_parsed_from_csv_entry_price_field() {
        let header = "ticker,direction,date,entry_price,adaptive_target,adaptive_risk,candidate_stop_pct,candidate_stop_price,stop_confidence,coverage_quality,n_obs,path_context,final_tier,exit_price,exit_time_ist,exit_reason,realized_return,max_adverse_excursion,max_favourable_excursion,bars_held,h300_counterfactual_ret,stop_consequence_pp";
        let row = "HDFCBANK_NS,LONG,20260907,710.500000,725.882513,693.238773,-0.002264,708.890000,MEDIUM,USABLE,5,STRONG_FAVOUR,mae_h300_p50,708.890000,10:10,STOP,-0.002266,-0.002393,0.001970,44,-0.002182,-0.000084";
        let h: Vec<&str> = header.split(',').collect();
        let f: Vec<&str> = row.split(',').collect();
        let pos = parse_paper_row(&h, &f).unwrap();
        assert_eq!(pos.ticker, "HDFCBANK_NS");
        assert_eq!(pos.date, "2026-09-07");
        assert_eq!(pos.paper_entry_price, 710.5);
        assert_eq!(pos.paper_exit_price, Some(708.89));
        assert_eq!(pos.exit_reason, "STOP");
        assert_eq!(pos.decision_id, None);
    }

    #[test]
    fn blotter_aggregates_from_replay_rows_only() {
        let p = PaperPosition {
            ticker: "A".into(),
            direction: "LONG".into(),
            date: "2026-09-07".into(),
            paper_entry_price: 100.0,
            paper_target: 101.0,
            paper_risk: 99.0,
            candidate_stop_pct: None,
            candidate_stop_price: None,
            stop_confidence: "LOW".into(),
            coverage_quality: "LIMITED".into(),
            n_obs: 1,
            path_context: "NORMAL".into(),
            final_tier: "t".into(),
            paper_exit_price: Some(100.5),
            exit_time_ist: Some("10:14".into()),
            exit_reason: "HORIZON".into(),
            realized_return: Some(0.01),
            max_adverse_excursion: Some(0.0),
            max_favourable_excursion: Some(0.01),
            bars_held: 48,
            h300_counterfactual_ret: Some(0.01),
            stop_consequence_pp: Some(0.0),
            decision_id: None,
            entry_action: None,
            oqs: None,
        };
        let blotter = assemble_paper_blotter(vec![p.clone(), {
            let mut q = p;
            q.ticker = "B".into();
            q.exit_reason = "STOP".into();
            q.realized_return = Some(-0.002);
            q
        }]);
        assert_eq!(blotter.total, 2);
        assert_eq!(blotter.n_horizon, 1);
        assert_eq!(blotter.n_stop, 1);
        assert_eq!(blotter.n_win, 1);
        assert!((blotter.total_realized_return.unwrap() - 0.008).abs() < 1e-12);
    }
}
