//! CS-P-006-C.3-R — one authorized Search #2 run.
//!
//! C.3-I remains the identity gate and does not evolve.
//! Evaluation is not an argument to evolution or selection.
//! Search #1 methodology and evidence stay unchanged.

use sha2::{Digest, Sha256};

use coralys_moga::engine::EvolutionEngineBuilder;
use coralys_moga::traits::Evaluated;

use super::c3_implementation::living_selection_pool;
use super::csp006_protocol::{
    coralys_search_is_authorized, CHRONOLOGICAL_PARTITION_HASH, MAX_RULES_FIRST_DISCOVERY,
    RESEARCH_DISCOVERY_ARTIFACT_HASH, RESEARCH_DISCOVERY_DIR, RESEARCH_DISCOVERY_METHODOLOGY_HASH,
    RESEARCH_SNAPSHOT_IDENTITY_HASH, RESEARCH_UNIVERSE,
};
use super::dataset_partition::{certified_research_partition, PartitionKind};
use super::decision_value_fitness::{score_decision_value, DevelopmentValue};
use super::observation_value::{ObservationSlice, SliceScore, DISCOVERY_HORIZON_DAYS};
use super::policy_artifact::{
    certified_factor_definitions, certified_input_schema, PolicyArtifact, TrainingProvenance,
    CONTRACT_FIXTURE_ENGINE, POLICY_ARTIFACT_SCHEMA_VERSION,
};
use super::policy_discovery::{
    frozen_evolution_config, methodology_hash, SelectedCandidate, DISCOVERY_ENGINE,
    DISCOVERY_POLICY_ID, DISCOVERY_POLICY_VERSION, FROZEN_SEED,
};
use super::policy_genome::{RuleListCrossover, RuleListFactory, RuleListGenome, RuleListMutation};
use super::search_observability::{RecordingObserver, SearchArchive};
use super::DecisionAction;

use std::path::Path;

pub const C3_RUN_CONTRACT_ID: &str = "csp006c3r.search_two.1";
pub const C3_RUN_AUTHORIZED: bool = true;

pub fn c3_run_is_authorized() -> bool {
    C3_RUN_AUTHORIZED
}

pub fn decision_value_methodology_hash() -> String {
    let payload = serde_json::json!({
        "engine": DISCOVERY_ENGINE,
        "horizon_days": DISCOVERY_HORIZON_DAYS,
        "aggregation": "mean_of_per_instrument_mean_decision_value",
        "no_trade": "enters_instrument_mean_as_zero",
        "empty_instrument": "protocol_error",
        "selection_pool": "unique_living_population_slot_identities",
        "max_rules": MAX_RULES_FIRST_DISCOVERY,
        "seed": FROZEN_SEED,
        "snapshot": RESEARCH_SNAPSHOT_IDENTITY_HASH,
        "partition": CHRONOLOGICAL_PARTITION_HASH,
    });
    format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(&payload).unwrap())
    )
}

pub fn decision_value_run_id() -> String {
    format!(
        "coralys.rulelist.dv.{}",
        &decision_value_methodology_hash()[..16]
    )
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct DecisionValueSearchEvidence {
    pub contract_id: String,
    pub discovery_engine: String,
    pub seed: u64,
    pub population_size: usize,
    pub generation_limit: usize,
    pub elite_count: usize,
    pub mutation_rate: f64,
    pub crossover_rate: f64,
    pub tournament_size: usize,
    pub horizon_days: u32,
    pub n_instruments: usize,
    pub snapshot_identity_hash: String,
    pub partition_hash: String,
    pub methodology_hash: String,
    pub development_best_value: f64,
    pub generation_best_value: Vec<f64>,
    pub average_value_history: Vec<f64>,
    pub n_living_candidates: usize,
}

pub fn seal_decision_value_artifact(genome: &RuleListGenome) -> Result<PolicyArtifact, String> {
    let partition = certified_research_partition();
    let artifact = PolicyArtifact {
        schema_version: POLICY_ARTIFACT_SCHEMA_VERSION.to_string(),
        policy_id: DISCOVERY_POLICY_ID.to_string(),
        policy_version: DISCOVERY_POLICY_VERSION.to_string(),
        discovery_engine: DISCOVERY_ENGINE.to_string(),
        discovery_run_id: decision_value_run_id(),
        input_schema: certified_input_schema(),
        factor_definitions: certified_factor_definitions(),
        action_space: vec![
            DecisionAction::Long,
            DecisionAction::Short,
            DecisionAction::NoTrade,
        ],
        rules: genome.rules.clone(),
        unmatched_action: genome.unmatched_action,
        training_provenance: TrainingProvenance::from_chronological_partition(&partition),
        allowed_information_timestamp: *partition.development.timestamps.last().unwrap(),
        artifact_hash: String::new(),
        methodology_hash: decision_value_methodology_hash(),
    };
    if artifact.discovery_engine == CONTRACT_FIXTURE_ENGINE {
        return Err("discovered artifact must not use the contract fixture engine".into());
    }
    artifact.seal().map_err(|e| e.to_string())
}

pub fn select_living_on_selection_value(
    candidates: &[RuleListGenome],
    development: &ObservationSlice,
    selection: &ObservationSlice,
) -> Result<SelectedCandidate, String> {
    if selection.kind != PartitionKind::Selection {
        return Err("select_living_on_selection_value requires the selection slice".into());
    }
    if development.kind != PartitionKind::Development {
        return Err("select_living_on_selection_value requires the development slice".into());
    }
    if candidates.is_empty() {
        return Err("no living-population candidates to select".into());
    }
    let mut best: Option<(RuleListGenome, SliceScore, SliceScore)> = None;
    for genome in candidates {
        let selection_value = score_decision_value(genome, selection)?;
        let better = match &best {
            None => true,
            Some((_, _, prev)) => {
                selection_value.fitness > prev.fitness
                    || (selection_value.fitness == prev.fitness
                        && serde_json::to_string(genome).unwrap()
                            < serde_json::to_string(&best.as_ref().unwrap().0).unwrap())
            }
        };
        if better {
            let development_value = score_decision_value(genome, development)?;
            best = Some((genome.clone(), development_value, selection_value));
        }
    }
    let (genome, development, selection) = best.unwrap();
    let artifact = seal_decision_value_artifact(&genome)?;
    if artifact.artifact_hash == RESEARCH_DISCOVERY_ARTIFACT_HASH {
        return Err("Search #2 must not reuse the Search #1 artifact identity".into());
    }
    if artifact.methodology_hash == RESEARCH_DISCOVERY_METHODOLOGY_HASH {
        return Err("Search #2 must not reuse the Search #1 methodology hash".into());
    }
    Ok(SelectedCandidate {
        genome,
        development,
        selection,
        artifact,
    })
}

/// Full frozen configuration. Observer on. Evaluation is not in the signature.
pub fn evolve_decision_value_on_development(
    development: ObservationSlice,
) -> Result<(DecisionValueSearchEvidence, SearchArchive), String> {
    if !C3_RUN_AUTHORIZED {
        return Err("C.3 Search #2 run is not authorized".into());
    }
    if !coralys_search_is_authorized() {
        return Err("Coralys search is not authorized".into());
    }
    if development.kind != PartitionKind::Development {
        return Err("decision-value evolve requires the development slice".into());
    }
    if methodology_hash() != RESEARCH_DISCOVERY_METHODOLOGY_HASH {
        return Err("Search #1 methodology_hash must remain unchanged".into());
    }
    let config = frozen_evolution_config();
    if config.generation_limit != 12 || config.population_size != 32 || config.seed != Some(42) {
        return Err("frozen MOGA configuration must be used in full".into());
    }
    let evaluator = DevelopmentValue::new(development)?;
    let recorder = std::sync::Arc::new(RecordingObserver::new());
    let engine = EvolutionEngineBuilder::new()
        .with_evaluator(evaluator)
        .with_mutator(RuleListMutation)
        .with_crossover(RuleListCrossover)
        .with_factory(RuleListFactory)
        .with_generation_observer(recorder.clone())
        .build()?;
    let result = engine.run_ga_evolution(config.clone())?;
    if result.generation_history.len() != config.generation_limit {
        return Err("search stopped before the frozen generation limit".into());
    }
    let archive = recorder.snapshot();
    let candidates = living_selection_pool(&archive)?;
    let evidence = DecisionValueSearchEvidence {
        contract_id: C3_RUN_CONTRACT_ID.to_string(),
        discovery_engine: DISCOVERY_ENGINE.to_string(),
        seed: FROZEN_SEED,
        population_size: config.population_size,
        generation_limit: config.generation_limit,
        elite_count: config.elite_count,
        mutation_rate: config.mutation_rate,
        crossover_rate: config.crossover_rate,
        tournament_size: config.tournament_size.unwrap_or(3),
        horizon_days: DISCOVERY_HORIZON_DAYS,
        n_instruments: RESEARCH_UNIVERSE.len(),
        snapshot_identity_hash: RESEARCH_SNAPSHOT_IDENTITY_HASH.to_string(),
        partition_hash: CHRONOLOGICAL_PARTITION_HASH.to_string(),
        methodology_hash: decision_value_methodology_hash(),
        development_best_value: result.global_best.fitness(),
        generation_best_value: result
            .generation_history
            .iter()
            .map(|e| e.fitness())
            .collect(),
        average_value_history: result.average_fitness_history,
        n_living_candidates: candidates.len(),
    };
    Ok((evidence, archive))
}

pub fn refuse_search_one_output(output: &Path) -> Result<(), String> {
    let search_one = Path::new(RESEARCH_DISCOVERY_DIR);
    if output == search_one {
        return Err("refusing to write Search #2 into the Search #1 evidence directory".into());
    }
    if output.ends_with("selected_policy.json") {
        return Err("refusing to overwrite a selected_policy.json path directly".into());
    }
    if output.file_name().and_then(|n| n.to_str()) == Some("20260814T195327Z") {
        return Err("refusing to write into the Search #1 stamp directory".into());
    }
    Ok(())
}

pub fn render_decision_value_search(evidence: &DecisionValueSearchEvidence) -> String {
    format!(
        "# CS-P-006-C.3-R Search #2 evidence\n\n\
         Coralys evolved on the development slice with M.1 protocol V. \
         Evaluation was not loaded during search.\n\n\
         - contract: `{}`\n\
         - engine: `{}`\n\
         - seed: {}\n\
         - population: {}\n\
         - generations: {} (complete)\n\
         - horizon_days: {}\n\
         - n_instruments: {}\n\
         - living candidates: {}\n\
         - development best value: {:.6}\n\
         - methodology hash: `{}`\n\
         - generation best value: {:?}\n\n\
         Search #1 remains `{}`.\n",
        evidence.contract_id,
        evidence.discovery_engine,
        evidence.seed,
        evidence.population_size,
        evidence.generation_limit,
        evidence.horizon_days,
        evidence.n_instruments,
        evidence.n_living_candidates,
        evidence.development_best_value,
        evidence.methodology_hash,
        evidence.generation_best_value,
        RESEARCH_DISCOVERY_ARTIFACT_HASH,
    )
}
