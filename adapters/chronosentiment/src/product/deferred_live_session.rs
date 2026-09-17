//! Session auto-arm for Deferred Live.
//!
//! Arms IC v1 `ACT` DecisionBriefs for one trading date. Does not decide
//! TARGET / STOP / HORIZON. Does not replay `paper_trader_v2_*.csv`.
//! `MONITOR` / `AVOID` stay unarmed (`NO TRADE` is first-class).
//!
//! See `docs/CS-P-001_DECISION_SUPPORT_PRODUCT_MODE.md` Stage B.

use serde::{Deserialize, Serialize};

use super::intraday_decision::DecisionBrief;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArmSkipReason {
    NotAct,
    MissingGeometry,
    MissingId,
}

/// ACT + finite target/risk. Geometry is required because the driver
/// refuses `open_from_brief` without it. Not a trading decision.
pub fn eligible_for_auto_arm(brief: &DecisionBrief) -> Result<(), ArmSkipReason> {
    if brief.id.trim().is_empty() || brief.ticker.trim().is_empty() {
        return Err(ArmSkipReason::MissingId);
    }
    if !brief.entry_action.eq_ignore_ascii_case("ACT") {
        return Err(ArmSkipReason::NotAct);
    }
    let target_ok = brief
        .execution
        .adaptive_target
        .filter(|v| v.is_finite() && *v > 0.0)
        .is_some();
    let risk_ok = brief
        .execution
        .adaptive_risk
        .filter(|v| v.is_finite() && *v > 0.0)
        .is_some();
    if !target_ok || !risk_ok {
        return Err(ArmSkipReason::MissingGeometry);
    }
    Ok(())
}

/// One ACT brief per ticker on `date` (highest OQS, then decision_id).
/// Deterministic. Does not invent a DecisionBrief.
pub fn select_session_briefs(briefs: &[DecisionBrief], date: &str) -> Vec<DecisionBrief> {
    let mut eligible: Vec<DecisionBrief> = briefs
        .iter()
        .filter(|b| b.date == date && eligible_for_auto_arm(b).is_ok())
        .cloned()
        .collect();
    eligible.sort_by(|a, b| {
        a.ticker
            .cmp(&b.ticker)
            .then(b.oqs.cmp(&a.oqs))
            .then(a.id.cmp(&b.id))
    });
    let mut seen = std::collections::HashSet::new();
    let mut out = Vec::new();
    for brief in eligible {
        if seen.insert(brief.ticker.clone()) {
            out.push(brief);
        }
    }
    out
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionState {
    pub date: String,
    pub auto_arm: bool,
    pub on_date: usize,
    pub eligible: usize,
    pub armed_at_open: usize,
    pub skipped_not_act: usize,
    pub skipped_missing_geometry: usize,
    pub note: String,
}

impl SessionState {
    pub fn summarize(briefs: &[DecisionBrief], date: &str, armed_at_open: usize) -> Self {
        let on_date: Vec<&DecisionBrief> = briefs.iter().filter(|b| b.date == date).collect();
        let mut skipped_not_act = 0;
        let mut skipped_missing_geometry = 0;
        for b in &on_date {
            match eligible_for_auto_arm(b) {
                Err(ArmSkipReason::NotAct) => skipped_not_act += 1,
                Err(ArmSkipReason::MissingGeometry) => skipped_missing_geometry += 1,
                Err(ArmSkipReason::MissingId) => skipped_missing_geometry += 1,
                Ok(()) => {}
            }
        }
        let eligible = select_session_briefs(briefs, date).len();
        Self {
            date: date.to_string(),
            auto_arm: true,
            on_date: on_date.len(),
            eligible,
            armed_at_open,
            skipped_not_act,
            skipped_missing_geometry,
            note: format!(
                "Session {date} · auto-arm ACT DecisionBriefs ({armed_at_open} armed). \
                 MONITOR/AVOID are NO TRADE. Not a broker. Not paper_trader_v2 replay."
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::product::intraday_decision::ExecutionFacts;

    fn brief(id: &str, ticker: &str, date: &str, action: &str, oqs: u32, geom: bool) -> DecisionBrief {
        DecisionBrief {
            id: id.into(),
            ticker: ticker.into(),
            date: date.into(),
            direction: "SHORT".into(),
            oqs,
            h60_class: "WAIT".into(),
            reference_price: Some(100.0),
            entry_price: Some(99.0),
            execution: ExecutionFacts {
                snap_unix: Some(1),
                adaptive_target: if geom { Some(95.0) } else { None },
                adaptive_risk: if geom { Some(105.0) } else { None },
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
    fn monitor_and_avoid_are_not_auto_armed() {
        assert!(eligible_for_auto_arm(&brief("a", "AAA_NS", "2026-09-07", "MONITOR", 90, true)).is_err());
        assert!(eligible_for_auto_arm(&brief("b", "BBB_NS", "2026-09-07", "AVOID", 90, true)).is_err());
        assert!(eligible_for_auto_arm(&brief("c", "CCC_NS", "2026-09-07", "ACT", 90, true)).is_ok());
        assert!(eligible_for_auto_arm(&brief("d", "DDD_NS", "2026-09-07", "ACT", 90, false)).is_err());
    }

    #[test]
    fn session_picks_one_act_per_ticker_highest_oqs() {
        let briefs = vec![
            brief("low", "AAA_NS", "2026-09-07", "ACT", 40, true),
            brief("high", "AAA_NS", "2026-09-07", "ACT", 80, true),
            brief("mon", "BBB_NS", "2026-09-07", "MONITOR", 99, true),
            brief("ok", "CCC_NS", "2026-09-07", "ACT", 50, true),
            brief("other-day", "DDD_NS", "2026-09-04", "ACT", 99, true),
        ];
        let selected = select_session_briefs(&briefs, "2026-09-07");
        let ids: Vec<_> = selected.iter().map(|b| b.id.as_str()).collect();
        assert_eq!(ids, ["high", "ok"]);
    }
}
