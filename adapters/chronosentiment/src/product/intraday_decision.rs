//! Intraday decision brief — adapter-owned assembly of existing contracts.
//!
//! This module flattens:
//! - frozen IC v1 classification (`classify_at_entry`, `reassess_at_h120`, `resolve_action`)
//! - Intelligence Contract v1 §7 historical performance reference
//! - existing source-record facts, including `reference_price` and `entry_price`
//! - LIVE-005 / TIME009 T0 geometry (`adaptive_target`, `adaptive_risk`, `snap_unix`)
//!
//! Price fields are copied from the source record (or the TIME009 T0 overlay).
//! They are never synthesized, aliased, or substituted for each other.
//! Live last-tick / current_price are not invented when no live quote exists.
//!
//! See `docs/INTELLIGENCE_CONTRACT_V1.md`.

use crate::reasoning::intraday_classification::{
    classify_at_entry, reassess_at_h120, resolve_action, Checkpoint, Direction, EntryInput,
    H120Input, H60Classification, OpportunityState,
};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

// ── Dataset record (mirrors p4_opportunity_dataset.json / LIVE ledger fields) ─

#[derive(Debug, Deserialize, Clone)]
pub struct RawOpportunityRecord {
    pub decision_id: String,
    pub cohort_date: String,
    pub ticker: String,
    pub direction: String,
    pub opportunity_dimensions: OpportunityDimensions,
    pub path_5m: Path5m,
    pub daily_outcome: Option<String>,
    pub daily_return: Option<f64>,
    /// T0 LTP / previous close. Adapter + LIVE + DecisionCore field. Never invented.
    #[serde(default)]
    pub reference_price: Option<f64>,
    /// Decision-time snapshot unix. Source-record `snap_unix`. Never invented.
    #[serde(default)]
    pub snap_unix: Option<i64>,
    /// LIVE-005 T0 target price. Optional on the golden dataset; TIME009 overlay fills it.
    #[serde(default)]
    pub adaptive_target: Option<f64>,
    /// LIVE-005 T0 risk boundary. Optional on the golden dataset; TIME009 overlay fills it.
    #[serde(default)]
    pub adaptive_risk: Option<f64>,
    /// LIVE-005 T0 horizon in sessions. Optional; TIME009 overlay fills it.
    #[serde(default)]
    pub adaptive_horizon_sessions: Option<f64>,
    /// TIME009 observation fact. `None` if the overlay is absent.
    #[serde(default)]
    pub horizon_elapsed: Option<bool>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct OpportunityDimensions {
    pub opportunity_quality_score: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Path5m {
    pub h60_classification: String,
    pub h15_ret: Option<f64>,
    pub h30_ret: Option<f64>,
    pub h60_ret: Option<f64>,
    pub h120_ret: Option<f64>,
    pub h180_ret: Option<f64>,
    pub h300_ret: Option<f64>,
    pub mfe_h60: Option<f64>,
    #[serde(default)]
    pub mfe_h120: Option<f64>,
    /// Existing source-record field (`path_5m.entry_price`). Never invented.
    #[serde(default)]
    pub entry_price: Option<f64>,
}

/// Adapter-owned execution facts. Projected by HTTP; never recalculated in the UI.
///
/// `current_price` / `last_tick_unix` stay `None` until a live quote exists.
/// Distances are computed from T0 geometry vs `reference_price` / `entry_price`.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ExecutionFacts {
    pub snap_unix: Option<i64>,
    pub adaptive_target: Option<f64>,
    pub adaptive_risk: Option<f64>,
    pub adaptive_horizon_sessions: Option<f64>,
    pub target_distance_abs: Option<f64>,
    pub target_distance_pct: Option<f64>,
    pub risk_distance_abs: Option<f64>,
    pub risk_distance_pct: Option<f64>,
    /// `(adaptive_target - entry_price) / entry_price`. `None` if either is missing.
    pub expected_move_pct: Option<f64>,
    pub current_price: Option<f64>,
    pub last_tick_unix: Option<i64>,
    /// `LIVE` only when `last_tick_unix` is present. Otherwise `STALE`.
    pub freshness: String,
    pub horizon_elapsed: Option<bool>,
}

/// Adapter-owned brief for the intradaily Decision Feed.
/// HTTP serializes this type; it must not redefine it.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct DecisionBrief {
    pub id: String,
    pub ticker: String,
    pub date: String,
    pub direction: String,
    pub oqs: u32,
    pub h60_class: String,

    /// Source-record `reference_price`. `None` if the source record lacked it.
    pub reference_price: Option<f64>,
    /// Source-record `path_5m.entry_price`. `None` if the source record lacked it.
    pub entry_price: Option<f64>,
    pub execution: ExecutionFacts,

    pub entry_state: String,
    pub entry_action: String,
    pub entry_confidence: String,
    pub entry_horizon: String,
    pub entry_why: String,
    pub entry_risk: String,

    pub h120_state: String,
    pub h120_action: String,
    pub h120_confidence: String,
    pub h120_horizon: String,
    pub h120_why: String,
    pub h120_risk: String,

    pub h15_ret: Option<f64>,
    pub h30_ret: Option<f64>,
    pub h60_ret: Option<f64>,
    pub h120_ret: Option<f64>,
    pub h180_ret: Option<f64>,
    pub h300_ret: Option<f64>,
    pub mfe_h60: Option<f64>,
    pub mfe_h120: Option<f64>,

    pub outcome: Option<String>,
    pub pnl: Option<f64>,

    pub hist_win: Option<f64>,
    pub hist_pf: Option<f64>,
    pub hist_med: Option<f64>,
}

/// TIME009 T0 geometry only. Outcome fields are not consumed.
#[derive(Debug, Deserialize, Clone, Default)]
struct Time009Geometry {
    #[serde(default)]
    adaptive_target: Option<f64>,
    #[serde(default)]
    adaptive_risk: Option<f64>,
    #[serde(default)]
    adaptive_horizon_sessions: Option<f64>,
    #[serde(default)]
    source_snapshot_unix: Option<i64>,
    #[serde(default)]
    horizon_elapsed: Option<bool>,
}

fn hist_ref(direction: Direction, state: OpportunityState) -> (Option<f64>, Option<f64>, Option<f64>) {
    let dir = match direction {
        Direction::Long => "LONG",
        Direction::Short => "SHORT",
    };
    let key = format!("{}:{}", dir, state.label());
    match key.as_str() {
        "SHORT:ENTER"      => (Some(0.717), Some(13.35), Some(0.0076)),
        "SHORT:WAIT-HIGH"  => (Some(0.775), Some(99.0),  Some(0.0056)),
        "SHORT:WAIT-LOW"   => (Some(0.044), Some(0.04),  Some(-0.0030)),
        "SHORT:AVOID"      => (Some(0.261), Some(0.23),  Some(-0.0032)),
        "LONG:ENTER"       => (Some(0.750), Some(19.46), Some(0.0061)),
        "LONG:WAIT-HIGH"   => (Some(0.824), Some(72.36), Some(0.0066)),
        "LONG:WAIT-MID"    => (Some(0.595), Some(10.06), Some(0.0026)),
        "LONG:WAIT-LOW"    => (Some(0.283), Some(0.67),  Some(-0.0023)),
        "LONG:AVOID"       => (Some(0.116), Some(0.08),  Some(-0.0065)),
        "LONG:ENTER-LATE"  => (Some(0.696), Some(11.67), Some(0.0046)),
        "LONG:WAIT-LATE"   => (Some(0.500), Some(6.47),  Some(0.0018)),
        "LONG:AVOID-LATE"  => (Some(0.000), None,        Some(0.0002)),
        _ => (None, None, None),
    }
}

fn parse_direction(s: &str) -> Direction {
    match s.to_uppercase().as_str() {
        "SHORT" => Direction::Short,
        _ => Direction::Long,
    }
}

fn parse_h60_class(s: &str) -> H60Classification {
    match s.to_uppercase().as_str() {
        "ENTER" => H60Classification::Enter,
        "AVOID" => H60Classification::Avoid,
        _ => H60Classification::Wait,
    }
}

/// Signed distance from `from` to `to`. Percent is `None` when `from` is 0.
fn signed_distance(from: Option<f64>, to: Option<f64>) -> (Option<f64>, Option<f64>) {
    match (from, to) {
        (Some(a), Some(b)) if a != 0.0 => (Some(b - a), Some((b - a) / a)),
        (Some(a), Some(b)) => (Some(b - a), None),
        _ => (None, None),
    }
}

fn assemble_execution(record: &RawOpportunityRecord) -> ExecutionFacts {
    let (target_distance_abs, target_distance_pct) =
        signed_distance(record.reference_price, record.adaptive_target);
    let (risk_distance_abs, risk_distance_pct) =
        signed_distance(record.reference_price, record.adaptive_risk);
    let (_expected_abs, expected_move_pct) =
        signed_distance(record.path_5m.entry_price, record.adaptive_target);
    let last_tick_unix = None;
    let current_price = None;
    let freshness = if last_tick_unix.is_some() {
        "LIVE".to_string()
    } else {
        "STALE".to_string()
    };
    ExecutionFacts {
        snap_unix: record.snap_unix,
        adaptive_target: record.adaptive_target,
        adaptive_risk: record.adaptive_risk,
        adaptive_horizon_sessions: record.adaptive_horizon_sessions,
        target_distance_abs,
        target_distance_pct,
        risk_distance_abs,
        risk_distance_pct,
        expected_move_pct,
        current_price,
        last_tick_unix,
        freshness,
        horizon_elapsed: record.horizon_elapsed,
    }
}

/// Assemble a [`DecisionBrief`] from a source record using frozen IC v1.
///
/// Price and T0 geometry fields are copied from the source. They are never
/// synthesized and never substituted for each other.
pub fn assemble_intraday_brief(record: &RawOpportunityRecord) -> DecisionBrief {
    let direction = parse_direction(&record.direction);
    let oqs = record.opportunity_dimensions.opportunity_quality_score;
    let h60_class = parse_h60_class(&record.path_5m.h60_classification);

    let entry_input = EntryInput {
        direction,
        opportunity_quality_score: oqs,
        h60_classification: h60_class,
    };
    let entry_state = classify_at_entry(&entry_input);

    let h120_input = H120Input {
        h120_ret: record.path_5m.h120_ret,
        mfe_h120: record.path_5m.mfe_h120,
    };
    let h120_state = reassess_at_h120(direction, entry_state, &h120_input);

    let entry_action_result = resolve_action(Checkpoint::Entry, entry_state, None);
    let h120_action_result = resolve_action(Checkpoint::H120, h120_state, Some(entry_state));
    let (hist_win, hist_pf, hist_med) = hist_ref(direction, entry_state);

    DecisionBrief {
        id: record.decision_id.clone(),
        ticker: record.ticker.clone(),
        date: record.cohort_date.clone(),
        direction: record.direction.clone(),
        oqs,
        h60_class: record.path_5m.h60_classification.clone(),
        reference_price: record.reference_price,
        entry_price: record.path_5m.entry_price,
        execution: assemble_execution(record),
        entry_state: entry_state.label().to_string(),
        entry_action: entry_action_result.action.label().to_string(),
        entry_confidence: entry_action_result.confidence.label().to_string(),
        entry_horizon: entry_action_result.horizon.to_string(),
        entry_why: entry_action_result.why.to_string(),
        entry_risk: entry_action_result.risk.to_string(),
        h120_state: h120_state.label().to_string(),
        h120_action: h120_action_result.action.label().to_string(),
        h120_confidence: h120_action_result.confidence.label().to_string(),
        h120_horizon: h120_action_result.horizon.to_string(),
        h120_why: h120_action_result.why.to_string(),
        h120_risk: h120_action_result.risk.to_string(),
        h15_ret: record.path_5m.h15_ret,
        h30_ret: record.path_5m.h30_ret,
        h60_ret: record.path_5m.h60_ret,
        h120_ret: record.path_5m.h120_ret,
        h180_ret: record.path_5m.h180_ret,
        h300_ret: record.path_5m.h300_ret,
        mfe_h60: record.path_5m.mfe_h60,
        mfe_h120: record.path_5m.mfe_h120,
        outcome: record.daily_outcome.clone(),
        pnl: record.daily_return,
        hist_win,
        hist_pf,
        hist_med,
    }
}

fn time009_dir_from_dataset(dataset_path: &str) -> Option<PathBuf> {
    Path::new(dataset_path)
        .parent()?
        .parent()
        .map(|root| root.join("time_machine/analysis/TIME009/observations"))
}

fn overlay_time009_geometry(records: &mut [RawOpportunityRecord], obs_dir: &Path) {
    if !obs_dir.is_dir() {
        return;
    }
    for record in records.iter_mut() {
        if record.adaptive_target.is_some()
            && record.adaptive_risk.is_some()
            && record.adaptive_horizon_sessions.is_some()
            && record.snap_unix.is_some()
            && record.horizon_elapsed.is_some()
        {
            continue;
        }
        let path = obs_dir.join(format!("TIME009-OBS-{}.json", record.decision_id));
        let Ok(content) = std::fs::read_to_string(&path) else {
            continue;
        };
        let Ok(obs) = serde_json::from_str::<Time009Geometry>(&content) else {
            continue;
        };
        if record.adaptive_target.is_none() {
            record.adaptive_target = obs.adaptive_target;
        }
        if record.adaptive_risk.is_none() {
            record.adaptive_risk = obs.adaptive_risk;
        }
        if record.adaptive_horizon_sessions.is_none() {
            record.adaptive_horizon_sessions = obs.adaptive_horizon_sessions;
        }
        if record.snap_unix.is_none() {
            record.snap_unix = obs.source_snapshot_unix;
        }
        if record.horizon_elapsed.is_none() {
            record.horizon_elapsed = obs.horizon_elapsed;
        }
    }
}

pub fn load_intraday_briefs(dataset_path: &str) -> Result<Vec<DecisionBrief>, String> {
    let content = std::fs::read_to_string(dataset_path)
        .map_err(|e| format!("Cannot read dataset at {dataset_path}: {e}"))?;
    let mut records: Vec<RawOpportunityRecord> = serde_json::from_str(&content)
        .map_err(|e| format!("Cannot parse dataset: {e}"))?;
    if let Some(obs_dir) = time009_dir_from_dataset(dataset_path) {
        overlay_time009_geometry(&mut records, &obs_dir);
    }
    Ok(records.iter().map(assemble_intraday_brief).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_record() -> RawOpportunityRecord {
        RawOpportunityRecord {
            decision_id: "LIVE-005-20260825-1000-HDFCBANK_NS".into(),
            cohort_date: "2026-08-25".into(),
            ticker: "HDFCBANK_NS".into(),
            direction: "LONG".into(),
            opportunity_dimensions: OpportunityDimensions {
                opportunity_quality_score: 55,
            },
            path_5m: Path5m {
                h60_classification: "WAIT".into(),
                h15_ret: Some(0.00027),
                h30_ret: Some(0.00213),
                h60_ret: Some(-0.00068),
                h120_ret: Some(0.00075),
                h180_ret: Some(0.00247),
                h300_ret: Some(0.00247),
                mfe_h60: Some(0.00213),
                mfe_h120: None,
                entry_price: Some(726.2999877929688),
            },
            daily_outcome: Some("LOSS".into()),
            daily_return: Some(-0.00041),
            reference_price: Some(727.5),
            snap_unix: Some(1787652000),
            adaptive_target: Some(743.2505677517032),
            adaptive_risk: Some(709.8257668006238),
            adaptive_horizon_sessions: Some(4.0),
            horizon_elapsed: Some(true),
        }
    }

    #[test]
    fn prices_pass_through_unchanged_and_are_not_aliased() {
        let brief = assemble_intraday_brief(&sample_record());
        assert_eq!(brief.reference_price, Some(727.5));
        assert_eq!(brief.entry_price, Some(726.2999877929688));
        assert_ne!(brief.reference_price, brief.entry_price);
    }

    #[test]
    fn missing_prices_stay_none() {
        let mut record = sample_record();
        record.reference_price = None;
        record.path_5m.entry_price = None;
        let brief = assemble_intraday_brief(&record);
        assert_eq!(brief.reference_price, None);
        assert_eq!(brief.entry_price, None);
    }

    #[test]
    fn entry_price_is_not_filled_from_reference_price() {
        let mut record = sample_record();
        record.path_5m.entry_price = None;
        let brief = assemble_intraday_brief(&record);
        assert_eq!(brief.reference_price, Some(727.5));
        assert_eq!(brief.entry_price, None);
    }

    #[test]
    fn ic_v1_classification_is_unchanged() {
        let brief = assemble_intraday_brief(&sample_record());
        assert_eq!(brief.entry_state, "WAIT-MID");
        assert_eq!(brief.entry_action, "MONITOR");
        assert_eq!(brief.oqs, 55);
        assert_eq!(brief.h60_class, "WAIT");
    }

    #[test]
    fn source_json_deserializes_both_price_fields() {
        let json = r#"{
            "decision_id": "id-1",
            "cohort_date": "2026-08-25",
            "ticker": "HDFCBANK_NS",
            "direction": "LONG",
            "opportunity_dimensions": { "opportunity_quality_score": 55 },
            "path_5m": {
                "h60_classification": "WAIT",
                "h15_ret": null, "h30_ret": null, "h60_ret": null,
                "h120_ret": null, "h180_ret": null, "h300_ret": null,
                "mfe_h60": null,
                "entry_price": 726.2999877929688
            },
            "reference_price": 727.5
        }"#;
        let record: RawOpportunityRecord = serde_json::from_str(json).unwrap();
        let brief = assemble_intraday_brief(&record);
        assert_eq!(brief.reference_price, Some(727.5));
        assert_eq!(brief.entry_price, Some(726.2999877929688));
    }

    #[test]
    fn t0_geometry_passes_through_and_distances_use_reference_price() {
        let brief = assemble_intraday_brief(&sample_record());
        let ex = &brief.execution;
        assert_eq!(ex.snap_unix, Some(1787652000));
        assert_eq!(ex.adaptive_target, Some(743.2505677517032));
        assert_eq!(ex.adaptive_risk, Some(709.8257668006238));
        assert_eq!(ex.adaptive_horizon_sessions, Some(4.0));
        assert_eq!(ex.horizon_elapsed, Some(true));
        assert_eq!(ex.target_distance_abs, Some(743.2505677517032 - 727.5));
        assert_eq!(ex.target_distance_pct, Some((743.2505677517032 - 727.5) / 727.5));
        assert_eq!(ex.risk_distance_abs, Some(709.8257668006238 - 727.5));
        assert_eq!(ex.risk_distance_pct, Some((709.8257668006238 - 727.5) / 727.5));
        assert_eq!(
            ex.expected_move_pct,
            Some((743.2505677517032 - 726.2999877929688) / 726.2999877929688)
        );
    }

    #[test]
    fn current_price_and_last_tick_are_not_invented() {
        let brief = assemble_intraday_brief(&sample_record());
        assert_eq!(brief.execution.current_price, None);
        assert_eq!(brief.execution.last_tick_unix, None);
        assert_eq!(brief.execution.freshness, "STALE");
        assert_ne!(brief.execution.current_price, brief.entry_price);
        assert_ne!(brief.execution.current_price, brief.reference_price);
    }

    #[test]
    fn missing_geometry_stays_none_and_distances_are_not_filled() {
        let mut record = sample_record();
        record.adaptive_target = None;
        record.adaptive_risk = None;
        let brief = assemble_intraday_brief(&record);
        assert_eq!(brief.execution.adaptive_target, None);
        assert_eq!(brief.execution.adaptive_risk, None);
        assert_eq!(brief.execution.target_distance_pct, None);
        assert_eq!(brief.execution.risk_distance_pct, None);
        assert_eq!(brief.execution.expected_move_pct, None);
        assert_eq!(brief.execution.freshness, "STALE");
    }
}
