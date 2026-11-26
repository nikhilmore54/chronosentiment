//! P1 acceptance: OF ±0.30 + depth linkage + stale suppression vs Python fixtures.
//! No production cutover. Threshold frozen at 0.30.

use chronosentiment_adapter::nse_market_events::{
    classify_order_flow_support, EngineConfig, FeedStatus, MarketEventEngine, OfSide,
    SourceProvenance, StaleReason, ORDER_FLOW_IMBALANCE_THRESHOLD, ORDER_FLOW_LABEL,
};
use serde_json::Value;
use std::fs;
use std::path::PathBuf;

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/nse_kite_full/p1_of_depth_differential.json")
}

fn load_fixture() -> Value {
    let raw = fs::read_to_string(fixture_path())
        .expect("P1 fixture missing — run scripts/rp608_nse_of_mark_event_ref.py");
    serde_json::from_str(&raw).expect("fixture JSON")
}

fn hex_to_bytes(hex: &str) -> Vec<u8> {
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).expect("hex"))
        .collect()
}

fn approx_opt(a: Option<f64>, b: Option<f64>) {
    match (a, b) {
        (None, None) => {}
        (Some(x), Some(y)) => assert!((x - y).abs() <= 1e-9, "{x} vs {y}"),
        _ => panic!("Option mismatch {a:?} vs {b:?}"),
    }
}

#[test]
fn p1_threshold_frozen() {
    assert!((ORDER_FLOW_IMBALANCE_THRESHOLD - 0.30).abs() < 1e-12);
    let fix = load_fixture();
    assert!((fix["threshold"].as_f64().unwrap() - 0.30).abs() < 1e-12);
}

#[test]
fn p1_classify_cases_match_python() {
    let fix = load_fixture();
    for c in fix["classify_cases"].as_array().unwrap() {
        let bid = c["bid_qty"].as_f64().unwrap();
        let ask = c["ask_qty"].as_f64().unwrap();
        let side = OfSide::parse(c["side"].as_str().unwrap()).unwrap();
        let (bp, ap) = if c["name"].as_str().unwrap() == "empty_book" {
            (0.0, 0.0)
        } else {
            (99.9, 100.1)
        };
        let r = classify_order_flow_support(side, bid, ask, bp, ap, "fixture_replay", 0.30);
        let py = &c["classify"];
        assert_eq!(
            r.order_flow_supported,
            py["order_flow_supported"].as_bool().unwrap(),
            "case {}",
            c["name"]
        );
        approx_opt(
            r.order_flow_imbalance,
            py["order_flow_imbalance"].as_f64(),
        );
        if r.order_flow_supported {
            assert_eq!(r.order_flow_label.as_deref(), Some(ORDER_FLOW_LABEL));
        }
    }
}

#[test]
fn p1_engine_links_of_depth_and_suppresses_stale_current() {
    let fix = load_fixture();
    let pkt_long = hex_to_bytes(fix["packets"]["pkt_long_hex"].as_str().unwrap());
    let pkt_a = hex_to_bytes(fix["packets"]["pkt_a_hex"].as_str().unwrap());
    let pkt_b = hex_to_bytes(fix["packets"]["pkt_b_hex"].as_str().unwrap());
    let receipt = 1_700_000_010_500u64;

    let mut eng = MarketEventEngine::new(EngineConfig::default());
    eng.subscribe([6401]);
    eng.ingest_packet(
        &pkt_long,
        receipt,
        SourceProvenance::FixtureReplay,
        Some("TEST.NS"),
    )
    .unwrap();
    assert!(eng.current_of_mark(6401).is_some());
    let py = &fix["engine"];
    assert_eq!(
        eng.current_of_mark(6401).is_some(),
        py["before_disconnect_has_current"].as_bool().unwrap()
    );

    // Linkage on first observation
    let link = &eng.linked_observations()[0];
    assert_eq!(link.ingest_seq, link.depth_snapshot.ingest_seq);
    assert_eq!(link.content_fp, link.depth_snapshot.content_fp);
    assert_eq!(link.ingest_seq, link.of_mark.ingest_seq);
    assert_eq!(link.content_fp, link.of_mark.content_fp);
    assert_eq!(link.depth_snapshot.depth_buy.len(), 5);
    assert_eq!(link.depth_snapshot.depth_sell.len(), 5);
    assert!(link.of_mark.is_current);

    eng.on_disconnect();
    assert!(matches!(
        eng.feed_status(),
        FeedStatus::Stale {
            reason: StaleReason::Disconnect
        }
    ));
    assert!(eng.current_of_mark(6401).is_none());

    eng.ingest_packet(
        &pkt_a,
        receipt + 1000,
        SourceProvenance::FixtureReplay,
        Some("TEST.NS"),
    )
    .unwrap();
    let stale_acc = eng.accepted().last().unwrap();
    assert!(!stale_acc.of_mark.is_current);
    assert_eq!(
        stale_acc.of_mark.suppressed_reason,
        Some(StaleReason::Disconnect)
    );
    assert!(eng.current_of_mark(6401).is_none());
    assert_eq!(
        stale_acc.of_mark.is_current,
        py["while_stale_of_is_current"].as_bool().unwrap()
    );

    eng.begin_reconcile();
    assert!(eng.current_of_mark(6401).is_none());
    eng.complete_reconcile();
    assert!(eng.current_of_mark(6401).is_none());
    assert_eq!(
        eng.current_of_mark(6401).is_none(),
        py["after_recon_current_cleared"].as_bool().unwrap()
    );

    eng.ingest_packet(
        &pkt_b,
        receipt + 2000,
        SourceProvenance::FixtureReplay,
        Some("TEST.NS"),
    )
    .unwrap();
    assert!(eng.current_of_mark(6401).is_some());
    assert!(eng.current_of_mark(6401).unwrap().is_current);

    assert_eq!(
        eng.of_history().len(),
        py["of_history_len"].as_u64().unwrap() as usize
    );
    assert_eq!(
        eng.depth_history().len(),
        py["depth_history_len"].as_u64().unwrap() as usize
    );
    assert_eq!(
        eng.accepted().len(),
        py["accepted_len"].as_u64().unwrap() as usize
    );

    // Every accepted row links event↔depth↔OF identities
    for (i, row) in eng.linked_observations().iter().enumerate() {
        let py_l = &py["linked"][i];
        assert_eq!(row.ingest_seq, py_l["ingest_seq"].as_u64().unwrap());
        assert_eq!(row.content_fp, py_l["content_fp"].as_u64().unwrap());
        assert_eq!(row.of_mark.is_current, py_l["of_is_current"].as_bool().unwrap());
        approx_opt(
            row.of_mark.inspection.order_flow_imbalance,
            py_l["order_flow_imbalance"].as_f64(),
        );
    }
}
