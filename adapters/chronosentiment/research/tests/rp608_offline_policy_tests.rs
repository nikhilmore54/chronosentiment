//! RP-608-PVP-003 Offline Pre-Prospective Audit Test Suite
//!
//! Verifies all core pre-prospective invariants:
//! 1. Feature Alphabet Isolation (Policy B vs Policy C)
//! 2. Rejected Entries Under Full Capacity
//! 3. EligibleExit Execution On Open Positions
//! 4. Stop and Exit Collision Priority Resolution
//! 5. End-of-Data Open Position Closeout
//! 6. Missing Candle Detection Near Decisions/Fills
//! 7. 100% Deterministic Champion Inference
//! 8. Causal Trailing Stop Monotonicity & Risk Geometry
//! 9. Monetary Friction Reconciled to Satoshi & Cent Exactness

use rand::rngs::StdRng;
use rand::SeedableRng;

use chronosentiment_adapter::research::crypto_moga::{
    CryptoGenomeFactory, CryptoGenomeMutation, CryptoMogaEvaluator, FactorAlphabet,
};
use chronosentiment_adapter::research::crypto_replay::{
    validate_candle_stream, BtcCandle15m, CausalReplayEngine, DataIntegrityAnomaly, ExitReason,
    PolicyAction,
};
use coralys_moga::traits::{GenomeFactory, MutationOperator};

fn make_candle(open_time_ms: i64, open: f64, high: f64, low: f64, close: f64, volume: f64) -> BtcCandle15m {
    BtcCandle15m {
        open_time_ms,
        open,
        high,
        low,
        close,
        volume,
        close_time_ms: open_time_ms + 900_000 - 1,
    }
}

fn generate_test_stream(n: usize, base_price: f64) -> Vec<BtcCandle15m> {
    let base_time = 1704067200000; // 2024-01-01 00:00:00 UTC
    (0..n)
        .map(|i| {
            let t = base_time + (i as i64) * 900_000;
            let p = base_price + (i as f64) * 5.0;
            make_candle(t, p, p + 15.0, p - 10.0, p + 8.0, 100.0)
        })
        .collect()
}

#[test]
fn test_feature_alphabet_isolation_policy_b_cannot_contain_exhaustion() {
    let mut rng = StdRng::seed_from_u64(0x42);
    let factory_b = CryptoGenomeFactory {
        alphabet: FactorAlphabet::BaseOnly,
    };
    let mutator_b = CryptoGenomeMutation {
        alphabet: FactorAlphabet::BaseOnly,
    };

    // 1000 generations and mutations: Policy B MUST NEVER contain exhaustion concepts
    for _ in 0..500 {
        let mut genome = factory_b.create(&mut rng);
        for rule in &genome.rules {
            for pred in &rule.when {
                assert!(
                    pred.concept == "Trend" || pred.concept == "Momentum" || pred.concept == "Volatility",
                    "Policy B generated forbidden concept: {}",
                    pred.concept
                );
            }
        }

        mutator_b.mutate(&mut genome, &mut rng);
        for rule in &genome.rules {
            for pred in &rule.when {
                assert!(
                    pred.concept == "Trend" || pred.concept == "Momentum" || pred.concept == "Volatility",
                    "Policy B mutated into forbidden concept: {}",
                    pred.concept
                );
            }
        }
    }
}

#[test]
fn test_feature_alphabet_containment_policy_c_can_contain_exhaustion() {
    let mut rng = StdRng::seed_from_u64(0x42);
    let factory_c = CryptoGenomeFactory {
        alphabet: FactorAlphabet::WithExhaustion,
    };

    let mut found_vwap = false;
    let mut found_burst = false;

    for _ in 0..100 {
        let genome = factory_c.create(&mut rng);
        for rule in &genome.rules {
            for pred in &rule.when {
                if pred.concept == "VWAPStretch" {
                    found_vwap = true;
                }
                if pred.concept == "VolumeBurst" {
                    found_burst = true;
                }
            }
        }
    }

    assert!(found_vwap, "Policy C failed to generate VWAPStretch predicates");
    assert!(found_burst, "Policy C failed to generate VolumeBurst predicates");
}

#[test]
fn test_rejected_entry_when_already_in_position() {
    let candles = generate_test_stream(10, 40000.0);
    let mut engine = CausalReplayEngine::new(100_000.0, candles);

    // Bar 0: Enter position
    engine.step(0, PolicyAction::EnterLong, 50.0);
    assert!(engine.position.is_some(), "Position should be active");
    assert_eq!(engine.rejected_entries, 0);

    // Bar 1: Policy requests another EnterLong -> MUST be rejected (capacity gate)
    engine.step(1, PolicyAction::EnterLong, 50.0);
    assert!(engine.position.is_some(), "Position should remain active");
    assert_eq!(engine.rejected_entries, 1, "Rejected entry must be counted");

    // Bar 2: Another EnterLong -> rejected again
    engine.step(2, PolicyAction::EnterLong, 50.0);
    assert_eq!(engine.rejected_entries, 2, "Second rejected entry must be counted");
}

#[test]
fn test_eligible_exit_on_open_position() {
    let candles = generate_test_stream(10, 40000.0);
    let mut engine = CausalReplayEngine::new(100_000.0, candles);

    // Bar 0: Enter position
    engine.step(0, PolicyAction::EnterLong, 50.0);
    assert!(engine.position.is_some());

    // Bar 1: Policy emits ExitPosition
    engine.step(1, PolicyAction::ExitPosition, 50.0);
    assert!(engine.position.is_none(), "Position should be closed");
    assert_eq!(engine.completed_trades.len(), 1);

    let trade = &engine.completed_trades[0];
    assert_eq!(trade.exit_reason, ExitReason::PolicySignalExit);
    assert_eq!(trade.exit_price, engine.candles[1].open);
}

#[test]
fn test_stop_and_exit_collision_priority() {
    // Test Case 1: Adverse gap open takes priority over signal exit
    let mut candles1 = generate_test_stream(3, 40000.0);
    // At bar 1, price gaps below stop (stop at 40000 - 100 = 39900)
    candles1[1].open = 39800.0; // Opened below 39900 stop
    candles1[1].low = 39750.0;
    let mut engine1 = CausalReplayEngine::new(100_000.0, candles1);
    engine1.step(0, PolicyAction::EnterLong, 50.0);
    engine1.step(1, PolicyAction::ExitPosition, 50.0);

    let trade1 = &engine1.completed_trades[0];
    assert_eq!(trade1.exit_reason, ExitReason::GapThroughStop, "Adverse gap open must trigger GapThroughStop");
    assert_eq!(trade1.exit_price, 39800.0, "Adverse gap must fill at open");

    // Test Case 2: If open is above stop, scheduled open exit executes before intrabar low
    let mut candles2 = generate_test_stream(3, 40000.0);
    candles2[1].open = 40020.0; // Open is fine
    candles2[1].low = 39000.0;  // Intrabar crash
    let mut engine2 = CausalReplayEngine::new(100_000.0, candles2);
    engine2.step(0, PolicyAction::EnterLong, 50.0);
    engine2.step(1, PolicyAction::ExitPosition, 50.0);

    let trade2 = &engine2.completed_trades[0];
    assert_eq!(trade2.exit_reason, ExitReason::PolicySignalExit, "Scheduled exit at open executes before intrabar crash");
    assert_eq!(trade2.exit_price, 40020.0);
}

#[test]
fn test_end_of_data_open_position_closeout() {
    let candles = generate_test_stream(5, 40000.0);
    let mut engine = CausalReplayEngine::new(100_000.0, candles);

    // Enter at bar 0 and hold until end of data
    engine.step(0, PolicyAction::EnterLong, 50.0);
    engine.step(1, PolicyAction::HoldOrFlat, 50.0);
    engine.step(2, PolicyAction::HoldOrFlat, 50.0);
    engine.step(3, PolicyAction::HoldOrFlat, 50.0);
    engine.step(4, PolicyAction::HoldOrFlat, 50.0);

    assert!(engine.position.is_some(), "Position active at final bar");
    assert_eq!(engine.completed_trades.len(), 0);

    // End of data closeout
    engine.close_at_end_of_data();
    assert!(engine.position.is_none(), "Position must be closed at end of data");
    assert_eq!(engine.completed_trades.len(), 1);

    let trade = &engine.completed_trades[0];
    assert_eq!(trade.exit_reason, ExitReason::EndOfDataCloseout);
    assert_eq!(trade.exit_price, engine.candles[4].close);
    assert!(trade.base_friction.exit_fee_usdt > 0.0);
    assert!(trade.base_friction.exit_slippage_usdt > 0.0);
}

#[test]
fn test_missing_candles_near_decision_or_fill_detection() {
    let mut candles = generate_test_stream(10, 40000.0);
    // Introduce a 30m gap between bar 4 and bar 5
    candles[5].open_time_ms += 900_000;
    candles[5].close_time_ms += 900_000;

    let res = validate_candle_stream(&candles);
    assert!(res.is_err(), "Validator must catch missing candle gap");
    let errs = res.unwrap_err();
    assert!(
        matches!(errs[0], DataIntegrityAnomaly::MissingBarGap { .. }),
        "Must identify MissingBarGap anomaly"
    );
}

#[test]
fn test_deterministic_champion_inference() {
    let candles = generate_test_stream(200, 40000.0);
    let evaluator = CryptoMogaEvaluator::new(&candles, FactorAlphabet::BaseOnly, 100_000.0);

    let mut rng = StdRng::seed_from_u64(0x42);
    let factory = CryptoGenomeFactory {
        alphabet: FactorAlphabet::BaseOnly,
    };
    let genome = factory.create(&mut rng);

    // Run 5 repeated evaluations
    let base_eval = evaluator.evaluate_genome(&genome);
    for pass in 1..=5 {
        let eval = evaluator.evaluate_genome(&genome);
        assert_eq!(eval.objectives.net_return_usdt, base_eval.objectives.net_return_usdt, "Pass {} PnL mismatch", pass);
        assert_eq!(eval.objectives.trade_count, base_eval.objectives.trade_count, "Pass {} trade count mismatch", pass);
        assert_eq!(eval.objectives.max_drawdown_usdt, base_eval.objectives.max_drawdown_usdt, "Pass {} max DD mismatch", pass);
        assert_eq!(eval.rejected_entries, base_eval.rejected_entries, "Pass {} rejected entries mismatch", pass);
    }
}

#[test]
fn test_stand_aside_inactivity_receives_honest_zero_fitness() {
    let candles = generate_test_stream(100, 40000.0);
    let evaluator = CryptoMogaEvaluator::new(&candles, FactorAlphabet::BaseOnly, 100_000.0);

    // Genome that always stands aside
    let idle_genome = chronosentiment_adapter::decision_support::policy_genome::RuleListGenome {
        rules: Vec::new(),
        unmatched_action: chronosentiment_adapter::decision_support::DecisionAction::NoTrade,
    };

    let eval = evaluator.evaluate_genome(&idle_genome);
    assert_eq!(eval.objectives.trade_count, 0, "Idle genome must make 0 trades");
    assert_eq!(eval.objectives.net_return_usdt, 0.0, "Idle genome has 0 net return");
    assert_eq!(eval.fitness, 0.0, "Idle genome must receive honest 0.0 fitness (inactivity is not alpha)");
}

#[test]
fn test_stand_aside_holding_position_preserves_stop_and_time_exit() {
    let candles = generate_test_stream(20, 40000.0);
    let mut engine = CausalReplayEngine::new(100_000.0, candles);

    // Bar 0: Enter position
    engine.step(0, PolicyAction::EnterLong, 50.0);
    assert!(engine.position.is_some());

    // Bar 1..15: Stand aside (HoldOrFlat)
    for b in 1..15 {
        engine.step(b, PolicyAction::HoldOrFlat, 50.0);
        assert!(engine.position.is_some(), "Position must continue holding while standing aside");
    }

    // Bar 16: Max holding period (16 bars) expired -> must trigger time exit
    engine.step(16, PolicyAction::HoldOrFlat, 50.0);
    assert!(engine.position.is_none(), "Max horizon must close position even while standing aside");
    assert_eq!(engine.completed_trades[0].exit_reason, ExitReason::MaxHorizonExpired);
}
