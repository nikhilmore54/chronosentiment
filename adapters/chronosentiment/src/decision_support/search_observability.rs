//! CS-P-006-C.2 observability contract for a future search archive.
//!
//! This module does not evolve, select, or score policies. It states what
//! Search #1 failed to persist. It does not authorize Search #2.

use serde::{Deserialize, Serialize};

pub const OBSERVABILITY_CONTRACT_ID: &str = "csp006c2.search_observability.1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ObservabilityRequirement {
    pub id: &'static str,
    pub required: bool,
}

pub fn required_archive_fields() -> &'static [&'static str] {
    &[
        "unique_genome_count_by_generation",
        "median_fitness_by_generation",
        "worst_fitness_by_generation",
        "population_action_symbol_histogram",
        "population_factor_consumption_histogram",
        "serialized_generation_best_rules",
        "near_best_genomes",
        "selected_per_instrument_development",
        "selected_per_instrument_selection",
    ]
}

/// Fields Search #1 did persist. Not sufficient for convergence diagnosis.
pub fn search_one_recorded_fields() -> &'static [&'static str] {
    &[
        "generation_best_fitness",
        "average_fitness_history",
        "generation_lineage_identity_and_fitness",
        "n_candidates_considered_for_selection",
        "selected_aggregate_development",
        "selected_aggregate_selection",
    ]
}

pub fn missing_from_search_one() -> Vec<&'static str> {
    required_archive_fields().to_vec()
}

pub fn search_one_satisfies_observability() -> bool {
    missing_from_search_one().is_empty()
}

#[derive(Debug, Clone, Serialize)]
pub struct GapReview {
    pub contract_id: String,
    pub search_one_satisfies_contract: bool,
    pub missing: Vec<String>,
    pub volatility_presence_discriminates_on_s1: bool,
    pub volatility_encoding_chosen: bool,
    pub search_two_authorized: bool,
}

pub fn search_one_gap_review() -> GapReview {
    GapReview {
        contract_id: OBSERVABILITY_CONTRACT_ID.to_string(),
        search_one_satisfies_contract: search_one_satisfies_observability(),
        missing: missing_from_search_one()
            .into_iter()
            .map(|s| s.to_string())
            .collect(),
        volatility_presence_discriminates_on_s1: false,
        volatility_encoding_chosen: false,
        search_two_authorized: false,
    }
}

use std::collections::HashSet;
use std::sync::Mutex;

use coralys_moga::observatory::GenerationObserver;
use coralys_moga::traits::Evaluated;

use super::csp006_protocol::RESEARCH_UNIVERSE;
use super::dataset_partition::PartitionKind;
use super::observation_value::{score_genome, GenomeEvaluation, ObservationSlice};
use super::policy_artifact::first_match_action;
use super::policy_discovery::SelectedCandidate;
use super::policy_genome::RuleListGenome;
use super::DecisionAction;

const NEAR_BEST_EPS: f64 = 1e-9;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ActionSymbolHistogram {
    pub genomes_with_long: u32,
    pub genomes_with_short: u32,
    pub genomes_with_no_trade: u32,
    pub unmatched_long: u32,
    pub unmatched_short: u32,
    pub unmatched_no_trade: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FactorConsumptionHistogram {
    pub genomes_using_trend: u32,
    pub genomes_using_momentum: u32,
    pub genomes_using_volatility: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedGenome {
    pub identity: String,
    pub development_fitness: f64,
    pub rules: Vec<super::policy_artifact::DecisionRule>,
    pub unmatched_action: DecisionAction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationPopulationRecord {
    pub generation: usize,
    pub population_size: usize,
    pub unique_genome_count: usize,
    pub best_fitness: f64,
    pub median_fitness: f64,
    pub mean_fitness: f64,
    pub worst_fitness: f64,
    pub action_symbols: ActionSymbolHistogram,
    pub factor_consumption: FactorConsumptionHistogram,
    pub generation_best: SerializedGenome,
    pub near_best: Vec<SerializedGenome>,
    /// Living-population slots for this generation. Not offspring that failed to enter.
    #[serde(default)]
    pub living_slots: Vec<SerializedGenome>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OffspringEdge {
    pub generation: usize,
    pub parent_a_identity: String,
    pub parent_b_identity: String,
    pub child_identity: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstrumentScore {
    pub instrument: String,
    pub n_rows: u32,
    pub n_traded: u32,
    pub n_stood_aside: u32,
    pub mean_signed_traded_return: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectedInstrumentVisibility {
    pub development: Vec<InstrumentScore>,
    pub selection: Vec<InstrumentScore>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchArchive {
    pub contract_id: String,
    pub generations: Vec<GenerationPopulationRecord>,
    pub offspring: Vec<OffspringEdge>,
    pub selected_instruments: Option<SelectedInstrumentVisibility>,
}

pub fn archive_satisfies_contract(archive: &SearchArchive) -> bool {
    !archive.generations.is_empty()
        && archive.generations.iter().all(|g| {
            g.unique_genome_count > 0
                && g.population_size > 0
                && !g.generation_best.identity.is_empty()
        })
}

pub struct RecordingObserver {
    inner: Mutex<SearchArchive>,
}

impl RecordingObserver {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(SearchArchive {
                contract_id: OBSERVABILITY_CONTRACT_ID.to_string(),
                generations: Vec::new(),
                offspring: Vec::new(),
                selected_instruments: None,
            }),
        }
    }

    pub fn into_archive(self) -> SearchArchive {
        self.inner.into_inner().expect("observability lock")
    }

    pub fn snapshot(&self) -> SearchArchive {
        self.inner.lock().expect("observability lock").clone()
    }
}

impl GenerationObserver<RuleListGenome, GenomeEvaluation> for RecordingObserver {
    fn on_evaluated_generation(&self, generation: usize, evaluations: &[GenomeEvaluation]) {
        if evaluations.is_empty() {
            return;
        }
        let fitnesses: Vec<f64> = evaluations.iter().map(|e| e.fitness()).collect();
        let best_fitness = fitnesses[0];
        let worst_fitness = *fitnesses.last().unwrap();
        let mean_fitness = fitnesses.iter().sum::<f64>() / fitnesses.len() as f64;
        let mut ordered = fitnesses.clone();
        ordered.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let median_fitness = if ordered.len() % 2 == 1 {
            ordered[ordered.len() / 2]
        } else {
            let hi = ordered.len() / 2;
            (ordered[hi - 1] + ordered[hi]) / 2.0
        };
        let mut identities = HashSet::new();
        let mut action_symbols = ActionSymbolHistogram::default();
        let mut factor_consumption = FactorConsumptionHistogram::default();
        let mut near_best = Vec::new();
        for e in evaluations {
            let genome = e.genome();
            let id = genome.identity_hash();
            identities.insert(id.clone());
            let actions: HashSet<DecisionAction> = genome
                .rules
                .iter()
                .map(|r| r.action)
                .chain(std::iter::once(genome.unmatched_action))
                .collect();
            if actions.contains(&DecisionAction::Long) {
                action_symbols.genomes_with_long += 1;
            }
            if actions.contains(&DecisionAction::Short) {
                action_symbols.genomes_with_short += 1;
            }
            if actions.contains(&DecisionAction::NoTrade) {
                action_symbols.genomes_with_no_trade += 1;
            }
            match genome.unmatched_action {
                DecisionAction::Long => action_symbols.unmatched_long += 1,
                DecisionAction::Short => action_symbols.unmatched_short += 1,
                DecisionAction::NoTrade => action_symbols.unmatched_no_trade += 1,
            }
            if genome.rules.iter().any(|r| r.when.iter().any(|p| p.concept == "Trend")) {
                factor_consumption.genomes_using_trend += 1;
            }
            if genome
                .rules
                .iter()
                .any(|r| r.when.iter().any(|p| p.concept == "Momentum"))
            {
                factor_consumption.genomes_using_momentum += 1;
            }
            if genome
                .rules
                .iter()
                .any(|r| r.when.iter().any(|p| p.concept == "Volatility"))
            {
                factor_consumption.genomes_using_volatility += 1;
            }
            if (best_fitness - e.fitness()).abs() <= NEAR_BEST_EPS {
                near_best.push(serialize_genome(genome, e.fitness()));
            }
        }
        let best = evaluations[0].genome();
        let record = GenerationPopulationRecord {
            generation,
            population_size: evaluations.len(),
            unique_genome_count: identities.len(),
            best_fitness,
            median_fitness,
            mean_fitness,
            worst_fitness,
            action_symbols,
            factor_consumption,
            generation_best: serialize_genome(best, best_fitness),
            near_best,
            living_slots: evaluations
                .iter()
                .map(|e| serialize_genome(e.genome(), e.fitness()))
                .collect(),
        };
        self.inner.lock().expect("observability lock").generations.push(record);
        let _ = fitnesses;
    }

    fn on_offspring(
        &self,
        generation: usize,
        parent_a: &RuleListGenome,
        parent_b: &RuleListGenome,
        child: &RuleListGenome,
    ) {
        self.inner.lock().expect("observability lock").offspring.push(OffspringEdge {
            generation,
            parent_a_identity: parent_a.identity_hash(),
            parent_b_identity: parent_b.identity_hash(),
            child_identity: child.identity_hash(),
        });
    }
}

fn serialize_genome(genome: &RuleListGenome, fitness: f64) -> SerializedGenome {
    SerializedGenome {
        identity: genome.identity_hash(),
        development_fitness: fitness,
        rules: genome.rules.clone(),
        unmatched_action: genome.unmatched_action,
    }
}

pub fn per_instrument_scores(
    genome: &RuleListGenome,
    slice: &ObservationSlice,
) -> Result<Vec<InstrumentScore>, String> {
    if slice.kind == PartitionKind::Evaluation {
        return Err("observability must not score the evaluation slice".into());
    }
    let _ = score_genome(genome, slice)?;
    let mut out = Vec::new();
    for ticker in RESEARCH_UNIVERSE {
        let mut n_traded = 0u32;
        let mut n_stood_aside = 0u32;
        let mut n_rows = 0u32;
        let mut traded = Vec::new();
        for row in slice.rows.iter().filter(|r| r.instrument == *ticker) {
            n_rows += 1;
            let action = first_match_action(&genome.rules, genome.unmatched_action, &row.profile);
            match action {
                DecisionAction::NoTrade => n_stood_aside += 1,
                DecisionAction::Long | DecisionAction::Short => {
                    if let Some(raw) = row.instrument_return {
                        n_traded += 1;
                        traded.push(if action == DecisionAction::Long {
                            raw
                        } else {
                            -raw
                        });
                    }
                }
            }
        }
        out.push(InstrumentScore {
            instrument: (*ticker).to_string(),
            n_rows,
            n_traded,
            n_stood_aside,
            mean_signed_traded_return: if traded.is_empty() {
                0.0
            } else {
                traded.iter().sum::<f64>() / traded.len() as f64
            },
        });
    }
    Ok(out)
}

pub fn selected_instrument_visibility(
    selected: &SelectedCandidate,
    development: &ObservationSlice,
    selection: &ObservationSlice,
) -> Result<SelectedInstrumentVisibility, String> {
    Ok(SelectedInstrumentVisibility {
        development: per_instrument_scores(&selected.genome, development)?,
        selection: per_instrument_scores(&selected.genome, selection)?,
    })
}

pub fn attach_selected_visibility(
    mut archive: SearchArchive,
    visibility: SelectedInstrumentVisibility,
) -> SearchArchive {
    archive.selected_instruments = Some(visibility);
    archive
}
