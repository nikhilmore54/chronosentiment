//! P0 acceptance: Rust vs recorded Python fixtures for Kite full decode,
//! ordering/dedupe, depth snapshots, and stale transitions.
//!
//! No production cutover. Authority 0%. No OF strategy logic.

use chronosentiment_adapter::nse_market_events::{
    decode_full_mode_frame, decode_full_mode_packet, DepthLevel, EngineConfig, FeedStatus,
    MarketEventEngine, SourceProvenance, StaleReason,
};
use serde_json::Value;
use std::fs;
use std::path::PathBuf;

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/nse_kite_full/p0_kite_full_differential.json")
}

fn load_fixture() -> Value {
    let raw = fs::read_to_string(fixture_path()).expect("P0 fixture missing — run scripts/rp608_nse_market_event_ref.py");
    serde_json::from_str(&raw).expect("fixture JSON")
}

fn hex_to_bytes(hex: &str) -> Vec<u8> {
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).expect("hex"))
        .collect()
}

fn approx_eq(a: f64, b: f64, eps: f64) {
    assert!(
        (a - b).abs() <= eps,
        "f64 mismatch: {a} vs {b} (eps={eps})"
    );
}

fn assert_depth_match(rust: &[DepthLevel; 5], py: &Value) {
    let arr = py.as_array().expect("depth array");
    assert_eq!(arr.len(), 5);
    for i in 0..5 {
        approx_eq(rust[i].price, arr[i]["price"].as_f64().unwrap(), 1e-9);
        approx_eq(rust[i].quantity, arr[i]["quantity"].as_f64().unwrap(), 1e-9);
    }
}

#[test]
fn p0_decode_matches_python_pkt_a() {
    let fix = load_fixture();
    let pkt = hex_to_bytes(fix["packets"]["pkt_a_hex"].as_str().unwrap());
    let py = &fix["decoded_pkt_a"];
    let ev = decode_full_mode_packet(&pkt, 1_700_000_000_500, SourceProvenance::FixtureReplay)
        .expect("decode");
    assert_eq!(ev.instrument_token, py["instrument_token"].as_u64().unwrap() as u32);
    approx_eq(ev.last_price, py["last_price"].as_f64().unwrap(), 1e-9);
    approx_eq(ev.volume, py["volume"].as_f64().unwrap(), 1e-9);
    assert_eq!(
        ev.exchange_timestamp_sec,
        py["exchange_timestamp_sec"].as_u64().unwrap() as u32
    );
    approx_eq(ev.best_bid, py["best_bid"].as_f64().unwrap(), 1e-9);
    approx_eq(ev.best_ask, py["best_ask"].as_f64().unwrap(), 1e-9);
    approx_eq(ev.bid_qty, py["bid_qty"].as_f64().unwrap(), 1e-9);
    approx_eq(ev.ask_qty, py["ask_qty"].as_f64().unwrap(), 1e-9);
    assert_depth_match(&ev.depth_buy, &py["depth_buy"]);
    assert_depth_match(&ev.depth_sell, &py["depth_sell"]);
    assert_eq!(ev.source, SourceProvenance::FixtureReplay);
}

#[test]
fn p0_frame_decode_two_packets() {
    let fix = load_fixture();
    let frame = hex_to_bytes(fix["packets"]["frame_ab_hex"].as_str().unwrap());
    let evs = decode_full_mode_frame(&frame, 1_700_000_000_500, SourceProvenance::FixtureReplay)
        .expect("frame");
    assert_eq!(evs.len(), 2);
    assert_eq!(evs[0].instrument_token, 6401);
    assert_eq!(evs[1].instrument_token, 6401);
    approx_eq(evs[0].last_price, 100.05, 1e-9);
    approx_eq(evs[1].last_price, 100.10, 1e-9);
}

#[test]
fn p0_engine_dedupe_order_depth_and_stale() {
    let fix = load_fixture();
    let pkt_a = hex_to_bytes(fix["packets"]["pkt_a_hex"].as_str().unwrap());
    let pkt_b = hex_to_bytes(fix["packets"]["pkt_b_hex"].as_str().unwrap());
    let pkt_c = hex_to_bytes(fix["packets"]["pkt_c_hex"].as_str().unwrap());
    let receipt0 = 1_700_000_000_500u64;

    let mut eng = MarketEventEngine::new(EngineConfig::default());
    eng.subscribe([6401]);
    eng.ingest_packet(&pkt_a, receipt0, SourceProvenance::FixtureReplay, Some("TEST.NS"))
        .unwrap();
    eng.ingest_packet(
        &pkt_a,
        receipt0 + 1,
        SourceProvenance::FixtureReplay,
        Some("TEST.NS"),
    )
    .unwrap();
    eng.ingest_packet(
        &pkt_b,
        receipt0 + 1_000,
        SourceProvenance::FixtureReplay,
        Some("TEST.NS"),
    )
    .unwrap();

    let py_eng = &fix["python_engine"];
    assert_eq!(eng.dropped_duplicates(), 1);
    assert_eq!(eng.accepted().len(), 2);
    assert_eq!(eng.depth_history().len(), 2);

    // Fingerprint parity with Python on first accepted event
    let py_fp = py_eng["accepted"][0]["event"]["content_fp"].as_u64().unwrap();
    assert_eq!(eng.accepted()[0].event.content_fp, py_fp);

    // Depth snapshots retain L1–L5 (not latest-only: history length == accepted)
    assert_eq!(eng.depth_history()[0].depth_buy.len(), 5);
    assert_eq!(eng.depth_history()[0].depth_sell.len(), 5);
    assert_eq!(
        eng.depth_history()[0].protocol_id,
        "RP-608-NSE-OF-BOOK-INTELLIGENCE-001"
    );

    // Stale transitions
    eng.on_disconnect();
    assert!(matches!(
        eng.feed_status(),
        FeedStatus::Stale {
            reason: StaleReason::Disconnect
        }
    ));
    assert!(eng.require_live(6401).is_err());
    eng.begin_reconcile();
    assert!(matches!(eng.feed_status(), FeedStatus::Reconciling { .. }));
    // Must not restore live until complete_reconcile
    assert!(eng.require_live(6401).is_err());
    eng.complete_reconcile();
    assert!(matches!(eng.feed_status(), FeedStatus::Live));
    assert!(eng.require_live(6401).is_ok());

    eng.ingest_packet(
        &pkt_c,
        receipt0 + 2_000,
        SourceProvenance::FixtureReplay,
        Some("RELIANCE.NS"),
    )
    .unwrap();
    assert_eq!(eng.accepted().len(), py_eng["accepted"].as_array().unwrap().len());
    assert_eq!(
        eng.depth_history().len(),
        py_eng["depth_history_len"].as_u64().unwrap() as usize
    );
    assert_eq!(
        eng.dropped_duplicates(),
        py_eng["dropped_duplicates"].as_u64().unwrap()
    );
    assert_eq!(eng.accepted()[2].event.instrument_token, 738561);

    // Ordering vs Python golden (exchange_ts, ingest_seq)
    let ordered = eng.ordered_events();
    let py_ord = py_eng["ordered_tokens_prices"].as_array().unwrap();
    assert_eq!(ordered.len(), py_ord.len());
    for (i, ev) in ordered.iter().enumerate() {
        assert_eq!(
            ev.instrument_token,
            py_ord[i][0].as_u64().unwrap() as u32
        );
        approx_eq(ev.last_price, py_ord[i][1].as_f64().unwrap(), 1e-9);
        assert_eq!(ev.ingest_seq, py_ord[i][2].as_u64().unwrap());
    }
}

#[test]
fn p0_quote_age_marks_stale() {
    let fix = load_fixture();
    let pkt_old = hex_to_bytes(fix["packets"]["pkt_old_hex"].as_str().unwrap());
    let mut eng = MarketEventEngine::new(EngineConfig::default());
    let acc = eng
        .ingest_packet(
            &pkt_old,
            1_700_000_000_000,
            SourceProvenance::FixtureReplay,
            Some("TEST.NS"),
        )
        .unwrap()
        .expect("accepted");
    assert!(acc.observed_while_stale);
    assert!(matches!(
        eng.feed_status(),
        FeedStatus::Stale {
            reason: StaleReason::QuoteAgeExceeded
        }
    ));
    let py_age = fix["age_stale"]["quote_age_ms"].as_f64().unwrap();
    approx_eq(acc.event.quote_age_ms, py_age, 1e-6);
}
