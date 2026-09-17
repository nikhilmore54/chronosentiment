//! Snap-to-fill clock — observation only.
//!
//! Measures `DecisionBrief.execution.snap_unix` versus the first Deferred Live
//! paper fill time. Does not change TARGET / STOP / HORIZON, does not rebase
//! risk, and does not mutate CS-P-001-D.
//!
//! A negative lag means the paper fill occurred *before* the recorded snap
//! (clock inversion), not that the decision went stale while waiting to fill.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SnapFillOrder {
    /// `fill_unix < snap_unix`
    FillBeforeSnap,
    /// `fill_unix == snap_unix`
    SameUnix,
    /// `fill_unix > snap_unix` (the stale-after-decision case)
    FillAfterSnap,
}

impl SnapFillOrder {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::FillBeforeSnap => "FILL_BEFORE_SNAP",
            Self::SameUnix => "SAME_UNIX",
            Self::FillAfterSnap => "FILL_AFTER_SNAP",
        }
    }

    pub fn from_unix(snap_unix: i64, fill_unix: i64) -> Self {
        match fill_unix.cmp(&snap_unix) {
            std::cmp::Ordering::Less => Self::FillBeforeSnap,
            std::cmp::Ordering::Equal => Self::SameUnix,
            std::cmp::Ordering::Greater => Self::FillAfterSnap,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SnapFillLag {
    pub snap_unix: i64,
    pub fill_unix: i64,
    pub lag_secs: i64,
    pub order: SnapFillOrder,
    pub first_bar_matches_fill: bool,
}

/// `lag_secs = fill_unix − snap_unix`. Negative ⇒ fill before snap.
pub fn snap_fill_lag(snap_unix: i64, fill_unix: i64, first_bar_matches_fill: bool) -> SnapFillLag {
    SnapFillLag {
        snap_unix,
        fill_unix,
        lag_secs: fill_unix - snap_unix,
        order: SnapFillOrder::from_unix(snap_unix, fill_unix),
        first_bar_matches_fill,
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SnapFillRow {
    pub symbol: String,
    pub date: String,
    pub direction: String,
    pub exit_reason: String,
    pub decision_id: String,
    pub fill_vs_decision: Option<f64>,
    pub lag: SnapFillLag,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SnapFillSummary {
    pub n: usize,
    pub n_fill_before_snap: usize,
    pub n_same_unix: usize,
    pub n_fill_after_snap: usize,
    pub n_first_bar_matches_fill: usize,
    pub unique_lag_secs: Vec<i64>,
    pub lag_is_constant: bool,
    pub dose_response_identifiable: bool,
    pub note: String,
}

pub fn summarize_snap_fill(rows: &[SnapFillRow]) -> SnapFillSummary {
    let mut lags: Vec<i64> = rows.iter().map(|r| r.lag.lag_secs).collect();
    lags.sort_unstable();
    lags.dedup();
    let n_before = rows
        .iter()
        .filter(|r| r.lag.order == SnapFillOrder::FillBeforeSnap)
        .count();
    let n_same = rows
        .iter()
        .filter(|r| r.lag.order == SnapFillOrder::SameUnix)
        .count();
    let n_after = rows
        .iter()
        .filter(|r| r.lag.order == SnapFillOrder::FillAfterSnap)
        .count();
    let n_match = rows.iter().filter(|r| r.lag.first_bar_matches_fill).count();
    let lag_is_constant = lags.len() <= 1;
    SnapFillSummary {
        n: rows.len(),
        n_fill_before_snap: n_before,
        n_same_unix: n_same,
        n_fill_after_snap: n_after,
        n_first_bar_matches_fill: n_match,
        unique_lag_secs: lags,
        lag_is_constant,
        dose_response_identifiable: !lag_is_constant && rows.len() >= 2,
        note: "Descriptive snap-to-fill clock. Negative lag is fill-before-snap, \
               not post-decision staleness. Constant lag cannot identify a \
               lag→displacement dose response. Not a G-GATE claim. CS-P-001-D frozen."
            .into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ist_open_versus_close_snap_is_fill_before_snap() {
        // 2026-09-07 09:15 IST vs 15:30 IST = −6.25h.
        let open = 1_788_752_700;
        let close = 1_788_775_200;
        let lag = snap_fill_lag(close, open, true);
        assert_eq!(lag.order, SnapFillOrder::FillBeforeSnap);
        assert_eq!(lag.lag_secs, -22_500);
        assert_eq!(lag.lag_secs, -6 * 3600 - 15 * 60);
    }

    #[test]
    fn fill_after_snap_is_the_stale_waiting_case() {
        let lag = snap_fill_lag(1_000, 1_000 + 600, true);
        assert_eq!(lag.order, SnapFillOrder::FillAfterSnap);
        assert_eq!(lag.lag_secs, 600);
    }

    #[test]
    fn constant_negative_lag_cannot_identify_dose_response() {
        let mk = |sym: &str, fill_vs: f64| SnapFillRow {
            symbol: sym.into(),
            date: "2026-09-07".into(),
            direction: "SHORT".into(),
            exit_reason: "OPEN".into(),
            decision_id: format!("id-{sym}"),
            fill_vs_decision: Some(fill_vs),
            lag: snap_fill_lag(1_788_775_200, 1_788_752_700, true),
        };
        let summary = summarize_snap_fill(&[
            mk("TCS", -0.0218),
            mk("HCLTECH", -0.0335),
            mk("IDEA", 0.0149),
        ]);
        assert_eq!(summary.n, 3);
        assert_eq!(summary.n_fill_before_snap, 3);
        assert_eq!(summary.n_fill_after_snap, 0);
        assert!(summary.lag_is_constant);
        assert!(!summary.dose_response_identifiable);
        assert_eq!(summary.unique_lag_secs, vec![-22_500]);
    }
}
