use std::collections::HashMap;
use crate::ga::{self, GaConfig};
use crate::{MarketEvent, SimEvent};
use crate::market_adapter::{Candle, convert_series_to_events};
pub use crate::exit::ExitReason;
use serde::{Serialize, Deserialize};
use std::path::Path;
use std::env;
use crate::data_source::CandleSource;
use crate::csv_source::CsvCandleSource;
use crate::folder_source::FolderCandleSource;

const VOLATILITY_THRESHOLD: f64 = 0.01;
const TREND_THRESHOLD: f64 = 0.01;
const VOL_NORM_FACTOR: f64 = 0.02;
const DEFAULT_CONFIDENCE_FLOOR: f64 = 0.30;
const DEFAULT_SCORE_FLOOR: f64 = 0.40;

#[derive(Debug)]
pub struct MetricAggregation {
    pub metric: String,
    pub mean: f64,
    pub std_dev: f64,
    pub min: f64,
    pub max: f64,
}

#[derive(Debug, Clone)]
pub struct AssetResult {
    pub asset: String,
    pub participation: f64,
    pub avg_pnl: f64,
    pub weak_executed_count: usize,
    pub score: f64,
}

#[derive(Debug, Clone)]
pub struct StrategyEvaluationDto {
    pub strategy_id: String,
    pub avg: f64,
    pub std: f64,
    pub score: f64,
    pub classification: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SignalAction {
    BUY,
    SELL,
    HOLD,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RecommendationStatus {
    NEW,      // Emitted by engine
    PENDING,  // Waiting for fill
    ACTIVE,   // Position open
    CLOSED,   // Completed
    REJECTED, // Filtered out
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AlphaPorosity {
    Dead,         // capture < 0
    Fragile,      // 0 <= capture < 0.25
    Transitional, // 0.25 <= capture < 0.6
    Live,         // capture >= 0.6
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EntryType {
    MARKET,
    PULLBACK,
    BREAKOUT,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeSignal {
    pub asset: String,
    pub regime: String,
    pub confidence: f64,
    pub action: SignalAction,
    pub entry_type: EntryType,
    pub entry_zone: Option<(f64, f64)>,
    pub stop_loss: Option<f64>,
    pub target: Option<f64>,
    pub expected_edge: f64,
    pub scenario_pnl: f64,
    pub risk_reward: f64,
    pub position_size: f64,
    pub conviction: f64,
    pub composite_score: f64,
    pub reject_reason: Option<String>,
    pub expected_holding_time: String,
    pub strategy_id: String,
    pub reason: String,
    pub status: RecommendationStatus,
    pub porosity: AlphaPorosity,
    pub porosity_trend: f64, // Rate of change in porosity
    pub is_open: bool,
    pub current_pnl: f64,
    pub peak_pnl: f64,
    pub exit_reason: Option<ExitReason>,
    pub rank_score: f64,
    pub rank_position: Option<u32>,
    pub allocated_capital: Option<f64>,
    pub quantity: Option<u64>,
}

#[derive(Debug, Clone, Copy)]
enum RejectReason {
    LowConfidence,
    LowScore,
    NegativeEdge,
    SidewaysRegime,
}

impl RejectReason {
    fn as_str(self) -> &'static str {
        match self {
            RejectReason::LowConfidence => "REJECT_LOW_CONF",
            RejectReason::LowScore => "REJECT_LOW_SCORE",
            RejectReason::NegativeEdge => "REJECT_NEG_EDGE",
            RejectReason::SidewaysRegime => "REJECT_SIDEWAYS",
        }
    }
}

fn edge_reason_from_gate_reject(reject_reason: Option<RejectReason>) -> EdgeLossReason {
    match reject_reason {
        None => EdgeLossReason::Accepted,
        Some(RejectReason::LowConfidence) => EdgeLossReason::LowConfidence,
        Some(RejectReason::LowScore) => EdgeLossReason::LowScore,
        Some(RejectReason::NegativeEdge) => EdgeLossReason::RiskFiltered,
        Some(RejectReason::SidewaysRegime) => EdgeLossReason::SidewaysMarket,
    }
}

#[derive(Debug, Clone, Copy)]
struct GateDecision {
    trade_allowed: bool,
    position_size: f64,
    composite_score: f64,
    reject_reason: Option<RejectReason>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalMeta {
    pub total_assets: usize,
    pub total_scenarios: usize,
    pub trades: usize,
    pub holds: usize,
    pub participation: f64,
    pub edge_loss_breakdown: EdgeLossBreakdown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalsSnapshot {
    pub timestamp: u64,
    pub signals: Vec<TradeSignal>,
    pub meta: SignalMeta,
    pub asset_name: String, // Add asset name to snapshot for context
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum EdgeLossReason {
    NoAggregateEvaluation,
    WeakEvalSurrogate,
    WeakLowVol,
    LowConfidence,
    LowScore,
    SidewaysMarket,
    RiskFiltered,
    QueueTooDeep,
    LowLiquidity,
    HighLatencyImpact,
    Accepted,
}

impl EdgeLossReason {
    fn as_str(self) -> &'static str {
        match self {
            EdgeLossReason::NoAggregateEvaluation => "NO_AGG_EVAL",
            EdgeLossReason::WeakEvalSurrogate => "WEAK_EVAL_SURROGATE",
            EdgeLossReason::WeakLowVol => "WEAK_LOW_VOL",
            EdgeLossReason::LowConfidence => "LOW_CONFIDENCE",
            EdgeLossReason::LowScore => "LOW_SCORE",
            EdgeLossReason::SidewaysMarket => "SIDEWAYS_MARKET",
            EdgeLossReason::RiskFiltered => "RISK_FILTERED",
            EdgeLossReason::QueueTooDeep => "QUEUE_TOO_DEEP",
            EdgeLossReason::LowLiquidity => "LOW_LIQUIDITY",
            EdgeLossReason::HighLatencyImpact => "HIGH_LATENCY_IMPACT",
            EdgeLossReason::Accepted => "ACCEPTED",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct EdgeTransfer {
    pub eval_edge: Option<f64>,
    pub weak_eval_edge: Option<f64>,
    pub has_strong_eval: bool,
    pub signal_edge: f64,
    pub delta: f64,
    pub confidence: f64,
    pub reason: EdgeLossReason,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeLossBreakdown {
    pub total_scenarios: usize,
    pub total_eval_edge: f64,
    pub total_signal_edge: f64,
    pub edge_retention_ratio: f64,
    pub true_edge_retention: f64,
    pub top_loss_reason: Option<String>,
    pub loss_distribution: Vec<ReasonLossShare>,
    pub loss_by_reason: HashMap<String, f64>,
    pub count_by_reason: HashMap<String, usize>,
    pub executed_strong_edge: f64,
    pub executed_weak_edge: f64,
    pub weak_rejected_low_conf: usize,
    pub weak_rejected_low_vol: usize,
    pub weak_executed_count: usize,
    pub transfer_traces: Vec<EdgeTransfer>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasonLossShare {
    pub reason: String,
    pub pct: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThresholdSweepRow {
    pub confidence_floor: f64,
    pub score_floor: f64,
    pub participation: f64,
    pub trades: usize,
    pub total_scenarios: usize,
    pub global_avg_pnl: f64,
    pub traded_avg_pnl: f64,
    pub std_dev: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Regime {
    Volatile,
    TrendingUp,
    TrendingDown,
    Sideways,
    Mixed,
}

impl Regime {
    fn as_str(self) -> &'static str {
        match self {
            Regime::Volatile => "volatile",
            Regime::TrendingUp => "trending_up",
            Regime::TrendingDown => "trending_down",
            Regime::Sideways => "sideways",
            Regime::Mixed => "mixed",
        }
    }
}

fn detect_regime_from_events(events: &[MarketEvent]) -> (Regime, f64) {
    if events.len() < 3 {
        return (Regime::Mixed, 0.0);
    }

    let prices: Vec<f64> = events.iter().map(|e| e.price as f64).collect();
    let first = prices.first().copied().unwrap_or(0.0);
    let last = prices.last().copied().unwrap_or(first);
    if first <= 0.0 {
        return (Regime::Mixed, 0.0);
    }

    let mut returns = Vec::with_capacity(prices.len().saturating_sub(1));
    for i in 1..prices.len() {
        let prev = prices[i - 1];
        let curr = prices[i];
        if prev > 0.0 {
            returns.push((curr / prev) - 1.0);
        }
    }

    let mean_ret = if returns.is_empty() {
        0.0
    } else {
        returns.iter().sum::<f64>() / returns.len() as f64
    };
    let variance = if returns.is_empty() {
        0.0
    } else {
        returns.iter().map(|r| (r - mean_ret).powi(2)).sum::<f64>() / returns.len() as f64
    };
    let volatility = variance.sqrt();
    let trend = (last - first) / first;
    let trend_strength = trend.abs().clamp(0.0, 1.0);
    let vol_strength = (volatility / VOL_NORM_FACTOR).clamp(0.0, 1.0);

    let regime = if volatility > VOLATILITY_THRESHOLD {
        Regime::Volatile
    } else if trend > TREND_THRESHOLD {
        Regime::TrendingUp
    } else if trend < -TREND_THRESHOLD {
        Regime::TrendingDown
    } else {
        Regime::Sideways
    };

    let dominance_denominator = trend_strength + vol_strength + 1e-6;
    let confidence = match regime {
        Regime::TrendingUp | Regime::TrendingDown => {
            trend_strength / dominance_denominator
        }
        Regime::Volatile => {
            vol_strength / dominance_denominator
        }
        Regime::Sideways => {
            1.0 - trend_strength.max(vol_strength)
        }
        Regime::Mixed => 0.0,
    }
    .clamp(0.0, 1.0);

    (regime, confidence)
}

pub fn scenarios_from_candles(asset: &str, candles: &[Candle]) -> HashMap<String, Vec<MarketEvent>> {
    let mut scenarios: HashMap<String, Vec<MarketEvent>> = HashMap::new();
    if candles.len() < 60 {
        return scenarios;
    }

    let window = 120usize.min(candles.len());
    let stride = (window / 2).max(20);
    let mut start = 0usize;
    let mut scenario_id = 0usize;

    while start + window <= candles.len() && scenario_id < 20 {
        let slice = &candles[start..start + window];
        let sim_events = convert_series_to_events(slice, 1);
        let mut market_events: Vec<MarketEvent> = Vec::new();

        for ev in sim_events {
            if let SimEvent::MarketEvent {
                subtype,
                price,
                quantity,
                side,
                timestamp,
                ..
            } = ev {
                market_events.push(MarketEvent {
                    subtype,
                    price,
                    quantity: quantity.max(1),
                    side,
                    exchange_ts: timestamp,
                });
            }
        }

        if !market_events.is_empty() {
            scenarios.insert(format!("{}_csv_window_{}", asset, scenario_id), market_events);
        }

        start += stride;
        scenario_id += 1;
    }

    scenarios
}

fn should_trade(execution_fitness: f64) -> bool {
    execution_fitness.is_finite() && execution_fitness > 0.0
}

fn regime_quality(regime: Regime) -> f64 {
    match regime {
        Regime::TrendingUp | Regime::TrendingDown => 1.0,
        Regime::Volatile => 0.7,
        Regime::Sideways => 0.0,
        Regime::Mixed => 0.4,
    }
}

fn edge_norm(execution_fitness: f64) -> f64 {
    // Calibrated to current bounded execution-fitness scale (~0.00-0.02 typical).
    (execution_fitness / 0.01).clamp(0.0, 1.0)
}

fn tiered_position_size(confidence: f64) -> f64 {
    if confidence > 0.75 {
        1.0
    } else if confidence > 0.65 {
        0.8
    } else if confidence > 0.55 {
        0.6
    } else if confidence > 0.45 {
        0.3
    } else {
        0.0
    }
}

fn evaluate_gate(
    regime: Regime,
    confidence: f64,
    execution_fitness: f64,
    confidence_floor: f64,
    score_floor: f64,
) -> GateDecision {
    if matches!(regime, Regime::Sideways) {
        return GateDecision {
            trade_allowed: false,
            position_size: 0.0,
            composite_score: 0.0,
            reject_reason: Some(RejectReason::SidewaysRegime),
        };
    }
    if confidence < confidence_floor {
        return GateDecision {
            trade_allowed: false,
            position_size: 0.0,
            composite_score: 0.0,
            reject_reason: Some(RejectReason::LowConfidence),
        };
    }
    if !should_trade(execution_fitness) {
        return GateDecision {
            trade_allowed: false,
            position_size: 0.0,
            composite_score: 0.0,
            reject_reason: Some(RejectReason::NegativeEdge),
        };
    }

    let score =
        0.5 * confidence + 0.3 * edge_norm(execution_fitness) + 0.2 * regime_quality(regime);
    if score < score_floor {
        return GateDecision {
            trade_allowed: false,
            position_size: 0.0,
            composite_score: score,
            reject_reason: Some(RejectReason::LowScore),
        };
    }

    GateDecision {
        trade_allowed: true,
        position_size: tiered_position_size(confidence),
        composite_score: score,
        reject_reason: None,
    }
}

fn compute_atr(events: &[MarketEvent]) -> f64 {
    if events.len() < 2 {
        return 0.0;
    }
    let mut true_ranges: Vec<f64> = Vec::with_capacity(events.len() - 1);
    for i in 1..events.len() {
        let prev = events[i - 1].price as f64;
        let curr = events[i].price as f64;
        true_ranges.push((curr - prev).abs());
    }
    true_ranges.iter().sum::<f64>() / true_ranges.len() as f64
}

fn build_trade_signal(
    asset: &str,
    regime: Regime,
    confidence: f64,
    selected_strategy_id: &str,
    report_fitness: f64,
    report_pnl: f64,
    gate: GateDecision,
    last_price: f64,
    atr: f64,
) -> TradeSignal {
    let trade_allowed = gate.trade_allowed;
    let (action, entry_type, entry_zone, stop_loss, target, expected_holding_time, reason) = if trade_allowed {
        match regime {
            Regime::TrendingUp => {
                let entry_zone = (last_price * 0.996, last_price * 1.000);
                let effective_atr = atr.max(last_price * 0.003);
                let stop_loss = last_price - (1.5 * effective_atr);
                let risk = (last_price - stop_loss).abs();
                let target = last_price + (2.0 * risk);
                assert!(target > entry_zone.1, "Target must exceed entry zone for BUY");
                assert!(stop_loss < entry_zone.0, "Stop loss must be below entry zone for BUY");
                (
                    SignalAction::BUY,
                    EntryType::PULLBACK,
                    Some(entry_zone),
                    Some(stop_loss),
                    Some(target),
                    "30m-2h".to_string(),
                    format!(
                        "Regime={} with confidence {:.2}, execution fitness {:.2}",
                        regime.as_str(),
                        confidence,
                        report_fitness
                    ),
                )
            }
            Regime::TrendingDown => {
                let entry_zone = (last_price * 1.000, last_price * 1.004);
                let effective_atr = atr.max(last_price * 0.003);
                let stop_loss = last_price + (1.5 * effective_atr);
                let risk = (stop_loss - last_price).abs();
                let target = last_price - (2.0 * risk);
                assert!(target < entry_zone.0, "Target must be below entry zone for SELL");
                assert!(stop_loss > entry_zone.1, "Stop loss must be above entry zone for SELL");
                (
                    SignalAction::SELL,
                    EntryType::PULLBACK,
                    Some(entry_zone),
                    Some(stop_loss),
                    Some(target),
                    "30m-2h".to_string(),
                    format!(
                        "Regime={} with confidence {:.2}, execution fitness {:.2}",
                        regime.as_str(),
                        confidence,
                        report_fitness
                    ),
                )
            }
            Regime::Sideways => (
                SignalAction::HOLD,
                EntryType::MARKET,
                None,
                None,
                None,
                "N/A".to_string(),
                "Low directional edge (sideways regime)".to_string(),
            ),
            _ => (
                SignalAction::HOLD,
                EntryType::MARKET,
                None,
                None,
                None,
                "N/A".to_string(),
                "Non-directional regime routed to HOLD for phase-1 safety".to_string(),
            ),
        }   
    } else {
        let reject_code = gate
            .reject_reason
            .map(|r| r.as_str())
            .unwrap_or("REJECT_UNKNOWN");
        (
            SignalAction::HOLD,
            EntryType::MARKET,
            None,
            None,
            None,
            "N/A".to_string(),
            format!("Rejected by gate: {}", reject_code),
        )
    };

    let risk_reward = match (entry_zone, stop_loss, target, &action) {
        (Some((entry_lo, entry_hi)), Some(sl), Some(tp), SignalAction::BUY) => {
            let entry = (entry_lo + entry_hi) * 0.5;
            let risk = (entry - sl).max(1e-9);
            let reward = (tp - entry).max(0.0);
            reward / risk
        }
        (Some((entry_lo, entry_hi)), Some(sl), Some(tp), SignalAction::SELL) => {
            let entry = (entry_lo + entry_hi) * 0.5;
            let risk = (sl - entry).max(1e-9);
            let reward = (entry - tp).max(0.0);
            reward / risk
        }
        _ => 0.0,
    };
    if matches!(action, SignalAction::BUY | SignalAction::SELL) {
        assert!(risk_reward >= 1.5, "Risk-reward below minimum threshold: {}", risk_reward);
    }

    TradeSignal {
        asset: asset.to_string(),
        regime: regime.as_str().to_string(),
        confidence,
        action,
        entry_type,
        entry_zone,
        stop_loss,
        target,
        expected_edge: report_fitness,
        scenario_pnl: if trade_allowed { report_pnl } else { 0.0 },
        risk_reward,
        position_size: if trade_allowed { gate.position_size } else { 0.0 },
        conviction: confidence * report_fitness.max(0.0),
        composite_score: gate.composite_score,
        reject_reason: gate.reject_reason.map(|r| r.as_str().to_string()),
        expected_holding_time,
        strategy_id: selected_strategy_id.to_string(),
        reason,
        status: RecommendationStatus::NEW,
        porosity: AlphaPorosity::Live, // Placeholder, usually updated by registry
        porosity_trend: 0.0,
        is_open: trade_allowed,
        current_pnl: 0.0,
        peak_pnl: 0.0,
        exit_reason: None,
        rank_score: 0.0,
        rank_position: None,
        allocated_capital: None,
        quantity: None,
    }
}

pub fn generate_latest_signals(
    assets: Vec<String>,
    global_lambda: f64,
) -> SignalsSnapshot {
    generate_latest_signals_with_thresholds(
        assets,
        global_lambda,
        DEFAULT_CONFIDENCE_FLOOR,
        DEFAULT_SCORE_FLOOR,
    )
}

pub fn generate_latest_signals_with_thresholds(
    assets: Vec<String>,
    global_lambda: f64,
    confidence_floor: f64,
    score_floor: f64,
) -> SignalsSnapshot {
    let mut all_signals: Vec<TradeSignal> = Vec::new();
    let mut max_timestamp = 0u64;
    let mut total_scenarios = 0usize;
    let mut trade_count = 0usize;
    let mut edge_transfers: Vec<EdgeTransfer> = Vec::new();
    let mut regime_eval_sums: HashMap<String, f64> = HashMap::new();
    let mut regime_eval_counts: HashMap<String, usize> = HashMap::new();
    let mut max_observed_eval_edge = 0.001_f64;
    let mut weak_rejected_low_conf = 0usize;
    let mut weak_rejected_low_vol = 0usize;
    let mut weak_executed_count = 0usize;

    for asset_name in &assets {
        let base_seed = 42;
        let base_price = 40000;
        let scenario_map = crate::synthetic::generate_deterministic_scenarios(asset_name, base_seed, base_price);
        if scenario_map.is_empty() {
            continue;
        }
        let first_scenario_events = scenario_map.values().next().expect("No scenarios generated");
        let initial_price = first_scenario_events.first().map(|e| e.price).unwrap_or(base_price);
        let initial_timestamp = first_scenario_events.first().map(|e| e.exchange_ts).unwrap_or(0);

        let mut config = GaConfig::default();
        config.population_size = 5;
        config.generations = 3;
        config.mutation_rate = 0.1;
        config.seed = 42;
        config.order_id_prefix = format!("SIGNAL_GA_{}", asset_name);
        config.order_price = initial_price;
        config.order_quantity_for_strategy = 100;
        config.order_timestamp = initial_timestamp;
        config.lambda = global_lambda;
        config.initial_queue_threshold = 200;

        let mut sorted_names: Vec<String> = scenario_map.keys().cloned().collect();
        sorted_names.sort();
        total_scenarios += sorted_names.len();

        let test_index = (config.seed as usize) % sorted_names.len();
        let test_scenario_name = sorted_names[test_index].clone();
        let mut train_scenarios: Vec<ga::ScenarioPair<'_>> = Vec::new();
        for name in &sorted_names {
            if *name != test_scenario_name {
                if let Some(events) = scenario_map.get(name) {
                    train_scenarios.push(ga::ScenarioPair {
                        name,
                        signal_symbol: asset_name,
                        execution_symbol: asset_name,
                        signal: events.as_slice(),
                        execution: events.as_slice(),
                    });
                }
            }
        }

        let global_state = ga::GlobalEvoState::default();
        let (ga_result, _asset_states) = ga::run_ga_evolution(config.clone(), &train_scenarios, &global_state);

        for scenario_name in &sorted_names {
            if let Some(events) = scenario_map.get(scenario_name) {
                max_timestamp = max_timestamp.max(events.last().map(|e| e.exchange_ts).unwrap_or(0));
                let (detected_regime, confidence) = detect_regime_from_events(events.as_slice());
                let regime_key = format!("{}_{}", asset_name, detected_regime.as_str());
                let selected_eval = ga_result
                    .best_per_regime
                    .get(&regime_key)
                    .unwrap_or(&ga_result.global_best);
                let one_scenario = [ga::ScenarioPair {
                    name: scenario_name,
                    signal_symbol: asset_name,
                    execution_symbol: asset_name,
                    signal: events.as_slice(),
                    execution: events.as_slice(),
                }];
                if let Some(report) = ga::evaluate_and_aggregate(
                    &selected_eval.strategy,
                    &config,
                    &one_scenario,
                    0,
                    0.0,
                    0,
                    1.0,
                    0,
                ) {
                    max_observed_eval_edge = max_observed_eval_edge.max(report.fitness.max(0.0));
                    let gate = evaluate_gate(
                        detected_regime,
                        confidence,
                        report.fitness,
                        confidence_floor,
                        score_floor,
                    );
                    let effective_eval_edge = report.fitness;
                    let normalized_edge =
                        (effective_eval_edge / max_observed_eval_edge.max(1e-9)).clamp(0.1, 1.0);
                    let scaled_size = normalized_edge;
                    let mut effective_gate = gate;
                    if effective_gate.trade_allowed {
                        effective_gate.position_size = scaled_size;
                        trade_count += 1;
                    }
                    let transfer_reason = effective_gate
                        .reject_reason
                        .map(|r| r.as_str())
                        .unwrap_or("EXECUTE");
                    let executable_edge = if effective_gate.trade_allowed {
                        effective_eval_edge
                    } else {
                        0.0
                    };
                    let regime_name = detected_regime.as_str().to_string();
                    *regime_eval_sums.entry(regime_name.clone()).or_insert(0.0) += report.fitness;
                    *regime_eval_counts.entry(regime_name).or_insert(0usize) += 1;
                    edge_transfers.push(EdgeTransfer {
                        eval_edge: Some(report.fitness),
                        weak_eval_edge: None,
                        has_strong_eval: true,
                        signal_edge: executable_edge,
                        delta: (report.fitness - executable_edge).max(0.0),
                        confidence,
                        reason: edge_reason_from_gate_reject(effective_gate.reject_reason),
                    });
                    println!(
                        "EDGE_TRANSFER_DEBUG → asset={} scenario={} regime={} eval_edge={:.6} weak_eval_edge=NA effective_eval_edge={:.6} has_strong_eval=true effective_conf_floor={:.3} execution_source=STRONG signal_edge={:.6} delta={:.6} confidence={:.3} decision={} reason={}",
                        asset_name,
                        scenario_name,
                        detected_regime.as_str(),
                        report.fitness,
                        effective_eval_edge,
                        confidence_floor,
                        executable_edge,
                        executable_edge - report.fitness,
                        confidence,
                        if effective_gate.trade_allowed { "TRADE" } else { "HOLD" },
                        transfer_reason
                    );
                    let last_price = events.last().map(|e| e.price as f64).unwrap_or(config.order_price as f64);
                    let atr = compute_atr(events.as_slice());
                    let signal = build_trade_signal(
                        asset_name,
                        detected_regime,
                        confidence,
                        &selected_eval.strategy_id,
                        effective_eval_edge,
                        report.avg_pnl,
                        effective_gate,
                        last_price,
                        atr,
                    );
                    println!(
                        "SIGNAL_GENERATED → asset={} scenario={} regime={} confidence={:.3} score={:.3} size={:.2} action={:?} reason={} strategy={} edge={:.6}",
                        asset_name,
                        scenario_name,
                        signal.regime,
                        signal.confidence,
                        signal.composite_score,
                        signal.position_size,
                        signal.action,
                        signal.reject_reason.as_deref().unwrap_or("EXECUTE"),
                        signal.strategy_id,
                        signal.expected_edge
                    );
                    all_signals.push(signal);
                } else {
                    let regime_name = detected_regime.as_str().to_string();
                    let avg_eval_edge_per_regime = if let (Some(sum), Some(count)) = (
                        regime_eval_sums.get(&regime_name),
                        regime_eval_counts.get(&regime_name),
                    ) {
                        if *count > 0 {
                            *sum / *count as f64
                        } else {
                            0.001
                        }
                    } else {
                        0.001
                    };
                    let weak_eval_edge =
                        (confidence * avg_eval_edge_per_regime.clamp(0.0, 1.0)).clamp(0.0, 1.0);
                    let effective_eval_edge = weak_eval_edge;
                    let effective_conf_floor = (confidence_floor * 0.75).max(0.45);
                    let first_price = events.first().map(|e| e.price as f64).unwrap_or(0.0);
                    let last_price_for_move = events.last().map(|e| e.price as f64).unwrap_or(first_price);
                    let move_abs = (last_price_for_move - first_price).abs();
                    let min_move = if first_price > 0.0 { first_price * 0.001 } else { 0.0 };
                    let low_volatility = detected_regime == Regime::Sideways || (first_price > 0.0 && move_abs < min_move);
                    let weak_execution_allowed = !low_volatility && confidence >= effective_conf_floor;
                    let normalized_edge =
                        (effective_eval_edge / max_observed_eval_edge.max(1e-9)).clamp(0.1, 1.0);
                    let weak_scaled_size = normalized_edge.min(0.5);
                    let weak_gate = GateDecision {
                        trade_allowed: weak_execution_allowed,
                        position_size: if weak_execution_allowed { weak_scaled_size } else { 0.0 },
                        composite_score: 0.0,
                        reject_reason: if weak_execution_allowed {
                            None
                        } else {
                            Some(RejectReason::LowConfidence)
                        },
                    };
                    let last_price =
                        events.last().map(|e| e.price as f64).unwrap_or(config.order_price as f64);
                    let atr = compute_atr(events.as_slice());
                    let signal = build_trade_signal(
                        asset_name,
                        detected_regime,
                        confidence,
                        &selected_eval.strategy_id,
                        effective_eval_edge,
                        0.0,
                        weak_gate,
                        last_price,
                        atr,
                    );
                    if signal.action != SignalAction::HOLD {
                        trade_count += 1;
                        weak_executed_count += 1;
                    } else if low_volatility {
                        weak_rejected_low_vol += 1;
                    } else if !weak_execution_allowed {
                        weak_rejected_low_conf += 1;
                    }
                    let weak_reason_tag = if weak_gate.trade_allowed {
                        "EXECUTE_WEAK_SURROGATE"
                    } else if low_volatility {
                        "WEAK_LOW_VOL"
                    } else {
                        "WEAK_EVAL_SURROGATE_REJECT_LOW_CONF"
                    };
                    edge_transfers.push(EdgeTransfer {
                        eval_edge: None,
                        weak_eval_edge: Some(weak_eval_edge),
                        has_strong_eval: false,
                        signal_edge: if weak_gate.trade_allowed {
                            effective_eval_edge
                        } else {
                            0.0
                        },
                        delta: (effective_eval_edge
                            - if weak_gate.trade_allowed {
                                effective_eval_edge
                            } else {
                                0.0
                            })
                        .max(0.0),
                        confidence,
                        reason: if low_volatility { EdgeLossReason::WeakLowVol } else { EdgeLossReason::WeakEvalSurrogate },
                    });
                    println!(
                        "EDGE_TRANSFER_DEBUG → asset={} scenario={} regime={} eval_edge=NA weak_eval_edge={:.6} effective_eval_edge={:.6} has_strong_eval=false effective_conf_floor={:.3} execution_source=WEAK low_volatility={} execution_blocked_by={} signal_edge={:.6} delta={:.6} confidence={:.3} decision={} reason={}",
                        asset_name,
                        scenario_name,
                        detected_regime.as_str(),
                        weak_eval_edge,
                        effective_eval_edge,
                        effective_conf_floor,
                        low_volatility,
                        if low_volatility { "LOW_VOL" } else { "NONE" },
                        if weak_gate.trade_allowed {
                            effective_eval_edge
                        } else {
                            0.0
                        },
                        (effective_eval_edge
                            - if weak_gate.trade_allowed {
                                effective_eval_edge
                            } else {
                                0.0
                            })
                        .max(0.0),
                        confidence
                        ,
                        if weak_gate.trade_allowed { "TRADE" } else { "HOLD" },
                        weak_reason_tag
                    );
                    println!(
                        "SIGNAL_GENERATED → asset={} scenario={} regime={} confidence={:.3} action={:?} strategy={} edge={:.6}",
                        asset_name,
                        scenario_name,
                        signal.regime,
                        signal.confidence,
                        signal.action,
                        signal.strategy_id,
                        signal.expected_edge
                    );
                    all_signals.push(signal);
                }
            }
        }
    }

    let holds = all_signals.iter().filter(|s| s.action == SignalAction::HOLD).count();
    let participation = if total_scenarios == 0 {
        0.0
    } else {
        trade_count as f64 / total_scenarios as f64
    };
    let total_eval_edge = edge_transfers
        .iter()
        .map(|e| e.eval_edge.unwrap_or(0.0))
        .sum::<f64>();
    let total_effective_eval_edge = edge_transfers
        .iter()
        .map(|e| e.eval_edge.or(e.weak_eval_edge).unwrap_or(0.0))
        .sum::<f64>();
    let total_signal_edge = edge_transfers.iter().map(|e| e.signal_edge).sum::<f64>();
    let estimated_missing_eval_edge = edge_transfers
        .iter()
        .map(|e| e.weak_eval_edge.unwrap_or(0.0))
        .sum::<f64>();
    let estimated_total_possible_eval_edge = total_eval_edge + estimated_missing_eval_edge;
    let edge_retention_ratio = if total_eval_edge > 0.0 {
        (total_signal_edge / total_eval_edge).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let true_edge_retention = if estimated_total_possible_eval_edge > 0.0 {
        (total_signal_edge / estimated_total_possible_eval_edge).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let mut loss_by_reason: HashMap<String, f64> = HashMap::new();
    let mut count_by_reason: HashMap<String, usize> = HashMap::new();
    let executed_strong_edge = edge_transfers
        .iter()
        .filter(|t| t.has_strong_eval && t.signal_edge > 0.0)
        .map(|t| t.signal_edge)
        .sum::<f64>();
    let executed_weak_edge = edge_transfers
        .iter()
        .filter(|t| !t.has_strong_eval && t.signal_edge > 0.0)
        .map(|t| t.signal_edge)
        .sum::<f64>();
    for transfer in &edge_transfers {
        let key = transfer.reason.as_str().to_string();
        *count_by_reason.entry(key.clone()).or_insert(0usize) += 1;
        *loss_by_reason.entry(key).or_insert(0.0) += transfer.delta;
    }
    let total_loss: f64 = loss_by_reason.values().sum();
    let mut loss_distribution: Vec<ReasonLossShare> = if total_loss > 0.0 {
        let mut rows: Vec<ReasonLossShare> = loss_by_reason
            .iter()
            .map(|(reason, loss)| ReasonLossShare {
                reason: reason.clone(),
                pct: ((*loss / total_loss) * 100.0).clamp(0.0, 100.0),
            })
            .collect();
        rows.sort_by(|a, b| b.pct.partial_cmp(&a.pct).unwrap_or(std::cmp::Ordering::Equal));
        rows
    } else {
        Vec::new()
    };
    let top_loss_reason = loss_distribution.first().map(|r| r.reason.clone());
    let edge_loss_breakdown = EdgeLossBreakdown {
        total_scenarios,
        total_eval_edge: total_effective_eval_edge,
        total_signal_edge,
        edge_retention_ratio,
        true_edge_retention,
        top_loss_reason,
        loss_distribution: std::mem::take(&mut loss_distribution),
        loss_by_reason,
        count_by_reason,
        executed_strong_edge,
        executed_weak_edge,
        weak_rejected_low_conf,
        weak_rejected_low_vol,
        weak_executed_count,
        transfer_traces: edge_transfers.clone(),
    };

    SignalsSnapshot {
        timestamp: max_timestamp,
        signals: all_signals,
        meta: SignalMeta {
            total_assets: assets.len(),
            total_scenarios,
            trades: trade_count,
            holds,
            participation,
            edge_loss_breakdown,
        },
        asset_name: assets.first().cloned().unwrap_or_else(|| "UNKNOWN".to_string()),
    }
}

pub fn run_threshold_sweep(
    assets: Vec<String>,
    global_lambda: f64,
    confidence_floors: &[f64],
    score_floors: &[f64],
) -> Vec<ThresholdSweepRow> {
    let mut rows: Vec<ThresholdSweepRow> = Vec::new();
    for &confidence_floor in confidence_floors {
        for &score_floor in score_floors {
            let snapshot = generate_latest_signals_with_thresholds(
                assets.clone(),
                global_lambda,
                confidence_floor,
                score_floor,
            );
            let pnls: Vec<f64> = snapshot.signals.iter().map(|s| s.scenario_pnl).collect();
            let global_avg = if pnls.is_empty() {
                0.0
            } else {
                pnls.iter().sum::<f64>() / pnls.len() as f64
            };
            let variance = if pnls.is_empty() {
                0.0
            } else {
                pnls.iter().map(|p| (p - global_avg).powi(2)).sum::<f64>() / pnls.len() as f64
            };
            let traded: Vec<f64> = snapshot
                .signals
                .iter()
                .filter(|s| s.action != SignalAction::HOLD)
                .map(|s| s.scenario_pnl)
                .collect();
            let traded_avg = if traded.is_empty() {
                0.0
            } else {
                traded.iter().sum::<f64>() / traded.len() as f64
            };
            rows.push(ThresholdSweepRow {
                confidence_floor,
                score_floor,
                participation: snapshot.meta.participation,
                trades: snapshot.meta.trades,
                total_scenarios: snapshot.meta.total_scenarios,
                global_avg_pnl: global_avg,
                traded_avg_pnl: traded_avg,
                std_dev: variance.sqrt(),
            });
        }
    }

    rows.sort_by(|a, b| {
        let a_in_band = (0.15..=0.30).contains(&a.participation);
        let b_in_band = (0.15..=0.30).contains(&b.participation);
        match (a_in_band, b_in_band) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => b
                .global_avg_pnl
                .partial_cmp(&a.global_avg_pnl)
                .unwrap_or(std::cmp::Ordering::Equal),
        }
    });

    rows
}

pub fn evaluate_on_real_data(
    assets: Vec<(String, String)>,
    global_lambda: f64,
) -> Vec<MetricAggregation> {
    let mut aggregated_metrics: Vec<MetricAggregation> = Vec::new();
    let mut folder_asset_metrics: Vec<AssetResult> = Vec::new();
    let data_source = env::var("DATA_SOURCE")
        .unwrap_or_else(|_| "synthetic".to_string())
        .to_lowercase();

    let folder_path = "/Users/nikhil/ChronoSentiment_MEGA_FINAL/test_assets".to_string();
    let mut folder_candles_by_asset: HashMap<String, Vec<Candle>> = HashMap::new();

    let assets_to_process: Vec<(String, String)> = if data_source == "folder" {
        println!("DATA_SOURCE=FOLDER");
        println!("folder_path={}", folder_path);
        let source = FolderCandleSource { folder_path };
        let datasets = source.load_all();
        println!("dataset_count={}", datasets.len());
        for (asset, candles) in datasets {
            folder_candles_by_asset.insert(asset.clone(), candles);
        }
        let mut assets_list: Vec<(String, String)> = folder_candles_by_asset
            .keys()
            .cloned()
            .map(|asset| (asset, String::new()))
            .collect();
        assets_list.sort_by(|a, b| a.0.cmp(&b.0));
        assets_list
    } else {
        assets
    };

    for (asset_name, csv_path) in assets_to_process {
        println!("=== START ASSET: {} ===", asset_name);

        let base_seed = 42;
        let base_price = 40000;

        let scenario_map = if data_source == "folder" {
            let candles = folder_candles_by_asset
                .get(&asset_name)
                .cloned()
                .unwrap_or_default();
            println!("Processing asset: {} ({} candles)", asset_name, candles.len());
            let scenarios = scenarios_from_candles(&asset_name, &candles);
            if scenarios.is_empty() {
                crate::synthetic::generate_deterministic_scenarios(&asset_name, base_seed, base_price)
            } else {
                scenarios
            }
        } else if data_source == "csv" && !csv_path.is_empty() && Path::new(&csv_path).exists() {
            println!("DATA_SOURCE=CSV asset={} path={}", asset_name, csv_path);
            let source: Box<dyn CandleSource> = Box::new(CsvCandleSource { path: csv_path.clone() });
            let candles = source.get_candles();
            println!("Loaded {} candles for asset {}", candles.len(), asset_name);
            let scenarios = scenarios_from_candles(&asset_name, &candles);
            if scenarios.is_empty() {
                crate::synthetic::generate_deterministic_scenarios(&asset_name, base_seed, base_price)
            } else {
                scenarios
            }
        } else {
            println!("DATA_SOURCE=SYNTHETIC asset={}", asset_name);
            crate::synthetic::generate_deterministic_scenarios(&asset_name, base_seed, base_price)
        };
        
        let first_scenario_events = scenario_map.values().next().expect("No scenarios generated");
        let initial_price = first_scenario_events.first().map(|e| e.price).unwrap_or(base_price);
        let initial_timestamp = first_scenario_events.first().map(|e| e.exchange_ts).unwrap_or(0);
        
        if scenario_map.is_empty() { continue; }
        
        let mut config = GaConfig::default();
        config.population_size = 5;
        config.generations = 3;
        config.mutation_rate = 0.1;
        config.seed = 42;
        config.order_id_prefix = format!("REAL_GA_{}", asset_name);
        config.order_price = initial_price;
        config.order_quantity_for_strategy = 100;
        config.order_timestamp = initial_timestamp;
        config.lambda = global_lambda;
        config.initial_queue_threshold = 200;
        
        // 3. Train/Test Split
        let mut sorted_names: Vec<String> = scenario_map.keys().cloned().collect();
        sorted_names.sort();
        
        let test_index = (config.seed as usize) % sorted_names.len();
        let test_scenario_name = sorted_names[test_index].clone();
        
        let mut train_scenarios: Vec<ga::ScenarioPair<'_>> = Vec::new();
        for name in &sorted_names {
            if *name != test_scenario_name {
                if let Some(events) = scenario_map.get(name) {
                    train_scenarios.push(ga::ScenarioPair {
                        name,
                        signal_symbol: &asset_name,
                        execution_symbol: &asset_name,
                        signal: events.as_slice(),
                        execution: events.as_slice(),
                    });
                }
            }
        }
        
        // 4. Run GA on Train Scenarios Only
        let global_state = ga::GlobalEvoState::default();
        let (ga_result, _asset_states) = ga::run_ga_evolution(config.clone(), &train_scenarios, &global_state);
        
        // 5. Evaluate with runtime regime routing + NoTrade gate.
        let global_strategy = &ga_result.global_best.strategy;
        println!(
            "\nGlobal Fallback Strategy: (Threshold: {}, BaseEdge: {}, TP: {}, SL: {})",
            global_strategy.queue_threshold,
            global_strategy.base_edge,
            global_strategy.take_profit,
            global_strategy.stop_loss
        );

        // Robustness guard: evaluate portfolio-level behavior across ALL scenarios.
        // No-trade scenarios contribute 0.0 pnl to prevent selective-sampling inflation.
        let mut pnls_all = Vec::with_capacity(sorted_names.len());
        let mut execution_fitnesses_all = Vec::with_capacity(sorted_names.len());
        let mut traded_pnls = Vec::with_capacity(sorted_names.len());
        let mut traded_scenarios = 0usize;
        let mut weak_executed_count = 0usize;
        let mut edge_positive_count = 0usize;
        let mut edge_zero_count = 0usize;
        let mut edge_negative_count = 0usize;
        for name in &sorted_names {
            if let Some(events) = scenario_map.get(name) {
                let (detected_regime, confidence) = detect_regime_from_events(events.as_slice());
                let regime_key = format!("{}_{}", asset_name, detected_regime.as_str());
                let selected_eval = ga_result
                    .best_per_regime
                    .get(&regime_key)
                    .unwrap_or(&ga_result.global_best);

                let one_scenario = [ga::ScenarioPair {
                    name,
                    signal_symbol: &asset_name,
                    execution_symbol: &asset_name,
                    signal: events.as_slice(),
                    execution: events.as_slice(),
                }];
                if let Some(report) = ga::evaluate_and_aggregate(
                    &selected_eval.strategy,
                    &config,
                    &one_scenario,
                    0,
                    0.0,
                    0,
                    1.0,
                    0,
                ) {
                    if report.fitness > 0.0 {
                        edge_positive_count += 1;
                    } else if report.fitness < 0.0 {
                        edge_negative_count += 1;
                    } else {
                        edge_zero_count += 1;
                    }
                    let gate = evaluate_gate(
                        detected_regime,
                        confidence,
                        report.fitness,
                        DEFAULT_CONFIDENCE_FLOOR,
                        DEFAULT_SCORE_FLOOR,
                    );
                    println!(
                        "ROUTING_DECISION → scenario={} regime={} confidence={:.3} score={:.3} size={:.2} edge={:.6} selected={} trade={} reason={}",
                        name,
                        detected_regime.as_str(),
                        confidence,
                        gate.composite_score,
                        gate.position_size,
                        report.fitness,
                        selected_eval.strategy_id,
                        if gate.trade_allowed { "YES" } else { "NO" },
                        gate.reject_reason.map(|r| r.as_str()).unwrap_or("EXECUTE")
                    );

                    if gate.trade_allowed {
                        traded_scenarios += 1;
                        if confidence < 0.60 {
                            weak_executed_count += 1;
                        }
                        traded_pnls.push(report.avg_pnl);
                        pnls_all.push(report.avg_pnl);
                        execution_fitnesses_all.push(report.fitness);
                        println!(
                            "  * Scenario {} Regime={} Strategy={} Trade=YES Fitness={:.6} PnL={:.6}",
                            name,
                            detected_regime.as_str(),
                            selected_eval.strategy_id,
                            report.fitness,
                            report.avg_pnl
                        );
                    } else {
                        pnls_all.push(0.0);
                        execution_fitnesses_all.push(0.0);
                        println!(
                            "  * Scenario {} Regime={} Strategy={} Trade=NO (NoTrade gate) Fitness={:.6} Confidence={:.3}",
                            name,
                            detected_regime.as_str(),
                            selected_eval.strategy_id,
                            report.fitness,
                            confidence
                        );
                    }
                }
            } else {
                pnls_all.push(0.0);
                execution_fitnesses_all.push(0.0);
                edge_zero_count += 1;
            }
        }
        
        if !pnls_all.is_empty() {
            let mean_pnl = pnls_all.iter().sum::<f64>() / pnls_all.len() as f64;
            let pnl_variance = pnls_all
                .iter()
                .map(|p| (p - mean_pnl).powi(2))
                .sum::<f64>()
                / pnls_all.len() as f64;
            let std_dev = pnl_variance.sqrt();
            let worst = pnls_all
                .iter()
                .copied()
                .fold(f64::INFINITY, f64::min);
            let mean_execution_fitness =
                execution_fitnesses_all.iter().sum::<f64>() / execution_fitnesses_all.len() as f64;
            let traded_avg_pnl = if traded_pnls.is_empty() {
                0.0
            } else {
                traded_pnls.iter().sum::<f64>() / traded_pnls.len() as f64
            };
            assert!(
                mean_execution_fitness.is_finite() &&
                mean_execution_fitness >= 0.0 &&
                mean_execution_fitness <= 1.0,
                "Pipeline produced invalid fitness: {}",
                mean_execution_fitness
            );

            println!("DEBUG_GLOBAL_AVG: {:.8}", mean_pnl);
            println!("DEBUG_TRADED_AVG: {:.8}", traded_avg_pnl);
            println!(
                "EDGE_DEBUG: total_scenarios={} edge_positive={} edge_zero={} edge_negative={}",
                sorted_names.len(),
                edge_positive_count,
                edge_zero_count,
                edge_negative_count
            );
            println!("  * Avg: {:.6}", mean_pnl);
            println!("  * Std Dev: {:.6}", std_dev);
            println!("  * Worst: {:.6}", worst);
            println!("  * Traded Scenarios: {}/{}", traded_scenarios, sorted_names.len());
            println!(
                "FINAL_PIPELINE_CHECK → fitness={:.6}, trades={}, participation={:.2}",
                mean_execution_fitness,
                traded_scenarios,
                traded_scenarios as f64 / sorted_names.len() as f64
            );

            let participation_rate = traded_scenarios as f64 / sorted_names.len() as f64;

            if data_source == "folder" {
                folder_asset_metrics.push(AssetResult {
                    asset: asset_name.clone(),
                    participation: participation_rate,
                    avg_pnl: mean_pnl,
                    weak_executed_count,
                    score: 0.0,
                });
            }

            aggregated_metrics.push(MetricAggregation {
                metric: format!("PnL_Asset_{}", asset_name), // Using asset_name to differentiate
                mean: mean_pnl,
                std_dev,
                min: worst,
                max: mean_pnl,
            });

            // 6. Generate report for routed execution path.
            println!("\n------------------------------------------------");
            println!("🚀 FINAL ROUTED STRATEGY EVALUATION - {}", asset_name);
            println!("------------------------------------------------");
            println!("  Fitness: {:.4}", mean_execution_fitness);
            println!("  Avg PnL: {:.6}", mean_pnl);
            println!("  Std Dev: {:.6}", std_dev);
            println!("  Worst PnL: {:.6}", worst);
            println!("  Traded Scenarios: {}", traded_scenarios);
            println!("  Participation Rate: {:.2}", participation_rate);
            println!("------------------------------------------------");
            println!("ASSET_METRICS {} -> candles {} | participation {:.2} | avg_pnl {:.6}", asset_name, scenario_map.len(), participation_rate, mean_pnl);
            println!("=== END ASSET: {} ===", asset_name);
        } else {
            println!("DEBUG: NoTrade gate rejected all scenarios for {}", asset_name);
            aggregated_metrics.push(MetricAggregation {
                metric: format!("PnL_Asset_{}", asset_name),
                mean: 0.0,
                std_dev: 0.0,
                min: 0.0,
                max: 0.0,
            });
            println!("ASSET_METRICS {} -> candles {} | participation 0.00 | avg_pnl 0.000000", asset_name, scenario_map.len());
            println!("=== END ASSET: {} ===", asset_name);
        }
    }

    if data_source == "folder" && !folder_asset_metrics.is_empty() {
        let max_weak = folder_asset_metrics
            .iter()
            .map(|r| r.weak_executed_count)
            .max()
            .unwrap_or(0);
        let weak_norm_den = if max_weak == 0 { 1.0 } else { max_weak as f64 };

        for result in &mut folder_asset_metrics {
            let weak_norm = result.weak_executed_count as f64 / weak_norm_den;
            result.score = 0.5 * result.avg_pnl + 0.3 * result.participation + 0.2 * weak_norm;
        }

        folder_asset_metrics.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.asset.cmp(&b.asset))
        });

        println!("=== TOP ASSETS ===");
        for (idx, result) in folder_asset_metrics.iter().take(3).enumerate() {
            println!(
                "{}. {} -> score {:.4} | pnl {:.4} | participation {:.2}",
                idx + 1,
                result.asset,
                result.score,
                result.avg_pnl,
                result.participation
            );
        }
    }

    aggregated_metrics
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MarketEventType;

    #[test]
    fn test_pipeline_final_uses_aggregate() {
        let assets = vec![("BTC".to_string(), "".to_string())];
        let metrics = evaluate_on_real_data(assets, 0.5);
        assert!(
            !metrics.is_empty(),
            "Pipeline produced no metrics — aggregation or routing likely skipped"
        );
    }

    #[test]
    fn test_detect_regime_is_deterministic() {
        let events = vec![
            MarketEvent { subtype: MarketEventType::Trade, price: 100, quantity: 1, side: None, exchange_ts: 1 },
            MarketEvent { subtype: MarketEventType::Trade, price: 101, quantity: 1, side: None, exchange_ts: 2 },
            MarketEvent { subtype: MarketEventType::Trade, price: 102, quantity: 1, side: None, exchange_ts: 3 },
            MarketEvent { subtype: MarketEventType::Trade, price: 103, quantity: 1, side: None, exchange_ts: 4 },
        ];
        let r1 = detect_regime_from_events(&events);
        let r2 = detect_regime_from_events(&events);
        assert_eq!(r1.0, r2.0);
        assert!((r1.1 - r2.1).abs() < 1e-12);
    }

    #[test]
    fn test_detect_regime_flat_is_sideways_or_mixed_with_high_confidence() {
        let events = vec![
            MarketEvent { subtype: MarketEventType::Trade, price: 100, quantity: 1, side: None, exchange_ts: 1 },
            MarketEvent { subtype: MarketEventType::Trade, price: 100, quantity: 1, side: None, exchange_ts: 2 },
            MarketEvent { subtype: MarketEventType::Trade, price: 100, quantity: 1, side: None, exchange_ts: 3 },
            MarketEvent { subtype: MarketEventType::Trade, price: 100, quantity: 1, side: None, exchange_ts: 4 },
        ];
        let (regime, confidence) = detect_regime_from_events(&events);
        assert!(matches!(regime, Regime::Sideways | Regime::Mixed));
        assert!(confidence >= 0.9 || matches!(regime, Regime::Mixed));
    }
}