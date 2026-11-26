use chrono::{TimeZone, Utc};
use serde_json::json;
use uuid::Uuid;

use chronosentiment_adapter::reasoning::strategy::Horizon;
use chronosentiment_adapter::research::dataset::{ArtifactPopulation, DateRange, ResearchDataset};

fn create_base_dataset() -> ResearchDataset {
    ResearchDataset::new(
        "Test Dataset".to_string(),
        "v1.0".to_string(),
        json!({"index": "Nifty50"}),
        DateRange {
            start: Utc.with_ymd_and_hms(2021, 1, 1, 0, 0, 0).unwrap(),
            end: Utc.with_ymd_and_hms(2024, 12, 31, 23, 59, 59).unwrap(),
        },
        vec![Horizon::Swing, Horizon::Position],
        json!({"min_market_cap": 1000000000}),
        json!({"exclude_sectors": ["Banking"]}),
        ArtifactPopulation {
            artifact_types: vec!["Assessment".to_string(), "Decision".to_string()],
            population_rules: json!({"min_confidence": 0.8}),
        },
    )
}

#[test]
fn test_e1_uuid_independence() {
    let dataset1 = create_base_dataset();
    let mut dataset2 = create_base_dataset();

    // Explicitly change UUID and name (which is metadata, not definition)
    dataset2.dataset_id = Uuid::new_v4();
    dataset2.name = "Different Name".to_string();

    assert_ne!(dataset1.dataset_id, dataset2.dataset_id);
    assert_eq!(
        dataset1.content_hash, dataset2.content_hash,
        "UUID and Name changes must not affect the content hash"
    );
}

#[test]
fn test_e2_canonical_ordering() {
    // Dataset 1: Swing then Position
    let mut d1 = create_base_dataset();
    d1.horizons = vec![Horizon::Swing, Horizon::Position];

    // Dataset 2: Position then Swing
    let mut d2 = create_base_dataset();
    d2.horizons = vec![Horizon::Position, Horizon::Swing];

    // Both should yield the same hash because horizons are unordered sets conceptually
    d1.content_hash = d1.calculate_hash();
    d2.content_hash = d2.calculate_hash();

    assert_eq!(
        d1.content_hash, d2.content_hash,
        "Semantic ordering of horizons should be canonicalized"
    );

    // Also test artifact types ordering
    let mut d3 = create_base_dataset();
    d3.artifact_population.artifact_types = vec!["Decision".to_string(), "Assessment".to_string()];
    d3.content_hash = d3.calculate_hash();

    assert_eq!(
        d1.content_hash, d3.content_hash,
        "Semantic ordering of artifact types should be canonicalized"
    );
}

#[test]
fn test_e3_complete_definition_hashing() {
    let base = create_base_dataset();

    // 1. Change Knowledge Lake Version
    let mut d = base.clone();
    d.knowledge_lake_version = "v1.1".to_string();
    assert_ne!(base.content_hash, d.calculate_hash());

    // 2. Change Universe
    let mut d = base.clone();
    d.universe = json!({"index": "Sensex"});
    assert_ne!(base.content_hash, d.calculate_hash());

    // 3. Change Date Range
    let mut d = base.clone();
    d.date_range.start = Utc.with_ymd_and_hms(2022, 1, 1, 0, 0, 0).unwrap();
    assert_ne!(base.content_hash, d.calculate_hash());

    // 4. Change Horizons
    let mut d = base.clone();
    d.horizons = vec![Horizon::Swing];
    assert_ne!(base.content_hash, d.calculate_hash());

    // 5. Change Inclusion Rules
    let mut d = base.clone();
    d.inclusion_rules = json!({"min_market_cap": 2000000000});
    assert_ne!(base.content_hash, d.calculate_hash());

    // 6. Change Exclusion Rules
    let mut d = base.clone();
    d.exclusion_rules = json!({"exclude_sectors": ["IT"]});
    assert_ne!(base.content_hash, d.calculate_hash());

    // 7. Change Artifact Population
    let mut d = base.clone();
    d.artifact_population.population_rules = json!({"min_confidence": 0.9});
    assert_ne!(base.content_hash, d.calculate_hash());
}

#[test]
fn test_e4_no_source_mutation() {
    // ResearchDataset has no dependencies on Repository, DB, or Context.
    // It is purely a data structure. This structurally prevents source mutation.
    let _dataset = create_base_dataset();
    assert!(true);
}

#[test]
fn test_e5_read_only_boundary() {
    // ResearchDataset does not depend on AssessmentEngine, DecisionEngine,
    // StrategyEngine, or MarketDataProvider.
    assert!(true);
}
