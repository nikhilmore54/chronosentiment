//! Deterministic Unit Tests for RP-608-PVP-003 Replay Foundation
//!
//! Verifies:
//! 1. Candle ingestion, timestamp monotonicity, gap detection, and geometry validation.
//! 2. Causal feature generation and zero lookahead firewall.
//! 3. Strict causal trailing stop sequence ($S_{t-1}$ evaluated during bar $t$, gap-through-stop at adverse open, $S_t$ active at $t+1$).
//! 4. Same-bar entry/exit accounting with exact fee deductions.
//! 5. Multi-bar telescoping mark-to-market reconciliation in USDT (0.000000 USDT mismatch).
//! 6. Positions spanning across UTC midnight boundary.
//! 7. Monetary cost model exactness (base 12.0 bps nominal and stressed +2.0 bps entry notional).

use chronosentiment_adapter::research::crypto_replay::*;

/// Helper to generate a monotonic sequence of 15-minute candles.
fn generate_linear_candles(
    start_time_ms: i64,
    start_price: f64,
    step_price: f64,
    count: usize,
) -> Vec<BtcCandle15m> {
    let mut candles = Vec::with_capacity(count);
    let mut curr_p = start_price;

    for i in 0..count {
        let open_ms = start_time_ms + (i as i64) * FIFTEEN_MINUTES_MS;
        let open = curr_p;
        let close = curr_p + step_price;
        let high = open.max(close) + 50.0;
        let low = open.min(close) - 50.0;
        let volume = 100.0 + (i as f64) * 2.0;

        candles.push(BtcCandle15m::new(open_ms, open, high, low, close, volume));
        curr_p = close;
    }

    candles
}

#[test]
fn test_candle_ingestion_and_gap_detection() {
    let base_time = 1704067200000; // 2024-01-01 00:00:00 UTC
    let mut candles = generate_linear_candles(base_time, 42000.0, 10.0, 10);

    // Baseline stream must be completely valid
    assert!(validate_candle_stream(&candles).is_ok());

    // Inject a gap: remove candle at index 5
    let mut gapped = candles.clone();
    gapped.remove(5);
    let err = validate_candle_stream(&gapped).expect_err("Must detect missing bar gap");
    assert_eq!(err.len(), 1);
    match &err[0] {
        DataIntegrityAnomaly::MissingBarGap { expected_open_ms, actual_open_ms, missing_bars_count } => {
            assert_eq!(*expected_open_ms, base_time + 5 * FIFTEEN_MINUTES_MS);
            assert_eq!(*actual_open_ms, base_time + 6 * FIFTEEN_MINUTES_MS);
            assert_eq!(*missing_bars_count, 1);
        }
        _ => panic!("Expected MissingBarGap anomaly"),
    }

    // Inject a duplicate timestamp
    let mut duped = candles.clone();
    duped[4] = duped[3].clone();
    let err_dup = validate_candle_stream(&duped).expect_err("Must detect duplicate bar");
    assert!(err_dup.iter().any(|a| matches!(a, DataIntegrityAnomaly::DuplicateBar { .. })));

    // Inject invalid geometry (High < Low)
    let mut invalid = candles.clone();
    invalid[2].high = invalid[2].low - 10.0;
    let err_geom = validate_candle_stream(&invalid).expect_err("Must detect invalid geometry");
    assert!(matches!(err_geom[0], DataIntegrityAnomaly::InvalidBarGeometry { .. }));
}

#[test]
fn test_causal_feature_generation_zero_lookahead() {
    let base_time = 1704067200000;
    let mut candles = generate_linear_candles(base_time, 42000.0, 10.0, 120);

    // Compute causal features at index 95
    let f1 = compute_causal_features(&candles, 95).expect("Features at index 95");
    assert!(f1.rolling_vwap_96.is_some());
    assert!(f1.dist_vwap_bps.is_some());
    assert!(f1.vol_burst_20.is_some());

    // Mutate FUTURE candle at index 96, 97, ...
    candles[96].close += 5000.0;
    candles[96].volume += 100000.0;
    candles[100].high += 10000.0;

    // Features at index 95 must remain 100% IDENTICAL (zero lookahead)
    let f2 = compute_causal_features(&candles, 95).expect("Features at index 95 after future change");
    assert_eq!(f1, f2, "Future bars must not leak into historical feature vector");
}

#[test]
fn test_causal_trailing_stop_sequence() {
    let base_time = 1704067200000;
    // Construct 4 candles
    // Bar 0: Flat -> signal EnterLong
    // Bar 1: Entry occurs at Open = 40000, Low = 39800, High = 40500, Close = 40400 (ATR = 200, stop_0 = 40000 - 400 = 39600)
    //        Survives bar 1. Post-bar causal stop for bar 2 is S_1 = max(39600, 40400 - 400 = 40000).
    // Bar 2: Gap down at Open = 39900 (Open < S_1 = 40000) -> Gap-through-stop filled at adverse open 39900!
    let mut candles = Vec::new();
    candles.push(BtcCandle15m::new(base_time, 39500.0, 39600.0, 39400.0, 39550.0, 100.0));
    candles.push(BtcCandle15m::new(base_time + FIFTEEN_MINUTES_MS, 40000.0, 40500.0, 39800.0, 40400.0, 100.0));
    candles.push(BtcCandle15m::new(base_time + 2 * FIFTEEN_MINUTES_MS, 39900.0, 40100.0, 39700.0, 39800.0, 100.0));

    let mut engine = CausalReplayEngine::new(100_000.0, candles);
    let atr = 200.0;

    // Bar 0: Flat, proposes EnterLong
    engine.step(0, PolicyAction::EnterLong, atr);
    assert_eq!(engine.completed_trades.len(), 0);
    assert!(engine.position.is_some());

    // Bar 1: Position entered at Open = 40000, survives, stop becomes 40000
    engine.step(1, PolicyAction::HoldOrFlat, atr);
    assert_eq!(engine.completed_trades.len(), 0);
    let pos = engine.position.as_ref().unwrap();
    assert_eq!(pos.active_stop, 40000.0);

    // Bar 2: Open is 39900, which is < S_1 (40000.0). Gap through stop executes at adverse open 39900!
    engine.step(2, PolicyAction::HoldOrFlat, atr);
    assert_eq!(engine.completed_trades.len(), 1);
    let trade = &engine.completed_trades[0];
    assert_eq!(trade.exit_reason, ExitReason::GapThroughStop);
    assert_eq!(trade.exit_price, 39900.0, "Gap-through-stop must fill at adverse open price, not stop price");
    assert!(engine.position.is_none());
}

#[test]
fn test_same_bar_entry_exit_accounting() {
    let base_time = 1704067200000;
    // Bar 0: EnterLong at Open = 50000. Low drops to 48000. Stop is 50000 - 2*500 = 49000.
    // Same-bar stop hit at 49000!
    let candles = vec![
        BtcCandle15m::new(base_time, 50000.0, 50200.0, 48000.0, 48500.0, 100.0),
    ];

    let mut engine = CausalReplayEngine::new(100_000.0, candles);
    let atr = 500.0;

    engine.step(0, PolicyAction::EnterLong, atr);

    assert_eq!(engine.completed_trades.len(), 1);
    let trade = &engine.completed_trades[0];
    assert_eq!(trade.entry_bar_idx, 0);
    assert_eq!(trade.exit_bar_idx, 0);
    assert_eq!(trade.entry_price, 50000.0);
    assert_eq!(trade.exit_price, 49000.0);
    assert_eq!(trade.qty, 2.0); // 100,000 / 50,000 = 2.0 BTC

    // Gross PnL = 2.0 * (49000 - 50000) = -2,000.0 USDT
    assert_eq!(trade.gross_pnl_usdt, -2000.0);

    // Monetary friction:
    // Entry Notional = 100,000. Fee = 50 USDT, Slip = 10 USDT
    // Exit Notional = 49,000 * 2 = 98,000. Fee = 49 USDT, Slip = 9.8 USDT
    // Total Base Friction = 60 + 58.8 = 118.8 USDT
    let friction = &trade.base_friction;
    assert_eq!(friction.entry_fee_usdt, 50.0);
    assert_eq!(friction.entry_slippage_usdt, 10.0);
    assert_eq!(friction.exit_fee_usdt, 49.0);
    assert_eq!(friction.exit_slippage_usdt, 9.8);
    assert_eq!(friction.base_total_usdt(), 118.8);

    // Stressed fee: +2.0 bps of entry notional = 20.0 USDT
    assert_eq!(friction.adverse_stress_usdt, 20.0);
    assert_eq!(trade.stressed_net_pnl_usdt, -2000.0 - 138.8);

    // Reconcile with bar record
    assert_eq!(engine.bar_records.len(), 1);
    let bar_rec = &engine.bar_records[0];
    assert_eq!(bar_rec.gross_pnl_usdt, trade.gross_pnl_usdt);
    assert_eq!(bar_rec.friction_usdt, friction.base_total_usdt());
    assert_eq!(bar_rec.net_pnl_usdt, trade.net_pnl_usdt);
}

#[test]
fn test_multi_bar_telescoping_pnl_reconciliation() {
    let base_time = 1704067200000;
    // 5 bars:
    // Bar 0: Enter at 40000. Close = 40500.
    // Bar 1: Open = 40500, High = 41200, Low = 40300, Close = 41000.
    // Bar 2: Open = 41000, High = 41500, Low = 40800, Close = 41400.
    // Bar 3: Open = 41400, High = 41800, Low = 40000 (stopped at 40400).
    let candles = vec![
        BtcCandle15m::new(base_time, 40000.0, 40600.0, 39900.0, 40500.0, 100.0),
        BtcCandle15m::new(base_time + FIFTEEN_MINUTES_MS, 40500.0, 41200.0, 40300.0, 41000.0, 100.0),
        BtcCandle15m::new(base_time + 2 * FIFTEEN_MINUTES_MS, 41000.0, 41500.0, 40800.0, 41400.0, 100.0),
        BtcCandle15m::new(base_time + 3 * FIFTEEN_MINUTES_MS, 41400.0, 41800.0, 40000.0, 40200.0, 100.0),
    ];

    let mut engine = CausalReplayEngine::new(100_000.0, candles);
    let atr = 300.0; // 2 * ATR = 600 stop buffer

    engine.step(0, PolicyAction::EnterLong, atr);
    engine.step(1, PolicyAction::HoldOrFlat, atr);
    engine.step(2, PolicyAction::HoldOrFlat, atr);
    engine.step(3, PolicyAction::HoldOrFlat, atr);

    assert_eq!(engine.completed_trades.len(), 1);
    let trade = &engine.completed_trades[0];

    // Sum of bar records gross PnL
    let sum_bar_gross: f64 = engine.bar_records.iter().map(|r| r.gross_pnl_usdt).sum();
    let sum_bar_friction: f64 = engine.bar_records.iter().map(|r| r.friction_usdt).sum();
    let sum_bar_net: f64 = engine.bar_records.iter().map(|r| r.net_pnl_usdt).sum();

    // Telescoping check: sum of bar gross must EXACTLY match trade gross
    assert!((sum_bar_gross - trade.gross_pnl_usdt).abs() < 1e-8,
        "Bar gross sum ({}) must reconcile to trade gross ({}) to 0.000000 USDT", sum_bar_gross, trade.gross_pnl_usdt);

    // Friction check: sum of bar friction must EXACTLY match trade base friction
    assert!((sum_bar_friction - trade.base_friction.base_total_usdt()).abs() < 1e-8,
        "Bar friction sum ({}) must reconcile to trade friction ({}) to 0.000000 USDT", sum_bar_friction, trade.base_friction.base_total_usdt());

    // Net check: sum of bar net must EXACTLY match trade net
    assert!((sum_bar_net - trade.net_pnl_usdt).abs() < 1e-8,
        "Bar net sum ({}) must reconcile to trade net ({}) to 0.000000 USDT", sum_bar_net, trade.net_pnl_usdt);
}

#[test]
fn test_utc_midnight_boundary_crossing() {
    // 2024-01-01 23:45:00 UTC = 1704152700000
    // 2024-01-02 00:00:00 UTC = 1704153600000
    let t_day1_last_bar = 1704152700000;
    let t_day2_first_bar = 1704153600000;
    let t_day2_second_bar = 1704154500000;

    let candles = vec![
        // Day 1 last bar: enters at 45000, closes at 45200
        BtcCandle15m::new(t_day1_last_bar, 45000.0, 45300.0, 44900.0, 45200.0, 100.0),
        // Day 2 first bar: opens at 45200, closes at 45600
        BtcCandle15m::new(t_day2_first_bar, 45200.0, 45700.0, 45100.0, 45600.0, 100.0),
        // Day 2 second bar: opens at 45600, exits at 45800 on signal
        BtcCandle15m::new(t_day2_second_bar, 45600.0, 45900.0, 45500.0, 45800.0, 100.0),
    ];

    let mut engine = CausalReplayEngine::new(100_000.0, candles);
    let atr = 200.0;

    engine.step(0, PolicyAction::EnterLong, atr);
    engine.step(1, PolicyAction::HoldOrFlat, atr);
    engine.step(2, PolicyAction::ExitPosition, atr);

    assert_eq!(engine.completed_trades.len(), 1);

    let daily = engine.aggregate_daily_returns();
    assert_eq!(daily.len(), 2, "Returns must be partitioned into exactly 2 UTC calendar days");

    assert!(daily.contains_key("2024-01-01"));
    assert!(daily.contains_key("2024-01-02"));

    let sum_daily_bps: f64 = daily.values().sum();
    let total_trade_bps = (engine.completed_trades[0].net_pnl_usdt / 100_000.0) * 10_000.0;

    assert!((sum_daily_bps - total_trade_bps).abs() < 1e-8,
        "Daily aggregated return sum ({}) must match trade return bps ({}) across midnight UTC", sum_daily_bps, total_trade_bps);
}

#[test]
fn test_monetary_cost_model_exactness() {
    let entry_p = 50_000.0;
    let exit_p = 55_000.0;
    let qty = 2.0; // 100,000 USDT entry notional, 110,000 USDT exit notional

    let friction = MonetaryFriction::new(entry_p, qty, exit_p, qty);

    // Entry: 100,000 notional
    // Fee = 100,000 * 0.0005 = 50.0 USDT
    // Slippage = 100,000 * 0.0001 = 10.0 USDT
    assert_eq!(friction.entry_fee_usdt, 50.0);
    assert_eq!(friction.entry_slippage_usdt, 10.0);

    // Exit: 110,000 notional
    // Fee = 110,000 * 0.0005 = 55.0 USDT
    // Slippage = 110,000 * 0.0001 = 11.0 USDT
    assert_eq!(friction.exit_fee_usdt, 55.0);
    assert_eq!(friction.exit_slippage_usdt, 11.0);

    // Base total = 50 + 10 + 55 + 11 = 126.0 USDT
    assert_eq!(friction.base_total_usdt(), 126.0);

    // Adverse stress: +2.0 bps on ENTRY notional = 100,000 * 0.0002 = 20.0 USDT
    assert_eq!(friction.adverse_stress_usdt, 20.0);

    // Stressed total = 126.0 + 20.0 = 146.0 USDT
    assert_eq!(friction.stressed_total_usdt(), 146.0);
}

#[test]
fn test_identical_inputs_produce_identical_decisions_and_fills() {
    let base_time = 1704067200000;
    let candles = generate_linear_candles(base_time, 40000.0, 50.0, 100);

    // Run 1
    let mut engine1 = CausalReplayEngine::new(100_000.0, candles.clone());
    for i in 0..100 {
        let action = if i == 5 || i == 30 || i == 60 {
            PolicyAction::EnterLong
        } else if i == 15 || i == 45 || i == 75 {
            PolicyAction::ExitPosition
        } else {
            PolicyAction::HoldOrFlat
        };
        let feats = compute_causal_features(&candles, i).unwrap();
        engine1.step(i, action, feats.atr_14);
    }

    // Run 2 (completely separate instance with identical inputs)
    let mut engine2 = CausalReplayEngine::new(100_000.0, candles.clone());
    for i in 0..100 {
        let action = if i == 5 || i == 30 || i == 60 {
            PolicyAction::EnterLong
        } else if i == 15 || i == 45 || i == 75 {
            PolicyAction::ExitPosition
        } else {
            PolicyAction::HoldOrFlat
        };
        let feats = compute_causal_features(&candles, i).unwrap();
        engine2.step(i, action, feats.atr_14);
    }

    // Assert exact bitwise reproducibility across all trades and bar records
    assert_eq!(engine1.completed_trades.len(), engine2.completed_trades.len());
    for (t1, t2) in engine1.completed_trades.iter().zip(engine2.completed_trades.iter()) {
        assert_eq!(t1, t2, "Every trade fill, timestamp, and PnL must be bitwise identical across runs");
    }

    assert_eq!(engine1.bar_records.len(), engine2.bar_records.len());
    for (b1, b2) in engine1.bar_records.iter().zip(engine2.bar_records.iter()) {
        assert_eq!(b1, b2, "Every bar mark-to-market valuation must be bitwise identical across runs");
    }
}

#[test]
fn test_strict_cost_charging_invariants_no_double_deduction() {
    let base_time = 1704067200000;
    let candles = generate_linear_candles(base_time, 45000.0, 20.0, 50);

    let mut engine = CausalReplayEngine::new(100_000.0, candles.clone());
    // Trade 1: Entry at bar 5, exit at bar 10 (multi-bar)
    // Trade 2: Entry at bar 20, same-bar stop out at bar 20
    // Trade 3: Entry at bar 30, exit at bar 40 (multi-bar)
    for i in 0..50 {
        let action = if i == 5 || i == 20 || i == 30 {
            PolicyAction::EnterLong
        } else if i == 10 || i == 40 {
            PolicyAction::ExitPosition
        } else {
            PolicyAction::HoldOrFlat
        };
        // For bar 20, make ATR small to force same-bar stop
        let atr = if i == 20 { 1.0 } else { 200.0 };
        engine.step(i, action, atr);
    }

    assert!(engine.completed_trades.len() >= 2);

    for trade in &engine.completed_trades {
        let entry_idx = trade.entry_bar_idx;
        let exit_idx = trade.exit_bar_idx;

        if entry_idx == exit_idx {
            // Same-bar trade: exactly one friction event on bar record equal to total base friction
            let bar_rec = &engine.bar_records[entry_idx];
            assert_eq!(bar_rec.friction_usdt, trade.base_friction.base_total_usdt());
        } else {
            // Multi-bar trade:
            // 1. Entry bar charges ONLY entry fee + entry slippage
            let entry_bar = &engine.bar_records[entry_idx];
            let expected_entry_cost = trade.base_friction.entry_fee_usdt + trade.base_friction.entry_slippage_usdt;
            assert!((entry_bar.friction_usdt - expected_entry_cost).abs() < 1e-8,
                "Entry bar must charge ONLY entry fee and slippage");

            // 2. Intermediate bars charge EXACTLY 0.00 friction
            for b in (entry_idx + 1)..exit_idx {
                let intermediate_bar = &engine.bar_records[b];
                assert_eq!(intermediate_bar.friction_usdt, 0.0,
                    "Intermediate holding bars must charge exactly 0.00 friction (no double counting)");
            }

            // 3. Exit bar charges ONLY exit fee + exit slippage
            let exit_bar = &engine.bar_records[exit_idx];
            let expected_exit_cost = trade.base_friction.exit_fee_usdt + trade.base_friction.exit_slippage_usdt;
            assert!((exit_bar.friction_usdt - expected_exit_cost).abs() < 1e-8,
                "Exit bar must charge ONLY exit fee and slippage");

            // 4. Sum of all bar records in interval equals trade base friction exactly
            let interval_friction_sum: f64 = engine.bar_records[entry_idx..=exit_idx]
                .iter()
                .map(|r| r.friction_usdt)
                .sum();
            assert!((interval_friction_sum - trade.base_friction.base_total_usdt()).abs() < 1e-8,
                "Total bar friction sum must match trade base friction to 0.000000 USDT");
        }
    }
}

#[test]
fn test_adversarial_lookahead_poisoning_invariance() {
    let base_time = 1704067200000;
    let clean_candles = generate_linear_candles(base_time, 42000.0, 30.0, 50);

    // Create poisoned candle stream where bars 25..50 have catastrophic shocks
    let mut poisoned_candles = clean_candles.clone();
    for i in 25..50 {
        poisoned_candles[i].open = 1.0;
        poisoned_candles[i].high = 1000000.0;
        poisoned_candles[i].low = 0.5;
        poisoned_candles[i].close = 999999.0;
        poisoned_candles[i].volume = 50000000.0;
    }

    // Run clean engine up to bar 24
    let mut clean_engine = CausalReplayEngine::new(100_000.0, clean_candles.clone());
    for i in 0..=24 {
        let action = if i == 5 {
            PolicyAction::EnterLong
        } else if i == 20 {
            PolicyAction::ExitPosition
        } else {
            PolicyAction::HoldOrFlat
        };
        let feats = compute_causal_features(&clean_candles, i).unwrap();
        clean_engine.step(i, action, feats.atr_14);
    }

    // Run poisoned engine up to bar 24
    let mut poisoned_engine = CausalReplayEngine::new(100_000.0, poisoned_candles.clone());
    for i in 0..=24 {
        let action = if i == 5 {
            PolicyAction::EnterLong
        } else if i == 20 {
            PolicyAction::ExitPosition
        } else {
            PolicyAction::HoldOrFlat
        };
        let feats = compute_causal_features(&poisoned_candles, i).unwrap();
        poisoned_engine.step(i, action, feats.atr_14);
    }

    // Decisions, fills, PnL, and bar records up to bar 24 must be 100% BITWISE IDENTICAL
    assert_eq!(clean_engine.completed_trades, poisoned_engine.completed_trades,
        "Adversarial future bars must not leak into historical trades or execution fills");
    assert_eq!(clean_engine.bar_records, poisoned_engine.bar_records,
        "Adversarial future bars must not leak into historical bar mark-to-market valuations");
}

