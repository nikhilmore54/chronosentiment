//! CS-P-006-C.3-R — one complete Search #2 run.
//!
//! Evaluation is loaded only after the artifact is sealed.
//! Does not write chrono_b3_test / chrono_b4_test.
//! Does not overwrite Search #1.

use std::env;
use std::fs;
use std::path::PathBuf;

use chronosentiment_adapter::decision_support::c3_implementation::{
    living_selection_pool, search_one_evidence_is_immutable,
};
use chronosentiment_adapter::decision_support::c3_run::{
    c3_run_is_authorized, evolve_decision_value_on_development, refuse_search_one_output,
    render_decision_value_search, select_living_on_selection_value, C3_RUN_CONTRACT_ID,
};
use chronosentiment_adapter::decision_support::csp006_protocol::{
    RESEARCH_DISCOVERY_ARTIFACT_HASH, RESEARCH_DISCOVERY_DIR, RESEARCH_SNAPSHOT_DIR,
    RESEARCH_SNAPSHOT_IDENTITY_HASH, RESEARCH_UNIVERSE,
};
use chronosentiment_adapter::decision_support::csp006_snapshot::load_required_yahoo_cache;
use chronosentiment_adapter::decision_support::dataset_partition::{
    certified_research_partition, PartitionKind,
};
use chronosentiment_adapter::decision_support::decision_value_harness::{
    measure_sealed_artifact, render_harness,
};
use chronosentiment_adapter::decision_support::observation_value::build_observation_slice;
use chronosentiment_adapter::decision_support::policy_discovery::FROZEN_SEED;
use chronosentiment_adapter::decision_support::population_ecology::{
    analyze_search_archive, render_ecology,
};
use chronosentiment_adapter::decision_support::recommendation_outcome::score_recommendations;
use chrono::Utc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (output, cache_dir, search_one) = parse_args()?;
    let db = env::var("DATABASE_URL").unwrap_or_default();
    if db.contains("chrono_b3_test") || db.contains("chrono_b4_test") {
        return Err("refusing certified database name in DATABASE_URL".into());
    }
    if !c3_run_is_authorized() {
        return Err("C.3 Search #2 run is not authorized".into());
    }
    refuse_search_one_output(&output)?;
    search_one_evidence_is_immutable(&search_one)?;

    fs::create_dir_all(&output)?;
    let cache = load_required_yahoo_cache(&cache_dir).map_err(|e| e.to_string())?;
    let partition = certified_research_partition();

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

    let (evidence, archive) =
        evolve_decision_value_on_development(development.clone()).map_err(|e| e.to_string())?;
    let (evidence_repeat, archive_repeat) =
        evolve_decision_value_on_development(development.clone()).map_err(|e| e.to_string())?;
    if evidence.development_best_value != evidence_repeat.development_best_value {
        return Err("same seed produced a different development best value".into());
    }
    let pool = living_selection_pool(&archive).map_err(|e| e.to_string())?;
    let pool_repeat = living_selection_pool(&archive_repeat).map_err(|e| e.to_string())?;
    let selected =
        select_living_on_selection_value(&pool, &development, &selection).map_err(|e| e.to_string())?;
    let selected_repeat = select_living_on_selection_value(&pool_repeat, &development, &selection)
        .map_err(|e| e.to_string())?;
    if selected.artifact.artifact_hash != selected_repeat.artifact.artifact_hash {
        return Err("same seed produced a different PolicyArtifact identity".into());
    }
    if selected.artifact.artifact_hash == RESEARCH_DISCOVERY_ARTIFACT_HASH {
        return Err("Search #2 reused the Search #1 artifact".into());
    }

    fs::write(
        output.join("search_evidence.json"),
        serde_json::to_vec_pretty(&evidence)?,
    )?;
    fs::write(
        output.join("SEARCH.md"),
        render_decision_value_search(&evidence),
    )?;
    fs::write(
        output.join("selected_policy.json"),
        serde_json::to_vec_pretty(&selected.artifact)?,
    )?;
    fs::write(
        output.join("SELECTED.md"),
        render_selected(&selected.artifact.artifact_hash, &selected),
    )?;
    fs::write(
        output.join("archive.json"),
        serde_json::to_vec_pretty(&archive)?,
    )?;

    let evaluation = build_observation_slice(
        &cache,
        &partition.evaluation.timestamps,
        PartitionKind::Evaluation,
    )
    .map_err(|e| e.to_string())?;
    let (recommendations, _) =
        score_recommendations(&selected.artifact, &development, &selection, &evaluation)?;
    let rec_dir = output.join("recommendations");
    fs::create_dir_all(&rec_dir)?;
    fs::write(
        rec_dir.join("recommendations.json"),
        serde_json::to_vec_pretty(&recommendations)?,
    )?;

    let (_rows, harness) =
        measure_sealed_artifact(&selected.artifact.artifact_hash, &recommendations)?;
    let harness_dir = output.join("harness");
    fs::create_dir_all(&harness_dir)?;
    fs::write(
        harness_dir.join("harness.json"),
        serde_json::to_vec_pretty(&harness)?,
    )?;
    fs::write(harness_dir.join("HARNESS.md"), render_harness(&harness))?;
    fs::write(
        harness_dir.join("table_a_decision_distribution.json"),
        serde_json::to_vec_pretty(&harness.table_a_decision_distribution)?,
    )?;
    fs::write(
        harness_dir.join("table_b_decision_value.json"),
        serde_json::to_vec_pretty(&harness.table_b_decision_value)?,
    )?;

    let ecology = analyze_search_archive(&archive, Some(&selected.genome.identity_hash()))
        .map_err(|e| e.to_string())?;
    let ecology_dir = output.join("ecology");
    fs::create_dir_all(&ecology_dir)?;
    fs::write(
        ecology_dir.join("ecology.json"),
        serde_json::to_vec_pretty(&ecology)?,
    )?;
    fs::write(ecology_dir.join("ECOLOGY.md"), render_ecology(&ecology))?;

    fs::write(
        output.join("COMPARISON.md"),
        render_comparison(&harness, &ecology.verdict, pool.len()),
    )?;
    fs::write(
        output.join("PROVENANCE.md"),
        render_provenance(&output, &cache_dir, &selected.artifact.artifact_hash, &evidence.methodology_hash),
    )?;

    search_one_evidence_is_immutable(&search_one)?;

    println!("result=PASS");
    println!("contract_id={C3_RUN_CONTRACT_ID}");
    println!("artifact_hash={}", selected.artifact.artifact_hash);
    println!("methodology_hash={}", selected.artifact.methodology_hash);
    println!("search_one_artifact_hash={RESEARCH_DISCOVERY_ARTIFACT_HASH}");
    println!("seed={FROZEN_SEED}");
    println!("n_living_candidates={}", pool.len());
    println!(
        "development_value={:.6}",
        harness.development.protocol_value.value
    );
    println!(
        "selection_value={:.6}",
        harness.selection.protocol_value.value
    );
    println!(
        "evaluation_value={:.6}",
        harness.evaluation.protocol_value.value
    );
    println!("output={}", output.display());
    Ok(())
}

fn render_selected(
    artifact_hash: &str,
    selected: &chronosentiment_adapter::decision_support::policy_discovery::SelectedCandidate,
) -> String {
    format!(
        "# Selected PolicyArtifact — Search #2\n\n\
         Sealed after selection on the selection slice. Not retuned against evaluation.\n\n\
         - artifact_hash: `{}`\n\
         - methodology_hash: `{}`\n\
         - genome identity: `{}`\n\
         - n_rules: {}\n\
         - unmatched_action: {:?}\n\
         - development value: {:.6} (traded {}, stood aside {})\n\
         - selection value: {:.6} (traded {}, stood aside {})\n\n\
         Rules are the search result.\n\n```json\n{}\n```\n",
        artifact_hash,
        selected.artifact.methodology_hash,
        selected.genome.identity_hash(),
        selected.genome.rules.len(),
        selected.genome.unmatched_action,
        selected.development.fitness,
        selected.development.n_traded,
        selected.development.n_stood_aside,
        selected.selection.fitness,
        selected.selection.n_traded,
        selected.selection.n_stood_aside,
        serde_json::to_string_pretty(&selected.artifact.rules).unwrap(),
    )
}

fn render_comparison(
    harness: &chronosentiment_adapter::decision_support::decision_value_harness::HarnessReport,
    ecology_verdict: &str,
    n_living: usize,
) -> String {
    format!(
        "# Search #1 vs Search #2 — protocol comparison\n\n\
         Primary comparison is development / selection / evaluation protocol V.\n\
         Unique-best and SHORT presence are not success criteria.\n\n\
         | Measure | Search #1 | Search #2 |\n\
         |---|---:|---:|\n\
         | Development protocol V | 0.007559 | {:.6} |\n\
         | Selection protocol V | 0.006724 | {:.6} |\n\
         | Evaluation protocol V | -0.005825 | {:.6} |\n\
         | Mean regret (evaluation) | 0.056182 | {:.6} |\n\
         | Unique-best (evaluation) | 18.7% | {:.1}% |\n\
         | Living selection candidates | 2 elites | {} |\n\
         | Population ecology | SEARCH-SPACE EXPLORED | {} |\n\n\
         Search #1 remains `{}`.\n\
         Search #2 artifact `{}`.\n\
         Stop. Inspect this evidence before changing anything.\n",
        harness.development.protocol_value.value,
        harness.selection.protocol_value.value,
        harness.evaluation.protocol_value.value,
        harness.evaluation.diagnostics.mean_regret,
        100.0 * harness.evaluation.diagnostics.unique_best_share,
        n_living,
        ecology_verdict,
        RESEARCH_DISCOVERY_ARTIFACT_HASH,
        harness.policy_artifact_hash,
    )
}

fn render_provenance(
    output: &PathBuf,
    cache_dir: &PathBuf,
    artifact_hash: &str,
    methodology_hash: &str,
) -> String {
    format!(
        "# CS-P-006-C.3-R provenance\n\n\
         One complete Search #2 experiment. Search #1 was not overwritten.\n\
         Evaluation was loaded only after seal.\n\n\
         - generated_at_wall_clock: {}\n\
         - output: {}\n\
         - yahoo_cache: {}\n\
         - snapshot: `{}`\n\
         - snapshot_identity_hash: `{}`\n\
         - seed: {}\n\
         - n_instruments: {}\n\
         - artifact_hash: `{}`\n\
         - methodology_hash: `{}`\n\
         - search_one_artifact_hash: `{}`\n",
        Utc::now(),
        output.display(),
        cache_dir.display(),
        RESEARCH_SNAPSHOT_DIR,
        RESEARCH_SNAPSHOT_IDENTITY_HASH,
        FROZEN_SEED,
        RESEARCH_UNIVERSE.len(),
        artifact_hash,
        methodology_hash,
        RESEARCH_DISCOVERY_ARTIFACT_HASH,
    )
}

fn parse_args() -> Result<(PathBuf, PathBuf, PathBuf), Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1);
    let mut output = None;
    let mut cache = None;
    let mut search_one = None;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--output" => output = Some(PathBuf::from(args.next().ok_or("missing --output")?)),
            "--yahoo-cache" => {
                cache = Some(PathBuf::from(args.next().ok_or("missing --yahoo-cache")?))
            }
            "--search-one-dir" => {
                search_one = Some(PathBuf::from(args.next().ok_or("missing --search-one-dir")?))
            }
            other => return Err(format!("unknown argument {other}").into()),
        }
    }
    Ok((
        output.ok_or("usage: csp006_c3_search --output DIR --yahoo-cache DIR --search-one-dir DIR")?,
        cache.ok_or("missing --yahoo-cache")?,
        search_one.unwrap_or_else(|| PathBuf::from(RESEARCH_DISCOVERY_DIR)),
    ))
}
