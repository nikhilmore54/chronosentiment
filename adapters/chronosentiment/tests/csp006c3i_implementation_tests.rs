//! CS-P-006-C.3-I — implementation and identity gate. Search #2 is not run.

use std::collections::BTreeMap;
use std::path::PathBuf;

use chrono::{TimeZone, Utc};
use chronosentiment_adapter::decision_support::c3_implementation::{
    evolve_on_development_value, genome_from_living_slot, identity_lineage_holds,
    living_selection_pool, post_seal_symbol_matrices_required, search_one_evidence_is_immutable,
    search_two_run_is_authorized, select_on_selection_value, verify_implementation_contract,
    SEARCH_TWO_RUN_AUTHORIZED,
};
use chronosentiment_adapter::decision_support::csp006_protocol::{
    RESEARCH_DISCOVERY_ARTIFACT_HASH, RESEARCH_DISCOVERY_DIR, RESEARCH_SNAPSHOT_DIR,
    RESEARCH_UNIVERSE,
};
use chronosentiment_adapter::decision_support::csp006_snapshot::load_required_yahoo_cache;
use chronosentiment_adapter::decision_support::dataset_partition::{
    certified_research_partition, PartitionKind,
};
use chronosentiment_adapter::decision_support::decision_value_fitness::{
    score_decision_value, DevelopmentValue,
};
use chronosentiment_adapter::decision_support::decision_value_harness::ProtocolValue;
use chronosentiment_adapter::decision_support::observation_value::{
    build_observation_slice, score_genome, ObservationRow, ObservationSlice,
};
use chronosentiment_adapter::decision_support::policy_artifact::{
    first_match_action, DecisionRule, FactorPredicate,
};
use chronosentiment_adapter::decision_support::policy_discovery::{
    evolve_on_development, evolve_on_development_observed, select_on_selection,
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

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

fn profile(
    ma20: f64,
    ma50: f64,
    roc: f64,
    atr: Option<f64>,
) -> chronosentiment_adapter::reasoning::assessment::AssessmentProfile {
    let t = Utc.with_ymd_and_hms(2021, 10, 31, 15, 30, 0).unwrap();
    let mut metrics = MetricReport::default();
    metrics
        .metrics
        .insert("ma_20".to_string(), MetricValue::Float(ma20));
    metrics
        .metrics
        .insert("ma_50".to_string(), MetricValue::Float(ma50));
    metrics
        .metrics
        .insert("roc_20".to_string(), MetricValue::Float(roc));
    if let Some(v) = atr {
        metrics
            .metrics
            .insert("atr_14".to_string(), MetricValue::Float(v));
    }
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
            profile: profile(110.0, 100.0, 0.05, Some(1.2)),
            instrument_return: Some(instrument_return),
        })
        .collect();
    ObservationSlice { kind, rows }
}

fn always(action: DecisionAction) -> RuleListGenome {
    RuleListGenome {
        rules: vec![],
        unmatched_action: action,
    }
}

fn slot(identity: &str, action: DecisionAction) -> SerializedGenome {
    SerializedGenome {
        identity: identity.to_string(),
        development_fitness: 0.0,
        rules: vec![],
        unmatched_action: action,
    }
}

fn living_archive() -> SearchArchive {
    SearchArchive {
        contract_id: "csp006c2.search_observability.1".to_string(),
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
                generation_best: slot("A", DecisionAction::Long),
                near_best: vec![],
                living_slots: vec![
                    slot("A", DecisionAction::Long),
                    slot("B", DecisionAction::Short),
                ],
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
                generation_best: slot("A", DecisionAction::Long),
                near_best: vec![],
                living_slots: vec![
                    slot("A", DecisionAction::Long),
                    slot("C", DecisionAction::NoTrade),
                ],
            },
        ],
        offspring: vec![OffspringEdge {
            generation: 1,
            parent_a_identity: "A".into(),
            parent_b_identity: "B".into(),
            child_identity: "D-never-entered".into(),
        }],
        selected_instruments: None,
    }
}

#[test]
fn search_two_run_stays_unauthorized() {
    assert!(!SEARCH_TWO_RUN_AUTHORIZED);
    assert!(!search_two_run_is_authorized());
    assert!(
        evolve_on_development_value(synthetic_slice(PartitionKind::Development, 0.01)).is_err()
    );
    assert_eq!(verify_implementation_contract().result, "PASS");
    assert!(post_seal_symbol_matrices_required());
    identity_lineage_holds().unwrap();
}

#[test]
fn search_one_evidence_is_byte_immutable() {
    let search_dir = workspace_root().join(RESEARCH_DISCOVERY_DIR);
    if !search_dir.join("selected_policy.json").exists() {
        return;
    }
    search_one_evidence_is_immutable(&search_dir).unwrap();
}

#[test]
fn m1_fitness_is_long_r_short_neg_r_no_trade_zero() {
    let long = score_decision_value(
        &always(DecisionAction::Long),
        &synthetic_slice(PartitionKind::Development, 0.04),
    )
    .unwrap();
    let short = score_decision_value(
        &always(DecisionAction::Short),
        &synthetic_slice(PartitionKind::Development, 0.04),
    )
    .unwrap();
    let aside = score_decision_value(
        &always(DecisionAction::NoTrade),
        &synthetic_slice(PartitionKind::Development, 0.04),
    )
    .unwrap();
    assert!((long.fitness - 0.04).abs() < 1e-15);
    assert!((short.fitness + 0.04).abs() < 1e-15);
    assert_eq!(aside.fitness, 0.0);
    assert_eq!(aside.n_stood_aside, 7);
}

#[test]
fn fitness_preserves_magnitude_instead_of_labels() {
    let long = always(DecisionAction::Long);
    let short = always(DecisionAction::Short);
    let a =
        score_decision_value(&long, &synthetic_slice(PartitionKind::Development, 0.001)).unwrap();
    let b =
        score_decision_value(&long, &synthetic_slice(PartitionKind::Development, 0.005)).unwrap();
    let c =
        score_decision_value(&long, &synthetic_slice(PartitionKind::Development, 0.05)).unwrap();
    assert!(a.fitness < b.fitness && b.fitness < c.fitness);
    assert!((a.fitness - 0.001).abs() < 1e-15);
    assert!((b.fitness - 0.005).abs() < 1e-15);
    assert!((c.fitness - 0.05).abs() < 1e-15);

    let na =
        score_decision_value(&short, &synthetic_slice(PartitionKind::Development, 0.001)).unwrap();
    let nb =
        score_decision_value(&short, &synthetic_slice(PartitionKind::Development, 0.005)).unwrap();
    let nc =
        score_decision_value(&short, &synthetic_slice(PartitionKind::Development, 0.05)).unwrap();
    assert!(nc.fitness < nb.fitness && nb.fitness < na.fitness);
    assert!((na.fitness + 0.001).abs() < 1e-15);
    assert!((nb.fitness + 0.005).abs() < 1e-15);
    assert!((nc.fitness + 0.05).abs() < 1e-15);
}

#[test]
fn no_trade_enters_the_instrument_mean_as_zero() {
    let t0 = Utc.with_ymd_and_hms(2021, 10, 31, 15, 30, 0).unwrap();
    let t1 = Utc.with_ymd_and_hms(2021, 11, 30, 15, 30, 0).unwrap();
    let genome = RuleListGenome {
        rules: vec![DecisionRule {
            when: vec![FactorPredicate {
                concept: "Trend".into(),
                present: Some(true),
                direction: Some("Bearish".into()),
            }],
            action: DecisionAction::Long,
        }],
        unmatched_action: DecisionAction::NoTrade,
    };
    let traded_profile = profile(90.0, 100.0, -0.02, Some(1.2));
    let aside_profile = profile(110.0, 100.0, 0.05, Some(1.2));
    assert_eq!(
        first_match_action(&genome.rules, genome.unmatched_action, &traded_profile),
        DecisionAction::Long
    );
    assert_eq!(
        first_match_action(&genome.rules, genome.unmatched_action, &aside_profile),
        DecisionAction::NoTrade
    );
    let rows = RESEARCH_UNIVERSE
        .iter()
        .flat_map(|ticker| {
            [
                ObservationRow {
                    instrument: (*ticker).to_string(),
                    as_of: t0,
                    profile: traded_profile.clone(),
                    instrument_return: Some(0.10),
                },
                ObservationRow {
                    instrument: (*ticker).to_string(),
                    as_of: t1,
                    profile: aside_profile.clone(),
                    instrument_return: Some(0.10),
                },
            ]
        })
        .collect();
    let slice = ObservationSlice {
        kind: PartitionKind::Development,
        rows,
    };
    let m1 = score_decision_value(&genome, &slice).unwrap();
    let search_one = score_genome(&genome, &slice).unwrap();
    assert!((m1.fitness - 0.05).abs() < 1e-12);
    assert!((search_one.fitness - 0.10).abs() < 1e-12);
}

#[test]
fn empty_instrument_is_a_protocol_error_not_silent_zero() {
    let t = Utc.with_ymd_and_hms(2021, 10, 31, 15, 30, 0).unwrap();
    let rows = RESEARCH_UNIVERSE
        .iter()
        .filter(|ticker| **ticker != "IDEA.NS")
        .map(|ticker| ObservationRow {
            instrument: (*ticker).to_string(),
            as_of: t,
            profile: profile(110.0, 100.0, 0.05, Some(1.2)),
            instrument_return: Some(0.01),
        })
        .collect();
    let slice = ObservationSlice {
        kind: PartitionKind::Development,
        rows,
    };
    assert!(score_decision_value(&always(DecisionAction::Long), &slice).is_err());
}

#[test]
fn evaluation_cannot_reach_decision_value_fitness() {
    let genome = always(DecisionAction::Long);
    assert!(
        score_decision_value(&genome, &synthetic_slice(PartitionKind::Evaluation, 0.01)).is_err()
    );
    assert!(DevelopmentValue::new(synthetic_slice(PartitionKind::Evaluation, 0.01)).is_err());
    assert!(DevelopmentValue::new(synthetic_slice(PartitionKind::Selection, 0.01)).is_err());
    assert!(DevelopmentValue::new(synthetic_slice(PartitionKind::Development, 0.01)).is_ok());
}

#[test]
fn regret_unique_best_and_accuracy_cannot_construct_fitness() {
    let fitness = include_str!("../src/decision_support/decision_value_fitness.rs");
    assert!(fitness.contains("ProtocolValue::from_per_instrument_v"));
    assert!(!fitness.contains("from_regret"));
    assert!(!fitness.contains("from_unique_best"));
    assert!(!fitness.contains("accuracy"));
    assert!(!fitness.contains("advantage_vs"));
    assert!(!fitness.contains("GOOD"));
    let mut per_instrument = BTreeMap::new();
    for ticker in RESEARCH_UNIVERSE {
        per_instrument.insert((*ticker).to_string(), vec![0.01]);
    }
    let _ = ProtocolValue::from_per_instrument_v(&per_instrument).unwrap();
}

#[test]
fn representation_can_emit_all_three_actions_without_a_threshold() {
    let genome = RuleListGenome {
        rules: vec![
            DecisionRule {
                when: vec![FactorPredicate {
                    concept: "Trend".into(),
                    present: Some(true),
                    direction: Some("Bearish".into()),
                }],
                action: DecisionAction::Long,
            },
            DecisionRule {
                when: vec![FactorPredicate {
                    concept: "Trend".into(),
                    present: Some(true),
                    direction: Some("Bullish".into()),
                }],
                action: DecisionAction::Short,
            },
        ],
        unmatched_action: DecisionAction::NoTrade,
    };
    let long_state = first_match_action(
        &genome.rules,
        genome.unmatched_action,
        &profile(90.0, 100.0, -0.02, Some(1.2)),
    );
    let short_state = first_match_action(
        &genome.rules,
        genome.unmatched_action,
        &profile(110.0, 100.0, 0.05, Some(1.2)),
    );
    let t = Utc.with_ymd_and_hms(2021, 10, 31, 15, 30, 0).unwrap();
    let mut metrics = MetricReport::default();
    metrics
        .metrics
        .insert("roc_20".to_string(), MetricValue::Float(0.0));
    let trend_absent = AssessmentEngine.assess_at(
        &metrics,
        &[Concept::Trend, Concept::Momentum, Concept::Volatility],
        t,
        Some(Uuid::from_u128(7)),
    );
    let aside_state = first_match_action(&genome.rules, genome.unmatched_action, &trend_absent);
    assert_eq!(long_state, DecisionAction::Long);
    assert_eq!(short_state, DecisionAction::Short);
    assert_eq!(aside_state, DecisionAction::NoTrade);
    let small = score_decision_value(
        &always(DecisionAction::Long),
        &synthetic_slice(PartitionKind::Development, 0.001),
    )
    .unwrap();
    assert_ne!(small.fitness, 0.0);
}

#[test]
fn living_pool_is_unique_slots_not_every_offspring() {
    let pool = living_selection_pool(&living_archive()).unwrap();
    assert_eq!(pool.len(), 3);
    assert!(pool
        .iter()
        .any(|g| g.unmatched_action == DecisionAction::Long));
    assert!(pool
        .iter()
        .any(|g| g.unmatched_action == DecisionAction::Short));
    assert!(pool
        .iter()
        .any(|g| g.unmatched_action == DecisionAction::NoTrade));
    assert!(!pool.iter().any(|g| g.identity_hash() == "D-never-entered"));
    let _ = genome_from_living_slot(&slot("A", DecisionAction::Long));
    let mut empty = living_archive();
    empty.generations[0].living_slots.clear();
    assert!(living_selection_pool(&empty).is_err());
}

#[test]
fn select_on_selection_value_uses_m1_not_traded_only() {
    let development = synthetic_slice(PartitionKind::Development, 0.02);
    let selection = synthetic_slice(PartitionKind::Selection, -0.03);
    let candidates = vec![
        always(DecisionAction::Long),
        always(DecisionAction::NoTrade),
    ];
    let selected = select_on_selection_value(&candidates, &development, &selection).unwrap();
    assert_eq!(selected.genome.unmatched_action, DecisionAction::NoTrade);
    assert_eq!(selected.selection.fitness, 0.0);
}

#[test]
fn search_one_path_is_unchanged() {
    let discovery = include_str!("../src/decision_support/policy_discovery.rs");
    assert!(discovery.contains("DevelopmentFitness"));
    assert!(!discovery.contains("DevelopmentValue"));
    assert!(!discovery.contains("score_decision_value"));
    assert!(discovery.contains("mean_of_per_instrument_mean_signed_traded_returns"));
}

#[test]
fn observer_on_off_identity_and_living_slots() {
    let development = synthetic_slice(PartitionKind::Development, 0.03);
    let selection = synthetic_slice(PartitionKind::Selection, -0.01);
    let (_, off_candidates) = evolve_on_development(development.clone()).unwrap();
    let off = select_on_selection(&off_candidates, &development, &selection).unwrap();
    let (_, on_candidates, archive) = evolve_on_development_observed(development.clone()).unwrap();
    let on = select_on_selection(&on_candidates, &development, &selection).unwrap();
    assert_eq!(off.artifact.artifact_hash, on.artifact.artifact_hash);
    assert!(!archive.generations.is_empty());
    for generation in &archive.generations {
        assert_eq!(generation.living_slots.len(), generation.population_size);
    }
    let pool = living_selection_pool(&archive).unwrap();
    assert!(!pool.is_empty());
    let offspring_only: Vec<_> = archive
        .offspring
        .iter()
        .map(|e| e.child_identity.clone())
        .filter(|id| {
            !archive
                .generations
                .iter()
                .any(|g| g.living_slots.iter().any(|s| &s.identity == id))
        })
        .collect();
    let pool_ids: Vec<String> = pool.iter().map(|g| g.identity_hash()).collect();
    for id in offspring_only {
        assert!(!pool_ids.contains(&id));
    }
}

#[test]
fn certified_search_one_identity_when_cache_present() {
    let cache_dir = workspace_root()
        .join(RESEARCH_SNAPSHOT_DIR)
        .join("yahoo_cache");
    if !cache_dir.join("HDFCBANK.NS.json").exists() {
        return;
    }
    let cache = load_required_yahoo_cache(&cache_dir).unwrap();
    let partition = certified_research_partition();
    let development = build_observation_slice(
        &cache,
        &partition.development.timestamps,
        PartitionKind::Development,
    )
    .unwrap();
    let selection = build_observation_slice(
        &cache,
        &partition.selection.timestamps,
        PartitionKind::Selection,
    )
    .unwrap();
    let (_, off_c) = evolve_on_development(development.clone()).unwrap();
    let off = select_on_selection(&off_c, &development, &selection).unwrap();
    let (_, on_c, archive) = evolve_on_development_observed(development.clone()).unwrap();
    let on = select_on_selection(&on_c, &development, &selection).unwrap();
    assert_eq!(off.artifact.artifact_hash, on.artifact.artifact_hash);
    assert_eq!(off.artifact.artifact_hash, RESEARCH_DISCOVERY_ARTIFACT_HASH);
    assert_eq!(
        archive.generations[0].living_slots.len(),
        archive.generations[0].population_size
    );
}

#[test]
fn domain_names_stay_semantic() {
    let files = [
        include_str!("../src/decision_support/decision_value_fitness.rs"),
        include_str!("../src/decision_support/c3_implementation.rs"),
        include_str!("../src/bin/csp006_c3_implementation.rs"),
    ];
    for src in files {
        assert!(!src.contains("train_fitness"));
        assert!(!src.contains("validation_candidates"));
        assert!(!src.contains("test_score"));
        assert!(!src.contains("phase_c3_population"));
        assert!(!src.contains("train_policy"));
        assert!(!src.contains("CoralysPhase"));
        assert!(!src.contains("b5_strategy"));
    }
}

#[test]
fn document_authorizes_implementation_not_execution() {
    let doc = include_str!("../../../docs/CS-P-006-C.3-I_IMPLEMENTATION.md");
    assert!(doc.contains(RESEARCH_DISCOVERY_ARTIFACT_HASH));
    assert!(doc.contains("Search #2 not run") || doc.contains("Search #2 was not run"));
    assert!(doc.contains("living-population"));
    assert!(doc.contains("AUTHORIZE RUN") || doc.contains("not authorize a run"));
    assert!(doc.contains("development value"));
}
