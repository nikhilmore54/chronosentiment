//! CS-P-006-C — first Coralys TMV discovery run.
//!
//! Coralys evolves on development and selects on selection.
//! ChronoSentiment scores the sealed artifact on evaluation afterwards.
//! Evaluation outcomes are never returned to search.
//!
//! Not B5. Does not write chrono_b3_test / chrono_b4_test.

use std::env;
use std::fs;
use std::path::PathBuf;

use chrono::Utc;
use chronosentiment_adapter::decision_support::csp006_protocol::{
    coralys_search_is_authorized, RESEARCH_SNAPSHOT_DIR, RESEARCH_SNAPSHOT_IDENTITY_HASH,
    RESEARCH_UNIVERSE,
};
use chronosentiment_adapter::decision_support::csp006_snapshot::load_required_yahoo_cache;
use chronosentiment_adapter::decision_support::dataset_partition::{
    certified_research_partition, PartitionKind,
};
use chronosentiment_adapter::decision_support::observation_value::build_observation_slice;
use chronosentiment_adapter::decision_support::policy_discovery::{
    evolve_on_development, render_search_evidence, render_selected_artifact, select_on_selection,
    selection_record, FROZEN_SEED,
};
use chronosentiment_adapter::decision_support::policy_handoff::{
    evaluate_sealed_candidate, render_handoff,
};
use serde::Serialize;

#[derive(Serialize)]
struct SearchOutput {
    evolution: chronosentiment_adapter::decision_support::policy_discovery::SearchEvidence,
    selection: chronosentiment_adapter::decision_support::policy_discovery::SelectionRecord,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (output, cache_dir) = parse_args()?;
    let db = env::var("DATABASE_URL").unwrap_or_default();
    if db.contains("chrono_b3_test") || db.contains("chrono_b4_test") {
        return Err("refusing certified database name in DATABASE_URL".into());
    }
    if !coralys_search_is_authorized() {
        return Err("Coralys search is not authorized".into());
    }

    fs::create_dir_all(&output)?;
    let cache = load_required_yahoo_cache(&cache_dir).map_err(|e| e.to_string())?;
    let partition = certified_research_partition();
    if partition.partition_hash
        != chronosentiment_adapter::decision_support::csp006_protocol::CHRONOLOGICAL_PARTITION_HASH
    {
        return Err("certified partition hash mismatch".into());
    }

    let development = build_observation_slice(
        &cache,
        &partition.development.timestamps,
        PartitionKind::Development,
    )
    .map_err(|e| e.to_string())?;
    let selection = build_observation_slice(
        &cache,
        &partition.selection.timestamps,
        PartitionKind::Selection,
    )
    .map_err(|e| e.to_string())?;
    let evaluation = build_observation_slice(
        &cache,
        &partition.evaluation.timestamps,
        PartitionKind::Evaluation,
    )
    .map_err(|e| e.to_string())?;

    let (evidence, candidates) =
        evolve_on_development(development.clone()).map_err(|e| e.to_string())?;
    let selected =
        select_on_selection(&candidates, &development, &selection).map_err(|e| e.to_string())?;

    let (evidence_repeat, candidates_repeat) =
        evolve_on_development(development.clone()).map_err(|e| e.to_string())?;
    let selected_repeat = select_on_selection(&candidates_repeat, &development, &selection)
        .map_err(|e| e.to_string())?;
    if selected.artifact.artifact_hash != selected_repeat.artifact.artifact_hash {
        return Err("same seed produced a different PolicyArtifact identity".into());
    }
    if evidence.development_best_fitness != evidence_repeat.development_best_fitness {
        return Err("same seed produced a different development best fitness".into());
    }

    let record = selection_record(&selected);
    let search_output = SearchOutput {
        evolution: evidence.clone(),
        selection: record,
    };

    // ChronoSentiment holdout — after the artifact is sealed. Not Coralys feedback.
    let handoff =
        evaluate_sealed_candidate(&selected.artifact, &evaluation).map_err(|e| e.to_string())?;

    fs::write(
        output.join("search_evidence.json"),
        serde_json::to_vec_pretty(&search_output)?,
    )?;
    fs::write(output.join("SEARCH.md"), render_search_evidence(&evidence))?;
    fs::write(
        output.join("selected_policy.json"),
        serde_json::to_vec_pretty(&selected.artifact)?,
    )?;
    fs::write(
        output.join("SELECTED.md"),
        render_selected_artifact(&selected),
    )?;
    fs::write(
        output.join("evaluation_handoff.json"),
        serde_json::to_vec_pretty(&handoff)?,
    )?;
    fs::write(output.join("EVALUATION.md"), render_handoff(&handoff))?;
    fs::write(
        output.join("PROVENANCE.md"),
        render_provenance(
            &output,
            &cache_dir,
            &selected.artifact.artifact_hash,
            &selected.artifact.methodology_hash,
        ),
    )?;

    println!("result=PASS");
    println!("artifact_hash={}", selected.artifact.artifact_hash);
    println!(
        "repeated_artifact_hash={}",
        selected_repeat.artifact.artifact_hash
    );
    println!("methodology_hash={}", selected.artifact.methodology_hash);
    println!("seed={}", FROZEN_SEED);
    println!(
        "development_best_fitness={}",
        evidence.development_best_fitness
    );
    println!("n_rules={}", selected.genome.rules.len());
    println!("n_instruments={}", RESEARCH_UNIVERSE.len());
    Ok(())
}

fn render_provenance(
    output: &PathBuf,
    cache_dir: &PathBuf,
    artifact_hash: &str,
    methodology_hash: &str,
) -> String {
    format!(
        "# CS-P-006-C policy discovery provenance\n\n\
         Coralys discovered. ChronoSentiment evaluated the sealed artifact independently.\n\n\
         **Not B4. Not B5.** Evaluation outcomes were not used for search.\n\n\
         - generated_at_wall_clock: {}\n\
         - output: {}\n\
         - yahoo_cache: {}\n\
         - snapshot: `{}`\n\
         - snapshot_identity_hash: `{}`\n\
         - seed: {}\n\
         - artifact_hash: `{}`\n\
         - methodology_hash: `{}`\n\
         - repeated in-process identity: same artifact_hash required\n",
        Utc::now(),
        output.display(),
        cache_dir.display(),
        RESEARCH_SNAPSHOT_DIR,
        RESEARCH_SNAPSHOT_IDENTITY_HASH,
        FROZEN_SEED,
        artifact_hash,
        methodology_hash,
    )
}

fn parse_args() -> Result<(PathBuf, PathBuf), Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1);
    let mut output = None;
    let mut cache = None;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--output" => output = Some(PathBuf::from(args.next().ok_or("missing --output")?)),
            "--yahoo-cache" => {
                cache = Some(PathBuf::from(args.next().ok_or("missing --yahoo-cache")?))
            }
            other => return Err(format!("unknown argument {other}").into()),
        }
    }
    Ok((
        output.ok_or("usage: csp006_policy_discovery --output DIR --yahoo-cache DIR")?,
        cache.ok_or("missing --yahoo-cache")?,
    ))
}
