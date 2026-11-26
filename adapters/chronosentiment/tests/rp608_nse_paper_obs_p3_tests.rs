//! P3: paper-lifecycle observations vs Python — no ledger writer change.

use chronosentiment_adapter::nse_market_events::{
    close_allowed, DepthLevel, MarketEvent, OpenPaperObs, PaperLifecycleObsEngine, PaperObsAction,
    SourceProvenance, OF_PAPER_MIN_HOLD_MS, OF_PAPER_OPPOSITE_PERSIST_MS,
};
use serde_json::Value;
use std::fs;
use std::path::PathBuf;

fn fixture() -> Value {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/nse_kite_full/p3_paper_obs_differential.json");
    serde_json::from_str(&fs::read_to_string(p).expect("run scripts/rp608_nse_paper_obs_ref.py"))
        .unwrap()
}

fn mk_ev(bid_q: f64, ask_q: f64, seq: u64, fp: u64, exch_ms: u64, receipt_ms: u64) -> MarketEvent {
    MarketEvent {
        instrument_token: 6401,
        last_price: 100.0,
        volume: 1.0,
        exchange_timestamp_sec: (exch_ms / 1000) as u32,
        exchange_timestamp_utc_ms: exch_ms,
        receipt_utc_ms: receipt_ms,
        quote_age_ms: receipt_ms.saturating_sub(exch_ms) as f64,
        best_bid: 99.9,
        best_ask: 100.1,
        bid_qty: bid_q,
        ask_qty: ask_q,
        depth_buy: [DepthLevel {
            price: 99.9,
            quantity: bid_q,
        }; 5],
        depth_sell: [DepthLevel {
            price: 100.1,
            quantity: ask_q,
        }; 5],
        source: SourceProvenance::FixtureReplay,
        ingest_seq: seq,
        content_fp: fp,
    }
}

#[test]
fn p3_constants_and_close_allowed_parity() {
    let fix = fixture();
    assert_eq!(OF_PAPER_MIN_HOLD_MS, 300_000);
    assert_eq!(OF_PAPER_OPPOSITE_PERSIST_MS, 30_000);
    assert_eq!(fix["constants"]["min_hold_ms"], 300_000);
    for c in fix["close_allowed_cases"].as_array().unwrap() {
        let (ok, reason, seen) = close_allowed(
            1_700_000_000_000,
            c["seen"].as_i64(),
            c["now_ms"].as_i64().unwrap(),
            true,
            300_000,
            30_000,
        );
        assert_eq!(ok, c["ok"].as_bool().unwrap(), "{}", c["name"]);
        assert_eq!(reason, c["reason"].as_str().unwrap(), "{}", c["name"]);
        assert_eq!(seen, c["new_seen"].as_i64(), "{}", c["name"]);
    }
}

#[test]
fn p3_tick_sequence_hold_arm_persist_fire() {
    let fix = fixture();
    let act = 1_700_000_000_000_i64;
    let mut eng = PaperLifecycleObsEngine::new();
    eng.mirror_open(OpenPaperObs {
        symbol: "TEST.NS".into(),
        direction: "LONG".into(),
        of_trade_id: "NSE-OF-PAPER-TEST.NS-10-1".into(),
        activated_ms: act,
        entry_price: 100.0,
        active: true,
        opposite_exit_seen_ms: None,
        last_book_imbalance: None,
        order_flow_imbalance: None,
        close_suppressed_reason: None,
    });

    let o1 = eng
        .on_depth_event(
            "TEST.NS",
            &mk_ev(90.0, 10.0, 1, 111, (act + 400_000) as u64, (act + 400_100) as u64),
            act + 400_000,
        )
        .unwrap();
    assert_eq!(o1.action, PaperObsAction::Hold);
    assert_eq!(
        format!("{:?}", o1.action).to_lowercase().contains("hold"),
        true
    );
    assert_eq!(
        fix["tick_sequence"][0]["action"].as_str().unwrap(),
        "hold"
    );

    let o2 = eng
        .on_depth_event(
            "TEST.NS",
            &mk_ev(10.0, 90.0, 2, 222, (act + 400_000) as u64, (act + 400_200) as u64),
            act + 400_000,
        )
        .unwrap();
    assert_eq!(o2.action, PaperObsAction::CloseSuppressed);
    assert_eq!(
        fix["tick_sequence"][1]["action"].as_str().unwrap(),
        "close_suppressed"
    );
    assert!(o2.deny_reason.as_ref().unwrap().contains("arming"));

    let o3 = eng
        .on_depth_event(
            "TEST.NS",
            &mk_ev(10.0, 90.0, 3, 333, (act + 410_000) as u64, (act + 410_100) as u64),
            act + 400_000 + 10_000,
        )
        .unwrap();
    assert_eq!(o3.action, PaperObsAction::CloseSuppressed);
    assert_eq!(
        fix["tick_sequence"][2]["action"].as_str().unwrap(),
        "close_suppressed"
    );

    let o4 = eng
        .on_depth_event(
            "TEST.NS",
            &mk_ev(10.0, 90.0, 4, 444, (act + 430_000) as u64, (act + 430_100) as u64),
            act + 400_000 + 30_000,
        )
        .unwrap();
    assert_eq!(o4.action, PaperObsAction::CloseOppositeSideOrderFlow);
    assert_eq!(
        fix["tick_sequence"][3]["action"].as_str().unwrap(),
        "close_opposite_side_order_flow"
    );
    assert_eq!(o4.ingest_seq, 4);
    assert_eq!(o4.content_fp, 444);
    assert_eq!(o4.ledger_writer, "python_authoritative");
    assert!(!eng.open("TEST.NS").unwrap().active);
}

#[test]
fn p3_stale_suppresses_current_observation() {
    let fix = fixture();
    let act = 1_700_000_000_000_i64;
    let mut eng = PaperLifecycleObsEngine::new();
    eng.mirror_open(OpenPaperObs {
        symbol: "TEST.NS".into(),
        direction: "LONG".into(),
        of_trade_id: "NSE-OF-PAPER-TEST.NS-10-1".into(),
        activated_ms: act,
        entry_price: 100.0,
        active: true,
        opposite_exit_seen_ms: None,
        last_book_imbalance: None,
        order_flow_imbalance: None,
        close_suppressed_reason: None,
    });
    eng.on_disconnect();
    let o = eng
        .on_depth_event(
            "TEST.NS",
            &mk_ev(10.0, 90.0, 9, 999, (act + 500_000) as u64, (act + 500_100) as u64),
            act + 500_000,
        )
        .unwrap();
    assert_eq!(o.action, PaperObsAction::SuppressedStale);
    assert!(!o.is_current);
    assert_eq!(
        fix["stale_obs"]["action"].as_str().unwrap(),
        "suppressed_stale"
    );
    assert_eq!(fix["governance"]["dual_write"], false);
    assert_eq!(fix["governance"]["cutover"], false);
}
