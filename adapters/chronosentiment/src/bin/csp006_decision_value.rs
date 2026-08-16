//! CS-P-006-C.2-D — decision-value landscape of the sealed Search #1 artifact.
//!
//! Consumes the existing C.2-R recommendation matrix. Does not evolve.
//! Does not write chrono_b3_test / chrono_b4_test.
//! Does not overwrite Search #1 evidence files. Evaluation is holdout
//! diagnosis and is not fed back to Coralys. Advantage is not fitness.
//! No borderline band is frozen.

use std::env;
use std::fs;
use std::path::PathBuf;

use chronosentiment_adapter::decision_support::csp006_protocol::{
    RESEARCH_DISCOVERY_ARTIFACT_HASH, RESEARCH_DISCOVERY_DIR,
};
use chronosentiment_adapter::decision_support::decision_value_landscape::{
    analyze_landscape, render_landscape,
};
use chronosentiment_adapter::decision_support::recommendation_outcome::RecommendationRow;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (search_dir, output) = parse_args()?;
    let db = env::var("DATABASE_URL").unwrap_or_default();
    if db.contains("chrono_b3_test") || db.contains("chrono_b4_test") {
        return Err("refusing certified database name in DATABASE_URL".into());
    }

    let artifact_path = search_dir.join("selected_policy.json");
    let artifact: serde_json::Value = serde_json::from_str(&fs::read_to_string(artifact_path)?)?;
    let artifact_hash = artifact["artifact_hash"]
        .as_str()
        .ok_or("selected_policy.json missing artifact_hash")?;
    if artifact_hash != RESEARCH_DISCOVERY_ARTIFACT_HASH {
        return Err("refusing to score an artifact that is not Search #1".into());
    }

    let rec_path = search_dir.join("recommendations").join("recommendations.json");
    let recommendations: Vec<RecommendationRow> =
        serde_json::from_str(&fs::read_to_string(rec_path)?)?;
    if recommendations.len() != 273 {
        return Err(format!(
            "expected the sealed 273-row C.2-R matrix, found {}",
            recommendations.len()
        )
        .into());
    }
    if recommendations
        .iter()
        .any(|r| r.policy_artifact_hash != RESEARCH_DISCOVERY_ARTIFACT_HASH)
    {
        return Err("recommendation matrix is not Search #1".into());
    }

    let (rows, card) = analyze_landscape(artifact_hash, &recommendations)?;

    fs::create_dir_all(&output)?;
    fs::write(output.join("rows.json"), serde_json::to_vec_pretty(&rows)?)?;
    fs::write(
        output.join("landscape.json"),
        serde_json::to_vec_pretty(&card)?,
    )?;
    fs::write(output.join("LANDSCAPE.md"), render_landscape(&card))?;

    println!("result=PASS");
    println!("artifact_hash={}", card.policy_artifact_hash);
    println!("n_rows={}", card.n_rows);
    println!(
        "mean_recommended_value={:.6}",
        card.overall.mean_recommended_value
    );
    println!("mean_regret={:.6}", card.overall.mean_regret);
    println!("borderline_band_frozen=false");
    println!("used_as_coralys_fitness=false");
    println!("search_two_authorized=false");
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
            "--yahoo-cache" => {
                let _ = args.next();
            }
            other => return Err(format!("unknown argument {other}").into()),
        }
    }
    Ok((
        search_dir.unwrap_or_else(|| PathBuf::from(RESEARCH_DISCOVERY_DIR)),
        output.unwrap_or_else(|| PathBuf::from(RESEARCH_DISCOVERY_DIR).join("decision_value")),
    ))
}
