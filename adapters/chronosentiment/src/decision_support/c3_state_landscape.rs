//! CS-P-006-C.3-F — certified TMV state × action value landscape.
//!
//! Measures LONG / SHORT / NO_TRADE on every observed certified state,
//! irrespective of which action Search #2 chose. Does not evolve, retune,
//! promote, or authorize a product claim.

use std::collections::BTreeMap;

use serde::Serialize;

use super::c3_rule_ecology::{SEARCH_THREE_AUTHORIZED, SEARCH_TWO_PROMOTION_STATUS};
use super::csp006_protocol::{
    RESEARCH_DISCOVERY_ARTIFACT_HASH, RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH, RESEARCH_UNIVERSE,
};
use super::dataset_partition::PartitionKind;
use super::decision_value_landscape::action_value;
use super::recommendation_outcome::RecommendationRow;
use super::DecisionAction;

pub const STATE_LANDSCAPE_CONTRACT_ID: &str = "csp006c3f.state_action.1";
pub const PASS_THRESHOLD_INTRODUCED: bool = false;
pub const PRODUCT_CLAIM_AUTHORIZED: bool = false;

#[derive(Debug, Clone, Serialize)]
pub struct ActionMeans {
    pub n: u32,
    pub long: f64,
    pub short: f64,
    pub no_trade: f64,
    pub median_long: f64,
    pub n_up: u32,
    pub n_down: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct SliceActionMeans {
    pub slice: String,
    pub n: u32,
    pub long: f64,
    pub short: f64,
    pub no_trade: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct InstrumentActionMeans {
    pub instrument: String,
    pub n: u32,
    pub n_evaluation: u32,
    pub long: f64,
    pub evaluation_long: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ObservedState {
    pub trend_state: String,
    pub momentum_state: String,
    pub volatility_state: String,
    pub n: u32,
    pub n_evaluation: u32,
    pub overall: ActionMeans,
    pub evaluation: ActionMeans,
    pub slices: Vec<SliceActionMeans>,
    pub instruments: Vec<InstrumentActionMeans>,
    pub search_one_actions: BTreeMap<String, u32>,
    pub search_two_actions: BTreeMap<String, u32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct OccupancyNote {
    pub n_observed_states: u32,
    pub n_trend_neutral: u32,
    pub n_momentum_neutral: u32,
    pub n_volatility_absent: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct StateLandscapeReport {
    pub contract_id: String,
    pub search_one_artifact_hash: String,
    pub search_two_artifact_hash: String,
    pub promotion_status: String,
    pub search_three_authorized: bool,
    pub used_as_coralys_fitness: bool,
    pub pass_threshold_introduced: bool,
    pub product_claim_authorized: bool,
    pub n_rows: u32,
    pub occupancy: OccupancyNote,
    pub states: Vec<ObservedState>,
}

fn mean(xs: &[f64]) -> f64 {
    if xs.is_empty() {
        0.0
    } else {
        xs.iter().sum::<f64>() / xs.len() as f64
    }
}

fn median(xs: &[f64]) -> f64 {
    if xs.is_empty() {
        return 0.0;
    }
    let mut ordered = xs.to_vec();
    ordered.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let mid = (ordered.len() - 1) / 2;
    if ordered.len() % 2 == 1 {
        ordered[mid]
    } else {
        (ordered[mid] + ordered[mid + 1]) / 2.0
    }
}

fn action_key(action: DecisionAction) -> String {
    match action {
        DecisionAction::Long => "LONG".into(),
        DecisionAction::Short => "SHORT".into(),
        DecisionAction::NoTrade => "NO_TRADE".into(),
    }
}

fn action_means(returns: &[f64]) -> ActionMeans {
    let long = mean(returns);
    ActionMeans {
        n: returns.len() as u32,
        long: action_value(DecisionAction::Long, long),
        short: action_value(DecisionAction::Short, long),
        no_trade: action_value(DecisionAction::NoTrade, long),
        median_long: median(returns),
        n_up: returns.iter().filter(|r| **r > 0.0).count() as u32,
        n_down: returns.iter().filter(|r| **r < 0.0).count() as u32,
    }
}

fn histogram(actions: &[DecisionAction]) -> BTreeMap<String, u32> {
    let mut map = BTreeMap::new();
    for action in actions {
        *map.entry(action_key(*action)).or_insert(0) += 1;
    }
    map
}

fn state_of(row: &RecommendationRow) -> (String, String, String) {
    (
        row.trend_state.clone(),
        row.momentum_state.clone(),
        row.volatility_state.clone(),
    )
}

fn returns_of<'a>(rows: impl Iterator<Item = &'a RecommendationRow>) -> Vec<f64> {
    rows.filter_map(|r| r.actual_forward_return).collect()
}

fn build_state(
    trend: String,
    momentum: String,
    volatility: String,
    two: &[&RecommendationRow],
    one_actions: &[DecisionAction],
) -> Result<ObservedState, String> {
    if two.iter().any(|r| r.actual_forward_return.is_none()) {
        return Err(format!(
            "state {trend}/{momentum}/{volatility} missing return"
        ));
    }
    let all_r = returns_of(two.iter().copied());
    let eval_r = returns_of(
        two.iter()
            .copied()
            .filter(|r| r.partition == PartitionKind::Evaluation),
    );
    let mut slices = Vec::new();
    for kind in [
        PartitionKind::Development,
        PartitionKind::Selection,
        PartitionKind::Evaluation,
    ] {
        let rs = returns_of(two.iter().copied().filter(|r| r.partition == kind));
        let long = mean(&rs);
        slices.push(SliceActionMeans {
            slice: match kind {
                PartitionKind::Development => "development".into(),
                PartitionKind::Selection => "selection".into(),
                PartitionKind::Evaluation => "evaluation".into(),
            },
            n: rs.len() as u32,
            long,
            short: -long,
            no_trade: 0.0,
        });
    }
    let mut instruments = Vec::new();
    for ticker in RESEARCH_UNIVERSE {
        let subset: Vec<&&RecommendationRow> =
            two.iter().filter(|r| r.instrument == *ticker).collect();
        let rs = returns_of(subset.iter().copied().copied());
        let evals = returns_of(
            subset
                .iter()
                .copied()
                .copied()
                .filter(|r| r.partition == PartitionKind::Evaluation),
        );
        instruments.push(InstrumentActionMeans {
            instrument: (*ticker).to_string(),
            n: subset.len() as u32,
            n_evaluation: evals.len() as u32,
            long: mean(&rs),
            evaluation_long: if evals.is_empty() {
                None
            } else {
                Some(mean(&evals))
            },
        });
    }
    Ok(ObservedState {
        trend_state: trend,
        momentum_state: momentum,
        volatility_state: volatility,
        n: two.len() as u32,
        n_evaluation: eval_r.len() as u32,
        overall: action_means(&all_r),
        evaluation: action_means(&eval_r),
        slices,
        instruments,
        search_one_actions: histogram(one_actions),
        search_two_actions: histogram(&two.iter().map(|r| r.recommendation).collect::<Vec<_>>()),
    })
}

pub fn analyze_state_landscape(
    search_one: &[RecommendationRow],
    search_two: &[RecommendationRow],
) -> Result<StateLandscapeReport, String> {
    if search_one.len() != 273 || search_two.len() != 273 {
        return Err(format!(
            "expected 273+273 rows, found {}+{}",
            search_one.len(),
            search_two.len()
        ));
    }
    if search_one
        .iter()
        .any(|r| r.policy_artifact_hash != RESEARCH_DISCOVERY_ARTIFACT_HASH)
    {
        return Err("first matrix is not Search #1".into());
    }
    if search_two
        .iter()
        .any(|r| r.policy_artifact_hash != RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH)
    {
        return Err("second matrix is not Search #2".into());
    }
    let one_by_key: BTreeMap<(String, String), &RecommendationRow> = search_one
        .iter()
        .map(|r| ((r.instrument.clone(), r.timestamp.clone()), r))
        .collect();
    if one_by_key.len() != 273 {
        return Err("Search #1 keys are not unique".into());
    }

    let mut buckets: BTreeMap<(String, String, String), Vec<&RecommendationRow>> = BTreeMap::new();
    let mut one_actions: BTreeMap<(String, String, String), Vec<DecisionAction>> = BTreeMap::new();
    for two in search_two {
        let key = (two.instrument.clone(), two.timestamp.clone());
        let one = one_by_key
            .get(&key)
            .ok_or_else(|| format!("Search #1 missing {} {}", two.instrument, two.timestamp))?;
        if one.trend_state != two.trend_state
            || one.momentum_state != two.momentum_state
            || one.volatility_state != two.volatility_state
        {
            return Err(format!(
                "certified state mismatch at {} {}",
                two.instrument, two.timestamp
            ));
        }
        match (one.actual_forward_return, two.actual_forward_return) {
            (Some(a), Some(b)) if (a - b).abs() <= 1e-12 => {}
            (Some(_), Some(_)) => {
                return Err(format!(
                    "forward-return mismatch at {} {}",
                    two.instrument, two.timestamp
                ));
            }
            _ => {
                return Err(format!(
                    "missing forward return at {} {}",
                    two.instrument, two.timestamp
                ));
            }
        }
        let state = state_of(two);
        buckets.entry(state.clone()).or_default().push(two);
        one_actions
            .entry(state)
            .or_default()
            .push(one.recommendation);
    }

    let mut states = Vec::new();
    for ((trend, momentum, volatility), rows) in buckets {
        let actions = one_actions
            .remove(&(trend.clone(), momentum.clone(), volatility.clone()))
            .unwrap_or_default();
        states.push(build_state(trend, momentum, volatility, &rows, &actions)?);
    }
    states.sort_by(|a, b| b.n.cmp(&a.n));

    let occupancy = OccupancyNote {
        n_observed_states: states.len() as u32,
        n_trend_neutral: search_two
            .iter()
            .filter(|r| r.trend_state == "Neutral")
            .count() as u32,
        n_momentum_neutral: search_two
            .iter()
            .filter(|r| r.momentum_state == "Neutral")
            .count() as u32,
        n_volatility_absent: search_two
            .iter()
            .filter(|r| r.volatility_state != "present")
            .count() as u32,
    };

    Ok(StateLandscapeReport {
        contract_id: STATE_LANDSCAPE_CONTRACT_ID.to_string(),
        search_one_artifact_hash: RESEARCH_DISCOVERY_ARTIFACT_HASH.to_string(),
        search_two_artifact_hash: RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH.to_string(),
        promotion_status: SEARCH_TWO_PROMOTION_STATUS.to_string(),
        search_three_authorized: SEARCH_THREE_AUTHORIZED,
        used_as_coralys_fitness: false,
        pass_threshold_introduced: PASS_THRESHOLD_INTRODUCED,
        product_claim_authorized: PRODUCT_CLAIM_AUTHORIZED,
        n_rows: 273,
        occupancy,
        states,
    })
}

pub fn render_state_landscape(report: &StateLandscapeReport) -> String {
    let mut out = String::from("# CS-P-006-C.3-F — certified TMV state × action landscape\n\n");
    out.push_str("Measures LONG / SHORT / NO_TRADE on every observed certified state. ");
    out.push_str("Independent of Search #2's choice. Candidate research artifact. ");
    out.push_str("Not a product claim. Search #3 is not authorized.\n\n");
    out.push_str(&format!(
        "- search_one: `{}`\n- search_two: `{}`\n- promotion_status: `{}`\n- product_claim_authorized: {}\n- observed_states: {}\n- rows: {}\n\n",
        report.search_one_artifact_hash,
        report.search_two_artifact_hash,
        report.promotion_status,
        report.product_claim_authorized,
        report.occupancy.n_observed_states,
        report.n_rows
    ));
    out.push_str("Occupancy: Trend Neutral = ");
    out.push_str(&format!(
        "{}; Momentum Neutral = {}; Volatility absent = {}.\n\n",
        report.occupancy.n_trend_neutral,
        report.occupancy.n_momentum_neutral,
        report.occupancy.n_volatility_absent
    ));
    out.push_str("## Observed states\n\n");
    out.push_str("| Certified state | n | Eval n | LONG V | SHORT V | NO_TRADE | Eval LONG | Eval SHORT | Search #1 | Search #2 |\n");
    out.push_str("|---|---:|---:|---:|---:|---:|---:|---:|---|---|\n");
    for state in &report.states {
        out.push_str(&format!(
            "| {} / {} / {} | {} | {} | {:.4}% | {:.4}% | 0 | {:.4}% | {:.4}% | {} | {} |\n",
            state.trend_state,
            state.momentum_state,
            state.volatility_state,
            state.n,
            state.n_evaluation,
            100.0 * state.overall.long,
            100.0 * state.overall.short,
            100.0 * state.evaluation.long,
            100.0 * state.evaluation.short,
            format_actions(&state.search_one_actions),
            format_actions(&state.search_two_actions)
        ));
    }
    out.push_str("\nSHORT V is the sign flip of LONG V. NO_TRADE is 0. ");
    out.push_str(
        "Search columns are what each sealed policy chose, not the object of measurement. ",
    );
    out.push_str("No threshold decides whether a state is useful. Search #3 is not authorized.\n");
    out
}

fn format_actions(map: &BTreeMap<String, u32>) -> String {
    map.iter()
        .map(|(k, n)| format!("{k}×{n}"))
        .collect::<Vec<_>>()
        .join(", ")
}
