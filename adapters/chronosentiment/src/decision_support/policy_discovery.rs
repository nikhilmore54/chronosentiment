//! Coralys policy discovery on certified TMV state.
//!
//! Evolution uses the development slice only. Candidate selection uses the
//! selection slice only. The evaluation slice is never scored here.

use coralys_moga::config::EvolutionConfig;
use coralys_moga::engine::EvolutionEngineBuilder;
use coralys_moga::traits::Evaluated;
use serde::Serialize;
use sha2::{Digest, Sha256};

use super::csp006_protocol::{
    coralys_search_is_authorized, CHRONOLOGICAL_PARTITION_HASH, MAX_RULES_FIRST_DISCOVERY,
    RESEARCH_SNAPSHOT_IDENTITY_HASH, RESEARCH_UNIVERSE,
};
use super::dataset_partition::{certified_research_partition, PartitionKind};
use super::observation_value::{
    score_genome, DevelopmentFitness, ObservationSlice, SliceScore, DISCOVERY_HORIZON_DAYS,
};
use super::policy_artifact::{
    certified_factor_definitions, certified_input_schema, PolicyArtifact, TrainingProvenance,
    CONTRACT_FIXTURE_ENGINE, POLICY_ARTIFACT_SCHEMA_VERSION,
};
use super::policy_genome::{RuleListCrossover, RuleListFactory, RuleListGenome, RuleListMutation};
use super::search_observability::{
    attach_selected_visibility, selected_instrument_visibility, RecordingObserver, SearchArchive,
};
use super::DecisionAction;

pub const DISCOVERY_ENGINE: &str = "coralys.moga.rulelist.v0";
pub const DISCOVERY_POLICY_ID: &str = "coralys.rulelist.discovered";
pub const DISCOVERY_POLICY_VERSION: &str = "v0";
pub const FROZEN_SEED: u64 = 42;

pub fn frozen_evolution_config() -> EvolutionConfig {
    EvolutionConfig {
        population_size: 32,
        mutation_rate: 0.25,
        crossover_rate: 0.8,
        elite_count: 4,
        generation_limit: 12,
        seed: Some(FROZEN_SEED),
        tournament_size: Some(3),
        termination_policy: None,
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct GenerationLineageEntry {
    pub generation: usize,
    pub development_fitness: f64,
    pub genome_identity: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SearchEvidence {
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
    pub development_best_fitness: f64,
    pub generation_best_fitness: Vec<f64>,
    pub average_fitness_history: Vec<f64>,
    pub generation_lineage: Vec<GenerationLineageEntry>,
    pub n_candidates_considered_for_selection: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct SelectionRecord {
    pub artifact_hash: String,
    pub genome_identity: String,
    pub n_rules: usize,
    pub unmatched_action: DecisionAction,
    pub development: SliceScore,
    pub selection: SliceScore,
}

#[derive(Debug, Clone, Serialize)]
pub struct SelectedCandidate {
    pub genome: RuleListGenome,
    pub development: SliceScore,
    pub selection: SliceScore,
    pub artifact: PolicyArtifact,
}

pub fn methodology_hash() -> String {
    let payload = serde_json::json!({
        "engine": DISCOVERY_ENGINE,
        "horizon_days": DISCOVERY_HORIZON_DAYS,
        "aggregation": "mean_of_per_instrument_mean_signed_traded_returns",
        "no_trade": "standing_aside_excluded_from_traded_mean",
        "untraded_instrument": 0.0,
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

pub fn discovery_run_id() -> String {
    format!("coralys.rulelist.{}", &methodology_hash()[..16])
}

pub fn evolve_on_development(
    development: ObservationSlice,
) -> Result<(SearchEvidence, Vec<RuleListGenome>), String> {
    if !coralys_search_is_authorized() {
        return Err("Coralys search is not authorized".into());
    }
    if development.kind != PartitionKind::Development {
        return Err("evolve_on_development requires the development slice".into());
    }
    let config = frozen_evolution_config();
    let evaluator = DevelopmentFitness::new(development)?;
    let engine = EvolutionEngineBuilder::new()
        .with_evaluator(evaluator)
        .with_mutator(RuleListMutation)
        .with_crossover(RuleListCrossover)
        .with_factory(RuleListFactory)
        .build()?;
    let result = engine.run_ga_evolution(config.clone())?;
    let mut candidates: Vec<RuleListGenome> = result
        .generation_history
        .iter()
        .map(|e| e.genome().clone())
        .collect();
    candidates.push(result.global_best.genome().clone());
    candidates.sort_by(|a, b| {
        serde_json::to_string(a)
            .unwrap()
            .cmp(&serde_json::to_string(b).unwrap())
    });
    candidates.dedup();
    let generation_lineage: Vec<GenerationLineageEntry> = result
        .generation_history
        .iter()
        .enumerate()
        .map(|(generation, e)| GenerationLineageEntry {
            generation,
            development_fitness: e.fitness(),
            genome_identity: e.genome().identity_hash(),
        })
        .collect();
    let evidence = SearchEvidence {
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
        methodology_hash: methodology_hash(),
        development_best_fitness: result.global_best.fitness(),
        generation_best_fitness: generation_lineage
            .iter()
            .map(|e| e.development_fitness)
            .collect(),
        average_fitness_history: result.average_fitness_history,
        generation_lineage,
        n_candidates_considered_for_selection: candidates.len(),
    };
    Ok((evidence, candidates))
}

/// Same search as `evolve_on_development`, with a read-only generation observer.
/// Must not change PolicyArtifact identity versus the unobserved path.
pub fn evolve_on_development_observed(
    development: ObservationSlice,
) -> Result<(SearchEvidence, Vec<RuleListGenome>, SearchArchive), String> {
    if !coralys_search_is_authorized() {
        return Err("Coralys search is not authorized".into());
    }
    if development.kind != PartitionKind::Development {
        return Err("evolve_on_development_observed requires the development slice".into());
    }
    let config = frozen_evolution_config();
    let evaluator = DevelopmentFitness::new(development)?;
    let recorder = std::sync::Arc::new(RecordingObserver::new());
    let engine = EvolutionEngineBuilder::new()
        .with_evaluator(evaluator)
        .with_mutator(RuleListMutation)
        .with_crossover(RuleListCrossover)
        .with_factory(RuleListFactory)
        .with_generation_observer(recorder.clone())
        .build()?;
    let result = engine.run_ga_evolution(config.clone())?;
    let mut candidates: Vec<RuleListGenome> = result
        .generation_history
        .iter()
        .map(|e| e.genome().clone())
        .collect();
    candidates.push(result.global_best.genome().clone());
    candidates.sort_by(|a, b| {
        serde_json::to_string(a)
            .unwrap()
            .cmp(&serde_json::to_string(b).unwrap())
    });
    candidates.dedup();
    let generation_lineage: Vec<GenerationLineageEntry> = result
        .generation_history
        .iter()
        .enumerate()
        .map(|(generation, e)| GenerationLineageEntry {
            generation,
            development_fitness: e.fitness(),
            genome_identity: e.genome().identity_hash(),
        })
        .collect();
    let evidence = SearchEvidence {
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
        methodology_hash: methodology_hash(),
        development_best_fitness: result.global_best.fitness(),
        generation_best_fitness: generation_lineage
            .iter()
            .map(|e| e.development_fitness)
            .collect(),
        average_fitness_history: result.average_fitness_history,
        generation_lineage,
        n_candidates_considered_for_selection: candidates.len(),
    };
    Ok((evidence, candidates, recorder.snapshot()))
}

pub fn select_and_observe(
    candidates: &[RuleListGenome],
    development: &ObservationSlice,
    selection: &ObservationSlice,
    archive: SearchArchive,
) -> Result<(SelectedCandidate, SearchArchive), String> {
    let selected = select_on_selection(candidates, development, selection)?;
    let visibility = selected_instrument_visibility(&selected, development, selection)?;
    Ok((selected, attach_selected_visibility(archive, visibility)))
}

pub fn select_on_selection(
    candidates: &[RuleListGenome],
    development: &ObservationSlice,
    selection: &ObservationSlice,
) -> Result<SelectedCandidate, String> {
    if selection.kind != PartitionKind::Selection {
        return Err("select_on_selection requires the selection slice".into());
    }
    if candidates.is_empty() {
        return Err("no candidates to select".into());
    }
    let mut best: Option<(RuleListGenome, SliceScore, SliceScore)> = None;
    for genome in candidates {
        let sel = score_genome(genome, selection)?;
        let better = match &best {
            None => true,
            Some((_, _, prev)) => {
                sel.fitness > prev.fitness
                    || (sel.fitness == prev.fitness
                        && serde_json::to_string(genome).unwrap()
                            < serde_json::to_string(&best.as_ref().unwrap().0).unwrap())
            }
        };
        if better {
            let dev = score_genome(genome, development)?;
            best = Some((genome.clone(), dev, sel));
        }
    }
    let (genome, development_score, selection_score) = best.unwrap();
    let artifact = seal_discovered_artifact(&genome)?;
    Ok(SelectedCandidate {
        genome,
        development: development_score,
        selection: selection_score,
        artifact,
    })
}

pub fn selection_record(selected: &SelectedCandidate) -> SelectionRecord {
    SelectionRecord {
        artifact_hash: selected.artifact.artifact_hash.clone(),
        genome_identity: selected.genome.identity_hash(),
        n_rules: selected.genome.rules.len(),
        unmatched_action: selected.genome.unmatched_action,
        development: selected.development.clone(),
        selection: selected.selection.clone(),
    }
}

pub fn seal_discovered_artifact(genome: &RuleListGenome) -> Result<PolicyArtifact, String> {
    let partition = certified_research_partition();
    let artifact = PolicyArtifact {
        schema_version: POLICY_ARTIFACT_SCHEMA_VERSION.to_string(),
        policy_id: DISCOVERY_POLICY_ID.to_string(),
        policy_version: DISCOVERY_POLICY_VERSION.to_string(),
        discovery_engine: DISCOVERY_ENGINE.to_string(),
        discovery_run_id: discovery_run_id(),
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
        methodology_hash: methodology_hash(),
    };
    if artifact.discovery_engine == CONTRACT_FIXTURE_ENGINE {
        return Err("discovered artifact must not use the contract fixture engine".into());
    }
    artifact.seal().map_err(|e| e.to_string())
}

pub fn render_search_evidence(evidence: &SearchEvidence) -> String {
    let mut md = String::from("# Policy discovery — search evidence\n\n");
    md.push_str("Coralys evolution on the **development** slice only. Evaluation outcomes were not used.\n\n");
    md.push_str(&format!("- engine: `{}`\n", evidence.discovery_engine));
    md.push_str(&format!("- seed: {}\n", evidence.seed));
    md.push_str(&format!("- population: {}\n", evidence.population_size));
    md.push_str(&format!("- generations: {}\n", evidence.generation_limit));
    md.push_str(&format!("- horizon_days: {}\n", evidence.horizon_days));
    md.push_str(&format!(
        "- development best fitness: {:.6}\n",
        evidence.development_best_fitness
    ));
    md.push_str(&format!(
        "- candidates for selection: {}\n",
        evidence.n_candidates_considered_for_selection
    ));
    md.push_str(&format!(
        "- methodology hash: `{}`\n",
        evidence.methodology_hash
    ));
    md.push_str(&format!(
        "- generation best: {:?}\n",
        evidence.generation_best_fitness
    ));
    md.push_str("\n## Generation lineage (development fitness only)\n\n");
    md.push_str("| generation | development_fitness | genome_identity |\n");
    md.push_str("|------------|---------------------|-----------------|\n");
    for row in &evidence.generation_lineage {
        md.push_str(&format!(
            "| {} | {:.6} | `{}` |\n",
            row.generation, row.development_fitness, row.genome_identity
        ));
    }
    md.push_str("\nEvaluation outcomes were not scored. Coralys receives no holdout feedback.\n");
    md
}

pub fn render_selected_artifact(selected: &SelectedCandidate) -> String {
    let mut md = String::from("# Selected PolicyArtifact\n\n");
    md.push_str("Sealed after **selection** on the selection slice. Immutable. Not retuned against evaluation.\n\n");
    md.push_str(&format!(
        "- policy: `{}@{}`\n",
        selected.artifact.policy_id, selected.artifact.policy_version
    ));
    md.push_str(&format!(
        "- discovery_engine: `{}`\n",
        selected.artifact.discovery_engine
    ));
    md.push_str(&format!(
        "- discovery_run_id: `{}`\n",
        selected.artifact.discovery_run_id
    ));
    md.push_str(&format!(
        "- artifact_hash: `{}`\n",
        selected.artifact.artifact_hash
    ));
    md.push_str(&format!(
        "- methodology_hash: `{}`\n",
        selected.artifact.methodology_hash
    ));
    md.push_str(&format!(
        "- genome identity: `{}`\n",
        selected.genome.identity_hash()
    ));
    md.push_str(&format!("- n_rules: {}\n", selected.genome.rules.len()));
    md.push_str(&format!(
        "- unmatched_action: {:?}\n",
        selected.genome.unmatched_action
    ));
    md.push_str(&format!(
        "- allowed_information_timestamp: {}\n",
        selected.artifact.allowed_information_timestamp
    ));
    md.push_str(&format!(
        "- development mean signed traded return: {:.6} (traded {}, stood aside {})\n",
        selected.development.fitness,
        selected.development.n_traded,
        selected.development.n_stood_aside
    ));
    md.push_str(&format!(
        "- selection mean signed traded return: {:.6} (traded {}, stood aside {})\n",
        selected.selection.fitness, selected.selection.n_traded, selected.selection.n_stood_aside
    ));
    md.push_str("\nRules are the search result, not a hand-written mapping.\n\n```json\n");
    md.push_str(&serde_json::to_string_pretty(&selected.artifact.rules).unwrap());
    md.push_str("\n```\n");
    md
}
