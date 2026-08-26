//! CS-P-006-C.3-I — implementation verification. Does not evolve Search #2.
//!
//! Does not write chrono_b3_test / chrono_b4_test.
//! Does not overwrite Search #1 control files.

use std::env;
use std::fs;
use std::path::PathBuf;

use chrono::{TimeZone, Utc};
use chronosentiment_adapter::decision_support::c3_implementation::{
    evolve_on_development_value, identity_lineage_holds, living_selection_pool,
    post_seal_symbol_matrices_required, search_one_evidence_is_immutable,
    search_two_run_is_authorized, verify_implementation_contract, C3I_CONTRACT_ID,
    SEARCH_ONE_SELECTED_POLICY_FILE_SHA256,
};
use chronosentiment_adapter::decision_support::csp006_protocol::{
    RESEARCH_DISCOVERY_ARTIFACT_HASH, RESEARCH_DISCOVERY_DIR, RESEARCH_UNIVERSE,
};
use chronosentiment_adapter::decision_support::dataset_partition::PartitionKind;
use chronosentiment_adapter::decision_support::decision_value_fitness::score_decision_value;
use chronosentiment_adapter::decision_support::observation_value::{
    ObservationRow, ObservationSlice,
};
use chronosentiment_adapter::decision_support::policy_genome::RuleListGenome;
use chronosentiment_adapter::decision_support::search_observability::{
    GenerationPopulationRecord, OffspringEdge, SearchArchive, SerializedGenome,
};
use chronosentiment_adapter::decision_support::DecisionAction;
use chronosentiment_adapter::metrics::concepts::Concept;
use chronosentiment_adapter::reasoning::assessment::AssessmentEngine;
use coralys_moga::runtime::optimization::metric::{MetricReport, MetricValue};
use uuid::Uuid;

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
    if output == search_dir {
        return Err("refusing to write into the Search #1 evidence directory".into());
    }

    identity_lineage_holds()?;
    search_one_evidence_is_immutable(&search_dir)?;

    let always_long = RuleListGenome {
        rules: vec![],
        unmatched_action: DecisionAction::Long,
    };
    let evaluation = synthetic_slice(PartitionKind::Evaluation, 0.01);
    if score_decision_value(&always_long, &evaluation).is_ok() {
        return Err("decision-value fitness must reject evaluation".into());
    }
    if evolve_on_development_value(synthetic_slice(PartitionKind::Development, 0.01)).is_ok() {
        return Err("decision-value evolve must stay blocked".into());
    }

    let small = score_decision_value(
        &always_long,
        &synthetic_slice(PartitionKind::Development, 0.001),
    )?;
    let mid = score_decision_value(
        &always_long,
        &synthetic_slice(PartitionKind::Development, 0.005),
    )?;
    let large = score_decision_value(
        &always_long,
        &synthetic_slice(PartitionKind::Development, 0.05),
    )?;
    if !(small.fitness < mid.fitness && mid.fitness < large.fitness) {
        return Err("fitness must preserve economic magnitude".into());
    }

    let pool = living_selection_pool(&synthetic_living_archive())?;
    if pool.len() != 3 {
        return Err("living pool must be unique living-slot identities, not offspring".into());
    }

    let contract = verify_implementation_contract();
    if search_two_run_is_authorized() || contract.result != "PASS" {
        return Err("Search #2 run is not authorized".into());
    }
    if !post_seal_symbol_matrices_required() {
        return Err("N symbol matrices must remain required after a future seal".into());
    }

    fs::create_dir_all(&output)?;
    let report = serde_json::json!({
        "contract_id": C3I_CONTRACT_ID,
        "search_one_artifact_hash": RESEARCH_DISCOVERY_ARTIFACT_HASH,
        "search_one_selected_policy_sha256": SEARCH_ONE_SELECTED_POLICY_FILE_SHA256,
        "search_one_immutable": true,
        "search_two_run_authorized": false,
        "n_instruments": RESEARCH_UNIVERSE.len(),
        "horizon_days": 20,
        "seed": 42,
        "evaluation_inaccessible": true,
        "living_pool_excludes_unentered_offspring": true,
        "magnitude_preserved": true,
        "post_seal_symbol_matrices_required": true,
        "result": "PASS",
    });
    fs::write(
        output.join("verification.json"),
        serde_json::to_vec_pretty(&report)?,
    )?;
    fs::write(output.join("IMPLEMENTATION.md"), render_report())?;

    println!("result=PASS");
    println!("contract_id={C3I_CONTRACT_ID}");
    println!("search_one_artifact_hash={RESEARCH_DISCOVERY_ARTIFACT_HASH}");
    println!("search_two_run_authorized=false");
    println!("c3i_implementation_pass=true");
    println!("search_two_not_run=true");
    println!("output={}", output.display());
    Ok(())
}

fn render_report() -> String {
    format!(
        "# CS-P-006-C.3-I implementation verification\n\n\
         **Result:** PASS\n\n\
         Search #1 remains `{RESEARCH_DISCOVERY_ARTIFACT_HASH}`.\n\
         Search #2 was not run. `SEARCH_TWO_RUN_AUTHORIZED` remains false.\n\n\
         Identity lineage (TMV snapshot, 7 instruments, 20D, seed 42, MOGA knobs,\n\
         Search #1 methodology hash, evaluation inaccessible) holds.\n\
         Living-population unique identities are the selection pool.\n\
         M.1 `ProtocolValue` is the only fitness constructor.\n"
    )
}

fn profile() -> chronosentiment_adapter::reasoning::assessment::AssessmentProfile {
    let t = Utc.with_ymd_and_hms(2021, 10, 31, 15, 30, 0).unwrap();
    let mut metrics = MetricReport::default();
    metrics
        .metrics
        .insert("ma_20".to_string(), MetricValue::Float(110.0));
    metrics
        .metrics
        .insert("ma_50".to_string(), MetricValue::Float(100.0));
    metrics
        .metrics
        .insert("roc_20".to_string(), MetricValue::Float(0.05));
    metrics
        .metrics
        .insert("atr_14".to_string(), MetricValue::Float(1.2));
    AssessmentEngine.assess_at(
        &metrics,
        &[Concept::Trend, Concept::Momentum, Concept::Volatility],
        t,
        Some(Uuid::from_u128(7)),
    )
}

fn synthetic_slice(kind: PartitionKind, instrument_return: f64) -> ObservationSlice {
    let t = Utc.with_ymd_and_hms(2021, 10, 31, 15, 30, 0).unwrap();
    let rows = RESEARCH_UNIVERSE
        .iter()
        .map(|ticker| ObservationRow {
            instrument: (*ticker).to_string(),
            as_of: t,
            profile: profile(),
            instrument_return: Some(instrument_return),
        })
        .collect();
    ObservationSlice { kind, rows }
}

fn slot(identity: &str) -> SerializedGenome {
    SerializedGenome {
        identity: identity.to_string(),
        development_fitness: 0.0,
        rules: vec![],
        unmatched_action: DecisionAction::NoTrade,
    }
}

fn synthetic_living_archive() -> SearchArchive {
    SearchArchive {
        contract_id: "csp006c2o.search_observability.1".to_string(),
        generations: vec![
            GenerationPopulationRecord {
                generation: 0,
                population_size: 2,
                unique_genome_count: 2,
                best_fitness: 0.0,
                median_fitness: 0.0,
                mean_fitness: 0.0,
                worst_fitness: 0.0,
                action_symbols: Default::default(),
                factor_consumption: Default::default(),
                generation_best: slot("A"),
                near_best: vec![],
                living_slots: vec![slot("A"), slot("B")],
            },
            GenerationPopulationRecord {
                generation: 1,
                population_size: 2,
                unique_genome_count: 2,
                best_fitness: 0.0,
                median_fitness: 0.0,
                mean_fitness: 0.0,
                worst_fitness: 0.0,
                action_symbols: Default::default(),
                factor_consumption: Default::default(),
                generation_best: slot("A"),
                near_best: vec![],
                living_slots: vec![slot("A"), slot("C")],
            },
        ],
        offspring: vec![OffspringEdge {
            generation: 1,
            parent_a_identity: "A".into(),
            parent_b_identity: "B".into(),
            child_identity: "D".into(),
        }],
        selected_instruments: None,
    }
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
        output.unwrap_or_else(|| PathBuf::from(RESEARCH_DISCOVERY_DIR).join("c3i")),
    ))
}
