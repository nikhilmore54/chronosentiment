//! CS-P-006-N — decision-value measurement harness.
//!
//! Applies CS-P-006-M.1 to a sealed artifact. Does not evolve.
//! ProtocolValue can be built only from per-instrument means of V.
//! Regret, unique_best, and advantage cannot construct ProtocolValue.
//! Symbol × slice matrices are a required contract, not a visualization.

use std::collections::BTreeMap;

use serde::Serialize;

use super::csp006_protocol::{RESEARCH_DISCOVERY_ARTIFACT_HASH, RESEARCH_UNIVERSE};
use super::dataset_partition::PartitionKind;
use super::decision_value_landscape::{action_value, landscape_row, DecisionValueRow};
use super::recommendation_outcome::RecommendationRow;
use super::DecisionAction;

pub const HARNESS_CONTRACT_ID: &str = "csp006n.decision_value_harness.1";
pub const C3_AUTHORIZED: bool = false;
pub const ROWS_PER_SYMBOL_PER_SLICE: u32 = 13;

/// Search-admissible scalar. Only constructible from per-instrument V means.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ProtocolValue {
    pub value: f64,
    pub instrument_means: BTreeMap<String, f64>,
    pub n_instruments: u32,
}

impl ProtocolValue {
    pub fn from_per_instrument_v(
        per_instrument: &BTreeMap<String, Vec<f64>>,
    ) -> Result<Self, String> {
        if RESEARCH_UNIVERSE
            .iter()
            .any(|ticker| !per_instrument.contains_key(*ticker))
        {
            return Err("protocol value requires all seven certified instruments".into());
        }
        if per_instrument
            .keys()
            .any(|k| !RESEARCH_UNIVERSE.iter().any(|t| t == k))
        {
            return Err("protocol value contains an instrument outside the certified universe".into());
        }
        let mut instrument_means = BTreeMap::new();
        for ticker in RESEARCH_UNIVERSE {
            let vs = per_instrument
                .get(ticker)
                .ok_or_else(|| format!("missing instrument {ticker}"))?;
            if vs.is_empty() {
                return Err(format!("{ticker} has no decisions; not a silent 0"));
            }
            instrument_means.insert(
                (*ticker).to_string(),
                vs.iter().sum::<f64>() / vs.len() as f64,
            );
        }
        let value = instrument_means.values().sum::<f64>() / instrument_means.len() as f64;
        Ok(Self {
            value,
            n_instruments: instrument_means.len() as u32,
            instrument_means,
        })
    }
}

/// ChronoSentiment diagnostics. There is no conversion into ProtocolValue.
#[derive(Debug, Clone, Serialize)]
pub struct DecisionDiagnostics {
    pub mean_regret: f64,
    pub unique_best_share: f64,
    pub mean_opportunity_cost: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct SymbolDecisionDistribution {
    pub instrument: String,
    pub slice: String,
    pub n_long: u32,
    pub n_short: u32,
    pub n_no_trade: u32,
    pub total: u32,
    pub pct_long: f64,
    pub pct_short: f64,
    pub pct_no_trade: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct SymbolDecisionValue {
    pub instrument: String,
    pub slice: String,
    pub n: u32,
    pub mean_v: f64,
    pub median_v: f64,
    pub mean_regret: f64,
    pub unique_best_n: u32,
    pub unique_best_share: f64,
    pub opportunity_cost: f64,
    pub mean_v_when_acted: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SliceHarness {
    pub slice: String,
    pub n: u32,
    pub protocol_value: ProtocolValue,
    pub diagnostics: DecisionDiagnostics,
    pub search_admissible: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct HarnessReport {
    pub contract_id: String,
    pub policy_artifact_hash: String,
    pub c3_authorized: bool,
    pub search_two_authorized: bool,
    pub used_as_coralys_fitness: bool,
    pub cost_term_present: bool,
    pub n_rows: u32,
    pub table_a_decision_distribution: Vec<SymbolDecisionDistribution>,
    pub table_b_decision_value: Vec<SymbolDecisionValue>,
    pub development: SliceHarness,
    pub selection: SliceHarness,
    pub evaluation: SliceHarness,
    pub all: SliceHarness,
}

fn median(xs: &[f64]) -> f64 {
    let mut ordered = xs.to_vec();
    ordered.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let mid = (ordered.len() - 1) / 2;
    ordered[mid]
}

fn mean(xs: &[f64]) -> f64 {
    if xs.is_empty() {
        0.0
    } else {
        xs.iter().sum::<f64>() / xs.len() as f64
    }
}

fn per_instrument_v(rows: &[&DecisionValueRow]) -> BTreeMap<String, Vec<f64>> {
    let mut map: BTreeMap<String, Vec<f64>> = BTreeMap::new();
    for ticker in RESEARCH_UNIVERSE {
        map.insert((*ticker).to_string(), Vec::new());
    }
    for row in rows {
        map.entry(row.instrument.clone())
            .or_default()
            .push(row.recommended_value);
    }
    map
}

fn diagnostics(rows: &[&DecisionValueRow]) -> DecisionDiagnostics {
    let regrets: Vec<f64> = rows.iter().map(|r| r.regret).collect();
    let unique = rows.iter().filter(|r| r.recommended_is_unique_best).count();
    let stood: Vec<f64> = rows
        .iter()
        .filter(|r| r.recommendation == DecisionAction::NoTrade)
        .map(|r| r.regret)
        .collect();
    DecisionDiagnostics {
        mean_regret: mean(&regrets),
        unique_best_share: if rows.is_empty() {
            0.0
        } else {
            unique as f64 / rows.len() as f64
        },
        mean_opportunity_cost: mean(&stood),
    }
}

fn slice_label(kind: Option<PartitionKind>) -> String {
    match kind {
        Some(PartitionKind::Development) => "development".into(),
        Some(PartitionKind::Selection) => "selection".into(),
        Some(PartitionKind::Evaluation) => "evaluation".into(),
        None => "all".into(),
    }
}

fn filter_rows<'a>(
    rows: &'a [DecisionValueRow],
    instrument: Option<&str>,
    kind: Option<PartitionKind>,
) -> Vec<&'a DecisionValueRow> {
    rows.iter()
        .filter(|r| instrument.map(|i| r.instrument == i).unwrap_or(true))
        .filter(|r| kind.map(|k| r.partition == k).unwrap_or(true))
        .collect()
}

fn table_a_row(
    instrument: &str,
    kind: Option<PartitionKind>,
    rows: &[&DecisionValueRow],
) -> SymbolDecisionDistribution {
    let n_long = rows
        .iter()
        .filter(|r| r.recommendation == DecisionAction::Long)
        .count() as u32;
    let n_short = rows
        .iter()
        .filter(|r| r.recommendation == DecisionAction::Short)
        .count() as u32;
    let n_no_trade = rows
        .iter()
        .filter(|r| r.recommendation == DecisionAction::NoTrade)
        .count() as u32;
    let total = rows.len() as u32;
    SymbolDecisionDistribution {
        instrument: instrument.to_string(),
        slice: slice_label(kind),
        n_long,
        n_short,
        n_no_trade,
        total,
        pct_long: if total == 0 {
            0.0
        } else {
            n_long as f64 / total as f64
        },
        pct_short: if total == 0 {
            0.0
        } else {
            n_short as f64 / total as f64
        },
        pct_no_trade: if total == 0 {
            0.0
        } else {
            n_no_trade as f64 / total as f64
        },
    }
}

fn table_b_row(
    instrument: &str,
    kind: Option<PartitionKind>,
    rows: &[&DecisionValueRow],
) -> SymbolDecisionValue {
    let values: Vec<f64> = rows.iter().map(|r| r.recommended_value).collect();
    let regrets: Vec<f64> = rows.iter().map(|r| r.regret).collect();
    let unique = rows.iter().filter(|r| r.recommended_is_unique_best).count() as u32;
    let stood: Vec<f64> = rows
        .iter()
        .filter(|r| r.recommendation == DecisionAction::NoTrade)
        .map(|r| r.regret)
        .collect();
    let acted: Vec<f64> = rows
        .iter()
        .filter(|r| r.recommendation != DecisionAction::NoTrade)
        .map(|r| r.recommended_value)
        .collect();
    SymbolDecisionValue {
        instrument: instrument.to_string(),
        slice: slice_label(kind),
        n: rows.len() as u32,
        mean_v: mean(&values),
        median_v: if values.is_empty() { 0.0 } else { median(&values) },
        mean_regret: mean(&regrets),
        unique_best_n: unique,
        unique_best_share: if rows.is_empty() {
            0.0
        } else {
            unique as f64 / rows.len() as f64
        },
        opportunity_cost: mean(&stood),
        mean_v_when_acted: if acted.is_empty() {
            None
        } else {
            Some(mean(&acted))
        },
    }
}

fn slice_harness(kind: Option<PartitionKind>, rows: &[&DecisionValueRow]) -> Result<SliceHarness, String> {
    let protocol = ProtocolValue::from_per_instrument_v(&per_instrument_v(rows))?;
    Ok(SliceHarness {
        slice: slice_label(kind),
        n: rows.len() as u32,
        protocol_value: protocol,
        diagnostics: diagnostics(rows),
        search_admissible: matches!(
            kind,
            Some(PartitionKind::Development) | Some(PartitionKind::Selection)
        ),
    })
}

/// Search-admissible protocol value. Evaluation is rejected.
pub fn search_admissible_protocol_value(
    rows: &[DecisionValueRow],
    kind: PartitionKind,
) -> Result<ProtocolValue, String> {
    if kind == PartitionKind::Evaluation {
        return Err("evaluation cannot influence search-admissible protocol value".into());
    }
    let filtered = filter_rows(rows, None, Some(kind));
    ProtocolValue::from_per_instrument_v(&per_instrument_v(&filtered))
}

pub fn measure_harness(
    artifact_hash: &str,
    recommendations: &[RecommendationRow],
) -> Result<(Vec<DecisionValueRow>, HarnessReport), String> {
    if artifact_hash != RESEARCH_DISCOVERY_ARTIFACT_HASH {
        return Err("harness identity-gates Search #1; refusing a different artifact".into());
    }
    if recommendations
        .iter()
        .any(|r| r.policy_artifact_hash != RESEARCH_DISCOVERY_ARTIFACT_HASH)
    {
        return Err("recommendation matrix is not Search #1".into());
    }
    measure_decision_ecology(artifact_hash, recommendations, false, false)
}

/// Measurement of a later sealed artifact. Search #1 stays on `measure_harness`.
pub fn measure_sealed_artifact(
    artifact_hash: &str,
    recommendations: &[RecommendationRow],
) -> Result<(Vec<DecisionValueRow>, HarnessReport), String> {
    if artifact_hash == RESEARCH_DISCOVERY_ARTIFACT_HASH {
        return Err("Search #1 measurement stays on measure_harness".into());
    }
    if artifact_hash.is_empty() {
        return Err("sealed artifact hash is required".into());
    }
    if recommendations
        .iter()
        .any(|r| r.policy_artifact_hash != artifact_hash)
    {
        return Err("recommendation matrix does not match the sealed artifact".into());
    }
    measure_decision_ecology(artifact_hash, recommendations, true, true)
}

fn measure_decision_ecology(
    artifact_hash: &str,
    recommendations: &[RecommendationRow],
    c3_authorized: bool,
    search_two_authorized: bool,
) -> Result<(Vec<DecisionValueRow>, HarnessReport), String> {
    let rows: Vec<DecisionValueRow> = recommendations.iter().filter_map(landscape_row).collect();
    if rows.len() != 273 {
        return Err(format!("expected 273 realized rows, found {}", rows.len()));
    }

    let mut table_a = Vec::new();
    let mut table_b = Vec::new();
    for ticker in RESEARCH_UNIVERSE {
        for kind in [
            Some(PartitionKind::Development),
            Some(PartitionKind::Selection),
            Some(PartitionKind::Evaluation),
            None,
        ] {
            let subset = filter_rows(&rows, Some(ticker), kind);
            if kind.is_some() && subset.len() as u32 != ROWS_PER_SYMBOL_PER_SLICE {
                return Err(format!(
                    "{ticker} {} must have {ROWS_PER_SYMBOL_PER_SLICE} decisions, found {}",
                    slice_label(kind),
                    subset.len()
                ));
            }
            if kind.is_none() && subset.len() != 39 {
                return Err(format!(
                    "{ticker} all-slice must have 39 decisions, found {}",
                    subset.len()
                ));
            }
            table_a.push(table_a_row(ticker, kind, &subset));
            table_b.push(table_b_row(ticker, kind, &subset));
        }
    }

    let development = slice_harness(
        Some(PartitionKind::Development),
        &filter_rows(&rows, None, Some(PartitionKind::Development)),
    )?;
    let selection = slice_harness(
        Some(PartitionKind::Selection),
        &filter_rows(&rows, None, Some(PartitionKind::Selection)),
    )?;
    let evaluation = slice_harness(
        Some(PartitionKind::Evaluation),
        &filter_rows(&rows, None, Some(PartitionKind::Evaluation)),
    )?;
    let all = slice_harness(None, &rows.iter().collect::<Vec<_>>())?;

    Ok((
        rows,
        HarnessReport {
            contract_id: HARNESS_CONTRACT_ID.to_string(),
            policy_artifact_hash: artifact_hash.to_string(),
            c3_authorized,
            search_two_authorized,
            used_as_coralys_fitness: false,
            cost_term_present: false,
            n_rows: 273,
            table_a_decision_distribution: table_a,
            table_b_decision_value: table_b,
            development,
            selection,
            evaluation,
            all,
        },
    ))
}

pub fn render_harness(report: &HarnessReport) -> String {
    let mut out = String::from("# CS-P-006-N decision-value harness\n\n");
    out.push_str("Measurement only. No evolution. C.3 is not authorized.\n\n");
    out.push_str(&format!(
        "- artifact: `{}`\n- rows: {}\n- C.3 authorized: {}\n- protocol V (all): {:.6}\n- protocol V (development, search-admissible): {:.6}\n- protocol V (selection, search-admissible): {:.6}\n- protocol V (evaluation, diagnostic): {:.6}\n\n",
        report.policy_artifact_hash,
        report.n_rows,
        report.c3_authorized,
        report.all.protocol_value.value,
        report.development.protocol_value.value,
        report.selection.protocol_value.value,
        report.evaluation.protocol_value.value
    ));

    out.push_str("## Table A — Decision distribution by symbol\n\n");
    out.push_str("| Symbol | Slice | LONG | SHORT | NO_TRADE | Total | % LONG | % SHORT | % NO_TRADE |\n");
    out.push_str("|--------|-------|-----:|------:|---------:|------:|-------:|--------:|-----------:|\n");
    for row in &report.table_a_decision_distribution {
        out.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {:.1}% | {:.1}% | {:.1}% |\n",
            row.instrument,
            row.slice,
            row.n_long,
            row.n_short,
            row.n_no_trade,
            row.total,
            100.0 * row.pct_long,
            100.0 * row.pct_short,
            100.0 * row.pct_no_trade
        ));
    }

    out.push_str("\n## Table B — Decision value by symbol\n\n");
    out.push_str("| Symbol | Slice | n | Mean V | Median V | Mean regret | Unique-best | Opp. cost | Mean V when acted |\n");
    out.push_str("|--------|-------|--:|-------:|---------:|------------:|------------:|----------:|------------------:|\n");
    for row in &report.table_b_decision_value {
        out.push_str(&format!(
            "| {} | {} | {} | {:.4}% | {:.4}% | {:.4}% | {} ({:.1}%) | {:.4}% | {} |\n",
            row.instrument,
            row.slice,
            row.n,
            100.0 * row.mean_v,
            100.0 * row.median_v,
            100.0 * row.mean_regret,
            row.unique_best_n,
            100.0 * row.unique_best_share,
            100.0 * row.opportunity_cost,
            row.mean_v_when_acted
                .map(|v| format!("{:.4}%", 100.0 * v))
                .unwrap_or_else(|| "—".to_string())
        ));
    }

    out.push_str("\n## Aggregate protocol scalar (mean of seven instrument means of V)\n\n");
    out.push_str("| Slice | Search-admissible | Protocol V | Mean regret | Unique-best |\n");
    out.push_str("|-------|-------------------|-----------:|------------:|------------:|\n");
    for s in [
        &report.development,
        &report.selection,
        &report.evaluation,
        &report.all,
    ] {
        out.push_str(&format!(
            "| {} | {} | {:.6} | {:.6} | {:.1}% |\n",
            s.slice,
            s.search_admissible,
            s.protocol_value.value,
            s.diagnostics.mean_regret,
            100.0 * s.diagnostics.unique_best_share
        ));
    }
    out.push_str("\nTable A is what the policy recommended. Table B is what those recommendations subsequently generated.\n");
    out.push_str("Regret and unique-best are diagnostics. They cannot construct ProtocolValue. C.3 is not authorized.\n");
    out
}

/// Search #1 traded-only mean, for contrast only. Not M.1 protocol value.
pub fn search_one_traded_mean(rows: &[&DecisionValueRow]) -> f64 {
    let mut means = Vec::new();
    for ticker in RESEARCH_UNIVERSE {
        let traded: Vec<f64> = rows
            .iter()
            .filter(|r| r.instrument == *ticker && r.recommendation != DecisionAction::NoTrade)
            .map(|r| r.recommended_value)
            .collect();
        means.push(if traded.is_empty() {
            0.0
        } else {
            mean(&traded)
        });
    }
    mean(&means)
}

pub fn action_values(raw: f64) -> (f64, f64, f64) {
    (
        action_value(DecisionAction::Long, raw),
        action_value(DecisionAction::Short, raw),
        action_value(DecisionAction::NoTrade, raw),
    )
}
