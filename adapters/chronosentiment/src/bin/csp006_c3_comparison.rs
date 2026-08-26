//! CS-P-006-C.3-C — sealed Search #1 vs Search #2 review.
//!
//! Does not evolve. Does not overwrite either selected_policy.json.

use std::env;
use std::fs;
use std::path::PathBuf;

use chronosentiment_adapter::decision_support::c3_comparison::{
    compare_sealed_recommendations, render_comparison,
};
use chronosentiment_adapter::decision_support::csp006_protocol::{
    RESEARCH_DISCOVERY_ARTIFACT_HASH, RESEARCH_DISCOVERY_DIR, RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH,
    RESEARCH_DISCOVERY_TWO_DIR,
};
use chronosentiment_adapter::decision_support::policy_artifact::PolicyArtifact;
use chronosentiment_adapter::decision_support::recommendation_outcome::RecommendationRow;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (search_one, search_two, output) = parse_args()?;
    let db = env::var("DATABASE_URL").unwrap_or_default();
    if db.contains("chrono_b3_test") || db.contains("chrono_b4_test") {
        return Err("refusing certified database name in DATABASE_URL".into());
    }
    if output.ends_with("selected_policy.json") {
        return Err("refusing to overwrite selected_policy.json".into());
    }

    let one_art: PolicyArtifact = serde_json::from_str(&fs::read_to_string(
        search_one.join("selected_policy.json"),
    )?)?;
    let two_art: PolicyArtifact = serde_json::from_str(&fs::read_to_string(
        search_two.join("selected_policy.json"),
    )?)?;
    if one_art.artifact_hash != RESEARCH_DISCOVERY_ARTIFACT_HASH {
        return Err("refusing a left artifact that is not Search #1".into());
    }
    if two_art.artifact_hash != RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH {
        return Err("refusing a right artifact that is not Search #2".into());
    }

    let one_recs: Vec<RecommendationRow> = serde_json::from_str(&fs::read_to_string(
        search_one
            .join("recommendations")
            .join("recommendations.json"),
    )?)?;
    let two_recs: Vec<RecommendationRow> = serde_json::from_str(&fs::read_to_string(
        search_two
            .join("recommendations")
            .join("recommendations.json"),
    )?)?;

    let report = compare_sealed_recommendations(&one_recs, &two_recs, &two_art)?;

    fs::create_dir_all(&output)?;
    fs::write(
        output.join("comparison.json"),
        serde_json::to_vec_pretty(&report)?,
    )?;
    fs::write(output.join("REVIEW.md"), render_comparison(&report))?;
    fs::write(
        output.join("pairwise_rows.json"),
        serde_json::to_vec_pretty(&report.pairwise_rows)?,
    )?;
    fs::write(
        output.join("conversion_rows.json"),
        serde_json::to_vec_pretty(&report.conversion_rows)?,
    )?;
    fs::write(
        output.join("action_matrix.json"),
        serde_json::to_vec_pretty(&report.action_matrix)?,
    )?;

    println!("result=PASS");
    println!("search_one={}", report.search_one_artifact_hash);
    println!("search_two={}", report.search_two_artifact_hash);
    println!(
        "pairwise_all={}/{}/{}",
        report.pairwise_all.search_two_better,
        report.pairwise_all.search_one_better,
        report.pairwise_all.tie
    );
    println!(
        "converted_nt={}+{}",
        report.no_trade_conversion.n_converted_to_long,
        report.no_trade_conversion.n_converted_to_short
    );
    println!("search_three_authorized=false");
    println!("output={}", output.display());
    Ok(())
}

fn parse_args() -> Result<(PathBuf, PathBuf, PathBuf), Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1);
    let mut search_one = None;
    let mut search_two = None;
    let mut output = None;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--search-one-dir" => {
                search_one = Some(PathBuf::from(
                    args.next().ok_or("missing --search-one-dir")?,
                ))
            }
            "--search-two-dir" => {
                search_two = Some(PathBuf::from(
                    args.next().ok_or("missing --search-two-dir")?,
                ))
            }
            "--output" => output = Some(PathBuf::from(args.next().ok_or("missing --output")?)),
            other => return Err(format!("unknown argument {other}").into()),
        }
    }
    Ok((
        search_one.unwrap_or_else(|| PathBuf::from(RESEARCH_DISCOVERY_DIR)),
        search_two.unwrap_or_else(|| PathBuf::from(RESEARCH_DISCOVERY_TWO_DIR)),
        output.unwrap_or_else(|| PathBuf::from(RESEARCH_DISCOVERY_TWO_DIR).join("review")),
    ))
}
