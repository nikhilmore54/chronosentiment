//! Deferred Live universe loss audit — observation only.
//!
//! Projects an already-walked live ledger into per-name loss facts.
//! Does not trade, does not reassess, does not mutate the driver, and is not
//! a G-GATE predictive-value claim.
//!
//! See `docs/CS-P-001_DECISION_SUPPORT_PRODUCT_MODE.md` Stage C.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::deferred_live::{LivePaperLedger, LivePaperPosition, LivePaperStatus};
use super::intraday_decision::DecisionBrief;
use super::paper_lifecycle::signed_return;

/// One paper position after a cached-universe Deferred Live session.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeferredLiveAuditRow {
    pub symbol: String,
    pub date: String,
    pub direction: String,
    pub oqs: Option<u32>,
    pub decision_price: Option<f64>,
    pub paper_fill: f64,
    pub fill_vs_decision: Option<f64>,
    pub exit_reason: String,
    pub exit_price: Option<f64>,
    pub realized_pct: Option<f64>,
    pub mae: f64,
    pub mfe: f64,
    pub mark_at_tape_end: Option<f64>,
    pub profile_quality: Option<String>,
    pub risk: Option<f64>,
    pub target: Option<f64>,
    pub session_state: String,
    pub decision_id: String,
    pub bars_held: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CountKey {
    pub key: String,
    pub n: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LossBreakdown {
    pub loss_count: usize,
    pub closed_count: usize,
    pub loss_rate: Option<f64>,
    pub open_underwater: usize,
    pub worst_symbols: Vec<CountKey>,
    pub worst_directions: Vec<CountKey>,
    pub worst_exit_reasons: Vec<CountKey>,
    pub largest_individual_losses: Vec<DeferredLiveAuditRow>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UniverseAuditSummary {
    pub sessions: usize,
    pub symbols_cached: usize,
    pub symbols_entered: usize,
    pub act_decisions: usize,
    pub paper_entries: usize,
    pub n_target: usize,
    pub n_stop: usize,
    pub n_horizon: usize,
    pub n_open: usize,
    pub n_win: usize,
    pub n_loss: usize,
    pub mean_return: Option<f64>,
    pub total_return: Option<f64>,
    pub mean_closed_return: Option<f64>,
    pub mean_open_mark: Option<f64>,
    pub mean_fill_vs_decision: Option<f64>,
    pub losses: LossBreakdown,
    pub note: String,
}

fn mean(xs: &[f64]) -> Option<f64> {
    if xs.is_empty() {
        None
    } else {
        Some(xs.iter().sum::<f64>() / xs.len() as f64)
    }
}

fn exit_reason(pos: &LivePaperPosition) -> String {
    pos.exit_reason().unwrap_or("OPEN").to_string()
}

fn paper_return(pos: &LivePaperPosition) -> f64 {
    if pos.status() == LivePaperStatus::Open {
        pos.unrealized_return()
    } else {
        pos.realized_return().unwrap_or_else(|| {
            signed_return(
                &pos.paper.direction,
                pos.paper_entry_price(),
                pos.paper_exit_price().unwrap_or(pos.current_price),
            )
        })
    }
}

/// Map one live ledger row onto the universe-audit schema.
pub fn audit_row_from_position(
    pos: &LivePaperPosition,
    brief: Option<&DecisionBrief>,
    profile_quality: Option<String>,
) -> DeferredLiveAuditRow {
    let open = pos.status() == LivePaperStatus::Open;
    let decision_price = brief.and_then(|b| b.entry_price);
    let fill = pos.paper_entry_price();
    let fill_vs_decision = decision_price.filter(|d| *d != 0.0).map(|d| {
        signed_return(&pos.paper.direction, d, fill)
    });
    let ret = paper_return(pos);
    DeferredLiveAuditRow {
        symbol: pos.paper.ticker.replace("_NS", "").replace(".NS", ""),
        date: pos.paper.date.clone(),
        direction: pos.paper.direction.clone(),
        oqs: brief.map(|b| b.oqs).or(pos.paper.oqs),
        decision_price,
        paper_fill: fill,
        fill_vs_decision,
        exit_reason: exit_reason(pos),
        exit_price: if open {
            None
        } else {
            pos.paper_exit_price()
        },
        realized_pct: if open { None } else { Some(ret) },
        mae: pos.walk.mae,
        mfe: pos.walk.mfe,
        mark_at_tape_end: if open { Some(ret) } else { None },
        profile_quality,
        risk: Some(pos.paper.paper_risk).filter(|v| v.is_finite() && *v > 0.0),
        target: Some(pos.paper.paper_target).filter(|v| v.is_finite() && *v > 0.0),
        session_state: brief
            .map(|b| b.entry_action.clone())
            .or_else(|| pos.paper.entry_action.clone())
            .unwrap_or_else(|| "ACT".into()),
        decision_id: pos.paper.decision_id.clone().unwrap_or_default(),
        bars_held: pos.paper.bars_held,
    }
}

pub fn audit_rows_from_ledger(
    ledger: &LivePaperLedger,
    briefs: &[DecisionBrief],
    profiles: &BTreeMap<(String, String), String>,
) -> Vec<DeferredLiveAuditRow> {
    let mut by_id = BTreeMap::new();
    for b in briefs {
        by_id.insert(b.id.as_str(), b);
    }
    let mut rows: Vec<DeferredLiveAuditRow> = ledger
        .positions
        .iter()
        .map(|pos| {
            let brief = pos
                .paper
                .decision_id
                .as_deref()
                .and_then(|id| by_id.get(id).copied());
            let profile = profiles
                .get(&(pos.paper.ticker.clone(), pos.paper.direction.to_ascii_uppercase()))
                .cloned();
            audit_row_from_position(pos, brief, profile)
        })
        .collect();
    rows.sort_by(|a, b| {
        a.date
            .cmp(&b.date)
            .then(a.symbol.cmp(&b.symbol))
            .then(a.decision_id.cmp(&b.decision_id))
    });
    rows
}

fn count_by<F>(rows: &[&DeferredLiveAuditRow], key: F) -> Vec<CountKey>
where
    F: Fn(&DeferredLiveAuditRow) -> String,
{
    let mut map: BTreeMap<String, usize> = BTreeMap::new();
    for row in rows {
        *map.entry(key(row)).or_insert(0) += 1;
    }
    let mut out: Vec<CountKey> = map
        .into_iter()
        .map(|(key, n)| CountKey { key, n })
        .collect();
    out.sort_by(|a, b| b.n.cmp(&a.n).then(a.key.cmp(&b.key)));
    out
}

/// Equal-weight descriptive book. Realized losses are closed `ret < 0` only.
/// OPEN negative marks are counted separately and are not treated as exits.
pub fn summarize_universe_audit(
    rows: &[DeferredLiveAuditRow],
    sessions: usize,
    symbols_cached: usize,
    act_decisions: usize,
) -> UniverseAuditSummary {
    let mut n_target = 0usize;
    let mut n_stop = 0usize;
    let mut n_horizon = 0usize;
    let mut n_open = 0usize;
    let mut closed = Vec::new();
    let mut open_marks = Vec::new();
    let mut slips = Vec::new();
    let mut book = Vec::new();
    let mut losses: Vec<&DeferredLiveAuditRow> = Vec::new();
    let mut open_underwater = 0usize;
    let mut symbols = std::collections::BTreeSet::new();

    for row in rows {
        symbols.insert(row.symbol.as_str());
        book.push(row.realized_pct.or(row.mark_at_tape_end).unwrap_or(0.0));
        if let Some(s) = row.fill_vs_decision {
            slips.push(s);
        }
        match row.exit_reason.as_str() {
            "TARGET" => n_target += 1,
            "STOP" => n_stop += 1,
            "HORIZON" => n_horizon += 1,
            "OPEN" => n_open += 1,
            _ => {}
        }
        if row.exit_reason == "OPEN" {
            if let Some(m) = row.mark_at_tape_end {
                open_marks.push(m);
                if m < 0.0 {
                    open_underwater += 1;
                }
            }
        } else if let Some(ret) = row.realized_pct {
            closed.push(ret);
            if ret < 0.0 {
                losses.push(row);
            }
        }
    }

    let n_win = closed.iter().filter(|r| **r > 0.0).count();
    let n_loss = losses.len();
    let closed_count = closed.len();
    let mut largest: Vec<DeferredLiveAuditRow> = losses.iter().map(|r| (*r).clone()).collect();
    largest.sort_by(|a, b| {
        a.realized_pct
            .unwrap_or(0.0)
            .partial_cmp(&b.realized_pct.unwrap_or(0.0))
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(a.symbol.cmp(&b.symbol))
    });
    largest.truncate(10);

    UniverseAuditSummary {
        sessions,
        symbols_cached,
        symbols_entered: symbols.len(),
        act_decisions,
        paper_entries: rows.len(),
        n_target,
        n_stop,
        n_horizon,
        n_open,
        n_win,
        n_loss,
        mean_return: mean(&book),
        total_return: if book.is_empty() {
            None
        } else {
            Some(book.iter().sum())
        },
        mean_closed_return: mean(&closed),
        mean_open_mark: mean(&open_marks),
        mean_fill_vs_decision: mean(&slips),
        losses: LossBreakdown {
            loss_count: n_loss,
            closed_count,
            loss_rate: if closed_count == 0 {
                None
            } else {
                Some(n_loss as f64 / closed_count as f64)
            },
            open_underwater,
            worst_symbols: count_by(&losses, |r| r.symbol.clone()),
            worst_directions: count_by(&losses, |r| r.direction.clone()),
            worst_exit_reasons: count_by(&losses, |r| r.exit_reason.clone()),
            largest_individual_losses: largest,
        },
        note: "Descriptive Deferred Live paper observation across cached 1m dates. \
               Realized losses are closed TARGET/STOP/HORIZON with ret < 0. \
               OPEN is mark-to-last-observation, not an exit. \
               Frozen path unchanged. Not a G-GATE claim. Not paper_trader_v2 replay."
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

    fn brief(id: &str, ticker: &str, action: &str, entry: f64, target: f64, risk: f64) -> DecisionBrief {
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
            entry_action: action.into(),
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
    fn open_negative_mark_is_not_a_realized_loss() {
        let b = brief("d1", "AAA_NS", "ACT", 473.0, 450.0, 490.0);
        let mut rt = DeferredLiveRuntime::new(DeferredLiveConfig { horizon_secs: 10_000, strict_t0_admission: false });
        rt.arm(b.clone());
        rt.ingest(MarketObservation::last("AAA_NS", 1_000, 480.5));
        rt.ingest(MarketObservation::last("AAA_NS", 1_060, 485.0));
        let rows = audit_rows_from_ledger(rt.ledger(), &[b], &BTreeMap::new());
        let summary = summarize_universe_audit(&rows, 1, 1, 1);
        assert_eq!(summary.n_open, 1);
        assert_eq!(summary.n_loss, 0);
        assert_eq!(summary.losses.closed_count, 0);
        assert_eq!(summary.losses.open_underwater, 1);
        assert!(rows[0].realized_pct.is_none());
        assert!(rows[0].mark_at_tape_end.unwrap() < 0.0);
        assert_eq!(rows[0].session_state, "ACT");
        assert_eq!(rows[0].paper_fill, 480.5);
        assert_eq!(rows[0].decision_price, Some(473.0));
    }

    #[test]
    fn stop_with_negative_return_is_a_realized_loss() {
        let b = brief("d1", "BBB_NS", "ACT", 473.0, 450.0, 482.0);
        let mut rt = DeferredLiveRuntime::new(DeferredLiveConfig { horizon_secs: 10_000, strict_t0_admission: false });
        rt.arm(b.clone());
        rt.ingest(MarketObservation::last("BBB_NS", 1_000, 480.5));
        rt.ingest(MarketObservation {
            ticker: "BBB_NS".into(),
            unix: 1_060,
            price: 483.0,
            high: Some(483.0),
            low: Some(480.0),
        });
        let rows = audit_rows_from_ledger(rt.ledger(), &[b], &BTreeMap::new());
        let summary = summarize_universe_audit(&rows, 1, 1, 1);
        assert_eq!(rows[0].exit_reason, "STOP");
        assert_eq!(summary.n_stop, 1);
        assert_eq!(summary.n_loss, 1);
        assert_eq!(summary.losses.loss_rate, Some(1.0));
        assert_eq!(summary.losses.worst_exit_reasons[0].key, "STOP");
        assert_eq!(summary.losses.worst_symbols[0].key, "BBB");
        assert!(rows[0].realized_pct.unwrap() < 0.0);
        assert!(rows[0].mae <= 0.0);
    }

    #[test]
    fn target_is_a_win_not_a_loss() {
        let b = brief("d1", "CCC_NS", "ACT", 473.0, 475.0, 490.0);
        let mut rt = DeferredLiveRuntime::new(DeferredLiveConfig::default());
        rt.arm(b.clone());
        rt.ingest(MarketObservation::last("CCC_NS", 1_000, 480.5));
        rt.ingest(MarketObservation {
            ticker: "CCC_NS".into(),
            unix: 1_060,
            price: 474.8,
            high: Some(476.0),
            low: Some(474.5),
        });
        let rows = audit_rows_from_ledger(rt.ledger(), &[b], &BTreeMap::new());
        let summary = summarize_universe_audit(&rows, 1, 1, 1);
        assert_eq!(rows[0].exit_reason, "TARGET");
        assert_eq!(summary.n_target, 1);
        assert_eq!(summary.n_win, 1);
        assert_eq!(summary.n_loss, 0);
        assert!(rows[0].realized_pct.unwrap() > 0.0);
    }
}
