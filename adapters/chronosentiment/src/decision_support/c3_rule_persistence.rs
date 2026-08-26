//! CS-P-006-C.3-E — persistence of the three live Search #2 rules.
//!
//! Measures the sealed artifact. Does not evolve, retune, or introduce a
//! pass/fail threshold. Search #2 remains a candidate research artifact.

use std::collections::BTreeMap;

use serde::Serialize;

use super::c3_rule_ecology::{
    first_match_rule_index, SEARCH_THREE_AUTHORIZED, SEARCH_TWO_PROMOTION_STATUS,
};
use super::csp006_protocol::{RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH, RESEARCH_UNIVERSE};
use super::dataset_partition::PartitionKind;
use super::decision_value_landscape::{action_value, landscape_row, DecisionValueRow};
use super::policy_artifact::PolicyArtifact;
use super::recommendation_outcome::RecommendationRow;
use super::DecisionAction;

pub const PERSISTENCE_CONTRACT_ID: &str = "csp006c3e.rule_persistence.1";
pub const PASS_THRESHOLD_INTRODUCED: bool = false;

const LIVE_RULE_INDICES: [usize; 3] = [0, 1, 3];

const YEAR_WINDOWS: [(&str, &[i32]); 3] = [
    ("2021–22", &[2021, 2022]),
    ("2022–23", &[2022, 2023]),
    ("2023–24", &[2023, 2024]),
];

#[derive(Debug, Clone, Serialize)]
pub struct ActionAdvantage {
    pub n: u32,
    pub recommended: f64,
    pub long: f64,
    pub short: f64,
    pub no_trade: f64,
    pub vs_long: f64,
    pub vs_short: f64,
    pub vs_no_trade: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct WindowValue {
    pub window: String,
    pub n: u32,
    pub mean_v: f64,
    pub median_v: f64,
    pub n_positive: u32,
    pub n_negative: u32,
    pub recommended: f64,
    pub long: f64,
    pub short: f64,
    pub no_trade: f64,
    pub vs_no_trade: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct InstrumentPersistence {
    pub instrument: String,
    pub n: u32,
    pub n_evaluation: u32,
    pub mean_v: f64,
    pub evaluation_mean_v: Option<f64>,
    pub n_positive: u32,
    pub n_negative: u32,
    pub gain_sum: f64,
    pub loss_sum: f64,
    pub long: f64,
    pub short: f64,
    pub vs_no_trade: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct StatePersistence {
    pub trend_state: String,
    pub momentum_state: String,
    pub volatility_state: String,
    pub n: u32,
    pub mean_v: f64,
    pub evaluation_n: u32,
    pub evaluation_mean_v: Option<f64>,
    pub n_positive: u32,
    pub n_negative: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct LossShare {
    pub key: String,
    pub n_loss: u32,
    pub loss_sum: f64,
    pub share_of_loss_sum: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct FailureCluster {
    pub n_gain: u32,
    pub n_loss: u32,
    pub n_zero: u32,
    pub gain_sum: f64,
    pub loss_sum: f64,
    pub instrument_loss_share: Vec<LossShare>,
    pub year_loss_share: Vec<LossShare>,
    pub slice_loss_share: Vec<LossShare>,
    pub largest_loss_instrument: Option<String>,
    pub largest_loss_year: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RulePersistence {
    pub rule_index: usize,
    pub label: String,
    pub action: DecisionAction,
    pub n: u32,
    pub n_evaluation: u32,
    pub mean_v: f64,
    pub evaluation_mean_v: Option<f64>,
    pub unique_best_n: u32,
    pub unique_best_share: f64,
    pub signed_value_sum: f64,
    pub contribution_share: f64,
    pub evaluation_signed_sum: f64,
    pub evaluation_contribution_share: f64,
    pub windows: Vec<WindowValue>,
    pub instruments: Vec<InstrumentPersistence>,
    pub states: Vec<StatePersistence>,
    pub action_advantage: ActionAdvantage,
    pub evaluation_action_advantage: ActionAdvantage,
    pub failures: FailureCluster,
}

#[derive(Debug, Clone, Serialize)]
pub struct PersistenceReport {
    pub contract_id: String,
    pub search_two_artifact_hash: String,
    pub promotion_status: String,
    pub search_three_authorized: bool,
    pub used_as_coralys_fitness: bool,
    pub pass_threshold_introduced: bool,
    pub n_rows: u32,
    pub rules: Vec<RulePersistence>,
}

type Pair<'a> = (&'a RecommendationRow, DecisionValueRow);

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
        3 => "Bullish ∧ Negative Momentum → SHORT",
        _ => "other",
    }
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

fn advantage(rows: &[Pair<'_>]) -> ActionAdvantage {
    if rows.is_empty() {
        return ActionAdvantage {
            n: 0,
            recommended: 0.0,
            long: 0.0,
            short: 0.0,
            no_trade: 0.0,
            vs_long: 0.0,
            vs_short: 0.0,
            vs_no_trade: 0.0,
        };
    }
    let n = rows.len() as f64;
    let recommended = rows.iter().map(|(_, l)| l.recommended_value).sum::<f64>() / n;
    let long = rows
        .iter()
        .map(|(_, l)| action_value(DecisionAction::Long, l.raw_forward_return))
        .sum::<f64>()
        / n;
    let short = rows
        .iter()
        .map(|(_, l)| action_value(DecisionAction::Short, l.raw_forward_return))
        .sum::<f64>()
        / n;
    ActionAdvantage {
        n: rows.len() as u32,
        recommended,
        long,
        short,
        no_trade: 0.0,
        vs_long: recommended - long,
        vs_short: recommended - short,
        vs_no_trade: recommended,
    }
}

fn window_of(label: &str, rows: &[Pair<'_>]) -> WindowValue {
    let values: Vec<f64> = rows.iter().map(|(_, l)| l.recommended_value).collect();
    let adv = advantage(rows);
    WindowValue {
        window: label.to_string(),
        n: rows.len() as u32,
        mean_v: mean(&values),
        median_v: median(&values),
        n_positive: values.iter().filter(|v| **v > 0.0).count() as u32,
        n_negative: values.iter().filter(|v| **v < 0.0).count() as u32,
        recommended: adv.recommended,
        long: adv.long,
        short: adv.short,
        no_trade: 0.0,
        vs_no_trade: adv.vs_no_trade,
    }
}

fn loss_shares(entries: BTreeMap<String, (u32, f64)>, total_loss: f64) -> Vec<LossShare> {
    let mut out: Vec<LossShare> = entries
        .into_iter()
        .map(|(key, (n_loss, loss_sum))| LossShare {
            key,
            n_loss,
            loss_sum,
            share_of_loss_sum: if total_loss.abs() < 1e-15 {
                0.0
            } else {
                loss_sum / total_loss
            },
        })
        .collect();
    out.sort_by(|a, b| {
        b.loss_sum
            .abs()
            .partial_cmp(&a.loss_sum.abs())
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    out
}

fn failures(rows: &[Pair<'_>]) -> FailureCluster {
    let values: Vec<f64> = rows.iter().map(|(_, l)| l.recommended_value).collect();
    let n_gain = values.iter().filter(|v| **v > 0.0).count() as u32;
    let n_loss = values.iter().filter(|v| **v < 0.0).count() as u32;
    let n_zero = values.iter().filter(|v| **v == 0.0).count() as u32;
    let gain_sum = values.iter().filter(|v| **v > 0.0).sum::<f64>();
    let loss_sum = values.iter().filter(|v| **v < 0.0).sum::<f64>();

    let mut by_instrument: BTreeMap<String, (u32, f64)> = BTreeMap::new();
    let mut by_year: BTreeMap<String, (u32, f64)> = BTreeMap::new();
    let mut by_slice: BTreeMap<String, (u32, f64)> = BTreeMap::new();
    for (row, landscape) in rows {
        if landscape.recommended_value >= 0.0 {
            continue;
        }
        let inst = by_instrument
            .entry(row.instrument.clone())
            .or_insert((0, 0.0));
        inst.0 += 1;
        inst.1 += landscape.recommended_value;
        let year = by_year
            .entry(year_of(&row.timestamp).to_string())
            .or_insert((0, 0.0));
        year.0 += 1;
        year.1 += landscape.recommended_value;
        let slice_name = match row.partition {
            PartitionKind::Development => "development",
            PartitionKind::Selection => "selection",
            PartitionKind::Evaluation => "evaluation",
        };
        let slice = by_slice.entry(slice_name.to_string()).or_insert((0, 0.0));
        slice.0 += 1;
        slice.1 += landscape.recommended_value;
    }
    let instrument_loss_share = loss_shares(by_instrument, loss_sum);
    let year_loss_share = loss_shares(by_year, loss_sum);
    let slice_loss_share = loss_shares(by_slice, loss_sum);
    FailureCluster {
        n_gain,
        n_loss,
        n_zero,
        gain_sum,
        loss_sum,
        largest_loss_instrument: instrument_loss_share.first().map(|s| s.key.clone()),
        largest_loss_year: year_loss_share.first().map(|s| s.key.clone()),
        instrument_loss_share,
        year_loss_share,
        slice_loss_share,
    }
}

fn persist_rule(index: usize, action: DecisionAction, rows: &[Pair<'_>]) -> RulePersistence {
    let values: Vec<f64> = rows.iter().map(|(_, l)| l.recommended_value).collect();
    let eval_rows: Vec<Pair<'_>> = rows
        .iter()
        .filter(|(r, _)| r.partition == PartitionKind::Evaluation)
        .cloned()
        .collect();
    let eval_values: Vec<f64> = eval_rows.iter().map(|(_, l)| l.recommended_value).collect();
    let unique_best_n = rows
        .iter()
        .filter(|(_, l)| l.recommended_is_unique_best)
        .count() as u32;

    let mut windows = Vec::new();
    for (label, years) in YEAR_WINDOWS {
        let subset: Vec<Pair<'_>> = rows
            .iter()
            .filter(|(r, _)| years.contains(&year_of(&r.timestamp)))
            .cloned()
            .collect();
        windows.push(window_of(label, &subset));
    }
    for kind in [
        PartitionKind::Development,
        PartitionKind::Selection,
        PartitionKind::Evaluation,
    ] {
        let label = match kind {
            PartitionKind::Development => "development",
            PartitionKind::Selection => "selection",
            PartitionKind::Evaluation => "evaluation",
        };
        let subset: Vec<Pair<'_>> = rows
            .iter()
            .filter(|(r, _)| r.partition == kind)
            .cloned()
            .collect();
        windows.push(window_of(label, &subset));
    }

    let mut instruments = Vec::new();
    for ticker in RESEARCH_UNIVERSE {
        let subset: Vec<Pair<'_>> = rows
            .iter()
            .filter(|(r, _)| r.instrument == *ticker)
            .cloned()
            .collect();
        let evals: Vec<f64> = subset
            .iter()
            .filter(|(r, _)| r.partition == PartitionKind::Evaluation)
            .map(|(_, l)| l.recommended_value)
            .collect();
        let vs: Vec<f64> = subset.iter().map(|(_, l)| l.recommended_value).collect();
        let adv = advantage(&subset);
        instruments.push(InstrumentPersistence {
            instrument: (*ticker).to_string(),
            n: subset.len() as u32,
            n_evaluation: evals.len() as u32,
            mean_v: mean(&vs),
            evaluation_mean_v: if evals.is_empty() {
                None
            } else {
                Some(mean(&evals))
            },
            n_positive: vs.iter().filter(|v| **v > 0.0).count() as u32,
            n_negative: vs.iter().filter(|v| **v < 0.0).count() as u32,
            gain_sum: vs.iter().filter(|v| **v > 0.0).sum(),
            loss_sum: vs.iter().filter(|v| **v < 0.0).sum(),
            long: adv.long,
            short: adv.short,
            vs_no_trade: adv.vs_no_trade,
        });
    }

    let mut state_map: BTreeMap<(String, String, String), Vec<Pair<'_>>> = BTreeMap::new();
    for pair in rows {
        state_map
            .entry((
                pair.0.trend_state.clone(),
                pair.0.momentum_state.clone(),
                pair.0.volatility_state.clone(),
            ))
            .or_default()
            .push(pair.clone());
    }
    let states = state_map
        .into_iter()
        .map(
            |((trend_state, momentum_state, volatility_state), subset)| {
                let vs: Vec<f64> = subset.iter().map(|(_, l)| l.recommended_value).collect();
                let evals: Vec<f64> = subset
                    .iter()
                    .filter(|(r, _)| r.partition == PartitionKind::Evaluation)
                    .map(|(_, l)| l.recommended_value)
                    .collect();
                StatePersistence {
                    trend_state,
                    momentum_state,
                    volatility_state,
                    n: subset.len() as u32,
                    mean_v: mean(&vs),
                    evaluation_n: evals.len() as u32,
                    evaluation_mean_v: if evals.is_empty() {
                        None
                    } else {
                        Some(mean(&evals))
                    },
                    n_positive: vs.iter().filter(|v| **v > 0.0).count() as u32,
                    n_negative: vs.iter().filter(|v| **v < 0.0).count() as u32,
                }
            },
        )
        .collect();

    RulePersistence {
        rule_index: index,
        label: live_label(index).to_string(),
        action,
        n: rows.len() as u32,
        n_evaluation: eval_rows.len() as u32,
        mean_v: mean(&values),
        evaluation_mean_v: if eval_values.is_empty() {
            None
        } else {
            Some(mean(&eval_values))
        },
        unique_best_n,
        unique_best_share: if rows.is_empty() {
            0.0
        } else {
            unique_best_n as f64 / rows.len() as f64
        },
        signed_value_sum: values.iter().sum(),
        contribution_share: 0.0,
        evaluation_signed_sum: eval_values.iter().sum(),
        evaluation_contribution_share: 0.0,
        windows,
        instruments,
        states,
        action_advantage: advantage(rows),
        evaluation_action_advantage: advantage(&eval_rows),
        failures: failures(rows),
    }
}

pub fn analyze_rule_persistence(
    recommendations: &[RecommendationRow],
    artifact: &PolicyArtifact,
) -> Result<PersistenceReport, String> {
    if artifact.artifact_hash != RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH {
        return Err("rule persistence identity-gates Search #2".into());
    }
    if recommendations
        .iter()
        .any(|r| r.policy_artifact_hash != RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH)
    {
        return Err("recommendation matrix is not Search #2".into());
    }
    if recommendations.len() != 273 {
        return Err(format!(
            "expected 273 rows, found {}",
            recommendations.len()
        ));
    }
    let mut buckets: BTreeMap<usize, Vec<Pair<'_>>> = BTreeMap::new();
    for row in recommendations {
        let idx = first_match_rule_index(artifact, row)
            .ok_or_else(|| format!("unmatched row {} {}", row.instrument, row.timestamp))?;
        let landscape = landscape_row(row).ok_or("row missing return")?;
        buckets.entry(idx).or_default().push((row, landscape));
    }
    let mut rules = Vec::new();
    for index in LIVE_RULE_INDICES {
        let rows = buckets.remove(&index).unwrap_or_default();
        let action = artifact.rules[index].action;
        rules.push(persist_rule(index, action, &rows));
    }
    if !buckets.is_empty() {
        return Err(format!(
            "rows assigned to non-live rules: {:?}",
            buckets.keys().collect::<Vec<_>>()
        ));
    }
    let total_sum: f64 = rules.iter().map(|r| r.signed_value_sum).sum();
    let eval_sum: f64 = rules.iter().map(|r| r.evaluation_signed_sum).sum();
    for rule in &mut rules {
        rule.contribution_share = if total_sum.abs() < 1e-15 {
            0.0
        } else {
            rule.signed_value_sum / total_sum
        };
        rule.evaluation_contribution_share = if eval_sum.abs() < 1e-15 {
            0.0
        } else {
            rule.evaluation_signed_sum / eval_sum
        };
    }
    Ok(PersistenceReport {
        contract_id: PERSISTENCE_CONTRACT_ID.to_string(),
        search_two_artifact_hash: RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH.to_string(),
        promotion_status: SEARCH_TWO_PROMOTION_STATUS.to_string(),
        search_three_authorized: SEARCH_THREE_AUTHORIZED,
        used_as_coralys_fitness: false,
        pass_threshold_introduced: PASS_THRESHOLD_INTRODUCED,
        n_rows: 273,
        rules,
    })
}

pub fn render_rule_persistence(report: &PersistenceReport) -> String {
    let mut out = String::from("# CS-P-006-C.3-E — Search #2 discovered-rule persistence\n\n");
    out.push_str("Sealed Search #2 only. Candidate research artifact. Not promoted. ");
    out.push_str("No pass/fail threshold. Search #3 is not authorized.\n\n");
    out.push_str(&format!(
        "- artifact: `{}`\n- promotion_status: `{}`\n- pass_threshold_introduced: {}\n- rows: {}\n\n",
        report.search_two_artifact_hash,
        report.promotion_status,
        report.pass_threshold_introduced,
        report.n_rows
    ));
    out.push_str("## Sample size and contribution\n\n");
    out.push_str(
        "| Live state | Action | n | Eval n | Mean V | Eval V | Value share | Eval share |\n",
    );
    out.push_str("|---|---|---:|---:|---:|---:|---:|---:|\n");
    for rule in &report.rules {
        let eval = rule
            .evaluation_mean_v
            .map(|v| format!("{:.4}%", 100.0 * v))
            .unwrap_or_else(|| "—".into());
        out.push_str(&format!(
            "| {} | {:?} | {} | {} | {:.4}% | {} | {:.1}% | {:.1}% |\n",
            rule.label,
            rule.action,
            rule.n,
            rule.n_evaluation,
            100.0 * rule.mean_v,
            eval,
            100.0 * rule.contribution_share,
            100.0 * rule.evaluation_contribution_share
        ));
    }
    out.push_str("\nCalendar windows 2021–22 / 2022–23 / 2023–24 overlap on the shared year. ");
    out.push_str("They are persistence views, not a second partition.\n\n");
    for rule in &report.rules {
        out.push_str(&format!("## {}\n\n", rule.label));
        out.push_str("| Window | n | Mean V | Median V | +/− | vs NO_TRADE |\n|---|---:|---:|---:|---:|---:|\n");
        for w in &rule.windows {
            out.push_str(&format!(
                "| {} | {} | {:.4}% | {:.4}% | {}/{} | {:.4}% |\n",
                w.window,
                w.n,
                100.0 * w.mean_v,
                100.0 * w.median_v,
                w.n_positive,
                w.n_negative,
                100.0 * w.vs_no_trade
            ));
        }
        out.push_str("\nFired states:\n\n");
        out.push_str("| Trend | Momentum | Volatility | n | Mean V | Eval n | Eval V |\n|---|---|---|---:|---:|---:|---:|\n");
        for s in &rule.states {
            let ev = s
                .evaluation_mean_v
                .map(|v| format!("{:.4}%", 100.0 * v))
                .unwrap_or_else(|| "—".into());
            out.push_str(&format!(
                "| {} | {} | {} | {} | {:.4}% | {} | {} |\n",
                s.trend_state,
                s.momentum_state,
                s.volatility_state,
                s.n,
                100.0 * s.mean_v,
                s.evaluation_n,
                ev
            ));
        }
        out.push_str(&format!(
            "\nAction advantage (all / evaluation): recommended {:.4}% / {:.4}%; LONG {:.4}% / {:.4}%; SHORT {:.4}% / {:.4}%.\n",
            100.0 * rule.action_advantage.recommended,
            100.0 * rule.evaluation_action_advantage.recommended,
            100.0 * rule.action_advantage.long,
            100.0 * rule.evaluation_action_advantage.long,
            100.0 * rule.action_advantage.short,
            100.0 * rule.evaluation_action_advantage.short
        ));
        out.push_str(&format!(
            "Losses cluster: n_loss={} loss_sum={:.4}% largest_instrument={} largest_year={}.\n\n",
            rule.failures.n_loss,
            100.0 * rule.failures.loss_sum,
            rule.failures
                .largest_loss_instrument
                .as_deref()
                .unwrap_or("—"),
            rule.failures.largest_loss_year.as_deref().unwrap_or("—")
        ));
    }
    out.push_str("Unique-best shares are diagnostics over this sample, not confidence. ");
    out.push_str("No threshold decides whether a rule persists. Search #3 is not authorized.\n");
    out
}
