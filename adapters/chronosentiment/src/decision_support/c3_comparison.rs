//! CS-P-006-C.3-C — sealed-artifact comparison of Search #1 and Search #2.
//!
//! Does not evolve, retune, or modify either policy.

use std::collections::BTreeMap;

use serde::Serialize;

use super::csp006_protocol::{
    RESEARCH_DISCOVERY_ARTIFACT_HASH, RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH, RESEARCH_UNIVERSE,
};
use super::dataset_partition::PartitionKind;
use super::decision_value_landscape::{best_action, landscape_row};
use super::policy_artifact::{DecisionRule, FactorPredicate, PolicyArtifact};
use super::recommendation_outcome::RecommendationRow;
use super::DecisionAction;

pub const COMPARISON_CONTRACT_ID: &str = "csp006c3c.comparative_review.1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PairwiseOutcome {
    SearchTwoBetter,
    SearchOneBetter,
    Tie,
}

#[derive(Debug, Clone, Serialize)]
pub struct ActionCounts {
    pub n_long: u32,
    pub n_short: u32,
    pub n_no_trade: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct SymbolSliceMix {
    pub instrument: String,
    pub slice: String,
    pub search_one: ActionCounts,
    pub search_two: ActionCounts,
}

#[derive(Debug, Clone, Serialize)]
pub struct SymbolEvaluationValue {
    pub instrument: String,
    pub search_one_v: f64,
    pub search_two_v: f64,
    pub delta_v: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct PairwiseRow {
    pub timestamp: String,
    pub instrument: String,
    pub partition: PartitionKind,
    pub trend_state: String,
    pub momentum_state: String,
    pub volatility_state: String,
    pub raw_return: f64,
    pub search_one_action: DecisionAction,
    pub search_one_v: f64,
    pub search_one_regret: f64,
    pub search_two_action: DecisionAction,
    pub search_two_v: f64,
    pub search_two_regret: f64,
    pub best_action: DecisionAction,
    pub pairwise: PairwiseOutcome,
}

#[derive(Debug, Clone, Serialize)]
pub struct PairwiseSummary {
    pub n: u32,
    pub search_two_better: u32,
    pub search_one_better: u32,
    pub tie: u32,
    pub mean_delta_v: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConversionRow {
    pub timestamp: String,
    pub instrument: String,
    pub partition: PartitionKind,
    pub trend_state: String,
    pub momentum_state: String,
    pub volatility_state: String,
    pub raw_return: f64,
    pub search_two_action: DecisionAction,
    pub search_two_v: f64,
    pub best_action: DecisionAction,
    pub search_one_regret: f64,
    pub search_two_regret: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConversionSummary {
    pub n_search_one_no_trade: u32,
    pub n_converted_to_long: u32,
    pub n_converted_to_short: u32,
    pub n_still_no_trade: u32,
    pub mean_search_two_v: f64,
    pub n_search_two_unique_best: u32,
    pub n_search_two_better_than_search_one: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct RuleInspection {
    pub index: usize,
    pub action: DecisionAction,
    pub concepts: Vec<String>,
    pub contradictory: bool,
    pub shadowed: bool,
    pub n_fired: u32,
    pub first_match_reachable: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ComparativeReport {
    pub contract_id: String,
    pub search_one_artifact_hash: String,
    pub search_two_artifact_hash: String,
    pub n_rows: u32,
    pub action_matrix: Vec<SymbolSliceMix>,
    pub evaluation_value_by_symbol: Vec<SymbolEvaluationValue>,
    pub pairwise_all: PairwiseSummary,
    pub pairwise_evaluation: PairwiseSummary,
    pub no_trade_conversion: ConversionSummary,
    pub conversion_rows: Vec<ConversionRow>,
    pub pairwise_rows: Vec<PairwiseRow>,
    pub search_two_rules: Vec<RuleInspection>,
    pub unmatched_action: DecisionAction,
    pub unmatched_fires: u32,
    pub trend_neutral_rows: u32,
    pub used_as_coralys_fitness: bool,
    pub search_three_authorized: bool,
}

fn key(row: &RecommendationRow) -> (String, String) {
    (row.instrument.clone(), row.timestamp.clone())
}

fn counts(rows: &[&RecommendationRow]) -> ActionCounts {
    ActionCounts {
        n_long: rows
            .iter()
            .filter(|r| r.recommendation == DecisionAction::Long)
            .count() as u32,
        n_short: rows
            .iter()
            .filter(|r| r.recommendation == DecisionAction::Short)
            .count() as u32,
        n_no_trade: rows
            .iter()
            .filter(|r| r.recommendation == DecisionAction::NoTrade)
            .count() as u32,
    }
}

fn slice_name(kind: Option<PartitionKind>) -> String {
    match kind {
        Some(PartitionKind::Development) => "development".into(),
        Some(PartitionKind::Selection) => "selection".into(),
        Some(PartitionKind::Evaluation) => "evaluation".into(),
        None => "all".into(),
    }
}

fn mean(xs: &[f64]) -> f64 {
    if xs.is_empty() {
        0.0
    } else {
        xs.iter().sum::<f64>() / xs.len() as f64
    }
}

fn pairwise_of(rows: &[PairwiseRow]) -> PairwiseSummary {
    let mut two = 0u32;
    let mut one = 0u32;
    let mut tie = 0u32;
    let mut deltas = Vec::new();
    for row in rows {
        match row.pairwise {
            PairwiseOutcome::SearchTwoBetter => two += 1,
            PairwiseOutcome::SearchOneBetter => one += 1,
            PairwiseOutcome::Tie => tie += 1,
        }
        deltas.push(row.search_two_v - row.search_one_v);
    }
    PairwiseSummary {
        n: rows.len() as u32,
        search_two_better: two,
        search_one_better: one,
        tie,
        mean_delta_v: mean(&deltas),
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

fn rule_matches_labels(rule: &DecisionRule, trend: &str, momentum: &str, volatility: &str) -> bool {
    rule.when
        .iter()
        .all(|p| label_matches(p, trend, momentum, volatility))
}

fn rule_contradictory(rule: &DecisionRule) -> bool {
    let mut vol_present = false;
    let mut vol_absent = false;
    for p in &rule.when {
        if p.concept == "Volatility" {
            match p.present {
                Some(true) => vol_present = true,
                Some(false) => vol_absent = true,
                None => {}
            }
        }
    }
    vol_present && vol_absent
}

pub fn compare_sealed_recommendations(
    search_one: &[RecommendationRow],
    search_two: &[RecommendationRow],
    search_two_artifact: &PolicyArtifact,
) -> Result<ComparativeReport, String> {
    if search_one.len() != 273 || search_two.len() != 273 {
        return Err(format!(
            "expected 273 rows each, found {} and {}",
            search_one.len(),
            search_two.len()
        ));
    }
    if search_one
        .iter()
        .any(|r| r.policy_artifact_hash != RESEARCH_DISCOVERY_ARTIFACT_HASH)
    {
        return Err("left matrix is not Search #1".into());
    }
    if search_two
        .iter()
        .any(|r| r.policy_artifact_hash != RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH)
    {
        return Err("right matrix is not Search #2".into());
    }
    if search_two_artifact.artifact_hash != RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH {
        return Err("Search #2 artifact hash mismatch".into());
    }

    let two_by_key: BTreeMap<_, _> = search_two.iter().map(|r| (key(r), r)).collect();
    let mut pairwise_rows = Vec::new();
    let mut conversion_rows = Vec::new();
    let mut n_search_one_nt = 0u32;
    let mut n_to_long = 0u32;
    let mut n_to_short = 0u32;
    let mut n_still_nt = 0u32;
    let mut conversion_v = Vec::new();
    let mut n_conv_unique = 0u32;
    let mut n_conv_better = 0u32;
    let mut fire = vec![0u32; search_two_artifact.rules.len()];
    let mut unmatched_fires = 0u32;
    let mut trend_neutral = 0u32;

    for one in search_one {
        let two = two_by_key
            .get(&key(one))
            .ok_or_else(|| format!("Search #2 missing {} {}", one.instrument, one.timestamp))?;
        if one.partition != two.partition {
            return Err("partition mismatch on joined row".into());
        }
        let one_l = landscape_row(one).ok_or("Search #1 row missing return")?;
        let two_l = landscape_row(two).ok_or("Search #2 row missing return")?;
        let pairwise = if two_l.recommended_value > one_l.recommended_value {
            PairwiseOutcome::SearchTwoBetter
        } else if one_l.recommended_value > two_l.recommended_value {
            PairwiseOutcome::SearchOneBetter
        } else {
            PairwiseOutcome::Tie
        };
        if one.trend_state == "Neutral" {
            trend_neutral += 1;
        }
        let mut fired = None;
        for (i, rule) in search_two_artifact.rules.iter().enumerate() {
            if rule_matches_labels(
                rule,
                &two.trend_state,
                &two.momentum_state,
                &two.volatility_state,
            ) {
                fired = Some(i);
                break;
            }
        }
        match fired {
            Some(i) => fire[i] += 1,
            None => unmatched_fires += 1,
        }
        pairwise_rows.push(PairwiseRow {
            timestamp: one.timestamp.clone(),
            instrument: one.instrument.clone(),
            partition: one.partition,
            trend_state: one.trend_state.clone(),
            momentum_state: one.momentum_state.clone(),
            volatility_state: one.volatility_state.clone(),
            raw_return: one_l.raw_forward_return,
            search_one_action: one.recommendation,
            search_one_v: one_l.recommended_value,
            search_one_regret: one_l.regret,
            search_two_action: two.recommendation,
            search_two_v: two_l.recommended_value,
            search_two_regret: two_l.regret,
            best_action: best_action(one_l.raw_forward_return),
            pairwise,
        });
        if one.recommendation == DecisionAction::NoTrade {
            n_search_one_nt += 1;
            match two.recommendation {
                DecisionAction::Long => n_to_long += 1,
                DecisionAction::Short => n_to_short += 1,
                DecisionAction::NoTrade => n_still_nt += 1,
            }
            conversion_v.push(two_l.recommended_value);
            if two_l.recommended_is_unique_best {
                n_conv_unique += 1;
            }
            if two_l.recommended_value > one_l.recommended_value {
                n_conv_better += 1;
            }
            conversion_rows.push(ConversionRow {
                timestamp: one.timestamp.clone(),
                instrument: one.instrument.clone(),
                partition: one.partition,
                trend_state: one.trend_state.clone(),
                momentum_state: one.momentum_state.clone(),
                volatility_state: one.volatility_state.clone(),
                raw_return: one_l.raw_forward_return,
                search_two_action: two.recommendation,
                search_two_v: two_l.recommended_value,
                best_action: best_action(one_l.raw_forward_return),
                search_one_regret: one_l.regret,
                search_two_regret: two_l.regret,
            });
        }
    }

    let mut action_matrix = Vec::new();
    for ticker in RESEARCH_UNIVERSE {
        for kind in [
            Some(PartitionKind::Development),
            Some(PartitionKind::Selection),
            Some(PartitionKind::Evaluation),
            None,
        ] {
            let one: Vec<_> = search_one
                .iter()
                .filter(|r| r.instrument == *ticker)
                .filter(|r| kind.map(|k| r.partition == k).unwrap_or(true))
                .collect();
            let two: Vec<_> = search_two
                .iter()
                .filter(|r| r.instrument == *ticker)
                .filter(|r| kind.map(|k| r.partition == k).unwrap_or(true))
                .collect();
            action_matrix.push(SymbolSliceMix {
                instrument: (*ticker).to_string(),
                slice: slice_name(kind),
                search_one: counts(&one),
                search_two: counts(&two),
            });
        }
    }

    let mut evaluation_value_by_symbol = Vec::new();
    for ticker in RESEARCH_UNIVERSE {
        let one: Vec<f64> = pairwise_rows
            .iter()
            .filter(|r| r.instrument == *ticker && r.partition == PartitionKind::Evaluation)
            .map(|r| r.search_one_v)
            .collect();
        let two: Vec<f64> = pairwise_rows
            .iter()
            .filter(|r| r.instrument == *ticker && r.partition == PartitionKind::Evaluation)
            .map(|r| r.search_two_v)
            .collect();
        let search_one_v = mean(&one);
        let search_two_v = mean(&two);
        evaluation_value_by_symbol.push(SymbolEvaluationValue {
            instrument: (*ticker).to_string(),
            search_one_v,
            search_two_v,
            delta_v: search_two_v - search_one_v,
        });
    }

    let eval_rows: Vec<_> = pairwise_rows
        .iter()
        .filter(|r| r.partition == PartitionKind::Evaluation)
        .cloned()
        .collect();

    let mut search_two_rules = Vec::new();
    for (i, rule) in search_two_artifact.rules.iter().enumerate() {
        let contradictory = rule_contradictory(rule);
        let shadowed = fire[i] == 0 && !contradictory;
        search_two_rules.push(RuleInspection {
            index: i,
            action: rule.action,
            concepts: rule.when.iter().map(|p| p.concept.clone()).collect(),
            contradictory,
            shadowed,
            n_fired: fire[i],
            first_match_reachable: fire[i] > 0,
        });
    }

    Ok(ComparativeReport {
        contract_id: COMPARISON_CONTRACT_ID.to_string(),
        search_one_artifact_hash: RESEARCH_DISCOVERY_ARTIFACT_HASH.to_string(),
        search_two_artifact_hash: RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH.to_string(),
        n_rows: 273,
        action_matrix,
        evaluation_value_by_symbol,
        pairwise_all: pairwise_of(&pairwise_rows),
        pairwise_evaluation: pairwise_of(&eval_rows),
        no_trade_conversion: ConversionSummary {
            n_search_one_no_trade: n_search_one_nt,
            n_converted_to_long: n_to_long,
            n_converted_to_short: n_to_short,
            n_still_no_trade: n_still_nt,
            mean_search_two_v: mean(&conversion_v),
            n_search_two_unique_best: n_conv_unique,
            n_search_two_better_than_search_one: n_conv_better,
        },
        conversion_rows,
        pairwise_rows,
        search_two_rules,
        unmatched_action: search_two_artifact.unmatched_action,
        unmatched_fires,
        trend_neutral_rows: trend_neutral,
        used_as_coralys_fitness: false,
        search_three_authorized: false,
    })
}

pub fn render_comparison(report: &ComparativeReport) -> String {
    let mut out = String::from("# CS-P-006-C.3-C — Search #1 vs Search #2\n\n");
    out.push_str("Sealed-artifact review only. Neither policy was modified. Search #3 is not authorized.\n\n");
    out.push_str(&format!(
        "- Search #1: `{}`\n- Search #2: `{}`\n- rows: {}\n\n",
        report.search_one_artifact_hash, report.search_two_artifact_hash, report.n_rows
    ));
    out.push_str("## Pairwise on the same 273 rows\n\n");
    out.push_str(&format!(
        "| Slice | n | Search #2 better | Search #1 better | Tie | Mean ΔV |\n|---|---:|---:|---:|---:|---:|\n| all | {} | {} | {} | {} | {:.6} |\n| evaluation | {} | {} | {} | {} | {:.6} |\n\n",
        report.pairwise_all.n,
        report.pairwise_all.search_two_better,
        report.pairwise_all.search_one_better,
        report.pairwise_all.tie,
        report.pairwise_all.mean_delta_v,
        report.pairwise_evaluation.n,
        report.pairwise_evaluation.search_two_better,
        report.pairwise_evaluation.search_one_better,
        report.pairwise_evaluation.tie,
        report.pairwise_evaluation.mean_delta_v
    ));
    out.push_str("## Evaluation value by symbol\n\n");
    out.push_str("| Symbol | Search #1 V | Search #2 V | ΔV |\n|---|---:|---:|---:|\n");
    for row in &report.evaluation_value_by_symbol {
        out.push_str(&format!(
            "| {} | {:.4}% | {:.4}% | {:+.4}% |\n",
            row.instrument,
            100.0 * row.search_one_v,
            100.0 * row.search_two_v,
            100.0 * row.delta_v
        ));
    }
    out.push_str("\n## NO_TRADE conversion\n\n");
    let c = &report.no_trade_conversion;
    out.push_str(&format!(
        "Search #1 stood aside on {} rows. Search #2 converted {} to LONG and {} to SHORT; {} remained NO_TRADE.\n",
        c.n_search_one_no_trade, c.n_converted_to_long, c.n_converted_to_short, c.n_still_no_trade
    ));
    out.push_str(&format!(
        "On those rows, mean Search #2 V = {:.6}; Search #2 unique-best {} / {}; Search #2 better than Search #1 {} / {}.\n",
        c.mean_search_two_v,
        c.n_search_two_unique_best,
        c.n_search_one_no_trade,
        c.n_search_two_better_than_search_one,
        c.n_search_one_no_trade
    ));
    out.push_str(&format!(
        "Trend Neutral appears on {} of 273 rows.\n\n",
        report.trend_neutral_rows
    ));
    out.push_str("## Search #2 first-match rules\n\n");
    out.push_str("| # | Action | Concepts | Fired | Reachable | Contradictory | Shadowed |\n|---|---|---|---:|---|---|---|\n");
    for rule in &report.search_two_rules {
        out.push_str(&format!(
            "| {} | {:?} | {} | {} | {} | {} | {} |\n",
            rule.index,
            rule.action,
            rule.concepts.join("+"),
            rule.n_fired,
            rule.first_match_reachable,
            rule.contradictory,
            rule.shadowed
        ));
    }
    out.push_str(&format!(
        "\nUnmatched action: {:?}. Unmatched fires: {}. Search #3 is not authorized. Unique-best is diagnostic, not fitness.\n",
        report.unmatched_action, report.unmatched_fires
    ));
    out
}
