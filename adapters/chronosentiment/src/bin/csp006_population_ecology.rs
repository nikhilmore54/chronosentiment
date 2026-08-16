//! CS-P-006-C.2-P — identity-gated replay of Search #1 with observability ON.
//!
//! This is not Search #2. The run is refused unless the sealed artifact matches
//! Search #1. Evaluation is not scored. Certified B3/B4 databases are refused.

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
use chronosentiment_adapter::decision_support::policy_discovery::{
    evolve_on_development_observed, select_and_observe,
};
use chronosentiment_adapter::decision_support::population_ecology::{
    analyze_search_archive, render_ecology,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (search_dir, cache_dir, output) = parse_args()?;
    let db = env::var("DATABASE_URL").unwrap_or_default();
    if db.contains("chrono_b3_test") || db.contains("chrono_b4_test") {
        return Err("refusing certified database name in DATABASE_URL".into());
    }

    let on_disk: PolicyArtifact =
        serde_json::from_str(&fs::read_to_string(search_dir.join("selected_policy.json"))?)?;
    if on_disk.artifact_hash != RESEARCH_DISCOVERY_ARTIFACT_HASH {
        return Err("refusing to analyze an artifact that is not Search #1".into());
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

    let (_, candidates, archive) =
        evolve_on_development_observed(development.clone()).map_err(|e| e.to_string())?;
    let (selected, archive) =
        select_and_observe(&candidates, &development, &selection, archive).map_err(|e| e.to_string())?;
    if selected.artifact.artifact_hash != RESEARCH_DISCOVERY_ARTIFACT_HASH {
        return Err("observability replay diverged from Search #1; refusing to write ecology".into());
    }
    if selected.artifact.artifact_hash != on_disk.artifact_hash {
        return Err("replay artifact does not match the on-disk Search #1 control".into());
    }

    let selected_identity = selected.genome.identity_hash();
    let report = analyze_search_archive(&archive, Some(&selected_identity))?;

    fs::create_dir_all(&output)?;
    fs::write(
        output.join("archive.json"),
        serde_json::to_vec_pretty(&archive)?,
    )?;
    fs::write(
        output.join("ecology.json"),
        serde_json::to_vec_pretty(&report)?,
    )?;
    fs::write(output.join("ECOLOGY.md"), render_ecology(&report))?;

    println!("result=PASS");
    println!("artifact_hash={}", selected.artifact.artifact_hash);
    println!("verdict={}", report.verdict);
    println!("output={}", output.display());
    println!("search_two_authorized=false");
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
        output.unwrap_or_else(|| PathBuf::from(RESEARCH_DISCOVERY_DIR).join("ecology")),
    ))
}
