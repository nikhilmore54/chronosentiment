//! Fill displacement vs frozen T0 geometry — observation only.
//!
//! Does not rebase `adaptive_risk` / `adaptive_target` onto the paper fill.
//! Does not mutate the Deferred Live driver. Not a G-GATE claim.
//!
//! Question: how much of each ACT name's frozen stop envelope is consumed
//! by `paper_entry_price` versus `DecisionBrief.entry_price`?

use serde::{Deserialize, Serialize};

use super::deferred_live_loss_audit::DeferredLiveAuditRow;
use super::paper_lifecycle::signed_return;

/// Signed fill vs T0 decision, plus remaining frozen stop/target from each price.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FillGeometry {
    pub fill_vs_decision: f64,
    /// `(risk − decision) / decision` for SHORT; `(decision − risk) / decision` for LONG.
    pub t0_stop_room: f64,
    /// Same construction from the paper fill. Not a new stop.
    pub stop_room_from_fill: f64,
    /// `(fill − decision) / (risk − decision)`. >1 means fill is through T0 risk.
    pub stop_span_consumed: f64,
    pub t0_target_from_decision: f64,
    pub target_from_fill: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FillDisplacementBucket {
    /// `fill_vs_decision >= 0`
    Favorable,
    /// `-1% <= fill_vs_decision < 0`
    MildAdverse,
    /// `-2% <= fill_vs_decision < -1%`
    Adverse,
    /// `fill_vs_decision < -2%`
    SevereAdverse,
}

impl FillDisplacementBucket {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Favorable => "favorable (>= 0)",
            Self::MildAdverse => "mild adverse [-1%, 0)",
            Self::Adverse => "adverse [-2%, -1%)",
            Self::SevereAdverse => "severe adverse (< -2%)",
        }
    }

    pub fn from_fill_vs_decision(x: f64) -> Self {
        if x >= 0.0 {
            Self::Favorable
        } else if x >= -0.01 {
            Self::MildAdverse
        } else if x >= -0.02 {
            Self::Adverse
        } else {
            Self::SevereAdverse
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StopCompressionBucket {
    /// Fill moved away from frozen risk (consumed < 0).
    FillAwayFromStop,
    /// 0 <= consumed < 50% of T0 decision→risk span.
    UnderHalf,
    /// 50% <= consumed < 90%.
    Majority,
    /// consumed >= 90%.
    Exhausted,
}

impl StopCompressionBucket {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::FillAwayFromStop => "fill away from stop (< 0)",
            Self::UnderHalf => "consumed 0–50%",
            Self::Majority => "consumed 50–90%",
            Self::Exhausted => "consumed >= 90%",
        }
    }

    pub fn from_consumed(x: f64) -> Self {
        if x < 0.0 {
            Self::FillAwayFromStop
        } else if x < 0.50 {
            Self::UnderHalf
        } else if x < 0.90 {
            Self::Majority
        } else {
            Self::Exhausted
        }
    }
}

fn stop_room(direction: &str, px: f64, risk: f64) -> Option<f64> {
    if !px.is_finite() || px == 0.0 || !risk.is_finite() {
        return None;
    }
    if direction.eq_ignore_ascii_case("SHORT") {
        Some((risk - px) / px)
    } else {
        Some((px - risk) / px)
    }
}

/// Frozen T0 geometry viewed from the decision price and from the paper fill.
pub fn fill_geometry(
    direction: &str,
    decision: f64,
    fill: f64,
    risk: f64,
    target: f64,
) -> Option<FillGeometry> {
    if !decision.is_finite()
        || decision == 0.0
        || !fill.is_finite()
        || fill == 0.0
        || !risk.is_finite()
        || !target.is_finite()
        || (risk - decision).abs() < f64::EPSILON
    {
        return None;
    }
    Some(FillGeometry {
        fill_vs_decision: signed_return(direction, decision, fill),
        t0_stop_room: stop_room(direction, decision, risk)?,
        stop_room_from_fill: stop_room(direction, fill, risk)?,
        stop_span_consumed: (fill - decision) / (risk - decision),
        t0_target_from_decision: signed_return(direction, decision, target),
        target_from_fill: signed_return(direction, fill, target),
    })
}

pub fn fill_geometry_from_row(row: &DeferredLiveAuditRow) -> Option<FillGeometry> {
    fill_geometry(
        &row.direction,
        row.decision_price?,
        row.paper_fill,
        row.risk?,
        row.target?,
    )
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FillDisplacementRow {
    pub symbol: String,
    pub date: String,
    pub direction: String,
    pub oqs: Option<u32>,
    pub exit_reason: String,
    pub decision_price: f64,
    pub paper_fill: f64,
    pub risk: f64,
    pub target: f64,
    pub geometry: FillGeometry,
    pub fill_bucket: FillDisplacementBucket,
    pub stop_bucket: StopCompressionBucket,
    pub outcome: f64,
}

impl FillDisplacementRow {
    pub fn from_audit_row(row: &DeferredLiveAuditRow) -> Option<Self> {
        let geometry = fill_geometry_from_row(row)?;
        let outcome = row.realized_pct.or(row.mark_at_tape_end)?;
        Some(Self {
            symbol: row.symbol.clone(),
            date: row.date.clone(),
            direction: row.direction.clone(),
            oqs: row.oqs,
            exit_reason: row.exit_reason.clone(),
            decision_price: row.decision_price?,
            paper_fill: row.paper_fill,
            risk: row.risk?,
            target: row.target?,
            fill_bucket: FillDisplacementBucket::from_fill_vs_decision(geometry.fill_vs_decision),
            stop_bucket: StopCompressionBucket::from_consumed(geometry.stop_span_consumed),
            geometry,
            outcome,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BucketStats {
    pub bucket: String,
    pub n: usize,
    pub n_stop: usize,
    pub mean_fill_vs_decision: Option<f64>,
    pub mean_stop_span_consumed: Option<f64>,
    pub mean_stop_room_from_fill: Option<f64>,
    pub mean_outcome: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FillDisplacementSummary {
    pub n: usize,
    pub n_stop: usize,
    pub n_severe_adverse: usize,
    pub n_stop_in_severe_adverse: usize,
    pub n_majority_or_exhausted: usize,
    pub n_stop_in_majority_or_exhausted: usize,
    pub fill_buckets: Vec<BucketStats>,
    pub stop_buckets: Vec<BucketStats>,
    pub note: String,
}

fn mean(xs: &[f64]) -> Option<f64> {
    if xs.is_empty() {
        None
    } else {
        Some(xs.iter().sum::<f64>() / xs.len() as f64)
    }
}

fn stats_for<'a, F>(rows: &'a [FillDisplacementRow], label: &str, pred: F) -> BucketStats
where
    F: Fn(&FillDisplacementRow) -> bool,
{
    let hit: Vec<&FillDisplacementRow> = rows.iter().filter(|r| pred(r)).collect();
    BucketStats {
        bucket: label.into(),
        n: hit.len(),
        n_stop: hit.iter().filter(|r| r.exit_reason == "STOP").count(),
        mean_fill_vs_decision: mean(
            &hit.iter()
                .map(|r| r.geometry.fill_vs_decision)
                .collect::<Vec<_>>(),
        ),
        mean_stop_span_consumed: mean(
            &hit.iter()
                .map(|r| r.geometry.stop_span_consumed)
                .collect::<Vec<_>>(),
        ),
        mean_stop_room_from_fill: mean(
            &hit.iter()
                .map(|r| r.geometry.stop_room_from_fill)
                .collect::<Vec<_>>(),
        ),
        mean_outcome: mean(&hit.iter().map(|r| r.outcome).collect::<Vec<_>>()),
    }
}

pub fn summarize_fill_displacement(rows: &[FillDisplacementRow]) -> FillDisplacementSummary {
    let n_stop = rows.iter().filter(|r| r.exit_reason == "STOP").count();
    let severe: Vec<&FillDisplacementRow> = rows
        .iter()
        .filter(|r| r.fill_bucket == FillDisplacementBucket::SevereAdverse)
        .collect();
    let compressed: Vec<&FillDisplacementRow> = rows
        .iter()
        .filter(|r| {
            matches!(
                r.stop_bucket,
                StopCompressionBucket::Majority | StopCompressionBucket::Exhausted
            )
        })
        .collect();
    FillDisplacementSummary {
        n: rows.len(),
        n_stop,
        n_severe_adverse: severe.len(),
        n_stop_in_severe_adverse: severe.iter().filter(|r| r.exit_reason == "STOP").count(),
        n_majority_or_exhausted: compressed.len(),
        n_stop_in_majority_or_exhausted: compressed
            .iter()
            .filter(|r| r.exit_reason == "STOP")
            .count(),
        fill_buckets: vec![
            stats_for(rows, FillDisplacementBucket::Favorable.as_str(), |r| {
                r.fill_bucket == FillDisplacementBucket::Favorable
            }),
            stats_for(rows, FillDisplacementBucket::MildAdverse.as_str(), |r| {
                r.fill_bucket == FillDisplacementBucket::MildAdverse
            }),
            stats_for(rows, FillDisplacementBucket::Adverse.as_str(), |r| {
                r.fill_bucket == FillDisplacementBucket::Adverse
            }),
            stats_for(
                rows,
                FillDisplacementBucket::SevereAdverse.as_str(),
                |r| r.fill_bucket == FillDisplacementBucket::SevereAdverse,
            ),
        ],
        stop_buckets: vec![
            stats_for(rows, StopCompressionBucket::FillAwayFromStop.as_str(), |r| {
                r.stop_bucket == StopCompressionBucket::FillAwayFromStop
            }),
            stats_for(rows, StopCompressionBucket::UnderHalf.as_str(), |r| {
                r.stop_bucket == StopCompressionBucket::UnderHalf
            }),
            stats_for(rows, StopCompressionBucket::Majority.as_str(), |r| {
                r.stop_bucket == StopCompressionBucket::Majority
            }),
            stats_for(rows, StopCompressionBucket::Exhausted.as_str(), |r| {
                r.stop_bucket == StopCompressionBucket::Exhausted
            }),
        ],
        note: "Descriptive fill vs frozen T0 geometry. Does not rebase adaptive_risk. \
               OPEN outcomes are tape-end marks, not losses. Not a G-GATE claim."
            .into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tcs_short_fill_consumes_three_quarters_of_t0_stop_span() {
        let g = fill_geometry("SHORT", 2296.0, 2346.0, 2363.1046034906876, 2226.314340019983)
            .unwrap();
        assert!((g.fill_vs_decision - (-0.021777)).abs() < 1e-5);
        assert!((g.t0_stop_room - 0.029226).abs() < 1e-5);
        assert!((g.stop_room_from_fill - 0.007291).abs() < 1e-5);
        assert!((g.stop_span_consumed - 0.7450).abs() < 1e-3);
        assert_eq!(
            FillDisplacementBucket::from_fill_vs_decision(g.fill_vs_decision),
            FillDisplacementBucket::SevereAdverse
        );
        assert_eq!(
            StopCompressionBucket::from_consumed(g.stop_span_consumed),
            StopCompressionBucket::Majority
        );
    }

    #[test]
    fn pidilitind_short_fill_exhausts_t0_stop_span() {
        let g = fill_geometry("SHORT", 1589.699951171875, 1625.5, 1627.5069246718867, 1521.0094857036993)
            .unwrap();
        assert!((g.fill_vs_decision - (-0.02252)).abs() < 1e-4);
        assert!((g.stop_room_from_fill - 0.001235).abs() < 1e-5);
        assert!((g.stop_span_consumed - 0.947).abs() < 1e-3);
        assert_eq!(
            FillDisplacementBucket::from_fill_vs_decision(g.fill_vs_decision),
            FillDisplacementBucket::SevereAdverse
        );
        assert_eq!(
            StopCompressionBucket::from_consumed(g.stop_span_consumed),
            StopCompressionBucket::Exhausted
        );
    }

    #[test]
    fn favorable_short_fill_moves_away_from_stop() {
        let g = fill_geometry("SHORT", 100.0, 99.0, 103.0, 95.0).unwrap();
        assert!(g.fill_vs_decision > 0.0);
        assert!(g.stop_span_consumed < 0.0);
        assert_eq!(
            FillDisplacementBucket::from_fill_vs_decision(g.fill_vs_decision),
            FillDisplacementBucket::Favorable
        );
        assert_eq!(
            StopCompressionBucket::from_consumed(g.stop_span_consumed),
            StopCompressionBucket::FillAwayFromStop
        );
    }

    #[test]
    fn long_fill_toward_stop_is_positive_consumption() {
        let g = fill_geometry("LONG", 100.0, 99.0, 95.0, 104.0).unwrap();
        assert!(g.fill_vs_decision < 0.0);
        assert!(g.stop_span_consumed > 0.0);
        assert!(g.stop_room_from_fill < g.t0_stop_room);
    }

    #[test]
    fn both_realized_stops_land_in_severe_adverse_and_compressed_stop() {
        let tcs = FillDisplacementRow {
            symbol: "TCS".into(),
            date: "2026-09-04".into(),
            direction: "SHORT".into(),
            oqs: Some(80),
            exit_reason: "STOP".into(),
            decision_price: 2296.0,
            paper_fill: 2346.0,
            risk: 2363.10,
            target: 2226.31,
            geometry: fill_geometry("SHORT", 2296.0, 2346.0, 2363.10, 2226.31).unwrap(),
            fill_bucket: FillDisplacementBucket::SevereAdverse,
            stop_bucket: StopCompressionBucket::Majority,
            outcome: -0.0073,
        };
        let pid = FillDisplacementRow {
            symbol: "PIDILITIND".into(),
            date: "2026-09-07".into(),
            direction: "SHORT".into(),
            oqs: Some(81),
            exit_reason: "STOP".into(),
            decision_price: 1589.70,
            paper_fill: 1625.50,
            risk: 1627.51,
            target: 1521.01,
            geometry: fill_geometry("SHORT", 1589.70, 1625.50, 1627.51, 1521.01).unwrap(),
            fill_bucket: FillDisplacementBucket::SevereAdverse,
            stop_bucket: StopCompressionBucket::Exhausted,
            outcome: -0.0012,
        };
        let open = FillDisplacementRow {
            symbol: "AAA".into(),
            date: "2026-09-03".into(),
            direction: "SHORT".into(),
            oqs: Some(50),
            exit_reason: "OPEN".into(),
            decision_price: 100.0,
            paper_fill: 100.1,
            risk: 103.0,
            target: 96.0,
            geometry: fill_geometry("SHORT", 100.0, 100.1, 103.0, 96.0).unwrap(),
            fill_bucket: FillDisplacementBucket::from_fill_vs_decision(
                fill_geometry("SHORT", 100.0, 100.1, 103.0, 96.0)
                    .unwrap()
                    .fill_vs_decision,
            ),
            stop_bucket: StopCompressionBucket::from_consumed(
                fill_geometry("SHORT", 100.0, 100.1, 103.0, 96.0)
                    .unwrap()
                    .stop_span_consumed,
            ),
            outcome: 0.01,
        };
        let summary = summarize_fill_displacement(&[tcs, pid, open]);
        assert_eq!(summary.n_stop, 2);
        assert_eq!(summary.n_stop_in_severe_adverse, 2);
        assert_eq!(summary.n_stop_in_majority_or_exhausted, 2);
        assert_eq!(summary.n_severe_adverse, 2);
    }
}
