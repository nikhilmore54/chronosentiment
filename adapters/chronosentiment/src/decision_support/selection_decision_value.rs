//! CS-P-006-C.2-S — selection bottleneck and decision-value review.
//!
//! Uses the sealed Search #1 artifact and the C.2-O archive only.
//! Does not evolve, invent a borderline cutoff, or feed evaluation to Coralys.

use std::collections::BTreeMap;

use serde::Serialize;

use super::csp006_protocol::RESEARCH_UNIVERSE;
use super::dataset_partition::PartitionKind;
use super::observation_value::ObservationSlice;
use super::policy_artifact::first_match_action;
use super::policy_genome::RuleListGenome;
use super::search_observability::{SearchArchive, SerializedGenome};
use super::DecisionAction;

pub const REVIEW_CONTRACT_ID: &str = "csp006c2s.selection_decision_value.1";
pub const SELECTED_IDENTITY: &str =
    "d8363a93e5afe518b7a4cbb8f5c3ac59efcf396f0d318ccdae0dd683e9d730d3";
pub const DEVELOPMENT_BEST_IDENTITY: &str =
    "9eb80355dca7dca22e9218bd0285368291b9d3005698ce0f8b510605a1e6973b";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CandidateRole {
    Selected,
    DevelopmentBest,
    NearBest,
}

#[derive(Debug, Clone, Serialize)]
pub struct OutcomeDistribution {
    pub n_rows: u32,
    pub n_traded: u32,
    pub n_stood_aside: u32,
    pub n_unavailable: u32,
    pub mean_signed_traded: Option<f64>,
    pub median_signed_traded: Option<f64>,
    pub p25_signed_traded: Option<f64>,
    pub p75_signed_traded: Option<f64>,
    pub min_signed_traded: Option<f64>,
    pub max_signed_traded: Option<f64>,
    pub n_positive: u32,
    pub n_negative: u32,
    pub n_zero: u32,
    pub share_positive: Option<f64>,
    pub share_negative: Option<f64>,
    pub sum_simple_return: f64,
    pub compounded_simple_return: f64,
    pub max_drawdown: f64,
    pub protocol_mean: f64,
    pub no_trade_n: u32,
    pub no_trade_mean_raw: Option<f64>,
    pub no_trade_mean_long_alternative: Option<f64>,
    pub no_trade_mean_short_alternative: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CandidateReview {
    pub identity: String,
    pub role: CandidateRole,
    pub n_rules: usize,
    pub uses_trend: bool,
    pub uses_momentum: bool,
    pub uses_volatility: bool,
    pub emits_long: bool,
    pub emits_short: bool,
    pub emits_no_trade: bool,
    pub development: OutcomeDistribution,
    pub selection: OutcomeDistribution,
    pub evaluation: Option<OutcomeDistribution>,
    pub beats_selected_on_selection: bool,
    pub selection_without_mahabank: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct SelectionBottleneck {
    pub protocol_requires_one_sealed_candidate: bool,
    pub protocol_requires_generation_best_only: bool,
    pub implementation_pool: String,
    pub n_candidates_presented_to_selection: usize,
    pub n_unique_archived_genomes: usize,
    pub n_near_best_identities: usize,
    pub n_that_beat_selected_on_selection: usize,
    pub note: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct FitnessAdequacy {
    pub objective_is_mean_of_per_instrument_means: bool,
    pub no_trade_is_standing_aside: bool,
    pub untraded_instrument_contributes_zero: bool,
    pub accuracy_is_the_objective: bool,
    pub cost_term_present: bool,
    pub drawdown_term_present: bool,
    pub borderline_band_frozen: bool,
    pub horizon_days: u32,
    pub single_name_can_dominate: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct SelectionDecisionValueReport {
    pub contract_id: String,
    pub policy_artifact_hash: String,
    pub search_two_authorized: bool,
    pub coralys_feedback: bool,
    pub borderline_boundary_frozen: bool,
    pub bottleneck: SelectionBottleneck,
    pub fitness: FitnessAdequacy,
    pub selected: CandidateReview,
    pub development_best: CandidateReview,
    pub n_archived_candidates_reviewed: usize,
    pub n_momentum_rich_that_beat_selected_on_selection: usize,
}

fn genome_from(serialized: &SerializedGenome) -> RuleListGenome {
    RuleListGenome {
        rules: serialized.rules.clone(),
        unmatched_action: serialized.unmatched_action,
    }
}

fn uses(genome: &RuleListGenome, concept: &str) -> bool {
    genome
        .rules
        .iter()
        .any(|r| r.when.iter().any(|p| p.concept == concept))
}

fn emits(genome: &RuleListGenome, action: DecisionAction) -> bool {
    genome.unmatched_action == action || genome.rules.iter().any(|r| r.action == action)
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

fn protocol_mean(genome: &RuleListGenome, slice: &ObservationSlice, skip: Option<&str>) -> f64 {
    let mut per = Vec::new();
    for ticker in RESEARCH_UNIVERSE {
        if skip == Some(ticker) {
            continue;
        }
        let mut traded = Vec::new();
        for row in slice.rows.iter().filter(|r| r.instrument == *ticker) {
            let action = first_match_action(&genome.rules, genome.unmatched_action, &row.profile);
            if action == DecisionAction::NoTrade {
                continue;
            }
            if let Some(raw) = row.instrument_return {
                traded.push(if action == DecisionAction::Long {
                    raw
                } else {
                    -raw
                });
            }
        }
        per.push(if traded.is_empty() {
            0.0
        } else {
            traded.iter().sum::<f64>() / traded.len() as f64
        });
    }
    if per.is_empty() {
        0.0
    } else {
        per.iter().sum::<f64>() / per.len() as f64
    }
}

fn distribution(genome: &RuleListGenome, slice: &ObservationSlice) -> OutcomeDistribution {
    let mut events: Vec<(chrono::DateTime<chrono::Utc>, f64)> = Vec::new();
    let mut no_trade_raw = Vec::new();
    let mut n_stood = 0u32;
    let mut n_unavail = 0u32;
    for row in &slice.rows {
        let action = first_match_action(&genome.rules, genome.unmatched_action, &row.profile);
        match (action, row.instrument_return) {
            (DecisionAction::NoTrade, Some(raw)) => {
                n_stood += 1;
                no_trade_raw.push(raw);
            }
            (DecisionAction::NoTrade, None) => {
                n_stood += 1;
                n_unavail += 1;
            }
            (DecisionAction::Long, Some(raw)) => events.push((row.as_of, raw)),
            (DecisionAction::Short, Some(raw)) => events.push((row.as_of, -raw)),
            (_, None) => n_unavail += 1,
        }
    }
    events.sort_by_key(|(t, _)| *t);
    let signed: Vec<f64> = events.iter().map(|(_, r)| *r).collect();
    let mut ordered = signed.clone();
    ordered.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let n_positive = signed.iter().filter(|v| **v > 0.0).count() as u32;
    let n_negative = signed.iter().filter(|v| **v < 0.0).count() as u32;
    let n_zero = signed.iter().filter(|v| **v == 0.0).count() as u32;
    let mut wealth = 1.0;
    let mut peak = 1.0;
    let mut max_dd: f64 = 0.0;
    for v in &signed {
        wealth *= 1.0 + *v;
        if wealth > peak {
            peak = wealth;
        }
        if peak > 0.0 {
            max_dd = max_dd.max((peak - wealth) / peak);
        }
    }
    let no_trade_short: Vec<f64> = no_trade_raw.iter().map(|v| -*v).collect();
    OutcomeDistribution {
        n_rows: slice.rows.len() as u32,
        n_traded: signed.len() as u32,
        n_stood_aside: n_stood,
        n_unavailable: n_unavail,
        mean_signed_traded: mean(&signed),
        median_signed_traded: percentile(&ordered, 0.50),
        p25_signed_traded: percentile(&ordered, 0.25),
        p75_signed_traded: percentile(&ordered, 0.75),
        min_signed_traded: ordered.first().copied(),
        max_signed_traded: ordered.last().copied(),
        n_positive,
        n_negative,
        n_zero,
        share_positive: if signed.is_empty() {
            None
        } else {
            Some(n_positive as f64 / signed.len() as f64)
        },
        share_negative: if signed.is_empty() {
            None
        } else {
            Some(n_negative as f64 / signed.len() as f64)
        },
        sum_simple_return: signed.iter().sum(),
        compounded_simple_return: wealth - 1.0,
        max_drawdown: max_dd,
        protocol_mean: protocol_mean(genome, slice, None),
        no_trade_n: no_trade_raw.len() as u32,
        no_trade_mean_raw: mean(&no_trade_raw),
        no_trade_mean_long_alternative: mean(&no_trade_raw),
        no_trade_mean_short_alternative: mean(&no_trade_short),
    }
}

fn review_candidate(
    serialized: &SerializedGenome,
    role: CandidateRole,
    development: &ObservationSlice,
    selection: &ObservationSlice,
    evaluation: Option<&ObservationSlice>,
    selected_selection_fitness: f64,
) -> CandidateReview {
    let genome = genome_from(serialized);
    let sel = distribution(&genome, selection);
    CandidateReview {
        identity: serialized.identity.clone(),
        role,
        n_rules: genome.rules.len(),
        uses_trend: uses(&genome, "Trend"),
        uses_momentum: uses(&genome, "Momentum"),
        uses_volatility: uses(&genome, "Volatility"),
        emits_long: emits(&genome, DecisionAction::Long),
        emits_short: emits(&genome, DecisionAction::Short),
        emits_no_trade: emits(&genome, DecisionAction::NoTrade),
        development: distribution(&genome, development),
        selection: sel.clone(),
        evaluation: evaluation.map(|s| distribution(&genome, s)),
        beats_selected_on_selection: sel.protocol_mean > selected_selection_fitness,
        selection_without_mahabank: protocol_mean(&genome, selection, Some("MAHABANK.NS")),
    }
}

pub fn archived_genomes(archive: &SearchArchive) -> BTreeMap<String, SerializedGenome> {
    let mut out = BTreeMap::new();
    for g in &archive.generations {
        out.entry(g.generation_best.identity.clone())
            .or_insert_with(|| g.generation_best.clone());
        for nb in &g.near_best {
            out.entry(nb.identity.clone()).or_insert_with(|| nb.clone());
        }
    }
    out
}

pub fn review_selection(
    artifact_hash: &str,
    archive: &SearchArchive,
    development: &ObservationSlice,
    selection: &ObservationSlice,
    evaluation: &ObservationSlice,
) -> Result<SelectionDecisionValueReport, String> {
    if artifact_hash.is_empty() {
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
    let genomes = archived_genomes(archive);
    let selected_ser = genomes
        .get(SELECTED_IDENTITY)
        .ok_or("selected genome missing from C.2-O archive")?
        .clone();
    let dev_best_ser = genomes
        .get(DEVELOPMENT_BEST_IDENTITY)
        .ok_or("development-best genome missing from C.2-O archive")?
        .clone();
    let selected_sel = protocol_mean(&genome_from(&selected_ser), selection, None);
    let selected = review_candidate(
        &selected_ser,
        CandidateRole::Selected,
        development,
        selection,
        Some(evaluation),
        selected_sel,
    );
    let development_best = review_candidate(
        &dev_best_ser,
        CandidateRole::DevelopmentBest,
        development,
        selection,
        Some(evaluation),
        selected_sel,
    );
    let mut n_beat = 0usize;
    let mut n_momentum_beat = 0usize;
    for (id, ser) in &genomes {
        if id == SELECTED_IDENTITY {
            continue;
        }
        let cand = review_candidate(
            ser,
            if id == DEVELOPMENT_BEST_IDENTITY {
                CandidateRole::DevelopmentBest
            } else {
                CandidateRole::NearBest
            },
            development,
            selection,
            None,
            selected_sel,
        );
        if cand.beats_selected_on_selection {
            n_beat += 1;
            if cand.uses_momentum {
                n_momentum_beat += 1;
            }
        }
    }
    Ok(SelectionDecisionValueReport {
        contract_id: REVIEW_CONTRACT_ID.to_string(),
        policy_artifact_hash: artifact_hash.to_string(),
        search_two_authorized: false,
        coralys_feedback: false,
        borderline_boundary_frozen: false,
        bottleneck: SelectionBottleneck {
            protocol_requires_one_sealed_candidate: true,
            protocol_requires_generation_best_only: false,
            implementation_pool: "generation_history plus global_best".to_string(),
            n_candidates_presented_to_selection: 2,
            n_unique_archived_genomes: genomes.len(),
            n_near_best_identities: genomes.len().saturating_sub(1),
            n_that_beat_selected_on_selection: n_beat,
            note: "CS-P-006-B requires one sealed candidate after selection. It does not require that only generation-best genomes enter that comparison. Search #1 implemented the pool as unique generation-bests because Coralys MOGA returned generation_history, not the living population. C.2-O now holds near-best rules; this review inspects them without changing Search #1.".to_string(),
        },
        fitness: FitnessAdequacy {
            objective_is_mean_of_per_instrument_means: true,
            no_trade_is_standing_aside: true,
            untraded_instrument_contributes_zero: true,
            accuracy_is_the_objective: false,
            cost_term_present: false,
            drawdown_term_present: false,
            borderline_band_frozen: false,
            horizon_days: 20,
            single_name_can_dominate: true,
        },
        selected,
        development_best,
        n_archived_candidates_reviewed: genomes.len(),
        n_momentum_rich_that_beat_selected_on_selection: n_momentum_beat,
    })
}

pub fn render_review(report: &SelectionDecisionValueReport) -> String {
    let mut out = String::from("# Search #1 selection and decision-value review\n\n");
    out.push_str("Existing Search #1 evidence only. Not Search #2. No borderline cutoff is frozen.\n\n");
    out.push_str(&format!(
        "- artifact: `{}`\n- archived genomes reviewed: {}\n- presented to selection: {}\n- near-best that beat selected on selection protocol mean: {}\n- Momentum-rich among those: {}\n\n",
        report.policy_artifact_hash,
        report.n_archived_candidates_reviewed,
        report.bottleneck.n_candidates_presented_to_selection,
        report.bottleneck.n_that_beat_selected_on_selection,
        report.n_momentum_rich_that_beat_selected_on_selection
    ));
    out.push_str("Accuracy is not the objective. Evaluation was not used to choose a candidate. Search #2 is not authorized.\n");
    out
}
