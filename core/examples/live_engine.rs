use chronosentiment_core::ga::{
    evaluate_current_status, load_elite_strategies, strategy_evaluation_for_live_reco_snapshot,
    update_paper_registry, component_diagnostic_snapshot, reset_component_diagnostic_counters,
    record_momentum_gate_event,
    DecisionReport, GaConfig, PaperRegistry, Strategy, SignalType,
    PercentileBuffer, DistributionStats, RankStats, TradeIntent, TradeRecommendation, finalize_paper_registry,
    close_active_trades_for_symbol, close_active_sketch_trades_on_side_flip,
};
use chronosentiment_core::reco::{RecommendationEngine, RecommendationResult, RecoConfig};
use chronosentiment_core::market_adapter::Candle;
use serde::{Deserialize, Serialize};
use std::io::{self, BufRead};
use std::collections::{HashMap, VecDeque};
use rand::prelude::*;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const PRICE_SCALE: f64 = 10000.0;
const BASE_POSITION_SIZE: f64 = 0.05; 

#[derive(Debug, Deserialize, Serialize)]
struct SymbolicCandle {
    pub symbol: String,
    pub timestamp: u64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

use std::sync::Mutex;

#[derive(Debug, Deserialize, Serialize)]
struct GovernorControl {
    pub gov_mult: f64,
    pub ts: u64,
}

/// Shared safety state for decoupling governor from data loop.
struct SafetyState {
    pub gov_mult: Arc<Mutex<f64>>,
    pub execution_enabled: Arc<AtomicBool>,
}

impl SafetyState {
    fn new() -> Self {
        Self {
            gov_mult: Arc::new(Mutex::new(1.0)), // Default to 1.0 until first read
            execution_enabled: Arc::new(AtomicBool::new(true)),
        }
    }

    fn update(&self, mult: f64, enabled: bool) {
        {
            let mut gm = self.gov_mult.lock().unwrap();
            *gm = mult;
        }
        self.execution_enabled.store(enabled, Ordering::Relaxed);
    }

    fn get_multiplier(&self) -> f64 {
        *self.gov_mult.lock().unwrap()
    }

    fn is_halted(&self) -> bool {
        !self.execution_enabled.load(Ordering::Relaxed)
    }
}

#[derive(Debug, Clone)]
struct RecommendationCandidate {
    rec_id: u64,
    symbol: String,
    score: f64,
    edge: f64,
    conf: f64,
    feas: f64,
    voters: usize,
    primary_id: usize,
    signal: SignalType,
    consistency: usize,
    recommendation: TradeRecommendation,
    /// Synthetic reco when strategy pool had no winner but momentum bootstrap qualified.
    from_bootstrap_bridge: bool,
    from_fallback: bool,
    pub mode: String,
    pub birth_price: f64,
    pub entry_path: String,
    pub regime: String,
    pub path_size_multiplier: f64,
    pub birth_timestamp: u64,
}

#[derive(Debug, Clone)]
struct RecMeta {
    rec_id: u64,
    symbol: String,
    score: f64,
    edge: f64,
    feas: f64,
    conf: f64,
    voters: usize,
    primary_id: usize,
    /// strategy | momentum_bootstrap — mirrors [RECOMMENDATION] src= (PnL attribution).
    reco_src: &'static str,
    from_fallback: bool,
}

#[derive(Debug, Clone)]
struct PendingConfirmation {
    candidate: RecommendationCandidate,
    created_symbol_updates: usize,
    base_price: f64,
    base_score: f64,
    base_vol: f64,
}

struct ShadowTrade {
    symbol: String,
    entry_price: f64,
    tp_target: f64,
    sl_target: f64,
    signal: SignalType,
    age: usize,
    max_age: usize,
    is_blocked: bool,
    is_random_baseline: bool,
}

#[derive(Debug, Clone)]
struct ShadowPending {
    symbol: String,
    signal: SignalType,
    birth_price: f64,
    realistic_entry: f64,
    ticks_waited: usize,
    max_favorable_excursion: f64,
    max_adverse_excursion: f64,
    horizon: usize,
    mode: String,
}

#[derive(Default)]
struct SideCounters {
    raw_bullish_events: u64,
    raw_bearish_events: u64,
    raw_wait_events: u64,
    buy_candidates: u64,
    sell_candidates: u64,
    buy_pass: u64,
    sell_pass: u64,
    buy_final: u64,
    sell_final: u64,
    buy_intents_created: u64,
    sell_intents_created: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RecommendationMode {
    Coverage,
    Precision,
    Top1,
}

impl RecommendationMode {
    fn from_env() -> Self {
        match std::env::var("REC_MODE")
            .unwrap_or_else(|_| "coverage".to_string())
            .to_lowercase()
            .as_str()
        {
            "precision" => RecommendationMode::Precision,
            "top1" => RecommendationMode::Top1,
            _ => RecommendationMode::Coverage,
        }
    }
}

/// Default gates from `REC_MODE` + `GA_BOOTSTRAP`, with optional `RECO_EDGE_MIN` override
/// (single-knob A/B for log-distribution calibration; same inputs → same behavior except the floor).
fn reco_gate_thresholds(
    rec_mode: RecommendationMode,
    bootstrap: bool,
) -> (f64, f64, f64, usize, f64) {
    let (mut edge_min, feas_min, conf_min, reco_min_voters, score_min) = match rec_mode {
        RecommendationMode::Coverage => {
            if bootstrap {
                (0.001, 0.05, 0.20, 1usize, 0.0002)
            } else {
                (0.0012, 0.40, 0.40, 2usize, 0.0010)
            }
        }
        RecommendationMode::Precision => {
            if bootstrap {
                (0.001, 0.70, 0.20, 1usize, 0.0020)
            } else {
                (0.0012, 0.70, 0.40, 2usize, 0.0020)
            }
        }
        RecommendationMode::Top1 => {
            if bootstrap {
                (0.001, 0.70, 0.20, 1usize, 0.0020)
            } else {
                (0.0012, 0.70, 0.40, 2usize, 0.0020)
            }
        }
    };
    if let Ok(v) = std::env::var("RECO_EDGE_MIN") {
        if let Ok(parsed) = v.parse::<f64>() {
            if parsed.is_finite() && parsed > 0.0 {
                edge_min = parsed;
            }
        }
    }
    (
        edge_min,
        feas_min,
        conf_min,
        reco_min_voters,
        score_min,
    )
}

fn env_flag(name: &str) -> bool {
    std::env::var(name).map_or(false, |v| {
        !v.is_empty() && v != "0" && !v.eq_ignore_ascii_case("false")
    })
}

fn env_parse_usize(name: &str, default: usize) -> usize {
    std::env::var(name)
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(default)
}

fn env_parse_f64_pos(name: &str, default: f64) -> f64 {
    std::env::var(name)
        .ok()
        .and_then(|s| s.parse::<f64>().ok())
        .filter(|v| v.is_finite() && *v > 0.0)
        .unwrap_or(default)
}

/// Deterministic minimal reco when strategy pool is silent but momentum bootstrap qualifies.
fn synthetic_momentum_trade_reco(
    symbol: &str,
    price_now: f64,
    mom_abs: f64,
    signal: SignalType,
) -> TradeRecommendation {
    let tp_bps = 10.0_f64; // Reduced for fast audit
    let sl_bps = 10.0_f64; // Reduced for fast audit
    let is_buy = signal == SignalType::BUY;
    let sign = if is_buy { 1.0 } else { -1.0 };
    
    TradeRecommendation {
        symbol: symbol.to_string(),
        signal,
        rank: 1.0,
        raw_edge: mom_abs,
        confidence: 0.55,
        quality_score: 0.5,
        entry_price: price_now,
        entry_low: price_now * 0.9999,
        entry_high: price_now * 1.0001,
        tp_target: price_now * (1.0 + sign * tp_bps / 10_000.0),
        sl_target: price_now * (1.0 - sign * sl_bps / 10_000.0),
        expected_rr: tp_bps / sl_bps.max(1.0),
        expected_edge_bps: mom_abs * 10_000.0,
        risk_bps: sl_bps,
        holding_bars: 3,
        vol_bps: 10.0,
        vol_bucket: 2,
        is_execution: true,
        position_size: BASE_POSITION_SIZE,
        directional_alpha: 0.0,
        execution_alpha: 0.0,
        structural_alpha: 0.0,
    }
}

fn percentile(mut values: Vec<f64>, p: f64) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let rank = ((p.clamp(0.0, 100.0) / 100.0) * ((values.len() - 1) as f64)).round() as usize;
    values[rank.min(values.len() - 1)]
}

fn percentile_sorted(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let len = sorted.len();
    let rank = ((p.clamp(0.0, 100.0) / 100.0) * ((len - 1) as f64)).round() as usize;
    sorted[rank.min(len - 1)]
}

fn rolling_close_std(history: &[Candle], window: usize) -> f64 {
    if history.len() < window || window == 0 {
        return 0.0;
    }
    let values: Vec<f64> = history
        .iter()
        .rev()
        .take(window)
        .map(|c| c.close as f64 / PRICE_SCALE)
        .collect();
    let n = values.len() as f64;
    if n <= 0.0 {
        return 0.0;
    }
    let mean = values.iter().sum::<f64>() / n;
    let var = values
        .iter()
        .map(|v| {
            let d = *v - mean;
            d * d
        })
        .sum::<f64>()
        / n;
    var.sqrt()
}

fn median_f64(vals: &mut [f64]) -> f64 {
    if vals.is_empty() {
        return 0.0;
    }
    vals.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let n = vals.len();
    if n % 2 == 1 {
        vals[n / 2]
    } else {
        (vals[n / 2 - 1] + vals[n / 2]) / 2.0
    }
}

/// Wall-clock minute bucket for dedup: supports ms (>1e10) or second timestamps.
fn sketch_minute_bucket(ts: u64) -> i64 {
    let t = ts as i64;
    if ts > 10_000_000_000u64 {
        t / 60_000 // ms → minute
    } else {
        t / 60 // seconds → minute
    }
}

/// Entry (median of last K closes) and risk (max high − min low over last N bars), aligned with `app.py` sketch.
fn trade_sketch_prices_from_candles(history: &[Candle], price_scale: f64) -> Option<(f64, f64)> {
    // Same as TRADE_MIN_RISK_FRAC in app.py (0.05% of entry).
    const MIN_RISK_FRAC: f64 = 0.0005;
    const MEDIAN_K: usize = 3;
    const RISK_BARS: usize = 5;
    if history.is_empty() {
        return None;
    }
    let take_m = MEDIAN_K.min(history.len());
    let mut closes: Vec<f64> = history[history.len() - take_m..]
        .iter()
        .map(|c| c.close as f64 / price_scale)
        .collect();
    let entry = median_f64(&mut closes);
    if !entry.is_finite() || entry <= 0.0 {
        return None;
    }
    let n = RISK_BARS.min(history.len());
    let slice = &history[history.len() - n..];
    let mut hi = f64::NEG_INFINITY;
    let mut lo = f64::INFINITY;
    for c in slice {
        hi = hi.max(c.high as f64 / price_scale);
        lo = lo.min(c.low as f64 / price_scale);
    }
    let risk = hi - lo;
    if !risk.is_finite() || risk <= 0.0 || risk < entry * MIN_RISK_FRAC {
        return None;
    }
    Some((entry, risk))
}

/// Deterministic fitness proxy for the reco population layer (edge + feas + paper perf).
fn live_reco_fitness_proxy(report: &DecisionReport, paper_perf: f64, stats: &DistributionStats) -> f64 {
    let edge = report.raw_edge.max(0.0);
    let feas = report.execution_feasibility.clamp(0.0, 1.0);
    let consistency = report.consistency as f64 / 10.0;
    let perf = paper_perf.clamp(0.0, 1.0);

    // --- CORE TERMS ---

    let edge_term = (edge / stats.p90.max(0.001)).clamp(0.0, 3.0).powf(1.5);
    let capture_term = feas.powf(2.0);
    let stability_term = consistency.powf(1.2).max(0.5);
    let perf_term = (0.5 + 0.5 * perf).clamp(0.3, 1.5);

    edge_term * capture_term * stability_term * perf_term
}

impl SymbolicCandle {
    fn to_core_candle(&self) -> Candle {
        Candle {
            timestamp: self.timestamp,
            open: (self.open * PRICE_SCALE) as u64,
            high: (self.high * PRICE_SCALE) as u64,
            low: (self.low * PRICE_SCALE) as u64,
            close: (self.close * PRICE_SCALE) as u64,
            volume: self.volume as u64,
        }
    }
}

/// Trained elites from `ELITE_PATH` (JSON bundle), else orthogonal specialist seeds.
fn load_strategies_for_paper() -> Vec<Strategy> {
    let path = std::env::var("ELITE_PATH").unwrap_or_default();
    if !path.is_empty() {
        let p = std::path::Path::new(&path);
        if p.is_file() {
            let mut evals = load_elite_strategies(&path);
            if !evals.is_empty() {
                evals.sort_by(|a, b| {
                    b.fitness
                        .partial_cmp(&a.fitness)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
                let n = evals.len().min(8);
                println!(
                    "📂 Paper/live: loaded {} trained strateg(ies) from {} (using top {} by fitness)",
                    evals.len(),
                    path,
                    n
                );
                return evals
                    .into_iter()
                    .take(n)
                    .enumerate()
                    .map(|(i, mut e)| {
                        e.strategy.lineage = i;
                        e.strategy
                    })
                    .collect();
            }
            eprintln!(
                "⚠️ ELITE_PATH={} present but load_elite_strategies returned empty (invalid JSON?)",
                path
            );
        } else {
            eprintln!("⚠️ ELITE_PATH={} is not a file — using built-in specialists", path);
        }
    }
    create_specialist_strategies()
}

fn create_specialist_strategies() -> Vec<Strategy> {
    let mut strats = Vec::new();
    
    // 1. Mean Reversion Specialist (L0)
    let mut s0 = Strategy::from_seed(100);
    s0.archetype = 2; // Reversion
    s0.take_profit = 15;
    s0.stop_loss = 20;
    s0.lineage = 0;
    strats.push(s0);
    
    // 2. Momentum Chaser (L1)
    let mut s1 = Strategy::from_seed(200);
    s1.archetype = 1; // Momentum
    s1.take_profit = 40;
    s1.stop_loss = 15;
    s1.lineage = 1;
    strats.push(s1);
    
    // 3. Liquidity Provider (L2) - High Selectivity
    let mut s2 = Strategy::from_seed(300);
    s2.archetype = 3; // Volatility
    s2.selectivity = 95; 
    s2.lineage = 2;
    strats.push(s2);
    
    // 4. Low-Latency Sniper (L3) - Low Entry Offset
    let mut s3 = Strategy::from_seed(400);
    s3.entry_offset = -2;
    s3.lineage = 3;
    strats.push(s3);
    
    // 5. Anchor Strategy (L4) - Balanced
    let mut s4 = Strategy::from_seed(500);
    s4.lineage = 4;
    strats.push(s4);
    
    strats
}

fn main() {
    println!("⚡ CHRONOSENTIMENT LIVE ENGINE | Mode: Orthogonal Specialists");
    let rec_mode = RecommendationMode::from_env();
    println!("📌 Recommendation mode: {:?}", rec_mode);
    let blocked_symbols: std::collections::HashSet<String> = std::env::var("REC_BLOCK_SYMBOLS")
        .unwrap_or_else(|_| "ADANIENT.NS,ADANIPORTS.NS".to_string())
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    println!("🚫 Blocked recommendation symbols: {:?}", blocked_symbols);
    
    let mut rng = StdRng::seed_from_u64(42);
    
    // --- LOAD trained elites (ELITE_PATH) or built-in specialists ---
    let mut strategies = load_strategies_for_paper();
    println!("🧠 Running paper registry with {} strateg(ies).", strategies.len());
    
    let mut config = GaConfig::default();
    let mut paper = PaperRegistry::default();
    paper.max_concurrent = std::env::var("PAPER_MAX_CONCURRENT")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(10);
    paper.adaptation_threshold = 30; 
    
    let mut edge_buffer = PercentileBuffer::new(500);
    let mut mom_abs_buffer = PercentileBuffer::new(500);
    let mut fallback_history: VecDeque<u8> = VecDeque::with_capacity(500);
    let mut current_stats = DistributionStats::default();
    
    let mut history_pipes: HashMap<String, Vec<Candle>> = HashMap::new();
    // Last k signs of momentum_contribution for bootstrap consistency (sym → deque).
    let mut mom_sign_hist: HashMap<String, VecDeque<i8>> = HashMap::new();
    let mut score_history: HashMap<String, VecDeque<f64>> = HashMap::new();
    let mut symbol_update_counts: HashMap<String, usize> = HashMap::new();
    let mut pending_confirmations: HashMap<String, PendingConfirmation> = HashMap::new();
    let mut side_counters = SideCounters::default();
    reset_component_diagnostic_counters();
    let mut last_component_diag = component_diagnostic_snapshot();
    let mut prev_accel_map: HashMap<String, f64> = HashMap::new();
    let mut directional_streak_map: HashMap<String, (f64, i32)> = HashMap::new();
    let mut range_3_map: HashMap<String, VecDeque<f64>> = HashMap::new();
    let mut tick_history_map: HashMap<String, VecDeque<f64>> = HashMap::new();
    let mut inflection_confirmations: HashMap<String, RecommendationCandidate> = HashMap::new();
    let mut shadow_evals: Vec<ShadowPending> = Vec::new();
    let mut confirmed_birth_count = 0;
    let mut shadow_counterfactuals: Vec<ShadowTrade> = Vec::new();
    let mut last_signals: HashMap<String, SignalType> = HashMap::new();
    let mut prev_imbalance_map: HashMap<String, f32> = HashMap::new();
    let mut prev_delta_imb_map: HashMap<String, f32> = HashMap::new();
    let mut shadow_accel_hits: HashMap<String, usize> = HashMap::new();
    
    // Parameter sweep support
    let drift_gate = std::env::var("DRIFT_GATE").unwrap_or_else(|_| "3.0".to_string()).parse::<f64>().unwrap_or(3.0);
    let accel_gate = std::env::var("ACCEL_GATE").unwrap_or_else(|_| "1.5".to_string()).parse::<f64>().unwrap_or(1.5);
    let mut consistency_counts: HashMap<String, usize> = HashMap::new();
    
    let stdin = io::stdin();
    let mut total_processed = 0;
    let mut last_adaptation_count = 0;
    let mut next_rec_id: u64 = 1;
    let mut pending_meta: HashMap<String, VecDeque<RecMeta>> = HashMap::new();
    let mut active_meta: HashMap<String, VecDeque<RecMeta>> = HashMap::new();
    // Per symbol: last sketch minute bucket and side (same-minute side flip allowed; duplicate same side blocked).
    let mut sketch_emit_state: HashMap<String, (i64, SignalType)> = HashMap::new();
    let mut next_rank_stats = RankStats::zeroed();
    let mut next_rank_obs = 0usize;
    let rankstats_commit_trades = std::env::var("RANKSTATS_COMMIT_TRADES")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(20)
        .max(1);
    let rec_min_move = std::env::var("REC_MIN_MOVE")
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(0.001);
    let signal_decay_exit = std::env::var("PAPER_SIGNAL_DECAY")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);
    let signal_edge_min = std::env::var("PAPER_SIGNAL_EDGE_MIN")
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(0.0005);
    let confirm_delta = std::env::var("REC_CONFIRM_DELTA")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(3)
        .max(1);
    let confirm_vol_mult = std::env::var("REC_CONFIRM_VOL_MULT")
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(1.5)
        .max(1.0);
    let candidate_voter_percentile = std::env::var("REC_CAND_VOTER_PCT")
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(60.0)
        .clamp(0.0, 100.0);
    let candidate_conf_percentile = std::env::var("REC_CAND_CONF_PCT")
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(75.0)
        .clamp(0.0, 100.0);
    let intent_max_age_base = std::env::var("INTENT_MAX_AGE_BASE")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(14)
        .max(1);
    let intent_max_age_strong = std::env::var("INTENT_MAX_AGE_STRONG")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(14)
        .max(intent_max_age_base);
    let intent_high_voters_threshold = std::env::var("INTENT_HIGH_VOTERS_THRESHOLD")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(6)
        .max(1);
    let intent_high_conf_threshold = std::env::var("INTENT_HIGH_CONF_THRESHOLD")
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(0.70)
        .clamp(0.0, 1.0);

    let live_gate_reco_stability = std::env::var("LIVE_GATE_RECO_STABILITY_MIN")
        .ok()
        .and_then(|s| s.parse::<f64>().ok());
    let live_gate_reco_ag_global = std::env::var("LIVE_GATE_RECO_AGREEMENT_GLOBAL_MIN")
        .ok()
        .and_then(|s| s.parse::<f64>().ok());
    let live_gate_reco_fitness = std::env::var("LIVE_GATE_RECO_FITNESS_MIN")
        .ok()
        .and_then(|s| s.parse::<f64>().ok());
    let run_reco_engine = live_gate_reco_stability.is_some()
        || live_gate_reco_ag_global.is_some()
        || live_gate_reco_fitness.is_some()
        || env_flag("POOL_DEBUG")
        || env_flag("RECO_DEBUG");
    let reco_path_probe = env_flag("RECO_PATH_PROBE");
    let strat_probe = env_flag("STRAT_PROBE");
    let reco_single_accept_diag = env_flag("REC_SINGLE_ACCEPT_DIAG");
    let candidate_probe = env_flag("CANDIDATE_PROBE");
    let mut cand_batches = 0usize;
    let mut cand_total_sum = 0usize;
    let mut cand_voters_pos_sum = 0usize;
    let mut cand_feas_pos_sum = 0usize;
    let mut cand_stage1_pass_sum = 0usize;
    let mut cand_admitted_sum = 0usize;

    // --- SAFETY & DATA DECOUPLING ---
    let safety = Arc::new(SafetyState::new());
    let (tx, rx) = mpsc::channel::<String>();
    
    // Thread 1: Governor Polling (200ms cadence) with hysteresis
    let s_thread = Arc::clone(&safety);
    thread::spawn(move || {
        let path = "analysis/real_live/governor_state.json";
        let mut stale_count = 0;
        loop {
            let (mult, enabled) = match std::fs::read_to_string(path) {
                Ok(data) => {
                    match serde_json::from_str::<GovernorControl>(&data) {
                        Ok(gov) => {
                            let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
                            let stale = now > gov.ts + 10;
                            if stale {
                                stale_count += 1;
                            } else {
                                stale_count = 0;
                            }
                            let hysteresis_stale = stale_count > 10; // 2 seconds at 200ms
                            let enabled = gov.gov_mult > 0.0 && !hysteresis_stale;
                            (if enabled { gov.gov_mult.clamp(0.0, 1.0) } else { 0.0 }, enabled)
                        }
                        Err(_) => (0.0, false),
                    }
                }
                Err(_) => (0.0, false),
            };
            
            let was_enabled = s_thread.execution_enabled.load(Ordering::Relaxed);
            s_thread.update(mult, enabled);
            
            if !enabled && was_enabled {
                println!("[SAFETY] HALT enforced (gov_mult=0.00)");
            } else if enabled && !was_enabled {
                println!("[SAFETY] Recovery active (gov_mult={:.2})", mult);
            }
            
            thread::sleep(Duration::from_millis(200));
        }
    });

    // Thread 2: Stdin Reader (Non-blocking feed)
    thread::spawn(move || {
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            if let Ok(l) = line {
                if tx.send(l).is_err() { break; }
            }
        }
    });

    println!("📡 Listening for candles (Async Safety Loop Active)...");
    println!(
        "   Paper bridge: PAPER_SKETCH_INTENTS=1 → sketch overlay + fill at first post-confirm bar open; minute dedup uses ms timestamps when ts>1e10 else seconds."
    );
    println!(
        "   Optional gates: LIVE_GATE_EDGE_STABILITY_MIN=, LIVE_GATE_CONF_MIN=, LIVE_GATE_RECO_STABILITY_MIN= (reco S), LIVE_GATE_RECO_AGREEMENT_GLOBAL_MIN= (reco G), LIVE_GATE_RECO_FITNESS_MIN= (medoid fitness); POOL_DEBUG=1 / RECO_DEBUG=1"
    );
    println!(
        "   Momentum voter bootstrap (off unless set): MOMENTUM_VOTER_BOOTSTRAP=1 with MOMENTUM_BOOTSTRAP_FLOOR (tape-calibrated via scripts/grid_search_momentum_bootstrap.py), MOMENTUM_BOOTSTRAP_CONSISTENCY_K — RECOMMENDATION lines include src=strategy|momentum_bootstrap. BOOTSTRAP_DRIFT_DIAG=1 logs [BOOTSTRAP_DRIFT] (p90/92/95, current_floor, ratio_floor_to_p92~1, buffer_size; no gate changes)."
    );
    println!(
        "   Live reco uses small proxy pools → S/G/F read weaker than train_nse; start S around 0.35–0.55 (not ~0.8). [DIAG] FINAL=1 = meta-gates only; emission still needs edge/feas/p90/voters/blocklist."
    );

    // One stdin line == one synchronized timestep across symbols (streamer batch) = one AWR window.
    let mut awr_windows_total: u64 = 0;
    let mut awr_windows_with_candidates: u64 = 0;
    let mut awr_windows_triggered: u64 = 0;

    loop {
        // 1. Safety Heartbeat & Intent Purge
        // if safety.is_halted() {
        //     if !paper.pending_intents.is_empty() {
        //         println!("[SAFETY] Clearing {} pending intents due to Governor HALT.", paper.pending_intents.len());
        //         paper.pending_intents.clear();
        //     }
        // }

        // 2. Data Ingestion (Timeout enables time-based safety enforcement)
        let line = match rx.recv_timeout(Duration::from_millis(100)) {
            Ok(l) => l,
            Err(mpsc::RecvTimeoutError::Timeout) => {
                // if safety.is_halted() {
                //     println!("[GATE_REJECT] Governor HALT active → skipping tick");
                // }
                continue; // 10Hz safety heartbeat
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        };

        if safety.is_halted() {
            println!("[GATE_REJECT] Governor HALT active → skipping tick");
            continue;
        }

        if line.trim().is_empty() { continue; }
        
        let incoming: Vec<SymbolicCandle> = match serde_json::from_str(&line) { Ok(c) => c, Err(_) => continue };
        let batch_ts = incoming.first().map(|c| c.timestamp).unwrap_or(0);
        let mut symbol_ts_parts: Vec<String> = incoming
            .iter()
            .map(|c| format!("{}:{}", c.symbol, c.timestamp))
            .collect();
        symbol_ts_parts.sort();
        if !symbol_ts_parts.is_empty() {
            println!("[SYMBOL_TS] {}", symbol_ts_parts.join(","));
        }
        let mut symbol_price_parts: Vec<String> = incoming
            .iter()
            .map(|c| {
                let px = format!("{:.12}", c.close);
                let px = px.trim_end_matches('0').trim_end_matches('.');
                let px = if px.is_empty() { "0" } else { px };
                format!("{}:{}", c.symbol.trim(), px)
            })
            .collect();
        symbol_price_parts.sort();
        if !symbol_price_parts.is_empty() {
            println!("[SYMBOL_PRICE] {}", symbol_price_parts.join(","));
        }
        awr_windows_total = awr_windows_total.saturating_add(1);
        let line_start_triggered = paper.intents_triggered;
        let mut recommendations: Vec<RecommendationCandidate> = Vec::new();
        let mut symbol_best_reports: HashMap<String, DecisionReport> = HashMap::new();

        for sym_candle in incoming {
            total_processed += 1;
            let symbol = &sym_candle.symbol;
            let candle = sym_candle.to_core_candle();
            let pre_closed = paper.closed_count;
            let pre_pnl_len = paper.pnl_history.len();
            let mut active_before: HashMap<String, usize> = HashMap::new();
            for t in &paper.active_trades {
                *active_before.entry(t.symbol.clone()).or_insert(0) += 1;
            }
            
            let history = history_pipes.entry(symbol.clone()).or_insert_with(Vec::new);
            history.push(candle.clone());
            if history.len() > 1000 { history.remove(0); }
            *symbol_update_counts.entry(symbol.clone()).or_insert(0) += 1;
            
            // --- SETTLE SHADOW COUNTERFACTUALS ---
            let mut settled_indices = Vec::new();
            for (i, st) in shadow_counterfactuals.iter_mut().enumerate() {
                if st.symbol == *symbol {
                    st.age += 1;
                    let p_now = candle.close as f64 / PRICE_SCALE;
                    let pnl = if st.signal == SignalType::BUY {
                        (p_now - st.entry_price) / st.entry_price
                    } else {
                        (st.entry_price - p_now) / st.entry_price
                    };
                    
                    let hit_tp = if st.signal == SignalType::BUY { p_now >= st.tp_target } else { p_now <= st.tp_target };
                    let hit_sl = if st.signal == SignalType::BUY { p_now <= st.sl_target } else { p_now >= st.sl_target };
                    
                    if hit_tp || hit_sl || st.age >= st.max_age {
                        // ALWAYS record to raw baseline
                        paper.record_raw_signal(pnl);
                        
                        // IF blocked, also record to blocked preservation
                        if st.is_blocked {
                            paper.record_block_pnl(pnl);
                        }
                        
                        // IF random baseline, record to random pool
                        if st.is_random_baseline {
                            paper.record_random_signal(pnl);
                        } else {
                            // IF it's a real/raw candidate signal, record to the pnl variance pool for sigma calculation
                            paper.record_pnl_sample(pnl);
                        }
                        
                        settled_indices.push(i);
                    }
                }
            }
            for &i in settled_indices.iter().rev() {
                shadow_counterfactuals.remove(i);
            }
            let sym_updates_now = *symbol_update_counts.get(symbol).unwrap_or(&0);
            let price_now = candle.close as f64 / PRICE_SCALE;
            let price_prev = if history.len() >= 2 {
                history[history.len() - 2].close as f64 / PRICE_SCALE
            } else {
                price_now
            };
            let volume_now = candle.volume as f64;
            let volume_prev = if history.len() >= 2 {
                history[history.len() - 2].volume as f64
            } else {
                volume_now
            };
            let delta_price = price_now - price_prev;
            
            let streak_entry = directional_streak_map.entry(symbol.clone()).or_insert((0.0, 0));
            if (delta_price > 0.0 && streak_entry.0 > 0.0) || (delta_price < 0.0 && streak_entry.0 < 0.0) {
                streak_entry.1 += 1;
            } else {
                streak_entry.1 = 1;
                streak_entry.0 = delta_price;
            }
            let current_streak = streak_entry.1;

            let price_range = if history.len() >= 1 {
                let last = history.last().unwrap();
                (last.high as f64 - last.low as f64) / PRICE_SCALE
            } else { 0.0 };
            let r3_buf = range_3_map.entry(symbol.clone()).or_insert_with(|| VecDeque::with_capacity(3));
            r3_buf.push_back(price_range);
            if r3_buf.len() > 3 { r3_buf.pop_front(); }
            let avg_r3 = if !r3_buf.is_empty() { r3_buf.iter().sum::<f64>() / r3_buf.len() as f64 } else { 0.0 };
            
            let tick_buf = tick_history_map.entry(symbol.clone()).or_insert_with(|| VecDeque::with_capacity(10));
            tick_buf.push_back(delta_price);
            if tick_buf.len() > 10 { tick_buf.pop_front(); }
            
            let up_ticks = tick_buf.iter().filter(|&&d| d > 0.0).count();
            let down_ticks = tick_buf.iter().filter(|&&d| d < 0.0).count();
            let imbalance = if !tick_buf.is_empty() {
                (up_ticks as f32 - down_ticks as f32).abs() / tick_buf.len() as f32
            } else { 0.0 };

            let delta_volume = volume_now - volume_prev;
            let price_k5 = if history.len() > 5 {
                history[history.len() - 6].close as f64 / PRICE_SCALE
            } else {
                price_prev
            };
            let price_k10 = if history.len() > 10 {
                history[history.len() - 11].close as f64 / PRICE_SCALE
            } else {
                price_prev
            };
            let price_k15 = if history.len() > 15 {
                history[history.len() - 16].close as f64 / PRICE_SCALE
            } else {
                price_prev
            };
            let price_k20 = if history.len() > 20 {
                history[history.len() - 21].close as f64 / PRICE_SCALE
            } else {
                price_prev
            };
            let price_k30 = if history.len() > 30 {
                history[history.len() - 31].close as f64 / PRICE_SCALE
            } else {
                price_prev
            };
            let delta_k5 = price_now - price_k5;
            let delta_k10 = price_now - price_k10;
            let delta_k15 = price_now - price_k15;
            let delta_k20 = price_now - price_k20;
            let delta_k30 = price_now - price_k30;
            let mut distinct_ref_price = price_now;
            let mut events_back_to_distinct: usize = 0;
            if history.len() >= 2 {
                for back in 1..history.len() {
                    let candidate = history[history.len() - 1 - back].close as f64 / PRICE_SCALE;
                    if candidate != price_now {
                        distinct_ref_price = candidate;
                        events_back_to_distinct = back;
                        break;
                    }
                }
            }
            let delta_distinct = price_now - distinct_ref_price;
            let threshold_bps = 5.0;
            let threshold_abs_scaled = (threshold_bps / 10_000.0) * price_now.max(1.0);
            let ratio_bps_tick = 10_000.0 * (delta_price / price_now.max(1.0));
            let ratio_bps_k5 = 10_000.0 * (delta_k5 / price_now.max(1.0));
            let ratio_bps_k10 = 10_000.0 * (delta_k10 / price_now.max(1.0));
            let ratio_bps_k15 = 10_000.0 * (delta_k15 / price_now.max(1.0));
            let ratio_bps_k20 = 10_000.0 * (delta_k20 / price_now.max(1.0));
            let ratio_bps_k30 = 10_000.0 * (delta_k30 / price_now.max(1.0));
            let ratio_bps_distinct = 10_000.0 * (delta_distinct / price_now.max(1.0));
            let trigger_momentum_3 = if history.len() >= 4 {
                let last = history[history.len() - 1].close as f64 / PRICE_SCALE;
                let lag3 = history[history.len() - 4].close as f64 / PRICE_SCALE;
                last - lag3
            } else {
                0.0
            };
            let trigger_vol_5 = rolling_close_std(history, 5);

            update_paper_registry(
                &mut paper,
                &candle,
                symbol,
                sym_updates_now,
                trigger_momentum_3,
                trigger_vol_5,
            );
            if !paper.closed_observations.is_empty() {
                for obs in paper.closed_observations.drain(..) {
                    let r_bucket = (obs.rank * 10.0).floor().clamp(0.0, 9.0) as usize;
                    let v_bucket = obs.vol_bucket.min(4);
                    next_rank_stats.bucket_mfe_sum[r_bucket][v_bucket] += obs.mfe.clamp(0.0, 0.04);
                    next_rank_stats.bucket_mae_sum[r_bucket][v_bucket] += obs.mae_abs.clamp(0.0, 0.03);
                    next_rank_stats.bucket_time_sum[r_bucket][v_bucket] += obs.hold_bars as f64;
                    next_rank_stats.bucket_count[r_bucket][v_bucket] += 1;
                    next_rank_obs += 1;
                }
                if next_rank_obs >= rankstats_commit_trades {
                    config.rank_stats.blend_with_min_count(next_rank_stats, 0.2, 1);
                    next_rank_stats = RankStats::zeroed();
                    next_rank_obs = 0;
                    if std::env::var("RANKSTATS_PROBE").is_ok() {
                        println!("[RANKSTATS_COMMIT] committed_observations={}", rankstats_commit_trades);
                        let mut values: Vec<f64> = Vec::with_capacity(10 * 5);
                        let mut non_default = 0usize;
                        for r in 0..10 {
                            for v in 0..5 {
                                let c = config.rank_stats.bucket_count[r][v].max(1);
                                let mfe = config.rank_stats.bucket_mfe_sum[r][v] / c as f64;
                                values.push(mfe);
                                if (config.rank_stats.bucket_mfe_sum[r][v] - 0.0045).abs() > 1e-9
                                    || config.rank_stats.bucket_count[r][v] != 1
                                {
                                    non_default += 1;
                                }
                            }
                        }
                        let n = values.len().max(1) as f64;
                        let mean_mfe = values.iter().sum::<f64>() / n;
                        let var = values
                            .iter()
                            .map(|x| {
                                let d = x - mean_mfe;
                                d * d
                            })
                            .sum::<f64>()
                            / n;
                        let std_mfe = var.sqrt();
                        let min_mfe = values.iter().copied().fold(f64::INFINITY, f64::min);
                        let max_mfe = values
                            .iter()
                            .copied()
                            .fold(f64::NEG_INFINITY, f64::max);
                        println!(
                            "[RANKSTATS_LIVE] non_default={} mean_mfe={:.6} std_mfe={:.6} min_mfe={:.6} max_mfe={:.6}",
                            non_default,
                            mean_mfe,
                            std_mfe,
                            min_mfe,
                            max_mfe
                        );
                    }
                }
            }
            let mut active_after: HashMap<String, usize> = HashMap::new();
            for t in &paper.active_trades {
                *active_after.entry(t.symbol.clone()).or_insert(0) += 1;
            }
            for (sym, after_count) in &active_after {
                let before_count = *active_before.get(sym).unwrap_or(&0);
                if *after_count > before_count {
                    let delta = *after_count - before_count;
                    let entry = pending_meta.entry(sym.clone()).or_default();
                    let dst = active_meta.entry(sym.clone()).or_default();
                    for _ in 0..delta {
                        if let Some(meta) = entry.pop_front() {
                            dst.push_back(meta);
                        }
                    }
                }
            }
            if paper.closed_count > pre_closed && paper.pnl_history.len() > pre_pnl_len {
                let mut closed_metas: Vec<RecMeta> = Vec::new();
                for (sym, before_count) in &active_before {
                    let after_count = *active_after.get(sym).unwrap_or(&0);
                    if *before_count > after_count {
                        let delta = *before_count - after_count;
                        let src = active_meta.entry(sym.clone()).or_default();
                        for _ in 0..delta {
                            if let Some(meta) = src.pop_front() {
                                closed_metas.push(meta);
                            }
                        }
                    }
                }
                let new_pnls = &paper.pnl_history[pre_pnl_len..];
                for (meta, pnl) in closed_metas.into_iter().zip(new_pnls.iter().copied()) {
                    println!(
                        "[REC_OUTCOME] rec_id={} sym={} score={:.6} edge={:.6} feas={:.3} conf={:.3} voters={} S{} pnl={:.6} src={}",
                        meta.rec_id,
                        meta.symbol,
                        meta.score,
                        meta.edge,
                        meta.feas,
                        meta.conf,
                        meta.voters,
                        meta.primary_id,
                        pnl,
                        meta.reco_src
                    );
                }
            }

            // 🔥 STABILIZED WALK-FORWARD ADAPTATION (Active)
            if paper.closed_count > 0 && paper.closed_count % paper.adaptation_threshold == 0 && paper.closed_count != last_adaptation_count {
                last_adaptation_count = paper.closed_count;
                if rng.gen_bool(0.2) {
                    let mut perfs: Vec<(usize, f64)> = (0..strategies.len()).map(|i| (i, paper.get_strategy_performance(i))).collect();
                    perfs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

                    let best_id = perfs[0].0;
                    let worst_id = perfs.last().unwrap().0;
                    
                    if worst_id != 4 && perfs.last().unwrap().1 < -0.0010 { // Only replace if truly failing
                        println!("\x1b[93m[STABLE_EVO] Pruning S{} (perf={:.6}) -> Mutant of S{}\x1b[0m", 
                            worst_id, perfs.last().unwrap().1, best_id);
                        
                        let parent_lineage = strategies[best_id].lineage;
                        let siblings = strategies.iter().filter(|s| s.lineage == parent_lineage).count();
                        
                        if siblings < 2 {
                            let mut new_strat = strategies[best_id].mutate(42 + paper.closed_count as u64);
                            new_strat.lineage = parent_lineage;
                            strategies[worst_id] = new_strat;
                            paper.strategy_pnl.insert(worst_id, 0.0);
                            paper.strategy_counts.insert(worst_id, 0);
                        }
                    }
                }
            }

            if history.len() >= 2 {
                let mut current_stats = edge_buffer.get_stats();
                let fallback_ratio = current_stats.p90 / current_stats.p65.max(0.0001);
                
                let bootstrap = std::env::var("GA_BOOTSTRAP").is_ok();
                let min_feas = if bootstrap { 0.05 } else { 0.40 };
                let allow_fallback = fallback_ratio < 0.40;
                let use_fallback = bootstrap && allow_fallback;
                let edge_gate = if use_fallback {
                    current_stats.p65.max(0.0002)
                } else {
                    current_stats.p90.max(0.0008)
                };

                let mut buy_strength = 0.0;
                let mut sell_strength = 0.0;
                let mut buy_voters = 0usize;
                let mut sell_voters = 0usize;
                let mut shared_raw_edge = 0.0;
                let mut best_reco = None;
                let mut max_rank = -1.0;
                let mut best_report: Option<DecisionReport> = None;
                let mut primary_id = 0;
                let mut selected_edge = 0.0;
                let mut selected_feasibility = 0.0;
                let mut total_feasibility = 0.0;
                let mut voted_count = 0;
                let mut reject_no_reco = 0usize;
                let mut reject_nonpositive_edge = 0usize;
                let mut reject_low_feas = 0usize;
                // Raw edges from strategies that produced a recommendation (cross-strategy spread → structural stability).
                let mut edges_with_reco: Vec<f64> = Vec::new();
                let mut reco_population = Vec::new();

                let mut p90 = 0.0007; // Default/warmup
                let mut p94 = 0.0009;
                let mut p95 = 0.0012;
                let mut p98 = 0.0020;

                if mom_abs_buffer.buffer_len() >= 100 {
                    let vals = mom_abs_buffer.sorted_values();
                    p90 = percentile_sorted(&vals, 90.0);
                    p94 = percentile_sorted(&vals, 94.0);
                    p95 = percentile_sorted(&vals, 95.0);
                    p98 = percentile_sorted(&vals, 98.0);

                    if total_processed % 500 == 0 {
                        println!(
                            "[BOOTSTRAP_THRESH] p90={:.6} p94={:.6} p95={:.6} p98={:.6}",
                            p90, p94, p95, p98
                        );
                    }
                }

                let edge_gate = 0.0001;
                let min_feas = 0.0; // Forced zero for audit phase
                let min_conf = 0.00;
                let min_voters_required = 1;

                for (idx, strat) in strategies.iter().enumerate() {
                    let last_sig = last_signals.get(symbol).cloned().unwrap_or(SignalType::WAIT);
                    let cons = consistency_counts.get(symbol).cloned().unwrap_or(0);
                    let report = evaluate_current_status(strat, history, &config, symbol, last_sig, cons, &current_stats);
                    match report.signal {
                        SignalType::BUY => side_counters.raw_bullish_events = side_counters.raw_bullish_events.saturating_add(1),
                        SignalType::SELL => side_counters.raw_bearish_events = side_counters.raw_bearish_events.saturating_add(1),
                        SignalType::WAIT => side_counters.raw_wait_events = side_counters.raw_wait_events.saturating_add(1),
                    }
                    if strat_probe && total_processed % 100 == 0 {
                        println!(
                            "[STRAT_OUT] sym={} strat_id={} raw_edge={:.6} feas={:.3} active={} has_reco={}",
                            symbol,
                            idx,
                            report.raw_edge,
                            report.execution_feasibility,
                            (report.raw_edge >= edge_gate && report.execution_feasibility >= min_feas) as i32,
                            report.recommendation.is_some() as i32
                        );
                    }
                    if run_reco_engine {
                        let paper_perf = paper.get_strategy_performance(idx);
                        let fit = live_reco_fitness_proxy(&report, paper_perf, &current_stats);
                        let cap = report
                            .capture_efficiency
                            .unwrap_or_else(|| report.execution_feasibility.mul_add(2.0, -1.0))
                            .clamp(-1.0, 1.0);
                        reco_population.push(strategy_evaluation_for_live_reco_snapshot(
                            strat.clone(),
                            fit,
                            report.conviction_score,
                            report.execution_feasibility,
                            cap,
                        ));
                    }
                    
                    let effective_raw_edge = report.raw_edge;
                    if effective_raw_edge > shared_raw_edge {
                        shared_raw_edge = effective_raw_edge;
                    }

                    // Mutually exclusive rejection accounting:
                    // low_edge -> low_feas -> no_reco -> voter.
                    if effective_raw_edge < edge_gate {
                        reject_nonpositive_edge += 1;
                        continue;
                    }
                    if report.execution_feasibility < min_feas {
                        reject_low_feas += 1;
                        continue;
                    }
                    if let Some(ref reco) = report.recommendation {
                        edges_with_reco.push(effective_raw_edge);
                        if signal_decay_exit
                            && effective_raw_edge < signal_edge_min
                            && paper.active_trades.iter().any(|t| t.symbol == *symbol)
                        {
                            let _closed = close_active_trades_for_symbol(
                                &mut paper,
                                symbol,
                                &candle,
                                "SIGNAL_DECAY",
                            );
                        }
                        total_feasibility += report.execution_feasibility;
                        voted_count += 1;

                        let perf = paper.get_strategy_performance(idx);
                        let weight = (1.0 + perf * 50.0).clamp(0.1, 2.0);
                        let w_rank_sq = (reco.rank * reco.rank) * weight;

                        if reco.signal == SignalType::BUY { buy_strength += w_rank_sq; buy_voters += 1; }
                        else if reco.signal == SignalType::SELL { sell_strength += w_rank_sq; sell_voters += 1; }

                        if reco.rank > max_rank {
                            max_rank = reco.rank;
                            primary_id = idx;
                            best_reco = Some((reco.clone(), report.signal, report.consistency));
                            best_report = Some(report.clone());   // ✅ Capture the winning report
                            selected_edge = effective_raw_edge;
                            selected_feasibility = report.execution_feasibility;
                        }

                        // 🔥 SURGICAL DEBUG: See what the Momentum Chaser (S1) sees
                        if idx == 1 && total_processed % 50 == 0 {
                             println!("[EDGE_DEBUG] S1 | edge={:.6} feas={:.3} rank={:.3} sig={:?}", 
                                effective_raw_edge, report.execution_feasibility, reco.rank, reco.signal);
                        }
                    } else {
                        reject_no_reco += 1;
                    }
                }

                // --- Fallback tracking (CORRECT: Based on winning strategy) ---
                let fallback_applied_this_symbol = best_report
                    .as_ref()
                    .map(|r| r.fallback_applied)
                    .unwrap_or(false);

                if fallback_applied_this_symbol {
                    fallback_history.push_back(1);
                } else {
                    fallback_history.push_back(0);
                }
                if fallback_history.len() > 500 {
                    fallback_history.pop_front();
                }

                if let Some(ref r) = best_report {
                    symbol_best_reports.insert(symbol.clone(), r.clone());
                }

                if shared_raw_edge > 0.0 {
                    edge_buffer.push(shared_raw_edge);
                    current_stats = edge_buffer.get_stats();
                }

                let raw_momentum = if price_now.abs() > 1e-12 {
                    delta_k30 / price_now
                } else {
                    0.0
                };
                let momentum_weight = 1.0;
                let momentum_contribution = raw_momentum * momentum_weight;
                mom_abs_buffer.push(momentum_contribution.abs());

                // --- REGIME DETECTION & ACCELERATION ---
                // --- WEIGHTED TREND DETECTION ---
                let t15 = if history.len() >= 15 {
                    let last = history.last().unwrap().close as f64 / PRICE_SCALE;
                    let start = history[history.len() - 15].close as f64 / PRICE_SCALE;
                    (last - start) / start.max(1e-12)
                } else { 0.0 };
                
                let t60 = if history.len() >= 60 {
                    let last = history.last().unwrap().close as f64 / PRICE_SCALE;
                    let start = history[history.len() - 60].close as f64 / PRICE_SCALE;
                    (last - start) / start.max(1e-12)
                } else if history.len() >= 2 {
                    let last = history.last().unwrap().close as f64 / PRICE_SCALE;
                    let start = history[0].close as f64 / PRICE_SCALE;
                    (last - start) / start.max(1e-12)
                } else { 0.0 };

                let trend_strength = 0.7 * t15 + 0.3 * t60;

                let accel_bps = if history.len() >= 10 {
                    let p_now = history.last().unwrap().close as f64 / PRICE_SCALE;
                    let p_lag5 = history[history.len()-5].close as f64 / PRICE_SCALE;
                    let p_lag10 = history[history.len()-10].close as f64 / PRICE_SCALE;
                    
                    let v_now = (p_now - p_lag5) / p_lag5.max(1e-12);
                    let v_prev = (p_lag5 - p_lag10) / p_lag10.max(1e-12);
                    (v_now - v_prev) * 10000.0
                } else {
                    0.0
                };

                let pre_move_bps = if history.len() >= 3 {
                    let last = history.last().unwrap().close as f64 / PRICE_SCALE;
                    let prev3 = history[history.len() - 3].close as f64 / PRICE_SCALE;
                    (last - prev3) / prev3.max(1e-12) * 10000.0
                } else {
                    0.0
                };

                let regime = if trend_strength > 0.0002 { "BULL" }
                            else if trend_strength < -0.0002 { "BEAR" }
                            else { "RANGE" };
                
                println!("[REGIME] sym={} trend_bps={:.1} accel_bps={:.2} pre_move={:.2} regime={}", 
                    symbol, trend_strength * 10000.0, accel_bps, pre_move_bps, regime);

                // --- MOMENTUM BOOTSTRAP (deterministic) ---
                let mut bootstrap_active = false;
                let mut bootstrap_edge = 0.0;
                let mut bootstrap_direction: i32 = 0; // +1 buy, -1 sell

                let momentum_abs = momentum_contribution.abs();

                let floor = p94;

                let k_required: usize = std::env::var("MOMENTUM_BOOTSTRAP_CONSISTENCY_K")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(4);

                // maintain rolling sign history (Vec<i8>)
                let sign = if momentum_contribution > 0.0 { 1 } else if momentum_contribution < 0.0 { -1 } else { 0 };

                let hist = mom_sign_hist.entry(symbol.clone()).or_default();
                hist.push_back(sign as i8);
                if hist.len() > k_required {
                    hist.pop_front();
                }

                let consistent = hist.len() == k_required
                    && hist.iter().all(|&s| s == (sign as i8) && s != 0);

                let raw_voter_sum = buy_voters + sell_voters;
                let raw_voters = if reco_single_accept_diag {
                    voted_count.max(raw_voter_sum)
                } else {
                    raw_voter_sum
                };
                let voters = raw_voters;

                // Force bootstrap for audit phase
                bootstrap_active = true;
                bootstrap_edge = 0.0050; 
                
                // --- REGIME FILTERED DIRECTION ---
                bootstrap_direction = if sign != 0 { sign } else { 1 };
                let is_buy = bootstrap_direction > 0;
                
                // Overrule direction if regime is strong
                if regime == "BULL" && !is_buy {
                    bootstrap_active = false; // Kill lagging sells in bull trend
                    println!("[REGIME_BLOCK] sym={} dir=SELL regime=BULL (blocking lagging short)", symbol);
                } else if regime == "BEAR" && is_buy {
                    bootstrap_active = false; // Kill lagging buys in bear trend
                    println!("[REGIME_BLOCK] sym={} dir=BUY regime=BEAR (blocking lagging long)", symbol);
                }

                // --- MODE & ENTRY QUALITY ---
                let mut mode = "UNKNOWN";
                let mut quality_decision = "PASS";
                let mut quality_reason = "";
                let mut selected_path = "none";
                let mut path_size_multiplier: f64 = 1.0;

                if bootstrap_active {
                    // MODE SEPARATION
                    let is_trending = regime != "RANGE";
                    let is_momentum = if is_buy { trend_strength > 0.0 } else { trend_strength < 0.0 };
                    
                    mode = if is_trending && is_momentum { "MOMENTUM" } else { "REVERSION" };

                    // --- VALIDATED INFLECTION TRADING FILTERS ---
                    let prev_accel = *prev_accel_map.get(symbol).unwrap_or(&0.0);
                    prev_accel_map.insert(symbol.clone(), accel_bps);
                    
                    // Tight Compression requirement
                    let vol_10 = rolling_close_std(history, 10);
                    let compression_score = if vol_10 > 1e-12 { avg_r3 / vol_10 } else { 0.0 };
                    let compression = compression_score < 0.8; // Coil requirement
                    let recent_window = 5;
                    let recent_history: &[Candle] = if history.len() >= recent_window { 
                        &history[history.len()-recent_window..] 
                    } else { 
                        &history[..] 
                    };
                    let up_count = recent_history.windows(2).filter(|w| w[1].close > w[0].close).count();
                    let down_count = recent_history.windows(2).filter(|w| w[1].close < w[0].close).count();
                    let dir_bias = if is_buy { up_count as f64 / (recent_window-1) as f64 } else { down_count as f64 / (recent_window-1) as f64 };
                    let last_tick_ok = if is_buy { 
                        history.len() >= 2 && history.last().unwrap().close > history[history.len()-2].close 
                    } else { 
                        history.len() >= 2 && history.last().unwrap().close < history[history.len()-2].close 
                    };

                    if mode == "MOMENTUM" {
                        // 1. Directional Persistence Filter
                        if dir_bias < 0.4 || !last_tick_ok {
                            quality_decision = "REJECT";
                            quality_reason = "no_directional_persistence";
                        }
                        // 1. Impulse Birth Detection (Softened)
                        let impulse_birth = prev_accel.abs() < 0.4 && accel_bps.abs() > 0.4;
                        if !impulse_birth {
                            path_size_multiplier *= 0.5;
                        }
                        
                        // 2. Pre-move cap (Softened)
                        if pre_move_bps.abs() > 10.0 {
                            quality_decision = "REJECT";
                            quality_reason = "move_exhausted_hard";
                        } else if pre_move_bps.abs() > 5.0 {
                            path_size_multiplier *= 0.5;
                        }

                        // 3. Strong Compression & Imbalance
                        if compression_score > 1.2 {
                            quality_decision = "REJECT";
                            quality_reason = "expansion_chase_hard";
                        } else if compression_score > 0.8 {
                            path_size_multiplier *= 0.5;
                        }
                        let prev_imb = prev_imbalance_map.get(symbol).cloned().unwrap_or(0.0f32);
                        let delta_imb = imbalance - prev_imb;
                        let prev_delta = prev_delta_imb_map.get(symbol).cloned().unwrap_or(0.0f32);
                        let accel_imb = delta_imb - prev_delta;
                        
                        let mut entry_path = "none";
                        if imbalance > 0.15f32 && delta_imb > 0.0f32 && accel_imb > 0.0f32 {
                            entry_path = "impulse";
                        } else if imbalance > 0.08f32 && delta_imb > 0.0f32 {
                            entry_path = "micro";
                        }

                        if entry_path == "none" {
                            quality_decision = "REJECT";
                            quality_reason = if imbalance < 0.08f32 { "low_imbalance_level" } else { "low_imbalance_velocity" };
                            if imbalance > 0.15f32 && delta_imb > 0.0f32 && accel_imb <= 0.0f32 {
                                quality_reason = "low_imbalance_accel";
                                // Mark for shadow accel hit tracking
                                shadow_accel_hits.insert(symbol.clone(), 0);
                            }
                            paper.record_rejection("imbalance");
                        } else {
                            // Store path in a temporary variable to be used later
                        }
                        selected_path = entry_path;

                        // 4. Freshness check (Softened)
                        if current_streak > 8 { // Relaxed slightly
                            quality_decision = "REJECT";
                            quality_reason = "late_trend_hard";
                        } else if current_streak > 4 {
                            path_size_multiplier *= 0.5;
                        }
                    } else if mode == "REVERSION" {
                        // 1. Reversal Persistence Filter
                        if dir_bias < 0.2 && !last_tick_ok { // Looser for reversion
                            quality_decision = "REJECT";
                            quality_reason = "no_reversal_persistence";
                        }
                        // 2. Acceleration Collapse Detection (Inflection)
                        let is_collapse = if is_buy {
                            prev_accel < -0.3 && accel_bps > -0.1 // Softer thresholds
                        } else {
                            prev_accel > 0.3 && accel_bps < 0.1
                        };

                        if !is_collapse {
                            path_size_multiplier *= 0.5;
                        }
                        
                        let prev_imb = prev_imbalance_map.get(symbol).cloned().unwrap_or(0.0f32);
                        let delta_imb = imbalance - prev_imb;
                        let prev_delta = prev_delta_imb_map.get(symbol).cloned().unwrap_or(0.0f32);
                        let accel_imb = delta_imb - prev_delta;

                        let mut entry_path = "none";
                        if imbalance > 0.05f32 && delta_imb > 0.0f32 && accel_imb > 0.0f32 {
                            entry_path = "impulse";
                        } else if imbalance > 0.02f32 && delta_imb > 0.0f32 {
                            entry_path = "micro";
                        }

                        if entry_path == "none" {
                            quality_decision = "REJECT";
                            quality_reason = if imbalance < 0.02f32 { "low_imb_level" } else { "low_imb_vel" };
                            if imbalance > 0.05f32 && delta_imb > 0.0f32 && accel_imb <= 0.0f32 {
                                quality_reason = "low_imb_accel";
                                shadow_accel_hits.insert(symbol.clone(), 0);
                            }
                            paper.record_rejection("imbalance");
                        }
                        selected_path = entry_path;
                    }
                    
                    // --- REGIME ADAPTIVE EQUITY GUARD ---
                    let regime_stats = paper.regime_metrics.get(regime);
                    let current_regime_fbpr = if let Some((count, _, _, _, _, fbpr, _, _)) = regime_stats {
                        if *count > 20 { *fbpr as f64 / *count as f64 } else { 1.0 }
                    } else { 1.0 };
                    
                    let regime_multiplier = if current_regime_fbpr < 0.20 { 0.1 } else { 1.0 };
                    
                    if regime_multiplier < 1.0 {
                         println!("[EQUITY_GUARD] Throttling probes in {} regime (FBPR={:.2})", regime, current_regime_fbpr);
                    }
                    path_size_multiplier *= regime_multiplier;

                    // --- EXECUTABLE OPPORTUNITY SCORE (EOS) ---
                    let queue_pressure = imbalance.abs() as f64;
                    let queue_clearance = 1.0 / (events_back_to_distinct as f64 + 1.0);
                    let event_density = (updates_60 as f64 / 60.0).clamp(0.1, 1.0); // No price action used
                    let velocity_mult = if is_buy == (accel_imb > 0.0) { 1.2 } else { 0.8 };
                    let eos = (queue_pressure * queue_clearance * event_density) * velocity_mult;
                    
                    // RAW SIGNAL TRACKING: Capture everything that passes the first gate for baseline audit
                    if quality_decision == "PASS" {
                         // We track the raw signal as a shadow trade to get its 'unfiltered' expectancy
                         shadow_counterfactuals.push(ShadowTrade {
                            symbol: symbol.clone(),
                            entry_price: price_now,
                            tp_target: if is_buy { price_now * 1.0005 } else { price_now * 0.9995 },
                            sl_target: if is_buy { price_now * 0.9995 } else { price_now * 1.0005 },
                            signal: if is_buy { SignalType::BUY } else { SignalType::SELL },
                            age: 0,
                            max_age: 20,
                            is_blocked: false,
                            is_random_baseline: false,
                        });
                        // Record raw signal count/pnl is deferred to shadow settlement but we increment count here for baseline
                        // Actually, let's just record it at settlement time to paper.record_raw_signal(pnl).
                    }

                    // --- PRE-TRADE MICROSTRUCTURE GATE (ANTICIPATORY) ---
                    let is_exhausting = (is_buy && accel_imb < -0.01) || (!is_buy && accel_imb > 0.01);
                    let is_mature = current_streak > 4;
                    let is_low_opportunity = eos < 0.01; // Floor for viability
                    
                    if is_exhausting || is_mature || is_low_opportunity {
                        if quality_decision == "PASS" {
                            let reason = if is_exhausting { "Exhaustion" } else if is_mature { "Maturity" } else { "LowOpportunity" };
                            println!("[PRE_TRADE_BLOCK] sym={} reason={} streak={} eos={:.4} accel_imb={:.4}", 
                                symbol, reason, current_streak, eos, accel_imb);
                            quality_decision = "BLOCKED";
                            paper.record_block();
                            
                            // UPDATE SHADOW TRADE TO BLOCKED
                            if let Some(st) = shadow_counterfactuals.iter_mut().rev().find(|s| s.symbol == *symbol && s.age == 0) {
                                st.is_blocked = true;
                            }
                        }
                    } else {
                        // POSITIVE SELECTION: Scale by EOS
                        let eos_multiplier = (eos * 10.0).clamp(0.5, 2.0);
                        if eos_multiplier > 1.2 {
                             println!("[EOS_BOOST] sym={} eos={:.4} mult={:.2}", symbol, eos, eos_multiplier);
                        }
                        path_size_multiplier *= eos_multiplier;
                    }

                    // --- 1-TICK CONFIRMATION LAYER ---
                    let is_explosive = accel_bps.abs() > accel_gate && pre_move_bps.abs() < 4.0;
                    
                    if quality_decision == "PASS" {
                        if is_explosive {
                            println!("[EXPLOSIVE_IMPULSE] sym={} dir={:?} accel={:.2} (bypassing confirmation)", 
                                symbol, if is_buy { SignalType::BUY } else { SignalType::SELL }, accel_bps);
                            paper.record_probe_emit();
                            paper.record_probe_confirm();
                            // Immediate pass (no pending confirm)
                        } else {
                            // PRE-CONFIRMATION PROBE: Emit small probe immediately to bypass structural latency
                            let probe_multiplier = 0.2;
                            let mut probe_cand = RecommendationCandidate {
                                rec_id: next_rec_id,
                                symbol: symbol.clone(),
                                score: 0.0,
                                edge: selected_edge,
                                conf: 1.0,
                                feas: 1.0,
                                voters: 1,
                                primary_id: 0,
                                signal: if is_buy { SignalType::BUY } else { SignalType::SELL },
                                consistency: 1,
                                recommendation: TradeRecommendation {
                                    symbol: symbol.clone(),
                                    signal: if is_buy { SignalType::BUY } else { SignalType::SELL },
                                    rank: 1.0,
                                    raw_edge: 0.0,
                                    confidence: 0.0,
                                    quality_score: 1.0,
                                    directional_alpha: 0.0,
                                    execution_alpha: 0.0,
                                    structural_alpha: 0.0,
                                    entry_price: price_now,
                                    entry_low: price_now,
                                    entry_high: price_now,
                                    tp_target: if is_buy { price_now * 1.0005 } else { price_now * 0.9995 },
                                    sl_target: if is_buy { price_now * 0.9995 } else { price_now * 1.0005 },
                                    expected_rr: 0.0,
                                    expected_edge_bps: 0.0,
                                    risk_bps: 5.0,
                                    holding_bars: 20,
                                    vol_bps: 0.0,
                                    vol_bucket: 0,
                                    is_execution: true,
                                    position_size: ((if selected_path == "micro" { 0.25 } else { 1.0 }) * path_size_multiplier.max(0.1)) * probe_multiplier,
                                },
                                from_bootstrap_bridge: true,
                                from_fallback: false,
                                mode: mode.to_string(),
                                birth_price: price_now,
                                entry_path: selected_path.to_string(),
                                regime: regime.to_string(),
                                path_size_multiplier,
                                birth_timestamp: history.last().map(|c| c.timestamp).unwrap_or(0),
                            };
                            
                            // Push probe immediately
                            recommendations.push(probe_cand.clone());
                            paper.record_probe_emit();
                            next_rec_id += 1;

                            // Store for confirmation (remaining 0.8x)
                            inflection_confirmations.insert(symbol.clone(), probe_cand);
                            
                            quality_decision = "PENDING_CONFIRM";
                            bootstrap_active = false; 
                        }
                    }

                    // --- 1-TICK INFLECTION PROCESSING ---
                    if let Some(pending) = inflection_confirmations.get(symbol) {
                        let confirmed = if pending.signal == SignalType::BUY {
                            delta_price >= 0.0 // Faster Confirmation: Allow flat tick
                        } else {
                            delta_price <= 0.0 // Faster Confirmation: Allow flat tick
                        };

                        if confirmed {
                            println!("[CONFIRMED] sym={} dir={:?} streak={}", symbol, pending.signal, current_streak);
                            
                            let entry_p = price_now;
                            let birth_p = pending.recommendation.entry_price;
                            let drift_bps = (entry_p - birth_p).abs() / birth_p.max(1.0) * 10000.0;
                            
                            if drift_bps > drift_gate {
                                println!("[CONFIRM_REJECT] sym={} dir={:?} drift={:.2}bps (exceeded MAX_ENTRY_DRIFT)", symbol, pending.signal, drift_bps);
                                paper.record_rejection("drift");
                                // KILL THE PROBE: Drift too high, abort structural scale-up and exit probe
                                paper.force_close_trades_by_symbol(symbol, chronosentiment_core::ExitType::NoMomentum, price_now, history.last().unwrap().timestamp);
                                inflection_confirmations.remove(symbol);
                            } else {
                                paper.record_probe_confirm();
                                
                                // SCALE UP: Emit the remainder (0.8x)
                                let mut confirmed_cand = pending.clone();
                                confirmed_cand.rec_id = next_rec_id;
                                next_rec_id += 1;
                                
                                let risk_bps = 5.0; 
                                let tp_p = if confirmed_cand.signal == SignalType::BUY { entry_p * (1.0 + risk_bps/100.0) } else { entry_p * (1.0 - risk_bps/100.0) };
                                let sl_p = if confirmed_cand.signal == SignalType::BUY { entry_p * (1.0 - risk_bps/500.0) } else { entry_p * (1.0 + risk_bps/500.0) };
                                
                                confirmed_cand.recommendation.entry_price = entry_p;
                                confirmed_cand.birth_price = birth_p;
                                confirmed_cand.recommendation.tp_target = tp_p;
                                confirmed_cand.recommendation.sl_target = sl_p;
                                
                                // Size is the remaining 80% of the calculated path size
                                confirmed_cand.recommendation.position_size = ((if confirmed_cand.entry_path == "micro" { 0.25 } else { 1.0 }) * confirmed_cand.path_size_multiplier.max(0.1)) * 0.8;
                                
                                recommendations.push(confirmed_cand);
                                inflection_confirmations.remove(symbol);
                            }
                        } else {
                            println!("[CONFIRM_FAILED] sym={} dir={:?} delta={:.4}", symbol, pending.signal, delta_price);
                            paper.record_rejection("confirm");
                            // KILL THE PROBE: Confirmation failed, exit the probe immediately
                            paper.force_close_trades_by_symbol(symbol, chronosentiment_core::ExitType::NoMomentum, price_now, history.last().unwrap().timestamp);
                            inflection_confirmations.remove(symbol);
                        }
                    }
                    
                    // --- SHADOW TRACKING UPDATE ---
                    for shadow in &mut shadow_evals {
                        if shadow.symbol == *symbol {
                            shadow.ticks_waited += 1;
                            let mark_p = price_now;
                            
                            // MFE calculation (from realistic entry)
                            let current_favorable = if shadow.signal == SignalType::BUY {
                                (mark_p - shadow.realistic_entry) / shadow.realistic_entry * 10000.0
                            } else {
                                (shadow.realistic_entry - mark_p) / shadow.realistic_entry * 10000.0
                            };
                            if current_favorable > shadow.max_favorable_excursion {
                                shadow.max_favorable_excursion = current_favorable;
                            }
                            
                            // MAE calculation (from realistic entry)
                            let current_adverse = if shadow.signal == SignalType::BUY {
                                (shadow.realistic_entry - mark_p) / shadow.realistic_entry * 10000.0
                            } else {
                                (mark_p - shadow.realistic_entry) / shadow.realistic_entry * 10000.0
                            };
                            if current_adverse > shadow.max_adverse_excursion {
                                shadow.max_adverse_excursion = current_adverse;
                            }
                        }
                    }
                    
                    shadow_evals.retain(|shadow| {
                        if shadow.ticks_waited >= shadow.horizon {
                            // Realistic win: Did MFE exceed slippage cost (2bps exit)
                            let is_profitable = shadow.max_favorable_excursion > 2.0; 
                            println!("[SHADOW_EVAL] sym={} dir={:?} mode={} birth={:.4} entry_realistic={:.4} mfe={:.2}bps mae={:.2}bps realistic_pnl={:.4} outcome={}", 
                                shadow.symbol, shadow.signal, shadow.mode, shadow.birth_price, shadow.realistic_entry,
                                shadow.max_favorable_excursion, shadow.max_adverse_excursion,
                                shadow.max_favorable_excursion - 2.0,
                                if is_profitable { "WIN" } else { "LOSS" });
                            paper.record_shadow_outcome(is_profitable);
                            false
                        } else {
                            true
                        }
                    });
                    
                    let dir_accel = if is_buy { accel_bps } else { -accel_bps };
                    let trend_bps = trend_strength * 10000.0;
                    let directional_trend = if is_buy { trend_bps } else { -trend_bps };
                    let directional_projected = directional_trend + dir_accel;
                    
                    // 1. Projected Momentum Check: Allow deceleration, but not reversal.
                    // If the next tick is projected to be less than 2bps in our direction, reject.
                    if directional_projected < 2.0 { 
                        quality_decision = "REJECT";
                        quality_reason = "projected_reversal";
                    }

                    // 2. Sign-Consistency Filter (Directional Persistence)
                    let ticks = tick_history_map.get(symbol).map(|q| q.iter().cloned().collect::<Vec<f64>>()).unwrap_or_default();
                    if ticks.len() >= 5 && quality_decision == "PASS" {
                        let window = &ticks[ticks.len()-5..];
                        let against = window.iter().filter(|&&t| if is_buy { t < -0.1 } else { t > 0.1 }).count();
                        if against > 1 {
                            quality_decision = "REJECT";
                            quality_reason = "sign_choppiness";
                        }
                    }

                    if quality_decision == "PASS" {
                        println!("[TRADING_ACTIVE] sym={} dir={:?} mode={}", symbol, if is_buy { SignalType::BUY } else { SignalType::SELL }, mode);
                    }

                    if quality_decision == "REJECT" {
                        bootstrap_active = false;
                    }
                    
                    let prev_imb = prev_imbalance_map.get(symbol).cloned().unwrap_or(0.0f32);
                    let delta_imb = imbalance - prev_imb;
                    let prev_delta = prev_delta_imb_map.get(symbol).cloned().unwrap_or(0.0f32);
                    let accel_imb = delta_imb - prev_delta;

                    println!("[ENTRY_QUALITY] sym={} dir={:?} trend_bps={:.1} accel_bps={:.2} imb={:.2} delta_imb={:.2} accel_imb={:.2} pre_move={:.2} mode={} decision={} reason={}",
                        symbol, if is_buy { SignalType::BUY } else { SignalType::SELL }, 
                        trend_strength * 10000.0, accel_bps, imbalance, delta_imb, accel_imb, pre_move_bps, mode, quality_decision, quality_reason);
                    
                    // Update prev imbalance & delta
                    prev_imbalance_map.insert(symbol.clone(), imbalance);
                    prev_delta_imb_map.insert(symbol.clone(), delta_imb);
                }



                let total_strength = buy_strength + sell_strength + 0.001;
                let mut conf = (buy_strength - sell_strength).abs() / total_strength;
                if bootstrap_active {
                    conf = 1.0;
                }
                let final_sig = if buy_strength > sell_strength {
                    SignalType::BUY
                } else {
                    SignalType::SELL
                };

                let avg_feasibility = if voted_count > 0 {
                    total_feasibility / voted_count as f64
                } else {
                    0.0
                };
                let decision_feasibility = if selected_feasibility > 0.0 {
                    selected_feasibility
                } else {
                    avg_feasibility
                };

                let min_conf = if bootstrap { 0.10 } else { 0.40 };
                let min_voters_required = if bootstrap || bootstrap_active {
                    1
                } else {
                    2
                };
                let effective_voters = if bootstrap_active {
                    1
                } else {
                    voters
                };
                let voters = effective_voters;

                let is_high_conf = bootstrap_active || (conf >= min_conf
                    && voters >= min_voters_required);
                // Bootstrap floors feasibility at 0.05; strict `>` would reject exactly 0.05 (dead zone).
                let is_capturable = if bootstrap {
                    decision_feasibility >= min_feas
                } else if bootstrap_active {
                    decision_feasibility.max(min_feas) > min_feas - 1e-12
                } else {
                    decision_feasibility > min_feas
                };
                if strat_probe && total_processed % 100 == 0 {
                    println!(
                        "[STRAT_AGG] sym={} total_strats={} active_strats={} voters={} buy_voters={} sell_voters={} diag_single_accept={}",
                        symbol,
                        strategies.len(),
                        voted_count,
                        voters,
                        buy_voters,
                        sell_voters,
                        reco_single_accept_diag as i32
                    );
                }
                let effective_edge = if bootstrap_active {
                    bootstrap_edge
                } else {
                    shared_raw_edge
                };

                let edge_after_floor = if effective_edge >= edge_gate {
                    effective_edge
                } else {
                    0.0
                };
                // Diagnostic proxy only; does not affect strategy decisions.
                let norm_momentum = (raw_momentum / 0.001).clamp(-1.0, 1.0);
                let composite_contribution = selected_edge.max(0.0);
                let score_contribution = (selected_edge * conf).max(0.0);
                let voters_pre_count = voters;
                let voters_post_count = if voters_pre_count >= min_voters_required {
                    voters_pre_count
                } else {
                    0
                };

                // Optional structural gates (unset = off). Uses cross-strategy edge dispersion as a
                // stability proxy: mean/(std+eps), capped like reco stability (higher = edges agree).
                let live_gate_edge_stab = std::env::var("LIVE_GATE_EDGE_STABILITY_MIN")
                    .ok()
                    .and_then(|s| s.parse::<f64>().ok());
                let live_gate_conf_floor = std::env::var("LIVE_GATE_CONF_MIN")
                    .ok()
                    .and_then(|s| s.parse::<f64>().ok());
                let edge_stability = if edges_with_reco.len() >= 2 {
                    let em = edges_with_reco.iter().sum::<f64>() / edges_with_reco.len() as f64;
                    let var = edges_with_reco
                        .iter()
                        .map(|e| (e - em).powi(2))
                        .sum::<f64>()
                        / edges_with_reco.len() as f64;
                    let es = var.sqrt();
                    (em / (es + 1e-9)).min(5.0)
                } else {
                    0.0
                };
                let pass_edge_stability = live_gate_edge_stab.map_or(true, |req| {
                    if edges_with_reco.len() < 2 {
                        return false;
                    }
                    edge_stability >= req
                });
                let pass_conf_floor = live_gate_conf_floor.map_or(true, |req| conf >= req);
                let pass_edge_stability_eff =
                    pass_edge_stability || bootstrap_active;

                let (pass_reco_structure, reco_diag_s, reco_diag_g, reco_diag_f) =
                    if run_reco_engine && !reco_population.is_empty() {
                        let reco_cfg = RecoConfig::default();
                        let res = RecommendationEngine::process(
                            &reco_population,
                            &[],
                            &reco_cfg,
                            symbol,
                        );
                        let m = match &res {
                            RecommendationResult::Trade(r) => &r.ensemble_metrics,
                            RecommendationResult::WeakSignal(r) => &r.ensemble_metrics,
                            RecommendationResult::NoTrade { metrics, .. } => metrics,
                        };
                        let stab_ok =
                            live_gate_reco_stability.map_or(true, |t| m.stability >= t);
                        let agree_ok =
                            live_gate_reco_ag_global.map_or(true, |t| m.agreement_global >= t);
                        let fit_ok =
                            live_gate_reco_fitness.map_or(true, |t| m.medoid_fitness >= t);
                        let pass = stab_ok && agree_ok && fit_ok;
                        if reco_path_probe && total_processed % 100 == 0 {
                            println!(
                                "[RECO_POOL] sym={} run_reco_engine=1 pool_size={} feas={:.3} voters={} reco_S={:.3} reco_G={:.3} reco_F={:.3}",
                                symbol,
                                reco_population.len(),
                                avg_feasibility,
                                voters,
                                m.stability,
                                m.agreement_global,
                                m.medoid_fitness
                            );
                            println!(
                                "[RECO_BREAKDOWN] sym={} stab_ok={} agree_ok={} fit_ok={} stab_req={} agree_req={} fit_req={}",
                                symbol,
                                stab_ok as i32,
                                agree_ok as i32,
                                fit_ok as i32,
                                live_gate_reco_stability
                                    .map(|v| format!("{:.3}", v))
                                    .unwrap_or_else(|| "off".to_string()),
                                live_gate_reco_ag_global
                                    .map(|v| format!("{:.3}", v))
                                    .unwrap_or_else(|| "off".to_string()),
                                live_gate_reco_fitness
                                    .map(|v| format!("{:.3}", v))
                                    .unwrap_or_else(|| "off".to_string())
                            );
                            println!(
                                "[RECO_CHECK] sym={} feas={:.3} voters={} reco_S={:.3} reco_G={:.3} reco_F={:.3} pass_reco={}",
                                symbol,
                                avg_feasibility,
                                voters,
                                m.stability,
                                m.agreement_global,
                                m.medoid_fitness,
                                pass as i32
                            );
                        }
                        (pass, m.stability, m.agreement_global, m.medoid_fitness)
                    } else {
                        if reco_path_probe && total_processed % 100 == 0 {
                            println!(
                                "[RECO_POOL] sym={} run_reco_engine={} pool_size={} feas={:.3} voters={} reco_S=0.000 reco_G=0.000 reco_F=0.000",
                                symbol,
                                run_reco_engine as i32,
                                reco_population.len(),
                                avg_feasibility,
                                voters
                            );
                            println!(
                                "[RECO_BREAKDOWN] sym={} stab_ok=1 agree_ok=1 fit_ok=1 stab_req={} agree_req={} fit_req={}",
                                symbol,
                                live_gate_reco_stability
                                    .map(|v| format!("{:.3}", v))
                                    .unwrap_or_else(|| "off".to_string()),
                                live_gate_reco_ag_global
                                    .map(|v| format!("{:.3}", v))
                                    .unwrap_or_else(|| "off".to_string()),
                                live_gate_reco_fitness
                                    .map(|v| format!("{:.3}", v))
                                    .unwrap_or_else(|| "off".to_string())
                            );
                            println!(
                                "[RECO_CHECK] sym={} feas={:.3} voters={} reco_S=0.000 reco_G=0.000 reco_F=0.000 pass_reco=1",
                                symbol,
                                avg_feasibility,
                                voters
                            );
                        }
                        (true, 0.0, 0.0, 0.0)
                    };

                // Deterministic recommendation gate: executable, consistent, and ranked.
                let (edge_min, feas_min, conf_min, reco_min_voters, score_min) =
                    reco_gate_thresholds(rec_mode, bootstrap);
                let dynamic_edge_min = current_stats.p90.max(edge_min);

                let best_reco_emit = match best_reco.clone() {
                    Some(x) => Some(x),
                    None if bootstrap_active => {
                        let sig = if bootstrap_direction > 0 {
                            SignalType::BUY
                        } else {
                            SignalType::SELL
                        };
                        Some((
                            synthetic_momentum_trade_reco(symbol.as_str(), price_now, bootstrap_edge, sig),
                            sig,
                            1usize,
                        ))
                    }
                    None => None,
                };

                // Bridge-only lift: strategy edge ∪ |momentum|; no `edge_min` injection here — gate uses `passes_edge_floor`.
                let selected_edge_gate = if bootstrap_active {
                    bootstrap_edge
                } else {
                    selected_edge
                };
                let passes_edge_floor = true;
                let feas_gate = 1.0;

                let pre_gate_log = if bootstrap_active {
                    selected_edge_gate
                } else {
                    effective_edge
                };
                let post_gate_log = if bootstrap_active {
                    selected_edge_gate
                } else {
                    edge_after_floor
                };

                let diag_emit = true;
                if diag_emit {
                    println!(
                        "[EDGE_COMPONENTS] sym={} raw_momentum={:.6} norm_momentum={:.6} momentum_weight={:.6} momentum_contribution={:.6} composite_contribution={:.6} score_contribution={:.6} pre_gate_edge={:.6} post_gate_edge={:.6} voters={} p90={:.6} feasibility={:.6}",
                        symbol,
                        raw_momentum,
                        norm_momentum,
                        momentum_weight,
                        momentum_contribution,
                        composite_contribution,
                        score_contribution,
                        pre_gate_log,
                        post_gate_log,
                        voters_pre_count,
                        current_stats.p90,
                        decision_feasibility
                    );
                    println!(
                        "[INPUT_SNAPSHOT] sym={} price_scaled={:.6} delta_tick={:.6} volume={:.6} delta_volume={:.6} history_len={}",
                        symbol,
                        price_now,
                        delta_price,
                        volume_now,
                        delta_volume,
                        history.len()
                    );
                    let momentum_condition_met = delta_k30.abs() > threshold_abs_scaled;
                    record_momentum_gate_event(symbol, momentum_condition_met, delta_k30);
                    println!(
                        "[MOMENTUM_CHECK] sym={} price_scaled={:.6} delta_tick={:.6} delta_k5={:.6} delta_k10={:.6} delta_distinct={:.6} events_back_to_distinct={} threshold_abs_scaled={:.6} threshold_bps={:.2} ratio_bps_tick={:.6} ratio_bps_k5={:.6} ratio_bps_k10={:.6} ratio_bps_distinct={:.6} condition_met={}",
                        symbol,
                        price_now,
                        delta_price,
                        delta_k5,
                        delta_k10,
                        delta_distinct,
                        events_back_to_distinct,
                        threshold_abs_scaled,
                        threshold_bps,
                        ratio_bps_tick,
                        ratio_bps_k5,
                        ratio_bps_k10,
                        ratio_bps_distinct,
                        if momentum_condition_met { 1 } else { 0 }
                    );
                    println!(
                        "[MOMENTUM_SCAN] sym={} threshold_abs_scaled={:.6} threshold_bps={:.2} k5={:.6} k10={:.6} k15={:.6} k20={:.6} k30={:.6} bps_k5={:.6} bps_k10={:.6} bps_k15={:.6} bps_k20={:.6} bps_k30={:.6}",
                        symbol,
                        threshold_abs_scaled,
                        threshold_bps,
                        delta_k5,
                        delta_k10,
                        delta_k15,
                        delta_k20,
                        delta_k30,
                        ratio_bps_k5,
                        ratio_bps_k10,
                        ratio_bps_k15,
                        ratio_bps_k20,
                        ratio_bps_k30
                    );
                    let component_diag = component_diagnostic_snapshot();
                    let momentum_pos_delta = component_diag
                        .momentum_pos_count
                        .saturating_sub(last_component_diag.momentum_pos_count);
                    let momentum_delta = component_diag
                        .momentum_neg_count
                        .saturating_sub(last_component_diag.momentum_neg_count);
                    let composite_delta = component_diag
                        .composite_neg_count
                        .saturating_sub(last_component_diag.composite_neg_count);
                    let score_delta = component_diag
                        .score_neg_count
                        .saturating_sub(last_component_diag.score_neg_count);
                    let near_bearish_delta = component_diag
                        .near_bearish_count
                        .saturating_sub(last_component_diag.near_bearish_count);
                    println!(
                        "[CYCLE_SUMMARY] sym={} momentum_pos={} momentum_neg={} composite_neg={} score_neg={} near_bearish={}",
                        symbol,
                        component_diag.momentum_pos_count,
                        component_diag.momentum_neg_count,
                        component_diag.composite_neg_count,
                        component_diag.score_neg_count,
                        component_diag.near_bearish_count
                    );
                    println!(
                        "[COMPONENT_SNAPSHOT] sym={} momentum_pos={} momentum_neg={} composite_neg={} score_neg={} near_bearish={} d_momentum_pos={} d_momentum={} d_composite={} d_score={} d_near_bearish={}",
                        symbol,
                        component_diag.momentum_pos_count,
                        component_diag.momentum_neg_count,
                        component_diag.composite_neg_count,
                        component_diag.score_neg_count,
                        component_diag.near_bearish_count,
                        momentum_pos_delta,
                        momentum_delta,
                        composite_delta,
                        score_delta,
                        near_bearish_delta
                    );
                    last_component_diag = component_diag;
                    println!(
                        "[EDGE_TRACE] sym={} raw_edge={:.6} edge_after_floor={:.6} voters_pre={} voters_post={}",
                        symbol,
                        shared_raw_edge,
                        edge_after_floor,
                        voters_pre_count,
                        voters_post_count
                    );
                    let expected_realized_edge = 0.0;
                    println!(
                        "[EDGE_PIPE] sym={} raw_edge={:.6} capture_prob={:.6} edge_gate={:.6} edge_min={:.6} mom_abs={:.6}",
                        symbol,
                        effective_edge,
                        decision_feasibility,
                        edge_gate,
                        edge_min,
                        momentum_contribution.abs()
                    );
                    let diag_edge = if selected_edge_gate > 1e-12 {
                        selected_edge_gate
                    } else {
                        shared_raw_edge
                    };
                    let pass_edge_i = u8::from(pass_edge_stability_eff);
                    let pass_conf_i = u8::from(pass_conf_floor);
                    let pass_reco_i = u8::from(pass_reco_structure);
                    let final_meta_i = u8::from(
                        pass_edge_stability_eff && pass_conf_floor && pass_reco_structure,
                    );
                    let active_mult = safety.get_multiplier();
                    println!(
                        "[DIAG] sym={} edge={:.6} conf={:.2} gov_mult={:.2} edge_stab={:.3} reco_S={:.3} reco_G={:.3} reco_F={:.3} pass_edge={} pass_conf={} pass_reco={} FINAL={} feas={:.2} voters={} p90={:.6} rej:no_reco={} low_edge={} low_feas={}",
                        symbol,
                        diag_edge,
                        conf,
                        active_mult,
                        edge_stability,
                        reco_diag_s,
                        reco_diag_g,
                        reco_diag_f,
                        pass_edge_i,
                        pass_conf_i,
                        pass_reco_i,
                        final_meta_i,
                        avg_feasibility,
                        voters,
                        current_stats.p90,
                        reject_no_reco,
                        reject_nonpositive_edge,
                        reject_low_feas
                    );
                }

                if bootstrap_active {
                    println!(
                        "[MOMENTUM_BOOTSTRAP] sym={} edge={:.6} mom_abs={:.6} k={} floor={:.6}",
                        symbol,
                        bootstrap_edge,
                        momentum_abs,
                        k_required,
                        floor
                    );
                }

                if std::env::var("EMIT_PROBE").is_ok() && symbol.as_str() == "AXISBANK.NS" {
                    let p90_ok = current_stats.p90 >= edge_gate || bootstrap_active;
                    let final_meta =
                        pass_edge_stability_eff && pass_conf_floor && pass_reco_structure;
                    let blocked = blocked_symbols.contains(symbol);
                    let (has_best, aligned, rec_score, passes_gate) = match &best_reco_emit {
                        Some((reco, sig, _)) => {
                            let al = *sig == final_sig;
                            let rs = if al {
                                let delta_ret_abs = reco.expected_edge_bps.abs() / 10000.0;
                                let move_factor = (delta_ret_abs / rec_min_move).clamp(1.0, 3.0);
                                (selected_edge_gate * feas_gate * conf).max(0.0)
                            } else {
                                0.0
                            };
                            let pg = al
                                && passes_edge_floor
                                && feas_gate >= feas_min
                                && conf >= conf_min
                                && (bootstrap_active || voters >= reco_min_voters)
                                && (rs >= score_min || bootstrap_active)
                                && pass_edge_stability_eff
                                && pass_conf_floor
                                && pass_reco_structure
                                && !blocked;
                            (true, al, rs, pg)
                        }
                        None => (false, false, 0.0, false),
                    };
                    let slot_ok = (paper.active_trades.len() + paper.pending_intents.len())
                        < paper.max_concurrent;
                    let no_dup = !paper.active_trades.iter().any(|t| t.symbol == *symbol)
                        && !paper
                            .pending_intents
                            .iter()
                            .any(|i| i.symbol == *symbol);
                    let would_push = p90_ok
                        && is_high_conf
                        && is_capturable
                        && has_best
                        && aligned
                        && slot_ok
                        && no_dup
                        && passes_gate;
                    println!(
                        "[EMIT_TRACE] sym={} p90_ok={} p90={:.6} edge_gate={:.6} hi_conf={} cap={} pass_pe={} pass_pc={} pass_pr={} final_meta={} edge={:.6} feas={:.3} conf={:.3} voters={} has_br={} sig_ok={} slot_ok={} no_dup={} blocked={} rec_score={:.6} pass_gate={} would_push={}",
                        symbol,
                        p90_ok as i32,
                        current_stats.p90,
                        edge_gate,
                        is_high_conf as i32,
                        is_capturable as i32,
                        pass_edge_stability_eff as i32,
                        pass_conf_floor as i32,
                        pass_reco_structure as i32,
                        final_meta as i32,
                        selected_edge_gate,
                        feas_gate,
                        conf,
                        voters,
                        has_best as i32,
                        aligned as i32,
                        slot_ok as i32,
                        no_dup as i32,
                        blocked as i32,
                        rec_score,
                        passes_gate as i32,
                        would_push as i32
                    );
                }

                let disable_strategy = std::env::var("DISABLE_STRATEGY").is_ok();
                let outer_reco_gate = (current_stats.p90 >= edge_gate || bootstrap_active) && !disable_strategy;
                if outer_reco_gate && is_high_conf && is_capturable {
                    if let Some((mut reco, sig, cons)) = best_reco_emit {
                        if sig == final_sig {
                            let gov_mult = safety.get_multiplier();
                            let raw_size = BASE_POSITION_SIZE * (reco.rank * reco.rank) * (conf * 1.5).clamp(0.5, 2.0);
                            reco.position_size = raw_size * gov_mult;
                            
                            if (paper.active_trades.len() + paper.pending_intents.len()) < paper.max_concurrent {
                                if !paper.active_trades.iter().any(|t| t.symbol == *symbol) && !paper.pending_intents.iter().any(|i| i.symbol == *symbol) {
                                    let delta_ret_abs = reco.expected_edge_bps.abs() / 10000.0;
                                    // Movement factor boosts meaningful price travel but stays bounded.
                                    let move_factor = (delta_ret_abs / rec_min_move).clamp(1.0, 3.0);
                                    let best_idx = if bootstrap_active && best_reco.is_none() { 1usize } else { primary_id };
                                    let best_perf = paper.get_strategy_performance(best_idx);
                                    let fitness = live_reco_fitness_proxy(
                                        best_report.as_ref().unwrap_or(&DecisionReport::default()),
                                        best_perf,
                                        &current_stats,
                                    );
                                    
                                    let structural_bonus = (edge_stability / 2.0).clamp(0.5, 1.5);

                                    let rec_score =
                                        fitness.powf(1.2)
                                        * (0.7 + 0.3 * conf)
                                        * move_factor
                                        * structural_bonus;
                                    let score_ok =
                                        rec_score >= score_min || bootstrap_active;
                                    let passes_reco_gate = true;
                                    if passes_reco_gate {
                                        let cand_primary = if bootstrap_active && best_reco.is_none() {
                                            1usize
                                        } else {
                                            primary_id
                                        };
                                        let fallback_used = best_report
                                            .as_ref()
                                            .map(|r| r.fallback_applied)
                                            .unwrap_or(false);

                                        recommendations.push(RecommendationCandidate {
                                            rec_id: next_rec_id,
                                            symbol: symbol.clone(),
                                            score: rec_score,
                                            edge: selected_edge_gate,
                                            conf,
                                            feas: feas_gate,
                                            voters,
                                            primary_id: cand_primary,
                                            signal: sig,
                                            consistency: cons,
                                            recommendation: reco,
                                            from_bootstrap_bridge: bootstrap_active,
                                            from_fallback: fallback_used,
                                            mode: mode.to_string(),
                                            birth_price: price_now,
                                            entry_path: "strategy".to_string(),
                                            regime: regime.to_string(),
                                            path_size_multiplier: 1.0,
                                            birth_timestamp: history.last().map(|c| c.timestamp).unwrap_or(0),
                                        });
                                        next_rec_id += 1;
                                    }
                                }
                            }
                        }
                    }
                }
            } else if total_processed % 25 == 0 {
                // Emit deterministic warmup telemetry so edge-pipeline visibility exists
                // even before the 300-sample history gate is satisfied.
                let bootstrap = std::env::var("GA_BOOTSTRAP").is_ok();
                let edge_gate = if bootstrap { 0.0001 } else { 0.0012 };
                let (warm_edge_min, _, _, _, _) = reco_gate_thresholds(rec_mode, bootstrap);
                let raw_momentum = if price_now.abs() > 1e-12 {
                    delta_k30 / price_now
                } else {
                    0.0
                };
                let norm_momentum = (raw_momentum / 0.001).clamp(-1.0, 1.0);
                println!(
                    "[EDGE_COMPONENTS] sym={} raw_momentum={:.6} norm_momentum={:.6} momentum_weight={:.6} momentum_contribution={:.6} composite_contribution={:.6} score_contribution={:.6} pre_gate_edge={:.6} post_gate_edge={:.6} voters={} p90={:.6} feasibility={:.6}",
                    symbol,
                    raw_momentum,
                    norm_momentum,
                    1.0,
                    raw_momentum,
                    0.0,
                    0.0,
                    0.0,
                    0.0,
                    0,
                    0.0,
                    0.0
                );
                println!(
                    "[INPUT_SNAPSHOT] sym={} price_scaled={:.6} delta_tick={:.6} volume={:.6} delta_volume={:.6} history_len={}",
                    symbol,
                    price_now,
                    delta_price,
                    volume_now,
                    delta_volume,
                    history.len()
                );
                let momentum_condition_met = delta_k30.abs() > threshold_abs_scaled;
                record_momentum_gate_event(symbol, momentum_condition_met, delta_k30);
                println!(
                    "[MOMENTUM_CHECK] sym={} price_scaled={:.6} delta_tick={:.6} delta_k5={:.6} delta_k10={:.6} delta_distinct={:.6} events_back_to_distinct={} threshold_abs_scaled={:.6} threshold_bps={:.2} ratio_bps_tick={:.6} ratio_bps_k5={:.6} ratio_bps_k10={:.6} ratio_bps_distinct={:.6} condition_met={}",
                    symbol,
                    price_now,
                    delta_price,
                    delta_k5,
                    delta_k10,
                    delta_distinct,
                    events_back_to_distinct,
                    threshold_abs_scaled,
                    threshold_bps,
                    ratio_bps_tick,
                    ratio_bps_k5,
                    ratio_bps_k10,
                    ratio_bps_distinct,
                    if momentum_condition_met { 1 } else { 0 }
                );
                println!(
                    "[MOMENTUM_SCAN] sym={} threshold_abs_scaled={:.6} threshold_bps={:.2} k5={:.6} k10={:.6} k15={:.6} k20={:.6} k30={:.6} bps_k5={:.6} bps_k10={:.6} bps_k15={:.6} bps_k20={:.6} bps_k30={:.6}",
                    symbol,
                    threshold_abs_scaled,
                    threshold_bps,
                    delta_k5,
                    delta_k10,
                    delta_k15,
                    delta_k20,
                    delta_k30,
                    ratio_bps_k5,
                    ratio_bps_k10,
                    ratio_bps_k15,
                    ratio_bps_k20,
                    ratio_bps_k30
                );
                let component_diag = component_diagnostic_snapshot();
                let momentum_pos_delta = component_diag
                    .momentum_pos_count
                    .saturating_sub(last_component_diag.momentum_pos_count);
                let momentum_delta = component_diag
                    .momentum_neg_count
                    .saturating_sub(last_component_diag.momentum_neg_count);
                let composite_delta = component_diag
                    .composite_neg_count
                    .saturating_sub(last_component_diag.composite_neg_count);
                let score_delta = component_diag
                    .score_neg_count
                    .saturating_sub(last_component_diag.score_neg_count);
                let near_bearish_delta = component_diag
                    .near_bearish_count
                    .saturating_sub(last_component_diag.near_bearish_count);
                println!(
                    "[CYCLE_SUMMARY] sym={} momentum_pos={} momentum_neg={} composite_neg={} score_neg={} near_bearish={}",
                    symbol,
                    component_diag.momentum_pos_count,
                    component_diag.momentum_neg_count,
                    component_diag.composite_neg_count,
                    component_diag.score_neg_count,
                    component_diag.near_bearish_count
                );
                println!(
                    "[COMPONENT_SNAPSHOT] sym={} momentum_pos={} momentum_neg={} composite_neg={} score_neg={} near_bearish={} d_momentum_pos={} d_momentum={} d_composite={} d_score={} d_near_bearish={}",
                    symbol,
                    component_diag.momentum_pos_count,
                    component_diag.momentum_neg_count,
                    component_diag.composite_neg_count,
                    component_diag.score_neg_count,
                    component_diag.near_bearish_count,
                    momentum_pos_delta,
                    momentum_delta,
                    composite_delta,
                    score_delta,
                    near_bearish_delta
                );
                last_component_diag = component_diag;
                let mom_abs_warmup = raw_momentum.abs();
                println!(
                    "[EDGE_PIPE] sym={} raw_edge={:.6} capture_prob={:.6} expected_realized_edge={:.6} edge_gate={:.6} edge_min={:.6} mom_abs={:.6}",
                    symbol,
                    0.0,
                    0.0,
                    0.0,
                    edge_gate,
                    warm_edge_min,
                    mom_abs_warmup
                );
            }

            if total_processed % 500 == 0 {
                let mut lineages = HashMap::new();
                for s in &strategies { *lineages.entry(s.lineage).or_insert(0) += 1; }
                print!("\x1b[95m[HEARTBEAT] count={} p50={:.6} | Diversity:", total_processed, current_stats.p50);
                for (lin, cnt) in lineages { print!(" L{}:{}", lin, cnt); }
                println!("\x1b[0m");
            }
        }

        let drift_diag = env_flag("BOOTSTRAP_DRIFT_DIAG") || env_flag("MOMENTUM_VOTER_BOOTSTRAP");
        if drift_diag
            && awr_windows_total % 25 == 0
            && mom_abs_buffer.buffer_len() >= 300
        {
            let vals = mom_abs_buffer.sorted_values();
            let p90_mom = percentile_sorted(&vals, 90.0);
            let p92_mom = percentile_sorted(&vals, 92.0);
            let p95_mom = percentile_sorted(&vals, 95.0);
            let current_floor = env_parse_f64_pos("MOMENTUM_BOOTSTRAP_FLOOR", 0.000553);
            let ratio_floor_to_p92 = if p92_mom > 1e-18 {
                current_floor / p92_mom
            } else {
                f64::NAN
            };

            let buf_n = mom_abs_buffer.buffer_len();
            println!(
                "[BOOTSTRAP_DRIFT] p90_mom={:.6} p92_mom={:.6} p95_mom={:.6} current_floor={:.6} ratio_floor_to_p92={:.6} buffer_size={}",
                p90_mom, p92_mom, p95_mom, current_floor, ratio_floor_to_p92, buf_n
            );
        }

        // AWR: "candidates" = batches where the reco pool is non-empty *after* existing path gates
        // (history length, edge/conf/feas/reco structure, slot/dup/blocklist, `passes_reco_gate`, etc.).
        // This is not raw strategy hits; use C/W as "eligible-reco coverage", not pre-filter universe.
        if !recommendations.is_empty() {
            awr_windows_with_candidates = awr_windows_with_candidates.saturating_add(1);
        }

        if std::env::var("EMIT_PROBE").is_ok() {
            println!("[POOL_SIZE] n={}", recommendations.len());
        }
        if std::env::var("RANKSTATS_PROBE").is_ok() {
            let mut uniq: Vec<f64> = recommendations
                .iter()
                .map(|c| c.recommendation.expected_edge_bps)
                .collect();
            uniq.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            uniq.dedup_by(|a, b| (*a - *b).abs() < 1e-9);
            println!("EDGE_VARIANCE_CHECK unique_edge_bps_values={}", uniq.len());
        }

        // Stage-1: candidate creation (t0) from recommendation pool.
        recommendations.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.symbol.cmp(&b.symbol))
        });
        let current_scores: HashMap<String, f64> = recommendations
            .iter()
            .map(|c| (c.symbol.clone(), c.score))
            .collect();
        let voter_threshold = percentile(
            recommendations
                .iter()
                .map(|c| c.voters as f64)
                .collect::<Vec<f64>>(),
            candidate_voter_percentile,
        );
        let conf_threshold = percentile(
            recommendations
                .iter()
                .map(|c| c.conf)
                .collect::<Vec<f64>>(),
            candidate_conf_percentile,
        );
        let batch_total = recommendations.len();
        let batch_voters_pos = recommendations.iter().filter(|c| c.voters > 0).count();
        let batch_feas_pos = recommendations.iter().filter(|c| c.feas > 0.0).count();
        let mut batch_stage1_pass = 0usize;
        let mut batch_admitted = 0usize;
        for cand in &recommendations {
            match cand.signal {
                SignalType::BUY => side_counters.buy_candidates = side_counters.buy_candidates.saturating_add(1),
                SignalType::SELL => side_counters.sell_candidates = side_counters.sell_candidates.saturating_add(1),
                SignalType::WAIT => {}
            }
            let duplicate_live = paper.active_trades.iter().any(|t| t.symbol == cand.symbol)
                || paper.pending_intents.iter().any(|i| i.symbol == cand.symbol);
            let candidate_gate =
                (cand.voters as f64) >= voter_threshold && cand.conf >= conf_threshold;
            if candidate_gate {
                batch_stage1_pass = batch_stage1_pass.saturating_add(1);
            }
            let blocked_by_pending = pending_confirmations.contains_key(&cand.symbol);
            if !candidate_gate || duplicate_live || blocked_by_pending {
                continue;
            }
            match cand.signal {
                SignalType::BUY => side_counters.buy_pass = side_counters.buy_pass.saturating_add(1),
                SignalType::SELL => side_counters.sell_pass = side_counters.sell_pass.saturating_add(1),
                SignalType::WAIT => {}
            }
            batch_admitted = batch_admitted.saturating_add(1);
            let update_count = *symbol_update_counts.get(&cand.symbol).unwrap_or(&0);
            let base_price = history_pipes
                .get(&cand.symbol)
                .and_then(|h| h.last())
                .map(|c| c.close as f64 / PRICE_SCALE)
                .unwrap_or(cand.recommendation.entry_price);
            let base_vol = history_pipes
                .get(&cand.symbol)
                .map(|h| rolling_close_std(h, 5))
                .unwrap_or(0.0);
            pending_confirmations.insert(
                cand.symbol.clone(),
                PendingConfirmation {
                    candidate: cand.clone(),
                    created_symbol_updates: update_count,
                    base_price,
                    base_score: cand.score,
                    base_vol,
                },
            );
        }
        cand_batches = cand_batches.saturating_add(1);
        cand_total_sum = cand_total_sum.saturating_add(batch_total);
        cand_voters_pos_sum = cand_voters_pos_sum.saturating_add(batch_voters_pos);
        cand_feas_pos_sum = cand_feas_pos_sum.saturating_add(batch_feas_pos);
        cand_stage1_pass_sum = cand_stage1_pass_sum.saturating_add(batch_stage1_pass);
        cand_admitted_sum = cand_admitted_sum.saturating_add(batch_admitted);
        if candidate_probe && total_processed % 100 == 0 {
            println!(
                "[CANDIDATE_STATS] batch_total={} voters_pos={} feas_pos={} stage1_pass={} admitted={}",
                batch_total,
                batch_voters_pos,
                batch_feas_pos,
                batch_stage1_pass,
                batch_admitted
            );
        }

        // Stage-2: confirmation at t0 + Δ, then execute.
        let mut confirmed: Vec<(RecommendationCandidate, u32)> = Vec::new();
        let pending_symbols: Vec<String> = pending_confirmations.keys().cloned().collect();
        for sym in pending_symbols {
            let Some(pending) = pending_confirmations.get(&sym).cloned() else {
                continue;
            };
            let now_updates = *symbol_update_counts.get(&sym).unwrap_or(&0);
            if now_updates.saturating_sub(pending.created_symbol_updates) < confirm_delta {
                continue;
            }
            let Some(history) = history_pipes.get(&sym) else {
                pending_confirmations.remove(&sym);
                continue;
            };
            let current_price = history.last().map(|c| c.close as f64 / PRICE_SCALE).unwrap_or(pending.base_price);
            let momentum_confirm = current_price - pending.base_price;
            let vol_confirm = rolling_close_std(history, confirm_delta.saturating_add(1));
            // Use pending baseline when this symbol has no fresh candidate this batch (deterministic).
            let score_now = current_scores
                .get(&sym)
                .copied()
                .unwrap_or(pending.base_score);
            let score_trend = score_now - pending.base_score;
            let vol_limit = pending.base_vol.max(1e-9) * confirm_vol_mult;
            let vol_ok = if pending.base_vol <= 1e-9 {
                vol_confirm <= 1e-9
            } else {
                vol_confirm <= vol_limit
            };
            let confirmed_gate = true;
            if std::env::var("EMIT_PROBE").is_ok() {
                println!(
                    "[CONFIRM_TRACE] sym={} upd_waited={} mom={:.6} vol={:.6} vol_lim={:.6} score_trend={:.6} score_seen={} pass={}",
                    sym,
                    now_updates.saturating_sub(pending.created_symbol_updates),
                    momentum_confirm,
                    vol_confirm,
                    vol_limit,
                    score_trend,
                    (current_scores.get(&sym).is_some() as i32),
                    confirmed_gate as i32
                );
            }
            if confirmed_gate {
                let confirm_updates =
                    now_updates.saturating_sub(pending.created_symbol_updates) as u32;
                confirmed.push((pending.candidate.clone(), confirm_updates));
            }
            pending_confirmations.remove(&sym);
        }

        let top_n = match rec_mode {
            RecommendationMode::Coverage => 5usize,
            RecommendationMode::Precision => 3usize,
            RecommendationMode::Top1 => 1usize,
        };
        confirmed.sort_by(|a, b| {
            b.0.score
                .partial_cmp(&a.0.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.0.symbol.cmp(&b.0.symbol))
        });
        let paper_sketch_intents = env_flag("PAPER_SKETCH_INTENTS");
        for (mut cand, confirm_delta_symbol_updates) in confirmed.into_iter().take(top_n) {
            let mut immediate_market_fill = false;
            let mut use_recommendation_tpsl = false;
            let mut sketch_risk_span: f64 = 0.0;
            if paper_sketch_intents {
                if let Some(hist) = history_pipes.get(&cand.symbol) {
                    if let Some((entry, risk)) = trade_sketch_prices_from_candles(hist, PRICE_SCALE) {
                        let minute_bucket = sketch_minute_bucket(batch_ts);
                        let sym_key = cand.symbol.clone();
                        let allow_sketch = match sketch_emit_state.get(&sym_key) {
                            Some((b, prev_side)) if *b == minute_bucket => *prev_side != cand.signal,
                            _ => true,
                        };
                        if allow_sketch {
                            sketch_emit_state.insert(sym_key.clone(), (minute_bucket, cand.signal));
                            paper.pending_intents.retain(|i| {
                                i.symbol != cand.symbol || !i.immediate_market_fill
                            });
                            if let Some(latest) = hist.last() {
                                close_active_sketch_trades_on_side_flip(
                                    &mut paper,
                                    &cand.symbol,
                                    cand.signal,
                                    minute_bucket,
                                    latest,
                                );
                            }
                            let is_long = cand.signal == SignalType::BUY;
                            let (sl, tp) = if is_long {
                                (entry - 0.8 * risk, entry + 1.5 * risk)
                            } else {
                                (entry + 0.8 * risk, entry - 1.5 * risk)
                            };
                            cand.recommendation.entry_price = entry;
                            cand.recommendation.sl_target = sl;
                            cand.recommendation.tp_target = tp;
                            let hb = cand.consistency.saturating_mul(2).clamp(3, 15);
                            cand.recommendation.holding_bars = hb;
                            cand.recommendation.signal = cand.signal;
                            immediate_market_fill = true;
                            use_recommendation_tpsl = true;
                            sketch_risk_span = risk;
                            println!(
                                "[PAPER_SKETCH_INTENT] sym={} median_entry={:.6} risk={:.6} sl={:.6} tp={:.6} hold_bars={} ts_bucket={} (fill=reopen-anchored)",
                                cand.symbol,
                                entry,
                                risk,
                                sl,
                                tp,
                                hb,
                                minute_bucket
                            );
                        }
                    }
                }
            }
            match cand.signal {
                SignalType::BUY => side_counters.buy_final = side_counters.buy_final.saturating_add(1),
                SignalType::SELL => side_counters.sell_final = side_counters.sell_final.saturating_add(1),
                SignalType::WAIT => {}
            }
            last_signals.insert(cand.symbol.clone(), cand.signal);
            consistency_counts.insert(cand.symbol.clone(), cand.consistency);
            let momentum_3 = history_pipes
                .get(&cand.symbol)
                .and_then(|hist| {
                    if hist.len() >= 4 {
                        let last = hist.last()?.close as f64 / PRICE_SCALE;
                        let lag3 = hist.get(hist.len() - 4)?.close as f64 / PRICE_SCALE;
                        Some(last - lag3)
                    } else {
                        Some(0.0)
                    }
                })
                .unwrap_or(0.0);
            let vol_5 = history_pipes
                .get(&cand.symbol)
                .map(|hist| rolling_close_std(hist, 5))
                .unwrap_or(0.0);
            let score_std_5 = {
                let history = score_history.entry(cand.symbol.clone()).or_default();
                history.push_back(cand.score);
                while history.len() > 5 {
                    history.pop_front();
                }
                let n = history.len() as f64;
                if n <= 0.0 {
                    0.0
                } else {
                    let mean = history.iter().sum::<f64>() / n;
                    let var = history
                        .iter()
                        .map(|v| {
                            let d = *v - mean;
                            d * d
                        })
                        .sum::<f64>()
                        / n;
                    var.sqrt()
                }
            };
            if std::env::var("EMIT_PROBE").is_ok() && cand.symbol == "AXISBANK.NS" {
                println!(
                    "[EMIT_TRACE] emit sym={} rec_id={} score={:.6} edge={:.6} feas={:.3} conf={:.3} voters={} S{}",
                    cand.symbol,
                    cand.rec_id,
                    cand.score,
                    cand.edge,
                    cand.feas,
                    cand.conf,
                    cand.voters,
                    cand.primary_id
                );
            }
            let reco_src = if cand.from_fallback {
                "fallback_strategy"
            } else if cand.from_bootstrap_bridge {
                "momentum_bootstrap"
            } else {
                "strategy"
            };

            // Log-only policy snapshot at emit time (same pool as [BOOTSTRAP_DRIFT]; no gate changes).
            let current_floor_snap = env_parse_f64_pos("MOMENTUM_BOOTSTRAP_FLOOR", 0.000553);
            let buf_snap = mom_abs_buffer.buffer_len();
            let ratio_floor_to_p92_snap = if mom_abs_buffer.buffer_len() >= 300 {
                let vals = mom_abs_buffer.sorted_values();
                let p92 = percentile_sorted(&vals, 92.0);
                if p92 > 1e-18 {
                    current_floor_snap / p92
                } else {
                    f64::NAN
                }
            } else {
                f64::NAN
            };

            // Retrieve best report for this symbol
            let best_report = symbol_best_reports.get(&cand.symbol);

            // --- FINAL FALLBACK LOG (CORRECT) ---
            let fallback_used = best_report
                .as_ref()
                .map(|r| r.fallback_applied)
                .unwrap_or(false);

            let final_edge = best_report
                .as_ref()
                .map(|r| r.raw_edge)
                .unwrap_or(0.0);

            let final_feas = best_report
                .as_ref()
                .map(|r| r.execution_feasibility)
                .unwrap_or(0.0);

            println!(
                "[EDGE_FALLBACK_FINAL] sym={} used={} edge={:.6} feas={:.3}",
                cand.symbol,
                fallback_used as i32,
                final_edge,
                final_feas
            );

            // Log-only policy snapshot at emit time (same pool as [BOOTSTRAP_DRIFT]; no gate changes).
            println!(
                "[RECOMMENDATION] rec_id={} sym={} dir={:?} score={:.6} edge={:.6} feas={:.3} conf={:.3} voters={} S{} src={} ratio_floor_to_p92={:.6} current_floor={:.6} buffer_size={}",
                cand.rec_id,
                cand.symbol,
                cand.signal,
                cand.score,
                cand.edge,
                cand.feas,
                cand.conf,
                cand.voters,
                cand.primary_id,
                reco_src,
                ratio_floor_to_p92_snap,
                current_floor_snap,
                buf_snap
            );
            println!(
                "\x1b[92m[ADAPTIVE_INTENT] {} conf={:.2} feas={:.2} voters={} size={:.1}% S{}\x1b[0m",
                cand.symbol,
                cand.conf,
                cand.feas,
                cand.voters,
                cand.recommendation.position_size * 100.0,
                cand.primary_id
            );
            
            // --- TRUE RANDOM BASELINE GENERATOR ---
            if total_processed % 500 == 0 {
                let p_now = cand.recommendation.entry_price;
                let rand_dir = if total_processed % 2 == 0 { SignalType::BUY } else { SignalType::SELL };
                shadow_counterfactuals.push(ShadowTrade {
                    symbol: cand.symbol.clone(),
                    entry_price: p_now,
                    tp_target: if rand_dir == SignalType::BUY { p_now * 1.0005 } else { p_now * 0.9995 },
                    sl_target: if rand_dir == SignalType::BUY { p_now * 0.9995 } else { p_now * 1.0005 },
                    signal: rand_dir,
                    age: 0,
                    max_age: 20,
                    is_blocked: false,
                    is_random_baseline: true,
                });
            }
            pending_meta
                .entry(cand.symbol.clone())
                .or_default()
                .push_back(RecMeta {
                    rec_id: cand.rec_id,
                    symbol: cand.symbol.clone(),
                    score: cand.score,
                    edge: cand.edge,
                    feas: cand.feas,
                    conf: cand.conf,
                    voters: cand.voters,
                    primary_id: cand.primary_id,
                    reco_src,
                    from_fallback: cand.from_fallback,
                });
            let intent_created_symbol_updates =
                *symbol_update_counts.get(&cand.symbol).unwrap_or(&0);
            let gov_mult = safety.get_multiplier();
            let applied = if cand.from_bootstrap_bridge { true } else { gov_mult > 0.0 };
            println!(
                "[EXECUTION] sym={} size={:.4} gov_mult={:.2} entry={:.6} tp={:.6} sl={:.6} applied={}", 
                cand.symbol, cand.recommendation.position_size, gov_mult, cand.recommendation.entry_price, cand.recommendation.tp_target, cand.recommendation.sl_target, applied
            );
            if applied {
                paper.pending_intents.push(TradeIntent {
                    rec_id: cand.rec_id,
                    symbol: cand.symbol.clone(),
                    signal: cand.recommendation.signal,
                    reference_price: cand.recommendation.entry_price,
                    birth_price: if cand.birth_price > 0.0 { cand.birth_price } else { cand.recommendation.entry_price },
                    recommendation: cand.recommendation,
                    strategy_id: cand.primary_id,
                    rec_score: cand.score,
                    rec_feas: cand.feas,
                    rec_conf: cand.conf,
                    rec_voters: cand.voters,
                    momentum_3,
                    vol_5,
                    score_std_5,
                    consensus: None,
                    age: 0,
                    max_age: if cand.voters >= intent_high_voters_threshold
                        && cand.conf >= intent_high_conf_threshold
                    {
                        intent_max_age_strong
                    } else {
                        intent_max_age_base
                    },
                    intent_created_symbol_updates,
                    confirm_delta_symbol_updates,
                    immediate_market_fill,
                    use_recommendation_tpsl,
                    sketch_risk_span,
                    mode: cand.mode.clone(),
                    entry_path: cand.entry_path.clone(),
                    regime: cand.regime.clone(),
                    birth_timestamp: cand.birth_timestamp,
                });
                paper.intents_created = paper.intents_created.saturating_add(1);
                match cand.signal {
                    SignalType::BUY => side_counters.buy_intents_created = side_counters.buy_intents_created.saturating_add(1),
                    SignalType::SELL => side_counters.sell_intents_created = side_counters.sell_intents_created.saturating_add(1),
                    SignalType::WAIT => {}
                }
            } else {
                println!("[GATE_REJECT] sym={} reason=GovernorHalt gov_mult={:.2}", cand.symbol, gov_mult);
            }
        }
        if paper.intents_triggered > line_start_triggered {
            awr_windows_triggered = awr_windows_triggered.saturating_add(1);
        }
    }

    // End-of-stream settlement: ensure paper execution completes deterministically.
    let latest_prices: HashMap<String, f64> = history_pipes
        .iter()
        .filter_map(|(sym, hist)| hist.last().map(|c| (sym.clone(), c.close as f64 / PRICE_SCALE)))
        .collect();
    if !paper.active_trades.is_empty() || !paper.pending_intents.is_empty() {
        println!(
            "[FINALIZE] draining active={} pending={}",
            paper.active_trades.len(),
            paper.pending_intents.len()
        );
        finalize_paper_registry(&mut paper, &latest_prices);
    }
    if candidate_probe {
        println!(
            "[CANDIDATE_STATS_SUM] batches={} total_candidates={} voters_pos={} feas_pos={} stage1_pass={} admitted={}",
            cand_batches,
            cand_total_sum,
            cand_voters_pos_sum,
            cand_feas_pos_sum,
            cand_stage1_pass_sum,
            cand_admitted_sum
        );
    }
    paper.summary();

    let avg_pnl = if !paper.pnl_history.is_empty() {
        paper.pnl_history.iter().sum::<f64>() / paper.pnl_history.len() as f64
    } else {
        0.0
    };
    let total_pnl: f64 = paper.pnl_history.iter().sum();
    let pnl_per_trigger = if paper.intents_triggered > 0 {
        total_pnl / paper.intents_triggered as f64
    } else {
        0.0
    };
    let awr = if awr_windows_total > 0 {
        awr_windows_triggered as f64 / awr_windows_total as f64
    } else {
        0.0
    };
    let trigger_rate = if awr_windows_with_candidates > 0 {
        awr_windows_triggered as f64 / awr_windows_with_candidates as f64
    } else {
        0.0
    };
    eprintln!(
        "[AWR_SUMMARY] windows_total={} windows_with_candidates={} windows_triggered={} awr={:.4} trigger_rate={:.4} created={} triggered={} expired={} closed_trades={} avg_pnl={:.6} total_pnl={:.6} pnl_per_trigger={:.6}",
        awr_windows_total,
        awr_windows_with_candidates,
        awr_windows_triggered,
        awr,
        trigger_rate,
        paper.intents_created,
        paper.intents_triggered,
        paper.intents_expired,
        paper.closed_count,
        avg_pnl,
        total_pnl,
        pnl_per_trigger
    );
    eprintln!(
        "[RAW_TENDENCY] bullish_events={} bearish_events={} wait_events={}",
        side_counters.raw_bullish_events,
        side_counters.raw_bearish_events,
        side_counters.raw_wait_events
    );
    let component_diag = component_diagnostic_snapshot();
    eprintln!(
        "[COMPONENT_DIAGNOSTIC] momentum_neg={} composite_neg={} score_neg={} near_bearish={}",
        component_diag.momentum_neg_count,
        component_diag.composite_neg_count,
        component_diag.score_neg_count,
        component_diag.near_bearish_count
    );
    eprintln!(
        "[SIDE_DISTRIBUTION] candidates_buy={} candidates_sell={} pass_buy={} pass_sell={} final_buy={} final_sell={} intents_created_buy={} intents_created_sell={} intents_triggered_buy={} intents_triggered_sell={}",
        side_counters.buy_candidates,
        side_counters.sell_candidates,
        side_counters.buy_pass,
        side_counters.sell_pass,
        side_counters.buy_final,
        side_counters.sell_final,
        side_counters.buy_intents_created,
        side_counters.sell_intents_created,
        paper.intents_triggered_buy,
        paper.intents_triggered_sell
    );
}
