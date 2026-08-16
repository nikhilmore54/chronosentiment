//! CS-P-006-C.2-D — decision-value landscape for sealed Search #1 recommendations.
//!
//! Measurement contract is specified here. No ±X% band is invented from data.
//! Advantage and unique-best fields are observational. They are not a Coralys
//! fitness function and must not be turned into one without a later methodology freeze.
//! Does not evolve, select, or feed evaluation to Coralys.

use serde::Serialize;

use super::dataset_partition::PartitionKind;
use super::recommendation_outcome::RecommendationRow;
use super::DecisionAction;

pub const LANDSCAPE_CONTRACT_ID: &str = "csp006c2d.decision_value.1";

/// Frozen action values on the certified 20-day observation path.
/// Costs are not certified, so they are not subtracted.
pub fn action_value(action: DecisionAction, raw_forward_return: f64) -> f64 {
    match action {
        DecisionAction::Long => raw_forward_return,
        DecisionAction::Short => -raw_forward_return,
        DecisionAction::NoTrade => 0.0,
    }
}

pub fn best_action(raw_forward_return: f64) -> DecisionAction {
    if raw_forward_return > 0.0 {
        DecisionAction::Long
    } else if raw_forward_return < 0.0 {
        DecisionAction::Short
    } else {
        DecisionAction::NoTrade
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct DecisionValueRow {
    pub timestamp: String,
    pub instrument: String,
    pub partition: PartitionKind,
    pub recommendation: DecisionAction,
    pub raw_forward_return: f64,
    pub value_long: f64,
    pub value_short: f64,
    pub value_no_trade: f64,
    pub recommended_value: f64,
    pub best_action: DecisionAction,
    pub best_value: f64,
    pub regret: f64,
    pub advantage_vs_no_trade: f64,
    pub advantage_vs_long: f64,
    pub advantage_vs_short: f64,
    pub recommended_is_unique_best: bool,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct ContinuousSummary {
    pub n: u32,
    pub mean: Option<f64>,
    pub median: Option<f64>,
    pub p25: Option<f64>,
    pub p75: Option<f64>,
    pub min: Option<f64>,
    pub max: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SliceLandscape {
    pub partition: PartitionKind,
    pub n: u32,
    pub n_acted: u32,
    pub n_stood_aside: u32,
    pub recommended_value: ContinuousSummary,
    pub regret: ContinuousSummary,
    pub acted_advantage_vs_no_trade: ContinuousSummary,
    pub no_trade_opportunity_cost: ContinuousSummary,
    pub share_acted_better_than_standing_aside: Option<f64>,
    pub share_recommended_is_unique_best: Option<f64>,
    pub mean_recommended_value: f64,
    pub mean_regret: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct DecisionValueLandscape {
    pub contract_id: String,
    pub policy_artifact_hash: String,
    pub search_two_authorized: bool,
    pub coralys_feedback: bool,
    pub borderline_band_frozen: bool,
    pub used_as_coralys_fitness: bool,
    pub cost_term_present: bool,
    pub n_rows: u32,
    pub overall: SliceLandscape,
    pub development: SliceLandscape,
    pub selection: SliceLandscape,
    pub evaluation: SliceLandscape,
}

fn percentile(sorted: &[f64], q: f64) -> Option<f64> {
    if sorted.is_empty() {
        return None;
    }
    let idx = ((sorted.len() as f64 - 1.0) * q).round() as usize;
    Some(sorted[idx.min(sorted.len() - 1)])
}

fn mean(xs: &[f64]) -> Option<f64> {
    if xs.is_empty() {
        None
    } else {
        Some(xs.iter().sum::<f64>() / xs.len() as f64)
    }
}

fn summarize(xs: &[f64]) -> ContinuousSummary {
    let mut ordered = xs.to_vec();
    ordered.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    ContinuousSummary {
        n: xs.len() as u32,
        mean: mean(xs),
        median: percentile(&ordered, 0.50),
        p25: percentile(&ordered, 0.25),
        p75: percentile(&ordered, 0.75),
        min: ordered.first().copied(),
        max: ordered.last().copied(),
    }
}

pub fn landscape_row(row: &RecommendationRow) -> Option<DecisionValueRow> {
    let raw = row.actual_forward_return?;
    let value_long = action_value(DecisionAction::Long, raw);
    let value_short = action_value(DecisionAction::Short, raw);
    let value_no_trade = action_value(DecisionAction::NoTrade, raw);
    let recommended_value = action_value(row.recommendation, raw);
    let best = best_action(raw);
    let best_value = action_value(best, raw);
    let unique_best = [DecisionAction::Long, DecisionAction::Short, DecisionAction::NoTrade]
        .into_iter()
        .filter(|&action| action != row.recommendation)
        .all(|action| recommended_value > action_value(action, raw));
    Some(DecisionValueRow {
        timestamp: row.timestamp.clone(),
        instrument: row.instrument.clone(),
        partition: row.partition,
        recommendation: row.recommendation,
        raw_forward_return: raw,
        value_long,
        value_short,
        value_no_trade,
        recommended_value,
        best_action: best,
        best_value,
        regret: best_value - recommended_value,
        advantage_vs_no_trade: recommended_value - value_no_trade,
        advantage_vs_long: recommended_value - value_long,
        advantage_vs_short: recommended_value - value_short,
        recommended_is_unique_best: unique_best,
    })
}

fn slice_landscape(kind: PartitionKind, rows: &[&DecisionValueRow]) -> SliceLandscape {
    let values: Vec<f64> = rows.iter().map(|r| r.recommended_value).collect();
    let regrets: Vec<f64> = rows.iter().map(|r| r.regret).collect();
    let acted: Vec<&&DecisionValueRow> = rows
        .iter()
        .filter(|r| r.recommendation != DecisionAction::NoTrade)
        .collect();
    let stood: Vec<&&DecisionValueRow> = rows
        .iter()
        .filter(|r| r.recommendation == DecisionAction::NoTrade)
        .collect();
    let acted_adv: Vec<f64> = acted.iter().map(|r| r.advantage_vs_no_trade).collect();
    let stood_cost: Vec<f64> = stood.iter().map(|r| r.regret).collect();
    let n_acted_better = acted.iter().filter(|r| r.advantage_vs_no_trade > 0.0).count();
    let n_unique_best = rows.iter().filter(|r| r.recommended_is_unique_best).count();
    SliceLandscape {
        partition: kind,
        n: rows.len() as u32,
        n_acted: acted.len() as u32,
        n_stood_aside: stood.len() as u32,
        recommended_value: summarize(&values),
        regret: summarize(&regrets),
        acted_advantage_vs_no_trade: summarize(&acted_adv),
        no_trade_opportunity_cost: summarize(&stood_cost),
        share_acted_better_than_standing_aside: if acted.is_empty() {
            None
        } else {
            Some(n_acted_better as f64 / acted.len() as f64)
        },
        share_recommended_is_unique_best: if rows.is_empty() {
            None
        } else {
            Some(n_unique_best as f64 / rows.len() as f64)
        },
        mean_recommended_value: mean(&values).unwrap_or(0.0),
        mean_regret: mean(&regrets).unwrap_or(0.0),
    }
}

pub fn analyze_landscape(
    artifact_hash: &str,
    recommendations: &[RecommendationRow],
) -> Result<(Vec<DecisionValueRow>, DecisionValueLandscape), String> {
    if artifact_hash.is_empty() {
        return Err("artifact is not sealed".into());
    }
    let rows: Vec<DecisionValueRow> = recommendations.iter().filter_map(landscape_row).collect();
    if rows.is_empty() {
        return Err("no realized outcomes to form a landscape".into());
    }
    let refs: Vec<&DecisionValueRow> = rows.iter().collect();
    let development: Vec<&DecisionValueRow> = refs
        .iter()
        .copied()
        .filter(|r| r.partition == PartitionKind::Development)
        .collect();
    let selection: Vec<&DecisionValueRow> = refs
        .iter()
        .copied()
        .filter(|r| r.partition == PartitionKind::Selection)
        .collect();
    let evaluation: Vec<&DecisionValueRow> = refs
        .iter()
        .copied()
        .filter(|r| r.partition == PartitionKind::Evaluation)
        .collect();
    let overall = slice_landscape(PartitionKind::Development, &refs);
    let development_l = slice_landscape(PartitionKind::Development, &development);
    let selection_l = slice_landscape(PartitionKind::Selection, &selection);
    let evaluation_l = slice_landscape(PartitionKind::Evaluation, &evaluation);
    let n_rows = rows.len() as u32;
    Ok((
        rows,
        DecisionValueLandscape {
            contract_id: LANDSCAPE_CONTRACT_ID.to_string(),
            policy_artifact_hash: artifact_hash.to_string(),
            search_two_authorized: false,
            coralys_feedback: false,
            borderline_band_frozen: false,
            used_as_coralys_fitness: false,
            cost_term_present: false,
            n_rows,
            overall: SliceLandscape {
                n: n_rows,
                ..overall
            },
            development: development_l,
            selection: selection_l,
            evaluation: evaluation_l,
        },
    ))
}

pub fn render_landscape(card: &DecisionValueLandscape) -> String {
    let mut out = String::from("# Search #1 decision-value landscape\n\n");
    out.push_str("Existing 273 recommendations only. Not Search #2. No borderline band is frozen.\n\n");
    out.push_str(&format!(
        "- artifact: `{}`\n- rows: {}\n- mean recommended value (all, NO_TRADE=0): {:.6}\n- mean regret vs best alternative: {:.6}\n\n",
        card.policy_artifact_hash,
        card.n_rows,
        card.overall.mean_recommended_value,
        card.overall.mean_regret
    ));
    out.push_str("| Slice | n | Acted | Stood aside | Mean value | Mean regret | Acted better than NO_TRADE | Unique best |\n");
    out.push_str("|-------|---|-------|-------------|------------|-------------|----------------------------|-------------|\n");
    for (name, s) in [
        ("all", &card.overall),
        ("development", &card.development),
        ("selection", &card.selection),
        ("evaluation", &card.evaluation),
    ] {
        out.push_str(&format!(
            "| {} | {} | {} | {} | {:.6} | {:.6} | {} | {} |\n",
            name,
            s.n,
            s.n_acted,
            s.n_stood_aside,
            s.mean_recommended_value,
            s.mean_regret,
            s.share_acted_better_than_standing_aside
                .map(|v| format!("{:.1}%", 100.0 * v))
                .unwrap_or_else(|| "—".to_string()),
            s.share_recommended_is_unique_best
                .map(|v| format!("{:.1}%", 100.0 * v))
                .unwrap_or_else(|| "—".to_string())
        ));
    }
    out.push_str("\nAdvantage versus alternatives is observational. It is not Coralys fitness.\n");
    out.push_str("Evaluation is diagnostic. Coralys receives no feedback. Search #2 is not authorized.\n");
    out
}
