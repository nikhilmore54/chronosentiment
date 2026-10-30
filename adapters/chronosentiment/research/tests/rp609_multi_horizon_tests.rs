//! RP-609 multi-horizon engine tests (parity with scripts/tests/test_rp609_multi_horizon_engine.py).

use chronosentiment_adapter::research::rp609_multi_horizon::{
    estimate_cnc_friction_inr, MultiHorizonEngine,
};
use serde_json::json;
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

fn tmp_engine() -> (PathBuf, MultiHorizonEngine) {
    let root = std::env::temp_dir().join(format!("rp609_test_{}", Uuid::new_v4()));
    let journals = root.join("journals");
    fs::create_dir_all(&journals).unwrap();
    let nse = root.join("nse");
    fs::create_dir_all(&nse).unwrap();
    let eng = MultiHorizonEngine::new(journals, nse).expect("engine");
    (root, eng)
}

#[test]
fn test_cnc_friction_positive() {
    let fr = estimate_cnc_friction_inr(1000.0, 1010.0, 10.0, 2.5);
    assert!(fr.total_friction > 0.0);
    assert!(fr.stt > 0.0);
}

#[test]
fn test_swing_target_exit() {
    let (root, mut eng) = tmp_engine();
    let t = eng
        .register_swing_candidate(
            "AAA.NS",
            "t",
            100.0,
            90.0,
            110.0,
            10.0,
            5,
            None,
            None,
            true,
            Some("2026-10-01"),
        )
        .unwrap();
    let tid = t["trade_id"].as_str().unwrap().to_string();
    let closed = eng.mark_swing(&tid, 111.0, Some("2026-10-02")).unwrap();
    assert_eq!(closed["status"], "CLOSED");
    assert_eq!(closed["exit_reason"], "TARGET");
    assert!(closed["net_pnl_inr"].as_f64().unwrap() > 0.0);
    assert_eq!(eng.swing["open"].as_object().unwrap().len(), 0);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn test_swing_stop_and_max_sessions() {
    let (root, mut eng) = tmp_engine();
    let t = eng
        .register_swing_candidate(
            "BBB.NS",
            "t",
            100.0,
            95.0,
            120.0,
            5.0,
            2,
            None,
            None,
            true,
            Some("2026-10-01"),
        )
        .unwrap();
    let tid = t["trade_id"].as_str().unwrap().to_string();
    let stopped = eng.mark_swing(&tid, 94.0, Some("2026-10-02")).unwrap();
    assert_eq!(stopped["exit_reason"], "INVALIDATION");

    let t2 = eng
        .register_swing_candidate(
            "CCC.NS",
            "t",
            100.0,
            80.0,
            150.0,
            5.0,
            2,
            None,
            None,
            true,
            Some("2026-10-01"),
        )
        .unwrap();
    let tid2 = t2["trade_id"].as_str().unwrap().to_string();
    eng.mark_swing(&tid2, 101.0, Some("2026-10-02")).unwrap();
    let closed = eng.mark_swing(&tid2, 102.0, Some("2026-10-03")).unwrap();
    assert_eq!(closed["exit_reason"], "MAX_SESSIONS");
    assert_eq!(closed["sessions_held"], 2);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn test_portfolio_caps_and_tranches() {
    let (root, mut eng) = tmp_engine();
    let bad = eng.register_portfolio_plan(
        "bad",
        100_000.0,
        &[json!({"symbol": "X.NS", "sector": "A", "target_weight_pct": 20.0})],
        4,
        10.0,
        30.0,
        None,
    );
    assert!(bad.is_err());

    let plan = eng
        .register_portfolio_plan(
            "ok",
            100_000.0,
            &[
                json!({"symbol": "X.NS", "sector": "A", "target_weight_pct": 8.0}),
                json!({"symbol": "Y.NS", "sector": "B", "target_weight_pct": 8.0}),
            ],
            2,
            10.0,
            30.0,
            Some("2026-10-01"),
        )
        .unwrap();
    assert_eq!(plan["status"], "ACTIVE");

    let mut prices = BTreeMap::new();
    prices.insert("X.NS".into(), 100.0);
    prices.insert("Y.NS".into(), 50.0);
    eng.advance_portfolio_tranche("ok", &prices, Some("2026-10-01"))
        .unwrap();
    assert_eq!(eng.portfolio["plans"]["ok"]["tranches_filled"], 1);
    eng.advance_portfolio_tranche("ok", &prices, Some("2026-10-01"))
        .unwrap();
    assert_eq!(eng.portfolio["plans"]["ok"]["tranches_filled"], 1);
    eng.advance_portfolio_tranche("ok", &prices, Some("2026-10-02"))
        .unwrap();
    assert_eq!(eng.portfolio["plans"]["ok"]["status"], "BUILT");
    assert_eq!(eng.portfolio["plans"]["ok"]["tranches_filled"], 2);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn test_long_term_review_due() {
    let (root, mut eng) = tmp_engine();
    let h = eng
        .open_long_term(
            "ZZ.NS",
            "observe",
            "v1",
            200.0,
            3.0,
            None,
            2,
            Some("2026-10-01"),
        )
        .unwrap();
    let hid = h["hold_id"].as_str().unwrap().to_string();
    eng.mark_long_term(&hid, 205.0, Some("2026-10-02")).unwrap();
    let marked = eng.mark_long_term(&hid, 210.0, Some("2026-10-03")).unwrap();
    assert_eq!(marked["status"], "REVIEW_DUE");
    let cont = eng
        .complete_long_term_review(&hid, "HOLD_CONTINUE")
        .unwrap();
    assert_eq!(cont["status"], "HOLDING");
    let _ = fs::remove_dir_all(root);
}

#[test]
fn test_eod_uses_price_map() {
    let (root, mut eng) = tmp_engine();
    eng.register_swing_candidate(
        "EEE.NS",
        "t",
        50.0,
        40.0,
        80.0,
        10.0,
        10,
        None,
        None,
        true,
        Some("2026-10-01"),
    )
    .unwrap();
    let mut prices = BTreeMap::new();
    prices.insert("EEE.NS".into(), 55.0);
    let summary = eng.run_eod(Some("2026-10-02"), Some(&prices)).unwrap();
    assert_eq!(summary["swing_marked"], 1);
    let open_pos = eng.swing["open"].as_object().unwrap().values().next().unwrap();
    assert_eq!(open_pos["last_mark_price"], 55.0);
    assert_eq!(open_pos["sessions_held"], 1);
    let _ = fs::remove_dir_all(root);
}
