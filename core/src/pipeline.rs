use crate::binance_adapter::load_binance_events_from_jsonl;
use crate::csv_source::CsvCandleSource;
use crate::data_source::CandleSource;
use crate::folder_source::FolderCandleSource;
use crate::ga::{self, GaConfig};
use crate::market_adapter::{convert_series_to_events, Candle};
use crate::selection_cap;
use crate::{MarketEvent, SimEvent};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::env;
use std::fs;
use std::path::Path;

/// Float tolerance for deterministic comparisons (not `f64::EPSILON`, which is ~2e-16 and too tight for scaled money/weights).
const F64_TOL: f64 = 1e-12;

const VOLATILITY_THRESHOLD: f64 = 0.01;
const TREND_THRESHOLD: f64 = 0.01;
const VOL_NORM_FACTOR: f64 = 0.02;
const DEFAULT_CONFIDENCE_FLOOR: f64 = 0.20;
/// Default slightly below mid grid so more signals pass without env; override with `SIGNAL_SCORE_FLOOR` or `SCORE_FLOOR`.
const DEFAULT_SCORE_FLOOR: f64 = 0.30;
const MIN_TRADABLE_EDGE: f64 = 0.0005;
const EDGE_OVERRIDE_THRESHOLD: f64 = 0.0012;
const EDGE_CONFIDENCE_SCALE: f64 = 500.0;
const EDGE_CONFIDENCE_GAIN: f64 = 5.0;
const SIDEWAYS_CONFIDENCE_FLOOR: f64 = 0.60;
const MIN_TARGET_PARTICIPATION: f64 = 0.15;
const MAX_TARGET_PARTICIPATION: f64 = 0.35;
const AUTO_CONFIDENCE_FLOORS: [f64; 5] = [0.30, 0.35, 0.40, 0.45, 0.50];
const AUTO_SCORE_FLOORS: [f64; 5] = [0.35, 0.40, 0.45, 0.50, 0.55];

fn env_f64(key: &str, default: f64) -> f64 {
    env::var(key)
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(default)
}

fn resolved_signal_confidence_floor(default: f64) -> f64 {
    env::var("SIGNAL_CONF_FLOOR")
        .or_else(|_| env::var("CONFIDENCE_FLOOR"))
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(default)
        .clamp(0.0, 1.0)
}

fn resolved_signal_score_floor(default: f64) -> f64 {
    env::var("SIGNAL_SCORE_FLOOR")
        .or_else(|_| env::var("SCORE_FLOOR"))
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(default)
        .clamp(0.0, 1.0)
}

fn resolved_min_tradable_edge() -> f64 {
    env_f64("MIN_TRADABLE_EDGE", MIN_TRADABLE_EDGE).max(0.0)
}

fn resolved_edge_override_threshold() -> f64 {
    env::var("EDGE_OVERRIDE_THRESHOLD")
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(EDGE_OVERRIDE_THRESHOLD)
        .max(0.0)
}

fn resolved_trailing_stop_factor() -> f64 {
    env_f64("EXIT_TRAILING_FACTOR", 0.75).clamp(0.0, 1.0)
}

/// NSE-style minimum notional per trade in **rupees** from env (`MIN_TRADE_VALUE`, default ₹1000).
/// Allocation converts to paise via `PRICE_SCALE` for internal math only.
fn resolved_min_trade_value_rupees() -> f64 {
    env_f64("MIN_TRADE_VALUE", 1000.0).max(0.0)
}

/// Max fraction of `total_capital` per single signal before redistribution (`MAX_ALLOCATION_RATIO`, default 0.4).
fn resolved_max_allocation_ratio() -> f64 {
    env_f64("MAX_ALLOCATION_RATIO", 0.4).clamp(0.0, 1.0)
}

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

/// Per-asset ranking row for API / UI (folder pipeline uses the same composite score as `evaluate_on_real_data`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetRanking {
    pub asset: String,
    pub score: f64,
    pub participation: f64,
    pub avg_pnl: f64,
    pub weak_executed_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedStrategyStore {
    pub version: u32,
    pub global_lambda: f64,
    pub assets: Vec<String>,
    pub by_asset: HashMap<String, ga::GaResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UnifiedStrategyEvaluation {
    pub strategy_id: String,
    pub avg: f64,
    pub std: f64,
    pub ga_fitness: Option<f64>,
    pub execution_fitness: f64,
    pub classification: String,
}

impl From<ga::StrategyEvaluation> for UnifiedStrategyEvaluation {
    fn from(eval: ga::StrategyEvaluation) -> Self {
        let classification = ga::get_strategy_classification(&eval);
        Self {
            strategy_id: eval.strategy_id,
            avg: eval.avg_pnl,
            std: eval.std_dev,
            ga_fitness: Some(eval.fitness),
            execution_fitness: eval.fitness,
            classification,
        }
    }
}

pub fn deterministic_strategy_id(
    strategy: &ga::Strategy,
    _scenario_names: &[String],
    _seed: u64,
) -> String {
    ga::strategy_to_id(strategy)
}

pub fn run_evaluation_orchestration(
    asset_name: &str,
    strategy: ga::Strategy,
    scenarios: &HashMap<String, Vec<MarketEvent>>,
    seed: u64,
) -> Result<UnifiedStrategyEvaluation, String> {
    let mut scenario_names: Vec<String> = scenarios.keys().cloned().collect();
    scenario_names.sort();

    let strategy_id = deterministic_strategy_id(&strategy, &scenario_names, seed);

    let ga_config = GaConfig {
        seed,
        ..GaConfig::default()
    };

    let scenarios_vec = map_to_pairs(asset_name, scenarios);
    let eval = ga::evaluate_and_aggregate(&strategy, &ga_config, &scenarios_vec, 0, 0.0, 0)
        .ok_or_else(|| "Strategy produced no evaluable trades".to_string())?;

    let mut unified = UnifiedStrategyEvaluation::from(eval);
    unified.strategy_id = strategy_id;
    Ok(unified)
}

pub fn run_comparison_orchestration(
    asset_name: &str,
    strategies: Vec<ga::Strategy>,
    scenarios: &HashMap<String, Vec<MarketEvent>>,
    seed: u64,
) -> Result<Vec<UnifiedStrategyEvaluation>, String> {
    let mut results = Vec::new();
    for strategy in strategies {
        let res = run_evaluation_orchestration(asset_name, strategy, scenarios, seed)?;
        results.push(res);
    }

    // Sort by execution fitness descending
    results.sort_by(|a, b| {
        b.execution_fitness
            .partial_cmp(&a.execution_fitness)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.strategy_id.cmp(&b.strategy_id))
    });

    Ok(results)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedGaResponse {
    pub global_best: UnifiedStrategyEvaluation,
    pub final_generation_best: UnifiedStrategyEvaluation,
    pub generation_history: Vec<UnifiedStrategyEvaluation>,
    pub best_per_regime: HashMap<String, UnifiedStrategyEvaluation>,
    pub global_best_generation: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SignalAction {
    BUY,
    SELL,
    HOLD,
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
    pub current_pnl: f64,
    pub peak_pnl: f64,
    pub exit_reason: Option<ExitReason>,
    pub is_open: bool,
    pub strategy_id: String,
    pub reason: String,
    /// `confidence * max(0, edge)^gamma` (`RANK_SCORE_EDGE_GAMMA`, default 1) — deterministic post-gate ranking (see `SIGNAL_TOP_K`).
    #[serde(default)]
    pub rank_score: f64,
    /// 1-based position among actionable signals after global Top-K sort; `None` if unlimited cap or not actionable.
    #[serde(default)]
    pub rank_position: Option<u32>,
    /// Deployed notional in **paise** (same unit as `entry_zone` tick prices).
    #[serde(default)]
    pub allocated_capital: Option<f64>,
    #[serde(default)]
    pub quantity: Option<u64>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ExitReason {
    TakeProfit,
    TrailingStop,
    EdgeDecay,
    RegimeFlip,
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
    /// `None` = no Top-K cap (`SIGNAL_TOP_K=0`); `Some(k)` = global rank cap applied.
    #[serde(default)]
    pub selection_cap_k: Option<usize>,
    pub edge_loss_breakdown: EdgeLossBreakdown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalsSnapshot {
    pub timestamp: u64,
    pub signals: Vec<TradeSignal>,
    pub meta: SignalMeta,
    pub asset_name: String, // Add asset name to snapshot for context
    #[serde(default)]
    pub asset_rankings: Vec<AssetRanking>,
}

impl Default for SignalsSnapshot {
    fn default() -> Self {
        Self {
            timestamp: 0,
            signals: Vec::new(),
            meta: SignalMeta {
                total_assets: 0,
                total_scenarios: 0,
                trades: 0,
                holds: 0,
                participation: 0.0,
                selection_cap_k: None,
                edge_loss_breakdown: EdgeLossBreakdown::default(),
            },
            asset_name: "UNKNOWN".to_string(),
            asset_rankings: Vec::new(),
        }
    }
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
    /// BUY/SELL passed gates but was not in the top-K rank after `SIGNAL_TOP_K` selection.
    CapacityRankedOut,
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
            EdgeLossReason::CapacityRankedOut => "CAPACITY_RANKED_OUT",
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
    pub current_pnl: f64,
    pub peak_pnl: f64,
    pub exit_triggered: bool,
    pub exit_reason: Option<ExitReason>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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
    pub exit_count: usize,
    pub exit_by_reason: HashMap<ExitReason, usize>,
    pub avg_exit_pnl: f64,
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
        Regime::TrendingUp | Regime::TrendingDown => trend_strength / dominance_denominator,
        Regime::Volatile => vol_strength / dominance_denominator,
        Regime::Sideways => 1.0 - trend_strength.max(vol_strength),
        Regime::Mixed => 0.0,
    }
    .clamp(0.0, 1.0);

    (regime, confidence)
}

pub fn scenarios_from_candles(
    asset: &str,
    candles: &[Candle],
) -> HashMap<String, Vec<MarketEvent>> {
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
            } = ev
            {
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
            scenarios.insert(
                format!("{}_csv_window_{}", asset, scenario_id),
                market_events,
            );
        }

        start += stride;
        scenario_id += 1;
    }

    scenarios
}

/// Deterministic (lexicographic scenario name): first event price and timestamp for GA config.
fn initial_order_from_scenario_map(
    scenario_map: &HashMap<String, Vec<MarketEvent>>,
) -> Option<(u64, u64)> {
    let mut keys: Vec<&String> = scenario_map.keys().collect();
    keys.sort();
    let first_key = keys.first().copied()?;
    let events = scenario_map.get(first_key)?;
    let e = events.first()?;
    Some((e.price, e.exchange_ts))
}

pub fn resolve_execution_symbol(symbol: &str) -> String {
    let s = symbol.to_uppercase();
    if s.contains("_FUT") {
        s
    } else {
        format!("{}_FUT", s)
    }
}

/// Scenarios from CSV candles only (`folder` preloads candles; `csv` loads `{folder_path}/{asset}_5m_clean.csv`).
pub fn scenario_map_for_signal_generation(
    asset_name: &str,
    data_source: &str,
    folder_candles: Option<&Vec<Candle>>,
    folder_path: &str,
) -> HashMap<String, Vec<MarketEvent>> {
    match data_source {
        "folder" => {
            let candles = folder_candles.cloned().unwrap_or_default();
            scenarios_from_candles(asset_name, &candles)
        }
        "csv" => {
            let path = format!(
                "{}/{}_5m_clean.csv",
                folder_path.trim_end_matches('/'),
                asset_name
            );
            if Path::new(&path).exists() {
                let source = CsvCandleSource { path };
                let candles = source.get_candles_sync();
                scenarios_from_candles(asset_name, &candles)
            } else {
                HashMap::new()
            }
        }
        _ => HashMap::new(),
    }
}

pub fn pair_scenarios_by_index<'a>(
    signal_symbol: &'a str,
    execution_symbol: &'a str,
    signal_scenarios: &'a HashMap<String, Vec<MarketEvent>>,
    execution_scenarios: &'a HashMap<String, Vec<MarketEvent>>,
) -> Vec<ga::ScenarioPair<'a>> {
    let mut s_names: Vec<&String> = signal_scenarios.keys().collect();
    s_names.sort();

    let mut e_names: Vec<&String> = execution_scenarios.keys().collect();
    e_names.sort();

    let strict = std::env::var("CS_STRICT").unwrap_or_else(|_| "1".into()) == "1";
    println!("ROUTE_INIT → {} -> {}", signal_symbol, execution_symbol);

    let mut pairs = Vec::new();
    let total_potential = s_names.len().min(e_names.len());
    let mut paired = 0usize;
    let mut skipped = 0usize;

    for i in 0..total_potential {
        let s_name = s_names[i];
        let e_name = e_names[i];

        let s_events = signal_scenarios.get(s_name).unwrap();
        let e_events = execution_scenarios.get(e_name).unwrap();

        // Unbreakable: Hard check on length and timestamps
        let len_mismatch = s_events.len() != e_events.len();
        let mut ts_mismatch = false;

        if !len_mismatch {
            for (se, ee) in s_events.iter().zip(e_events.iter()) {
                if se.exchange_ts != ee.exchange_ts {
                    ts_mismatch = true;
                    break;
                }
            }
        }

        if len_mismatch || ts_mismatch {
            if strict {
                panic!(
                    "FATAL: Scenario alignment mismatch for {}/{} (strict mode)",
                    s_name, e_name
                );
            } else {
                skipped += 1;
                continue;
            }
        }

        // Pointer safety assert: Different symbols MUST have different memory
        if signal_symbol != execution_symbol {
            assert!(
                !std::ptr::eq(s_events.as_ptr(), e_events.as_ptr()),
                "FATAL: Dual-stream collapsed into single stream for symbol pair {} -> {}",
                signal_symbol,
                execution_symbol
            );
        }

        pairs.push(ga::ScenarioPair {
            name: s_name.as_str(),
            signal_symbol,
            execution_symbol,
            signal: s_events.as_slice(),
            execution: e_events.as_slice(),
        });
        paired += 1;
    }

    // PAIR_HEALTH metric for observability
    let ratio = paired as f64 / (paired + skipped).max(1) as f64;
    println!(
        "PAIR_HEALTH → paired={} skipped={} ratio={:.2}",
        paired, skipped, ratio
    );

    if strict {
        assert!(
            ratio >= 0.6,
            "Too many windows skipped for {} (ratio={:.2}); check data alignment",
            signal_symbol,
            ratio
        );
    }

    pairs
}

fn strategy_store_path_from_env() -> String {
    env::var("STRATEGY_STORE_PATH").unwrap_or_else(|_| {
        "/Users/nikhil/ChronoSentiment_MEGA_FINAL/test_assets/strategy_store.json".to_string()
    })
}

pub fn load_strategy_store(path: &str) -> Result<PersistedStrategyStore, String> {
    let raw = fs::read_to_string(path)
        .map_err(|e| format!("failed to read strategy store '{}': {}", path, e))?;
    serde_json::from_str::<PersistedStrategyStore>(&raw)
        .map_err(|e| format!("failed to parse strategy store '{}': {}", path, e))
}

fn save_strategy_store(path: &str, store: &PersistedStrategyStore) -> Result<(), String> {
    let json = serde_json::to_string_pretty(store)
        .map_err(|e| format!("failed to serialize strategy store: {}", e))?;
    fs::write(path, json).map_err(|e| format!("failed to write strategy store '{}': {}", path, e))
}

fn train_asset_strategy(
    asset_name: &str,
    paired_scenarios: &[ga::ScenarioPair],
    global_lambda: f64,
) -> Option<ga::GaResult> {
    if paired_scenarios.is_empty() {
        return None;
    }

    let initial_price = paired_scenarios[0]
        .signal
        .first()
        .map(|e| e.price)
        .unwrap_or(0);
    let initial_timestamp = paired_scenarios[0]
        .signal
        .first()
        .map(|e| e.exchange_ts)
        .unwrap_or(0);

    let config = GaConfig {
        population_size: 5,
        generations: 3,
        mutation_rate: 0.1,
        seed: 42,
        order_id_prefix: format!("SIGNAL_GA_{}", asset_name),
        order_price: initial_price,
        order_quantity_for_strategy: 100,
        order_timestamp: initial_timestamp,
        lambda: global_lambda,
        initial_queue_threshold: 200,
        ..GaConfig::default()
    };

    let mut sorted_pairs = paired_scenarios.to_vec();
    sorted_pairs.sort_by(|a, b| a.name.cmp(&b.name));

    let test_index = (config.seed as usize) % sorted_pairs.len();
    let test_scenario_name = sorted_pairs[test_index].name;

    let mut train_scenarios: Vec<ga::ScenarioPair> = Vec::new();
    for pair in &sorted_pairs {
        if pair.name != test_scenario_name {
            train_scenarios.push(pair.clone());
        }
    }

    Some(ga::run_ga_evolution(config, &train_scenarios))
}

pub fn train_and_persist_strategies(
    assets: Vec<String>,
    global_lambda: f64,
    path: Option<String>,
) -> Result<usize, String> {
    let data_source = env::var("DATA_SOURCE")
        .unwrap_or_else(|_| "folder".to_string())
        .to_lowercase();
    let folder_path = "/Users/nikhil/ChronoSentiment_MEGA_FINAL/test_assets".to_string();
    let mut folder_candles_by_asset: HashMap<String, Vec<Candle>> = HashMap::new();
    if data_source == "folder" {
        let source = FolderCandleSource {
            folder_path: folder_path.clone(),
        };
        for (asset, candles) in source.load_all() {
            folder_candles_by_asset.insert(asset, candles);
        }

        // UNBREAKABLE: Add memory pointer collision guard after loading (Requested)
        for asset in folder_candles_by_asset.keys() {
            let exec_asset = resolve_execution_symbol(asset);
            if exec_asset != *asset && folder_candles_by_asset.contains_key(&exec_asset) {
                let spot_ptr = folder_candles_by_asset[asset].as_ptr();
                let fut_ptr = folder_candles_by_asset[&exec_asset].as_ptr();
                assert!(
                    spot_ptr != fut_ptr,
                    "FATAL: Spot and Futures share same memory ({})",
                    asset
                );
            }
        }
    }

    let mut trained_assets: Vec<String> = Vec::new();
    let mut by_asset: HashMap<String, ga::GaResult> = HashMap::new();
    for asset_name in &assets {
        let folder_candles = folder_candles_by_asset.get(asset_name);
        let signal_scenario_map = scenario_map_for_signal_generation(
            asset_name,
            data_source.as_str(),
            folder_candles,
            folder_path.as_str(),
        );
        if signal_scenario_map.is_empty() {
            continue;
        }

        let exec_asset = resolve_execution_symbol(asset_name);
        let exec_folder_candles = folder_candles_by_asset.get(&exec_asset);
        let strict = std::env::var("CS_STRICT").unwrap_or_else(|_| "1".into()) == "1";

        // Remove ALL silent fallbacks: MUST load separately if symbols differ
        let exec_scenario_map = if exec_asset != *asset_name {
            let map = scenario_map_for_signal_generation(
                &exec_asset,
                data_source.as_str(),
                exec_folder_candles,
                folder_path.as_str(),
            );

            if map.is_empty() {
                if strict {
                    panic!(
                        "FATAL: Missing execution data for {} (strict mode)",
                        exec_asset
                    );
                } else {
                    println!(
                        "SKIPPING → Missing execution scenario map for symbol={} (non-strict)",
                        exec_asset
                    );
                    continue;
                }
            }
            map
        } else {
            // Self-execution (e.g. Stocks) is allowed only if explicitly identified
            signal_scenario_map.clone()
        };

        let aligned_pairs = pair_scenarios_by_index(
            asset_name,
            &exec_asset,
            &signal_scenario_map,
            &exec_scenario_map,
        );
        if aligned_pairs.is_empty() {
            continue;
        }

        if let Some(ga_result) = train_asset_strategy(asset_name, &aligned_pairs, global_lambda) {
            by_asset.insert(asset_name.clone(), ga_result);
            trained_assets.push(asset_name.clone());
        }
    }

    let mut assets_sorted = trained_assets.clone();
    assets_sorted.sort();
    let store = PersistedStrategyStore {
        version: 1,
        global_lambda,
        assets: assets_sorted,
        by_asset,
    };
    let target = path.unwrap_or_else(strategy_store_path_from_env);
    save_strategy_store(&target, &store)?;
    println!(
        "STRATEGY_STORE_SAVED path={} assets={}",
        target,
        trained_assets.len()
    );
    Ok(trained_assets.len())
}

pub fn map_to_pairs<'a>(
    symbol: &'a str,
    scenarios: &'a HashMap<String, Vec<MarketEvent>>,
) -> Vec<ga::ScenarioPair<'a>> {
    scenarios
        .iter()
        .map(|(n, ev)| ga::ScenarioPair {
            name: n.as_str(),
            signal_symbol: symbol,
            execution_symbol: symbol,
            signal: ev.as_slice(),
            execution: ev.as_slice(),
        })
        .collect()
}

pub fn map_to_hash_pairs<'a>(
    symbol: &'a str,
    scenarios: &'a HashMap<String, Vec<MarketEvent>>,
) -> HashMap<String, ga::ScenarioPair<'a>> {
    scenarios
        .iter()
        .map(|(n, ev)| {
            (
                n.clone(),
                ga::ScenarioPair {
                    name: n.as_str(),
                    signal_symbol: symbol,
                    execution_symbol: symbol,
                    signal: ev.as_slice(),
                    execution: ev.as_slice(),
                },
            )
        })
        .collect()
}

pub fn map_to_pairs_from_refs<'a>(
    symbol: &'a str,
    scenarios: &'a HashMap<String, &'a [MarketEvent]>,
) -> Vec<ga::ScenarioPair<'a>> {
    scenarios
        .iter()
        .map(|(n, ev)| ga::ScenarioPair {
            name: n.as_str(),
            signal_symbol: symbol,
            execution_symbol: symbol,
            signal: ev,
            execution: ev,
        })
        .collect()
}

pub fn map_to_hash_pairs_from_refs<'a>(
    symbol: &'a str,
    scenarios: &'a HashMap<String, &'a [MarketEvent]>,
) -> HashMap<String, ga::ScenarioPair<'a>> {
    scenarios
        .iter()
        .map(|(n, ev)| {
            (
                n.clone(),
                ga::ScenarioPair {
                    name: n.as_str(),
                    signal_symbol: symbol,
                    execution_symbol: symbol,
                    signal: ev,
                    execution: ev,
                },
            )
        })
        .collect()
}

pub fn run_ga_orchestration(
    asset_name: &str,
    config: GaConfig,
    scenarios: &HashMap<String, Vec<MarketEvent>>,
    holdout_ratio: f64,
) -> Result<UnifiedGaResponse, String> {
    let mut scenario_names: Vec<String> = scenarios.keys().cloned().collect();
    scenario_names.sort();

    if scenario_names.is_empty() {
        return Err("No scenarios provided for GA orchestration".to_string());
    }

    // Split scenarios into train and holdout
    let (train_names, holdout_names): (Vec<String>, Vec<String>) = if scenario_names.len() <= 2 {
        (scenario_names.clone(), scenario_names.clone())
    } else {
        let holdout_count = ((scenario_names.len() as f64) * holdout_ratio).round() as usize;
        let holdout_count = holdout_count.clamp(1, scenario_names.len() - 1);
        let split_at = scenario_names.len() - holdout_count;
        (
            scenario_names[..split_at].to_vec(),
            scenario_names[split_at..].to_vec(),
        )
    };

    let mut train_scenarios: HashMap<String, Vec<MarketEvent>> = HashMap::new();
    for name in &train_names {
        if let Some(events) = scenarios.get(name) {
            train_scenarios.insert(name.clone(), events.clone());
        }
    }

    let mut holdout_scenarios: HashMap<String, Vec<MarketEvent>> = HashMap::new();
    for name in &holdout_names {
        if let Some(events) = scenarios.get(name) {
            holdout_scenarios.insert(name.clone(), events.clone());
        }
    }

    // Run GA Evolution
    let train_pairs = map_to_pairs(asset_name, &train_scenarios);
    let ga_result = ga::run_ga_evolution(config.clone(), &train_pairs);

    // Cross-evaluate best strategies on holdout data for "Execution Fitness"
    let execution_scenarios = if holdout_scenarios.is_empty() {
        &train_scenarios
    } else {
        &holdout_scenarios
    };

    let to_unified = |ga_eval: ga::StrategyEvaluation,
                      exec_eval: ga::StrategyEvaluation|
     -> UnifiedStrategyEvaluation {
        let mut unified = UnifiedStrategyEvaluation::from(ga_eval);
        unified.execution_fitness = exec_eval.fitness;
        unified
    };

    let exec_hash_pairs = map_to_hash_pairs(asset_name, execution_scenarios);
    let exec_vec_pairs: Vec<ga::ScenarioPair> = exec_hash_pairs.values().cloned().collect();

    let evaluate_on_exec = |strategy: &ga::Strategy| -> ga::StrategyEvaluation {
        let generation = 0; // Standard non-evolutionary evaluation
        ga::evaluate_and_aggregate(strategy, &config, &exec_vec_pairs, generation, 0.0, 0).unwrap_or_else(
            || ga::StrategyEvaluation {
                strategy: strategy.clone(),
                ..ga::StrategyEvaluation::default()
            },
        )
    };

    let global_best = to_unified(
        ga_result.global_best.clone(),
        evaluate_on_exec(&ga_result.global_best.strategy),
    );

    let final_generation_best = to_unified(
        ga_result.final_generation_best.clone(),
        evaluate_on_exec(&ga_result.final_generation_best.strategy),
    );

    let mut generation_history = Vec::new();
    for eval in ga_result.generation_history {
        generation_history.push(to_unified(eval.clone(), evaluate_on_exec(&eval.strategy)));
    }

    let mut best_per_regime = HashMap::new();
    for (regime, eval) in ga_result.best_per_regime {
        best_per_regime.insert(
            regime,
            to_unified(eval.clone(), evaluate_on_exec(&eval.strategy)),
        );
    }

    Ok(UnifiedGaResponse {
        global_best,
        final_generation_best,
        generation_history,
        best_per_regime,
        global_best_generation: ga_result.global_best_generation,
    })
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

fn edge_to_confidence(execution_fitness: f64) -> f64 {
    (execution_fitness.max(0.0) * EDGE_CONFIDENCE_SCALE).clamp(0.0, 1.0)
}

fn calibrated_confidence(raw_confidence: f64, execution_fitness: f64) -> f64 {
    let boosted = raw_confidence * (1.0 + EDGE_CONFIDENCE_GAIN * execution_fitness.max(0.0));
    boosted.clamp(0.0, 1.0)
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

fn blended_position_size(confidence: f64, execution_fitness: f64) -> f64 {
    let conf_tier = tiered_position_size(confidence);
    let edge_component = edge_norm(execution_fitness);
    // Edge-first sizing with confidence as stabilizer; deterministic and bounded.
    (0.6 * conf_tier + 0.4 * edge_component).clamp(0.1, 1.0)
}

fn evaluate_gate(
    regime: Regime,
    confidence: f64,
    execution_fitness: f64,
    confidence_floor: f64,
    score_floor: f64,
    min_tradable_edge: f64,
    edge_override_threshold: f64,
) -> GateDecision {
    if execution_fitness <= 0.0 {
        return GateDecision {
            trade_allowed: false,
            position_size: 0.0,
            composite_score: 0.0,
            reject_reason: Some(RejectReason::NegativeEdge),
        };
    }

    if execution_fitness < min_tradable_edge {
        return GateDecision {
            trade_allowed: false,
            position_size: 0.0,
            composite_score: 0.0,
            reject_reason: Some(RejectReason::NegativeEdge),
        };
    }

    let edge_aligned_confidence = confidence.max(edge_to_confidence(execution_fitness));
    let effective_confidence = calibrated_confidence(edge_aligned_confidence, execution_fitness);
    let override_by_edge = execution_fitness >= edge_override_threshold;
    let sideways_allowed_conf = confidence_floor.max(SIDEWAYS_CONFIDENCE_FLOOR);
    if matches!(regime, Regime::Sideways)
        && effective_confidence < sideways_allowed_conf
        && !override_by_edge
    {
        return GateDecision {
            trade_allowed: false,
            position_size: 0.0,
            composite_score: 0.0,
            reject_reason: Some(RejectReason::SidewaysRegime),
        };
    }

    if effective_confidence < confidence_floor && !override_by_edge {
        return GateDecision {
            trade_allowed: false,
            position_size: 0.0,
            composite_score: 0.0,
            reject_reason: Some(RejectReason::LowConfidence),
        };
    }

    let score = 0.5 * effective_confidence
        + 0.3 * edge_norm(execution_fitness)
        + 0.2 * regime_quality(regime);
    if score < score_floor && !override_by_edge {
        return GateDecision {
            trade_allowed: false,
            position_size: 0.0,
            composite_score: score,
            reject_reason: Some(RejectReason::LowScore),
        };
    }

    GateDecision {
        trade_allowed: true,
        position_size: blended_position_size(effective_confidence, execution_fitness),
        composite_score: score,
        reject_reason: None,
    }
}

/// Intrinsic pre-floor metrics (same formula as inside `evaluate_gate` after min-edge, before sideways/conf/score floors).
/// Used for sweep diagnostics only; returns `None` if edge is non-positive or below `min_tradable_edge`.
fn intrinsic_gate_strength(
    regime: Regime,
    raw_confidence: f64,
    execution_fitness: f64,
    min_tradable_edge: f64,
) -> Option<(f64, f64)> {
    if execution_fitness <= 0.0 || execution_fitness < min_tradable_edge {
        return None;
    }
    let edge_aligned_confidence = raw_confidence.max(edge_to_confidence(execution_fitness));
    let effective_confidence = calibrated_confidence(edge_aligned_confidence, execution_fitness);
    let score = 0.5 * effective_confidence
        + 0.3 * edge_norm(execution_fitness)
        + 0.2 * regime_quality(regime);
    Some((effective_confidence, score))
}

fn print_sweep_gate_intrinsic_summary(rows: &[(f64, f64)]) {
    if rows.is_empty() {
        println!("SWEEP_GATE_DEBUG: n_above_min_edge=0 (no scenarios cleared min_tradable_edge)");
        return;
    }
    let mut ec_min = f64::INFINITY;
    let mut ec_max = f64::NEG_INFINITY;
    let mut sc_min = f64::INFINITY;
    let mut sc_max = f64::NEG_INFINITY;
    let mut ec_sum = 0.0;
    let mut sc_sum = 0.0;
    for &(ec, sc) in rows {
        ec_min = ec_min.min(ec);
        ec_max = ec_max.max(ec);
        sc_min = sc_min.min(sc);
        sc_max = sc_max.max(sc);
        ec_sum += ec;
        sc_sum += sc;
    }
    let n = rows.len() as f64;
    println!(
        "SWEEP_GATE_DEBUG: n_above_min_edge={} eff_conf min/max/mean={:.4}/{:.4}/{:.4} composite_score min/max/mean={:.4}/{:.4}/{:.4} (floors do not bind if all means >> grid)",
        rows.len(),
        ec_min,
        ec_max,
        ec_sum / n,
        sc_min,
        sc_max,
        sc_sum / n
    );
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

fn infer_exit_state(
    signal: &TradeSignal,
    events: &[MarketEvent],
    entry_regime: Regime,
    initial_edge: f64,
    effective_eval_edge: f64,
) -> (f64, f64, Option<ExitReason>, bool) {
    if !matches!(signal.action, SignalAction::BUY | SignalAction::SELL) || events.is_empty() {
        return (0.0, 0.0, None, false);
    }
    let entry_price = signal
        .entry_zone
        .map(|(lo, hi)| (lo + hi) * 0.5)
        .unwrap_or_else(|| events.first().map(|e| e.price as f64).unwrap_or(0.0));
    if entry_price <= 0.0 {
        return (0.0, 0.0, None, false);
    }
    let target_price = signal.target.unwrap_or(entry_price * 1.01);
    let take_profit_threshold = match signal.action {
        SignalAction::BUY => ((target_price - entry_price) / entry_price).max(0.0),
        SignalAction::SELL => ((entry_price - target_price) / entry_price).max(0.0),
        SignalAction::HOLD => 0.0,
    };

    let mut current_pnl = 0.0;
    let mut peak_pnl = f64::NEG_INFINITY;
    for ev in events {
        let px = ev.price as f64;
        let pnl = match signal.action {
            SignalAction::BUY => (px - entry_price) / entry_price,
            SignalAction::SELL => (entry_price - px) / entry_price,
            SignalAction::HOLD => 0.0,
        };
        current_pnl = pnl;
        peak_pnl = peak_pnl.max(pnl);
    }
    if !peak_pnl.is_finite() {
        peak_pnl = 0.0;
    }

    let mut exit_reason = None;
    if take_profit_threshold > 0.0 && current_pnl >= take_profit_threshold {
        exit_reason = Some(ExitReason::TakeProfit);
    } else if peak_pnl > 0.0 && current_pnl < peak_pnl * resolved_trailing_stop_factor() {
        exit_reason = Some(ExitReason::TrailingStop);
    } else if effective_eval_edge < (initial_edge * 0.5) {
        exit_reason = Some(ExitReason::EdgeDecay);
    } else {
        let (current_regime, _) = detect_regime_from_events(events);
        if current_regime != entry_regime {
            exit_reason = Some(ExitReason::RegimeFlip);
        }
    }
    let is_open = exit_reason.is_none();
    (current_pnl, peak_pnl, exit_reason, is_open)
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
    let (action, entry_type, entry_zone, stop_loss, target, expected_holding_time, reason) =
        if trade_allowed {
            match regime {
                Regime::TrendingUp => {
                    let entry_lo = crate::round_to_tick((last_price * 0.996).round() as u64) as f64;
                    let entry_hi = crate::round_to_tick((last_price * 1.000).round() as u64) as f64;
                    let entry_zone = (entry_lo, entry_hi);
                    let effective_atr = atr.max(last_price * 0.003);
                    let mut stop_loss =
                        crate::round_to_tick((last_price - (1.5 * effective_atr)).round() as u64)
                            as f64;
                    // Ensure stop loss is below entry zone after rounding
                    if stop_loss >= entry_lo {
                        stop_loss = entry_lo - (if entry_lo >= 2000.0 { 5.0 } else { 1.0 });
                    }
                    let risk = (last_price - stop_loss).abs();
                    let mut target =
                        crate::round_to_tick((last_price + (2.0 * risk)).round() as u64) as f64;
                    // Ensure target is above entry zone after rounding
                    if target <= entry_hi {
                        target = entry_hi + (if entry_hi >= 2000.0 { 5.0 } else { 1.0 });
                    }
                    assert!(
                        target > entry_zone.1,
                        "Target must exceed entry zone for BUY"
                    );
                    assert!(
                        stop_loss < entry_zone.0,
                        "Stop loss must be below entry zone for BUY"
                    );
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
                    let entry_lo = crate::round_to_tick((last_price * 1.000).round() as u64) as f64;
                    let entry_hi = crate::round_to_tick((last_price * 1.004).round() as u64) as f64;
                    let entry_zone = (entry_lo, entry_hi);
                    let effective_atr = atr.max(last_price * 0.003);
                    let mut stop_loss =
                        crate::round_to_tick((last_price + (1.5 * effective_atr)).round() as u64)
                            as f64;
                    // Ensure stop loss is above entry zone after rounding
                    if stop_loss <= entry_hi {
                        stop_loss = entry_hi + (if entry_hi >= 2000.0 { 5.0 } else { 1.0 });
                    }
                    let risk = (stop_loss - last_price).abs();
                    let mut target =
                        crate::round_to_tick((last_price - (2.0 * risk)).round() as u64) as f64;
                    // Ensure target is below entry zone after rounding
                    if target >= entry_lo {
                        target = entry_lo - (if entry_lo >= 2000.0 { 5.0 } else { 1.0 });
                    }
                    assert!(
                        target < entry_zone.0,
                        "Target must be below entry zone for SELL"
                    );
                    assert!(
                        stop_loss > entry_zone.1,
                        "Stop loss must be above entry zone for SELL"
                    );
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
        assert!(
            risk_reward >= 1.5,
            "Risk-reward below minimum threshold: {}",
            risk_reward
        );
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
        position_size: if trade_allowed {
            gate.position_size
        } else {
            0.0
        },
        conviction: confidence * report_fitness.max(0.0),
        composite_score: gate.composite_score,
        reject_reason: gate.reject_reason.map(|r| r.as_str().to_string()),
        expected_holding_time,
        current_pnl: 0.0,
        peak_pnl: 0.0,
        exit_reason: None,
        is_open: trade_allowed,
        strategy_id: selected_strategy_id.to_string(),
        reason,
        rank_score: selection_cap::rank_score_edge_confidence(report_fitness, confidence),
        rank_position: None,
        allocated_capital: None,
        quantity: None,
    }
}

fn demote_signal_for_capacity(s: &mut TradeSignal) {
    let prev_edge = s.expected_edge;
    s.action = SignalAction::HOLD;
    s.entry_type = EntryType::MARKET;
    s.entry_zone = None;
    s.stop_loss = None;
    s.target = None;
    s.position_size = 0.0;
    s.scenario_pnl = 0.0;
    s.risk_reward = 0.0;
    s.reject_reason = Some("CAPACITY_RANKED_OUT".to_string());
    s.expected_holding_time = "N/A".to_string();
    s.reason = format!(
        "Capacity-ranked: not in top SIGNAL_TOP_K (edge={:.4}, rank_score={:.6})",
        prev_edge, s.rank_score
    );
    s.is_open = false;
    s.exit_reason = None;
    s.current_pnl = 0.0;
    s.peak_pnl = 0.0;
    s.rank_position = None;
    s.allocated_capital = None;
    s.quantity = None;
}

const RANK_BIAS_FACTOR: f64 = 1.0;
const EDGE_AWARE_SIZING_FACTOR: f64 = 0.3; // 0.7 * rank_score + 0.3 * effective_eval_edge

/// `total_capital_rupees` comes from `TOTAL_CAPITAL` (rupees). All sizing, caps, and `allocated_capital` are computed in **paise** only.
fn allocate_capital(signals: &mut Vec<TradeSignal>, total_capital_rupees: f64) {
    let scale = crate::PRICE_SCALE as f64;
    let budget_paise = (total_capital_rupees * scale).max(0.0);
    if signals.is_empty() || budget_paise <= F64_TOL {
        return;
    }

    let entry_mid_paise =
        |s: &TradeSignal| -> f64 { s.entry_zone.map(|(lo, hi)| (lo + hi) * 0.5).unwrap_or(0.0) };

    let min_trade_paise = resolved_min_trade_value_rupees() * scale;
    let max_allocation_ratio = resolved_max_allocation_ratio();

    let k_cap = selection_cap::resolved_signal_top_k()
        .map(|k| k as f64)
        .unwrap_or(1.0)
        .max(1.0);
    let mut total_adjusted_score = 0.0;

    // Step 5: Add rank bias and edge-aware sizing
    // Calculate adjusted scores for all signals
    let mut adjusted_signals: Vec<(usize, f64)> = signals
        .iter()
        .enumerate()
        .filter_map(|(i, s)| {
            if matches!(s.action, SignalAction::BUY | SignalAction::SELL)
                && s.rank_position.is_some()
            {
                let rank_position = s.rank_position.unwrap_or(1) as f64;
                let rank_bias = (k_cap - rank_position) / k_cap;

                // Apply rank bias
                let rank_score_biased = s.rank_score * (1.0 + RANK_BIAS_FACTOR * rank_bias);

                // Apply edge-aware sizing
                let final_score = (1.0 - EDGE_AWARE_SIZING_FACTOR) * rank_score_biased
                    + (EDGE_AWARE_SIZING_FACTOR * s.expected_edge);

                total_adjusted_score += final_score;
                Some((i, final_score))
            } else {
                None
            }
        })
        .collect();

    if total_adjusted_score <= F64_TOL {
        // If all adjusted scores are zero, we cannot allocate
        for s in signals.iter_mut() {
            s.allocated_capital = Some(0.0);
            s.quantity = Some(0);
        }
        return;
    }

    // Step 1-4, and NSE constraints (all amounts in paise)
    for (idx, adjusted_score) in adjusted_signals.iter_mut() {
        let s = &mut signals[*idx];

        // Step 1: normalize weights
        let weight_i = *adjusted_score / total_adjusted_score;

        // Step 2: apply capital
        let mut allocation_i = budget_paise * weight_i;

        // NSE Constraint: Max per-asset cap
        let max_cap = budget_paise * max_allocation_ratio;
        allocation_i = allocation_i.min(max_cap);

        // Drop if weighted slice cannot meet minimum notional (deterministic gate)
        if allocation_i < min_trade_paise {
            demote_signal_for_capacity(s);
            s.allocated_capital = Some(0.0);
            s.quantity = Some(0);
            continue;
        }

        // Step 3: convert to quantity (paise per share × qty = paise notional)
        let entry_p = entry_mid_paise(s);
        let mut qty_i = 0u64;
        if entry_p > F64_TOL {
            qty_i = (allocation_i / entry_p).floor() as u64;
        }

        // NSE Constraint: Lot size / liquidity guard (minimum trade value on filled notional)
        if (qty_i as f64 * entry_p) < min_trade_paise {
            qty_i = 0;
        }

        // Step 4: enforce minimum size (drop signal if quantity is 0)
        if qty_i == 0 {
            demote_signal_for_capacity(s);
            s.allocated_capital = Some(0.0);
            s.quantity = Some(0);
        } else {
            let used_i = (qty_i as f64) * entry_p;
            s.allocated_capital = Some(used_i);
            s.quantity = Some(qty_i);
        }
    }

    // Deterministic leftover redistribution: spend remaining budget in rank order (1..K), round-robin rounds.
    let mut ranked_active: Vec<(u32, usize, f64)> = signals
        .iter()
        .enumerate()
        .filter_map(|(i, s)| {
            if !matches!(s.action, SignalAction::BUY | SignalAction::SELL) {
                return None;
            }
            let rp = s.rank_position?;
            let q = s.quantity.unwrap_or(0);
            if q == 0 {
                return None;
            }
            let entry = entry_mid_paise(s);
            if entry <= F64_TOL {
                return None;
            }
            Some((rp, i, entry))
        })
        .collect();
    ranked_active.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));

    let mut used_total: f64 = signals
        .iter()
        .map(|s| {
            let q = s.quantity.unwrap_or(0);
            let p = entry_mid_paise(s);
            (q as f64) * p
        })
        .sum();
    let mut leftover = (budget_paise - used_total).max(0.0);
    let max_cap_per_asset = budget_paise * max_allocation_ratio;

    let mut progress = true;
    while progress && leftover + F64_TOL >= min_trade_paise {
        progress = false;
        for (_, idx, price) in &ranked_active {
            if leftover + F64_TOL < *price {
                continue;
            }
            let s = &mut signals[*idx];
            let q = s.quantity.unwrap_or(0);
            let new_q = q.saturating_add(1);
            let new_used = (new_q as f64) * price;
            if new_used > max_cap_per_asset + F64_TOL {
                continue;
            }
            s.quantity = Some(new_q);
            s.allocated_capital = Some(new_used);
            leftover -= price;
            used_total += price;
            progress = true;
            if leftover + F64_TOL < min_trade_paise {
                break;
            }
        }
    }

    // Final notional floor after redistribution (drop marginal micro-positions deterministically).
    for s in signals.iter_mut() {
        if !matches!(s.action, SignalAction::BUY | SignalAction::SELL) {
            continue;
        }
        let q = s.quantity.unwrap_or(0);
        if q == 0 {
            continue;
        }
        let entry_p = entry_mid_paise(s);
        if (q as f64) * entry_p + F64_TOL < min_trade_paise {
            demote_signal_for_capacity(s);
            s.allocated_capital = Some(0.0);
            s.quantity = Some(0);
        }
    }

    // NSE Constraint: Max positions
    // If more positions are allocated than SIGNAL_TOP_K, demote the lowest ranked signals
    if let Some(max_positions_k) = selection_cap::resolved_signal_top_k() {
        let active_positions = signals
            .iter()
            .filter(|s| s.allocated_capital.unwrap_or(0.0) > F64_TOL)
            .count();
        if active_positions > max_positions_k {
            let mut current_allocated_signals: Vec<(usize, f64)> = signals
                .iter()
                .enumerate()
                .filter_map(|(i, s)| {
                    if s.allocated_capital.unwrap_or(0.0) > F64_TOL {
                        Some((i, s.rank_score))
                    } else {
                        None
                    }
                })
                .collect();

            current_allocated_signals
                .sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(Ordering::Equal));

            for i in max_positions_k..current_allocated_signals.len() {
                let (idx_to_demote, _) = current_allocated_signals[i];
                let s = &mut signals[idx_to_demote];
                demote_signal_for_capacity(s);
                s.allocated_capital = Some(0.0);
                s.quantity = Some(0);
            }
        }
    }

    // Debug output: ranked executable rows only + one portfolio summary
    let portfolio_used: f64 = signals
        .iter()
        .map(|x| {
            let q = x.quantity.unwrap_or(0);
            let p = entry_mid_paise(x);
            (q as f64) * p
        })
        .sum();
    let portfolio_leftover = (budget_paise - portfolio_used).max(0.0);
    println!(
        "ALLOC_DEBUG_SUMMARY: budget_paise={:.0} used_paise={:.0} leftover_paise={:.0}",
        budget_paise, portfolio_used, portfolio_leftover
    );
    for s in signals.iter() {
        if s.rank_position.is_none() {
            continue;
        }
        if !matches!(s.action, SignalAction::BUY | SignalAction::SELL) {
            continue;
        }
        let q = s.quantity.unwrap_or(0);
        if q == 0 {
            continue;
        }
        if let (Some(used_c), Some(qty_u)) = (s.allocated_capital, s.quantity) {
            println!(
                "ALLOC_DEBUG: rank={:?} score={:.6} used_paise={:.0} qty={}",
                s.rank_position, s.rank_score, used_c, qty_u
            );
        }
    }
}

/// Deterministic portfolio layer: among gated BUY/SELL signals, keep only top `SIGNAL_TOP_K` by `rank_score`.
fn apply_capacity_selection(signals: &mut Vec<TradeSignal>, transfers: &mut Vec<EdgeTransfer>) {
    let Some(k) = selection_cap::resolved_signal_top_k() else {
        return;
    };
    if signals.len() != transfers.len() {
        return;
    }
    let mut actionable: Vec<usize> = signals
        .iter()
        .enumerate()
        .filter(|(_, s)| matches!(s.action, SignalAction::BUY | SignalAction::SELL))
        .map(|(i, _)| i)
        .collect();
    if actionable.is_empty() {
        return;
    }
    actionable.sort_by(|&a, &b| {
        let sa = &signals[a];
        let sb = &signals[b];
        sb.rank_score
            .partial_cmp(&sa.rank_score)
            .unwrap_or(Ordering::Equal)
            .then_with(|| sa.asset.cmp(&sb.asset))
            .then_with(|| sa.strategy_id.cmp(&sb.strategy_id))
            .then_with(|| sa.regime.cmp(&sb.regime))
    });
    let keep = actionable.len().min(k);
    for (rank, &idx) in actionable.iter().take(keep).enumerate() {
        signals[idx].rank_position = Some((rank as u32) + 1);
    }
    if actionable.len() <= k {
        return;
    }
    let drop: HashSet<usize> = actionable.iter().skip(k).copied().collect();
    for idx in drop {
        demote_signal_for_capacity(&mut signals[idx]);
        let t = &mut transfers[idx];
        let eval = t.eval_edge.or(t.weak_eval_edge).unwrap_or(0.0);
        t.signal_edge = 0.0;
        t.delta = eval.max(0.0);
        t.reason = EdgeLossReason::CapacityRankedOut;
        t.exit_triggered = false;
        t.exit_reason = None;
    }
}

pub fn run_pipeline_with_config(jsonl_path: &str, ga_config: GaConfig) -> SignalsSnapshot {
    let (auto_confidence_floor, auto_score_floor) =
        select_optimal_signal_thresholds_for_jsonl(jsonl_path, &ga_config);
    let confidence_floor = resolved_signal_confidence_floor(auto_confidence_floor);
    let score_floor = resolved_signal_score_floor(auto_score_floor);
    println!(
        "AUTO_THRESHOLD_SELECTION: auto_conf={:.2}, auto_score={:.2}, conf={:.2}, score={:.2}, min_edge={:.6}, edge_override={:.6}",
        auto_confidence_floor,
        auto_score_floor,
        confidence_floor,
        score_floor,
        resolved_min_tradable_edge(),
        resolved_edge_override_threshold(),
    );

    let all_events = match load_binance_events_from_jsonl(jsonl_path, 1) {
        Ok(events) => events,
        Err(e) => {
            eprintln!("Error loading Binance events from {}: {}", jsonl_path, e);
            return SignalsSnapshot::default();
        }
    };
    let mut assets: Vec<String> = all_events.iter().map(|e| e.asset.clone()).collect();
    assets.sort();
    assets.dedup();

    generate_latest_signals_with_thresholds_internal(
        assets,
        ga_config.lambda,
        confidence_floor,
        score_floor,
        None,
        Some(jsonl_path),
        Some(&ga_config),
        false,
        None,
    )
}

fn select_optimal_signal_thresholds_for_jsonl(
    jsonl_path: &str,
    ga_config: &GaConfig,
) -> (f64, f64) {
    let all_events = match load_binance_events_from_jsonl(jsonl_path, 1) {
        Ok(events) => events,
        Err(e) => {
            eprintln!(
                "Error loading Binance events for threshold sweep from {}: {}",
                jsonl_path, e
            );
            return (DEFAULT_CONFIDENCE_FLOOR, DEFAULT_SCORE_FLOOR);
        }
    };
    let mut assets: Vec<String> = all_events.iter().map(|e| e.asset.clone()).collect();
    assets.sort();
    assets.dedup();

    let rows = run_threshold_sweep(
        assets,
        ga_config.lambda,
        &AUTO_CONFIDENCE_FLOORS,
        &AUTO_SCORE_FLOORS,
        Some(jsonl_path),
        Some(ga_config),
    );
    if rows.is_empty() {
        return (DEFAULT_CONFIDENCE_FLOOR, DEFAULT_SCORE_FLOOR);
    }
    let avg_participation = rows.iter().map(|r| r.participation).sum::<f64>() / rows.len() as f64;
    let target_participation =
        (avg_participation * 0.8).clamp(MIN_TARGET_PARTICIPATION, MAX_TARGET_PARTICIPATION);

    let min_traded = rows
        .iter()
        .map(|r| r.traded_avg_pnl)
        .fold(f64::INFINITY, f64::min);
    let max_traded = rows
        .iter()
        .map(|r| r.traded_avg_pnl)
        .fold(f64::NEG_INFINITY, f64::max);
    let min_std = rows.iter().map(|r| r.std_dev).fold(f64::INFINITY, f64::min);
    let max_std = rows
        .iter()
        .map(|r| r.std_dev)
        .fold(f64::NEG_INFINITY, f64::max);
    let max_participation_distance = rows
        .iter()
        .map(|r| (r.participation - target_participation).abs())
        .fold(0.0_f64, f64::max);

    let mut best: Option<(&ThresholdSweepRow, f64)> = None;
    for row in &rows {
        if row.total_scenarios == 0 || row.trades == 0 {
            continue;
        }
        let traded_norm = if (max_traded - min_traded).abs() < F64_TOL {
            0.5
        } else {
            ((row.traded_avg_pnl - min_traded) / (max_traded - min_traded)).clamp(0.0, 1.0)
        };
        let stability_norm = if (max_std - min_std).abs() < F64_TOL {
            0.5
        } else {
            (1.0 - ((row.std_dev - min_std) / (max_std - min_std))).clamp(0.0, 1.0)
        };
        let participation_distance = (row.participation - target_participation).abs();
        let participation_norm = if max_participation_distance <= F64_TOL {
            1.0
        } else {
            (1.0 - (participation_distance / max_participation_distance)).clamp(0.0, 1.0)
        };
        let confidence_norm = ((row.confidence_floor - AUTO_CONFIDENCE_FLOORS[0])
            / (AUTO_CONFIDENCE_FLOORS[AUTO_CONFIDENCE_FLOORS.len() - 1]
                - AUTO_CONFIDENCE_FLOORS[0]))
            .clamp(0.0, 1.0);

        // Weighted utility: favor profitable signals, then stable PnL, then participation and confidence strictness.
        let utility = 0.55 * traded_norm
            + 0.20 * stability_norm
            + 0.15 * participation_norm
            + 0.10 * confidence_norm;

        match best {
            None => best = Some((row, utility)),
            Some((best_row, best_utility)) => {
                let better = utility > best_utility + F64_TOL
                    || ((utility - best_utility).abs() <= F64_TOL
                        && row.confidence_floor > best_row.confidence_floor)
                    || ((utility - best_utility).abs() <= F64_TOL
                        && (row.confidence_floor - best_row.confidence_floor).abs() <= F64_TOL
                        && row.traded_avg_pnl > best_row.traded_avg_pnl);
                if better {
                    best = Some((row, utility));
                }
            }
        }
    }

    if let Some((row, _)) = best {
        let execution_efficiency = if row.global_avg_pnl.abs() > F64_TOL {
            row.traded_avg_pnl / row.global_avg_pnl
        } else {
            0.0
        };
        println!(
            "THRESHOLD_TUNER: target_participation={:.3} selected_participation={:.3} traded_avg={:.6} global_avg={:.6} std={:.6} execution_efficiency={:.3}",
            target_participation,
            row.participation,
            row.traded_avg_pnl,
            row.global_avg_pnl,
            row.std_dev,
            execution_efficiency
        );
        (row.confidence_floor, row.score_floor)
    } else {
        (DEFAULT_CONFIDENCE_FLOOR, DEFAULT_SCORE_FLOOR)
    }
}

pub fn generate_latest_signals_with_thresholds(
    assets: Vec<String>,
    global_lambda: f64,
    confidence_floor: f64,
    score_floor: f64,
) -> SignalsSnapshot {
    generate_latest_signals_with_thresholds_internal(
        assets,
        global_lambda,
        confidence_floor,
        score_floor,
        None,
        None,
        None,
        false,
        None,
    )
}

pub fn generate_latest_signals_from_saved_strategies(
    assets: Vec<String>,
    global_lambda: f64,
    confidence_floor: f64,
    score_floor: f64,
    path: Option<String>,
) -> Result<SignalsSnapshot, String> {
    let target = path.unwrap_or_else(strategy_store_path_from_env);
    let store = load_strategy_store(&target)?;
    Ok(generate_latest_signals_with_thresholds_internal(
        assets,
        global_lambda,
        confidence_floor,
        score_floor,
        Some(&store),
        None,
        None,
        false,
        None,
    ))
}

fn scenarios_from_binance_events(
    asset: &str,
    events: &[crate::binance_adapter::NormalizedMarketEvent],
) -> HashMap<String, Vec<MarketEvent>> {
    let mut scenarios: HashMap<String, Vec<MarketEvent>> = HashMap::new();
    if events.is_empty() {
        return scenarios;
    }

    let window_size = 500;
    let stride = 250;
    let mut scenario_id = 0;
    let mut start = 0;

    while start + window_size <= events.len() && scenario_id < 20 {
        let slice = &events[start..start + window_size];
        let market_events: Vec<MarketEvent> = slice
            .iter()
            .map(|e| MarketEvent {
                subtype: crate::MarketEventType::Trade,
                price: (e.price * crate::PRICE_SCALE as f64).round() as u64,
                quantity: e.volume as u64,
                side: e.side.clone(),
                exchange_ts: e.exchange_ts,
            })
            .collect();

        if !market_events.is_empty() {
            scenarios.insert(
                format!("{}_jsonl_window_{}", asset, scenario_id),
                market_events,
            );
        }

        start += stride;
        scenario_id += 1;
    }

    scenarios
}

/// When `sweep_disable_edge_override` is true (threshold sweep / auto tuner grid only), the gate
/// does not bypass confidence or score floors via edge override, so sweep rows can differ across
/// floor values. Live signal paths pass `false` and use `EDGE_OVERRIDE_THRESHOLD` as usual.
fn generate_latest_signals_with_thresholds_internal(
    assets: Vec<String>,
    global_lambda: f64,
    confidence_floor: f64,
    score_floor: f64,
    strategy_store: Option<&PersistedStrategyStore>,
    jsonl_path: Option<&str>,
    ga_config: Option<&GaConfig>,
    sweep_disable_edge_override: bool,
    mut sweep_gate_metrics_sink: Option<&mut Vec<(f64, f64)>>,
) -> SignalsSnapshot {
    let min_tradable_edge = resolved_min_tradable_edge();
    let edge_override_threshold = if sweep_disable_edge_override {
        f64::MAX
    } else {
        resolved_edge_override_threshold()
    };

    let mut all_scenarios_by_asset: HashMap<String, HashMap<String, Vec<MarketEvent>>> =
        HashMap::new();

    if let Some(path) = jsonl_path {
        let all_events = match load_binance_events_from_jsonl(path, 1) {
            Ok(events) => events,
            Err(e) => {
                eprintln!("Error loading Binance events from {}: {}", path, e);
                return SignalsSnapshot::default();
            }
        };

        for asset_name in &assets {
            let asset_events: Vec<_> = all_events
                .iter()
                .filter(|e| &e.asset == asset_name)
                .cloned()
                .collect();
            let scenarios = scenarios_from_binance_events(asset_name, &asset_events);
            if !scenarios.is_empty() {
                all_scenarios_by_asset.insert(asset_name.clone(), scenarios);
            }
        }
    } else {
        let data_source = env::var("DATA_SOURCE")
            .unwrap_or_else(|_| "folder".to_string())
            .to_lowercase();
        let folder_path = "/Users/nikhil/ChronoSentiment_MEGA_FINAL/test_assets".to_string();
        let mut folder_candles_by_asset: HashMap<String, Vec<Candle>> = HashMap::new();
        if data_source == "folder" {
            let source = FolderCandleSource {
                folder_path: folder_path.clone(),
            };
            for (asset, candles) in source.load_all() {
                folder_candles_by_asset.insert(asset, candles);
            }
        }

        for asset_name in &assets {
            let folder_candles = folder_candles_by_asset.get(asset_name);
            let scenario_map = scenario_map_for_signal_generation(
                asset_name,
                data_source.as_str(),
                folder_candles,
                folder_path.as_str(),
            );
            if !scenario_map.is_empty() {
                // all_scenarios_by_asset expects HashMap, scenario_map is Vec<(String, Vec<MarketEvent>)>
                let mut converted: HashMap<String, Vec<MarketEvent>> = HashMap::new();
                for (name, events) in scenario_map {
                    converted.insert(name, events);
                }
                all_scenarios_by_asset.insert(asset_name.clone(), converted);
            }
        }
    }

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
    let mut asset_ranking_rows: Vec<(String, f64, f64, usize)> = Vec::new();
    let mut traded_sizes: Vec<f64> = Vec::new();
    let mut traded_pnls: Vec<f64> = Vec::new();
    let mut traded_edge_weighted_sizes: Vec<f64> = Vec::new();

    for asset_name in &assets {
        let Some(scenario_map) = all_scenarios_by_asset.get(asset_name) else {
            continue;
        };

        let mut acc_pnls: Vec<f64> = Vec::new();
        let mut traded_gate_count = 0usize;
        let mut weak_exec_rank_metric = 0usize;
        let Some((initial_price, initial_timestamp)) =
            initial_order_from_scenario_map(scenario_map)
        else {
            continue;
        };

        let config = if let Some(ga_cfg) = ga_config {
            let mut cfg = ga_cfg.clone();
            cfg.order_id_prefix = format!("SIGNAL_GA_{}", asset_name);
            cfg.order_price = initial_price;
            cfg.order_timestamp = initial_timestamp;
            cfg
        } else {
            GaConfig {
                population_size: 5,
                generations: 3,
                mutation_rate: 0.1,
                seed: 42,
                order_id_prefix: format!("SIGNAL_GA_{}", asset_name),
                order_price: initial_price,
                order_quantity_for_strategy: 100,
                order_timestamp: initial_timestamp,
                lambda: global_lambda,
                initial_queue_threshold: 200,
                ..GaConfig::default()
            }
        };

        let mut sorted_names: Vec<String> = scenario_map.keys().cloned().collect();
        sorted_names.sort();
        total_scenarios += sorted_names.len();

        let ga_result = if let Some(store) = strategy_store {
            if let Some(saved) = store.by_asset.get(asset_name) {
                saved.clone()
            } else {
                // If asset not in store, return empty/zero result instead of running GA during "recommend" phase
                ga::GaResult {
                    global_best: ga::StrategyEvaluation::default(),
                    global_best_generation: 0,
                    final_generation_best: ga::StrategyEvaluation::default(),
                    generation_history: Vec::new(),
                    best_per_regime: std::collections::HashMap::new(),
                    clusters_per_regime: std::collections::HashMap::new(),
                    population_stats: ga::PopulationStats::default(),
                    final_population: Vec::new(),
                    consensus_recommendations: None,
                }
            }
        } else {
            let test_index = (config.seed as usize) % sorted_names.len();
            let test_scenario_name = sorted_names[test_index].clone();
            let mut train_scenarios: HashMap<String, &[MarketEvent]> = HashMap::new();
            for name in &sorted_names {
                if *name != test_scenario_name {
                    if let Some(events) = scenario_map.get(name) {
                        train_scenarios.insert(name.clone(), events.as_slice());
                    }
                }
            }
            let train_pairs = map_to_pairs_from_refs(asset_name, &train_scenarios);
            ga::run_ga_evolution(config.clone(), &train_pairs)
        };

        for scenario_name in &sorted_names {
            if let Some(events) = scenario_map.get(scenario_name) {
                max_timestamp =
                    max_timestamp.max(events.last().map(|e| e.exchange_ts).unwrap_or(0));
                let (detected_regime, confidence) = detect_regime_from_events(events.as_slice());
                let regime_key = format!("{}_{}", asset_name, detected_regime.as_str());
                let selected_eval = ga_result
                    .best_per_regime
                    .get(&regime_key)
                    .unwrap_or(&ga_result.global_best);
                let mut one_scenario: HashMap<String, Vec<MarketEvent>> = HashMap::new();
                one_scenario.insert(scenario_name.clone(), events.clone());
                let one_scenario_vec = map_to_pairs(asset_name, &one_scenario);
                let generation = 0; // Standard non-evolutionary evaluation
                if let Some(report) = ga::evaluate_and_aggregate(
                    &selected_eval.strategy,
                    &config,
                    &one_scenario_vec,
                    generation,
                    0.0,
                    0,
                ) {
                    max_observed_eval_edge = max_observed_eval_edge.max(report.fitness.max(0.0));
                    if sweep_disable_edge_override {
                        if let Some(sink) = sweep_gate_metrics_sink.as_mut() {
                            if let Some(pair) = intrinsic_gate_strength(
                                detected_regime,
                                confidence,
                                report.fitness,
                                min_tradable_edge,
                            ) {
                                sink.push(pair);
                            }
                        }
                    }
                    let gate = evaluate_gate(
                        detected_regime,
                        confidence,
                        report.fitness,
                        confidence_floor,
                        score_floor,
                        min_tradable_edge,
                        edge_override_threshold,
                    );
                    let effective_eval_edge = report.fitness;
                    let mut effective_gate = gate;
                    if effective_gate.trade_allowed {
                        effective_gate.position_size = blended_position_size(
                            effective_gate.position_size,
                            effective_eval_edge,
                        );
                        trade_count += 1;
                    }
                    let executable_edge = if effective_gate.trade_allowed {
                        effective_eval_edge
                    } else {
                        0.0
                    };
                    let regime_name = detected_regime.as_str().to_string();
                    *regime_eval_sums.entry(regime_name.clone()).or_insert(0.0) += report.fitness;
                    *regime_eval_counts.entry(regime_name).or_insert(0usize) += 1;
                    let last_price = events
                        .last()
                        .map(|e| e.price as f64)
                        .unwrap_or(config.order_price as f64);
                    let atr = compute_atr(events.as_slice());
                    let mut signal = build_trade_signal(
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
                    let (current_pnl, peak_pnl, exit_reason, is_open) = infer_exit_state(
                        &signal,
                        events.as_slice(),
                        detected_regime,
                        report.fitness,
                        effective_eval_edge,
                    );
                    signal.current_pnl = current_pnl;
                    signal.peak_pnl = peak_pnl;
                    signal.exit_reason = exit_reason;
                    signal.is_open = is_open;
                    edge_transfers.push(EdgeTransfer {
                        eval_edge: Some(report.fitness),
                        weak_eval_edge: None,
                        has_strong_eval: true,
                        signal_edge: executable_edge,
                        delta: (report.fitness - executable_edge).max(0.0),
                        confidence,
                        reason: edge_reason_from_gate_reject(effective_gate.reject_reason),
                        current_pnl,
                        peak_pnl,
                        exit_triggered: exit_reason.is_some(),
                        exit_reason,
                    });
                    println!(
                        "EDGE_TRANSFER_DEBUG: asset={} scenario={} current_pnl={:.6} peak_pnl={:.6} exit_triggered={} exit_reason={:?}",
                        asset_name,
                        scenario_name,
                        current_pnl,
                        peak_pnl,
                        exit_reason.is_some(),
                        exit_reason
                    );
                    all_signals.push(signal);
                    if effective_gate.trade_allowed {
                        traded_sizes.push(effective_gate.position_size);
                        traded_pnls.push(report.avg_pnl);
                        traded_edge_weighted_sizes
                            .push(effective_gate.position_size * effective_eval_edge.max(0.0));
                    }
                    acc_pnls.push(if effective_gate.trade_allowed {
                        report.avg_pnl
                    } else {
                        0.0
                    });
                    if effective_gate.trade_allowed {
                        traded_gate_count += 1;
                        if confidence < 0.60 {
                            weak_exec_rank_metric += 1;
                        }
                    }
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
                    let effective_conf_floor = (confidence_floor - (effective_eval_edge * 0.25))
                        .clamp(confidence_floor.min(0.20), confidence_floor);
                    let first_price = events.first().map(|e| e.price as f64).unwrap_or(0.0);
                    let last_price_for_move =
                        events.last().map(|e| e.price as f64).unwrap_or(first_price);
                    let move_abs = (last_price_for_move - first_price).abs();
                    let min_move = if first_price > 0.0 {
                        first_price * 0.001
                    } else {
                        0.0
                    };
                    let low_volatility = detected_regime == Regime::Sideways
                        || (first_price > 0.0 && move_abs < min_move);
                    let weak_execution_allowed = !low_volatility
                        && (effective_eval_edge >= edge_override_threshold
                            || (confidence >= effective_conf_floor
                                && effective_eval_edge >= min_tradable_edge));
                    let weak_scaled_size = (0.7
                        * blended_position_size(
                            confidence.max(edge_to_confidence(effective_eval_edge)),
                            effective_eval_edge,
                        ))
                    .clamp(0.1, 0.7);
                    let weak_gate = GateDecision {
                        trade_allowed: weak_execution_allowed,
                        position_size: if weak_execution_allowed {
                            weak_scaled_size
                        } else {
                            0.0
                        },
                        composite_score: 0.0,
                        reject_reason: if weak_execution_allowed {
                            None
                        } else {
                            Some(RejectReason::LowConfidence)
                        },
                    };
                    let last_price = events
                        .last()
                        .map(|e| e.price as f64)
                        .unwrap_or(config.order_price as f64);
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
                        traded_sizes.push(weak_gate.position_size);
                        traded_pnls.push(signal.scenario_pnl);
                        traded_edge_weighted_sizes
                            .push(weak_gate.position_size * effective_eval_edge.max(0.0));
                    } else if low_volatility {
                        weak_rejected_low_vol += 1;
                    } else if !weak_execution_allowed {
                        weak_rejected_low_conf += 1;
                    }
                    let (current_pnl, peak_pnl, exit_reason, is_open) = infer_exit_state(
                        &signal,
                        events.as_slice(),
                        detected_regime,
                        weak_eval_edge,
                        effective_eval_edge,
                    );
                    let mut signal = signal;
                    signal.current_pnl = current_pnl;
                    signal.peak_pnl = peak_pnl;
                    signal.exit_reason = exit_reason;
                    signal.is_open = is_open;
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
                        reason: if low_volatility {
                            EdgeLossReason::WeakLowVol
                        } else {
                            EdgeLossReason::WeakEvalSurrogate
                        },
                        current_pnl,
                        peak_pnl,
                        exit_triggered: exit_reason.is_some(),
                        exit_reason,
                    });
                    println!(
                        "EDGE_TRANSFER_DEBUG: asset={} scenario={} current_pnl={:.6} peak_pnl={:.6} exit_triggered={} exit_reason={:?}",
                        asset_name,
                        scenario_name,
                        current_pnl,
                        peak_pnl,
                        exit_reason.is_some(),
                        exit_reason
                    );
                    all_signals.push(signal);
                    acc_pnls.push(0.0);
                    if weak_gate.trade_allowed {
                        traded_gate_count += 1;
                        if confidence < 0.60 {
                            weak_exec_rank_metric += 1;
                        }
                    }
                }
            } else {
                acc_pnls.push(0.0);
            }
        }
        if !sorted_names.is_empty() && !acc_pnls.is_empty() {
            let mean_pnl = acc_pnls.iter().sum::<f64>() / acc_pnls.len() as f64;
            let participation = traded_gate_count as f64 / sorted_names.len() as f64;
            asset_ranking_rows.push((
                asset_name.clone(),
                mean_pnl,
                participation,
                weak_exec_rank_metric,
            ));
        }
    }

    let max_weak = asset_ranking_rows
        .iter()
        .map(|(_, _, _, w)| *w)
        .max()
        .unwrap_or(0);
    let weak_den = if max_weak == 0 { 1.0 } else { max_weak as f64 };
    let mut asset_rankings: Vec<AssetRanking> = asset_ranking_rows
        .into_iter()
        .map(|(asset, avg_pnl, participation, weak_exec_rank_metric)| {
            let weak_norm = weak_exec_rank_metric as f64 / weak_den;
            let score = 0.5 * avg_pnl + 0.3 * participation + 0.2 * weak_norm;
            AssetRanking {
                asset,
                score,
                participation,
                avg_pnl,
                weak_executed_count: weak_exec_rank_metric,
            }
        })
        .collect();
    asset_rankings.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.asset.cmp(&b.asset))
    });

    apply_capacity_selection(&mut all_signals, &mut edge_transfers);

    let total_capital = env::var("TOTAL_CAPITAL")
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(10000.0); // Default to 10,000 as per example

    allocate_capital(&mut all_signals, total_capital);

    trade_count = all_signals
        .iter()
        .filter(|s| {
            matches!(s.action, SignalAction::BUY | SignalAction::SELL)
                && s.quantity.unwrap_or(0) > 0
        })
        .count();

    let holds = all_signals
        .iter()
        .filter(|s| s.action == SignalAction::HOLD)
        .count();
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
    let mut exit_by_reason: HashMap<ExitReason, usize> = HashMap::new();
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
        if let Some(exit_reason) = transfer.exit_reason {
            *exit_by_reason.entry(exit_reason).or_insert(0usize) += 1;
        }
    }
    let exit_count: usize = exit_by_reason.values().sum();
    let avg_exit_pnl = if exit_count > 0 {
        edge_transfers
            .iter()
            .filter(|t| t.exit_reason.is_some())
            .map(|t| t.current_pnl)
            .sum::<f64>()
            / exit_count as f64
    } else {
        0.0
    };
    let total_loss: f64 = loss_by_reason.values().sum();
    let mut loss_distribution: Vec<ReasonLossShare> = if total_loss > 0.0 {
        let mut rows: Vec<ReasonLossShare> = loss_by_reason
            .iter()
            .map(|(reason, loss)| ReasonLossShare {
                reason: reason.clone(),
                pct: ((*loss / total_loss) * 100.0).clamp(0.0, 100.0),
            })
            .collect();
        rows.sort_by(|a, b| {
            b.pct
                .partial_cmp(&a.pct)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
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
        exit_count,
        exit_by_reason,
        avg_exit_pnl,
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
            selection_cap_k: selection_cap::resolved_signal_top_k(),
            edge_loss_breakdown,
        },
        asset_name: assets
            .first()
            .cloned()
            .unwrap_or_else(|| "UNKNOWN".to_string()),
        asset_rankings,
    }
}

/// Grid over confidence/score floors. Disables edge override in the gate so floors can bind;
/// production snapshots still respect `EDGE_OVERRIDE_THRESHOLD` from the environment.
///
/// Set **`SWEEP_GATE_DEBUG=1`** to print one line of intrinsic `effective_confidence` / composite
/// score min–max–mean (scenarios above `MIN_TRADABLE_EDGE` only), sampled from the **first** grid cell
/// so you can see whether your floor range can bind.
pub fn run_threshold_sweep(
    assets: Vec<String>,
    global_lambda: f64,
    confidence_floors: &[f64],
    score_floors: &[f64],
    jsonl_path: Option<&str>,
    ga_config: Option<&GaConfig>,
) -> Vec<ThresholdSweepRow> {
    let sweep_gate_debug = std::env::var("SWEEP_GATE_DEBUG").ok().as_deref() == Some("1");
    let mut sweep_gate_metrics: Vec<(f64, f64)> = Vec::new();
    let mut first_cell = true;
    let mut rows: Vec<ThresholdSweepRow> = Vec::new();
    for &conf_raw in confidence_floors {
        for &score_raw in score_floors {
            // f105: Temporarily relax floors for debugging
            let confidence_floor = (conf_raw * 0.5).max(0.01);
            let score_floor = (score_raw * 0.5).max(0.01);

            let snapshot = if first_cell && sweep_gate_debug {
                generate_latest_signals_with_thresholds_internal(
                    assets.clone(),
                    global_lambda,
                    confidence_floor,
                    score_floor,
                    None,
                    jsonl_path,
                    ga_config,
                    true,
                    Some(&mut sweep_gate_metrics),
                )
            } else {
                generate_latest_signals_with_thresholds_internal(
                    assets.clone(),
                    global_lambda,
                    confidence_floor,
                    score_floor,
                    None,
                    jsonl_path,
                    ga_config,
                    true,
                    None,
                )
            };
            first_cell = false;
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

    if sweep_gate_debug {
        print_sweep_gate_intrinsic_summary(&sweep_gate_metrics);
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
    let min_tradable_edge = resolved_min_tradable_edge();
    let edge_override_threshold = resolved_edge_override_threshold();
    let confidence_floor = resolved_signal_confidence_floor(DEFAULT_CONFIDENCE_FLOOR);
    let score_floor = resolved_signal_score_floor(DEFAULT_SCORE_FLOOR);
    let data_source = env::var("DATA_SOURCE")
        .unwrap_or_else(|_| "folder".to_string())
        .to_lowercase();

    let folder_path = "/Users/nikhil/ChronoSentiment_MEGA_FINAL/test_assets".to_string();
    let mut folder_candles_by_asset: HashMap<String, Vec<Candle>> = HashMap::new();

    let assets_to_process: Vec<(String, String)> = if data_source == "folder" {
        let source = FolderCandleSource { folder_path };
        let datasets = source.load_all();
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
        let scenario_map = if data_source == "folder" {
            let candles = folder_candles_by_asset
                .get(&asset_name)
                .cloned()
                .unwrap_or_default();
            scenarios_from_candles(&asset_name, &candles)
        } else if data_source == "csv" && !csv_path.is_empty() && Path::new(&csv_path).exists() {
            let source: Box<dyn CandleSource> = Box::new(CsvCandleSource {
                path: csv_path.clone(),
            });
            let candles = source.get_candles_sync();
            scenarios_from_candles(&asset_name, &candles)
        } else {
            HashMap::new()
        };

        if scenario_map.is_empty() {
            continue;
        }

        let Some((initial_price, initial_timestamp)) =
            initial_order_from_scenario_map(&scenario_map)
        else {
            continue;
        };

        let config = GaConfig {
            population_size: 5,
            generations: 3,
            mutation_rate: 0.1,
            seed: 42,
            order_id_prefix: format!("REAL_GA_{}", asset_name),
            order_price: initial_price,
            order_quantity_for_strategy: 100,
            order_timestamp: initial_timestamp,
            lambda: global_lambda,
            initial_queue_threshold: 200,
            ..GaConfig::default()
        };

        let mut sorted_names: Vec<String> = scenario_map.keys().cloned().collect();
        sorted_names.sort();

        let test_index = (config.seed as usize) % sorted_names.len();
        let test_scenario_name = sorted_names[test_index].clone();

        let mut train_scenarios_map: HashMap<String, Vec<MarketEvent>> = HashMap::new();
        for name in &sorted_names {
            if name != &test_scenario_name {
                if let Some(events) = scenario_map.get(name) {
                    train_scenarios_map.insert(name.clone(), events.clone());
                }
            }
        }
        let train_px_pairs = map_to_pairs(&asset_name, &train_scenarios_map);
        let ga_result = ga::run_ga_evolution(config.clone(), &train_px_pairs);

        let mut pnls_all = Vec::with_capacity(sorted_names.len());
        let mut execution_fitnesses_all = Vec::with_capacity(sorted_names.len());
        let mut traded_pnls = Vec::with_capacity(sorted_names.len());
        let mut traded_scenarios = 0usize;
        let mut weak_executed_count = 0usize;

        for name in &sorted_names {
            if let Some(events) = scenario_map.get(name) {
                let mut one_scenario_map: HashMap<String, Vec<MarketEvent>> = HashMap::new();
                one_scenario_map.insert(name.clone(), events.clone());
                let one_scenario_px_pairs = map_to_pairs(&asset_name, &one_scenario_map);

                let (detected_regime, confidence) = detect_regime_from_events(events.as_slice());
                let regime_key = format!("{}_{}", asset_name, detected_regime.as_str());
                let selected_eval = ga_result
                    .best_per_regime
                    .get(&regime_key)
                    .unwrap_or(&ga_result.global_best);

                if let Some(report) = ga::evaluate_and_aggregate(
                    &selected_eval.strategy,
                    &config,
                    &one_scenario_px_pairs,
                    0,
                    0.0,
                    0,
                ) {
                    let gate = evaluate_gate(
                        detected_regime,
                        confidence,
                        report.fitness,
                        confidence_floor,
                        score_floor,
                        min_tradable_edge,
                        edge_override_threshold,
                    );

                    if gate.trade_allowed {
                        traded_scenarios += 1;
                        if confidence < 0.60 {
                            weak_executed_count += 1;
                        }
                        traded_pnls.push(report.avg_pnl);
                        pnls_all.push(report.avg_pnl);
                        execution_fitnesses_all.push(report.fitness);
                    } else {
                        pnls_all.push(0.0);
                        execution_fitnesses_all.push(0.0);
                    }
                }
            } else {
                pnls_all.push(0.0);
                execution_fitnesses_all.push(0.0);
            }
        }

        if !pnls_all.is_empty() {
            let mean_pnl = pnls_all.iter().sum::<f64>() / pnls_all.len() as f64;
            let pnl_variance = pnls_all.iter().map(|p| (p - mean_pnl).powi(2)).sum::<f64>()
                / pnls_all.len() as f64;
            let std_dev = pnl_variance.sqrt();
            let worst = pnls_all.iter().copied().fold(f64::INFINITY, f64::min);
            let _mean_execution_fitness =
                execution_fitnesses_all.iter().sum::<f64>() / execution_fitnesses_all.len() as f64;

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
                metric: format!("PnL_Asset_{}", asset_name),
                mean: mean_pnl,
                std_dev,
                min: worst,
                max: mean_pnl,
            });
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
    }

    aggregated_metrics
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MarketEventType;

    #[test]
    fn test_detect_regime_is_deterministic() {
        let events = vec![
            MarketEvent {
                subtype: MarketEventType::Trade,
                price: 100,
                quantity: 1,
                side: None,
                exchange_ts: 1,
            },
            MarketEvent {
                subtype: MarketEventType::Trade,
                price: 101,
                quantity: 1,
                side: None,
                exchange_ts: 2,
            },
            MarketEvent {
                subtype: MarketEventType::Trade,
                price: 102,
                quantity: 1,
                side: None,
                exchange_ts: 3,
            },
            MarketEvent {
                subtype: MarketEventType::Trade,
                price: 103,
                quantity: 1,
                side: None,
                exchange_ts: 4,
            },
        ];
        let r1 = detect_regime_from_events(&events);
        let r2 = detect_regime_from_events(&events);
        assert_eq!(r1.0, r2.0);
        assert!((r1.1 - r2.1).abs() < 1e-12);
    }
}
