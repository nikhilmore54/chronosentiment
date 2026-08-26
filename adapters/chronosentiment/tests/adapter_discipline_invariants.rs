//! CS-P-CLEAN-001 / CS-P-CLEAN-002 repository invariants.

use std::fs;
use std::path::{Path, PathBuf};

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(rel: &str) -> String {
    fs::read_to_string(crate_root().join(rel)).unwrap_or_else(|e| panic!("read {rel}: {e}"))
}

/// Decision-producing product sources (no lake generators, no research, no outcome SQL).
const DECISION_PRODUCERS: &[&str] = &[
    "src/decision_support/mod.rs",
    "src/decision_support/policy.rs",
    "src/decision_support/policy_artifact.rs",
    "src/decision_support/replay.rs",
    "src/decision_support/forward.rs",
    "src/decision_support/forward_tick.rs",
    "src/decision_support/backtest.rs",
];

/// Sources that must not import the outcome/performance layers.
/// `forward.rs` currently also hosts `ForwardJournal::performance` (evaluation);
/// splitting that is not part of CS-P-CLEAN-001.
const DECIDE_WITHOUT_EVALUATION: &[&str] = &[
    "src/decision_support/policy.rs",
    "src/decision_support/policy_artifact.rs",
    "src/decision_support/csp006_snapshot.rs",
    "src/decision_support/dataset_partition.rs",
    "src/decision_support/replay.rs",
    "src/decision_support/forward_tick.rs",
    "src/decision_support/backtest.rs",
];

fn walk_rs(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            walk_rs(&path, out);
        } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
            out.push(path);
        }
    }
}

#[test]
fn product_lib_gates_research_module() {
    let lib = read("src/lib.rs");
    assert!(
        lib.contains("#[cfg(feature = \"research\")]"),
        "research module must be feature-gated"
    );
}

#[test]
fn decision_producers_do_not_import_research_or_lake_generators() {
    for rel in DECISION_PRODUCERS {
        let src = read(rel);
        assert!(
            !src.contains("crate::research") && !src.contains("chronosentiment_adapter::research"),
            "{rel} must not import research"
        );
        assert!(
            !src.contains("DecisionEngine"),
            "{rel} must not instantiate the B3/B4 DecisionEngine"
        );
        assert!(
            !src.contains("StrategyEngine"),
            "{rel} must not instantiate the B3/B4 StrategyEngine"
        );
        let lower = src.to_lowercase();
        assert!(
            !lower.contains("from knowledge_outcomes")
                && !lower.contains("join knowledge_outcomes"),
            "{rel} must not query knowledge_outcomes"
        );
    }
}

#[test]
fn decide_sources_do_not_import_outcome_or_performance() {
    for rel in DECIDE_WITHOUT_EVALUATION {
        let src = read(rel);
        assert!(
            !src.contains("crate::decision_support::outcome")
                && !src.contains("super::outcome::")
                && !src.contains("crate::decision_support::performance")
                && !src.contains("super::performance::")
                && !src.contains("measure_performance"),
            "{rel} must not import outcome/performance"
        );
    }
}

#[test]
fn default_src_tree_has_no_research_module() {
    assert!(
        !crate_root().join("src/research").exists(),
        "src/research must not exist on the product surface; use research/ behind the feature"
    );
}

#[test]
fn phase_c_gate_script_must_not_exist() {
    assert!(
        !crate_root().join("scripts/phase_c_gate.sh").exists(),
        "phase_c_gate.sh printed PASS without running tests and must stay removed"
    );
}

#[test]
fn week2_tests_are_quarantined_not_compiled() {
    assert!(
        crate_root()
            .join("legacy/quarantine/tests/week2_tests.rs")
            .exists(),
        "week2_tests.rs must be preserved in quarantine"
    );
    assert!(
        !crate_root().join("tests/week2_tests.rs").exists(),
        "week2_tests.rs must not be an active test"
    );
}

#[test]
fn b3_b4_generators_are_preserved_behind_legacy_lake() {
    let populate = crate_root().join("legacy/bin/m4_populate_knowledge_lake.rs");
    assert!(
        populate.exists(),
        "m4_populate_knowledge_lake must remain for B3/B4 reproduction"
    );
    let decision = read("src/reasoning/decision.rs");
    assert!(
        decision.contains("#[cfg(feature = \"legacy-lake\")]"),
        "DecisionEngine must be feature-gated, not deleted"
    );
    assert!(
        decision.contains("pub struct DecisionEngine"),
        "DecisionEngine source must be preserved"
    );
    let strategy = read("src/reasoning/strategy.rs");
    assert!(
        strategy.contains("#[cfg(feature = \"legacy-lake\")]"),
        "StrategyEngine must be feature-gated, not deleted"
    );
    assert!(
        strategy.contains("pub struct StrategyEngine"),
        "StrategyEngine source must be preserved"
    );
}

#[test]
fn product_src_bin_contains_only_csp_binaries() {
    let mut bins = Vec::new();
    walk_rs(&crate_root().join("src/bin"), &mut bins);
    let names: Vec<String> = bins
        .iter()
        .filter_map(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .map(|s| s.to_string())
        })
        .collect();
    for name in &names {
        assert!(
            name.starts_with("csp"),
            "src/bin/{name} is not a product CS-P binary; move it to legacy/ or research/"
        );
    }
    assert!(
        !names.is_empty(),
        "product CS-P binaries must remain in src/bin"
    );
}

#[test]
fn product_decide_paths_require_an_explicit_policy_argument() {
    let replay = read("src/decision_support/replay.rs");
    assert!(
        !replay.contains("fn decide_from_inputs(inputs: ReplayInputs)"),
        "one-argument decide_from_inputs is the implicit-default hole"
    );
    assert!(
        replay.contains("policy: &P") || replay.contains("policy: &dyn DecisionPolicy"),
        "decide_from_inputs must take a policy"
    );
    assert!(
        replay.contains("policy: &dyn DecisionPolicy"),
        "DecideAt::decide_at must take &dyn DecisionPolicy"
    );
    assert!(
        !replay.contains("TrendMappingPolicy"),
        "old implicit TrendMappingPolicy name must not remain on the replay path"
    );
    let backtest = read("src/decision_support/backtest.rs");
    assert!(
        backtest.contains("policy: &dyn DecisionPolicy"),
        "backtest must thread an explicit policy"
    );
    let forward = read("src/decision_support/forward.rs");
    assert!(
        forward.contains("policy: &P") || forward.contains("policy: &dyn DecisionPolicy"),
        "forward decide must take a policy"
    );
}

#[test]
fn policy_artifact_is_an_evaluator_not_an_optimizer() {
    let src = read("src/decision_support/policy_artifact.rs");
    assert!(
        !src.contains("EvolutionEngine")
            && !src.contains("FitnessEvaluator")
            && !src.contains("rand::"),
        "CS-P-006-A must not contain a search engine"
    );
    assert!(
        !src.contains("from knowledge_outcomes") && !src.contains("OutcomeReport"),
        "PolicyArtifact evaluation must not read outcomes"
    );
}

#[test]
fn product_binaries_still_select_the_baseline_fixture_explicitly() {
    for rel in [
        "src/bin/csp002_b4_historical_run.rs",
        "src/bin/csp003_forward_session.rs",
        "src/bin/csp004_historical_lab.rs",
    ] {
        let src = read(rel);
        assert!(
            src.contains("BaselineTrendMappingPolicy"),
            "{rel} must keep explicit baseline fixture; CS-P-006-A does not replace it"
        );
        assert!(
            !src.contains("ArtifactDecisionPolicy"),
            "{rel} must not switch to a policy artifact in CS-P-006-A"
        );
    }
}

#[test]
fn search_diagnosis_binary_does_not_evolve() {
    let src = read("src/bin/csp006_search_diagnosis.rs");
    assert!(
        src.contains("diagnose_sealed_artifact"),
        "diagnosis binary must inspect the sealed artifact"
    );
    assert!(
        !src.contains("evolve_on_development") && !src.contains("BaselineTrendMappingPolicy"),
        "diagnosis must not search or promote the baseline"
    );
}

#[test]
fn policy_discovery_binary_does_not_hand_write_or_promote_the_baseline() {
    let src = read("src/bin/csp006_policy_discovery.rs");
    assert!(
        src.contains("evolve_on_development") && src.contains("evaluate_sealed_candidate"),
        "discovery binary must run Coralys search then ChronoSentiment handoff"
    );
    assert!(
        !src.contains("BaselineTrendMappingPolicy"),
        "discovery must not promote the baseline fixture"
    );
    assert!(
        !src.contains("train_policy")
            && !src.contains("CoralysPhase")
            && !src.contains("b5_strategy"),
        "discovery binary must not introduce phase names into the run"
    );
}
