//! Deferred Live session performance — descriptive paper marks.
//!
//! Projects the live ledger against DecisionBrief facts. Does not trade,
//! does not reassess, does not replay `paper_trader_v2_*.csv`, and is not a
//! G-GATE predictive-value claim.
//!
//! OPEN returns are mark-to-last-observation, not lifecycle exits.
//! See `docs/CS-P-001_DECISION_SUPPORT_PRODUCT_MODE.md` Stage C.

use serde::{Deserialize, Serialize};

use super::deferred_live::{LivePaperLedger, LivePaperPosition, LivePaperStatus};
use super::intraday_decision::DecisionBrief;
use super::paper_lifecycle::signed_return;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeferredLivePerformanceRow {
    pub decision_id: String,
    pub ticker: String,
    pub direction: String,
    pub oqs: Option<u32>,
    pub decision_entry_price: Option<f64>,
    pub paper_entry_price: f64,
    /// Signed fill vs DecisionBrief.entry_price. `None` if the brief has no entry.
    pub fill_vs_decision: Option<f64>,
    pub last_or_exit_price: f64,
    pub outcome: String,
    /// `MARK` while OPEN; `REALIZED` after TARGET / STOP / HORIZON.
    pub ret_kind: String,
    pub ret: f64,
    pub bars_held: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeferredLivePerformance {
    pub date: Option<String>,
    pub n_entered: usize,
    pub n_open: usize,
    pub n_target: usize,
    pub n_stop: usize,
    pub n_horizon: usize,
    pub n_win_closed: usize,
    pub mean_closed_return: Option<f64>,
    pub mean_open_mark: Option<f64>,
    pub mean_fill_vs_decision: Option<f64>,
    pub rows: Vec<DeferredLivePerformanceRow>,
    pub note: String,
}

fn outcome_label(pos: &LivePaperPosition) -> String {
    match pos.exit_reason() {
        Some(r) => r.to_string(),
        None => "OPEN".into(),
    }
}

fn mean(xs: &[f64]) -> Option<f64> {
    if xs.is_empty() {
        None
    } else {
        Some(xs.iter().sum::<f64>() / xs.len() as f64)
    }
}

/// Score the deferred-live book. Equal-weight names. No invented notionals.
pub fn score_deferred_live(
    ledger: &LivePaperLedger,
    briefs: &[DecisionBrief],
    date: Option<&str>,
) -> DeferredLivePerformance {
    let mut by_id = std::collections::HashMap::new();
    for b in briefs {
        by_id.insert(b.id.as_str(), b);
    }
    let mut rows = Vec::new();
    let mut n_open = 0usize;
    let mut n_target = 0usize;
    let mut n_stop = 0usize;
    let mut n_horizon = 0usize;
    let mut closed = Vec::new();
    let mut open_marks = Vec::new();
    let mut slips = Vec::new();

    for pos in &ledger.positions {
        let id = pos.paper.decision_id.clone().unwrap_or_default();
        let brief = by_id.get(id.as_str()).copied();
        let decision_entry = brief.and_then(|b| b.entry_price);
        let fill = pos.paper_entry_price();
        let fill_vs_decision = decision_entry.filter(|d| *d != 0.0).map(|d| {
            signed_return(&pos.paper.direction, d, fill)
        });
        if let Some(s) = fill_vs_decision {
            slips.push(s);
        }
        let outcome = outcome_label(pos);
        match outcome.as_str() {
            "OPEN" => n_open += 1,
            "TARGET" => n_target += 1,
            "STOP" => n_stop += 1,
            "HORIZON" => n_horizon += 1,
            _ => {}
        }
        let open = pos.status() == LivePaperStatus::Open;
        let last = if open {
            pos.current_price
        } else {
            pos.paper_exit_price().unwrap_or(pos.current_price)
        };
        let ret = if open {
            pos.unrealized_return()
        } else {
            pos.realized_return().unwrap_or_else(|| {
                signed_return(&pos.paper.direction, fill, last)
            })
        };
        if open {
            open_marks.push(ret);
        } else {
            closed.push(ret);
        }
        rows.push(DeferredLivePerformanceRow {
            decision_id: id,
            ticker: pos.paper.ticker.clone(),
            direction: pos.paper.direction.clone(),
            oqs: brief.map(|b| b.oqs).or(pos.paper.oqs),
            decision_entry_price: decision_entry,
            paper_entry_price: fill,
            fill_vs_decision,
            last_or_exit_price: last,
            outcome,
            ret_kind: if open { "MARK".into() } else { "REALIZED".into() },
            ret,
            bars_held: pos.paper.bars_held,
        });
    }
    rows.sort_by(|a, b| a.ticker.cmp(&b.ticker).then(a.decision_id.cmp(&b.decision_id)));
    let n_win_closed = closed.iter().filter(|r| **r > 0.0).count();
    DeferredLivePerformance {
        date: date.map(str::to_string),
        n_entered: ledger.positions.len(),
        n_open,
        n_target,
        n_stop,
        n_horizon,
        n_win_closed,
        mean_closed_return: mean(&closed),
        mean_open_mark: mean(&open_marks),
        mean_fill_vs_decision: mean(&slips),
        rows,
        note: "Descriptive Deferred Live paper marks (equal-weight names). \
               OPEN is mark-to-last-observation, not an exit. \
               Not a G-GATE predictive-value claim. Not paper_trader_v2 replay."
            .into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::product::deferred_live::{DeferredLiveConfig, MarketObservation};
    use crate::product::deferred_live_loop::DeferredLiveRuntime;
    use crate::product::intraday_decision::ExecutionFacts;
    use crate::product::DecisionBrief;

    fn brief(id: &str, ticker: &str, entry: f64, target: f64, risk: f64) -> DecisionBrief {
        DecisionBrief {
            id: id.into(),
            ticker: ticker.into(),
            date: "2026-09-07".into(),
            direction: "SHORT".into(),
            oqs: 53,
            h60_class: "WAIT".into(),
            reference_price: Some(entry + 2.0),
            entry_price: Some(entry),
            execution: ExecutionFacts {
                snap_unix: Some(1_000),
                adaptive_target: Some(target),
                adaptive_risk: Some(risk),
                adaptive_horizon_sessions: Some(1.0),
                target_distance_abs: None,
                target_distance_pct: None,
                risk_distance_abs: None,
                risk_distance_pct: None,
                expected_move_pct: None,
                current_price: None,
                last_tick_unix: None,
                freshness: "STALE".into(),
                horizon_elapsed: Some(false),
            },
            entry_state: "WAIT-HIGH".into(),
            entry_action: "ACT".into(),
            entry_confidence: "HIGH".into(),
            entry_horizon: "H300".into(),
            entry_why: "t".into(),
            entry_risk: "t".into(),
            h120_state: "WAIT-HIGH".into(),
            h120_action: "ACT".into(),
            h120_confidence: "HIGH".into(),
            h120_horizon: "H300".into(),
            h120_why: "t".into(),
            h120_risk: "t".into(),
            h15_ret: None,
            h30_ret: None,
            h60_ret: None,
            h120_ret: None,
            h180_ret: None,
            h300_ret: None,
            mfe_h60: None,
            mfe_h120: None,
            outcome: None,
            pnl: None,
            hist_win: None,
            hist_pf: None,
            hist_med: None,
        }
    }

    #[test]
    fn open_mark_is_not_a_closed_return() {
        let b = brief("d1", "AAA_NS", 473.0, 450.0, 490.0);
        let mut rt = DeferredLiveRuntime::new(DeferredLiveConfig { horizon_secs: 10_000, strict_t0_admission: false, ..Default::default() });
        rt.arm(b.clone());
        rt.ingest(MarketObservation::last("AAA_NS", 1_000, 480.5));
        rt.ingest(MarketObservation::last("AAA_NS", 1_060, 475.4));
        let p = score_deferred_live(rt.ledger(), &[b], Some("2026-09-07"));
        assert_eq!(p.n_entered, 1);
        assert_eq!(p.n_open, 1);
        assert_eq!(p.n_target + p.n_stop + p.n_horizon, 0);
        assert_eq!(p.rows[0].ret_kind, "MARK");
        assert_eq!(p.rows[0].paper_entry_price, 480.5);
        assert_eq!(p.rows[0].decision_entry_price, Some(473.0));
        assert!(p.mean_closed_return.is_none());
        assert!(p.mean_open_mark.is_some());
        assert!(p.mean_fill_vs_decision.unwrap() < 0.0, "SHORT fill above decision entry is adverse");
    }

    #[test]
    fn target_exit_is_realized_and_excluded_from_open_mark() {
        let b = brief("d1", "AAA_NS", 473.0, 475.0, 490.0);
        let mut rt = DeferredLiveRuntime::new(DeferredLiveConfig::default());
        rt.arm(b.clone());
        rt.ingest(MarketObservation::last("AAA_NS", 1_000, 480.5));
        rt.ingest(MarketObservation {
            ticker: "AAA_NS".into(),
            unix: 1_060,
            price: 474.8,
            high: Some(476.0),
            low: Some(474.5),
        });
        let p = score_deferred_live(rt.ledger(), &[b], None);
        assert_eq!(p.n_target, 1);
        assert_eq!(p.n_open, 0);
        assert_eq!(p.rows[0].ret_kind, "REALIZED");
        assert_eq!(p.rows[0].outcome, "TARGET");
        assert_eq!(p.rows[0].last_or_exit_price, 475.0);
        assert!(p.mean_open_mark.is_none());
        assert!(p.mean_closed_return.is_some());
    }
}
