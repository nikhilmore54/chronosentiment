//! CS-P-006-C.2-R — recommendation-vs-outcome of the sealed Search #1 artifact.
//!
//! Does not evolve. Does not write chrono_b3_test / chrono_b4_test.
//! Does not overwrite Search #1 evidence files. Evaluation is holdout
//! diagnosis and is not fed back to Coralys.

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
use chronosentiment_adapter::decision_support::recommendation_outcome::{
    render_scorecard, score_recommendations,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (search_dir, cache_dir, output) = parse_args()?;
    let db = env::var("DATABASE_URL").unwrap_or_default();
    if db.contains("chrono_b3_test") || db.contains("chrono_b4_test") {
        return Err("refusing certified database name in DATABASE_URL".into());
    }

    let artifact: PolicyArtifact = serde_json::from_str(&fs::read_to_string(
        search_dir.join("selected_policy.json"),
    )?)?;
    if artifact.artifact_hash != RESEARCH_DISCOVERY_ARTIFACT_HASH {
        return Err("refusing to score an artifact that is not Search #1".into());
    }

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

    let (rows, card) = score_recommendations(&artifact, &development, &selection, &evaluation)?;

    fs::create_dir_all(&output)?;
    fs::write(
        output.join("recommendations.json"),
        serde_json::to_vec_pretty(&rows)?,
    )?;
    fs::write(
        output.join("scorecard.json"),
        serde_json::to_vec_pretty(&card)?,
    )?;
    fs::write(output.join("SCORECARD.md"), render_scorecard(&card))?;

    println!("result=PASS");
    println!("artifact_hash={}", card.policy_artifact_hash);
    println!("n_recommendations={}", card.n_recommendations);
    println!("generalization={}", card.generalization);
    println!("search_two_authorized=false");
    println!("output={}", output.display());
    Ok(())
}

fn parse_args() -> Result<(PathBuf, PathBuf, PathBuf), Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1);
    let mut search_dir = None;
    let mut cache = None;
    let mut output = None;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--search-dir" => {
                search_dir = Some(PathBuf::from(args.next().ok_or("missing --search-dir")?))
            }
            "--yahoo-cache" => {
                cache = Some(PathBuf::from(args.next().ok_or("missing --yahoo-cache")?))
            }
            "--output" => output = Some(PathBuf::from(args.next().ok_or("missing --output")?)),
            other => return Err(format!("unknown argument {other}").into()),
        }
    }
    Ok((
        search_dir.unwrap_or_else(|| PathBuf::from(RESEARCH_DISCOVERY_DIR)),
        cache.unwrap_or_else(|| PathBuf::from(RESEARCH_SNAPSHOT_DIR).join("yahoo_cache")),
        output.unwrap_or_else(|| PathBuf::from(RESEARCH_DISCOVERY_DIR).join("recommendations")),
    ))
}
