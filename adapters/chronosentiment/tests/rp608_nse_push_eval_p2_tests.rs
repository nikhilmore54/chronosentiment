//! P2: push watchlist + recommendation eval vs Python fixtures.

use chronosentiment_adapter::nse_market_events::{
    evaluate_recommendation, rank_watchlist, traj15_live, PushEvalEngine, QuoteFlags,
    WatchCandidateInput, MAX_NEW_ENTRY_BAR, MANDATORY_SQUARE_OFF_BAR, TRAJ15_ENTRY_THRESHOLD,
    WATCHLIST_TOP_N,
};
use serde_json::Value;
use std::fs;
use std::path::PathBuf;

fn fixture() -> Value {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/nse_kite_full/p2_push_eval_differential.json");
    serde_json::from_str(&fs::read_to_string(p).expect("run scripts/rp608_nse_push_eval_ref.py"))
        .unwrap()
}

fn closes_from(v: &Value) -> Vec<f64> {
    v.as_array()
        .unwrap()
        .iter()
        .map(|x| x.as_f64().unwrap())
        .collect()
}

#[test]
fn p2_constants_frozen() {
    let fix = fixture();
    assert!((TRAJ15_ENTRY_THRESHOLD - 0.05).abs() < 1e-12);
    assert_eq!(MAX_NEW_ENTRY_BAR, 19);
    assert_eq!(MANDATORY_SQUARE_OFF_BAR, 23);
    assert_eq!(WATCHLIST_TOP_N, 20);
    assert_eq!(fix["constants"]["watchlist_top_n"], 20);
}

#[test]
fn p2_recommendation_cases_match_python() {
    let fix = fixture();
    for c in fix["recommendation_cases"].as_array().unwrap() {
        let closes = closes_from(&c["closes"]);
        let py_traj = c["traj15_live"].as_f64().unwrap();
        assert!(
            (traj15_live(&closes) - py_traj).abs() < 1e-9,
            "traj {}",
            c["name"]
        );
        let ev = evaluate_recommendation(
            c["symbol"].as_str().unwrap(),
            c["bar_idx"].as_u64().unwrap() as u32,
            &closes,
            c["has_valid_fresh_quote"].as_bool().unwrap(),
            false,
            c.get("quote_stale").and_then(|x| x.as_bool()).unwrap_or(false),
            false,
            false,
            false,
            None,
        );
        let py = &c["eval"];
        assert_eq!(
            ev.decision,
            py["decision"].as_str().unwrap(),
            "decision {}",
            c["name"]
        );
        assert_eq!(
            ev.decision_code,
            py["decision_code"].as_str().unwrap(),
            "code {}",
            c["name"]
        );
        assert!(
            (ev.traj15 - py["traj15"].as_f64().unwrap()).abs() < 1e-9,
            "traj field {}",
            c["name"]
        );
    }
}

#[test]
fn p2_watchlist_top20_and_ordering() {
    let fix = fixture();
    // Rebuild inputs from fixture watchlist alone is insufficient — re-rank from
    // embedded cases by reconstructing via Python golden length/order.
    let py_watch = fix["watchlist"].as_array().unwrap();
    assert!(py_watch.len() <= 20);
    assert_eq!(py_watch.len(), fix["watchlist_n"].as_u64().unwrap() as usize);
    // Scores non-increasing
    for w in py_watch.windows(2) {
        assert!(w[0]["score"].as_f64().unwrap() >= w[1]["score"].as_f64().unwrap());
    }
}

#[test]
fn p2_push_engine_stale_suppresses_current_rec() {
    let fix = fixture();
    let rising: Vec<f64> = (0..8).map(|i| 100.0 + i as f64 * 0.5).collect();
    let mut eng = PushEvalEngine::new(vec!["AAA.NS".into(), "BBB.NS".into()]);
    // seed
    for (i, c) in rising.iter().enumerate().take(7) {
        eng.on_bar_complete(
            "AAA.NS",
            (i + 1) as u32,
            *c,
            true,
            QuoteFlags::default(),
        );
    }
    let e = eng
        .on_bar_complete("AAA.NS", 10, rising[7], true, QuoteFlags::default())
        .unwrap();
    assert_eq!(e.decision, "RECOMMEND_LONG");
    assert!(eng.current_recommendation("AAA.NS").is_some());
    assert_eq!(
        eng.current_recommendation("AAA.NS").unwrap().decision,
        fix["engine"]["before_disconnect_decision"].as_str().unwrap()
    );

    eng.on_disconnect();
    assert!(eng.current_recommendation("AAA.NS").is_none());
    eng.begin_reconcile();
    eng.complete_reconcile();
    assert!(eng.current_recommendation("AAA.NS").is_none());

    let e2 = eng
        .on_bar_complete("AAA.NS", 11, rising[7] + 0.5, true, QuoteFlags::default())
        .unwrap();
    assert_eq!(
        e2.decision_code,
        fix["engine"]["e2_decision_code"].as_str().unwrap()
    );
    assert!(eng.current_recommendation("AAA.NS").is_some());
    assert!(eng.current_recommendation("AAA.NS").unwrap().is_current);
}

#[test]
fn p2_rank_watchlist_rust_matches_fixture_top() {
    let fix = fixture();
    // Minimal parity: first watch symbol and OF support flags from rebuilt inputs
    // matching the Python fixture generator's W1/W2/W3 + Z fillers.
    let rising: Vec<f64> = (0..8).map(|i| 100.0 + i as f64 * 0.5).collect();
    let weak: Vec<f64> = (0..8).map(|i| 100.0 - i as f64 * 0.1).collect();
    let mut inputs = vec![
        WatchCandidateInput {
            symbol: "W1.NS".into(),
            closes: rising.clone(),
            bid_qty: 90.0,
            ask_qty: 10.0,
            bid_price: 99.9,
            ask_price: 100.1,
        },
        WatchCandidateInput {
            symbol: "W2.NS".into(),
            closes: weak,
            bid_qty: 50.0,
            ask_qty: 50.0,
            bid_price: 99.9,
            ask_price: 100.1,
        },
        WatchCandidateInput {
            symbol: "W3.NS".into(),
            closes: (0..10).map(|i| 100.0 + i as f64 * 0.3).collect(),
            bid_qty: 10.0,
            ask_qty: 90.0,
            bid_price: 99.9,
            ask_price: 100.1,
        },
        WatchCandidateInput {
            symbol: "W4.NS".into(),
            closes: vec![100.0; 4],
            bid_qty: 90.0,
            ask_qty: 10.0,
            bid_price: 99.9,
            ask_price: 100.1,
        },
    ];
    for i in 0..25 {
        inputs.push(WatchCandidateInput {
            symbol: format!("Z{i:02}.NS"),
            closes: (0..8)
                .map(|j| 100.0 + i as f64 * 0.01 + j as f64 * 0.2)
                .collect(),
            bid_qty: 60.0,
            ask_qty: 40.0,
            bid_price: 99.9,
            ask_price: 100.1,
        });
    }
    let ranked = rank_watchlist(&inputs, 20);
    let py = fix["watchlist"].as_array().unwrap();
    assert_eq!(ranked.len(), py.len());
    for (r, p) in ranked.iter().zip(py.iter()) {
        assert_eq!(r.symbol, p["symbol"].as_str().unwrap());
        assert!((r.score - p["score"].as_f64().unwrap()).abs() < 1e-9);
        assert_eq!(
            r.order_flow_supported,
            p["order_flow_supported"].as_bool().unwrap()
        );
    }
}
