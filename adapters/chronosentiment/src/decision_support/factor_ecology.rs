//! CS-P-005 Factor Ecology Analysis v0.1.
//!
//! Discovery of Trend / Momentum / Volatility state structure at T.
//! Outcomes are attached as measurement only. This module does not choose a policy.

use std::collections::BTreeMap;

use chrono::{DateTime, Datelike, Utc};
use serde::Serialize;

use crate::decision_support::policy::{DecisionPolicy, TrendMappingPolicy};
use crate::decision_support::DecisionAction;
use crate::metrics::concepts::Concept;
use crate::reasoning::assessment::{AssessmentProfile, FactorAvailability};

#[derive(Debug, Clone, Serialize)]
pub struct EcologyRow {
    pub instrument: String,
    pub as_of: DateTime<Utc>,
    pub year: i32,
    pub trend_available: bool,
    pub trend: Option<String>,
    pub momentum_available: bool,
    pub momentum: Option<String>,
    pub roc_20: Option<f64>,
    pub volatility_available: bool,
    pub atr_14: Option<f64>,
    pub current_policy_action: String,
    pub outcome_5d: Option<f64>,
    pub outcome_10d: Option<f64>,
    pub outcome_20d: Option<f64>,
    pub outcome_60d: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct Quantiles {
    pub n: u32,
    pub min: Option<f64>,
    pub p25: Option<f64>,
    pub median: Option<f64>,
    pub p75: Option<f64>,
    pub max: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FactorEcologyReport {
    pub n_rows: u32,
    pub availability: BTreeMap<String, u32>,
    pub trend_states: BTreeMap<String, u32>,
    pub momentum_states: BTreeMap<String, u32>,
    pub current_policy_actions: BTreeMap<String, u32>,
    pub trend_x_momentum: BTreeMap<String, u32>,
    pub trend_x_volatility_available: BTreeMap<String, u32>,
    pub trend_x_momentum_x_vol: BTreeMap<String, u32>,
    pub by_instrument: BTreeMap<String, u32>,
    pub by_year: BTreeMap<i32, u32>,
    pub roc_20: Quantiles,
    pub atr_14: Quantiles,
    pub outcome_60d_by_trend: BTreeMap<String, Quantiles>,
    pub outcome_60d_by_joint: BTreeMap<String, Quantiles>,
    pub n_outcome_60d_available: u32,
    pub n_outcome_60d_unavailable: u32,
    pub design_constraints: Vec<String>,
}

pub fn state_key(row: &EcologyRow) -> String {
    format!(
        "Trend={} | Momentum={} | Vol={}",
        row.trend.as_deref().unwrap_or("UNAVAILABLE"),
        row.momentum.as_deref().unwrap_or("UNAVAILABLE"),
        if row.volatility_available {
            "AVAILABLE"
        } else {
            "UNAVAILABLE"
        }
    )
}

pub fn row_from_profile(
    profile: &AssessmentProfile,
    instrument: String,
    roc_20: Option<f64>,
    atr_14: Option<f64>,
) -> EcologyRow {
    let trend = profile
        .assessments
        .iter()
        .find(|a| a.concept == Concept::Trend)
        .map(|a| format!("{:?}", a.direction));
    let momentum = profile
        .assessments
        .iter()
        .find(|a| a.concept == Concept::Momentum)
        .map(|a| format!("{:?}", a.direction));
    let trend_available = profile
        .factor_status
        .iter()
        .find(|s| s.concept == Concept::Trend)
        .map(|s| s.availability == FactorAvailability::Available)
        .unwrap_or(trend.is_some());
    let momentum_available = profile
        .factor_status
        .iter()
        .find(|s| s.concept == Concept::Momentum)
        .map(|s| s.availability == FactorAvailability::Available)
        .unwrap_or(momentum.is_some());
    let volatility_available = profile
        .factor_status
        .iter()
        .find(|s| s.concept == Concept::Volatility)
        .map(|s| s.availability == FactorAvailability::Available)
        .unwrap_or(false);
    let action = TrendMappingPolicy.decide(profile, profile.metadata.evaluation_timestamp);
    let action = match action.action {
        DecisionAction::Long => "LONG",
        DecisionAction::Short => "SHORT",
        DecisionAction::NoTrade => "NO_TRADE",
    };
    EcologyRow {
        instrument,
        as_of: profile.metadata.evaluation_timestamp,
        year: profile.metadata.evaluation_timestamp.year(),
        trend_available,
        trend,
        momentum_available,
        momentum,
        roc_20,
        volatility_available,
        atr_14,
        current_policy_action: action.to_string(),
        outcome_5d: None,
        outcome_10d: None,
        outcome_20d: None,
        outcome_60d: None,
    }
}

pub fn analyze(rows: &[EcologyRow]) -> FactorEcologyReport {
    let mut availability = BTreeMap::new();
    let mut trend_states = BTreeMap::new();
    let mut momentum_states = BTreeMap::new();
    let mut current_policy_actions = BTreeMap::new();
    let mut trend_x_momentum = BTreeMap::new();
    let mut trend_x_volatility_available = BTreeMap::new();
    let mut trend_x_momentum_x_vol = BTreeMap::new();
    let mut by_instrument = BTreeMap::new();
    let mut by_year = BTreeMap::new();
    let mut roc = Vec::new();
    let mut atr = Vec::new();
    let mut outcome_60d_by_trend: BTreeMap<String, Vec<f64>> = BTreeMap::new();
    let mut outcome_60d_by_joint: BTreeMap<String, Vec<f64>> = BTreeMap::new();
    let mut n_outcome_60d_available = 0u32;
    let mut n_outcome_60d_unavailable = 0u32;

    for row in rows {
        *availability
            .entry(format!(
                "Trend={}",
                if row.trend_available {
                    "AVAILABLE"
                } else {
                    "UNAVAILABLE"
                }
            ))
            .or_insert(0) += 1;
        *availability
            .entry(format!(
                "Momentum={}",
                if row.momentum_available {
                    "AVAILABLE"
                } else {
                    "UNAVAILABLE"
                }
            ))
            .or_insert(0) += 1;
        *availability
            .entry(format!(
                "Volatility={}",
                if row.volatility_available {
                    "AVAILABLE"
                } else {
                    "UNAVAILABLE"
                }
            ))
            .or_insert(0) += 1;
        *trend_states
            .entry(row.trend.clone().unwrap_or_else(|| "UNAVAILABLE".into()))
            .or_insert(0) += 1;
        *momentum_states
            .entry(row.momentum.clone().unwrap_or_else(|| "UNAVAILABLE".into()))
            .or_insert(0) += 1;
        *current_policy_actions
            .entry(row.current_policy_action.clone())
            .or_insert(0) += 1;
        *trend_x_momentum
            .entry(format!(
                "{}×{}",
                row.trend.as_deref().unwrap_or("UNAVAILABLE"),
                row.momentum.as_deref().unwrap_or("UNAVAILABLE")
            ))
            .or_insert(0) += 1;
        *trend_x_volatility_available
            .entry(format!(
                "{}×Vol={}",
                row.trend.as_deref().unwrap_or("UNAVAILABLE"),
                if row.volatility_available {
                    "AVAILABLE"
                } else {
                    "UNAVAILABLE"
                }
            ))
            .or_insert(0) += 1;
        *trend_x_momentum_x_vol.entry(state_key(row)).or_insert(0) += 1;
        *by_instrument.entry(row.instrument.clone()).or_insert(0) += 1;
        *by_year.entry(row.year).or_insert(0) += 1;
        if let Some(v) = row.roc_20 {
            roc.push(v);
        }
        if let Some(v) = row.atr_14 {
            atr.push(v);
        }
        match row.outcome_60d {
            Some(v) => {
                n_outcome_60d_available += 1;
                outcome_60d_by_trend
                    .entry(row.trend.clone().unwrap_or_else(|| "UNAVAILABLE".into()))
                    .or_default()
                    .push(v);
                outcome_60d_by_joint
                    .entry(state_key(row))
                    .or_default()
                    .push(v);
            }
            None => n_outcome_60d_unavailable += 1,
        }
    }

    FactorEcologyReport {
        n_rows: rows.len() as u32,
        availability,
        trend_states,
        momentum_states,
        current_policy_actions,
        trend_x_momentum,
        trend_x_volatility_available,
        trend_x_momentum_x_vol,
        by_instrument,
        by_year,
        roc_20: quantiles(&roc),
        atr_14: quantiles(&atr),
        outcome_60d_by_trend: outcome_60d_by_trend
            .into_iter()
            .map(|(k, v)| (k, quantiles(&v)))
            .collect(),
        outcome_60d_by_joint: outcome_60d_by_joint
            .into_iter()
            .map(|(k, v)| (k, quantiles(&v)))
            .collect(),
        n_outcome_60d_available,
        n_outcome_60d_unavailable,
        design_constraints: design_constraints(rows),
    }
}

fn design_constraints(rows: &[EcologyRow]) -> Vec<String> {
    let mut out = vec![
        "Specify the candidate policy before evaluating it. Do not search thresholds on these outcomes.".into(),
        "TrendMappingPolicy remains the live default until a candidate is frozen as a new version.".into(),
        "NO_TRADE must be an explicit confluence miss, not an accident of missing Trend.".into(),
        "Volatility may be used only as a magnitude available at T (atr_14). Do not invent High/Low.".into(),
        "atr_14 is in price units and is not comparable across instruments; do not use a global ATR cutoff.".into(),
        "Outcomes in this report are measurement, not decision inputs.".into(),
        "Do not freeze Decision Engine v1.0 from this analysis.".into(),
    ];
    let joints: std::collections::BTreeSet<_> = rows.iter().map(state_key).collect();
    out.push(format!(
        "Observed distinct Trend×Momentum×Vol states: {}.",
        joints.len()
    ));
    let no_trade = rows
        .iter()
        .filter(|r| r.current_policy_action == "NO_TRADE")
        .count();
    out.push(format!(
        "Current TrendMappingPolicy NO_TRADE count on this snapshot: {no_trade}/{} (descriptive).",
        rows.len()
    ));
    out
}

fn quantiles(values: &[f64]) -> Quantiles {
    let mut v: Vec<f64> = values.iter().copied().filter(|x| x.is_finite()).collect();
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    if v.is_empty() {
        return Quantiles::default();
    }
    let pct = |p: f64| {
        let idx = ((p * (v.len() as f64 - 1.0)).round() as usize).min(v.len() - 1);
        v[idx]
    };
    Quantiles {
        n: v.len() as u32,
        min: Some(v[0]),
        p25: Some(pct(0.25)),
        median: Some(pct(0.50)),
        p75: Some(pct(0.75)),
        max: Some(v[v.len() - 1]),
    }
}

pub fn render_ecology(report: &FactorEcologyReport) -> String {
    let mut md = String::from("# CS-P-005 Factor Ecology Analysis v0.1\n\n");
    md.push_str("Information-state discovery. **Not a trading recommendation.** Not B5. Not G-GATE. Not Decision Engine v1.0.\n\n");
    md.push_str(&format!("Rows: {}\n\n", report.n_rows));
    md.push_str("## Availability\n\n");
    for (k, n) in &report.availability {
        md.push_str(&format!("- {k}: {n}\n"));
    }
    md.push_str("\n## Current TrendMappingPolicy actions (descriptive, not a candidate)\n\n");
    for (k, n) in &report.current_policy_actions {
        md.push_str(&format!("- {k}: {n}\n"));
    }
    md.push_str("\n## Trend × Momentum\n\n");
    md.push_str("| State | n |\n|---|---:|\n");
    for (k, n) in &report.trend_x_momentum {
        md.push_str(&format!("| {k} | {n} |\n"));
    }
    md.push_str("\n## Trend × Momentum × Volatility\n\n");
    md.push_str("| State | n |\n|---|---:|\n");
    for (k, n) in &report.trend_x_momentum_x_vol {
        md.push_str(&format!("| {k} | {n} |\n"));
    }
    md.push_str("\n## roc_20 distribution (bars ≤ T; not a threshold)\n\n");
    md.push_str(&format!("{:#?}\n\n", report.roc_20));
    md.push_str("## atr_14 distribution (bars ≤ T; magnitude only; not High/Low)\n\n");
    md.push_str(&format!("{:#?}\n\n", report.atr_14));
    md.push_str(&format!(
        "## 60D outcomes attached as measurement only\n\navailable={} unavailable={}\n\n",
        report.n_outcome_60d_available, report.n_outcome_60d_unavailable
    ));
    md.push_str("These numbers must not be used to pick X/Y cutoffs.\n\n");
    md.push_str("## Design constraints for a later candidate\n\n");
    for c in &report.design_constraints {
        md.push_str(&format!("- {c}\n"));
    }
    md
}
