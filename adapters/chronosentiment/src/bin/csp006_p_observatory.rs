//! CS-P-006-P.3–P.7 — sealed-then-measured observatory path and product screens.
//!
//! Historical path demonstration on the evaluation slice. Does not evolve.
//! Does not overwrite selected_policy.json. Does not start C.3-G.

use std::env;
use std::fs;
use std::path::PathBuf;

use chronosentiment_adapter::decision_support::csp006_protocol::{
    RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH, RESEARCH_DISCOVERY_TWO_DIR,
};
use chronosentiment_adapter::decision_support::dataset_partition::PartitionKind;
use chronosentiment_adapter::decision_support::observatory_slice::{
    append_observation, empty_ledger, generate_decision, observe_outcome, render_observatory_html,
    seal_into_ledger,
};
use chronosentiment_adapter::decision_support::policy_artifact::PolicyArtifact;
use chronosentiment_adapter::decision_support::recommendation_outcome::RecommendationRow;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (search_two, output) = parse_args()?;
    let db = env::var("DATABASE_URL").unwrap_or_default();
    if db.contains("chrono_b3_test") || db.contains("chrono_b4_test") {
        return Err("refusing certified database name in DATABASE_URL".into());
    }
    if output.ends_with("selected_policy.json") {
        return Err("refusing to overwrite selected_policy.json".into());
    }

    let artifact: PolicyArtifact =
        serde_json::from_str(&fs::read_to_string(search_two.join("selected_policy.json"))?)?;
    if artifact.artifact_hash != RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH {
        return Err("refusing an artifact that is not C3-002 / Search #2".into());
    }
    let recommendations: Vec<RecommendationRow> = serde_json::from_str(&fs::read_to_string(
        search_two.join("recommendations").join("recommendations.json"),
    )?)?;

    let mut ledger = empty_ledger();
    let mut sealed = 0u32;
    let mut observed = 0u32;
    for row in recommendations
        .iter()
        .filter(|r| r.partition == PartitionKind::Evaluation)
    {
        let decision = generate_decision(
            &artifact,
            &row.instrument,
            &row.timestamp,
            &row.trend_state,
            &row.momentum_state,
            &row.volatility_state,
        )?;
        if decision.action != row.recommendation {
            return Err(format!(
                "C3-002 action mismatch at {} {}",
                row.instrument, row.timestamp
            )
            .into());
        }
        seal_into_ledger(&mut ledger, decision.clone())?;
        sealed += 1;
        if let Some(realized) = row.actual_forward_return {
            let observation = observe_outcome(&decision, &row.timestamp, realized)?;
            append_observation(&mut ledger, observation)?;
            observed += 1;
        }
    }

    fs::create_dir_all(&output)?;
    fs::write(output.join("ledger.json"), serde_json::to_vec_pretty(&ledger)?)?;
    fs::write(
        output.join("observatory.html"),
        render_observatory_html(&ledger, chrono::Utc::now()),
    )?;

    println!("result=PASS");
    println!("policy_id={}", ledger.policy_id);
    println!("artifact_hash={}", ledger.policy_artifact_sha256);
    println!("path_kind={}", ledger.path_kind);
    println!("sealed={sealed}");
    println!("observed={observed}");
    println!("search_three_authorized={}", ledger.search_three_authorized);
    println!("output={}", output.display());
    Ok(())
}

fn parse_args() -> Result<(PathBuf, PathBuf), Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1);
    let mut search_two = None;
    let mut output = None;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--search-two-dir" => {
                search_two = Some(PathBuf::from(args.next().ok_or("missing --search-two-dir")?))
            }
            "--output" => output = Some(PathBuf::from(args.next().ok_or("missing --output")?)),
            other => return Err(format!("unknown argument {other}").into()),
        }
    }
    Ok((
        search_two.unwrap_or_else(|| PathBuf::from(RESEARCH_DISCOVERY_TWO_DIR)),
        output.unwrap_or_else(|| PathBuf::from("product_validation/CS-P-006/observatory")),
    ))
}
