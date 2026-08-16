//! CS-P-006-C.3-D — live-rule ecology of the sealed Search #2 artifact.
//!
//! Does not evolve, retune, or rewrite the seven-rule genome.
//! Search #2 remains a candidate research artifact, not a promoted strategy.

use std::collections::BTreeMap;

use serde::Serialize;

use super::csp006_protocol::{RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH, RESEARCH_UNIVERSE};
use super::dataset_partition::PartitionKind;
use super::decision_value_landscape::{action_value, landscape_row, DecisionValueRow};
use super::policy_artifact::{DecisionRule, FactorPredicate, PolicyArtifact};
use super::recommendation_outcome::RecommendationRow;
use super::DecisionAction;

pub const RULE_ECOLOGY_CONTRACT_ID: &str = "csp006c3d.rule_ecology.1";
pub const SEARCH_TWO_PROMOTION_STATUS: &str = "candidate_research_artifact";
pub const SEARCH_THREE_AUTHORIZED: bool = false;

const LIVE_RULE_INDICES: [usize; 3] = [0, 1, 3];

#[derive(Debug, Clone, Serialize)]
pub struct ContinuousStats {
    pub n: u32,
    pub mean: f64,
    pub median: f64,
    pub p25: f64,
    pub p75: f64,
    pub min: f64,
    pub max: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct StateCount {
    pub state: String,
    pub n: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct InstrumentRuleValue {
    pub instrument: String,
    pub n: u32,
    pub n_evaluation: u32,
    pub mean_v: f64,
    pub evaluation_mean_v: Option<f64>,
    pub unique_best_n: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct SliceCount {
    pub slice: String,
    pub n: u32,
    pub mean_v: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct YearCount {
    pub year: i32,
    pub n: u32,
    pub mean_v: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct FiredState {
    pub trend_state: String,
    pub momentum_state: String,
    pub volatility_state: String,
    pub n: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct LiveRuleEcology {
    pub rule_index: usize,
    pub label: String,
    pub action: DecisionAction,
    pub n: u32,
    pub slices: Vec<SliceCount>,
    pub instruments: Vec<InstrumentRuleValue>,
    pub value: ContinuousStats,
    pub evaluation_value: Option<ContinuousStats>,
    pub regret: ContinuousStats,
    pub unique_best_n: u32,
    pub unique_best_share: f64,
    pub n_positive_v: u32,
    pub n_negative_v: u32,
    pub alternative_mean_long: f64,
    pub alternative_mean_short: f64,
    pub alternative_mean_no_trade: f64,
    pub momentum_states: Vec<StateCount>,
    pub trend_states: Vec<StateCount>,
    pub volatility_states: Vec<StateCount>,
    pub fired_states: Vec<FiredState>,
    pub years: Vec<YearCount>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RuleEcologyReport {
    pub contract_id: String,
    pub search_two_artifact_hash: String,
    pub promotion_status: String,
    pub search_three_authorized: bool,
    pub used_as_coralys_fitness: bool,
    pub n_rows: u32,
    pub live_rules: Vec<LiveRuleEcology>,
    pub otherwise_means: Vec<FiredState>,
    pub value_share_of_sum: BTreeMap<String, f64>,
}

fn percentile(sorted: &[f64], q: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let idx = ((sorted.len() as f64 - 1.0) * q).round() as usize;
    sorted[idx.min(sorted.len() - 1)]
}

fn stats(xs: &[f64]) -> ContinuousStats {
    let mut ordered = xs.to_vec();
    ordered.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let n = xs.len() as u32;
    let mean = if xs.is_empty() {
        0.0
    } else {
        xs.iter().sum::<f64>() / xs.len() as f64
    };
    ContinuousStats {
        n,
        mean,
        median: percentile(&ordered, 0.50),
        p25: percentile(&ordered, 0.25),
        p75: percentile(&ordered, 0.75),
        min: ordered.first().copied().unwrap_or(0.0),
        max: ordered.last().copied().unwrap_or(0.0),
    }
}

fn label_matches(pred: &FactorPredicate, trend: &str, momentum: &str, volatility: &str) -> bool {
    let (state, present) = match pred.concept.as_str() {
        "Trend" => (trend, trend != "absent"),
        "Momentum" => (momentum, momentum != "absent"),
        "Volatility" => (volatility, volatility == "present"),
        _ => return false,
    };
    match pred.present {
        Some(true) if !present => return false,
        Some(false) if present => return false,
        _ => {}
    }
    match &pred.direction {
        None => true,
        Some(dir) => present && state == dir,
    }
}

fn rule_matches(rule: &DecisionRule, row: &RecommendationRow) -> bool {
    rule.when.iter().all(|p| {
        label_matches(p, &row.trend_state, &row.momentum_state, &row.volatility_state)
    })
}

pub fn first_match_rule_index(artifact: &PolicyArtifact, row: &RecommendationRow) -> Option<usize> {
    artifact
        .rules
        .iter()
        .position(|rule| rule_matches(rule, row))
}

fn histogram(values: &[String]) -> Vec<StateCount> {
    let mut map: BTreeMap<String, u32> = BTreeMap::new();
    for v in values {
        *map.entry(v.clone()).or_insert(0) += 1;
    }
    map.into_iter()
        .map(|(state, n)| StateCount { state, n })
        .collect()
}

fn year_of(timestamp: &str) -> i32 {
    timestamp
        .get(0..4)
        .and_then(|s| s.parse().ok())
        .unwrap_or(0)
}

fn live_label(index: usize) -> &'static str {
    match index {
        0 => "Bearish → LONG",
        1 => "Bullish ∧ Positive Momentum → LONG",
        3 => "Bullish otherwise → SHORT",
        _ => "other",
    }
}

fn ecology_for(
    index: usize,
    action: DecisionAction,
    rows: &[(&RecommendationRow, DecisionValueRow)],
) -> LiveRuleEcology {
    let values: Vec<f64> = rows.iter().map(|(_, l)| l.recommended_value).collect();
    let regrets: Vec<f64> = rows.iter().map(|(_, l)| l.regret).collect();
    let eval_values: Vec<f64> = rows
        .iter()
        .filter(|(r, _)| r.partition == PartitionKind::Evaluation)
        .map(|(_, l)| l.recommended_value)
        .collect();
    let unique_best_n = rows
        .iter()
        .filter(|(_, l)| l.recommended_is_unique_best)
        .count() as u32;
    let mut slices = Vec::new();
    for kind in [
        PartitionKind::Development,
        PartitionKind::Selection,
        PartitionKind::Evaluation,
    ] {
        let vs: Vec<f64> = rows
            .iter()
            .filter(|(r, _)| r.partition == kind)
            .map(|(_, l)| l.recommended_value)
            .collect();
        slices.push(SliceCount {
            slice: match kind {
                PartitionKind::Development => "development".into(),
                PartitionKind::Selection => "selection".into(),
                PartitionKind::Evaluation => "evaluation".into(),
            },
            n: vs.len() as u32,
            mean_v: if vs.is_empty() {
                0.0
            } else {
                vs.iter().sum::<f64>() / vs.len() as f64
            },
        });
    }
    let mut instruments = Vec::new();
    for ticker in RESEARCH_UNIVERSE {
        let subset: Vec<_> = rows
            .iter()
            .filter(|(r, _)| r.instrument == *ticker)
            .collect();
        let evals: Vec<f64> = subset
            .iter()
            .filter(|(r, _)| r.partition == PartitionKind::Evaluation)
            .map(|(_, l)| l.recommended_value)
            .collect();
        let vs: Vec<f64> = subset.iter().map(|(_, l)| l.recommended_value).collect();
        instruments.push(InstrumentRuleValue {
            instrument: (*ticker).to_string(),
            n: subset.len() as u32,
            n_evaluation: evals.len() as u32,
            mean_v: if vs.is_empty() {
                0.0
            } else {
                vs.iter().sum::<f64>() / vs.len() as f64
            },
            evaluation_mean_v: if evals.is_empty() {
                None
            } else {
                Some(evals.iter().sum::<f64>() / evals.len() as f64)
            },
            unique_best_n: subset
                .iter()
                .filter(|(_, l)| l.recommended_is_unique_best)
                .count() as u32,
        });
    }
    let mut year_map: BTreeMap<i32, Vec<f64>> = BTreeMap::new();
    for (r, l) in rows {
        year_map
            .entry(year_of(&r.timestamp))
            .or_default()
            .push(l.recommended_value);
    }
    let years = year_map
        .into_iter()
        .map(|(year, vs)| YearCount {
            year,
            n: vs.len() as u32,
            mean_v: vs.iter().sum::<f64>() / vs.len() as f64,
        })
        .collect();
    let mut state_map: BTreeMap<(String, String, String), u32> = BTreeMap::new();
    for (r, _) in rows {
        *state_map
            .entry((
                r.trend_state.clone(),
                r.momentum_state.clone(),
                r.volatility_state.clone(),
            ))
            .or_insert(0) += 1;
    }
    let fired_states = state_map
        .into_iter()
        .map(|((trend_state, momentum_state, volatility_state), n)| FiredState {
            trend_state,
            momentum_state,
            volatility_state,
            n,
        })
        .collect();
    LiveRuleEcology {
        rule_index: index,
        label: live_label(index).to_string(),
        action,
        n: rows.len() as u32,
        slices,
        instruments,
        value: stats(&values),
        evaluation_value: if eval_values.is_empty() {
            None
        } else {
            Some(stats(&eval_values))
        },
        regret: stats(&regrets),
        unique_best_n,
        unique_best_share: if rows.is_empty() {
            0.0
        } else {
            unique_best_n as f64 / rows.len() as f64
        },
        n_positive_v: values.iter().filter(|v| **v > 0.0).count() as u32,
        n_negative_v: values.iter().filter(|v| **v < 0.0).count() as u32,
        alternative_mean_long: rows
            .iter()
            .map(|(_, l)| action_value(DecisionAction::Long, l.raw_forward_return))
            .sum::<f64>()
            / rows.len().max(1) as f64,
        alternative_mean_short: rows
            .iter()
            .map(|(_, l)| action_value(DecisionAction::Short, l.raw_forward_return))
            .sum::<f64>()
            / rows.len().max(1) as f64,
        alternative_mean_no_trade: 0.0,
        momentum_states: histogram(&rows.iter().map(|(r, _)| r.momentum_state.clone()).collect::<Vec<_>>()),
        trend_states: histogram(&rows.iter().map(|(r, _)| r.trend_state.clone()).collect::<Vec<_>>()),
        volatility_states: histogram(
            &rows
                .iter()
                .map(|(r, _)| r.volatility_state.clone())
                .collect::<Vec<_>>(),
        ),
        fired_states,
        years,
    }
}

pub fn analyze_live_rules(
    recommendations: &[RecommendationRow],
    artifact: &PolicyArtifact,
) -> Result<RuleEcologyReport, String> {
    if artifact.artifact_hash != RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH {
        return Err("rule ecology identity-gates Search #2".into());
    }
    if recommendations
        .iter()
        .any(|r| r.policy_artifact_hash != RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH)
    {
        return Err("recommendation matrix is not Search #2".into());
    }
    if recommendations.len() != 273 {
        return Err(format!("expected 273 rows, found {}", recommendations.len()));
    }
    let mut buckets: BTreeMap<usize, Vec<(&RecommendationRow, DecisionValueRow)>> = BTreeMap::new();
    for row in recommendations {
        let idx = first_match_rule_index(artifact, row)
            .ok_or_else(|| format!("unmatched row {} {}", row.instrument, row.timestamp))?;
        let landscape = landscape_row(row).ok_or("row missing return")?;
        buckets.entry(idx).or_default().push((row, landscape));
    }
    let mut live_rules = Vec::new();
    for index in LIVE_RULE_INDICES {
        let rows = buckets.remove(&index).unwrap_or_default();
        let action = artifact.rules[index].action;
        live_rules.push(ecology_for(index, action, &rows));
    }
    if !buckets.is_empty() {
        return Err(format!(
            "rows assigned to non-live rules: {:?}",
            buckets.keys().collect::<Vec<_>>()
        ));
    }
    let otherwise = live_rules
        .iter()
        .find(|r| r.rule_index == 3)
        .map(|r| r.fired_states.clone())
        .unwrap_or_default();
    let total_sum: f64 = live_rules
        .iter()
        .map(|r| r.value.mean * r.n as f64)
        .sum();
    let mut value_share_of_sum = BTreeMap::new();
    for rule in &live_rules {
        let share = if total_sum.abs() < 1e-15 {
            0.0
        } else {
            (rule.value.mean * rule.n as f64) / total_sum
        };
        value_share_of_sum.insert(rule.label.clone(), share);
    }
    Ok(RuleEcologyReport {
        contract_id: RULE_ECOLOGY_CONTRACT_ID.to_string(),
        search_two_artifact_hash: RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH.to_string(),
        promotion_status: SEARCH_TWO_PROMOTION_STATUS.to_string(),
        search_three_authorized: SEARCH_THREE_AUTHORIZED,
        used_as_coralys_fitness: false,
        n_rows: 273,
        live_rules,
        otherwise_means: otherwise,
        value_share_of_sum,
    })
}

pub fn render_rule_ecology(report: &RuleEcologyReport) -> String {
    let mut out = String::from("# CS-P-006-C.3-D — Search #2 live-rule ecology\n\n");
    out.push_str("Sealed Search #2 only. Candidate research artifact. Not promoted. Search #3 is not authorized.\n\n");
    out.push_str(&format!(
        "- artifact: `{}`\n- promotion_status: `{}`\n- rows: {}\n\n",
        report.search_two_artifact_hash, report.promotion_status, report.n_rows
    ));
    out.push_str("## Live rules\n\n");
    out.push_str("| Live state | Action | n | Mean V | Eval V | Unique-best |\n|---|---|---:|---:|---:|---:|\n");
    for rule in &report.live_rules {
        let eval = rule
            .evaluation_value
            .as_ref()
            .map(|s| format!("{:.4}%", 100.0 * s.mean))
            .unwrap_or_else(|| "—".into());
        out.push_str(&format!(
            "| {} | {:?} | {} | {:.4}% | {} | {} ({:.1}%) |\n",
            rule.label,
            rule.action,
            rule.n,
            100.0 * rule.value.mean,
            eval,
            rule.unique_best_n,
            100.0 * rule.unique_best_share
        ));
    }
    out.push_str("\n## What `Bullish otherwise → SHORT` actually fired on\n\n");
    out.push_str("| Trend | Momentum | Volatility | n |\n|---|---|---|---:|\n");
    for state in &report.otherwise_means {
        out.push_str(&format!(
            "| {} | {} | {} | {} |\n",
            state.trend_state, state.momentum_state, state.volatility_state, state.n
        ));
    }
    out.push_str("\nDo not rewrite the rule. Search #3 is not authorized.\n");
    out
}
