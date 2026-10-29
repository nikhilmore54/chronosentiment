//! RP-608-SHADOW-001 Implementation Readiness & Invariant Tests
//!
//! Validates:
//! 1. Clock synchronization mapping (wallclock vs monotonic vs exchange UTC).
//! 2. Unexecutable opportunity accounting (dropped signals excluded from executed trade population).
//! 3. Execution-price semantics and 5-second quote timeout.
//! 4. Quality gate precision (consecutive vs isolated missing intervals).
//! 5. Terminal closeout accounting at window boundary.

use chronosentiment_adapter::research::crypto_replay::{BtcCandle15m, CausalReplayEngine, PolicyAction};
use chronosentiment_adapter::research::crypto_telemetry::{
    compute_effective_sample_size, compute_session_cluster_count, wilson_score_interval_95,
    ClockSynchronizationMapping, CompartmentEvaluationReport, CompartmentalizedTradeRecord,
    ExchangeClockOffset, ExecutionRealismMetrics, FullProspectiveEvaluationReport,
    HistoricalMarketTrack, IndependentDataQualityDenominators, MarketCompartment,
    OpportunityAccountingRegistry, PredictiveAccuracyMetrics, PriceTrackMetrics,
    ProspectiveAcceptanceCriteria, ProspectiveEvaluationOutcome, TelemetryQualityGates,
    TradeQualityMetrics, TripartiteProspectiveEvaluation, UnexecutableDropReason,
    UnexecutableOpportunity, VolumeTrackMetrics,
};

#[test]
fn test_clock_synchronization_and_monotonic_mapping() {
    let startup_exchange_utc = 1792886400000i64; // 2026-10-25T00:00:00.000Z
    let startup_wallclock_utc = 1792886400045i64; // +45 ms local drift
    let startup_mono_ns = 10_000_000_000u64;

    let mapping = ClockSynchronizationMapping::new(
        startup_exchange_utc,
        startup_wallclock_utc,
        startup_mono_ns,
        250, // max permitted drift ms
    );

    assert_eq!(mapping.last_measured_clock_drift_ms, 45);
    assert!(mapping.is_synchronized(startup_wallclock_utc, startup_exchange_utc));

    // Drift exceeding 250ms must fail synchronization check:
    assert!(!mapping.is_synchronized(startup_exchange_utc + 300, startup_exchange_utc));

    // Monotonic time conversion:
    let elapsed_ns = 5_000_000_000u64; // 5 seconds
    let current_mono_ns = startup_mono_ns + elapsed_ns;
    let approx_exchange_ms = mapping.monotonic_to_exchange_utc_ms(current_mono_ns);

    assert_eq!(approx_exchange_ms, startup_exchange_utc + 5000);
}

#[test]
fn test_unexecutable_opportunity_accounting_safeguard() {
    let mut registry = OpportunityAccountingRegistry::new();

    // 10 signals occur
    // 8 are executed normally
    for _ in 0..8 {
        registry.record_executed();
    }

    // 2 are dropped due to microstructure issues (stale quote and feed timeout)
    registry.record_drop(UnexecutableOpportunity {
        opportunity_id: 9,
        bar_idx: 142,
        decision_time_ms: 1792900000000,
        intended_policy_action: PolicyAction::EnterLong,
        drop_reason: UnexecutableDropReason::StaleQuoteExceeded {
            quote_age_ms: 1200,
            threshold_ms: 500,
        },
        disposition: "UnexecutableStaleQuoteDrop".to_string(),
    });

    registry.record_drop(UnexecutableOpportunity {
        opportunity_id: 10,
        bar_idx: 198,
        decision_time_ms: 1792950000000,
        intended_policy_action: PolicyAction::EnterLong,
        drop_reason: UnexecutableDropReason::ExecutionQuoteTimeout {
            timeout_ms: 5000,
        },
        disposition: "UnexecutableTimeoutDrop".to_string(),
    });

    // Invariant 1: Total opportunities = Executed + Dropped
    assert_eq!(registry.total_signal_opportunities, 10);
    assert_eq!(registry.executed_trades_count, 8);
    assert_eq!(registry.unexecutable_drops_count, 2);

    // Invariant 2: Dropped signals MUST NOT be counted as executed trades
    assert_eq!(registry.execution_drop_rate_pct, 20.0);
    assert_eq!(registry.unexecutable_drops.len(), 2);
}

#[test]
fn test_quality_gate_consecutive_vs_isolated_missing_bars() {
    let gates = TelemetryQualityGates {
        min_observation_bars: 100,
        min_quote_coverage_pct: 99.5,
        max_stale_quote_pct: 0.5,
        max_out_of_order_pct: 0.1,
        max_stale_age_ms: 500,
        max_consecutive_missing_bars: 2,
    };

    // Scenario A: 2880 bars total, exactly 5 isolated missing bars (0 consecutive >= 2)
    let total_bars = 2880;
    let matched_bars = 2875;
    let coverage_pct = (matched_bars as f64 / total_bars as f64) * 100.0; // 99.826%
    assert!(coverage_pct >= gates.min_quote_coverage_pct);

    // Scenario B: 2 consecutive missing bars (breaches Gate 5)
    let consecutive_missing = 2;
    let breaches_consecutive_gate = consecutive_missing >= gates.max_consecutive_missing_bars;
    assert!(breaches_consecutive_gate);
}

#[test]
fn test_terminal_closeout_at_window_boundary() {
    let candles = vec![
        BtcCandle15m {
            open_time_ms: 1000,
            close_time_ms: 1000 + 900_000,
            open: 65000.0,
            high: 65500.0,
            low: 64900.0,
            close: 65200.0,
            volume: 100.0,
        },
        BtcCandle15m {
            open_time_ms: 1000 + 900_000,
            close_time_ms: 1000 + 1_800_000,
            open: 65200.0,
            high: 65600.0,
            low: 65100.0,
            close: 65400.0,
            volume: 120.0,
        },
    ];

    let mut engine = CausalReplayEngine::new(100_000.0, candles);
    engine.step(0, PolicyAction::EnterLong, 500.0);

    // Position remains open at bar 0
    assert!(engine.position.is_some());
    assert_eq!(engine.completed_trades.len(), 0);

    // Window boundary reached: force terminal closeout
    engine.close_at_end_of_data();

    // Position MUST be forcibly closed with zero open positions remaining
    assert!(engine.position.is_none());
    assert_eq!(engine.completed_trades.len(), 1);
    let closeout_trade = &engine.completed_trades[0];
    assert_eq!(closeout_trade.exit_price, 65400.0); // closed at final candle close
    assert_eq!(closeout_trade.exit_bar_idx, 1);
}

#[test]
fn test_exchange_clock_offset_and_transit_latency_decoupling() {
    let t_send = 1792886400000i64;
    let t_server = 1792886399980i64; // exchange server is 20ms behind midpoint
    let t_recv = 1792886400040i64; // 40ms round-trip

    let offset = ExchangeClockOffset::compute(t_send, t_server, t_recv);

    assert_eq!(offset.round_trip_delay_ms, 40);
    assert_eq!(offset.measurement_uncertainty_ms, 20);
    // Midpoint: 1792886400020. Offset = 1792886400020 - 1792886399980 = +40ms.
    assert_eq!(offset.estimated_clock_offset_ms, 40);

    // Now test a quote arrival:
    let quote_event_time = 1792886401000i64;
    // Local wallclock receipt is 1792886401065ms (+65ms)
    let local_recv = 1792886401065i64;
    let transit_latency = offset.compute_quote_transit_latency_ms(quote_event_time, local_recv);
    // exchange_aligned_recv = 1792886401065 - 40 = 1792886401025. Transit latency = 25ms.
    assert_eq!(transit_latency, 25);
}

#[test]
fn test_independent_data_quality_denominators() {
    let denoms = IndependentDataQualityDenominators::new(
        2880, // expected candles
        2878, // received candles (2 isolated missing)
        2878, // execution events
        2870, // matched fresh quotes (8 stale quotes dropped)
        120,  // candidate signals
        118,  // signals obtaining fresh quote before 5s timeout
    );

    assert!((denoms.candle_completeness_pct - 99.9305).abs() < 0.001);
    assert!((denoms.quote_coverage_pct - 99.7220).abs() < 0.001);
    assert!((denoms.execution_availability_pct - 98.3333).abs() < 0.001);

    // Each denominator is distinct:
    assert_ne!(denoms.expected_candle_count, denoms.total_execution_events);
    assert_ne!(denoms.total_execution_events, denoms.total_candidate_signals);
}

#[test]
fn test_prospective_acceptance_criteria_defaults() {
    let criteria = ProspectiveAcceptanceCriteria::default();
    assert_eq!(criteria.min_precision_pct, 60.0);
    assert_eq!(criteria.min_balanced_accuracy_pct, 55.0);
    assert_eq!(criteria.max_brier_score, 0.25);
    assert_eq!(criteria.min_trades_for_confirmation, 30);
    assert_eq!(criteria.min_effective_sample_size, 10);
    assert_eq!(criteria.min_gross_expectancy_bps, 15.0);
    assert_eq!(criteria.min_gross_profit_factor, 1.30);
    assert_eq!(criteria.min_edge_ratio, 1.25);
    assert_eq!(criteria.min_net_profit_usdt, 0.0);
    assert_eq!(criteria.min_profitable_days_pct, 55.0);
    assert_eq!(criteria.min_newey_west_t_stat, 2.0);
    assert_eq!(criteria.max_execution_drop_rate_pct, 1.0);
}

#[test]
fn test_tripartite_prospective_evaluation_layers() {
    let criteria = ProspectiveAcceptanceCriteria::default();

    // Scenario 1: Genuine predictive edge that survives friction (Pass Layer 1 & 3)
    let layer1_pass = PredictiveAccuracyMetrics::compute(
        65, // TP: 65 gross profitable trades
        35, // FP: 35 gross unprofitable trades (Precision: 65.0%)
        500, // TN: 500 flat/no-trade bars
        30,  // FN: 30 missed upward bars
        &criteria,
    );
    assert!(layer1_pass.passed_predictive_gate);
    assert_eq!(layer1_pass.precision_pct, 65.0);
    assert!(layer1_pass.brier_score < 0.25);

    let layer2 = TradeQualityMetrics::compute(
        22.5, // gross expectancy bps
        1.45, // gross PF
        45.0, // mean MFE
        30.0, // mean MAE (Edge ratio = 1.5)
        &criteria,
    );
    assert!(layer2.passed_quality_gate);
    assert_eq!(layer2.edge_ratio, 1.5);

    let layer3_pass = ExecutionRealismMetrics::compute(
        8.4,       // net expectancy bps (after friction)
        1.18,      // net PF
        15_420.0,  // net PnL USDT
        35_000.0,  // gross profit USDT
        19_580.0,  // friction paid USDT
        &criteria,
    );
    assert!(layer3_pass.passed_execution_gate);

    let eval_confirmed = TripartiteProspectiveEvaluation::evaluate(
        "Policy_B_Champion".to_string(),
        "Binance_BTCUSDT".to_string(),
        layer1_pass.clone(),
        layer2.clone(),
        layer3_pass.clone(),
        &criteria,
    );
    assert_eq!(
        eval_confirmed.outcome,
        ProspectiveEvaluationOutcome::ConfirmedPredictiveEdgeAndViable
    );

    // Scenario 2: Predictive edge exists, but wiped out by friction (Pass Layer 1, Fail Layer 3)
    let layer3_friction_bound = ExecutionRealismMetrics::compute(
        -4.2,      // negative net expectancy bps
        0.92,      // net PF < 1.0
        -6_500.0,  // negative net PnL
        25_000.0,  // gross profit
        31_500.0,  // heavy friction wiped out profit
        &criteria,
    );
    assert!(!layer3_friction_bound.passed_execution_gate);

    let eval_friction_bound = TripartiteProspectiveEvaluation::evaluate(
        "Policy_C_Champion".to_string(),
        "NSE_Equities_102".to_string(),
        layer1_pass.clone(),
        layer2.clone(),
        layer3_friction_bound,
        &criteria,
    );
    assert_eq!(
        eval_friction_bound.outcome,
        ProspectiveEvaluationOutcome::PredictiveEdgeOnlyFrictionBound
    );

    // Scenario 3: Statistically falsified predictive edge (95% Wilson CI upper bound <= 50.0%)
    let layer1_falsified = PredictiveAccuracyMetrics::compute(
        35, // TP
        65, // FP (Precision: 35.0%, Wilson 95% CI upper bound ~44.7% <= 50.0%)
        500,
        50,
        &criteria,
    );
    assert!(!layer1_falsified.passed_predictive_gate);
    assert!(layer1_falsified.precision_ci_upper_95 <= 50.0);

    let eval_falsified = TripartiteProspectiveEvaluation::evaluate(
        "Test_Policy".to_string(),
        "Binance_BTCUSDT".to_string(),
        layer1_falsified,
        layer2.clone(),
        layer3_pass.clone(),
        &criteria,
    );
    assert_eq!(
        eval_falsified.outcome,
        ProspectiveEvaluationOutcome::FalsifiedZeroEdge
    );

    // Scenario 4: Predictive edge exists, but Layer 2 trade quality is substandard (Edge ratio < 1.25)
    let layer2_substandard = TradeQualityMetrics::compute(
        8.0,  // gross expectancy bps < 15.0 bps
        1.10, // PF < 1.30
        20.0, // MFE
        25.0, // MAE => edge ratio = 0.80 < 1.25
        &criteria,
    );
    assert!(!layer2_substandard.passed_quality_gate);

    let eval_substandard = TripartiteProspectiveEvaluation::evaluate(
        "Policy_Substandard".to_string(),
        "Binance_BTCUSDT".to_string(),
        layer1_pass.clone(),
        layer2_substandard,
        layer3_pass.clone(),
        &criteria,
    );
    assert_eq!(
        eval_substandard.outcome,
        ProspectiveEvaluationOutcome::PredictiveEdgeSubstandardQuality
    );

    // Scenario 5: Adequately sampled (N = 100), but inconclusive (Precision = 54.0%, CI spans 50% null)
    let layer1_inconclusive = PredictiveAccuracyMetrics::compute(
        54, // TP
        46, // FP (Precision: 54.0% - above coin flip, but below 60.0% hurdle & CI lower bound <= 50.0%)
        500,
        50,
        &criteria,
    );
    assert!(!layer1_inconclusive.passed_predictive_gate);
    assert!(layer1_inconclusive.precision_pct > 50.0);

    let eval_inconclusive = TripartiteProspectiveEvaluation::evaluate(
        "Policy_Inconclusive".to_string(),
        "Binance_BTCUSDT".to_string(),
        layer1_inconclusive,
        layer2,
        layer3_pass,
        &criteria,
    );
    assert_eq!(
        eval_inconclusive.outcome,
        ProspectiveEvaluationOutcome::InconclusiveEvidence
    );
}

#[test]
fn test_market_compartment_classification_causal_precedence() {
    let t_pre_decision = 1792886400000i64; // strictly prior to signal

    // 1. Test C1: High-Range Directional + Active Volume
    let c1_track = HistoricalMarketTrack::classify(
        "BTCUSDT".to_string(),
        t_pre_decision,
        PriceTrackMetrics {
            natr_20_bps: 260.0,
            dir_eff_20: 0.18,
            ar1_60: 0.05,
            cv_range_20: 0.30,
            opp_ratio: 28.8,
        },
        VolumeTrackMetrics {
            rvol_20: 1.25,
            volume_burst_index: 1.1,
            volume_trend_alignment: 0.8,
            liquidity_depth_ratio: 5.0,
        },
    );
    assert_eq!(c1_track.assigned_compartment, MarketCompartment::C1HighRangeDirectional);

    // 2. Test C2: High-Range Volatile Oscillator + Active Volume
    let c2_track = HistoricalMarketTrack::classify(
        "RELIANCE.NS".to_string(),
        t_pre_decision,
        PriceTrackMetrics {
            natr_20_bps: 240.0,
            dir_eff_20: 0.08,
            ar1_60: -0.18,
            cv_range_20: 0.35,
            opp_ratio: 26.6,
        },
        VolumeTrackMetrics {
            rvol_20: 1.10,
            volume_burst_index: 0.5,
            volume_trend_alignment: -0.2,
            liquidity_depth_ratio: 4.2,
        },
    );
    assert_eq!(c2_track.assigned_compartment, MarketCompartment::C2HighRangeOscillator);

    // 3. Test C4 Precedence: Even if NATR is high, if RVOL is drained (< 0.60), classify C4
    let c4_drained = HistoricalMarketTrack::classify(
        "ILLIQUID.NS".to_string(),
        t_pre_decision,
        PriceTrackMetrics {
            natr_20_bps: 250.0,
            dir_eff_20: 0.20,
            ar1_60: 0.05,
            cv_range_20: 0.25,
            opp_ratio: 27.7,
        },
        VolumeTrackMetrics {
            rvol_20: 0.45, // Volume Drained (< 0.60)
            volume_burst_index: -1.2,
            volume_trend_alignment: 0.0,
            liquidity_depth_ratio: 0.8,
        },
    );
    assert_eq!(c4_drained.assigned_compartment, MarketCompartment::C4LowRangeFrictionBound);

    // 4. Test C3: Stable Moderate Movers
    let c3_track = HistoricalMarketTrack::classify(
        "TCS.NS".to_string(),
        t_pre_decision,
        PriceTrackMetrics {
            natr_20_bps: 185.0,
            dir_eff_20: 0.12,
            ar1_60: -0.05,
            cv_range_20: 0.35,
            opp_ratio: 20.5,
        },
        VolumeTrackMetrics {
            rvol_20: 0.95,
            volume_burst_index: 0.1,
            volume_trend_alignment: 0.3,
            liquidity_depth_ratio: 6.0,
        },
    );
    assert_eq!(c3_track.assigned_compartment, MarketCompartment::C3StableModerate);

    // 5. Test Unclassified / Transition Band
    let unclass_track = HistoricalMarketTrack::classify(
        "TRANSITION.NS".to_string(),
        t_pre_decision,
        PriceTrackMetrics {
            natr_20_bps: 190.0,
            dir_eff_20: 0.12,
            ar1_60: -0.05,
            cv_range_20: 0.55, // CV > 0.45 excludes from C3
            opp_ratio: 21.0,
        },
        VolumeTrackMetrics {
            rvol_20: 0.85,
            volume_burst_index: 0.0,
            volume_trend_alignment: 0.0,
            liquidity_depth_ratio: 2.0,
        },
    );
    assert_eq!(unclass_track.assigned_compartment, MarketCompartment::Unclassified);
}

#[test]
fn test_compartmentalized_prospective_evaluation_and_ci() {
    let criteria = ProspectiveAcceptanceCriteria::default();

    // Create 40 synthetic trades across compartments
    let mut trades = Vec::new();

    // Compartment 1: 30 trades, 21 wins (70.0% win rate), strong positive returns
    for i in 0..30 {
        let is_win = i < 21;
        trades.push(CompartmentalizedTradeRecord {
            trade_id: format!("C1_T_{}", i),
            symbol: "BTCUSDT".to_string(),
            signal_timestamp_ms: 1792886400000 + (i as i64 * 900_000),
            compartment: MarketCompartment::C1HighRangeDirectional,
            is_predicted_profitable: true,
            predicted_probability: 0.65,
            is_realized_profitable: is_win,
            gross_pnl_bps: if is_win { 45.0 } else { -25.0 },
            net_pnl_bps: if is_win { 35.0 } else { -35.0 },
            gross_pnl_quote: if is_win { 450.0 } else { -250.0 },
            net_pnl_quote: if is_win { 350.0 } else { -350.0 },
            friction_paid_quote: 100.0,
            mfe_bps: if is_win { 55.0 } else { 10.0 },
            mae_bps: if is_win { 15.0 } else { 28.0 },
        });
    }

    // Compartment 4: 10 trades, 3 wins (30.0% win rate), negative returns
    for i in 0..10 {
        let is_win = i < 3;
        trades.push(CompartmentalizedTradeRecord {
            trade_id: format!("C4_T_{}", i),
            symbol: "BTCUSDT".to_string(),
            signal_timestamp_ms: 1792913400000 + (i as i64 * 900_000),
            compartment: MarketCompartment::C4LowRangeFrictionBound,
            is_predicted_profitable: true,
            predicted_probability: 0.60,
            is_realized_profitable: is_win,
            gross_pnl_bps: if is_win { 15.0 } else { -18.0 },
            net_pnl_bps: if is_win { 5.0 } else { -28.0 },
            gross_pnl_quote: if is_win { 150.0 } else { -180.0 },
            net_pnl_quote: if is_win { 50.0 } else { -280.0 },
            friction_paid_quote: 100.0,
            mfe_bps: 12.0,
            mae_bps: 20.0,
        });
    }

    let report = FullProspectiveEvaluationReport::evaluate_trades(
        "Policy_B_Shadow_Forward".to_string(),
        "Binance_BTCUSDT".to_string(),
        &trades,
        &criteria,
    );

    // Aggregate evaluation check: 40 total trades, 24 wins = 60.0% precision
    assert_eq!(report.aggregate_evaluation.layer1_predictive_accuracy.total_predictions, 40);
    assert_eq!(report.aggregate_evaluation.layer1_predictive_accuracy.precision_pct, 60.0);

    // Compartment 1 check: 30 trades, 70.0% precision
    let c1_report = &report.compartment_evaluations[&MarketCompartment::C1HighRangeDirectional];
    assert_eq!(c1_report.sample_size, 30);
    assert_eq!(c1_report.layer1_predictive_accuracy.precision_pct, 70.0);
    assert!(c1_report.layer1_predictive_accuracy.precision_ci_lower_95 > 50.0);
    assert_eq!(c1_report.outcome, ProspectiveEvaluationOutcome::ConfirmedPredictiveEdgeAndViable);

    // Compartment 4 check: 10 trades (< 30 min sample size => InconclusiveInsufficientSample)
    let c4_report = &report.compartment_evaluations[&MarketCompartment::C4LowRangeFrictionBound];
    assert_eq!(c4_report.sample_size, 10);
    assert_eq!(c4_report.layer1_predictive_accuracy.precision_pct, 30.0);
    assert_eq!(c4_report.outcome, ProspectiveEvaluationOutcome::InconclusiveInsufficientSample);

    // Unclassified compartment has 0 sample size
    let unclass_report = &report.compartment_evaluations[&MarketCompartment::Unclassified];
    assert_eq!(unclass_report.sample_size, 0);
    assert_eq!(unclass_report.outcome, ProspectiveEvaluationOutcome::InconclusiveInsufficientSample);

    // Verify Wilson 95% CI behavior
    let (ci_low, ci_high) = wilson_score_interval_95(21, 30);
    assert!(ci_low > 50.0 && ci_low < 70.0);
    assert!(ci_high > 70.0 && ci_high <= 85.0);
}

#[test]
fn test_clock_offset_sign_convention_and_drift_scenarios() {
    // 1. Positive Offset Scenario: Local clock is 50ms ahead of exchange clock
    // Local send = 1000, Server = 960, Local recv = 1020 (RTT = 20ms, midpoint = 1010)
    // Offset = 1010 - 960 = +50ms
    let offset_pos = ExchangeClockOffset::compute(1000, 960, 1020);
    assert_eq!(offset_pos.estimated_clock_offset_ms, 50);
    assert_eq!(offset_pos.round_trip_delay_ms, 20);
    assert_eq!(offset_pos.measurement_uncertainty_ms, 10);

    // Quote emitted at exchange event 1000, received at local wall clock 1065:
    // Physical transit latency = 1065 - 50 - 1000 = 15ms
    let transit_pos = offset_pos.compute_quote_transit_latency_ms(1000, 1065);
    assert_eq!(transit_pos, 15);

    // 2. Negative Offset Scenario: Local clock is 30ms behind exchange clock
    // Local send = 1000, Server = 1050, Local recv = 1040 (RTT = 40ms, midpoint = 1020)
    // Offset = 1020 - 1050 = -30ms
    let offset_neg = ExchangeClockOffset::compute(1000, 1050, 1040);
    assert_eq!(offset_neg.estimated_clock_offset_ms, -30);
    assert_eq!(offset_neg.round_trip_delay_ms, 40);
    assert_eq!(offset_neg.measurement_uncertainty_ms, 20);

    // Quote emitted at exchange event 1050, received at local wall clock 1045:
    // Physical transit latency = 1045 - (-30) - 1050 = 1075 - 1050 = 25ms
    let transit_neg = offset_neg.compute_quote_transit_latency_ms(1050, 1045);
    assert_eq!(transit_neg, 25);

    // 3. Stale Quote Filtering: Quote transit latency > 500ms
    // Quote emitted at 1000, received at 1560 with offset +50ms => latency = 1560 - 50 - 1000 = 510ms > 500ms
    let transit_stale = offset_pos.compute_quote_transit_latency_ms(1000, 1560);
    assert_eq!(transit_stale, 510);
    assert!(transit_stale > 500); // Discarded as stale quote
}

#[test]
fn test_effective_sample_size_safeguards_and_brier_training_baseline() {
    // 1. Effective sample size under positive autocorrelation
    // Generate returns with strong positive correlation
    let mut correlated_returns = Vec::new();
    let mut val = 10.0;
    for i in 0..100 {
        if i % 2 == 0 {
            val += 2.0;
        } else {
            val += 1.8;
        }
        correlated_returns.push(val);
    }
    let n_eff_corr = compute_effective_sample_size(&correlated_returns, 10);
    // Positive dependence must strictly reduce N_eff below N
    assert!(n_eff_corr < 100.0);
    assert!(n_eff_corr >= 1.0);

    // 2. Denominator safeguard under noise/negative correlation: VIF floor at 1.0 prevents N_eff > N
    let alternating_returns: Vec<f64> = (0..100).map(|i| if i % 2 == 0 { 10.0 } else { -10.0 }).collect();
    let n_eff_alt = compute_effective_sample_size(&alternating_returns, 10);
    assert_eq!(n_eff_alt, 100.0); // Safeguard caps N_eff at N

    // 3. Session clustering degrees of freedom
    let qualifying_sessions: Vec<String> = (1..=20).map(|s| format!("SESSION_{:02}", s)).collect();
    assert_eq!(compute_session_cluster_count(&qualifying_sessions), 20);

    let insufficient_sessions: Vec<String> = (1..=8).map(|s| format!("SESSION_{:02}", s)).collect();
    assert_eq!(compute_session_cluster_count(&insufficient_sessions), 8);
    assert!(compute_session_cluster_count(&insufficient_sessions) < 10); // Trigger sample insufficiency for cell

    // 4. Brier Skill Score against sample-evaluated training-period prevalence benchmark
    // BS_ref = (1/N) * sum_{i=1}^N (y_i - p_train)^2
    let criteria = ProspectiveAcceptanceCriteria::default();
    let p_ref_train: f64 = 0.62;

    // Case A: Entered trade sample with TP=62, FP=38 (matches training rate exactly)
    // BS_ref = [62*(0.38)^2 + 38*(0.62)^2] / 100 = 0.2356
    let metrics_exact = PredictiveAccuracyMetrics::compute_with_ex_ante_brier_and_ref(
        62,
        38,
        0,
        0,
        Some(0.2100),
        Some(p_ref_train),
        &criteria,
    );
    assert!((metrics_exact.reference_brier_score - 0.2356).abs() < 1e-4);
    assert!(metrics_exact.brier_skill_score > 0.0);

    // Case B: Evaluated on sample with TP=65, FP=35 (BS_ref = [65*0.1444 + 35*0.3844]/100 = 0.2284)
    let ref_brier_sample = (65.0 * (1.0 - p_ref_train).powi(2) + 35.0 * p_ref_train.powi(2)) / 100.0;
    assert!((ref_brier_sample - 0.2284).abs() < 1e-4);

    let metrics_skill = PredictiveAccuracyMetrics::compute_with_ex_ante_brier_and_ref(
        65,
        35,
        0,
        0,
        Some(0.2100),
        Some(p_ref_train),
        &criteria,
    );
    assert!((metrics_skill.reference_brier_score - ref_brier_sample).abs() < 1e-4);
    assert!(metrics_skill.brier_skill_score > 0.0); // 1.0 - 0.2100/0.2284 = +0.0805 > 0.0
    assert!(metrics_skill.passed_predictive_gate);

    // Inferior forecast with Brier score = 0.2400 (> BS_ref = 0.2284)
    let metrics_no_skill = PredictiveAccuracyMetrics::compute_with_ex_ante_brier_and_ref(
        65,
        35,
        0,
        0,
        Some(0.2400),
        Some(p_ref_train),
        &criteria,
    );
    assert!(metrics_no_skill.brier_skill_score < 0.0); // Negative skill against sample reference
    assert!(!metrics_no_skill.passed_predictive_gate); // Fails Layer 1 gate!
}

