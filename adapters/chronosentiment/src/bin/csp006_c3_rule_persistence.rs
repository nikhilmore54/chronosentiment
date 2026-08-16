//! CS-P-006-C.3-E — persistence of sealed Search #2 live rules.
//!
//! Does not evolve. Does not overwrite selected_policy.json.
//! Does not introduce a pass/fail threshold.

use std::env;
use std::fs;
use std::path::PathBuf;

use chronosentiment_adapter::decision_support::c3_rule_ecology::SEARCH_THREE_AUTHORIZED;
use chronosentiment_adapter::decision_support::c3_rule_persistence::{
    analyze_rule_persistence, render_rule_persistence, PASS_THRESHOLD_INTRODUCED,
};
use chronosentiment_adapter::decision_support::csp006_protocol::{
    RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH, RESEARCH_DISCOVERY_TWO_DIR,
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
        return Err("refusing an artifact that is not Search #2".into());
    }
    let recommendations: Vec<RecommendationRow> = serde_json::from_str(&fs::read_to_string(
        search_two.join("recommendations").join("recommendations.json"),
    )?)?;

    let report = analyze_rule_persistence(&recommendations, &artifact)?;
    fs::create_dir_all(&output)?;
    fs::write(
        output.join("persistence.json"),
        serde_json::to_vec_pretty(&report)?,
    )?;
    fs::write(output.join("PERSISTENCE.md"), render_rule_persistence(&report))?;

    println!("result=PASS");
    println!("artifact_hash={}", report.search_two_artifact_hash);
    println!("promotion_status={}", report.promotion_status);
    println!("search_three_authorized={SEARCH_THREE_AUTHORIZED}");
    println!("pass_threshold_introduced={PASS_THRESHOLD_INTRODUCED}");
    for rule in &report.rules {
        println!(
            "rule_{} n={} mean_v={:.6} eval_n={} eval_v={}",
            rule.rule_index,
            rule.n,
            rule.mean_v,
            rule.n_evaluation,
            rule.evaluation_mean_v
                .map(|v| format!("{v:.6}"))
                .unwrap_or_else(|| "none".into())
        );
    }
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
        output.unwrap_or_else(|| PathBuf::from(RESEARCH_DISCOVERY_TWO_DIR).join("rule_persistence")),
    ))
}
