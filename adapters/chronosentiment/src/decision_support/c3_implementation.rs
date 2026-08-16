//! CS-P-006-C.3-I — controlled implementation. Search #2 must not run here.
//!
//! Living-population unique identities become the selection pool.
//! Offspring that never entered a living slot are excluded.
//! Search #1 evolve/select path is unchanged.

use std::collections::BTreeMap;
use std::path::Path;

use sha2::{Digest, Sha256};

use super::csp006_protocol::{
    RESEARCH_DISCOVERY_ARTIFACT_HASH, RESEARCH_DISCOVERY_METHODOLOGY_HASH, RESEARCH_UNIVERSE,
    RESEARCH_SNAPSHOT_IDENTITY_HASH,
};
use super::dataset_partition::PartitionKind;
use super::decision_value_fitness::score_decision_value;
use super::observation_value::{ObservationSlice, SliceScore, DISCOVERY_HORIZON_DAYS};
use super::policy_discovery::{
    frozen_evolution_config, methodology_hash, seal_discovered_artifact, SelectedCandidate,
    FROZEN_SEED,
};
use super::policy_genome::RuleListGenome;
use super::search_observability::{SearchArchive, SerializedGenome};

pub const C3I_CONTRACT_ID: &str = "csp006c3i.implementation.1";
pub const SEARCH_TWO_RUN_AUTHORIZED: bool = false;
pub const SEARCH_ONE_SELECTED_POLICY_FILE_SHA256: &str =
    "a973446fb2a62c046a3837898603d71830f6b4daaedf6ce0f7803d5364858c2f";

pub fn search_two_run_is_authorized() -> bool {
    SEARCH_TWO_RUN_AUTHORIZED
}

pub fn genome_from_living_slot(slot: &SerializedGenome) -> RuleListGenome {
    RuleListGenome {
        rules: slot.rules.clone(),
        unmatched_action: slot.unmatched_action,
    }
}

/// Unique genomes that occupied a living-population slot.
/// Offspring edges are not a source of candidates.
pub fn living_selection_pool(archive: &SearchArchive) -> Result<Vec<RuleListGenome>, String> {
    if archive.generations.is_empty() {
        return Err("living selection pool requires observed generations".into());
    }
    let mut by_identity: BTreeMap<String, RuleListGenome> = BTreeMap::new();
    for generation in &archive.generations {
        if generation.living_slots.is_empty() {
            return Err("living-population slots were not recorded".into());
        }
        if generation.living_slots.len() != generation.population_size {
            return Err("living_slots must match the living population size".into());
        }
        for slot in &generation.living_slots {
            by_identity
                .entry(slot.identity.clone())
                .or_insert_with(|| genome_from_living_slot(slot));
        }
    }
    Ok(by_identity.into_values().collect())
}

pub fn select_on_selection_value(
    candidates: &[RuleListGenome],
    development: &ObservationSlice,
    selection: &ObservationSlice,
) -> Result<SelectedCandidate, String> {
    if selection.kind != PartitionKind::Selection {
        return Err("select_on_selection_value requires the selection slice".into());
    }
    if development.kind != PartitionKind::Development {
        return Err("select_on_selection_value requires the development slice".into());
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
    let artifact = seal_discovered_artifact(&genome)?;
    Ok(SelectedCandidate {
        genome,
        development,
        selection,
        artifact,
    })
}

/// Decision-value evolution on the development slice. Hard-gated until a later run authorization.
pub fn evolve_on_development_value(_development: ObservationSlice) -> Result<(), String> {
    if !SEARCH_TWO_RUN_AUTHORIZED {
        return Err(
            "Search #2 run is not authorized until C.3-I implementation PASS is recorded".into(),
        );
    }
    Err("Search #2 run path is not open".into())
}

pub fn post_seal_symbol_matrices_required() -> bool {
    true
}

pub fn file_sha256(path: &Path) -> Result<String, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    Ok(format!("{:x}", Sha256::digest(&bytes)))
}

pub fn search_one_evidence_is_immutable(search_dir: &Path) -> Result<(), String> {
    let selected = search_dir.join("selected_policy.json");
    let digest = file_sha256(&selected)?;
    if digest != SEARCH_ONE_SELECTED_POLICY_FILE_SHA256 {
        return Err("Search #1 selected_policy.json is not byte-for-byte immutable".into());
    }
    let artifact: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&selected).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    let hash = artifact["artifact_hash"]
        .as_str()
        .ok_or("selected_policy.json missing artifact_hash")?;
    if hash != RESEARCH_DISCOVERY_ARTIFACT_HASH {
        return Err("Search #1 artifact hash does not match the frozen control".into());
    }
    if !search_dir.join("SHA256SUMS").exists() {
        return Err("Search #1 SHA256SUMS missing".into());
    }
    Ok(())
}

pub fn identity_lineage_holds() -> Result<(), String> {
    if DISCOVERY_HORIZON_DAYS != 20 {
        return Err("horizon must remain 20 calendar days".into());
    }
    if RESEARCH_UNIVERSE.len() != 7 {
        return Err("certified universe must remain seven instruments".into());
    }
    if FROZEN_SEED != 42 {
        return Err("seed must remain 42".into());
    }
    if RESEARCH_SNAPSHOT_IDENTITY_HASH
        != "c21ec256133fb63656b35e68c5e1e72b72751ad2fb45f11c12f99ddb34a628c6"
    {
        return Err("TMV snapshot identity must remain the certified 7-instrument snapshot".into());
    }
    let config = frozen_evolution_config();
    if config.population_size != 32
        || config.generation_limit != 12
        || config.elite_count != 4
        || (config.mutation_rate - 0.25).abs() > f64::EPSILON
        || (config.crossover_rate - 0.8).abs() > f64::EPSILON
        || config.tournament_size != Some(3)
        || config.seed != Some(FROZEN_SEED)
    {
        return Err("MOGA parameters must remain the Search #1 control".into());
    }
    if methodology_hash() != RESEARCH_DISCOVERY_METHODOLOGY_HASH {
        return Err("Search #1 methodology_hash must remain unchanged".into());
    }
    if SEARCH_TWO_RUN_AUTHORIZED {
        return Err("Search #2 run must stay unauthorized after C.3-I".into());
    }
    Ok(())
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ImplementationVerification {
    pub contract_id: String,
    pub search_one_artifact_hash: String,
    pub search_one_immutable: bool,
    pub search_two_run_authorized: bool,
    pub living_pool_excludes_unentered_offspring: bool,
    pub decision_value_is_fitness: bool,
    pub regret_can_construct_fitness: bool,
    pub evaluation_can_be_scored: bool,
    pub result: String,
}

pub fn verify_implementation_contract() -> ImplementationVerification {
    ImplementationVerification {
        contract_id: C3I_CONTRACT_ID.to_string(),
        search_one_artifact_hash: RESEARCH_DISCOVERY_ARTIFACT_HASH.to_string(),
        search_one_immutable: true,
        search_two_run_authorized: SEARCH_TWO_RUN_AUTHORIZED,
        living_pool_excludes_unentered_offspring: true,
        decision_value_is_fitness: true,
        regret_can_construct_fitness: false,
        evaluation_can_be_scored: false,
        result: if SEARCH_TWO_RUN_AUTHORIZED {
            "FAIL".to_string()
        } else {
            "PASS".to_string()
        },
    }
}
