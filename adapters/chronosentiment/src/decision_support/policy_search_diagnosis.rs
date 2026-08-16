//! Post-search diagnosis of a sealed CS-P-006-C artifact.
//!
//! Does not evolve, mutate, or select. Does not feed results back to Coralys.
//! Evaluation numbers here are holdout diagnosis of Search #1, not search inputs.

use std::collections::BTreeMap;

use serde::Serialize;

use super::csp006_protocol::RESEARCH_UNIVERSE;
use super::dataset_partition::PartitionKind;
use super::observation_value::ObservationSlice;
use super::policy::{ensure_factor, factors_from_profile};
use super::policy_artifact::{first_match_action, PolicyArtifact, CERTIFIED_INPUT_CONCEPTS};
use super::DecisionAction;

#[derive(Debug, Clone, Serialize)]
pub struct ActionCounts {
    pub long: u32,
    pub short: u32,
    pub no_trade: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct InstrumentBreakdown {
    pub instrument: String,
    pub n_rows: u32,
    pub n_long: u32,
    pub n_short: u32,
    pub n_no_trade: u32,
    pub n_traded: u32,
    pub mean_signed_traded_return: f64,
    pub mean_raw_when_bearish: Option<f64>,
    pub n_bearish: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct StateOccupancy {
    pub trend: BTreeMap<String, u32>,
    pub momentum: BTreeMap<String, u32>,
    pub volatility: BTreeMap<String, u32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BearishAttractiveness {
    pub n_bearish: u32,
    pub n_other: u32,
    pub mean_raw_return_when_bearish: Option<f64>,
    pub mean_raw_return_when_other: Option<f64>,
    pub long_payoff_if_bearish: Option<f64>,
    pub short_payoff_if_bearish: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SliceDiagnosis {
    pub kind: PartitionKind,
    pub n_rows: usize,
    pub actions: ActionCounts,
    pub instruments: Vec<InstrumentBreakdown>,
    pub occupancy: StateOccupancy,
    pub bearish: BearishAttractiveness,
    pub mean_signed_traded_return: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct RepresentationAccess {
    pub factory_samples_trend: bool,
    pub factory_samples_momentum: bool,
    pub factory_samples_volatility: bool,
    pub factory_samples_conjunctions: bool,
    pub factory_samples_long: bool,
    pub factory_samples_short: bool,
    pub factory_samples_no_trade: bool,
    pub selected_uses_trend: bool,
    pub selected_uses_momentum: bool,
    pub selected_uses_volatility: bool,
    pub selected_uses_conjunction: bool,
    pub selected_emits_long: bool,
    pub selected_emits_short: bool,
    pub selected_emits_no_trade: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ArchiveLimitation {
    pub n_generation_bests_recorded: usize,
    pub n_unique_generation_best_genomes: usize,
    pub n_candidates_presented_to_selection: usize,
    pub population_median_recorded: bool,
    pub population_worst_recorded: bool,
    pub population_diversity_recorded: bool,
    pub development_best_genome_rules_archived: bool,
    pub note: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SearchDiagnosis {
    pub artifact_hash: String,
    pub genome_identity: String,
    pub representation: RepresentationAccess,
    pub archive: ArchiveLimitation,
    pub development: SliceDiagnosis,
    pub selection: SliceDiagnosis,
    pub evaluation: SliceDiagnosis,
}

pub fn diagnose_sealed_artifact(
    artifact: &PolicyArtifact,
    development: &ObservationSlice,
    selection: &ObservationSlice,
    evaluation: &ObservationSlice,
    n_generation_bests: usize,
    n_unique_generation_bests: usize,
    n_selection_candidates: usize,
) -> Result<SearchDiagnosis, String> {
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
    Ok(SearchDiagnosis {
        artifact_hash: artifact.artifact_hash.clone(),
        genome_identity: super::policy_genome::RuleListGenome {
            rules: artifact.rules.clone(),
            unmatched_action: artifact.unmatched_action,
        }
        .identity_hash(),
        representation: representation_access(artifact),
        archive: ArchiveLimitation {
            n_generation_bests_recorded: n_generation_bests,
            n_unique_generation_best_genomes: n_unique_generation_bests,
            n_candidates_presented_to_selection: n_selection_candidates,
            population_median_recorded: false,
            population_worst_recorded: false,
            population_diversity_recorded: false,
            development_best_genome_rules_archived: false,
            note: "Search #1 recorded generation-best identity and fitness only. Median, worst, and population diversity were not persisted. The development-best genome rules were not serialized.".to_string(),
        },
        development: diagnose_slice(artifact, development),
        selection: diagnose_slice(artifact, selection),
        evaluation: diagnose_slice(artifact, evaluation),
    })
}

fn representation_access(artifact: &PolicyArtifact) -> RepresentationAccess {
    let uses_trend = artifact
        .rules
        .iter()
        .any(|r| r.when.iter().any(|p| p.concept == "Trend"));
    let uses_momentum = artifact
        .rules
        .iter()
        .any(|r| r.when.iter().any(|p| p.concept == "Momentum"));
    let uses_vol = artifact
        .rules
        .iter()
        .any(|r| r.when.iter().any(|p| p.concept == "Volatility"));
    let uses_and = artifact.rules.iter().any(|r| r.when.len() > 1);
    let rule_actions: Vec<DecisionAction> = artifact.rules.iter().map(|r| r.action).collect();
    RepresentationAccess {
        factory_samples_trend: true,
        factory_samples_momentum: true,
        factory_samples_volatility: true,
        factory_samples_conjunctions: true,
        factory_samples_long: true,
        factory_samples_short: true,
        factory_samples_no_trade: true,
        selected_uses_trend: uses_trend,
        selected_uses_momentum: uses_momentum,
        selected_uses_volatility: uses_vol,
        selected_uses_conjunction: uses_and,
        selected_emits_long: rule_actions.contains(&DecisionAction::Long)
            || artifact.unmatched_action == DecisionAction::Long,
        selected_emits_short: rule_actions.contains(&DecisionAction::Short)
            || artifact.unmatched_action == DecisionAction::Short,
        selected_emits_no_trade: rule_actions.contains(&DecisionAction::NoTrade)
            || artifact.unmatched_action == DecisionAction::NoTrade,
    }
}

fn diagnose_slice(artifact: &PolicyArtifact, slice: &ObservationSlice) -> SliceDiagnosis {
    let mut actions = ActionCounts {
        long: 0,
        short: 0,
        no_trade: 0,
    };
    let mut occupancy = StateOccupancy {
        trend: BTreeMap::new(),
        momentum: BTreeMap::new(),
        volatility: BTreeMap::new(),
    };
    let mut bearish_raw = Vec::new();
    let mut other_raw = Vec::new();
    let mut per_name: BTreeMap<String, Vec<(DecisionAction, Option<f64>, bool, Option<f64>)>> =
        BTreeMap::new();

    for row in &slice.rows {
        let action = first_match_action(&artifact.rules, artifact.unmatched_action, &row.profile);
        match action {
            DecisionAction::Long => actions.long += 1,
            DecisionAction::Short => actions.short += 1,
            DecisionAction::NoTrade => actions.no_trade += 1,
        }
        let (trend, momentum, vol) = tmv_labels(&row.profile);
        *occupancy.trend.entry(trend.clone()).or_insert(0) += 1;
        *occupancy.momentum.entry(momentum).or_insert(0) += 1;
        *occupancy.volatility.entry(vol).or_insert(0) += 1;
        let bearish = trend == "Bearish";
        if let Some(raw) = row.instrument_return {
            if bearish {
                bearish_raw.push(raw);
            } else {
                other_raw.push(raw);
            }
        }
        per_name.entry(row.instrument.clone()).or_default().push((
            action,
            row.instrument_return,
            bearish,
            row.instrument_return,
        ));
    }

    let instruments: Vec<InstrumentBreakdown> = RESEARCH_UNIVERSE
        .iter()
        .map(|ticker| {
            let rows = per_name.get(*ticker).cloned().unwrap_or_default();
            instrument_breakdown((*ticker).to_string(), &rows)
        })
        .collect();
    let mean_signed = if instruments.is_empty() {
        0.0
    } else {
        instruments
            .iter()
            .map(|i| i.mean_signed_traded_return)
            .sum::<f64>()
            / instruments.len() as f64
    };

    SliceDiagnosis {
        kind: slice.kind,
        n_rows: slice.rows.len(),
        actions,
        instruments,
        occupancy,
        bearish: BearishAttractiveness {
            n_bearish: bearish_raw.len() as u32,
            n_other: other_raw.len() as u32,
            mean_raw_return_when_bearish: mean_opt(&bearish_raw),
            mean_raw_return_when_other: mean_opt(&other_raw),
            long_payoff_if_bearish: mean_opt(&bearish_raw),
            short_payoff_if_bearish: mean_opt(&bearish_raw).map(|v| -v),
        },
        mean_signed_traded_return: mean_signed,
    }
}

fn instrument_breakdown(
    instrument: String,
    rows: &[(DecisionAction, Option<f64>, bool, Option<f64>)],
) -> InstrumentBreakdown {
    let mut n_long = 0;
    let mut n_short = 0;
    let mut n_no_trade = 0;
    let mut traded = Vec::new();
    let mut bearish_raw = Vec::new();
    for (action, ret, bearish, raw) in rows {
        match action {
            DecisionAction::Long => n_long += 1,
            DecisionAction::Short => n_short += 1,
            DecisionAction::NoTrade => n_no_trade += 1,
        }
        if *bearish {
            if let Some(v) = raw {
                bearish_raw.push(*v);
            }
        }
        match (*action, ret) {
            (DecisionAction::Long, Some(v)) => traded.push(*v),
            (DecisionAction::Short, Some(v)) => traded.push(-*v),
            _ => {}
        }
    }
    InstrumentBreakdown {
        instrument,
        n_rows: rows.len() as u32,
        n_long,
        n_short,
        n_no_trade,
        n_traded: traded.len() as u32,
        mean_signed_traded_return: mean_opt(&traded).unwrap_or(0.0),
        mean_raw_when_bearish: mean_opt(&bearish_raw),
        n_bearish: bearish_raw.len() as u32,
    }
}

fn tmv_labels(profile: &crate::reasoning::assessment::AssessmentProfile) -> (String, String, String) {
    let mut factors = factors_from_profile(profile);
    for concept in CERTIFIED_INPUT_CONCEPTS {
        ensure_factor(&mut factors, concept);
    }
    let label = |concept: &str| {
        match factors.iter().find(|f| f.concept == concept) {
            Some(f) if f.present => f
                .direction
                .clone()
                .unwrap_or_else(|| "present".to_string()),
            _ => "absent".to_string(),
        }
    };
    (label("Trend"), label("Momentum"), label("Volatility"))
}

fn mean_opt(xs: &[f64]) -> Option<f64> {
    if xs.is_empty() {
        None
    } else {
        Some(xs.iter().sum::<f64>() / xs.len() as f64)
    }
}

pub fn render_diagnosis(report: &SearchDiagnosis) -> String {
    let mut md = String::from("# CS-P-006-C Search #1 — post-search diagnosis\n\n");
    md.push_str("Diagnosis of the **already sealed** Search #1 artifact. Coralys was not re-run. Evaluation figures are holdout diagnosis, not search feedback.\n\n");
    md.push_str(&format!("- artifact_hash: `{}`\n", report.artifact_hash));
    md.push_str(&format!("- genome identity: `{}`\n\n", report.genome_identity));

    md.push_str("## 1. Search-space utilization\n\n");
    md.push_str("The factory can sample Trend, Momentum, Volatility (presence), conjunctions, LONG, SHORT, and NO_TRADE. The **selected** artifact uses only Trend=Bearish → LONG; unmatched NO_TRADE.\n\n");
    md.push_str("| Capability | Factory can sample | Selected artifact uses |\n");
    md.push_str("|------------|--------------------|------------------------|\n");
    md.push_str(&format!(
        "| Trend | {} | {} |\n",
        report.representation.factory_samples_trend, report.representation.selected_uses_trend
    ));
    md.push_str(&format!(
        "| Momentum | {} | {} |\n",
        report.representation.factory_samples_momentum,
        report.representation.selected_uses_momentum
    ));
    md.push_str(&format!(
        "| Volatility | {} | {} |\n",
        report.representation.factory_samples_volatility,
        report.representation.selected_uses_volatility
    ));
    md.push_str(&format!(
        "| Conjunctions | {} | {} |\n",
        report.representation.factory_samples_conjunctions,
        report.representation.selected_uses_conjunction
    ));
    md.push_str(&format!(
        "| LONG | {} | {} |\n",
        report.representation.factory_samples_long, report.representation.selected_emits_long
    ));
    md.push_str(&format!(
        "| SHORT | {} | {} |\n",
        report.representation.factory_samples_short, report.representation.selected_emits_short
    ));
    md.push_str(&format!(
        "| NO_TRADE | {} | {} |\n\n",
        report.representation.factory_samples_no_trade,
        report.representation.selected_emits_no_trade
    ));

    md.push_str("## 2. Population archive limitation\n\n");
    md.push_str(&format!("{}\n\n", report.archive.note));
    md.push_str(&format!(
        "- generation bests recorded: {}\n",
        report.archive.n_generation_bests_recorded
    ));
    md.push_str(&format!(
        "- unique generation-best genomes: {}\n",
        report.archive.n_unique_generation_best_genomes
    ));
    md.push_str(&format!(
        "- candidates presented to selection: {}\n",
        report.archive.n_candidates_presented_to_selection
    ));
    md.push_str(&format!(
        "- population median recorded: {}\n",
        report.archive.population_median_recorded
    ));
    md.push_str(&format!(
        "- population worst recorded: {}\n",
        report.archive.population_worst_recorded
    ));
    md.push_str(&format!(
        "- population diversity recorded: {}\n\n",
        report.archive.population_diversity_recorded
    ));

    md.push_str("## 3. Fitness trajectory (from Search #1 evidence)\n\n");
    md.push_str("See `search_evidence.json`. Recorded best jumped once (generation 2) and then stayed flat. Average rose from ~0.0018 toward ~0.012 and never reached the recorded best. Median/worst/diversity were not persisted, so early population collapse cannot be proven or disproven from the archive.\n\n");

    md.push_str("## 4–6. Sealed-policy decomposition\n\n");
    for slice in [
        &report.development,
        &report.selection,
        &report.evaluation,
    ] {
        let role = match slice.kind {
            PartitionKind::Development => "development (search-visible)",
            PartitionKind::Selection => "selection (search-visible)",
            PartitionKind::Evaluation => "evaluation (holdout diagnosis only)",
        };
        md.push_str(&format!("### {}\n\n", role));
        md.push_str(&format!(
            "- actions: LONG {} / SHORT {} / NO_TRADE {}\n",
            slice.actions.long, slice.actions.short, slice.actions.no_trade
        ));
        md.push_str(&format!(
            "- mean signed traded return: {:.6}\n",
            slice.mean_signed_traded_return
        ));
        md.push_str(&format!(
            "- Trend occupancy: {:?}\n",
            slice.occupancy.trend
        ));
        md.push_str(&format!(
            "- bearish n={} mean raw 20D {:?} (LONG payoff); SHORT payoff {:?}\n",
            slice.bearish.n_bearish,
            slice.bearish.mean_raw_return_when_bearish,
            slice.bearish.short_payoff_if_bearish
        ));
        md.push_str(&format!(
            "- other n={} mean raw 20D {:?}\n\n",
            slice.bearish.n_other, slice.bearish.mean_raw_return_when_other
        ));
        md.push_str("| instrument | n | LONG | SHORT | NO_TRADE | mean signed traded | n_bearish | mean raw when bearish |\n");
        md.push_str("|------------|---|------|-------|----------|--------------------|-----------|-----------------------|\n");
        for row in &slice.instruments {
            md.push_str(&format!(
                "| {} | {} | {} | {} | {} | {:.6} | {} | {:?} |\n",
                row.instrument,
                row.n_rows,
                row.n_long,
                row.n_short,
                row.n_no_trade,
                row.mean_signed_traded_return,
                row.n_bearish,
                row.mean_raw_when_bearish
            ));
        }
        md.push('\n');
    }
    md.push_str("Do not retune the genome from these tables. Search #2 is not authorized by this diagnosis.\n");
    md
}
