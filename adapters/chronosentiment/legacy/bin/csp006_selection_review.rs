//! CS-P-006-C.2-S — selection and decision-value review of Search #1.
//!
//! Does not evolve. Does not overwrite Search #1 files.
//! Evaluation is holdout diagnosis of the two protocol elites only.

use std::env;
use std::fs;
use std::path::PathBuf;

use chronosentiment_adapter::decision_support::csp006_protocol::{
    RESEARCH_DISCOVERY_ARTIFACT_HASH, RESEARCH_DISCOVERY_DIR, RESEARCH_SNAPSHOT_DIR,
};
use chronosentiment_adapter::decision_support::csp006_snapshot::load_required_yahoo_cache;
use chronosentiment_adapter::decision_support::dataset_partition::{
    certified_research_partition, PartitionKind,
};
use chronosentiment_adapter::decision_support::observation_value::build_observation_slice;
use chronosentiment_adapter::decision_support::policy_artifact::PolicyArtifact;
use chronosentiment_adapter::decision_support::search_observability::SearchArchive;
use chronosentiment_adapter::decision_support::selection_decision_value::{
    render_review, review_selection,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (search_dir, cache_dir, archive_path, output) = parse_args()?;
    let db = env::var("DATABASE_URL").unwrap_or_default();
    if db.contains("chrono_b3_test") || db.contains("chrono_b4_test") {
        return Err("refusing certified database name in DATABASE_URL".into());
    }

    let artifact: PolicyArtifact = serde_json::from_str(&fs::read_to_string(
        search_dir.join("selected_policy.json"),
    )?)?;
    if artifact.artifact_hash != RESEARCH_DISCOVERY_ARTIFACT_HASH {
        return Err("refusing to review an artifact that is not Search #1".into());
    }
    let archive: SearchArchive = serde_json::from_str(&fs::read_to_string(archive_path)?)?;

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
    let evaluation = build_observation_slice(
        &cache,
        &partition.evaluation.timestamps,
        PartitionKind::Evaluation,
    )
    .map_err(|e| e.to_string())?;

    let report = review_selection(
        &artifact.artifact_hash,
        &archive,
        &development,
        &selection,
        &evaluation,
    )?;

    fs::create_dir_all(&output)?;
    fs::write(
        output.join("selection_review.json"),
        serde_json::to_vec_pretty(&report)?,
    )?;
    fs::write(output.join("SELECTION_REVIEW.md"), render_review(&report))?;

    println!("result=PASS");
    println!("artifact_hash={}", report.policy_artifact_hash);
    println!(
        "n_beat_selected={}",
        report.bottleneck.n_that_beat_selected_on_selection
    );
    println!("search_two_authorized=false");
    println!("output={}", output.display());
    Ok(())
}

fn parse_args() -> Result<(PathBuf, PathBuf, PathBuf, PathBuf), Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1);
    let mut search_dir = None;
    let mut cache = None;
    let mut archive = None;
    let mut output = None;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--search-dir" => {
                search_dir = Some(PathBuf::from(args.next().ok_or("missing --search-dir")?))
            }
            "--yahoo-cache" => {
                cache = Some(PathBuf::from(args.next().ok_or("missing --yahoo-cache")?))
            }
            "--archive" => archive = Some(PathBuf::from(args.next().ok_or("missing --archive")?)),
            "--output" => output = Some(PathBuf::from(args.next().ok_or("missing --output")?)),
            other => return Err(format!("unknown argument {other}").into()),
        }
    }
    let search = search_dir.unwrap_or_else(|| PathBuf::from(RESEARCH_DISCOVERY_DIR));
    let archive_path = archive.unwrap_or_else(|| search.join("ecology").join("archive.json"));
    Ok((
        search.clone(),
        cache.unwrap_or_else(|| PathBuf::from(RESEARCH_SNAPSHOT_DIR).join("yahoo_cache")),
        archive_path,
        output.unwrap_or_else(|| search.join("selection_review")),
    ))
}
