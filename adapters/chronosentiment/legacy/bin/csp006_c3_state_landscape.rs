//! CS-P-006-C.3-F — certified TMV state × action landscape.
//!
//! Does not evolve. Does not overwrite selected_policy.json.
//! Does not authorize a product claim.

use std::env;
use std::fs;
use std::path::PathBuf;

use chronosentiment_adapter::decision_support::c3_rule_ecology::SEARCH_THREE_AUTHORIZED;
use chronosentiment_adapter::decision_support::c3_state_landscape::{
    analyze_state_landscape, render_state_landscape, PRODUCT_CLAIM_AUTHORIZED,
};
use chronosentiment_adapter::decision_support::csp006_protocol::{
    RESEARCH_DISCOVERY_DIR, RESEARCH_DISCOVERY_TWO_DIR,
};
use chronosentiment_adapter::decision_support::recommendation_outcome::RecommendationRow;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (search_one_dir, search_two_dir, output) = parse_args()?;
    let db = env::var("DATABASE_URL").unwrap_or_default();
    if db.contains("chrono_b3_test") || db.contains("chrono_b4_test") {
        return Err("refusing certified database name in DATABASE_URL".into());
    }
    if output.ends_with("selected_policy.json") {
        return Err("refusing to overwrite selected_policy.json".into());
    }

    let search_one: Vec<RecommendationRow> = serde_json::from_str(&fs::read_to_string(
        search_one_dir
            .join("recommendations")
            .join("recommendations.json"),
    )?)?;
    let search_two: Vec<RecommendationRow> = serde_json::from_str(&fs::read_to_string(
        search_two_dir
            .join("recommendations")
            .join("recommendations.json"),
    )?)?;

    let report = analyze_state_landscape(&search_one, &search_two)?;
    fs::create_dir_all(&output)?;
    fs::write(
        output.join("landscape.json"),
        serde_json::to_vec_pretty(&report)?,
    )?;
    fs::write(output.join("LANDSCAPE.md"), render_state_landscape(&report))?;

    println!("result=PASS");
    println!("search_one={}", report.search_one_artifact_hash);
    println!("search_two={}", report.search_two_artifact_hash);
    println!("promotion_status={}", report.promotion_status);
    println!("search_three_authorized={SEARCH_THREE_AUTHORIZED}");
    println!("product_claim_authorized={PRODUCT_CLAIM_AUTHORIZED}");
    println!("observed_states={}", report.occupancy.n_observed_states);
    for state in &report.states {
        println!(
            "state={}/{}/{} n={} long={:.6} eval_long={:.6}",
            state.trend_state,
            state.momentum_state,
            state.volatility_state,
            state.n,
            state.overall.long,
            state.evaluation.long
        );
    }
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
        output.unwrap_or_else(|| PathBuf::from(RESEARCH_DISCOVERY_TWO_DIR).join("state_landscape")),
    ))
}
