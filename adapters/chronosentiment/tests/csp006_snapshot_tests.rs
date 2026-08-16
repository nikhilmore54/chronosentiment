//! CS-P-006 7-instrument snapshot fidelity tests.
//!
//! Not B5. Does not freeze TRAIN/VAL/TEST dates.

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use chrono::{TimeZone, Utc};
use chronosentiment_adapter::decision_support::csp006_protocol::{
    CERTIFIED_FIVE_INSTRUMENT_SNAPSHOT, RESEARCH_SNAPSHOT_DIR, RESEARCH_SNAPSHOT_IDENTITY_HASH,
    RESEARCH_UNIVERSE,
};
use chronosentiment_adapter::decision_support::csp006_snapshot::{
    build_research_snapshot, certify_research_snapshot, parse_enrichment_identity_file,
    repeated_identity_matches,
};
use chronosentiment_adapter::decision_support::enrichment_certify::{
    load_yahoo_cache_dir, replay_month_ends_2021_10_to_2024_12,
};
use chronosentiment_adapter::ingestion::yahoo::YahooHistoricalBar;

fn synthetic_bars() -> Vec<YahooHistoricalBar> {
    let start = Utc.with_ymd_and_hms(2020, 1, 2, 10, 0, 0).unwrap().timestamp();
    (0..800)
        .map(|i| {
            let close = 100.0 + (i as f64) * 0.05;
            YahooHistoricalBar {
                timestamp: start + i * 86400,
                open: close,
                high: close + 1.0,
                low: close - 1.0,
                close,
                adj_close: close,
                volume: 1_000_000.0,
            }
        })
        .collect()
}

fn seven_cache() -> BTreeMap<String, Vec<YahooHistoricalBar>> {
    let bars = synthetic_bars();
    RESEARCH_UNIVERSE
        .iter()
        .map(|t| ((*t).to_string(), bars.clone()))
        .collect()
}

#[test]
fn replay_grid_is_39_month_ends() {
    let grid = replay_month_ends_2021_10_to_2024_12();
    assert_eq!(grid.len(), 39);
    assert_eq!(
        grid[0],
        Utc.with_ymd_and_hms(2021, 10, 31, 15, 30, 0).unwrap()
    );
    assert_eq!(
        *grid.last().unwrap(),
        Utc.with_ymd_and_hms(2024, 12, 31, 15, 30, 0).unwrap()
    );
}

#[test]
fn seven_instrument_snapshot_is_balanced_and_repeatable() {
    let cache = seven_cache();
    let a = build_research_snapshot(&cache).expect("build");
    let b = build_research_snapshot(&cache).expect("build again");
    assert_eq!(a.n_rows, 7 * 39);
    assert_eq!(a.instruments.len(), 7);
    assert!(a.instruments.contains(&"IDEA.NS".to_string()));
    assert!(a.instruments.contains(&"MAHABANK.NS".to_string()));
    assert!(repeated_identity_matches(&a, &b));
    assert_eq!(a.identity_hash, b.identity_hash);
    let cert = certify_research_snapshot(&a, &cache, None);
    assert_eq!(cert.duplicate_instrument_t, 0);
    assert_eq!(cert.tmv_incomplete_rows, 0);
    assert_eq!(cert.tmv_complete_rows, 273);
    assert_eq!(cert.result, "PASS");
    assert_eq!(cert.discovery_ready, "READY");
    assert_eq!(a.rows[0].as_of, a.rows[0].evaluation_timestamp);
}

#[test]
fn chronological_coverage_is_seven_by_thirty_nine_month_ends() {
    let snapshot = build_research_snapshot(&seven_cache()).unwrap();
    let grid = replay_month_ends_2021_10_to_2024_12();
    let mut seen = BTreeSet::new();
    for row in &snapshot.rows {
        assert!(grid.contains(&row.as_of), "{} as_of {} not on replay grid", row.instrument, row.as_of);
        assert!(seen.insert((row.instrument.clone(), row.as_of)));
    }
    assert_eq!(seen.len(), 7 * 39);
    for ticker in RESEARCH_UNIVERSE {
        let n = snapshot.rows.iter().filter(|r| r.instrument == *ticker).count();
        assert_eq!(n, 39, "{ticker} missing chronological coverage");
    }
}

#[test]
fn future_bars_are_not_consumed_during_state_construction() {
    let cache = seven_cache();
    let mut with_future = cache.clone();
    let future_ts = Utc.with_ymd_and_hms(2026, 6, 1, 10, 0, 0).unwrap().timestamp();
    for bars in with_future.values_mut() {
        let last = bars.last().cloned().expect("bars");
        bars.push(YahooHistoricalBar {
            timestamp: future_ts,
            open: last.close * 4.0,
            high: last.close * 4.0,
            low: last.close * 4.0,
            close: last.close * 4.0,
            adj_close: last.close * 4.0,
            volume: 9_999_999.0,
        });
    }
    let a = build_research_snapshot(&cache).unwrap();
    let b = build_research_snapshot(&with_future).unwrap();
    assert!(
        repeated_identity_matches(&a, &b),
        "bars after T must not change factor signatures or decision identities"
    );
}

#[test]
fn every_row_records_tmv_availability_and_assessment_decision_lineage() {
    let snapshot = build_research_snapshot(&seven_cache()).unwrap();
    for row in &snapshot.rows {
        assert!(row.trend_available, "{} {} missing Trend", row.instrument, row.as_of);
        assert!(row.momentum_available, "{} {} missing Momentum", row.instrument, row.as_of);
        assert!(row.volatility_available, "{} {} missing Volatility", row.instrument, row.as_of);
        assert_eq!(row.as_of, row.evaluation_timestamp);
        assert_ne!(row.assessment_id, uuid::Uuid::nil());
        assert_ne!(row.decision_id, uuid::Uuid::nil());
        assert_ne!(row.assessment_id, row.decision_id);
    }
}

#[test]
fn five_instrument_signatures_match_csp004_when_certified_cache_is_present() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();
    let five_cache = root.join("product_validation/assessment_enrichment_v0.1/yahoo_cache");
    let identity_path = root.join(
        "product_validation/assessment_enrichment_v0.1/provenance/identity_run1.txt",
    );
    if !five_cache.join("HDFCBANK.NS.json").exists() || !identity_path.exists() {
        return;
    }
    let mut cache = load_yahoo_cache_dir(&five_cache).expect("load CS-P-004-E1-S1 yahoo cache");
    if CERTIFIED_FIVE_INSTRUMENT_SNAPSHOT
        .iter()
        .any(|t| !cache.contains_key(*t))
    {
        panic!("CS-P-004-E1-S1 yahoo cache is missing one of the certified five");
    }
    let synthetic = synthetic_bars();
    cache.insert("IDEA.NS".to_string(), synthetic.clone());
    cache.insert("MAHABANK.NS".to_string(), synthetic);
    let snapshot = build_research_snapshot(&cache).expect("build mixed cache");
    let prior = parse_enrichment_identity_file(&std::fs::read_to_string(identity_path).unwrap())
        .expect("parse CS-P-004-E1-S1 identity");
    let cert = certify_research_snapshot(&snapshot, &cache, Some(&prior));
    assert_eq!(
        cert.five_instrument_signature_mismatches, 0,
        "existing five instruments must remain reproducible vs CS-P-004-E1-S1: {:?}",
        cert.failures
            .iter()
            .filter(|f| f.contains("CS-P-004-E1-S1"))
            .take(8)
            .collect::<Vec<_>>()
    );
}

#[test]
fn evaluation_timestamp_equals_t_and_lineage_exists() {
    let snapshot = build_research_snapshot(&seven_cache()).unwrap();
    for row in &snapshot.rows {
        assert_eq!(row.as_of, row.evaluation_timestamp);
        assert_ne!(row.assessment_id, uuid::Uuid::nil());
        assert_ne!(row.decision_id, uuid::Uuid::nil());
    }
}

#[test]
fn five_instrument_identity_parser_accepts_enrichment_format() {
    let sample = "HDFCBANK.NS\t2021-10-31 21:00:00+05:30\tabc\n";
    let parsed = parse_enrichment_identity_file(sample).unwrap();
    let t = Utc.with_ymd_and_hms(2021, 10, 31, 15, 30, 0).unwrap();
    assert_eq!(parsed.get(&("HDFCBANK.NS".to_string(), t)).unwrap(), "abc");
}

#[test]
fn certified_on_disk_snapshot_matches_frozen_identity_when_present() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();
    let cert_path = root.join(RESEARCH_SNAPSHOT_DIR).join("certification.json");
    if !cert_path.exists() {
        return;
    }
    let cert: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(cert_path).unwrap()).unwrap();
    assert_eq!(cert["result"], "PASS");
    assert_eq!(cert["discovery_ready"], "READY");
    assert_eq!(cert["n_rows"], 273);
    assert_eq!(cert["tmv_incomplete_rows"], 0);
    assert_eq!(cert["five_instrument_signature_mismatches"], 0);
    assert_eq!(
        cert["identity_hash"].as_str().unwrap(),
        RESEARCH_SNAPSHOT_IDENTITY_HASH
    );
}

#[test]
fn snapshot_module_does_not_consume_outcomes() {
    let src = include_str!("../src/decision_support/csp006_snapshot.rs");
    assert!(!src.contains("knowledge_outcomes"));
    assert!(!src.contains("OutcomeReport"));
    assert!(!src.contains("measure_performance"));
    assert!(!src.contains("chrono_b3_test"));
    assert!(!src.contains("chrono_b4_test"));
    assert!(src.contains("Not B5") || src.contains("Not B4. Not B5"));
}
