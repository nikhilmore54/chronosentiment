//! CS-P-006-N — decision-value measurement harness.
//!
//! Does not evolve. Does not write chrono_b3_test / chrono_b4_test.
//! Does not overwrite Search #1 control files. C.3 is not authorized.

use std::env;
use std::fs;
use std::path::PathBuf;

use chronosentiment_adapter::decision_support::csp006_protocol::{
    RESEARCH_DISCOVERY_ARTIFACT_HASH, RESEARCH_DISCOVERY_DIR,
};
use chronosentiment_adapter::decision_support::decision_value_harness::{
    measure_harness, render_harness,
};
use chronosentiment_adapter::decision_support::recommendation_outcome::RecommendationRow;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (search_dir, output) = parse_args()?;
    let db = env::var("DATABASE_URL").unwrap_or_default();
    if db.contains("chrono_b3_test") || db.contains("chrono_b4_test") {
        return Err("refusing certified database name in DATABASE_URL".into());
    }
    if output.ends_with("selected_policy.json")
        || output.file_name().and_then(|n| n.to_str()) == Some("selected_policy.json")
    {
        return Err("refusing to overwrite Search #1 selected_policy.json".into());
    }

    let artifact: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(search_dir.join("selected_policy.json"))?)?;
    let artifact_hash = artifact["artifact_hash"]
        .as_str()
        .ok_or("selected_policy.json missing artifact_hash")?;
    if artifact_hash != RESEARCH_DISCOVERY_ARTIFACT_HASH {
        return Err("refusing to score an artifact that is not Search #1".into());
    }

    let rec_path = search_dir.join("recommendations").join("recommendations.json");
    let recommendations: Vec<RecommendationRow> =
        serde_json::from_str(&fs::read_to_string(rec_path)?)?;
    let (_rows, report) = measure_harness(artifact_hash, &recommendations)?;

    fs::create_dir_all(&output)?;
    fs::write(
        output.join("harness.json"),
        serde_json::to_vec_pretty(&report)?,
    )?;
    fs::write(output.join("HARNESS.md"), render_harness(&report))?;
    fs::write(
        output.join("table_a_decision_distribution.json"),
        serde_json::to_vec_pretty(&report.table_a_decision_distribution)?,
    )?;
    fs::write(
        output.join("table_b_decision_value.json"),
        serde_json::to_vec_pretty(&report.table_b_decision_value)?,
    )?;

    println!("result=PASS");
    println!("artifact_hash={}", report.policy_artifact_hash);
    println!("n_rows={}", report.n_rows);
    println!("protocol_value_all={:.6}", report.all.protocol_value.value);
    println!(
        "protocol_value_development={:.6}",
        report.development.protocol_value.value
    );
    println!(
        "protocol_value_selection={:.6}",
        report.selection.protocol_value.value
    );
    println!(
        "protocol_value_evaluation={:.6}",
        report.evaluation.protocol_value.value
    );
    println!("c3_authorized=false");
    println!("search_two_authorized=false");
    println!("used_as_coralys_fitness=false");
    println!("output={}", output.display());
    Ok(())
}

fn parse_args() -> Result<(PathBuf, PathBuf), Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1);
    let mut search_dir = None;
    let mut output = None;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--search-dir" => {
                search_dir = Some(PathBuf::from(args.next().ok_or("missing --search-dir")?))
            }
            "--output" => output = Some(PathBuf::from(args.next().ok_or("missing --output")?)),
            other => return Err(format!("unknown argument {other}").into()),
        }
    }
    Ok((
        search_dir.unwrap_or_else(|| PathBuf::from(RESEARCH_DISCOVERY_DIR)),
        output.unwrap_or_else(|| PathBuf::from(RESEARCH_DISCOVERY_DIR).join("harness")),
    ))
}
