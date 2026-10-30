//! RP-610 crypto multi-horizon tests (parity with scripts/tests/test_rp610_crypto_multi_horizon.py).

use chrono::{Duration, TimeZone, Utc};
use chronosentiment_adapter::research::rp610_crypto_multi_horizon::{
    admit_candidate, amend_hold, build_candidate, estimate_crypto_friction_usdt,
    evaluate_open_hold, find_open_hold, open_holds_from_book, write_daily_observation, AmendArgs,
    Rp610Store,
};
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

fn tmp_store() -> (PathBuf, Rp610Store) {
    let root = std::env::temp_dir().join(format!("rp610_test_{}", Uuid::new_v4()));
    let journals = root.join("journals");
    fs::create_dir_all(&journals).unwrap();
    (root, Rp610Store::new(journals))
}

#[test]
fn test_friction_positive() {
    let fr = estimate_crypto_friction_usdt(100.0, 110.0, 1.0, 12.0);
    assert!(fr.total_friction_usdt > 0.0);
    assert_eq!(fr.round_trip_friction_bps, 12.0);
}

#[test]
fn test_h0_reject() {
    let err = build_candidate(
        "BTCUSDT",
        "LONG",
        68000.0,
        Utc.with_ymd_and_hms(2026, 10, 1, 0, 0, 0).unwrap(),
        "H0_INTRADAY_60M",
        "",
        "",
        1000.0,
        None,
        None,
    );
    assert!(err.is_err());
}

#[test]
fn test_h1_target_exit() {
    let (root, store) = tmp_store();
    let c = build_candidate(
        "BTCUSDT",
        "LONG",
        100.0,
        Utc.with_ymd_and_hms(2026, 10, 1, 0, 0, 0).unwrap(),
        "H1_MULTI_DAY",
        "",
        "",
        1000.0,
        Some(0.05),
        Some(0.10),
    )
    .unwrap();
    admit_candidate(&store, &c).unwrap();
    let closed = evaluate_open_hold(
        &c,
        Utc.with_ymd_and_hms(2026, 10, 3, 0, 0, 0).unwrap(),
        112.0,
        false,
        "",
    )
    .unwrap();
    assert_eq!(closed["exit_reason"], "TARGET");
    assert!(closed["net_pnl_usdt"].as_f64().unwrap() > 0.0);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn test_h1_invalidation_and_max_hold() {
    let (root, _store) = tmp_store();
    let c = build_candidate(
        "BTCUSDT",
        "LONG",
        100.0,
        Utc.with_ymd_and_hms(2026, 10, 1, 0, 0, 0).unwrap(),
        "H1_MULTI_DAY",
        "",
        "",
        1000.0,
        Some(0.05),
        Some(0.50),
    )
    .unwrap();
    let stopped = evaluate_open_hold(
        &c,
        Utc.with_ymd_and_hms(2026, 10, 2, 0, 0, 0).unwrap(),
        94.0,
        false,
        "",
    )
    .unwrap();
    assert_eq!(stopped["exit_reason"], "INVALIDATION");

    let c2 = build_candidate(
        "BTCUSDT",
        "LONG",
        100.0,
        Utc.with_ymd_and_hms(2026, 10, 1, 0, 0, 0).unwrap(),
        "H1_MULTI_DAY",
        "",
        "",
        1000.0,
        Some(0.50),
        Some(0.50),
    )
    .unwrap();
    let maxed = evaluate_open_hold(
        &c2,
        Utc.with_ymd_and_hms(2026, 10, 1, 0, 0, 0).unwrap() + Duration::days(14),
        101.0,
        false,
        "",
    )
    .unwrap();
    assert_eq!(maxed["exit_reason"], "MAX_HOLD_DAYS");
    let _ = fs::remove_dir_all(root);
}

#[test]
fn test_h2_defaults_and_eod() {
    let (root, store) = tmp_store();
    let c = build_candidate(
        "BTCUSDT",
        "LONG",
        68000.0,
        Utc.with_ymd_and_hms(2026, 10, 1, 0, 0, 0).unwrap(),
        "H2_MULTI_WEEK",
        "",
        "",
        1000.0,
        None,
        None,
    )
    .unwrap();
    assert_eq!(c["hold_days_min"], 14);
    assert_eq!(c["hold_days_max"], 56);
    admit_candidate(&store, &c).unwrap();
    assert_eq!(open_holds_from_book(&store, "H2_MULTI_WEEK").unwrap().len(), 1);
    let mut marks = BTreeMap::new();
    marks.insert("BTCUSDT".into(), 68500.0);
    let summary = write_daily_observation(
        &store,
        Utc.with_ymd_and_hms(2026, 10, 2, 0, 0, 0).unwrap(),
        &marks,
        "",
    )
    .unwrap();
    assert_eq!(summary["live_order_authority"], 0.0);
    assert_eq!(summary["decision_status"], "OBSERVE_ONLY_NO_CONCLUSION");
    assert_eq!(summary["open_counts"]["H2_MULTI_WEEK"], 1);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn test_h3_no_max_hold_exit() {
    let c = build_candidate(
        "BTCUSDT",
        "LONG",
        68000.0,
        Utc.with_ymd_and_hms(2026, 10, 1, 0, 0, 0).unwrap(),
        "H3_LONG_TERM",
        "",
        "",
        1000.0,
        None,
        None,
    )
    .unwrap();
    assert!(c["hold_days_max"].is_null());
    assert!(c["max_exit_ts"].is_null());
    let still = evaluate_open_hold(
        &c,
        Utc.with_ymd_and_hms(2027, 10, 1, 0, 0, 0).unwrap(),
        70000.0,
        false,
        "",
    );
    assert!(still.is_none());
}

#[test]
fn test_registry_gates_closed() {
    let (root, store) = tmp_store();
    let path = store.write_horizon_registry().unwrap();
    let data = fs::read_to_string(path).unwrap();
    assert!(data.contains("G-LIVE"));
    assert!(data.contains("CLOSED"));
    assert!(data.contains("H1_MULTI_DAY"));
    let _ = fs::remove_dir_all(root);
}

#[test]
fn test_amend_target_then_exit() {
    let (root, store) = tmp_store();
    let c = build_candidate(
        "BTCUSDT",
        "LONG",
        100.0,
        Utc.with_ymd_and_hms(2026, 10, 1, 0, 0, 0).unwrap(),
        "H1_MULTI_DAY",
        "",
        "",
        1000.0,
        Some(0.05),
        Some(0.20),
    )
    .unwrap();
    admit_candidate(&store, &c).unwrap();
    let cid = c["candidate_id"].as_str().unwrap();
    let out = amend_hold(
        &store,
        cid,
        AmendArgs {
            reason: "raise_target",
            stop_pct: None,
            target_pct: Some(0.08),
            hold_days_max: None,
            hold_days_min: None,
            max_exit_ts: None,
            thesis: None,
            thesis_version: None,
            source_signal_id: None,
            new_horizon_id: None,
            mark_price: None,
            mark_ts: None,
        },
    )
    .unwrap();
    assert_eq!(out["amendment"]["prior"]["target_pct"], 0.20);
    assert_eq!(out["effective"]["target_pct"], 0.08);
    let mut marks = BTreeMap::new();
    marks.insert("BTCUSDT".into(), 105.0);
    write_daily_observation(
        &store,
        Utc.with_ymd_and_hms(2026, 10, 2, 0, 0, 0).unwrap(),
        &marks,
        "",
    )
    .unwrap();
    let hold = find_open_hold(&store, cid).unwrap().unwrap();
    assert_eq!(hold["target_pct"], 0.08);
    assert_eq!(hold["entry_price"], 100.0);
    let closed = evaluate_open_hold(
        &hold,
        Utc.with_ymd_and_hms(2026, 10, 3, 0, 0, 0).unwrap(),
        109.0,
        false,
        "",
    )
    .unwrap();
    assert_eq!(closed["exit_reason"], "TARGET");
    let _ = fs::remove_dir_all(root);
}

#[test]
fn test_horizon_roll_preserves_entry() {
    let (root, store) = tmp_store();
    let c = build_candidate(
        "BTCUSDT",
        "LONG",
        68000.0,
        Utc.with_ymd_and_hms(2026, 10, 1, 0, 0, 0).unwrap(),
        "H1_MULTI_DAY",
        "",
        "",
        1000.0,
        None,
        None,
    )
    .unwrap();
    admit_candidate(&store, &c).unwrap();
    let cid = c["candidate_id"].as_str().unwrap();
    let out = amend_hold(
        &store,
        cid,
        AmendArgs {
            reason: "extend_to_multi_week",
            stop_pct: None,
            target_pct: None,
            hold_days_max: None,
            hold_days_min: None,
            max_exit_ts: None,
            thesis: None,
            thesis_version: None,
            source_signal_id: None,
            new_horizon_id: Some("H2_MULTI_WEEK"),
            mark_price: Some(68500.0),
            mark_ts: Some(Utc.with_ymd_and_hms(2026, 10, 5, 0, 0, 0).unwrap()),
        },
    )
    .unwrap();
    assert_eq!(out["supersede"]["status"], "SUPERSEDED");
    let child = &out["child"];
    assert_eq!(child["parent_candidate_id"], cid);
    assert_eq!(child["entry_price"], 68000.0);
    assert_eq!(child["horizon_id"], "H2_MULTI_WEEK");
    assert!(find_open_hold(&store, cid).unwrap().is_none());
    assert!(find_open_hold(&store, child["candidate_id"].as_str().unwrap())
        .unwrap()
        .is_some());
    assert_eq!(open_holds_from_book(&store, "H1_MULTI_DAY").unwrap().len(), 0);
    assert_eq!(open_holds_from_book(&store, "H2_MULTI_WEEK").unwrap().len(), 1);
    let _ = fs::remove_dir_all(root);
}
