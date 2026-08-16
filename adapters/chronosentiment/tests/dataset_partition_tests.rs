//! Chronological dataset partition — CS-P-006-B.1 freeze.
//!
//! Domain kinds: development / selection / evaluation.
//! Does not freeze a numeric fitness formula. Does not start search.

use std::collections::BTreeSet;

use chrono::{TimeZone, Utc};
use chronosentiment_adapter::decision_support::csp006_protocol::{
    coralys_search_is_authorized, CHRONOLOGICAL_PARTITION_HASH, RESEARCH_UNIVERSE,
    RESEARCH_SNAPSHOT_CERTIFIED,
};
use chronosentiment_adapter::decision_support::dataset_partition::{
    assign_timestamp, certified_research_partition, partition_contiguous_equal_thirds,
    search_may_observe_outcomes, search_may_use_for_evolution, search_may_use_for_selection,
    timestamp_cohort_is_atomic, PartitionKind, SearchOutcomeAccess, search_outcome_access,
};
use chronosentiment_adapter::decision_support::enrichment_certify::replay_month_ends_2021_10_to_2024_12;
use chronosentiment_adapter::decision_support::policy_artifact::TrainingProvenance;

#[test]
fn certified_grid_is_thirty_nine_unique_timestamps() {
    let grid = replay_month_ends_2021_10_to_2024_12();
    let unique: BTreeSet<_> = grid.iter().copied().collect();
    assert_eq!(grid.len(), 39);
    assert_eq!(unique.len(), 39);
}

#[test]
fn equal_thirds_need_no_tie_break_on_the_certified_grid() {
    let partition = certified_research_partition();
    assert_eq!(partition.timestamps.len(), 39);
    assert_eq!(partition.development.n_timestamps, 13);
    assert_eq!(partition.selection.n_timestamps, 13);
    assert_eq!(partition.evaluation.n_timestamps, 13);
    assert_eq!(partition.development.n_observations, 91);
    assert_eq!(partition.selection.n_observations, 91);
    assert_eq!(partition.evaluation.n_observations, 91);
    assert_eq!(partition.tie_break, "none_applicable");
    assert_eq!(partition.n_instruments_per_timestamp, 7);
    assert_eq!(partition.instruments.len(), 7);
}

#[test]
fn partitions_are_strictly_chronological_and_atomic() {
    let partition = certified_research_partition();
    let last_dev = *partition.development.timestamps.last().unwrap();
    let first_sel = partition.selection.timestamps[0];
    let last_sel = *partition.selection.timestamps.last().unwrap();
    let first_eval = partition.evaluation.timestamps[0];
    assert!(last_dev < first_sel);
    assert!(last_sel < first_eval);
    assert!(partition.development.exclusive_end <= partition.selection.inclusive_start);
    assert!(partition.selection.exclusive_end <= partition.evaluation.inclusive_start);
    assert_eq!(
        last_dev,
        Utc.with_ymd_and_hms(2022, 10, 31, 15, 30, 0).unwrap()
    );
    assert_eq!(
        first_sel,
        Utc.with_ymd_and_hms(2022, 11, 30, 15, 30, 0).unwrap()
    );
    assert_eq!(
        last_sel,
        Utc.with_ymd_and_hms(2023, 11, 30, 15, 30, 0).unwrap()
    );
    assert_eq!(
        first_eval,
        Utc.with_ymd_and_hms(2023, 12, 31, 15, 30, 0).unwrap()
    );
    assert_eq!(
        *partition.evaluation.timestamps.last().unwrap(),
        Utc.with_ymd_and_hms(2024, 12, 31, 15, 30, 0).unwrap()
    );

    let mut rows = Vec::new();
    for t in &partition.timestamps {
        for ticker in RESEARCH_UNIVERSE {
            rows.push(((*ticker).to_string(), *t));
        }
    }
    timestamp_cohort_is_atomic(&rows, &partition).unwrap();
    assert_eq!(rows.len(), 273);
}

#[test]
fn every_instrument_at_t_shares_one_partition() {
    let partition = certified_research_partition();
    for t in &partition.timestamps {
        let kinds: BTreeSet<_> = RESEARCH_UNIVERSE
            .iter()
            .map(|_| assign_timestamp(&partition, *t).unwrap())
            .collect();
        assert_eq!(kinds.len(), 1);
    }
}

#[test]
fn evaluation_is_invisible_to_search() {
    assert_eq!(
        search_outcome_access(PartitionKind::Development),
        SearchOutcomeAccess::Evolution
    );
    assert_eq!(
        search_outcome_access(PartitionKind::Selection),
        SearchOutcomeAccess::SelectionFeedback
    );
    assert_eq!(
        search_outcome_access(PartitionKind::Evaluation),
        SearchOutcomeAccess::Forbidden
    );
    assert!(search_may_use_for_evolution(PartitionKind::Development));
    assert!(!search_may_use_for_evolution(PartitionKind::Selection));
    assert!(!search_may_use_for_evolution(PartitionKind::Evaluation));
    assert!(search_may_use_for_selection(PartitionKind::Selection));
    assert!(!search_may_use_for_selection(PartitionKind::Evaluation));
    assert!(!search_may_observe_outcomes(PartitionKind::Evaluation));
}

#[test]
fn remainder_timestamps_stay_in_development() {
    let ts: Vec<_> = (0..10)
        .map(|i| Utc.with_ymd_and_hms(2021, 1, 1 + i, 15, 30, 0).unwrap())
        .collect();
    let p = partition_contiguous_equal_thirds(&ts, &["A"]).unwrap();
    assert_eq!(p.development.n_timestamps, 4);
    assert_eq!(p.selection.n_timestamps, 3);
    assert_eq!(p.evaluation.n_timestamps, 3);
    assert_eq!(p.tie_break, "remainder_timestamps_assigned_to_development");
}

#[test]
fn artifact_windows_are_strictly_ordered() {
    let partition = certified_research_partition();
    let windows = TrainingProvenance::from_chronological_partition(&partition);
    let train = windows.train.unwrap();
    let val = windows.validation.unwrap();
    let test = windows.test.unwrap();
    assert!(train.inclusive_start < train.exclusive_end);
    assert!(train.exclusive_end <= val.inclusive_start);
    assert!(val.exclusive_end <= test.inclusive_start);
}

#[test]
fn domain_module_does_not_encode_protocol_phase_names() {
    let src = include_str!("../src/decision_support/dataset_partition.rs");
    assert!(!src.contains("TRAIN"));
    assert!(!src.contains("VALIDATION"));
    assert!(!src.contains("TEST"));
    assert!(!src.contains("B5"));
    assert!(!src.contains("phase_c"));
    assert!(!src.contains("g_gate"));
}

#[test]
fn partition_hash_is_deterministic() {
    let a = certified_research_partition();
    let b = certified_research_partition();
    assert_eq!(a.partition_hash, b.partition_hash);
    assert_eq!(a.partition_hash, CHRONOLOGICAL_PARTITION_HASH);
}

#[test]
fn on_disk_manifest_matches_frozen_partition_hash() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();
    let path = root.join("product_validation/CS-P-006/partition/manifest.json");
    if !path.exists() {
        return;
    }
    let v: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    assert_eq!(v["authorization"], "PASS");
    assert_eq!(v["n_timestamps"], 39);
    assert_eq!(v["development"]["n_timestamps"], 13);
    assert_eq!(v["selection"]["n_timestamps"], 13);
    assert_eq!(v["evaluation"]["n_timestamps"], 13);
    assert_eq!(
        v["partition_hash"].as_str().unwrap(),
        CHRONOLOGICAL_PARTITION_HASH
    );
    assert_eq!(v["atomic_unit"], "timestamp");
}

#[test]
fn search_authorization_requires_certified_snapshot_and_frozen_partition() {
    assert!(RESEARCH_SNAPSHOT_CERTIFIED);
    assert!(coralys_search_is_authorized());
}
