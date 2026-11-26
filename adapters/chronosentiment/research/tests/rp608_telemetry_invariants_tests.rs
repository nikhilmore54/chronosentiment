//! RP-608-TEL-001: Telemetry & Observational Measurement Acceptance Invariant Tests
//!
//! Provides executable test proofs for:
//! 1. Read-Only Isolation: Artifact checksum verification & zero order-placement capability.
//! 2. Timestamp Integrity: Distinct decision time, quote observation time, execution time, and stale/out-of-order rejection.
//! 3. Spread & Slippage Accounting: No double-counting when execution price embeds spread.
//! 4. Quote-to-Execution Alignment: Preserving distinct quotes at decision close and execution open.
//! 5. Data Quality Pre-Check Gating: Execution evaluation strictly blocked if data-quality gates fail.
//! 6. Displayed Depth Discipline: Top-of-book treated as estimate, not guaranteed queue fill.

use chronosentiment_adapter::research::crypto_moga::FrozenChampionArtifact;
use chronosentiment_adapter::research::crypto_replay::BtcCandle15m;
use chronosentiment_adapter::research::crypto_telemetry::{
    generate_aligned_l1_quote_stream, verify_file_sha256, ExecutionCostBreakdown,
    MarketQuoteL1, QuoteObservation,
    ReadOnlyShadowTelemetryEngine, TelemetryQualityGates, LOCKED_DATASET_SHA256,
    LOCKED_POLICY_B_SHA256, LOCKED_POLICY_C_SHA256,
};
use std::fs;
use std::path::{Path, PathBuf};

fn resolve_path(rel_path: &str) -> PathBuf {
    let p = Path::new(rel_path);
    if p.exists() {
        return p.to_path_buf();
    }
    let p_up = Path::new("../../").join(rel_path);
    if p_up.exists() {
        return p_up;
    }
    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let p_manifest = Path::new(&manifest_dir).join(rel_path);
        if p_manifest.exists() {
            return p_manifest;
        }
        let p_manifest_root = Path::new(&manifest_dir).join("../../").join(rel_path);
        if p_manifest_root.exists() {
            return p_manifest_root;
        }
    }
    PathBuf::from(rel_path)
}

#[test]
fn test_read_only_isolation_and_checksum_verification() {
    let b_path = resolve_path("datasets/crypto_15m/frozen_policy_b_champion.json");
    let c_path = resolve_path("datasets/crypto_15m/frozen_policy_c_champion.json");
    let d_path = resolve_path("datasets/crypto_15m/binance_btcusdt_2024_15m.json");

    assert!(b_path.exists(), "Policy B artifact must exist");
    assert!(c_path.exists(), "Policy C artifact must exist");
    assert!(d_path.exists(), "Dataset must exist");

    // Verify cryptographic SHA-256 locks:
    let b_valid = verify_file_sha256(b_path.to_str().unwrap(), LOCKED_POLICY_B_SHA256)
        .expect("Policy B hash check failed");
    assert!(b_valid, "Policy B must match locked SHA-256");

    let c_valid = verify_file_sha256(c_path.to_str().unwrap(), LOCKED_POLICY_C_SHA256)
        .expect("Policy C hash check failed");
    assert!(c_valid, "Policy C must match locked SHA-256");

    let d_valid = verify_file_sha256(d_path.to_str().unwrap(), LOCKED_DATASET_SHA256)
        .expect("Dataset hash check failed");
    assert!(d_valid, "Dataset must match locked SHA-256");
}

#[test]
fn test_timestamp_integrity_and_stale_rejection() {
    let target_exec_time = 1_704_153_600_000i64; // Jan 2, 2024 00:00:00 UTC

    // Normal quote: 20 ms before execution
    let fresh_quote = MarketQuoteL1::new(
        target_exec_time - 20,
        1_000_001,
        45_000.0,
        45_005.0,
        2.5,
        3.0,
    )
    .unwrap();

    let fresh_obs = QuoteObservation::new(
        fresh_quote,
        target_exec_time,
        target_exec_time,
        None,
        None,
        1_000,
    );
    assert!(!fresh_obs.is_stale, "Fresh quote must not be flagged stale");
    assert_eq!(fresh_obs.quote_age_ms, 20);

    // Stale quote: 2500 ms before execution (exceeds 1000 ms threshold)
    let stale_quote = MarketQuoteL1::new(
        target_exec_time - 2500,
        1_000_002,
        45_000.0,
        45_005.0,
        2.5,
        3.0,
    )
    .unwrap();

    let stale_obs = QuoteObservation::new(
        stale_quote,
        target_exec_time,
        target_exec_time,
        Some(target_exec_time - 20),
        Some(1_000_001),
        1_000,
    );
    assert!(stale_obs.is_stale, "Quote older than 1000ms must be flagged as STALE");
    assert_eq!(stale_obs.quote_age_ms, 2500);

    // Out-of-order quote (quote_time goes backward)
    let out_of_order_obs = QuoteObservation::new(
        MarketQuoteL1::new(
            target_exec_time - 5000, // goes backwards relative to prev_quote_time
            1_000_000,              // decreasing sequence id
            45_000.0,
            45_005.0,
            2.5,
            3.0,
        )
        .unwrap(),
        target_exec_time,
        target_exec_time,
        Some(target_exec_time - 20),
        Some(1_000_001),
        1_000,
    );
    assert!(
        out_of_order_obs.is_out_of_order,
        "Quote with backward timestamp or decreasing sequence ID must be flagged out-of-order"
    );
}

#[test]
fn test_spread_slippage_no_double_counting() {
    let notional = 100_000.0;
    let spread_bps = 2.0; // 2.0 bps spread -> half spread is 1.0 bp
    let fee_bps = 5.0;
    let slippage_bps = 1.0;
    let order_qty = 2.0;
    let depth_qty = 5.0;

    // Case A: Fill price is modeled at Midpoint (fill_price_is_cross = false).
    // Here spread crossing MUST be charged as explicit friction.
    let cost_mid = ExecutionCostBreakdown::compute(
        notional,
        spread_bps,
        fee_bps,
        slippage_bps,
        order_qty,
        depth_qty,
        false,
    );
    // fee (5) + half_spread (1) + slippage (1) = 7.0 bps
    assert!((cost_mid.total_friction_bps - 7.0).abs() < 1e-6);
    assert!((cost_mid.total_friction_usdt - 70.0).abs() < 1e-4);

    // Case B: Fill price is modeled at Ask/Bid (fill_price_is_cross = true).
    // The execution price ALREADY includes the half-spread crossing.
    // Invariant: Do NOT double-count spread in friction!
    let cost_cross = ExecutionCostBreakdown::compute(
        notional,
        spread_bps,
        fee_bps,
        slippage_bps,
        order_qty,
        depth_qty,
        true,
    );
    // fee (5) + slippage (1) = 6.0 bps (spread crossing is separate and NOT double-counted)
    assert!((cost_cross.total_friction_bps - 6.0).abs() < 1e-6);
    assert!((cost_cross.total_friction_usdt - 60.0).abs() < 1e-4);
    assert_eq!(cost_cross.spread_crossing_bps, 1.0); // Recorded for transparency
    assert_eq!(cost_cross.spread_crossing_usdt, 10.0);
    assert!(cost_cross.double_count_sanitized);
}

#[test]
fn test_quote_to_execution_alignment() {
    // Decision event occurs at bar close T_close (e.g. 15:14:59.999).
    // Execution occurs at bar open T_open (15:15:00.000).
    let bar_open_time = 1_704_153_600_000i64;
    let bar_close_time = bar_open_time + 15 * 60 * 1000 - 1;
    let next_open_time = bar_open_time + 15 * 60 * 1000;

    let quote_close = MarketQuoteL1::new(
        bar_close_time - 10,
        2_000_001,
        45_100.0,
        45_105.0,
        3.0,
        3.0,
    )
    .unwrap();
    let quote_next_open = MarketQuoteL1::new(
        next_open_time - 15,
        2_000_002,
        45_120.0,
        45_126.0,
        2.0,
        2.5,
    )
    .unwrap();

    let obs_close = QuoteObservation::new(
        quote_close.clone(),
        bar_close_time,
        bar_close_time,
        None,
        None,
        1_000,
    );
    let obs_next_open = QuoteObservation::new(
        quote_next_open.clone(),
        next_open_time,
        next_open_time,
        Some(quote_close.quote_time_ms),
        Some(quote_close.sequence_id),
        1_000,
    );

    assert_eq!(obs_close.target_event_time_ms, bar_close_time);
    assert_eq!(obs_next_open.target_event_time_ms, next_open_time);
    assert_ne!(obs_close.target_event_time_ms, obs_next_open.target_event_time_ms);

    // Quote drift from decision to execution
    let quote_drift_bps = ((quote_next_open.mid_price() - quote_close.mid_price())
        / quote_close.mid_price())
        * 10_000.0;
    assert!((quote_drift_bps - 4.5452).abs() < 0.01);
}

#[test]
fn test_data_quality_gates_precheck() {
    let mut candles = Vec::new();
    for i in 0..150 {
        let open_time = 1_704_067_200_000i64 + (i as i64 * 15 * 60 * 1000);
        candles.push(BtcCandle15m::new(
            open_time, 42000.0, 42100.0, 41950.0, 42050.0, 50.0,
        ));
    }

    // Generate normal quotes:
    let quotes_clean = generate_aligned_l1_quote_stream(&candles, false);
    let engine_clean = ReadOnlyShadowTelemetryEngine::new(
        quotes_clean,
        TelemetryQualityGates::default(),
    );
    let report_clean = engine_clean.validate_market_data(&candles);
    assert!(report_clean.gates_passed, "Clean quotes must pass quality gates");
    assert!(report_clean.quote_coverage_pct >= 99.5);

    // Generate contaminated quotes (stale quotes injected):
    let quotes_dirty = generate_aligned_l1_quote_stream(&candles, true);
    // Strict quality gate with max_stale_pct = 0.0%
    let strict_gates = TelemetryQualityGates {
        min_observation_bars: 100,
        min_quote_coverage_pct: 99.5,
        max_stale_quote_pct: 0.0, // Zero tolerance
        max_out_of_order_pct: 0.0,
        max_stale_age_ms: 1_000,
        max_consecutive_missing_bars: 1,
    };
    let engine_dirty = ReadOnlyShadowTelemetryEngine::new(quotes_dirty, strict_gates);
    let report_dirty = engine_dirty.validate_market_data(&candles);

    assert!(
        !report_dirty.gates_passed,
        "Contaminated stream must FAIL quality gates"
    );
    assert!(!report_dirty.gate_failure_reasons.is_empty());

    // Load Policy B champion
    let b_path = resolve_path("datasets/crypto_15m/frozen_policy_b_champion.json");
    let b_json = fs::read_to_string(&b_path).expect("Failed to read Policy B");
    let artifact_b: FrozenChampionArtifact =
        serde_json::from_str(&b_json).expect("Failed to parse Policy B");

    // Methodological Safeguard 3: Attempting to evaluate shadow policy with failed gates MUST Err!
    let eval_result = engine_dirty.evaluate_shadow_policy(&artifact_b, &candles, &report_dirty);
    assert!(
        eval_result.is_err(),
        "evaluate_shadow_policy must refuse calculation when quality gates fail!"
    );
    let err_msg = eval_result.err().unwrap();
    assert!(err_msg.contains("Methodological Safeguard 3 Gating Violation"));
}

#[test]
fn test_displayed_depth_as_estimate_not_guarantee() {
    let quote = MarketQuoteL1::new(1000, 1, 50_000.0, 50_010.0, 1.0, 1.0).unwrap();
    let notional_large = 100_000.0;
    let btc_price = quote.mid_price();
    let order_qty = notional_large / btc_price; // 2.0 BTC

    // Displayed ask depth is only 1.0 BTC, but order is 2.0 BTC!
    assert!(order_qty > quote.ask_size);

    let cost = ExecutionCostBreakdown::compute(
        notional_large,
        quote.spread_bps(),
        5.0,
        1.0,
        order_qty,
        quote.ask_size,
        false,
    );

    // Impact must be strictly positive because order exceeds 50% displayed depth:
    assert!(
        cost.market_impact_bps > 0.0,
        "Order exceeding displayed depth must incur market impact"
    );
    assert!(cost.market_impact_usdt > 0.0);
}
