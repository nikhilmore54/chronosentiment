//! CS-P-006-C.2-R — recommendation vs realized outcome for a sealed artifact.
//!
//! ChronoSentiment measures what happened after each recommendation.
//! Does not evolve, select, or feed results back to Coralys.
//! Evaluation rows are scored here as holdout diagnosis, not as search fitness.

use serde::{Deserialize, Serialize};

use super::csp006_protocol::RESEARCH_UNIVERSE;
use super::dataset_partition::PartitionKind;
use super::observation_value::{ObservationSlice, DISCOVERY_HORIZON_DAYS};
use super::policy::{ensure_factor, factors_from_profile};
use super::policy_artifact::{first_match_action, PolicyArtifact, CERTIFIED_INPUT_CONCEPTS};
use super::DecisionAction;

pub const SCORECARD_CONTRACT_ID: &str = "csp006c2r.recommendation_outcome.1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DirectionalCall {
    Correct,
    Incorrect,
    Flat,
    NotApplicable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendationRow {
    pub timestamp: String,
    pub instrument: String,
    pub partition: PartitionKind,
    pub trend_state: String,
    pub momentum_state: String,
    pub volatility_state: String,
    pub recommendation: DecisionAction,
    pub actual_forward_return: Option<f64>,
    pub return_contribution: Option<f64>,
    pub directional_call: DirectionalCall,
    pub long_alternative_return: Option<f64>,
    pub short_alternative_return: Option<f64>,
    pub no_trade_winning_alternative: Option<DecisionAction>,
    pub horizon_days: u32,
    pub policy_artifact_hash: String,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct ActionAccuracy {
    pub n: u32,
    pub n_correct: u32,
    pub n_incorrect: u32,
    pub n_flat: u32,
    pub n_unavailable: u32,
    pub directional_accuracy: Option<f64>,
    pub mean_signed_return: Option<f64>,
    pub min_signed_return: Option<f64>,
    pub p25_signed_return: Option<f64>,
    pub median_signed_return: Option<f64>,
    pub p75_signed_return: Option<f64>,
    pub max_signed_return: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct NoTradeOpportunity {
    pub n: u32,
    pub n_unavailable: u32,
    pub n_market_up: u32,
    pub n_market_down: u32,
    pub n_flat: u32,
    pub mean_raw_return: Option<f64>,
    pub mean_long_alternative: Option<f64>,
    pub mean_short_alternative: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SliceScorecard {
    pub partition: PartitionKind,
    pub n_recommendations: u32,
    pub long: ActionAccuracy,
    pub short: ActionAccuracy,
    pub no_trade: NoTradeOpportunity,
    pub protocol_mean_signed_traded_return: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct InstrumentScorecard {
    pub instrument: String,
    pub n_long: u32,
    pub n_short: u32,
    pub n_no_trade: u32,
    pub long_correct: u32,
    pub long_incorrect: u32,
    pub mean_signed_when_traded: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RecommendationScorecard {
    pub contract_id: String,
    pub policy_artifact_hash: String,
    pub horizon_days: u32,
    pub search_two_authorized: bool,
    pub coralys_feedback: bool,
    pub n_recommendations: u32,
    pub overall: SliceScorecard,
    pub development: SliceScorecard,
    pub selection: SliceScorecard,
    pub evaluation: SliceScorecard,
    pub instruments: Vec<InstrumentScorecard>,
    pub generalization: String,
}

pub fn tmv_labels(profile: &crate::reasoning::assessment::AssessmentProfile) -> (String, String, String) {
    let mut factors = factors_from_profile(profile);
    for concept in CERTIFIED_INPUT_CONCEPTS {
        ensure_factor(&mut factors, concept);
    }
    let label = |concept: &str| match factors.iter().find(|f| f.concept == concept) {
        Some(f) if f.present => f
            .direction
            .clone()
            .unwrap_or_else(|| "present".to_string()),
        _ => "absent".to_string(),
    };
    (label("Trend"), label("Momentum"), label("Volatility"))
}

fn directional_call(action: DecisionAction, raw: Option<f64>) -> DirectionalCall {
    let Some(ret) = raw else {
        return DirectionalCall::NotApplicable;
    };
    match action {
        DecisionAction::NoTrade => DirectionalCall::NotApplicable,
        DecisionAction::Long if ret > 0.0 => DirectionalCall::Correct,
        DecisionAction::Long if ret < 0.0 => DirectionalCall::Incorrect,
        DecisionAction::Short if ret < 0.0 => DirectionalCall::Correct,
        DecisionAction::Short if ret > 0.0 => DirectionalCall::Incorrect,
        DecisionAction::Long | DecisionAction::Short => DirectionalCall::Flat,
    }
}

fn signed_contribution(action: DecisionAction, raw: Option<f64>) -> Option<f64> {
    match (action, raw) {
        (DecisionAction::Long, Some(r)) => Some(r),
        (DecisionAction::Short, Some(r)) => Some(-r),
        _ => None,
    }
}

fn winning_alternative(action: DecisionAction, raw: Option<f64>) -> Option<DecisionAction> {
    if action != DecisionAction::NoTrade {
        return None;
    }
    match raw {
        Some(r) if r > 0.0 => Some(DecisionAction::Long),
        Some(r) if r < 0.0 => Some(DecisionAction::Short),
        _ => None,
    }
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

fn action_accuracy(rows: &[&RecommendationRow], action: DecisionAction) -> ActionAccuracy {
    let chosen: Vec<&&RecommendationRow> = rows
        .iter()
        .filter(|r| r.recommendation == action)
        .collect();
    let mut signed = Vec::new();
    let mut n_correct = 0u32;
    let mut n_incorrect = 0u32;
    let mut n_flat = 0u32;
    let mut n_unavailable = 0u32;
    for r in &chosen {
        match r.directional_call {
            DirectionalCall::Correct => n_correct += 1,
            DirectionalCall::Incorrect => n_incorrect += 1,
            DirectionalCall::Flat => n_flat += 1,
            DirectionalCall::NotApplicable => {
                if r.actual_forward_return.is_none() {
                    n_unavailable += 1;
                }
            }
        }
        if let Some(v) = r.return_contribution {
            signed.push(v);
        }
    }
    signed.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let decided = n_correct + n_incorrect;
    ActionAccuracy {
        n: chosen.len() as u32,
        n_correct,
        n_incorrect,
        n_flat,
        n_unavailable,
        directional_accuracy: if decided == 0 {
            None
        } else {
            Some(n_correct as f64 / decided as f64)
        },
        mean_signed_return: mean(&signed),
        min_signed_return: signed.first().copied(),
        p25_signed_return: percentile(&signed, 0.25),
        median_signed_return: percentile(&signed, 0.50),
        p75_signed_return: percentile(&signed, 0.75),
        max_signed_return: signed.last().copied(),
    }
}

fn no_trade_opportunity(rows: &[&RecommendationRow]) -> NoTradeOpportunity {
    let chosen: Vec<&&RecommendationRow> = rows
        .iter()
        .filter(|r| r.recommendation == DecisionAction::NoTrade)
        .collect();
    let mut raws = Vec::new();
    let mut longs = Vec::new();
    let mut shorts = Vec::new();
    let mut n_up = 0u32;
    let mut n_down = 0u32;
    let mut n_flat = 0u32;
    let mut n_unavailable = 0u32;
    for r in &chosen {
        match r.actual_forward_return {
            None => n_unavailable += 1,
            Some(v) => {
                raws.push(v);
                longs.push(v);
                shorts.push(-v);
                if v > 0.0 {
                    n_up += 1;
                } else if v < 0.0 {
                    n_down += 1;
                } else {
                    n_flat += 1;
                }
            }
        }
    }
    NoTradeOpportunity {
        n: chosen.len() as u32,
        n_unavailable,
        n_market_up: n_up,
        n_market_down: n_down,
        n_flat,
        mean_raw_return: mean(&raws),
        mean_long_alternative: mean(&longs),
        mean_short_alternative: mean(&shorts),
    }
}

fn protocol_mean(rows: &[&RecommendationRow]) -> f64 {
    let mut per_instrument = Vec::new();
    for ticker in RESEARCH_UNIVERSE {
        let traded: Vec<f64> = rows
            .iter()
            .filter(|r| r.instrument == *ticker)
            .filter_map(|r| r.return_contribution)
            .collect();
        per_instrument.push(if traded.is_empty() {
            0.0
        } else {
            traded.iter().sum::<f64>() / traded.len() as f64
        });
    }
    per_instrument.iter().sum::<f64>() / per_instrument.len() as f64
}

fn slice_scorecard(kind: PartitionKind, rows: &[&RecommendationRow]) -> SliceScorecard {
    SliceScorecard {
        partition: kind,
        n_recommendations: rows.len() as u32,
        long: action_accuracy(rows, DecisionAction::Long),
        short: action_accuracy(rows, DecisionAction::Short),
        no_trade: no_trade_opportunity(rows),
        protocol_mean_signed_traded_return: protocol_mean(rows),
    }
}

fn instrument_scorecard(instrument: &str, rows: &[&RecommendationRow]) -> InstrumentScorecard {
    let mine: Vec<&&RecommendationRow> = rows
        .iter()
        .filter(|r| r.instrument == instrument)
        .collect();
    let traded: Vec<f64> = mine.iter().filter_map(|r| r.return_contribution).collect();
    InstrumentScorecard {
        instrument: instrument.to_string(),
        n_long: mine
            .iter()
            .filter(|r| r.recommendation == DecisionAction::Long)
            .count() as u32,
        n_short: mine
            .iter()
            .filter(|r| r.recommendation == DecisionAction::Short)
            .count() as u32,
        n_no_trade: mine
            .iter()
            .filter(|r| r.recommendation == DecisionAction::NoTrade)
            .count() as u32,
        long_correct: mine
            .iter()
            .filter(|r| {
                r.recommendation == DecisionAction::Long
                    && r.directional_call == DirectionalCall::Correct
            })
            .count() as u32,
        long_incorrect: mine
            .iter()
            .filter(|r| {
                r.recommendation == DecisionAction::Long
                    && r.directional_call == DirectionalCall::Incorrect
            })
            .count() as u32,
        mean_signed_when_traded: mean(&traded),
    }
}

pub fn score_recommendations(
    artifact: &PolicyArtifact,
    development: &ObservationSlice,
    selection: &ObservationSlice,
    evaluation: &ObservationSlice,
) -> Result<(Vec<RecommendationRow>, RecommendationScorecard), String> {
    if artifact.artifact_hash.is_empty() {
        return Err("artifact is not sealed".into());
    }
    if development.kind != PartitionKind::Development {
        return Err("development slice required".into());
    }
    if selection.kind != PartitionKind::Selection {
        return Err("selection slice required".into());
    }
    if evaluation.kind != PartitionKind::Evaluation {
        return Err("evaluation slice required".into());
    }
    let mut rows = Vec::new();
    for (kind, slice) in [
        (PartitionKind::Development, development),
        (PartitionKind::Selection, selection),
        (PartitionKind::Evaluation, evaluation),
    ] {
        for row in &slice.rows {
            let action = first_match_action(&artifact.rules, artifact.unmatched_action, &row.profile);
            let (trend, momentum, volatility) = tmv_labels(&row.profile);
            rows.push(RecommendationRow {
                timestamp: row.as_of.to_rfc3339(),
                instrument: row.instrument.clone(),
                partition: kind,
                trend_state: trend,
                momentum_state: momentum,
                volatility_state: volatility,
                recommendation: action,
                actual_forward_return: row.instrument_return,
                return_contribution: signed_contribution(action, row.instrument_return),
                directional_call: directional_call(action, row.instrument_return),
                long_alternative_return: row.instrument_return,
                short_alternative_return: row.instrument_return.map(|r| -r),
                no_trade_winning_alternative: winning_alternative(action, row.instrument_return),
                horizon_days: DISCOVERY_HORIZON_DAYS,
                policy_artifact_hash: artifact.artifact_hash.clone(),
            });
        }
    }
    let refs: Vec<&RecommendationRow> = rows.iter().collect();
    let development_rows: Vec<&RecommendationRow> = refs
        .iter()
        .copied()
        .filter(|r| r.partition == PartitionKind::Development)
        .collect();
    let selection_rows: Vec<&RecommendationRow> = refs
        .iter()
        .copied()
        .filter(|r| r.partition == PartitionKind::Selection)
        .collect();
    let evaluation_rows: Vec<&RecommendationRow> = refs
        .iter()
        .copied()
        .filter(|r| r.partition == PartitionKind::Evaluation)
        .collect();
    let overall = slice_scorecard(PartitionKind::Development, &refs);
    let development_card = slice_scorecard(PartitionKind::Development, &development_rows);
    let selection_card = slice_scorecard(PartitionKind::Selection, &selection_rows);
    let evaluation_card = slice_scorecard(PartitionKind::Evaluation, &evaluation_rows);
    let generalization = if evaluation_card.protocol_mean_signed_traded_return < 0.0
        && development_card.protocol_mean_signed_traded_return > 0.0
        && selection_card.protocol_mean_signed_traded_return > 0.0
    {
        "FAIL".to_string()
    } else if evaluation_card.protocol_mean_signed_traded_return > 0.0 {
        "HOLD".to_string()
    } else {
        "INCONCLUSIVE".to_string()
    };
    let instruments = RESEARCH_UNIVERSE
        .iter()
        .map(|ticker| instrument_scorecard(ticker, &refs))
        .collect();
    let n_recommendations = refs.len() as u32;
    let overall_long = overall.long;
    let overall_short = overall.short;
    let overall_no_trade = overall.no_trade;
    let overall_protocol = protocol_mean(&refs);
    drop((refs, development_rows, selection_rows, evaluation_rows));
    Ok((
        rows,
        RecommendationScorecard {
            contract_id: SCORECARD_CONTRACT_ID.to_string(),
            policy_artifact_hash: artifact.artifact_hash.clone(),
            horizon_days: DISCOVERY_HORIZON_DAYS,
            search_two_authorized: false,
            coralys_feedback: false,
            n_recommendations,
            overall: SliceScorecard {
                partition: PartitionKind::Development,
                n_recommendations,
                long: overall_long,
                short: overall_short,
                no_trade: overall_no_trade,
                protocol_mean_signed_traded_return: overall_protocol,
            },
            development: development_card,
            selection: selection_card,
            evaluation: evaluation_card,
            instruments,
            generalization,
        },
    ))
}

pub fn render_scorecard(card: &RecommendationScorecard) -> String {
    let mut out = String::from("# Search #1 recommendation outcome\n\n");
    out.push_str("Sealed PolicyArtifact applied to every certified decision point. Not Search #2.\n\n");
    out.push_str(&format!(
        "- artifact: `{}`\n- horizon: {} calendar days\n- recommendations: {}\n- generalization: **{}**\n\n",
        card.policy_artifact_hash, card.horizon_days, card.n_recommendations, card.generalization
    ));
    out.push_str("| Slice | n | LONG | SHORT | NO_TRADE | LONG correct | LONG incorrect | LONG accuracy | Protocol mean |\n");
    out.push_str("|-------|---|------|-------|----------|--------------|----------------|---------------|---------------|\n");
    for (name, s) in [
        ("all", &card.overall),
        ("development", &card.development),
        ("selection", &card.selection),
        ("evaluation", &card.evaluation),
    ] {
        out.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} | {} | {:.6} |\n",
            name,
            s.n_recommendations,
            s.long.n,
            s.short.n,
            s.no_trade.n,
            s.long.n_correct,
            s.long.n_incorrect,
            s.long
                .directional_accuracy
                .map(|v| format!("{:.1}%", 100.0 * v))
                .unwrap_or_else(|| "—".to_string()),
            s.protocol_mean_signed_traded_return
        ));
    }
    out.push_str("\nNO_TRADE is not scored as correct or incorrect. Evaluation was not fed to Coralys. Search #2 is not authorized.\n");
    out
}
