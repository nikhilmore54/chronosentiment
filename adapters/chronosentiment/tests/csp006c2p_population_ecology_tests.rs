//! CS-P-006-C.2-P — ecology analysis of Search #1. Not a new search.

use std::path::PathBuf;

use chronosentiment_adapter::decision_support::csp006_protocol::{
    RESEARCH_DISCOVERY_ARTIFACT_HASH, RESEARCH_DISCOVERY_DIR,
};
use chronosentiment_adapter::decision_support::policy_artifact::DecisionRule;
use chronosentiment_adapter::decision_support::population_ecology::{
    analyze_search_archive, DiversityBand, FamilyOccupancy, MomentumQuestion, VERDICT_EXPLORED,
    VERDICT_INDETERMINATE, VERDICT_UNDER_EXPLORED,
};
use chronosentiment_adapter::decision_support::search_observability::{
    ActionSymbolHistogram, FactorConsumptionHistogram, GenerationPopulationRecord, SearchArchive,
    SerializedGenome, OBSERVABILITY_CONTRACT_ID,
};
use chronosentiment_adapter::decision_support::DecisionAction;

fn genome(id: &str, fitness: f64, momentum: bool, short: bool) -> SerializedGenome {
    let mut rules = vec![DecisionRule {
        when: vec![chronosentiment_adapter::decision_support::policy_artifact::FactorPredicate {
            concept: "Trend".to_string(),
            present: Some(true),
            direction: Some("Bearish".to_string()),
        }],
        action: DecisionAction::Long,
    }];
    if momentum {
        rules.push(DecisionRule {
            when: vec![chronosentiment_adapter::decision_support::policy_artifact::FactorPredicate {
                concept: "Momentum".to_string(),
                present: Some(true),
                direction: Some("Positive".to_string()),
            }],
            action: DecisionAction::Long,
        });
    }
    SerializedGenome {
        identity: id.to_string(),
        development_fitness: fitness,
        rules,
        unmatched_action: if short {
            DecisionAction::Short
        } else {
            DecisionAction::NoTrade
        },
    }
}

fn generation(
    n: usize,
    unique: usize,
    best: SerializedGenome,
    trend: u32,
    momentum: u32,
    volatility: u32,
    long: u32,
    short: u32,
    no_trade: u32,
    near_best: Vec<SerializedGenome>,
) -> GenerationPopulationRecord {
    GenerationPopulationRecord {
        generation: n,
        population_size: 32,
        unique_genome_count: unique,
        best_fitness: best.development_fitness,
        median_fitness: best.development_fitness - 0.01,
        mean_fitness: best.development_fitness - 0.008,
        worst_fitness: 0.0,
        action_symbols: ActionSymbolHistogram {
            genomes_with_long: long,
            genomes_with_short: short,
            genomes_with_no_trade: no_trade,
            unmatched_long: 0,
            unmatched_short: if short > 0 { 1 } else { 0 },
            unmatched_no_trade: no_trade,
        },
        factor_consumption: FactorConsumptionHistogram {
            genomes_using_trend: trend,
            genomes_using_momentum: momentum,
            genomes_using_volatility: volatility,
        },
        generation_best: best,
        near_best,
        living_slots: Vec::new(),
    }
}

fn archive(generations: Vec<GenerationPopulationRecord>) -> SearchArchive {
    SearchArchive {
        contract_id: OBSERVABILITY_CONTRACT_ID.to_string(),
        generations,
        offspring: Vec::new(),
        selected_instruments: None,
    }
}

#[test]
fn thin_momentum_and_short_is_under_explored() {
    let winner = genome("winner", 0.02, false, false);
    let gens: Vec<_> = (0..12)
        .map(|i| {
            generation(
                i,
                2,
                winner.clone(),
                32,
                0,
                0,
                32,
                0,
                32,
                vec![winner.clone()],
            )
        })
        .collect();
    let report = analyze_search_archive(&archive(gens), Some("winner")).unwrap();
    assert_eq!(report.verdict, VERDICT_UNDER_EXPLORED);
    assert_eq!(report.momentum.occupancy, FamilyOccupancy::Absent);
    assert_eq!(report.short_action.occupancy, FamilyOccupancy::Absent);
    assert_eq!(
        report.momentum_question,
        MomentumQuestion::BarelyExplored
    );
    assert!(!report.search_two_authorized);
    assert!(!report.evaluation_scored);
}

#[test]
fn recurrent_momentum_short_and_diversity_is_explored() {
    let winner = genome("winner", 0.02, false, false);
    let alt = genome("alt", 0.02, true, true);
    let gens: Vec<_> = (0..12)
        .map(|i| {
            generation(
                i,
                16,
                winner.clone(),
                32,
                10,
                4,
                32,
                10,
                32,
                vec![winner.clone(), alt.clone()],
            )
        })
        .collect();
    let report = analyze_search_archive(&archive(gens), Some("winner")).unwrap();
    assert_eq!(report.verdict, VERDICT_EXPLORED);
    assert_eq!(report.diversity_band, DiversityBand::High);
    assert_eq!(
        report.momentum_question,
        MomentumQuestion::ExploredAndUnusedByWinner
    );
    assert!(!report.distinct_near_best_from_selected.is_empty());
}

#[test]
fn mixed_signals_are_indeterminate() {
    let winner = genome("winner", 0.02, false, false);
    let gens: Vec<_> = (0..12)
        .map(|i| {
            generation(
                i,
                16,
                winner.clone(),
                32,
                10,
                0,
                32,
                0,
                32,
                vec![winner.clone()],
            )
        })
        .collect();
    let report = analyze_search_archive(&archive(gens), Some("winner")).unwrap();
    assert_eq!(report.verdict, VERDICT_INDETERMINATE);
    assert_eq!(report.momentum.occupancy, FamilyOccupancy::Recurrent);
    assert_eq!(report.short_action.occupancy, FamilyOccupancy::Absent);
}

#[test]
fn analysis_module_does_not_search_or_score_evaluation() {
    let src = include_str!("../src/decision_support/population_ecology.rs");
    assert!(!src.contains("evolve_on_development"));
    assert!(!src.contains("evaluate_sealed_candidate"));
    assert!(!src.contains("train_policy"));
    assert!(!src.contains("test_fitness"));
    assert!(!src.contains("CoralysPhase"));
    assert!(!src.contains("b5_strategy"));
    assert!(src.contains("search_two_authorized: false"));
}

#[test]
fn ecology_binary_is_identity_gated_and_is_not_search_two() {
    let src = include_str!("../src/bin/csp006_population_ecology.rs");
    assert!(src.contains("evolve_on_development_observed"));
    assert!(src.contains("RESEARCH_DISCOVERY_ARTIFACT_HASH"));
    assert!(src.contains("refusing to analyze an artifact that is not Search #1"));
    assert!(!src.contains("evaluate_sealed_candidate"));
    assert!(!src.contains("train_policy"));
    assert!(!src.contains("CoralysPhase"));
}

#[test]
fn ecology_document_does_not_authorize_search_two() {
    let doc = include_str!("../../../docs/CS-P-006-C.2-P_POPULATION_ECOLOGY.md");
    assert!(doc.contains(RESEARCH_DISCOVERY_ARTIFACT_HASH));
    assert!(doc.contains("Search #2"));
    assert!(doc.contains("not authorized") || doc.contains("not authorize"));
    assert!(doc.contains("SEARCH-SPACE EXPLORED"));
    assert!(doc.contains("C.3"));
}

#[test]
fn on_disk_ecology_matches_search_one_and_does_not_authorize_c3() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();
    let ecology = root
        .join(RESEARCH_DISCOVERY_DIR)
        .join("ecology")
        .join("ecology.json");
    if !ecology.exists() {
        return;
    }
    let report: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(ecology).unwrap()).unwrap();
    assert_eq!(report["verdict"], VERDICT_EXPLORED);
    assert_eq!(report["search_two_authorized"], false);
    assert_eq!(report["evaluation_scored"], false);
    assert_eq!(
        report["selected_identity"],
        "d8363a93e5afe518b7a4cbb8f5c3ac59efcf396f0d318ccdae0dd683e9d730d3"
    );
}
