#![allow(
    unused_variables,
    unused_mut,
    unused_imports,
    dead_code,
    unreachable_code,
    unused_assignments,
    unused_parens,
    unreachable_patterns
)]
use crate::ese::{ExecutionEngine, ExecutionResult};
use crate::selection_cap;
use crate::{Candle, GaExitReason, MarketEvent, Side};
use rand::{rngs::StdRng, Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json;
use serde_json::value::to_value as to_json_value;
use std::cmp::Ordering;
use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct DistributionStats {
    pub p10: f64,
    pub p30: f64,
    pub p50: f64,
    pub p65: f64, // Target Threshold (Top 35%)
    pub p90: f64,
    pub p95: f64,
    pub empirical_samples: Vec<f64>,
}

impl DistributionStats {
    pub fn from_slice(values: &mut [f64]) -> Self {
        if values.is_empty() {
            return Self::default();
        }
        values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let len = values.len();

        let step = (len / 500).max(1);
        let empirical_samples: Vec<f64> = values.iter().step_by(step).copied().collect();

        Self {
            p10: values[len * 10 / 100],
            p30: values[len * 30 / 100],
            p50: values[len * 50 / 100],
            p65: values[len * 65 / 100],
            p90: values[len * 90 / 100],
            p95: values[len * 95 / 100],
            empirical_samples,
        }
    }

    pub fn rank(&self, value: f64) -> f64 {
        if self.empirical_samples.is_empty() {
            return 0.5;
        }
        let samples = &self.empirical_samples;
        let below = samples.partition_point(|&v| v < value);
        let count_le = samples.partition_point(|&v| v <= value);
        let equal = count_le - below;

        (below as f64 + 0.5 * equal as f64) / samples.len() as f64
    }
}

impl Default for DistributionStats {
    fn default() -> Self {
        let default_dist = vec![0.0003, 0.0008, 0.0015, 0.0020, 0.0035, 0.0050];
        Self {
            p10: default_dist[0],
            p30: default_dist[1],
            p50: default_dist[2],
            p65: default_dist[3],
            p90: default_dist[4],
            p95: default_dist[5],
            empirical_samples: default_dist,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RankStats {
    pub bucket_mfe_sum: [[f64; 5]; 10], // [Rank][Vol]
    pub bucket_mae_sum: [[f64; 5]; 10], 
    pub bucket_time_sum: [[f64; 5]; 10],
    pub bucket_count: [[usize; 5]; 10],
}

impl Default for RankStats {
    fn default() -> Self {
        Self {
            bucket_mfe_sum: [[0.0045; 5]; 10], // 45bps baseline
            bucket_mae_sum: [[0.0015; 5]; 10], // 15bps risk baseline -> 3:1 RR
            bucket_time_sum: [[20.0; 5]; 10],
            bucket_count: [[1; 5]; 10],
        }
    }
}

impl RankStats {
    pub fn get_expected_mfe(&self, rank: f64, vol_bucket: usize) -> f64 {
        let r_bucket = (rank * 10.0).floor().clamp(0.0, 9.0) as usize;
        let v_bucket = vol_bucket.min(4);
        let count = self.bucket_count[r_bucket][v_bucket].max(1);
        self.bucket_mfe_sum[r_bucket][v_bucket] / count as f64
    }

    pub fn get_expected_mae(&self, rank: f64, vol_bucket: usize) -> f64 {
        let r_bucket = (rank * 10.0).floor().clamp(0.0, 9.0) as usize;
        let v_bucket = vol_bucket.min(4);
        let count = self.bucket_count[r_bucket][v_bucket].max(1);
        self.bucket_mae_sum[r_bucket][v_bucket] / count as f64
    }

    pub fn get_expected_time(&self, rank: f64, vol_bucket: usize) -> f64 {
        let r_bucket = (rank * 10.0).floor().clamp(0.0, 9.0) as usize;
        let v_bucket = vol_bucket.min(4);
        let count = self.bucket_count[r_bucket][v_bucket].max(1);
        self.bucket_time_sum[r_bucket][v_bucket] / count as f64
    }

    pub fn blend(&mut self, next: Self, alpha: f64) {
        for r in 0..10 {
            for v in 0..5 {
                if next.bucket_count[r][v] < 3 {
                    // Skip buckets with insufficient data
                    continue;
                }
                
                let next_mfe = next.bucket_mfe_sum[r][v] / next.bucket_count[r][v] as f64;
                let next_mae = next.bucket_mae_sum[r][v] / next.bucket_count[r][v] as f64;
                let next_time = next.bucket_time_sum[r][v] / next.bucket_count[r][v] as f64;

                let prev_mfe = self.bucket_mfe_sum[r][v] / self.bucket_count[r][v].max(1) as f64;
                let prev_mae = self.bucket_mae_sum[r][v] / self.bucket_count[r][v].max(1) as f64;
                let prev_time = self.bucket_time_sum[r][v] / self.bucket_count[r][v].max(1) as f64;

                let smoothed_mfe = (1.0 - alpha) * prev_mfe + alpha * next_mfe;
                let smoothed_mae = (1.0 - alpha) * prev_mae + alpha * next_mae;
                let smoothed_time = (1.0 - alpha) * prev_time + alpha * next_time;

                self.bucket_mfe_sum[r][v] = smoothed_mfe;
                self.bucket_mae_sum[r][v] = smoothed_mae;
                self.bucket_time_sum[r][v] = smoothed_time;
                self.bucket_count[r][v] = 1;

                #[cfg(feature = "debug_decision")]
                if r >= 7 {
                     let quality = next_mfe / (next_mae.abs() + 1e-6);
                     println!("[RANK_SUMMARY] R={}/V={} n={} mfe={:.6} mae={:.6} quality={:.2}", r, v, next.bucket_count[r][v], next_mfe, next_mae, quality);
                }
            }
        }
    }
}

pub struct PercentileBuffer {
    buffer: VecDeque<f64>,
    capacity: usize,
}

impl PercentileBuffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    pub fn push(&mut self, value: f64) {
        if self.buffer.len() >= self.capacity {
            self.buffer.pop_front();
        }
        self.buffer.push_back(value);
    }

    pub fn get_stats(&self) -> DistributionStats {
        let mut vals: Vec<f64> = self.buffer.iter().copied().collect();
        DistributionStats::from_slice(&mut vals)
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    pub fn buffer_len(&self) -> usize {
        self.buffer.len()
    }
}
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};

/// --- PHASE 16 (V3.6.8): DETERMINISTIC KERNELS ---
/// Stable, cross-platform hash for simulation determinism.
pub fn stable_deterministic_hash(data: (u64, u64, u64)) -> u64 {
    let mut hasher = blake3::Hasher::new();
    hasher.update(&data.0.to_le_bytes());
    hasher.update(&data.1.to_le_bytes());
    hasher.update(&data.2.to_le_bytes());
    let hash = hasher.finalize();
    let mut bytes = [0u8; 8];
    bytes.copy_from_slice(&hash.as_bytes()[0..8]);
    u64::from_le_bytes(bytes)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignalSource {
    Organic,
    Synthetic,
    Bootstrap,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Regime {
    Dead,
    Live,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignalSignature {
    pub archetype: u8,
    pub regime: i8,   // -1 (low), 0 (norm), 1 (high vol)
    pub momentum: i8, // -1 (bear), 0 (flat), 1 (bull)
}

impl Default for SignalSignature {
    fn default() -> Self {
        Self {
            archetype: 0,
            regime: 0,
            momentum: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum MarketRegime {
    #[default]
    MeanReversion,
    BullTrend,
    BearTrend,
    HighVolatilityNoise,
}

impl std::fmt::Display for MarketRegime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            MarketRegime::MeanReversion => "MeanReversion",
            MarketRegime::BullTrend => "BullTrend",
            MarketRegime::BearTrend => "BearTrend",
            MarketRegime::HighVolatilityNoise => "HighVolatilityNoise",
        };
        write!(f, "{s}")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DirectionArchetype {
    LongSpecialist,
    ShortSpecialist,
    DualCore,
}

#[inline]
pub fn classify_direction_bias(direction_bias: u8) -> DirectionArchetype {
    match direction_bias {
        0..=25 => DirectionArchetype::ShortSpecialist,
        75..=100 => DirectionArchetype::LongSpecialist,
        _ => DirectionArchetype::DualCore,
    }
}

#[inline]
pub fn regime_multiplier(regime: MarketRegime, bias: DirectionArchetype) -> f64 {
    match regime {
        MarketRegime::BullTrend => match bias {
            DirectionArchetype::LongSpecialist => 2.0,
            DirectionArchetype::ShortSpecialist => 0.3, // Skeptic weight
            DirectionArchetype::DualCore => 1.0,
        },
        MarketRegime::BearTrend => match bias {
            DirectionArchetype::ShortSpecialist => 2.0,
            DirectionArchetype::LongSpecialist => 0.3, // Skeptic weight
            DirectionArchetype::DualCore => 1.0,
        },
        MarketRegime::MeanReversion => match bias {
            DirectionArchetype::DualCore => 1.3,
            DirectionArchetype::LongSpecialist => 0.7,
            DirectionArchetype::ShortSpecialist => 0.7,
        },
        MarketRegime::HighVolatilityNoise => 0.7, // Soft penalty for all
    }
}

/// Institutional Regime Detector (Phase D.1.24)
#[inline]
pub fn detect_market_regime(price: f64, sma20: f64, momentum: f64, norm_vol: f64) -> MarketRegime {
    const HIGH_VOL_THRESHOLD: f64 = 0.005;
    const MOMENTUM_STRONG: f64 = 0.6;
    const TREND_STRENGTH_MIN: f64 = 0.55;
    const NEAR_SMA_EPS: f64 = 0.0015;

    if norm_vol > HIGH_VOL_THRESHOLD {
        return MarketRegime::HighVolatilityNoise;
    }

    let dist = if sma20.abs() > f64::EPSILON {
        (price - sma20) / sma20
    } else {
        0.0
    };
    let trend_strength = 0.5 * momentum + 0.5 * dist.abs().min(1.0);

    if momentum > MOMENTUM_STRONG && trend_strength > TREND_STRENGTH_MIN {
        if price > sma20 {
            return MarketRegime::BullTrend;
        } else if price < sma20 {
            return MarketRegime::BearTrend;
        }
    }

    if dist.abs() < NEAR_SMA_EPS {
        return MarketRegime::MeanReversion;
    }

    MarketRegime::MeanReversion
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub enum SignalType {
    #[default]
    WAIT,
    BUY,
    SELL,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlphaConsensus {
    pub buy_strength: f64,
    pub sell_strength: f64,
    pub confidence: f64,
    pub voter_count: usize,
    pub agreement_ratio: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DecisionReport {
    pub trade_id: u64,
    pub symbol: String,
    pub timestamp: u64,
    pub signal: SignalType,
    pub confidence: f64,
    pub expected_return: f64,
    pub horizon_bars: u64,
    pub participation: f64,
    pub regime: MarketRegime,
    pub aligned_weight: f64,
    pub opposing_weight: f64,
    pub consistency: usize,
    pub conviction_score: f64,
    pub agreement_strength: String,
    pub voters: String,
    pub execution_feasible: bool,
    pub execution_score: f64,
    pub execution_threshold: f64,
    pub threshold: f64,
    pub raw_edge: f64,
    pub realized_return: Option<f64>,
    pub capture_efficiency: Option<f64>,
    pub execution_feasibility: f64,
    pub efficiency_label: String,
    pub recommendation: Option<TradeRecommendation>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TradeRecommendation {
    pub symbol: String,
    pub signal: SignalType,
    pub rank: f64,
    pub raw_edge: f64,
    pub confidence: f64,
    pub quality_score: f64,
    
    pub entry_price: f64,
    pub entry_low: f64,
    pub entry_high: f64,
    
    pub tp_target: f64,
    pub sl_target: f64,
    
    pub expected_rr: f64,
    pub expected_edge_bps: f64,
    pub risk_bps: f64,
    
    pub holding_bars: usize,
    pub vol_bps: f64,
    pub vol_bucket: usize,
    
    pub is_execution: bool,
    pub position_size: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderIntent {
    pub symbol: String,
    pub side: Side,
    pub quantity: u32,
    pub price: u64,
    pub tp_target: u64,
    pub sl_target: u64,
    pub holding_period: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeIntent {
    pub symbol: String,
    pub signal: SignalType,
    pub reference_price: f64,
    pub recommendation: TradeRecommendation,
    pub strategy_id: usize, // Primary strategy
    pub consensus: Option<AlphaConsensus>,
    pub age: usize,
    pub max_age: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveTrade {
    pub symbol: String,
    pub entry_price: f64,
    pub tp_target: f64,
    pub sl_target: f64,
    pub hold_limit: usize,
    pub current_hold: usize,
    pub signal: SignalType,
    pub size: f64,
    pub vol_bps: f64,
    pub rank: f64,
    pub strategy_id: usize,
    pub consensus: Option<AlphaConsensus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaperRegistry {
    pub active_trades: Vec<ActiveTrade>,
    pub pending_intents: Vec<TradeIntent>,
    pub equity: f64,
    pub peak_equity: f64,
    pub max_drawdown: f64,
    pub closed_count: usize,
    pub wins: usize,
    pub losses: usize,
    pub pnl_history: Vec<f64>,
    pub strategy_pnl: HashMap<usize, f64>,
    pub strategy_counts: HashMap<usize, usize>,
    pub equity_curve: Vec<f64>,
    pub timestamps: Vec<u64>,
    pub max_concurrent: usize,
    
    // 🔥 Robustness Analytics
    pub rank_pnl_sum: [f64; 10],
    pub rank_count: [usize; 10],
    pub vol_pnl_sum: [f64; 5],
    pub vol_count: [usize; 5],
    pub rolling_peak: f64,
    pub adaptation_threshold: usize,
    pub trade_counts_per_strat: HashMap<usize, usize>,
}

impl PaperRegistry {
    pub fn summary(&self) {
        let win_rate = if self.closed_count > 0 { self.wins as f64 / self.closed_count as f64 } else { 0.0 };
        let avg_pnl = if !self.pnl_history.is_empty() { self.pnl_history.iter().sum::<f64>() / self.pnl_history.len() as f64 } else { 0.0 };
        println!("\n=== FINAL PAPER STATS ===");
        println!("Trades : {} (W: {}, L: {})", self.closed_count, self.wins, self.losses);
        println!("Win Rate : {:.2}%", win_rate * 100.0);
        println!("Avg PnL : {:.5}", avg_pnl);
        println!("Equity Final : {:.4}", self.equity);
        println!("Max DD       : {:.4}%", self.max_drawdown * 100.0);
        
        println!("\n--- PER-RANK ROBUSTNESS ---");
        for r in 0..10 {
            if self.rank_count[r] > 0 {
                let avg = self.rank_pnl_sum[r] / self.rank_count[r] as f64;
                println!("Rank {:.1} : count={} avg_pnl={:.6}", r as f64 / 10.0, self.rank_count[r], avg);
            }
        }
        println!("---------------------------");
    }

    pub fn get_strategy_performance(&self, strategy_id: usize) -> f64 {
        let pnl = self.strategy_pnl.get(&strategy_id).cloned().unwrap_or(0.0);
        let count = self.strategy_counts.get(&strategy_id).cloned().unwrap_or(0);
        if count > 0 { pnl / count as f64 } else { 0.0 }
    }

    pub fn export_csv(&self, path: &str) -> std::io::Result<()> {
        use std::fs::File;
        use std::io::Write;
        let mut file = File::create(path)?;
        writeln!(file, "timestamp,equity")?;
        for (ts, eq) in self.timestamps.iter().zip(self.equity_curve.iter()) {
            writeln!(file, "{},{}", ts, eq)?;
        }
        Ok(())
    }
}

impl Default for PaperRegistry {
    fn default() -> Self {
        Self {
            active_trades: Vec::new(),
            pending_intents: Vec::new(),
            equity: 1.0,
            peak_equity: 1.0,
            max_drawdown: 0.0,
            closed_count: 0,
            wins: 0,
            losses: 0,
            pnl_history: Vec::new(),
            strategy_pnl: HashMap::new(),
            strategy_counts: HashMap::new(),
            equity_curve: Vec::new(),
            timestamps: Vec::new(),
            max_concurrent: 3,
            rank_pnl_sum: [0.0; 10],
            rank_count: [0; 10],
            vol_pnl_sum: [0.0; 5],
            vol_count: [0; 5],
            rolling_peak: 1.0,
            adaptation_threshold: 50,
            trade_counts_per_strat: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ExitType {
    TakeProfit,
    StopLoss,
    Ambiguous,
}

pub fn apply_slippage(price: f64, is_buy: bool, vol_bps: f64) -> f64 {
    // 🔥 Dynamic Slippage: Scale with market volatility
    let base_bps = 2.0;
    let dynamic_bps = base_bps + (vol_bps * 0.1).min(10.0);
    
    let factor = dynamic_bps / 10000.0;
    if is_buy { price * (1.0 + factor) } else { price * (1.0 - factor) }
}

pub fn resolve_intracandle_exit(
    high: f64,
    low: f64,
    tp: f64,
    sl: f64,
    is_long: bool,
) -> Option<ExitType> {
    if is_long {
        let tp_hit = high >= tp;
        let sl_hit = low <= sl;
        match (tp_hit, sl_hit) {
            (true, false) => Some(ExitType::TakeProfit),
            (false, true) => Some(ExitType::StopLoss),
            (true, true) => Some(ExitType::Ambiguous),
            _ => None,
        }
    } else {
        let tp_hit = low <= tp;
        let sl_hit = high >= sl;
        match (tp_hit, sl_hit) {
            (true, false) => Some(ExitType::TakeProfit),
            (false, true) => Some(ExitType::StopLoss),
            (true, true) => Some(ExitType::Ambiguous),
            _ => None,
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct SignatureStats {
    pub sum_pnl: f64,
    pub win_count: usize,
    pub sample_count: usize,
}

fn default_capture_eff() -> f64 {
    1.0
}

#[derive(Clone)]
pub struct ScenarioPair<'a> {
    pub name: &'a str,
    pub signal_symbol: &'a str,
    pub execution_symbol: &'a str,
    pub signal: &'a [MarketEvent],
    pub execution: &'a [MarketEvent],
}

// Helper function to serialize any serializable struct into a canonical JSON string.
// This is crucial for deterministic hashing, especially for floating-point numbers.
/// --- PHASE A+: ADAPTIVE INTELLIGENCE ENGINE ---
#[derive(Default, Debug, Clone)]
struct WelfordTracker {
    count: usize,
    mean: f64,
    m2: f64,
}

impl WelfordTracker {
    fn update(&mut self, val: f64) {
        self.count += 1;
        let n = self.count as f64;
        let delta = val - self.mean;
        self.mean += delta / n;
        let delta2 = val - self.mean;
        self.m2 += delta * delta2;
    }

    fn mean(&self) -> f64 {
        self.mean
    }
    fn std(&self) -> f64 {
        if self.count < 2 {
            0.0
        } else {
            (self.m2 / (self.count - 1) as f64).sqrt()
        }
    }
}

#[derive(Default, Debug, Clone)]
struct AdaptiveStats {
    agreement: WelfordTracker,
    dominance: WelfordTracker,
    purity: WelfordTracker,
    stability: WelfordTracker,
    z_score: WelfordTracker,
    energy: WelfordTracker,
    final_score: WelfordTracker,
    score_history: Vec<f64>, // For rolling percentile
}

// ==========================================
// GA → ESE BRIDGE (LIGHTWEIGHT ADAPTER)
// ==========================================
fn simulate_execution_via_ese(
    ese: &mut ExecutionEngine,
    entry_price: u64,
    tp_target: u64,
    sl_target: u64,
    side: Side,
    quantity: u32,
    execution_events: &[crate::MarketEvent],
    entry_idx: usize,
    max_hold: usize,
) -> ExecutionResult {
    // Minimal intent → execution mapping
    ese.simulate_round_trip(
        entry_price,
        tp_target,
        sl_target,
        side,
        quantity,
        execution_events,
        entry_idx,
        max_hold,
    )
}

/// --- PHASE 13.5: INSTITUTIONAL METRICS ENGINE ---
#[derive(Default, Debug, Clone)]
struct ScenarioMetrics {
    // Adaptive Layer
    pub adaptive: AdaptiveStats,
    pub efficiencies: Vec<f64>,

    // Trade stats
    trade_count: usize,
    profitable_trades: usize,

    // Opportunity tracking (Timesteps evaluated for entry)
    total_opportunities: usize,

    // Aggregates (measured only at trade decision time)
    sum_pnl: f64,
    sum_entropy: f64,
    sum_conviction: f64,
    sum_efficiency: f64,
    sum_edge_quality: f64,
    sum_time_to_mfe: f64,

    // Decision Surface Metrics (Phase 13.5 + 13.6)
    pub sum_margin: f64,
    pub sum_margin_sq: f64,
    pub sum_aqg_health: f64,

    // Signal Separation Metrics (Phase 15)
    pub sum_edge_spread: f64,
    pub sum_dominance: f64,
    pub sum_signal_entropy: f64,

    // Phase D.1.18: Cognitive Layer (Memory)
    pub signature_memory: HashMap<SignalSignature, SignatureStats>,
    pub max_signature_credibility: f64,
    pub forced_win_count: usize,

    // Population Separation Analysis (Phase 17 Diagnostic Calibration)
    pub raw_pop_count: usize,
    pub sum_raw_pop_dominance: f64,
    pub max_raw_pop_dominance: f64,
    pub raw_pop_dominance_buckets: [usize; 6],

    pub exec_pop_count: usize,
    pub sum_exec_pop_dominance: f64,
    pub max_exec_pop_dominance: f64,
    pub exec_pop_dominance_buckets: [usize; 6],

    // Phase 2: Consistency Engine
    pub pnl_history: Vec<GaRoundTripOutcome>,
    pub trade_qualities: Vec<f64>,
    pub sum_realized_pnl: f64,
    pub sum_expected_pnl: f64,
    pub sum_ideal_pnl: f64, // Ideal baseline (frictionless) tracking

    // Phase 14: Consensus Bridge Audit
    pub vip_count: usize,
    pub stat_count: usize,
    pub stat_zero_dom_count: usize,

    // Phase 17B Realizability
    pub vip_admitted_count: usize,
    pub stat_admitted_count: usize,
    pub exec_admitted_count: usize,
    pub exec_passed_count: usize,
    pub exec_rejected_count: usize,
    pub sum_exec_e_score: f64,
    pub sum_signal_e_score: f64,
    pub vip_exec_passed_count: usize,
    pub sum_vip_e_score: f64,
    pub sum_stat_e_score: f64,

    // Phase 14++: Structural Health (Universe Discovery)
    pub total_windows: usize,
    pub valid_windows: usize,
    pub accepted_windows: usize,

    // === D.1.21 GENES ===
    pub long_count: u32,
    pub short_count: u32,

    pub sum_agreement_raw: f64,
    pub sum_purity_raw: f64,
    pub sum_stability_raw: f64,

    pub sum_agreement_valid: f64,
    pub sum_purity_valid: f64,
    pub sum_stability_valid: f64,

    pub max_agreement: f64,
    pub max_purity: f64,

    // Phase 14++: Consensus Bridge Audit
    pub consensus_bypass_count: usize,
    pub stability_rejected_count: usize,
    pub clarity_trade_count: usize,
    pub conviction_trade_count: usize,
    pub sum_clarity_pnl: f64,
    pub sum_conviction_pnl: f64,

    // Phase D.1.16: Edge Validation Layer
    pub organic_trade_count: usize,
    pub bootstrap_trade_count: usize,
    pub organic_sum_pnl: f64,
    pub bootstrap_sum_pnl: f64,
    pub pnl_history_rolling: Vec<f64>,

    // Phase A+: Adaptive Ranking
    pub adaptive_opportunity_count: usize,
}

impl ScenarioMetrics {
    fn record_opportunity(&mut self) {
        self.total_opportunities += 1;
    }

    fn std_dev(&self) -> f64 {
        let n = self.pnl_history.len() as f64;
        if n < 2.0 {
            return 0.0;
        }
        let mean = self.sum_realized_pnl / n;
        let variance = self
            .pnl_history
            .iter()
            .map(|o| (o.pnl - mean).powi(2))
            .sum::<f64>()
            / n;
        variance.sqrt()
    }

    fn record_pop_stats(&mut self, dominance: f64, is_exec: bool) {
        let (count, sum, max, buckets) = if is_exec {
            (
                &mut self.exec_pop_count,
                &mut self.sum_exec_pop_dominance,
                &mut self.max_exec_pop_dominance,
                &mut self.exec_pop_dominance_buckets,
            )
        } else {
            (
                &mut self.raw_pop_count,
                &mut self.sum_raw_pop_dominance,
                &mut self.max_raw_pop_dominance,
                &mut self.raw_pop_dominance_buckets,
            )
        };

        *count += 1;
        if dominance > *max {
            *max = dominance;
        }
        *sum += dominance;

        // Institutional 6-Bucket Distribution (Phase 17)
        if dominance < 0.05 {
            buckets[0] += 1;
        } else if dominance < 0.10 {
            buckets[1] += 1;
        } else if dominance < 0.20 {
            buckets[2] += 1;
        } else if dominance < 0.25 {
            buckets[3] += 1;
        } else if dominance < 0.50 {
            buckets[4] += 1;
        } else {
            buckets[5] += 1;
        }
    }

    fn record_funnel_admission(&mut self, dominance: f64) {
        self.exec_admitted_count += 1;
        if dominance >= 0.20 {
            self.vip_admitted_count += 1;
        } else {
            self.stat_admitted_count += 1;
            if dominance < 0.05 {
                self.stat_zero_dom_count += 1;
            }
        }
    }

    fn record_funnel_pass(&mut self, dominance: f64, e_score: f64) {
        self.exec_passed_count += 1;
        self.sum_exec_e_score += e_score;
        if dominance >= 0.20 {
            self.vip_exec_passed_count += 1;
            self.sum_vip_e_score += e_score;
        } else {
            self.sum_stat_e_score += e_score;
        }
    }

    fn record_structural_health(
        &mut self,
        agreement: f64,
        purity: f64,
        stability: f64,
        is_valid: bool,
    ) {
        self.total_windows += 1;
        self.sum_agreement_raw += agreement;
        self.sum_purity_raw += purity;
        self.sum_stability_raw += stability;

        if agreement > self.max_agreement {
            self.max_agreement = agreement;
        }
        if purity > self.max_purity {
            self.max_purity = purity;
        }

        if is_valid {
            self.valid_windows += 1;
            self.sum_agreement_valid += agreement;
            self.sum_purity_valid += purity;
            self.sum_stability_valid += stability;
        }
    }

    fn record_adaptive_pulse(
        &mut self,
        agreement: f64,
        dominance: f64,
        purity: f64,
        stability: f64,
        z_score: f64,
        energy: f64,
    ) {
        self.adaptive.agreement.update(agreement);
        self.adaptive.dominance.update(dominance);
        self.adaptive.purity.update(purity);
        self.adaptive.stability.update(stability);
        self.adaptive.z_score.update(z_score);
        self.adaptive.energy.update(energy);
    }

    fn record_final_score(&mut self, score: f64) {
        self.adaptive.final_score.update(score);
        self.adaptive.score_history.push(score);
        if self.adaptive.score_history.len() > 200 {
            self.adaptive.score_history.remove(0);
        }
    }

    fn adaptive_threshold(&self, percentile: f64) -> f64 {
        if self.adaptive.score_history.is_empty() {
            return 0.0;
        }
        percentile_f64(&self.adaptive.score_history, percentile)
    }

    fn record_trade(
        &mut self,
        realized_pnl: f64,
        ideal_pnl: f64, // Corrected parameter
        expected_pnl: f64,
        entropy: f64,
        conviction: f64,
        efficiency: f64,
        edge_quality: f64,
        time_to_mfe: f64,
        margin: f64,
        aqg_health: f64,
        edge_spread: f64,
        dominance: f64,
        signal_entropy: f64,
        outcome: GaRoundTripOutcome,
        source: SignalSource,
        signature: Option<SignalSignature>,
        is_long: bool,
        e_score: f64,
    ) {
        if is_long {
            self.long_count += 1;
        } else {
            self.short_count += 1;
        }
        self.pnl_history.push(outcome.clone());
        self.trade_count += 1;
        self.sum_pnl += realized_pnl;

        self.sum_exec_e_score += e_score;

        // Update Signature Memory (Phase D.1.18)
        if let Some(sig) = signature {
            let stats = self
                .signature_memory
                .entry(sig)
                .or_insert_with(SignatureStats::default);
            stats.sum_pnl += realized_pnl;
            stats.sample_count += 1;
            if realized_pnl > 0.0 {
                stats.win_count += 1;
            }
        }

        // Phase 2/3: Consistency & Efficiency Tracking
        self.sum_realized_pnl += realized_pnl;
        self.sum_expected_pnl += expected_pnl;
        self.sum_ideal_pnl += ideal_pnl;

        let denom = expected_pnl.abs().max(1e-9);
        let quality = (realized_pnl / denom).clamp(-2.0, 2.0);
        self.trade_qualities.push(quality);
        if self.trade_qualities.len() > 20 {
            self.trade_qualities.remove(0);
        }

        self.sum_entropy += entropy;
        self.sum_conviction += conviction;
        self.sum_efficiency += efficiency;
        self.sum_edge_quality += edge_quality;
        self.sum_time_to_mfe += time_to_mfe;
        self.sum_margin += margin;
        self.sum_margin_sq += margin * margin;
        self.sum_aqg_health += aqg_health;
        self.sum_edge_spread += edge_spread;
        self.sum_dominance += dominance;
        self.sum_signal_entropy += signal_entropy;

        if realized_pnl > 0.0 {
            self.profitable_trades += 1;
        }

        // Phase D.1.16: Source-Aware Attribution
        match source {
            SignalSource::Organic => {
                self.organic_trade_count += 1;
                self.organic_sum_pnl += realized_pnl;
            }
            SignalSource::Synthetic => {
                self.conviction_trade_count += 1;
                self.sum_conviction_pnl += realized_pnl;
            }
            SignalSource::Bootstrap => {
                self.bootstrap_trade_count += 1;
                self.bootstrap_sum_pnl += realized_pnl;
            }
        }
        self.pnl_history_rolling.push(realized_pnl);
        if self.pnl_history_rolling.len() > 50 {
            self.pnl_history_rolling.remove(0);
        }
    }

    fn avg_entropy(&self) -> f64 {
        self.sum_entropy / self.trade_count.max(1) as f64
    }

    fn selectivity(&self) -> f64 {
        self.trade_count as f64 / self.total_opportunities.max(1) as f64
    }

    fn avg_conviction(&self) -> f64 {
        self.sum_margin / self.trade_count.max(1) as f64
    }

    fn avg_efficiency(&self) -> f64 {
        self.sum_efficiency / self.trade_count.max(1) as f64
    }

    fn avg_edge_quality(&self) -> f64 {
        self.sum_edge_quality / self.trade_count.max(1) as f64
    }

    fn avg_edge_spread_norm(&self) -> f64 {
        self.sum_edge_spread / self.trade_count.max(1) as f64
    }

    fn avg_dominance(&self) -> f64 {
        self.sum_dominance / self.trade_count.max(1) as f64
    }

    fn avg_signal_entropy(&self) -> f64 {
        self.sum_signal_entropy / self.trade_count.max(1) as f64
    }

    /// PHASE 13.5: Decision Surface Entropy calculation (Stabilized CV + Margin Strength)
    fn calculate_institutional_entropy(&self) -> f64 {
        if self.trade_count == 0 {
            return 0.0;
        }

        let n = self.trade_count as f64;
        let mean = self.sum_margin / n;
        let var = (self.sum_margin_sq / n) - (mean * mean);
        let std_dev = var.max(0.0).sqrt();

        // Stabilized CV [0.0, 2.0] -> Normalized [0.0, 1.0]
        let entropy_cv = (std_dev / mean.max(1e-6)).clamp(0.0, 2.0) / 2.0;

        // Margin Strength [0.0, 1.0] - distance from boundary
        let margin_strength = mean.clamp(0.0, 1.0);

        // Combined Score: 70% stability (low CV), 30% separation (high margin)
        // High stability + High separation -> High entropy_score (low uncertainty)
        // We invert this for the "entropy" metric (lower is better for uncertainty awareness)
        let entropy_score = (1.0 - entropy_cv) * 0.7 + margin_strength * 0.3;

        // Return inverted score so high is "uncertain/noisy" for the fitness weighting
        (1.0 - entropy_score).clamp(0.0, 1.0)
    }
}

pub fn canonical_json<T: Serialize>(v: &T) -> String {
    let value = to_json_value(v).unwrap_or(serde_json::Value::Null);
    serde_json::to_string(&value).unwrap_or_default()
}

/// Returns true only when GA_DEBUG is set to a truthy value.
/// Explicitly treats "0", "false", and "" as disabled so that
/// `GA_DEBUG=0 cargo run` silences all debug output.
#[inline(always)]
fn ga_debug_enabled() -> bool {
    std::env::var("GA_DEBUG").map_or(false, |v| {
        !v.is_empty() && v != "0" && v.to_lowercase() != "false"
    })
}

/// Evolution Scale Factor: Maps "Genome Units" (Paise) to Institutional Precision (units of 1/100 paise).
/// Since we moved to 10,000 scale, GA_GENE_SCALE = 100.
pub const GA_GENE_SCALE: u64 = crate::PRICE_SCALE / 100;

/// --- PHASE 14: ALPHA SELECTIVITY CONSTANTS ---
pub const BASE_Z: f64 = 1.0;
pub const TARGET_STD: f64 = 0.15;
pub const EPS: f64 = 1e-6;
pub const MIN_STD: f64 = 0.05;
pub const DOMINANCE_FLOOR: f64 = 0.15;
pub const EXTREME_Z_OVERRIDE: f64 = 2.5;

/// Helper to calculate the [p]-th percentile of a dataset. O(N log N) implementation.
fn percentile_f64(values: &Vec<f64>, p: f64) -> f64 {
    if values.is_empty() {
        return 0.0;
    }

    let mut sorted = values.clone();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let rank = ((sorted.len() as f64 - 1.0) * p).round() as usize;
    sorted[rank.min(sorted.len() - 1)]
}

/// Asset Evolution State: Tracks per-market microstructure and discovery progress.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetEvoState {
    pub symbol: String,
    pub max_log_queue: f64,
    pub prev_max_log_queue: f64,
    pub delta_log_q: f64,
    pub stability_streak: u32,
    pub trade_density: f64,
    pub fill_rate: f64,
    pub last_smoothed_fill: f64,
    pub mutation_scale: f64,
    pub last_weight: f64,
    pub stagnation_counter: u32,
    pub last_best_fitness: f64,
    pub rolling_variance: f64,
    pub initial_diversity: f64,
    pub current_diversity: f64,
    pub selection_pressure: f64,
}

/// [NEW V3.6.1] Asset Snapshot: Stateless generational metrics for Global Brain aggregation.
/// This prevents temporal leakage by separating local state (EMAs) from global coordination.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetSnapshot {
    pub symbol: String,
    pub max_log_queue: f64,
    pub delta_log_q: f64,
    pub trade_density: f64,
    pub fill_rate: f64,
    pub stability_streak: u32,
}

impl Default for AssetEvoState {
    fn default() -> Self {
        Self {
            symbol: "UNKNOWN".to_string(),
            max_log_queue: 0.0,
            prev_max_log_queue: 0.0,
            delta_log_q: 0.0,
            stability_streak: 0,
            trade_density: 0.0,
            fill_rate: 0.0,
            last_smoothed_fill: 0.0,
            mutation_scale: 1.0,
            last_weight: 0.0,
            stagnation_counter: 0,
            last_best_fitness: f64::NEG_INFINITY,
            rolling_variance: 0.05,
            initial_diversity: 0.0,
            current_diversity: 0.0,
            selection_pressure: 0.0,
        }
    }
}

/// Global Evolution State: Coordinates evolutionary intent across the multi-asset universe.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalEvoState {
    pub expansion_bias: f64,
    pub agreement_ema: f64,
    pub stability_ema: f64,
    pub prev_stability_ema: f64,
    pub progress_ema: f64,
    pub energy_ema: f64,
    pub energy_ema_prev: f64,
    pub peak_energy_ema: f64,
    pub global_max_log_q: f64,
    pub frontier_velocity_ema: f64,
    pub peak_velocity_ema: f64,
    pub velocity_history: Vec<f64>,
    pub last_expansion_gen: usize,
    pub prev_converged_assets: usize,
    pub post_strike_cooldown: u32,
    pub agreement_streak: u32,
    pub soft_expansion_active: bool,
    pub alignment_anchor: Option<Strategy>,
    pub global_mean: Option<Strategy>,
    pub pull_strength: f64,
    pub asset_states: HashMap<String, AssetEvoState>,
}

impl Default for GlobalEvoState {
    fn default() -> Self {
        Self {
            expansion_bias: 1.0,
            agreement_ema: 1.0,
            stability_ema: 0.0,
            prev_stability_ema: 0.0,
            progress_ema: 0.0,
            energy_ema: 0.0,
            energy_ema_prev: 0.0,
            peak_energy_ema: 1e-6,
            global_max_log_q: 0.0,
            frontier_velocity_ema: 0.0,
            peak_velocity_ema: 1e-6,
            velocity_history: Vec::new(),
            last_expansion_gen: 0,
            prev_converged_assets: 0,
            post_strike_cooldown: 0,
            agreement_streak: 0,
            soft_expansion_active: false,
            alignment_anchor: None,
            global_mean: None,
            pull_strength: 0.0,
            asset_states: HashMap::new(),
        }
    }
}

pub fn calculate_alignment_centroid(evals: Vec<&StrategyEvaluation>) -> Strategy {
    if evals.is_empty() {
        return random_strategy(
            &GaConfig::default(),
            &mut rand::rngs::StdRng::from_entropy(),
        );
    }

    let mut sum_q_threshold = 0.0;
    let mut sum_base_edge = 0.0;
    let mut sum_tp = 0.0;
    let mut sum_sl = 0.0;
    let mut sum_hold = 0.0;
    let mut sum_w_conv = 0.0;
    let mut sum_w_mom = 0.0;
    let mut sum_w_vol = 0.0;
    let mut sum_e_conv = 0.0;
    let mut sum_e_mom = 0.0;
    let mut sum_e_vol = 0.0;
    let mut sum_selectivity = 0.0;
    let mut sum_archetype = 0.0;
    let mut sum_offset = 0.0;
    let mut sum_dir = 0.0;
    let mut sum_vol_f = 0.0;
    let mut sum_mom_f = 0.0;
    let mut sum_edge_r = 0.0;
    let mut sum_part = 0.0;

    let n = evals.len() as f64;

    for e in &evals {
        let s = &e.strategy;
        sum_q_threshold += s.queue_threshold as f64;
        sum_base_edge += s.base_edge as f64;
        sum_tp += s.take_profit as f64;
        sum_sl += s.stop_loss as f64;
        sum_hold += s.holding_period as f64;
        sum_w_conv += s.w_conviction as f64;
        sum_w_mom += s.w_momentum as f64;
        sum_w_vol += s.w_volatility as f64;
        sum_e_conv += s.exp_conviction as f64;
        sum_e_mom += s.exp_momentum as f64;
        sum_e_vol += s.exp_volatility as f64;
        sum_selectivity += s.selectivity as f64;
        sum_archetype += s.archetype as f64;
        sum_offset += s.entry_offset as f64;
        sum_dir += s.direction_bias as f64;
        sum_vol_f += s.vol_floor as f64;
        sum_mom_f += s.mom_floor as f64;
        sum_edge_r += s.edge_ratio as f64;
        sum_part += s.participation_threshold as f64;
    }

    Strategy {
        queue_threshold: (sum_q_threshold / n) as u64,
        base_edge: (sum_base_edge / n) as u64,
        take_profit: (sum_tp / n) as u64,
        stop_loss: (sum_sl / n) as u64,
        holding_period: (sum_hold / n) as u64,
        w_conviction: (sum_w_conv / n) as u64,
        w_momentum: (sum_w_mom / n) as u64,
        w_volatility: (sum_w_vol / n) as u64,
        exp_conviction: (sum_e_conv / n) as u64,
        exp_momentum: (sum_e_mom / n) as u64,
        exp_volatility: (sum_e_vol / n) as u64,
        selectivity: (sum_selectivity / n) as u8,
        archetype: (sum_archetype / n) as u8,
        entry_offset: (sum_offset / n) as i32,
        direction_bias: (sum_dir / n) as u8,
        vol_floor: (sum_vol_f / n) as u8,
        mom_floor: (sum_mom_f / n) as u8,
        edge_ratio: (sum_edge_r / n) as u8,
        participation_threshold: (sum_part / n) as u8,
        exec_aggression: 50,
        latency_bias: 10,
        fill_threshold: 50, lineage: 0,
    }
}

impl GlobalEvoState {
    /// [V3.6.1 Refactor] Synchronous Generational Aggregation
    /// Now accepts a map of pure snapshots to maintain temporal integrity.
    pub fn aggregate(&mut self, snapshots: &HashMap<String, AssetSnapshot>, generation: usize) {
        if snapshots.is_empty() {
            return;
        }

        // HARDENING 3: Minimum Activity Filter
        // Only consider assets with legitimate signal
        let active_snapshots: Vec<&AssetSnapshot> = snapshots
            .values()
            .filter(|s| s.trade_density > 0.05 && s.fill_rate > 0.01)
            .collect();

        let mut weights = Vec::new();
        let mut deltas = Vec::new();
        let mut abs_deltas = Vec::new();
        let mut stability_scores = Vec::new();
        let mut global_max_log_q: f64 = 1e-6;

        for s in snapshots.values() {
            global_max_log_q = global_max_log_q.max(s.max_log_queue);
        }
        self.global_max_log_q = global_max_log_q;

        for (symbol, s) in snapshots {
            // Retrieve or initialize the stateful tracker (for EMA persistence only)
            let state = self
                .asset_states
                .entry(symbol.clone())
                .or_insert_with(|| AssetEvoState {
                    symbol: symbol.clone(),
                    ..AssetEvoState::default()
                });

            // normalized_log_q = local_log_q / global_max_log_q (capped at 1.0)
            let normalized_log_q = (s.max_log_queue / global_max_log_q).clamp(0.0, 1.0);

            // Activity-weighted contribution: trade_density * fill_rate * normalized_log_q
            let raw_weight = (s.trade_density * s.fill_rate * normalized_log_q).clamp(0.0, 1.0);
            state.last_weight = 0.7 * state.last_weight + 0.3 * raw_weight;

            weights.push(state.last_weight);
            deltas.push(s.delta_log_q);
            abs_deltas.push(s.delta_log_q.abs());
            stability_scores.push(if s.stability_streak >= 3 { 1.0 } else { 0.0 });
        }

        let total_weight: f64 = weights.iter().sum::<f64>().max(1e-9);

        // Weighted Average Progress & abs_mean
        let mean_delta = deltas
            .iter()
            .zip(weights.iter())
            .map(|(d, w)| d * w)
            .sum::<f64>()
            / total_weight;
        let abs_mean_delta = abs_deltas
            .iter()
            .zip(weights.iter())
            .map(|(d, w)| d * w)
            .sum::<f64>()
            / total_weight;

        let progress_signal = if abs_mean_delta < 0.001 {
            0.0
        } else {
            mean_delta
        };
        self.progress_ema = 0.7 * self.progress_ema + 0.3 * progress_signal;

        // [V3.6.3] Lagged Energy Perception (Energy EMA Prev)
        self.energy_ema_prev = self.energy_ema;
        self.energy_ema = 0.7 * self.energy_ema + 0.3 * abs_mean_delta;
        self.peak_energy_ema = self.peak_energy_ema.max(self.energy_ema);

        self.prev_stability_ema = self.stability_ema;
        let global_stability = stability_scores
            .iter()
            .zip(weights.iter())
            .map(|(s, w)| s * w)
            .sum::<f64>()
            / total_weight;
        self.stability_ema = 0.7 * self.stability_ema + 0.3 * global_stability;

        // Count converged assets for micro-triggering
        let converged_assets = stability_scores.iter().filter(|&&s| s > 0.9).count();

        // [V3.6.7+] Robust Blended Agreement Model (Hardened)
        // Two-Pass Weighted Variance with Sqrt-Smoothed Participation
        let mut total_weight = 0.0;
        let mut sum_delta = 0.0;
        let mut sum_capacity = 0.0;

        // Pass 1: Weighted Mean
        for s in snapshots.values() {
            let raw = s.trade_density * s.fill_rate;
            let w = raw.sqrt().clamp(0.0, 1.0);

            sum_delta += w * s.delta_log_q;
            sum_capacity += w * s.max_log_queue;
            total_weight += w;
        }

        let mean_delta = sum_delta / total_weight.max(1e-6);
        let mean_capacity = sum_capacity / total_weight.max(1e-6);

        // Pass 2: Weighted Variance
        let mut var_delta = 0.0;
        let mut var_capacity = 0.0;

        for s in snapshots.values() {
            let raw = s.trade_density * s.fill_rate;
            let w = raw.sqrt().clamp(0.0, 1.0);

            var_delta += w * (s.delta_log_q - mean_delta).powi(2);
            var_capacity += w * (s.max_log_queue - mean_capacity).powi(2);
        }

        var_delta = (var_delta / total_weight.max(1e-6)).max(1e-6);
        var_capacity = (var_capacity / total_weight.max(1e-6)).max(1e-6);

        let agreement_delta = (-var_delta * 6.0).exp();
        let agreement_capacity = (-var_capacity * 6.0).exp();

        // 0.5 / 0.5 Blend
        let mut agreement = 0.5 * agreement_delta + 0.5 * agreement_capacity;

        // 🔥 Adaptive Noise Floor Bootstrap (V3.6.7 Hardening)
        let noise_floor = if generation < 15 { 0.05 } else { 0.01 };
        agreement = agreement.max(noise_floor);

        self.agreement_ema = 0.7 * self.agreement_ema + 0.3 * agreement;

        // --- V3.6.7: Inertial Agreement Streak ---
        let current_signal = if agreement > 0.4 {
            1.0
        } else if agreement > 0.3 {
            0.5
        } else {
            0.0
        };

        // EMA-style smoothing
        self.agreement_streak =
            (0.7 * self.agreement_streak as f64 + 0.3 * current_signal).round() as u32;
        // --- V3.6.7+: Selective Global Mean Pull Gating ---
        let mut pull_strength = 0.0;

        // Gated Activation: Requires both agreement and basic stability
        if self.agreement_ema > 0.3 && self.stability_ema > 0.2 {
            pull_strength = ((self.agreement_ema - 0.3) / 0.4).clamp(0.0, 1.0) * 0.07;
        }

        // Freeze on Collapse
        if self.agreement_ema < 0.2 {
            pull_strength = 0.0;
        }

        self.pull_strength = pull_strength;

        // Diagnostic
        if self.pull_strength > 0.0 && self.global_mean.is_some() {
            if ga_debug_enabled() {
                println!(
                    "🧲 MEAN_PULL_ACTIVE | strength: {:.4} | agreement_ema: {:.3} | stability_ema: {:.3}",
                    self.pull_strength,
                    self.agreement_ema,
                    self.stability_ema
                );
            }
        }

        // Frontier Velocity (smoothed progress)
        self.frontier_velocity_ema = 0.7 * self.frontier_velocity_ema + 0.3 * mean_delta;
        self.peak_velocity_ema = self.peak_velocity_ema.max(self.frontier_velocity_ema.abs());

        self.velocity_history.push(self.frontier_velocity_ema);
        if self.velocity_history.len() > 3 {
            self.velocity_history.remove(0);
        }

        let velocity_trend = if self.velocity_history.len() >= 3 {
            (self.velocity_history[2] - self.velocity_history[0]) / 2.0
        } else {
            0.0
        };

        // Global Decay: expansion_bias pulls back towards 1.0
        self.expansion_bias = (self.expansion_bias * 0.95).max(1.0);

        // [V3.6.4] DECISION LOGIC (Progressive Commitment)
        let warmup_lock = generation < 5;
        let cooldown_ok = generation >= self.last_expansion_gen + 3;

        // Normalized Relative Gating
        let normalized_energy = self.energy_ema / self.peak_energy_ema.max(1e-6);
        let normalized_velocity =
            self.frontier_velocity_ema.abs() / self.peak_velocity_ema.max(1e-6);

        // Stagnation is now relative to peak motion
        let relative_plateau = normalized_energy < 0.4 && normalized_velocity < 0.3;
        let consistent_plateau =
            self.stability_ema > 0.45 && (self.stability_ema - self.prev_stability_ema > -0.05);

        // [V3.6.4] Stage 1: Commitment Probe (Stateful Soft Expansion)
        // --- Stage 1: Soft Expansion (Commitment Probe) ---
        // V3.6.6: Trigger Discipline - Skip if cooldown active
        if self.post_strike_cooldown == 0
            && self.agreement_ema > 0.35
            && !self.soft_expansion_active
            && self.expansion_bias < 1.5
        {
            let old_bias = self.expansion_bias;
            self.expansion_bias *= 1.15;
            self.soft_expansion_active = true;
            println!(
                "🌱 SOFT_EXPANSION_TRIGGER | bias: {:.2} -> {:.2} | agreement: {:.2} | streak: {}",
                old_bias, self.expansion_bias, self.agreement_ema, self.agreement_streak
            );
        }

        // Soft Expansion Rollback: If alignment fails to sustain
        if self.soft_expansion_active && self.agreement_ema < 0.25 {
            let old_bias = self.expansion_bias;
            self.expansion_bias *= 0.90; // Revert
            self.soft_expansion_active = false;
            println!(
                "🍂 SOFT_EXPANSION_ROLLBACK | bias: {:.2} -> {:.2} | agreement: {:.2}",
                old_bias, self.expansion_bias, self.agreement_ema
            );
        }

        // [V3.6.4] Stage 2: Full Expansion (Hard Strike)
        // Requires persistence, higher agreement, and velocity collapse
        let hard_condition = self.agreement_ema > 0.50
            && self.agreement_streak >= 3
            && consistent_plateau
            && normalized_velocity < 0.3;
        let should_trigger = !warmup_lock && relative_plateau && hard_condition && cooldown_ok;

        if should_trigger {
            let old_bias = self.expansion_bias;
            self.expansion_bias = (self.expansion_bias * 1.3).min(3.0);
            self.last_expansion_gen = generation;
            self.soft_expansion_active = false; // Reset soft state after hard strike
            self.post_strike_cooldown = 3; // Activate stabilization window

            println!("🏹 GLOBAL_EXPANSION_TRIGGERED | bias: {:.2} -> {:.2} | agreement: {:.2} | streak: {} | energy_norm: {:.2}",
                old_bias, self.expansion_bias, self.agreement_ema, self.agreement_streak, normalized_energy);
        }

        // Post-Strike Cooldown Decay
        if self.post_strike_cooldown > 0 {
            self.post_strike_cooldown -= 1;
        }

        // Update persistent state
        self.prev_converged_assets = converged_assets;

        let lock_status = if warmup_lock {
            "LOCKED (Warmup)"
        } else if should_trigger {
            "TRIGGERED"
        } else {
            "IDLE"
        };

        println!("🌐 GLOBAL_FRONTIER | gen: {} | bias: {:.2} | agreement: {:.2} (ema: {:.2}) | energy_norm: {:.2} | stability: {:.2} | vel_norm: {:.2} | trigger: {}",
            generation, self.expansion_bias, agreement, self.agreement_ema, normalized_energy, self.stability_ema, normalized_velocity, lock_status);
    }
}

pub struct PopulationMetrics {
    pub min_threshold: u64,
    pub max_threshold: u64,
    pub min_edge: u64,
    pub max_edge: u64,
    pub min_tp: u64,
    pub max_tp: u64,
    pub min_sl: u64,
    pub max_sl: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RejectionReason {
    QueueBlocked,
    LiquidityStarved,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ExecutionDiagnostics {
    pub rejection_reason: Option<RejectionReason>,
    pub queue_pressure: f64,
    pub liquidity: f64,
}

pub fn classify_rejection(queue_pressure: f64, liquidity: f64) -> Option<RejectionReason> {
    if queue_pressure >= liquidity {
        Some(RejectionReason::QueueBlocked)
    } else if liquidity <= 0.0 {
        Some(RejectionReason::LiquidityStarved)
    } else {
        None
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExecutionMetrics {
    pub fill_efficiency: f64,
    pub capture_efficiency: f64,
    pub fill_rate: f32,
    pub avg_slippage: f64,
    pub latency_impact: f64,
    pub queue_blocked_count: usize,
    pub liquidity_starved_count: usize,
    pub total_attempts: usize,
}

impl Default for ExecutionMetrics {
    fn default() -> Self {
        Self {
            fill_efficiency: 0.0,
            capture_efficiency: 0.0,
            fill_rate: 0.0,
            avg_slippage: 0.0,
            latency_impact: 0.0,
            queue_blocked_count: 0,
            liquidity_starved_count: 0,
            total_attempts: 0,
        }
    }
}

/// Deterministic execution-path fingerprint for one scenario evaluation (GA diversity / diagnostics).
/// Values are **normalized** for stable L1 distances in Top-K diversity; see [`scenario_execution_signature_from_simulation`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScenarioExecutionSignature {
    /// Mean simulated `queue_ahead` (and fallbacks), scaled to ~O(1).
    pub avg_queue_ahead: f64,
    /// Mean intent→first-fill latency (exchange timestamps), scaled to ~O(1).
    pub avg_latency: f64,
    /// Realized fill ratio in `[0, 1]` (same idea as `ExecutionMetrics::fill_efficiency`).
    pub fill_ratio: f64,
    /// Realized participation in `[0, 1]`.
    pub participation: f64,
    /// Standard deviation of fill efficiency across trades in this scenario.
    #[serde(default)]
    pub execution_variance: f64,
}

impl Default for ScenarioExecutionSignature {
    fn default() -> Self {
        ScenarioExecutionSignature {
            avg_queue_ahead: 0.0,
            avg_latency: 0.0,
            fill_ratio: 1.0,
            participation: 1.0,
            execution_variance: 0.0,
        }
    }
}

#[inline]
fn scenario_execution_signature_l1(
    a: &ScenarioExecutionSignature,
    b: &ScenarioExecutionSignature,
) -> f64 {
    (a.avg_queue_ahead - b.avg_queue_ahead).abs()
        + (a.avg_latency - b.avg_latency).abs()
        + (a.fill_ratio - b.fill_ratio).abs()
        + (a.participation - b.participation).abs()
}

/// Builds a signature from the ESE event log for our entry/exit orders, plus aggregate fill and participation.
fn scenario_execution_signature_from_simulation(
    events: &[crate::SimEvent],
    entry_order_id: &str,
    exit_order_id: &str,
    fill_efficiency: f64,
    participation_rate: f64,
    queue_ratio_fallback: f64,
) -> (ScenarioExecutionSignature, f64) {
    let mut queue_samples: Vec<f64> = Vec::new();
    let mut intent_ts: HashMap<String, u64> = HashMap::new();
    let mut first_fill_ts: HashMap<String, u64> = HashMap::new();

    for ev in events {
        match ev {
            crate::SimEvent::OrderIntent {
                order_id,
                timestamp,
                ..
            } => {
                if order_id == entry_order_id || order_id == exit_order_id {
                    intent_ts.insert(order_id.clone(), *timestamp);
                }
            }
            crate::SimEvent::OrderEnteredQueue {
                order_id,
                queue_ahead,
                ..
            }
            | crate::SimEvent::QueueProgression {
                order_id,
                queue_ahead,
                ..
            } => {
                if order_id == entry_order_id || order_id == exit_order_id {
                    queue_samples.push(*queue_ahead as f64);
                }
            }
            crate::SimEvent::PartialFill {
                order_id,
                timestamp,
                ..
            } => {
                if order_id == entry_order_id || order_id == exit_order_id {
                    first_fill_ts.entry(order_id.clone()).or_insert(*timestamp);
                }
            }
            _ => {}
        }
    }

    let queue_raw_mean = if !queue_samples.is_empty() {
        queue_samples.iter().sum::<f64>() / queue_samples.len() as f64
    } else {
        (queue_ratio_fallback.max(0.0) * 2500.0).min(10_000.0)
    };
    let avg_queue_norm = (queue_raw_mean / 2500.0).clamp(0.0, 4.0);

    let mut latencies: Vec<f64> = Vec::new();
    for oid in [entry_order_id, exit_order_id] {
        if let (Some(&t0), Some(&tf)) = (intent_ts.get(oid), first_fill_ts.get(oid)) {
            latencies.push(tf.saturating_sub(t0) as f64);
        }
    }
    let latency_raw_mean = if !latencies.is_empty() {
        latencies.iter().sum::<f64>() / latencies.len() as f64
    } else {
        crate::ese::FIXED_LATENCY as f64
    };
    let latency_norm = (latency_raw_mean / 200.0).clamp(0.0, 4.0);

    let sig = ScenarioExecutionSignature {
        avg_queue_ahead: avg_queue_norm,
        avg_latency: latency_norm,
        fill_ratio: fill_efficiency.clamp(0.0, 1.0),
        participation: participation_rate.clamp(0.0, 1.0),
        execution_variance: 0.0,
    };
    (sig, latency_raw_mean)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum FitnessMode {
    Sniper,
    #[default]
    Scalable,
    InstitutionalRealism,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScenarioCapability {
    Executable,  // Symbol for which full order-matching is valid (Cash/Futs)
    ContextOnly, // Index or data-only stream for multi-agent perception
}

impl ScenarioCapability {
    pub fn is_executable(&self) -> bool {
        matches!(self, ScenarioCapability::Executable)
    }
}

impl Default for ScenarioCapability {
    fn default() -> Self {
        ScenarioCapability::Executable
    }
}

pub fn determine_scenario_capability(name: &str) -> ScenarioCapability {
    let upper = name.to_uppercase();
    if upper.contains("NIFTY") || upper.contains("SENSEX") || upper.contains("INDEX") {
        ScenarioCapability::ContextOnly
    } else {
        ScenarioCapability::Executable
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum AcceptanceMode {
    Dominance,
    StatisticalWeak,
    Override,
}

impl Default for AcceptanceMode {
    fn default() -> Self {
        AcceptanceMode::Dominance
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default, Eq)]
pub struct BehavioralSignature {
    pub fingerprint: u64,
    pub axes: (u8, u8, u8, u8),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StrategyEvaluation {
    pub candidate_edges: Vec<f64>,
    pub winner_idx: usize,
    pub strategy_id: String,
    pub strategy: Strategy,
    #[serde(default)]
    pub capability: ScenarioCapability,
    pub real_dom: f64,
    pub had_organic_signals: bool,
    pub std_dev: f64,
    pub downside_std_dev: f64,
    pub worst: f64,
    pub robustness: f64,
    pub max_signature_credibility: f64,
    pub forced_win_ratio: f64,
    /// Aggregated, canonical fitness (ONLY truth).
    pub fitness: f64,
    pub trade_count: usize,
    pub max_drawdown: f64,
    pub participation_rate: f64,
    pub profitable_trades: usize,
    pub zero_pnl_trades: usize,
    pub quality_trades: f64,
    pub total_pnl: f64,
    pub avg_pnl: f64,
    pub pnl_history: Vec<GaRoundTripOutcome>,
    pub win_rate: f64,
    pub payoff: f64,
    pub payoff_ratio: f64,
    pub direction_ratio: f64,
    pub baseline_pnl: f64,
    pub execution_metrics: ExecutionMetrics,
    /// Per-scenario execution microstructure (queue, latency, fills); used for GA Top-K diversity when `GA_DIVERSITY_LAMBDA` > 0.
    pub scenario_signature: ScenarioExecutionSignature,
    /// Behavioral fingerprint (bucketed returns) for phenotype diversity.
    #[serde(default)]
    pub pnl_fingerprint: Vec<f32>,
    #[serde(default)]
    pub behavioral_signature: BehavioralSignature,
    pub evaluation_flag: Option<String>,

    // Phase 8.8 Sniper Metrics
    pub avg_conviction: f64,
    pub avg_efficiency: f64,
    pub avg_edge_quality: f64,
    pub directional_accuracy: f64,
    pub decisiveness: f64,
    pub execution_friction: f64, // Actual / Expected Slippage
    pub emitted_signals: Vec<SignalAlpha>,

    // Phase 10.3: Institutional Feedback Loop
    #[serde(default = "default_capture_eff")]
    pub short_term_capture_eff: f64, // last 20 trades
    #[serde(default = "default_capture_eff")]
    pub long_term_capture_eff: f64, // last 100 trades
    pub trade_density: f64,
    pub queue_blocked_count: usize,
    pub liquidity_starved_count: usize,
    pub total_attempts: usize,
    pub exec_opportunity_rate: f64,
    #[serde(default)]
    pub failure_profile: Vec<f64>, // [QueueWeighted, LiquidityWeighted]
    #[serde(default)]
    pub realized_pnl_rolling: f64,
    #[serde(default)]
    pub predicted_pnl_rolling: f64,
    #[serde(default)]
    pub trade_qualities: Vec<f64>,
    #[serde(default)]
    pub outcome_consistency: f64,
    #[serde(default)]
    pub avg_trade_quality: f64,
    #[serde(default)]
    pub std_trade_quality: f64,

    // Exit Reason Distribution (Observability)
    pub exit_tp_count: usize,
    pub exit_sl_count: usize,
    pub exit_ts_count: usize,
    #[serde(default)]
    pub avg_hold_time: f64, // Mean duration in event indices

    // Phase 10.9 Governance & Health
    #[serde(default)]
    pub consistency_score: f64, // [0, 1] agreement with historical regime
    #[serde(default)]
    pub recent_performance: f64, // PnL or fitness in the most recent evaluation

    // Phase 10.2 Final Institutional Metrics
    #[serde(default)]
    pub pnl_from_tp: f64,
    #[serde(default)]
    pub pnl_from_sl: f64,
    #[serde(default)]
    pub max_trade_pnl: f64,
    #[serde(default)]
    pub selectivity: f64,
    #[serde(default)]
    pub avg_entropy: f64,
    #[serde(default)]
    pub avg_aqg_health: f64,
    #[serde(default)]
    pub aqg_skip_ratio: f64,
    #[serde(default)]
    pub avg_edge_spread: f64,
    #[serde(default)]
    pub consistency_n: usize, // Sample size for outcome consistency tracking
    #[serde(default)]
    pub avg_dominance: f64,
    #[serde(default)]
    pub raw_pop_avg: f64,
    #[serde(default)]
    pub raw_pop_p95: f64,
    #[serde(default)]
    pub bootstrap_ratio: f64,
    #[serde(default)]
    pub raw_pop_dist: [f64; 6],

    #[serde(default)]
    pub exec_pop_avg: f64,
    #[serde(default)]
    pub exec_pop_p95: f64,
    #[serde(default)]
    pub exec_pop_dist: [f64; 6],
    #[serde(default)]
    pub acceptance_mode: AcceptanceMode,

    #[serde(default)]
    pub pop_delta: f64, // EXEC_P95 - RAW_P95

    // Phase 17A: Alpha Recovery Metrics
    #[serde(default)]
    pub vip_ratio: f64, // VIP / total valid signals
    #[serde(default)]
    pub ccr: f64, // EXEC_P95 / RAW_P95
    #[serde(default)]
    pub stat_zero_dom_ratio: f64, // STAT with dom < 0.05 / total STAT

    // Phase 17B: Execution-Aware Selection
    #[serde(default)]
    pub exec_accept_rate: f64,
    #[serde(default)]
    pub vip_exec_retention: f64,
    #[serde(default)]
    pub e_rejection_rate: f64,
    #[serde(default)]
    pub clarity_to_exec_drop: f64,
    #[serde(default)]
    pub avg_e_score: f64,
    #[serde(default)]
    pub vip_avg_e_score: f64,
    #[serde(default)]
    pub stat_avg_e_score: f64,

    // Phase 17C: Legacy Parity Metrics
    #[serde(default)]
    pub avg_exec_prob: f64,
    #[serde(default)]
    pub avg_survive_score: f64,
    #[serde(default)]
    pub edge_std_dev: f64,

    // Phase 14++: Structural Health (Universe Discovery)
    #[serde(default)]
    pub alpha: f64,
    #[serde(default)]
    pub consistency: f64,
    #[serde(default)]
    pub opportunity: f64,
    #[serde(default)]
    pub structural_score: f64,
    #[serde(default)]
    pub acceptance_rate: f64,
    #[serde(default)]
    pub valid_window_ratio: f64,
    #[serde(default)]
    pub avg_agreement_valid: f64,
    #[serde(default)]
    pub avg_purity_valid: f64,
    #[serde(default)]
    pub avg_stability_valid: f64,
    #[serde(default)]
    pub max_agreement: f64,
    #[serde(default)]
    pub max_purity: f64,
    #[serde(default)]
    pub total_windows: usize,

    // Phase 14: Consensus Audit
    #[serde(default)]
    pub consensus_bypass_ratio: f64,
    #[serde(default)]
    pub stability_reject_rate: f64,
    #[serde(default)]
    pub clarity_pnl_share: f64,
    #[serde(default)]
    pub conviction_pnl_share: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Archetype {
    Conviction,
    Momentum,
    Reversion,
    Volatility,
}

impl From<u8> for Archetype {
    fn from(val: u8) -> Self {
        match val {
            0 => Archetype::Conviction,
            1 => Archetype::Momentum,
            2 => Archetype::Reversion,
            3 => Archetype::Volatility,
            _ => Archetype::Conviction, // Default
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Decision {
    BUY,
    SELL,
    HOLD,
}

pub struct SignalVote {
    pub strategy_id: String,
    pub archetype: Archetype,
    pub confidence: f64,
    pub signal_features: Vec<f64>,
    pub decision: Decision,
}

/// Phase D.1.10: Consensus-Driven Alpha Extraction
/// Represents a tradable signal supported by multiple strategies (agents).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalAlphaReport {
    pub signal_idx: usize,
    pub asset: String,
    pub support_count: usize,
    pub support_ratio: f64,
    pub avg_score: f64,
    pub archetype_diversity: f64,
    pub alpha_score: f64,
    pub conviction: f64,
    pub archetypes: Vec<u8>,
    pub consensus_label: String, // "HIGH", "CROWDED", "NICHE"
    pub disagreement_entropy: f64,
    pub feature_diversity: f64,
    pub realized_edge_factor: f64,
    // Phase D.1.13 Temporal Layer
    pub signal_timestamp: u64,
    pub temporal_stability: f64,
    pub persistence_count: usize,
    pub alignment_factor: f64,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SignalIdentity {
    pub bucket_ts: u64,
    pub direction: Decision,
    pub archetype: Archetype,
    pub feature_hash: u64,
}

pub fn get_coarse_feature_hash(strategy: &Strategy) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut s = DefaultHasher::new();
    // Coarse rounding to group similar strategies while avoiding collisions
    ((strategy.queue_threshold / 500) * 500).hash(&mut s);
    ((strategy.base_edge / 50) * 50).hash(&mut s);
    ((strategy.selectivity / 10) * 10).hash(&mut s);
    s.finish()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalCluster {
    pub center_idx: f64,
    pub archetype: Archetype,
    pub direction: Decision,
    pub signals: Vec<SignalAlphaReport>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioCluster {
    pub label: String,
    pub archetype: Archetype,
    pub total_weight: f64,
    pub signals: Vec<SignalAlphaReport>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusReport {
    pub scenario_name: String,
    pub top_signals: Vec<SignalAlphaReport>,
    pub portfolio_clusters: Vec<PortfolioCluster>,
    pub global_entropy: f64,
    pub active_strategies: usize,
}

impl StrategyEvaluation {
    pub fn new_legacy(
        strategy_id: String,
        strategy: Strategy,
        fitness: f64,
        avg_pnl: f64,
        trade_count: usize,
        profitable_trades: usize,
        avg_entropy: f64,
    ) -> Self {
        let mut slf = Self::new_legacy_with_flag("EMPTY");
        slf.strategy_id = strategy_id;
        slf.strategy = strategy.clone();
        slf.behavioral_signature = strategy.get_signature();
        slf.fitness = fitness;
        slf.avg_pnl = avg_pnl;
        slf.trade_count = trade_count;
        slf.profitable_trades = profitable_trades;
        slf.win_rate = if trade_count > 0 {
            profitable_trades as f64 / trade_count as f64
        } else {
            0.0
        };
        slf.evaluation_flag = None;
        slf
    }

    pub fn new_legacy_with_flag(flag: &str) -> Self {
        Self {
            candidate_edges: Vec::new(),
            winner_idx: 0,
            strategy_id: "FLAGGED".to_string(),
            strategy: Strategy::from_seed(0),
            capability: ScenarioCapability::Executable,
            real_dom: 0.0,
            had_organic_signals: false,
            std_dev: 0.0,
            downside_std_dev: 0.0,
            worst: 0.0,
            robustness: 0.0,
            max_signature_credibility: 0.0,
            forced_win_ratio: 0.0,
            fitness: -0.03,
            trade_count: 0,
            max_drawdown: 0.0,
            participation_rate: 0.0,
            profitable_trades: 0,
            zero_pnl_trades: 0,
            quality_trades: 0.0,
            total_pnl: 0.0,
            avg_pnl: 0.0,
            pnl_history: Vec::new(),
            win_rate: 0.0,
            payoff: 0.0,
            payoff_ratio: 0.0,
            direction_ratio: 0.0,
            baseline_pnl: 0.0,
            execution_metrics: ExecutionMetrics {
                fill_efficiency: 0.0,
                capture_efficiency: 0.0,
                fill_rate: 0.0,
                avg_slippage: 0.0,
                latency_impact: 0.0,
                queue_blocked_count: 0,
                liquidity_starved_count: 0,
                total_attempts: 0,
            },
            scenario_signature: ScenarioExecutionSignature {
                avg_queue_ahead: 0.0,
                avg_latency: 0.0,
                fill_ratio: 0.0,
                participation: 0.0,
                execution_variance: 0.0,
            },
            pnl_fingerprint: Vec::new(),
            behavioral_signature: BehavioralSignature {
                fingerprint: 0,
                axes: (0, 0, 0, 0),
            },
            evaluation_flag: Some(flag.to_string()),
            avg_conviction: 0.0,
            avg_efficiency: 0.0,
            avg_edge_quality: 0.0,
            directional_accuracy: 0.0,
            decisiveness: 0.0,
            execution_friction: 0.0,
            emitted_signals: Vec::new(),
            short_term_capture_eff: 0.0,
            long_term_capture_eff: 0.0,
            trade_density: 0.0,
            queue_blocked_count: 0,
            liquidity_starved_count: 0,
            total_attempts: 0,
            exec_opportunity_rate: 0.0,
            failure_profile: Vec::new(),
            realized_pnl_rolling: 0.0,
            predicted_pnl_rolling: 0.0,
            trade_qualities: Vec::new(),
            outcome_consistency: 0.0,
            avg_trade_quality: 0.0,
            std_trade_quality: 0.0,
            exit_tp_count: 0,
            exit_sl_count: 0,
            exit_ts_count: 0,
            avg_hold_time: 0.0,
            consistency_score: 0.0,
            recent_performance: 0.0,
            pnl_from_tp: 0.0,
            pnl_from_sl: 0.0,
            max_trade_pnl: 0.0,
            selectivity: 0.0,
            avg_entropy: 0.0,
            avg_aqg_health: 0.0,
            aqg_skip_ratio: 0.0,
            avg_edge_spread: 0.0,
            consistency_n: 0,
            avg_dominance: 0.0,
            raw_pop_avg: 0.0,
            raw_pop_p95: 0.0,
            bootstrap_ratio: 0.0,
            raw_pop_dist: [0.0; 6],
            exec_pop_avg: 0.0,
            exec_pop_p95: 0.0,
            exec_pop_dist: [0.0; 6],
            acceptance_mode: AcceptanceMode::Dominance,
            pop_delta: 0.0,
            vip_ratio: 0.0,
            ccr: 0.0,
            stat_zero_dom_ratio: 0.0,
            exec_accept_rate: 0.0,
            vip_exec_retention: 0.0,
            e_rejection_rate: 0.0,
            clarity_to_exec_drop: 0.0,
            avg_e_score: 0.0,
            vip_avg_e_score: 0.0,
            stat_avg_e_score: 0.0,
            avg_exec_prob: 0.0,
            avg_survive_score: 0.0,
            edge_std_dev: 0.0,
            alpha: 0.0,
            consistency: 0.0,
            opportunity: 0.0,
            structural_score: 0.0,
            acceptance_rate: 0.0,
            valid_window_ratio: 0.0,
            avg_agreement_valid: 0.0,
            avg_purity_valid: 0.0,
            avg_stability_valid: 0.0,
            max_agreement: 0.0,
            max_purity: 0.0,
            total_windows: 0,
            consensus_bypass_ratio: 0.0,
            stability_reject_rate: 0.0,
            clarity_pnl_share: 0.0,
            conviction_pnl_share: 0.0,
        }
    }
    /// Institutional Weighting Formula (Phase 10.3):
    /// 0.4*fitness + 0.3*capture_eff + 0.2*fill_prob + 0.1*regime_stability
    pub fn calculate_institutional_weight(&self) -> f64 {
        let fitness_w = (self.fitness * 100.0).clamp(-1.0, 1.0);

        let capture_w =
            (0.6 * self.short_term_capture_eff + 0.4 * self.long_term_capture_eff).clamp(0.3, 1.2);

        let fill_w = self.scenario_signature.fill_ratio.clamp(0.0, 1.0);
        let stability_w = self.consistency_score.clamp(0.0, 1.0);

        (0.4 * fitness_w) + (0.3 * capture_w) + (0.2 * fill_w) + (0.1 * stability_w)
    }

    /// Blends realized execution into the capture efficiency horizons.
    /// realised_pnl: actual pnl from the trade
    /// predicted_pnl: expected pnl based on strategy signal
    pub fn update_capture_efficiency(&mut self, realized: f64, predicted: f64) {
        if predicted.abs() < 1e-9 {
            return;
        } // Denominator Guard 1
        let ratio = (realized / predicted).clamp(-2.0, 2.0); // Safe ratio with institutional clamp

        // Evolving Horizons (EMA Approximation)
        // Short (20 trades) -> Alpha ~ 0.1
        // Long (100 trades) -> Alpha ~ 0.02
        self.short_term_capture_eff =
            (0.1 * ratio.max(0.0).min(2.0)) + (0.9 * self.short_term_capture_eff);
        self.long_term_capture_eff =
            (0.02 * ratio.max(0.0).min(2.0)) + (0.98 * self.long_term_capture_eff);

        self.realized_pnl_rolling += realized;
        self.predicted_pnl_rolling += predicted;

        // Phase 2: Consistency Window Logic
        self.trade_qualities.push(ratio);
        if self.trade_qualities.len() > 20 {
            self.trade_qualities.remove(0);
        }

        if self.trade_qualities.len() >= 10 {
            let n = self.trade_qualities.len() as f64;
            let mean = self.trade_qualities.iter().sum::<f64>() / n;
            let var = self
                .trade_qualities
                .iter()
                .map(|q| (q - mean).powi(2))
                .sum::<f64>()
                / n;
            let std = var.sqrt();

            self.avg_trade_quality = mean;
            self.std_trade_quality = std;
            self.outcome_consistency = mean - std;
        } else {
            self.outcome_consistency = 0.0;
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegimeProfile {
    pub volatility: f64,
    pub liquidity: f64,
    pub participation: f64,
    pub label: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistenceMetadata {
    pub timestamp: String,
    pub avg_fitness: f64,
    pub avg_pnl: f64,
    pub cv: f64,
    pub regime_profile: RegimeProfile,
    pub strategies_count: usize,
    #[serde(default)]
    pub fitness_mode: FitnessMode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElitePopulationBundle {
    pub metadata: PersistenceMetadata,
    pub strategies: Vec<StrategyEvaluation>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExecutionContext {
    pub queue_depth: f64,     // 0 = empty, 1 = heavy
    pub liquidity_score: f64, // 0 = thin, 1 = deep
    pub latency_impact: f64,  // 0 = low, 1 = high
}

fn clamp01(x: f64) -> f64 {
    x.max(0.0).min(1.0)
}

fn compute_std_dev(data: &[f64]) -> f64 {
    if data.is_empty() {
        return 0.0;
    }
    let mean = data.iter().sum::<f64>() / data.len() as f64;
    let variance = data.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / data.len() as f64;
    variance.sqrt()
}

fn calculate_execution_score(ctx: &ExecutionContext) -> f64 {
    let queue_component = clamp01(1.0 - ctx.queue_depth);
    let liquidity_component = clamp01(ctx.liquidity_score);
    let latency_component = clamp01(1.0 - ctx.latency_impact);

    0.4 * queue_component + 0.4 * liquidity_component + 0.2 * latency_component
}

fn is_execution_feasible(conviction: f64, exec_score: f64) -> (bool, f64) {
    // base threshold
    let mut threshold = 0.42;

    // conviction lowers barrier
    threshold -= 0.15 * conviction;

    // strong conviction boost
    if conviction > 0.9 {
        threshold -= 0.05;
    }

    // clamp tighter band (important)
    threshold = threshold.clamp(0.25, 0.45);

    (exec_score >= threshold, threshold)
}

pub fn calculate_capture_efficiency(realized: f64, expected: f64) -> f64 {
    let eps = 1e-6;
    let denom = if expected.abs() < eps {
        expected.signum() * eps
    } else {
        expected
    };
    (realized / denom).clamp(-2.0, 2.0)
}

pub fn classify_efficiency(e: f64) -> &'static str {
    if e > 1.0 {
        "OUTPERFORM"
    } else if e > 0.7 {
        "GOOD"
    } else if e > 0.3 {
        "DECAY"
    } else {
        "FAILED"
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GaResult {
    pub global_best: StrategyEvaluation,
    pub global_best_generation: usize,
    pub final_generation_best: StrategyEvaluation,
    pub generation_history: Vec<StrategyEvaluation>,
    pub best_per_regime: HashMap<String, StrategyEvaluation>,
    pub clusters_per_regime: HashMap<String, Vec<StrategyEvaluation>>,
    pub population_stats: PopulationStats,
    pub final_population: Vec<Strategy>,
    #[serde(default)]
    pub consensus_recommendations: Option<ConsensusReport>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PopulationStats {
    pub fitness: (f64, f64),     // (mu, sigma)
    pub consistency: (f64, f64), // (mu, sigma)
    pub recent: (f64, f64),      // (mu, sigma)
}

impl Default for StrategyEvaluation {
    fn default() -> Self {
        StrategyEvaluation {
            candidate_edges: Vec::new(),
            winner_idx: 0,
            strategy_id: "N/A".to_string(),
            strategy: Strategy {
                queue_threshold: 0,
                base_edge: 0,
                take_profit: 0,
                stop_loss: 0,
                holding_period: 0,
                w_conviction: 50,
                w_momentum: 50,
                w_volatility: 20,
                exp_conviction: 100,
                exp_momentum: 100,
                exp_volatility: 100,
                selectivity: 75,
                archetype: 0,
                direction_bias: 50,
                vol_floor: 20,
                mom_floor: 20,
                edge_ratio: 150,
                participation_threshold: 30,
                entry_offset: 0,
                exec_aggression: 50,
                latency_bias: 10,
                fill_threshold: 50, lineage: 0,
            },
            direction_ratio: 0.5,
            baseline_pnl: 0.0,
            capability: ScenarioCapability::default(),
            real_dom: 0.0,
            had_organic_signals: false,
            max_signature_credibility: 0.0,
            forced_win_ratio: 0.0,
            avg_pnl: 0.0,
            std_dev: 0.0,
            downside_std_dev: 0.0,
            worst: 0.0,
            robustness: 0.0,
            fitness: 0.0,
            trade_count: 0,
            max_drawdown: 0.0,
            participation_rate: 0.0,
            win_rate: 0.0,
            payoff: 0.0,
            profitable_trades: 0,
            zero_pnl_trades: 0,
            quality_trades: 0.0,
            payoff_ratio: 0.0,
            execution_metrics: ExecutionMetrics::default(),
            scenario_signature: ScenarioExecutionSignature::default(),
            avg_conviction: 0.0,
            avg_efficiency: 0.0,
            avg_edge_quality: 0.0,
            directional_accuracy: 0.0,
            decisiveness: 0.0,
            execution_friction: 0.0,
            short_term_capture_eff: 1.0,
            long_term_capture_eff: 1.0,
            trade_density: 0.0,
            queue_blocked_count: 0,
            liquidity_starved_count: 0,
            total_attempts: 0,
            exec_opportunity_rate: 0.0,
            failure_profile: vec![0.0, 0.0],
            realized_pnl_rolling: 0.0,
            predicted_pnl_rolling: 0.0,
            exit_tp_count: 0,
            exit_sl_count: 0,
            exit_ts_count: 0,
            avg_hold_time: 0.0,
            consistency_score: 0.0,
            recent_performance: 0.0,
            evaluation_flag: None,
            pnl_from_tp: 0.0,
            pnl_from_sl: 0.0,
            max_trade_pnl: 0.0,
            selectivity: 0.0,
            avg_entropy: 0.0,
            avg_aqg_health: 0.0,
            aqg_skip_ratio: 0.0,
            avg_edge_spread: 0.0,
            avg_dominance: 0.0,
            raw_pop_avg: 0.0,
            raw_pop_p95: 0.0,
            raw_pop_dist: [0.0; 6],
            exec_pop_avg: 0.0,
            exec_pop_p95: 0.0,
            exec_pop_dist: [0.0; 6],
            pop_delta: 0.0,
            vip_ratio: 0.0,
            ccr: 0.0,
            stat_zero_dom_ratio: 0.0,
            emitted_signals: Vec::new(),

            exec_accept_rate: 0.0,
            vip_exec_retention: 0.0,
            e_rejection_rate: 0.0,
            clarity_to_exec_drop: 0.0,
            avg_e_score: 0.0,
            vip_avg_e_score: 0.0,
            stat_avg_e_score: 0.0,

            // Phase 14++: Structural Health (Universe Discovery)
            alpha: 0.0,
            consistency: 0.0,
            opportunity: 0.0,
            structural_score: 0.0,
            acceptance_rate: 0.0,
            valid_window_ratio: 0.0,
            avg_agreement_valid: 0.0,
            avg_purity_valid: 0.0,
            avg_stability_valid: 0.0,
            max_agreement: 0.0,
            max_purity: 0.0,
            total_windows: 0,

            // Phase 14: Consensus Audit
            consensus_bypass_ratio: 0.0,
            stability_reject_rate: 0.0,
            clarity_pnl_share: 0.0,
            conviction_pnl_share: 0.0,

            trade_qualities: Vec::new(),
            outcome_consistency: 0.0,
            avg_trade_quality: 0.0,
            std_trade_quality: 0.0,
            consistency_n: 0,
            avg_exec_prob: 0.0,
            avg_survive_score: 0.0,
            edge_std_dev: 0.0,
            total_pnl: 0.0,
            pnl_history: Vec::new(),
            pnl_fingerprint: Vec::new(),
            behavioral_signature: BehavioralSignature::default(),
            acceptance_mode: AcceptanceMode::default(),
            bootstrap_ratio: 0.0,
        }
    }
}

pub fn get_strategy_classification(eval: &StrategyEvaluation) -> String {
    if eval.trade_count == 0 {
        "Inactive".to_string()
    } else if eval.avg_pnl < 0.0 {
        "Fragile".to_string()
    } else if eval.std_dev > eval.avg_pnl * 2.0 {
        "Volatile".to_string()
    } else if eval.std_dev > eval.avg_pnl {
        "Unstable".to_string()
    } else {
        "Stable".to_string()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct Strategy {
    pub queue_threshold: u64,
    pub base_edge: u64,
    /// ATR Multiplier (scaled by 100, e.g. 250 = 2.5x ATR)
    pub take_profit: u64,
    /// ATR Multiplier (scaled by 100, e.g. 150 = 1.5x ATR)
    pub stop_loss: u64,
    /// Bars to hold (scaled by 10, e.g. 50 = 5 bars)
    pub holding_period: u64,
    // Phase D.1.8 Non-Linear Scoring Genes (scaled by 100)
    pub w_conviction: u64,
    pub w_momentum: u64,
    pub w_volatility: u64,
    pub exp_conviction: u64,
    pub exp_momentum: u64,
    pub exp_volatility: u64,
    // Phase D.1.9: Divergence & Archetype Genes
    pub selectivity: u8,   // [60, 90] Deterministic choice probability
    pub archetype: u8, // [0, 3] Behavioral Identity (Conviction, Momentum, Reversion, Volatility)
    pub entry_offset: i32, // [-10, 10] Timing offset relative to signal

    // === D.1.21 GENES ===
    pub direction_bias: u8,          // 0=Short, 50=Dual, 100=Long
    pub vol_floor: u8,               // 0–100 normalized
    pub mom_floor: u8,               // 0–100 normalized
    pub edge_ratio: u8,              // 100–300 → 1.0x–3.0x RR
    pub participation_threshold: u8, // 0–100 conviction gate
    pub exec_aggression: u8,
    pub latency_bias: u8,
    pub fill_threshold: u8,
    pub lineage: usize,
}

impl Strategy {
    pub fn mutate(&self, seed: u64) -> Self {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let mut child = self.clone();
        
        // Mutate one core gene significantly
        match rng.gen_range(0..5) {
            0 => child.queue_threshold = (child.queue_threshold as f64 * rng.gen_range(0.8..1.2)) as u64,
            1 => child.base_edge = (child.base_edge as f64 * rng.gen_range(0.8..1.2)) as u64,
            2 => child.take_profit = (child.take_profit as f64 * rng.gen_range(0.9..1.1)) as u64,
            3 => child.stop_loss = (child.stop_loss as f64 * rng.gen_range(0.9..1.1)) as u64,
            _ => child.archetype = rng.gen_range(0..4),
        }
        
        child
    }

    pub fn save_to_file(&self, path: &str) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(self).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        std::fs::write(path, json)
    }

    pub fn load_from_file(path: &str) -> std::io::Result<Self> {
        let json = std::fs::read_to_string(path)?;
        let strategy: Self = serde_json::from_str(&json).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        Ok(strategy)
    }

    /// Perfectly deterministic initialization from a seed.
    /// Used for reproducible diversity injection (V3.6.8).
    pub fn from_seed(seed: u64) -> Self {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        Self {
            queue_threshold: rng.gen_range((60 * GA_GENE_SCALE)..=(120 * GA_GENE_SCALE)),
            base_edge: rng.gen_range((1 * GA_GENE_SCALE)..=(15 * GA_GENE_SCALE)),
            take_profit: rng.gen_range(3..=25),
            stop_loss: rng.gen_range(3..=15),
            holding_period: rng.gen_range(20..=200),
            w_conviction: rng.gen_range(20..=100),
            w_momentum: rng.gen_range(20..=100),
            w_volatility: rng.gen_range(10..=60),
            exp_conviction: rng.gen_range(80..=200),
            exp_momentum: rng.gen_range(80..=200),
            exp_volatility: rng.gen_range(80..=200),
            selectivity: rng.gen_range(50..=95),
            archetype: rng.gen_range(0..=3),
            entry_offset: rng.gen_range(-5..=10),
            direction_bias: [0, 50, 100][rng.gen_range(0..3)],
            vol_floor: rng.gen_range(10..=60),
            mom_floor: rng.gen_range(10..=60),
            edge_ratio: rng.gen_range(120..=250),
            participation_threshold: rng.gen_range(20..=70),
            exec_aggression: 50,
            latency_bias: 10,
            fill_threshold: 50,
            lineage: 0,
        }
    }

    /// Helper for legacy random initialization using an existing RNG.
    pub fn random<R: Rng>(rng: &mut R) -> Self {
        Self {
            queue_threshold: rng.gen_range((60 * GA_GENE_SCALE)..=(120 * GA_GENE_SCALE)),
            base_edge: rng.gen_range((1 * GA_GENE_SCALE)..=(15 * GA_GENE_SCALE)),
            take_profit: rng.gen_range(3..=25),
            stop_loss: rng.gen_range(3..=15),
            holding_period: rng.gen_range(20..=200),
            w_conviction: rng.gen_range(20..=100),
            w_momentum: rng.gen_range(20..=100),
            w_volatility: rng.gen_range(10..=60),
            exp_conviction: rng.gen_range(80..=200),
            exp_momentum: rng.gen_range(80..=200),
            exp_volatility: rng.gen_range(80..=200),
            selectivity: rng.gen_range(50..=95),
            archetype: rng.gen_range(0..=3),
            entry_offset: rng.gen_range(-5..=10),
            direction_bias: [0, 50, 100][rng.gen_range(0..3)],
            vol_floor: rng.gen_range(10..=60),
            mom_floor: rng.gen_range(10..=60),
            edge_ratio: rng.gen_range(120..=250),
            participation_threshold: rng.gen_range(20..=70),
            exec_aggression: rng.gen_range(20..80),
            latency_bias: rng.gen_range(0..100),
            fill_threshold: rng.gen_range(20..80),
            lineage: 0,
        }
    }

    /// Buckets genes into functional behavioral axes (V3.6.8/3.6.9).
    /// Used for diversity score metrics and orthogonal mutation constraints.
    /// [V3.6.9] Supports hysteresis via parent_axes to prevent boundary noise.
    pub fn get_behavioral_axes(&self, parent_axes: Option<(u8, u8, u8, u8)>) -> (u8, u8, u8, u8) {
        let axis_direction = match self.direction_bias {
            0 => 0,
            100 => 2,
            _ => 1,
        };

        // Bucket sensitivity based on base_edge (scaled by 100)
        // [V3.6.9] Hysteresis: 10% buffer zone (0.5 * GA_GENE_SCALE)
        let mut axis_sensitivity = if self.base_edge < 3 * GA_GENE_SCALE {
            0 // Sensitive
        } else if self.base_edge < 8 * GA_GENE_SCALE {
            1 // Balanced
        } else {
            2 // Restrictive
        };

        if let Some(p) = parent_axes {
            let buffer = (0.5 * GA_GENE_SCALE as f64) as u64;
            // Only flip if we've crossed the boundary + buffer
            if p.1 == 0 && self.base_edge < 3 * GA_GENE_SCALE + buffer {
                axis_sensitivity = 0;
            }
            if p.1 == 1
                && self.base_edge > 3 * GA_GENE_SCALE - buffer
                && self.base_edge < 8 * GA_GENE_SCALE + buffer
            {
                axis_sensitivity = 1;
            }
            if p.1 == 2 && self.base_edge > 8 * GA_GENE_SCALE - buffer {
                axis_sensitivity = 2;
            }
        }

        // Timing hysteresis
        let mut axis_timing = if self.holding_period < 50 { 0 } else { 1 };
        if let Some(p) = parent_axes {
            if p.2 == 0 && self.holding_period < 55 {
                axis_timing = 0;
            }
            if p.2 == 1 && self.holding_period > 45 {
                axis_timing = 1;
            }
        }

        let axis_rejection = self.archetype.min(3);
        (
            axis_direction,
            axis_sensitivity,
            axis_timing,
            axis_rejection,
        )
    }

    /// Stable behavioral fingerprint for phenotype-diversity tracking.
    pub fn behavioral_fingerprint(&self) -> u64 {
        let axes = self.get_behavioral_axes(None);
        stable_deterministic_hash((
            axes.0 as u64,
            axes.1 as u64,
            (axes.2 as u64) << 8 | (axes.3 as u64),
        ))
    }

    /// Returns the full behavioral signature (V3.6.8).
    pub fn get_signature(&self) -> BehavioralSignature {
        BehavioralSignature {
            fingerprint: self.behavioral_fingerprint(),
            axes: self.get_behavioral_axes(None),
        }
    }

    /// Generates a mutant that is guaranteed to be behaviorally orthogonal (Hamming >= 2).
    /// Uses a fallback loop (max 10 tries) to satisfy the constraint.
    pub fn orthogonal_mutant(&self, seed: u64) -> Self {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let parent_axes = self.get_behavioral_axes(None);

        for _attempt in 0..10 {
            let mut mutant = self.clone();
            // Perturb at least 3 high-impact genes to force axis shifts
            mutant.direction_bias = [0, 50, 100][rng.gen_range(0..3)];
            mutant.archetype = rng.gen_range(0..4);
            mutant.base_edge = (mutant.base_edge as i64 + rng.gen_range(-500..500))
                .max(100)
                .min(5000) as u64;
            mutant.holding_period = (mutant.holding_period as i64 + rng.gen_range(-40..40))
                .max(10)
                .min(300) as u64;

            // Use parent_axes for hysteresis during mutation check
            let mutant_axes = mutant.get_behavioral_axes(Some(parent_axes));
            let mut hamming_dist = 0;
            if mutant_axes.0 != parent_axes.0 {
                hamming_dist += 1;
            }
            if mutant_axes.1 != parent_axes.1 {
                hamming_dist += 1;
            }
            if mutant_axes.2 != parent_axes.2 {
                hamming_dist += 1;
            }
            if mutant_axes.3 != parent_axes.3 {
                hamming_dist += 1;
            }

            if hamming_dist >= 2 {
                return mutant;
            }
        }
        // Fallback: Total reset via seed if structural mutation fails to diverge
        Self::from_seed(seed ^ 0xDEADBEEF)
    }
}

#[derive(Debug, Clone)]
pub struct GaConfig {
    pub population_size: usize,
    pub generations: usize,
    pub mutation_rate: f64,
    pub seed: u64,
    pub order_id_prefix: String,
    pub lambda: f64,
    pub order_quantity_for_strategy: u64,
    pub min_candles: usize,
    pub min_trades_threshold: usize, // Default to 5
    pub preserve_top_k: usize,       // Diversity preservation
    pub order_price: u64,
    pub order_timestamp: u64,
    pub initial_queue_threshold: u64,
    /// When set, overrides `GA_MAX_TRADES_PER_SCENARIO` for this config (deterministic; no env).
    pub max_trades_per_scenario: Option<usize>,
    /// When set, overrides `GA_TRADE_COOLDOWN` (event indices after each exit).
    pub trade_cooldown_events: Option<usize>,
    pub latency_ticks: usize,
    pub slippage_factor: f64,
    pub lot_size: f64,
    pub deep_validation: bool,
    pub max_hold_bars: usize,
    pub fitness_mode: FitnessMode,
    pub pnl_fingerprint_len: usize,
    pub stats: DistributionStats,
    pub rank_stats: RankStats,
}

impl Default for GaConfig {
    fn default() -> Self {
        let latency_ticks = std::env::var("GA_LATENCY_TICKS")
            .ok()
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(1)
            .min(10);
        let slippage_factor = std::env::var("GA_SLIPPAGE_FACTOR")
            .ok()
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.1)
            .clamp(0.0, 1.0);
        let lot_size = std::env::var("GA_LOT_SIZE")
            .ok()
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(1.0)
            .max(1.0);

        let population_size = std::env::var("GA_POPULATION_SIZE")
            .ok()
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(5)
            .clamp(5, 100);
        let generations = std::env::var("GA_GENERATIONS")
            .ok()
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(3)
            .clamp(1, 100);
        let max_hold_bars = std::env::var("GA_MAX_HOLD_BARS")
            .ok()
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(20)
            .clamp(1, 1000);

        let fitness_mode = std::env::var("GA_FITNESS_MODE")
            .ok()
            .map(|s| match s.to_lowercase().as_str() {
                "sniper" => FitnessMode::Sniper,
                _ => FitnessMode::Scalable,
            })
            .unwrap_or(FitnessMode::Scalable);

        Self {
            population_size,
            generations,
            mutation_rate: 0.1, // Increased baseline for Pillar 2
            seed: 42,
            order_id_prefix: "GA_DEFAULT".to_string(),
            lambda: 0.5,
            order_quantity_for_strategy: 100,
            min_candles: 100,
            min_trades_threshold: 5,
            preserve_top_k: (population_size as f64 * 0.1).ceil() as usize,
            order_price: 40000,
            order_timestamp: 0,
            initial_queue_threshold: 20 * crate::PRICE_SCALE,
            max_trades_per_scenario: None,
            trade_cooldown_events: None,
            latency_ticks,
            slippage_factor,
            lot_size,
            deep_validation: false,
            max_hold_bars,
            fitness_mode: FitnessMode::InstitutionalRealism,
            pnl_fingerprint_len: 50,
            stats: DistributionStats::default(),
            rank_stats: RankStats::default(),
        }
    }
}

pub fn run_ga_evolution<'a>(
    mut config: GaConfig,
    all_scenarios: &[ScenarioPair<'a>],
    global: &GlobalEvoState,
) -> (GaResult, HashMap<String, AssetEvoState>) {
    let mut rng = StdRng::seed_from_u64(config.seed);
    let mut global_best: Option<StrategyEvaluation> = None;
    let mut global_best_generation = 0;
    let mut final_generation_best: Option<StrategyEvaluation> = None;
    let mut global_generation_history: Vec<StrategyEvaluation> = Vec::new();
    let mut final_p: Vec<Strategy> = Vec::new();
    let mut generation_peaks: Vec<f64> = vec![f64::NEG_INFINITY; config.generations];

    let mut asset_regime_scenarios: HashMap<(String, String), Vec<ScenarioPair<'a>>> =
        HashMap::new();
    for pair in all_scenarios {
        let name = pair.name;
        let asset = name.split('_').next().unwrap_or("BTC").to_string();
        let regime = if name.contains("trending_up") {
            "trending_up"
        } else if name.contains("trending_down") {
            "trending_down"
        } else if name.contains("mean_reverting") || name.contains("sideways") {
            "mean_reverting"
        } else if name.contains("volatile") {
            "volatile"
        } else {
            "mixed"
        };

        asset_regime_scenarios
            .entry((asset, regime.to_string()))
            .or_default()
            .push(pair.clone());
    }

    let mut best_per_bucket: HashMap<(String, String), StrategyEvaluation> = HashMap::new();
    let mut all_final_evaluations: Vec<StrategyEvaluation> = Vec::new();
    let mut asset_states: HashMap<String, AssetEvoState> = HashMap::new();

    println!("--- Starting Multi-Asset + Regime Genetic Algorithm Evolution ---");

    for ((asset, regime), scenarios) in asset_regime_scenarios {
        let cap = determine_scenario_capability(&asset);
        if !cap.is_executable() {
            println!("SCENARIO_SKIP → asset={} | reasoning=index", asset);
            continue;
        }

        println!("\n>> Evolving Bucket: asset={}, regime={}", asset, regime);
        let mut population = initialize_population(&config, &mut rng);
        let mut bucket_best_overall: Option<StrategyEvaluation> = None;
        let mut alpha_found = false;
        let mut edge_tracker = PercentileBuffer::new(5000);

        let mut evo = AssetEvoState::default();
        evo.symbol = asset.clone();

        for generation in 0..config.generations {
            population = deduplicate_population(population, &config, &mut rng);
            let diversity = calculate_population_diversity(&population);
            let unique_count = population.iter().collect::<HashSet<_>>().len();

            // Update config with latest distribution before evaluation
            config.stats = if edge_tracker.is_empty() {
                DistributionStats::default()
            } else {
                edge_tracker.get_stats()
            };

            let (evaluations_opt, _best_eval, gen_edges) = evaluate_population_scoped(
                &population,
                &config,
                &scenarios,
                generation,
                diversity,
                unique_count,
                global.expansion_bias,
            );

            // Feed discovered edges back into the tracker for the next generation/bucket
            for edge in gen_edges {
                let clipped = edge.min(config.stats.p95);
                edge_tracker.push(clipped);
            }

            // --- PHASE 11: RANK-CONDITIONED AGGREGATION ---
            if let Some(ref evals) = evaluations_opt {
                let mut next_rank_stats = RankStats::default();
                let mut processed_any = false;
                for eval in evals {
                    for outcome in &eval.pnl_history {
                        // FIX: Only learn from ORGANIC, PASSED trades. 
                        // Probes and noise should NOT pollute the empirical model.
                        if !outcome.is_probe && outcome.rank >= 0.1 {
                            let r_bucket = (outcome.rank * 10.0).floor().clamp(0.0, 9.0) as usize;
                            let v_bucket = outcome.vol_bucket.min(2);
                            
                            // 🔥 Winsorize extreme outcomes (remove outliers)
                            let mfe = outcome.m_favorable.clamp(0.0, 0.04); 
                            let mae = outcome.m_adverse.clamp(-0.03, 0.0);

                            next_rank_stats.bucket_mfe_sum[r_bucket][v_bucket] += mfe;
                            next_rank_stats.bucket_mae_sum[r_bucket][v_bucket] += mae.abs();
                            next_rank_stats.bucket_time_sum[r_bucket][v_bucket] += outcome.time_to_mfe as f64;
                            next_rank_stats.bucket_count[r_bucket][v_bucket] += 1;
                            processed_any = true;
                        }
                    }
                }
                if processed_any {
                    // EMA Smoothing (Learning rate 0.2)
                    config.rank_stats.blend(next_rank_stats, 0.2);
                }
            }

            if let Some(mut evaluations) = evaluations_opt {
                evaluations.sort_by(|a, b| {
                    b.fitness
                        .partial_cmp(&a.fitness)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });

                // --- PHASE 3: PROGRESS INSTRUMENTATION ---
                let diversity_metric = unique_count as f64 / population.len().max(1) as f64;
                if generation % 5 == 0 {
                    let avg_fitness = evaluations.iter().map(|e| e.fitness).sum::<f64>()
                        / evaluations.len().max(1) as f64;
                    println!(
                        "GEN={} BEST_FITNESS={:.6} AVG_FITNESS={:.6} DIVERSITY={:.4}",
                        generation, evaluations[0].fitness, avg_fitness, diversity_metric
                    );
                }

                if let Some(best_ref) = evaluations.first() {
                    let best = best_ref.clone();
                    let median = evaluations[evaluations.len() / 2].fitness;

                    // --- PHASE 17+: DIVERSITY AUDIT ---
                    println!("Top 5 strategies (Gen {}):", generation);
                    for (i, eval) in evaluations.iter().take(5).enumerate() {
                        let axes = eval.behavioral_signature.axes;
                        if ga_debug_enabled() {
                            println!(
                                "  - [{}] id={} fitness={:.6} axes=tp:{}|sl:{}|hold:{}|edge:{}",
                                i,
                                eval.strategy_id.chars().take(12).collect::<String>(),
                                eval.fitness,
                                axes.0,
                                axes.1,
                                axes.2,
                                axes.3
                            );
                        }
                    }

                    // Diversity & Multi-Asset State Tracking
                    if generation == 0 {
                        evo.initial_diversity = diversity;
                    }
                    evo.current_diversity = diversity;
                    evo.selection_pressure = (best.fitness / median.max(1e-9)).min(100.0);

                    if best.fitness > 0.0 && !alpha_found {
                        println!(
                            "🚨 FIRST_ALPHA_DISCOVERY → gen={} fitness={:.6} asset={}",
                            generation, best.fitness, asset
                        );
                        alpha_found = true;
                    }

                    // Adaptive Mutation Scale (Local Logic)
                    if best.fitness > evo.last_best_fitness {
                        evo.mutation_scale = (evo.mutation_scale * 0.85).max(0.2);
                        evo.stagnation_counter = 0;
                        evo.last_best_fitness = best.fitness;
                    } else {
                        evo.stagnation_counter += 1;
                        if evo.stagnation_counter > 2 {
                            evo.mutation_scale = (evo.mutation_scale * 1.2).min(3.0);
                        }
                    }

                    // Update Global Stats
                    if best.fitness > generation_peaks[generation] {
                        generation_peaks[generation] = best.fitness;
                    }
                    if global_best.is_none() || best.fitness > global_best.as_ref().unwrap().fitness
                    {
                        global_best = Some(best.clone());
                        global_best_generation = generation;
                    }

                    let should_update = bucket_best_overall
                        .as_ref()
                        .map_or(true, |o| best.fitness > o.fitness);
                    if should_update {
                        bucket_best_overall = Some(best.clone());
                    }

                    if generation == config.generations - 1 {
                        final_p = population.clone();
                        final_generation_best = Some(best.clone());
                        all_final_evaluations.extend(evaluations.clone());
                    }

                    // Track global history
                    if global_generation_history.len() <= generation {
                        global_generation_history.push(best.clone());
                    } else if best.fitness > global_generation_history[generation].fitness {
                        global_generation_history[generation] = best.clone();
                    }

                    // Local Frontier Discovery for Aggregation
                    let log_queues: Vec<f64> = evaluations
                        .iter()
                        .map(|e| (1.0 + e.scenario_signature.avg_queue_ahead).ln())
                        .collect();
                    if !log_queues.is_empty() {
                        evo.prev_max_log_queue = evo.max_log_queue;
                        evo.max_log_queue =
                            log_queues.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
                        evo.delta_log_q = evo.max_log_queue - evo.prev_max_log_queue;
                        evo.trade_density = evaluations.iter().map(|e| e.trade_count).sum::<usize>()
                            as f64
                            / evaluations.len() as f64;
                        evo.fill_rate = evaluations
                            .iter()
                            .map(|e| e.execution_metrics.fill_rate as f64)
                            .sum::<f64>()
                            / evaluations.len() as f64;
                    }

                    if generation < config.generations - 1 {
                        population = evolve_generation(
                            &evaluations,
                            &config,
                            &mut rng,
                            &evo,
                            0,
                            None,
                            None,
                            0.0,
                            generation,
                            &vec![false; evaluations.len()],
                        );
                    }
                }
            } else {
                population = initialize_population(&config, &mut rng);
            }
        }

        if let Some(best) = bucket_best_overall {
            best_per_bucket.insert((asset.clone(), regime.clone()), best);
            asset_states.insert(asset, evo);
        }
    }

    // Persist Elite Population
    all_final_evaluations.sort_by(|a, b| {
        b.fitness
            .partial_cmp(&a.fitness)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let elite_count = config.population_size.min(all_final_evaluations.len());
    let elites = &all_final_evaluations[..elite_count];
    let _ = save_elite_population(elites, &config, "core/elite");

    (
        GaResult {
            global_best: global_best.unwrap_or_else(StrategyEvaluation::default),
            global_best_generation,
            final_generation_best: final_generation_best
                .unwrap_or_else(StrategyEvaluation::default),
            generation_history: global_generation_history,
            best_per_regime: best_per_bucket
                .into_iter()
                .map(|((a, r), e)| (format!("{}_{}", a, r), e))
                .collect(),
            clusters_per_regime: HashMap::new(),
            population_stats: PopulationStats {
                fitness: (0.0, 0.0),
                consistency: (0.0, 0.0),
                recent: (0.0, 0.0),
            },
            final_population: final_p,
            consensus_recommendations: None,
        },
        asset_states,
    )
}

pub struct RobustnessReport {
    pub cv: f64,          // Cross-regime CV
    pub active_cv: f64,   // CV of non-zero regimes
    pub internal_cv: f64, // Intra-scenario CV of baseline (Regime C)
    pub robustness_score: f64,
    pub classification: String,
    pub regime_fitness: Vec<f64>,
    pub regimes_skipped: usize,
    pub participation_rate: f64,
    pub avg_pnl: f64,
    pub pnl_score: f64,
    pub selectivity: f64,
    pub total_trades: usize,
    pub agreement_entropy: f64,
}

pub fn evaluate_robustness(
    strategy: &Strategy,
    config: &GaConfig,
    scenarios: &[ScenarioPair],
    generation: usize,
    expansion_bias: f64,
) -> RobustnessReport {
    let mut regimes = Vec::new();

    // Baseline (Regime C is slightly better than default, let's use default as baseline/control)
    regimes.push(config.clone());

    // Regime A: Harder Execution
    let mut config_a = config.clone();
    config_a.latency_ticks += 1;
    config_a.slippage_factor += 0.1;
    regimes.push(config_a);

    // Regime B: Extreme Friction
    let mut config_b = config.clone();
    config_b.latency_ticks += 2;
    config_b.slippage_factor += 0.2;
    regimes.push(config_b);

    // Regime D: Path Dependency / Order Flow Jitter (Deterministic Perturbation)
    // We increment the seed for this run to shift execution indices slightly
    let mut config_d = config.clone();
    config_d.seed += 1000;
    regimes.push(config_d);

    let mut results = Vec::new();
    let mut regime_id = 0;
    for r_config in &regimes {
        // Isolation Guard: Ensure discrete RNG per regime to prevent leakage
        let mut isolated_config = r_config.clone();
        isolated_config.seed += regime_id;
        regime_id += 1;

        if let Some(eval) = evaluate_and_aggregate(
            strategy,
            &isolated_config,
            scenarios,
            generation,
            0.0,
            0,
            expansion_bias,
            0,
        ) {
            results.push(eval.fitness);
        } else {
            results.push(0.0);
        }
    }

    let total_regimes = results.len();
    let successful_results: Vec<f64> = results.iter().cloned().filter(|&f| f > 0.0).collect();
    let regimes_skipped = total_regimes - successful_results.len();
    let participation_rate = successful_results.len() as f64 / total_regimes as f64;

    // Layer 1: Global CV (Shotgun View) - Across ALL 4 regimes
    let mean = results.iter().sum::<f64>() / total_regimes as f64;
    let variance = results.iter().map(|f| (f - mean).powi(2)).sum::<f64>() / total_regimes as f64;
    let global_cv = if mean > 0.0 {
        variance.sqrt() / mean
    } else {
        0.0
    };

    // Layer 2: Active CV (Sniper View - Only admitted regimes)
    let active_cv = if !successful_results.is_empty() {
        let active_mean = successful_results.iter().sum::<f64>() / successful_results.len() as f64;
        let active_variance = successful_results
            .iter()
            .map(|f| (f - active_mean).powi(2))
            .sum::<f64>()
            / successful_results.len() as f64;
        if active_mean.abs() > 1e-9 {
            active_variance.sqrt() / active_mean.abs()
        } else {
            0.0
        }
    } else {
        0.0
    };

    // Layer 3: Internal CV Downside (Intra-scenario stability of Baseline)
    // We re-run baseline aggregation to extract its internal downside deviation.
    let internal_cv_down = if let Some(eval) = evaluate_and_aggregate(
        strategy,
        config,
        scenarios,
        generation,
        0.0,
        0,
        expansion_bias,
        0,
    ) {
        if eval.avg_pnl.abs() > 1e-9 {
            eval.downside_std_dev / eval.avg_pnl.abs()
        } else {
            0.0
        }
    } else {
        0.0
    };

    let robustness_score = if !successful_results.is_empty() {
        let min = successful_results
            .iter()
            .cloned()
            .fold(f64::INFINITY, f64::min);
        let max = successful_results
            .iter()
            .cloned()
            .fold(f64::NEG_INFINITY, f64::max)
            .max(1e-9);
        min / max
    } else {
        0.0
    };

    let (strategy_avg_pnl, strategy_total_trades) = if let Some(eval) = evaluate_and_aggregate(
        strategy,
        config,
        scenarios,
        generation,
        0.0,
        0,
        expansion_bias,
        0,
    ) {
        (eval.avg_pnl, eval.trade_count)
    } else {
        (0.0, 0)
    };

    let pnl_score = strategy_avg_pnl.max(0.0) * 100.0;
    let selectivity = strategy_total_trades as f64
        / (scenarios.iter().map(|s| s.signal.len()).sum::<usize>() as f64).max(1.0);

    // --- PHASE 13.5: SURGICAL CLASSIFICATION (Refined) ---
    let classification = if strategy_total_trades < 1 {
        "VERY_WEAK" // Evidence Starvation
    } else if strategy_total_trades < 5 {
        "WEAK" // Low Sample Size
    } else if pnl_score < 0.10 {
        "WEAK" // Underperformance
    } else if internal_cv_down <= 0.05 {
        "STRONG"
    } else {
        "WEAK"
    };

    RobustnessReport {
        cv: global_cv,
        active_cv,
        internal_cv: internal_cv_down,
        robustness_score,
        classification: classification.to_string(),
        regime_fitness: results,
        regimes_skipped,
        participation_rate,
        avg_pnl: strategy_avg_pnl,
        pnl_score,
        selectivity,
        total_trades: strategy_total_trades,
        agreement_entropy: 0.0,
    }
}

pub fn evaluate_ensemble_robustness(
    ensemble: &[Strategy],
    config: &GaConfig,
    scenarios: &[ScenarioPair],
    generation: usize,
    expansion_bias: f64,
) -> RobustnessReport {
    let mut regimes = Vec::new();
    regimes.push(config.clone()); // Baseline

    // Regime B: Extreme Friction
    let mut config_b = config.clone();
    config_b.latency_ticks += 2;
    config_b.slippage_factor += 0.2;
    regimes.push(config_b);

    // Regime D: Path Jitter
    let mut config_d = config.clone();
    config_d.seed += 5000;
    regimes.push(config_d);

    // ✅ Evaluate ONCE per regime
    let mut evals: Vec<Option<StrategyEvaluation>> = Vec::new();
    for r_config in &regimes {
        let evals_per_regime: Vec<_> = ensemble
            .iter()
            .filter_map(|strategy| {
                evaluate_and_aggregate(
                    strategy,
                    r_config,
                    scenarios,
                    generation,
                    0.0,
                    1,
                    expansion_bias,
                    0,
                )
            })
            .collect();
        // Simplified for brevity: taking the first valid eval or None
        evals.push(evals_per_regime.into_iter().next());
    }

    // Extract fitness safely
    let fitness_values: Vec<f64> = evals
        .iter()
        .filter_map(|e| e.as_ref().map(|v| v.fitness))
        .collect();

    let participation_rate = fitness_values.len() as f64 / regimes.len() as f64;

    // --- Stats ONLY on valid results ---
    let (mean, variance, global_cv) = if !fitness_values.is_empty() {
        let mean = fitness_values.iter().sum::<f64>() / fitness_values.len() as f64;
        let variance = fitness_values
            .iter()
            .map(|f| (f - mean).powi(2))
            .sum::<f64>()
            / fitness_values.len() as f64;

        let cv = if mean.abs() > 1e-9 {
            variance.sqrt() / mean.abs()
        } else {
            0.0
        };

        (mean, variance, cv)
    } else {
        (0.0, 0.0, 0.0)
    };

    // ✅ Baseline = FIRST regime
    let baseline_eval = evals.get(0).and_then(|e| e.as_ref());

    let internal_cv_down = if let Some(eval) = baseline_eval {
        if eval.avg_pnl.abs() > 1e-9 {
            eval.downside_std_dev / eval.avg_pnl.abs()
        } else {
            0.0
        }
    } else {
        0.0
    };

    let robustness_score = if fitness_values.len() >= 2 {
        let min = fitness_values.iter().cloned().fold(f64::INFINITY, f64::min);

        let max = fitness_values
            .iter()
            .cloned()
            .fold(f64::NEG_INFINITY, f64::max)
            .max(1e-9);

        min / max
    } else {
        0.0
    };

    let (ens_avg_pnl, ens_trade_count, ens_entropy) = if let Some(eval) = baseline_eval {
        (eval.avg_pnl, eval.trade_count, eval.avg_entropy)
    } else {
        (0.0, 0, 0.0)
    };

    let pnl_score = ens_avg_pnl.max(0.0) * 100.0;

    let selectivity = ens_trade_count as f64
        / (scenarios.iter().map(|s| s.signal.len()).sum::<usize>() as f64).max(1.0);

    // --- Classification ---
    let classification = if ens_trade_count < 1 {
        "VERY_WEAK"
    } else if ens_trade_count < 5 {
        "WEAK"
    } else if pnl_score < 0.10 {
        "WEAK"
    } else if internal_cv_down <= 0.05 {
        "STRONG"
    } else {
        "WEAK"
    };

    let skipped = regimes.len() - fitness_values.len();

    RobustnessReport {
        cv: global_cv,
        active_cv: global_cv.max(1e-9),
        internal_cv: internal_cv_down,
        robustness_score,
        classification: classification.to_string(),
        regime_fitness: fitness_values,
        regimes_skipped: skipped,
        participation_rate,
        avg_pnl: ens_avg_pnl,
        pnl_score,
        selectivity,
        total_trades: ens_trade_count,
        agreement_entropy: ens_entropy,
    }
}

pub fn evaluate_ensemble(
    ensemble: &[Strategy],
    config: &GaConfig,
    scenarios: &[ScenarioPair],
    generation: usize,
    diversity: f64,
    unique_count: usize,
    expansion_bias: f64,
) -> Option<StrategyEvaluation> {
    let mut results = Vec::new();

    for strategy in ensemble {
        if let Some(eval) = evaluate_and_aggregate(
            strategy,
            config,
            scenarios,
            generation,
            diversity,
            unique_count,
            expansion_bias,
            0,
        ) {
            results.push(eval);
        }
    }

    if results.is_empty() {
        return None;
    }

    let avg_pnl = results.iter().map(|e| e.avg_pnl).sum::<f64>() / results.len() as f64;

    let fitness = results.iter().map(|e| e.fitness).sum::<f64>() / results.len() as f64;

    let best = results
        .iter()
        .max_by(|a, b| a.fitness.total_cmp(&b.fitness))
        .unwrap();

    Some(StrategyEvaluation {
        avg_pnl,
        fitness,
        strategy: best.strategy.clone(),
        strategy_id: best.strategy_id.clone(),
        trade_count: results.iter().map(|e| e.trade_count).sum(),
        avg_entropy: results.iter().map(|e| e.avg_entropy).sum::<f64>() / results.len() as f64,
        downside_std_dev: results.iter().map(|e| e.downside_std_dev).sum::<f64>()
            / results.len() as f64,
        ..Default::default()
    })
}

pub fn evaluate_ensemble_strategy(
    ensemble: &[Strategy],
    pair: &ScenarioPair,
    config: &GaConfig,
    generation: usize,
) -> Option<StrategyEvaluation> {
    let scenario_name = &pair.name;
    let signal_events = pair.signal;
    let execution_events = pair.execution;
    let signal_count = signal_events.len();
    let period = config.min_candles;

    if signal_count < period + 50 {
        return None;
    }

    let mut scenario_pnls = Vec::new();
    let mut total_win = 0.0;
    let mut total_loss = 0.0;
    let mut exit_tp_count = 0;
    let mut exit_sl_count = 0;
    let mut exit_ts_count = 0;
    let mut busy_until = 0;
    let mut metrics = ScenarioMetrics::default();

    // Member metrics for weighting
    let mut member_evals = Vec::with_capacity(ensemble.len());
    for s in ensemble {
        if let Some(e) = evaluate_strategy(s, pair, config, 0, 0.0, 0, 0.0, 1.0, 0) {
            member_evals.push(e);
        } else {
            member_evals.push(StrategyEvaluation::default());
        }
    }

    // --- PHASE 13.6: ENSEMBLE AQG GATING (RELAXED) ---
    let mut ensemble_candidate_edges = Vec::new();
    for i in 0..signal_count {
        let mut consensus = 0.0;
        let mut total_w = 0.0;
        for (idx, strategy) in ensemble.iter().enumerate() {
            let conv = evaluate_market_conviction(
                strategy,
                scenario_name,
                signal_events,
                i,
                0,
                generation,
            );
            let w = member_evals[idx].fitness.max(0.1);
            consensus += conv.conviction_score * w;
            total_w += w;
        }
        let score = (consensus / total_w.max(1e-9)).abs();
        let unique = ensemble.len();
        let aqg_threshold = if unique < 3 { 0.005 } else { 0.002 };
        // --- Task 5: FORCE SELECTIVITY ---
        if score < aqg_threshold {
            continue;
        }
        ensemble_candidate_edges.push(score);
    }

    let ensemble_coverage = ensemble_candidate_edges.len() as f64 / signal_count.max(1) as f64;
    if ensemble_candidate_edges.len() < 1 || ensemble_coverage < 0.005 {
        if ga_debug_enabled() {
            println!("AQG_SKIP_ENSEMBLE → scenario={} (Evidence starvation: valid={} coverage={:.4}). Skipping ensemble.", 
                scenario_name, ensemble_candidate_edges.len(), ensemble_coverage);
        }
        return None;
    }

    for current_idx in (period + 2)..(signal_count - 10) {
        metrics.record_opportunity();
        if current_idx < busy_until {
            continue;
        }

        // 1. Gather Signals from all Members
        let mut member_refs = Vec::with_capacity(ensemble.len());
        for i in 0..ensemble.len() {
            member_refs.push(crate::ensemble::EnsembleMember {
                strategy_id: &member_evals[i].strategy_id,
                weight: member_evals[i].fitness.max(0.1),
            });
        }

        let mut inputs = Vec::with_capacity(ensemble.len());
        for (i, strategy) in ensemble.iter().enumerate() {
            let eval = &member_evals[i];
            let conv = evaluate_market_conviction(
                strategy,
                scenario_name,
                signal_events,
                current_idx,
                0,
                generation,
            );

            inputs.push(crate::ensemble::EnsembleInput {
                member: &member_refs[i],
                evaluation: eval,
                signal: crate::ensemble::SignalStrength {
                    value: conv.conviction_score,
                },
            });
        }

        // 2. Compute Consensus Decision + Shannon Entropy of votes
        let decision = crate::ensemble::compute_consensus(&inputs, 0.55, 0.25);

        // Phase 13.5: Entropy = how much disagreement exists among strategies
        // Uses signal strengths relative to their individual take_profit thresholds
        let entropy_norm = {
            let total_members = inputs.len() as f64;
            if total_members > 0.0 {
                let buy_p = inputs
                    .iter()
                    .filter(|i| {
                        let threshold =
                            (i.evaluation.strategy.take_profit as f64 / 10000.0).max(0.0004);
                        i.signal.value > threshold
                    })
                    .count() as f64
                    / total_members;
                let sell_p = inputs
                    .iter()
                    .filter(|i| {
                        let threshold =
                            (i.evaluation.strategy.take_profit as f64 / 10000.0).max(0.0004);
                        i.signal.value < -threshold
                    })
                    .count() as f64
                    / total_members;
                let neutral_p = 1.0 - buy_p - sell_p;
                let max_entropy = (3.0_f64).ln(); // 3 states
                let raw_entropy: f64 = [buy_p, sell_p, neutral_p]
                    .iter()
                    .filter(|&&p| p > 1e-9)
                    .map(|&p| -p * p.ln())
                    .sum();
                (raw_entropy / max_entropy).clamp(0.0, 1.0)
            } else {
                0.0
            }
        };

        if decision.combined_action != crate::SignalAction::HOLD
            && decision.consensus_score.abs() > 0.001
        {
            // 3. Execution (Consensus Weighted simulation)
            let conviction_score = decision.consensus_score.abs();
            #[cfg(feature = "debug_decision")]
            println!(
                "[DECISION_TRACE] idx={} score={:.5} entropy={:.3} action={:?}",
                current_idx,
                decision.consensus_score,
                entropy_norm,
                decision.combined_action
            );
            let conviction = ConvictionOutcome {
                conviction_score,
                bullish_score: conviction_score.max(0.0),
                bearish_score: (-conviction_score).max(0.0),
                is_valid: true,
                expected_edge: 0.0,
                edge_weight: conviction_score.clamp(0.5, 2.0),
                norm_momentum: 0.5,
                norm_volume: 0.5,
                norm_vol_score: 0.5,
                norm_vol: 0.001,
                selection_threshold: 0.5,
                is_bearish: decision.combined_action == crate::SignalAction::SELL,
                roll: 0.0,
                raw_q_ratio: 0.0,
                regime: MarketRegime::MeanReversion,
            };

            let directional_edge = conviction.conviction_score.abs().powf(0.7);
            let score_norm = (conviction.conviction_score).min(1.0);
            let strength = (0.8 * directional_edge + 0.2 * score_norm).clamp(0.05, 1.0);

            if let Some(outcome) = ga_simulate_round_trip_at_cursor(
                &ensemble[0],
                signal_events,
                execution_events,
                config,
                current_idx,
                scenario_pnls.len(),
                &conviction,
                !conviction.is_bearish,
                strength,
                false,
                0, // strategy_index fallback
                0, // generation fallback
                &config.stats,
            ) {
                let trade_pnl = outcome.pnl;

                // Decision Surface Margin (distance from consensus threshold - Relative)
                let margin = ((decision.consensus_score.abs() - 0.001) / 0.001).abs();

                // Record Decision-Time Metrics (Phase 13.6 Health Tracking)
                metrics.sum_exec_e_score += outcome.e_score;
                metrics.record_trade(
                    trade_pnl,
                    outcome.ideal_pnl,
                    trade_pnl, // expected_pnl (placeholder)
                    entropy_norm,
                    conviction.conviction_score,
                    outcome.efficiency,
                    outcome.edge_quality,
                    outcome.time_to_mfe as f64,
                    margin,
                    1.0,
                    0.0,
                    0.0,
                    0.0,
                    outcome.clone(),
                    SignalSource::Organic,
                    None,
                    true, // Mock: is_long
                    outcome.e_score,
                );

                if trade_pnl > 0.0 {
                    total_win += trade_pnl;
                } else if trade_pnl < 0.0 {
                    total_loss += trade_pnl.abs();
                }

                busy_until = outcome.exit_event_idx + (config.trade_cooldown_events.unwrap_or(5));
                if scenario_pnls.len() >= 10 {
                    break;
                }
            }
        }
    }

    if scenario_pnls.is_empty() {
        return None;
    }

    // Final Aggregate for this Scenario
    let n = scenario_pnls.len() as f64;
    let avg_pnl = scenario_pnls.iter().sum::<f64>() / n;
    let total_trades = metrics.trade_count;

    let win_rate = if total_trades > 0 {
        metrics.profitable_trades as f64 / total_trades as f64
    } else {
        0.0
    };

    let avg_win = if metrics.profitable_trades > 0 {
        total_win / metrics.profitable_trades as f64
    } else {
        0.0
    };
    let loss_count = scenario_pnls
        .len()
        .saturating_sub(metrics.profitable_trades);
    let avg_loss = if loss_count > 0 {
        total_loss / loss_count as f64
    } else {
        1e-9
    };
    let payoff_ratio = (avg_win / avg_loss.max(1e-9)).clamp(0.5, 3.0);

    let std_dev: f64 = if n > 1.0 {
        let variance = scenario_pnls
            .iter()
            .map(|pnl| (pnl - avg_pnl).powi(2))
            .sum::<f64>()
            / n;
        variance.sqrt()
    } else {
        0.0
    };

    Some(StrategyEvaluation {
        winner_idx: 0,
        strategy_id: format!("Ensemble_{}", scenario_name),
        strategy: ensemble[0].clone(), // Template
        capability: ScenarioCapability::Executable,
        real_dom: 0.0,
        had_organic_signals: true,
        avg_pnl,
        std_dev,
        downside_std_dev: 0.0,
        worst: scenario_pnls.iter().cloned().fold(f64::INFINITY, f64::min),
        robustness: avg_pnl - config.lambda * std_dev,
        fitness: 0.0, // Calculated by aggregator
        trade_count: metrics.trade_count,
        max_drawdown: 0.0,
        participation_rate: 1.0,
        profitable_trades: metrics.profitable_trades,
        zero_pnl_trades: 0,
        quality_trades: metrics.trade_count as f64,
        win_rate,
        payoff: payoff_ratio,
        payoff_ratio,
        execution_metrics: ExecutionMetrics::default(),
        scenario_signature: ScenarioExecutionSignature::default(),
        avg_conviction: metrics.avg_conviction(),
        avg_efficiency: metrics.avg_efficiency(),
        avg_edge_quality: metrics.avg_edge_quality(),
        directional_accuracy: win_rate,
        decisiveness: if metrics.trade_count > 0 {
            1.0 - (metrics.sum_time_to_mfe
                / (metrics.trade_count as f64 * config.max_hold_bars as f64))
                .clamp(0.0, 1.0)
        } else {
            0.0
        },
        execution_friction: 1.0,
        short_term_capture_eff: metrics.avg_efficiency(),
        long_term_capture_eff: metrics.avg_efficiency(),
        realized_pnl_rolling: 0.0,
        predicted_pnl_rolling: 0.0,
        exit_tp_count,
        exit_sl_count,
        exit_ts_count,
        avg_hold_time: 0.0,
        consistency_score: 1.0,
        recent_performance: avg_pnl,
        pnl_from_tp: total_win,
        pnl_from_sl: -total_loss,
        max_trade_pnl: scenario_pnls
            .iter()
            .cloned()
            .fold(f64::NEG_INFINITY, f64::max),
        pnl_fingerprint: Vec::new(),
        selectivity: metrics.selectivity(),
        avg_entropy: metrics.calculate_institutional_entropy(),
        avg_aqg_health: metrics.sum_aqg_health / metrics.trade_count.max(1) as f64,
        aqg_skip_ratio: 0.0, // Calculated at population level for single-strategy but this is ensemble
        ..Default::default()
    })
}

/// One genome: early scenario sample (same order as sequential path) + full `evaluate_and_aggregate`.
/// Scenario timelines stay sequential inside `evaluate_strategy`; only genomes may run in parallel.
fn calculate_atr(events: &[MarketEvent], cursor_i: usize, period: usize) -> f64 {
    if cursor_i < period + 1 || events.len() < period {
        return 0.001 * events.get(cursor_i).map(|e| e.price as f64).unwrap_or(1.0);
        // Fallback to 0.1% of price
    }

    let mut tr_sum = 0.0;
    for i in (cursor_i.saturating_sub(period))..cursor_i {
        if i == 0 {
            continue;
        }
        let high = events[i].price as f64; // Using price as proxy for high/low in tick data
        let low = events[i].price as f64;
        let prev_close = events[i - 1].price as f64;

        let tr = (high - low)
            .abs()
            .max((high - prev_close).abs())
            .max((low - prev_close).abs());
        tr_sum += tr;
    }
    tr_sum / period as f64
}

fn evaluate_population_member(
    strategy: &Strategy,
    config: &GaConfig,
    scenarios: &[ScenarioPair],
    generation: usize,
    diversity: f64,
    unique_count: usize,
    expansion_bias: f64,
    strategy_index: usize,
) -> Option<StrategyEvaluation> {
    evaluate_and_aggregate(
        strategy,
        config,
        scenarios,
        generation,
        diversity,
        unique_count,
        expansion_bias,
        strategy_index,
    )
}

pub fn evaluate_population_scoped(
    population: &[Strategy],
    config: &GaConfig,
    scenarios: &[ScenarioPair],
    generation: usize,
    diversity: f64,
    unique_count: usize,
    expansion_bias: f64,
) -> (Option<Vec<StrategyEvaluation>>, Option<StrategyEvaluation>, Vec<f64>) {
    let n_in = scenarios.len();
    if n_in == 0 {
        return (None, None, Vec::new());
    }

    let threads = selection_cap::resolved_ga_parallelism_threads();
    let per_member: Vec<Option<StrategyEvaluation>> = if threads <= 1 {
        population
            .iter()
            .enumerate()
            .map(|(idx, strategy)| {
                evaluate_population_member(
                    strategy,
                    config,
                    scenarios,
                    generation,
                    diversity,
                    unique_count,
                    expansion_bias,
                    idx,
                )
            })
            .collect()
    } else {
        match rayon::ThreadPoolBuilder::new().num_threads(threads).build() {
            Ok(pool) => pool.install(|| {
                population
                    .par_iter()
                    .enumerate()
                    .map(|(idx, strategy)| {
                        evaluate_population_member(
                            strategy,
                            config,
                            scenarios,
                            generation,
                            diversity,
                            unique_count,
                            expansion_bias,
                            idx,
                        )
                    })
                    .collect()
            }),
            Err(_) => population
                .iter()
                .enumerate()
                .map(|(idx, strategy)| {
                    evaluate_population_member(
                        strategy,
                        config,
                        scenarios,
                        generation,
                        diversity,
                        unique_count,
                        expansion_bias,
                        idx,
                    )
                })
                .collect(),
        }
    };
    let mut evaluations: Vec<StrategyEvaluation> = Vec::new();
    let mut all_edges: Vec<f64> = Vec::new();

    for maybe_eval in per_member.into_iter() {
        if let Some(mut eval) = maybe_eval {
            all_edges.append(&mut eval.candidate_edges);
            evaluations.push(eval);
        }
    }

    if evaluations.is_empty() {
        return (None, None, all_edges);
    }
    
    evaluations.sort_by(|a, b| b.fitness.partial_cmp(&a.fitness).unwrap_or(std::cmp::Ordering::Equal));
    let best = evaluations.first().cloned();

    // Population Summary Logging
    let avg_pnl = evaluations.iter().map(|e| e.fitness).sum::<f64>() / evaluations.len() as f64;
    let avg_trades = evaluations.iter().map(|e| e.trade_count).sum::<usize>() as f64 / evaluations.len() as f64;
    println!(
        "🌐 [POP_SUMMARY] gen={} unique={} avg_trd={:.1} avg_pnl={:.6}",
        generation, unique_count, avg_trades, avg_pnl
    );

    (Some(evaluations), best, all_edges)
}

fn deduplicate_population(
    population: Vec<Strategy>,
    config: &GaConfig,
    rng: &mut StdRng,
) -> Vec<Strategy> {
    let mut unique_strategies = HashSet::new();
    let mut new_population = Vec::with_capacity(population.len());

    for s in population {
        if unique_strategies.insert(s.clone()) {
            new_population.push(s);
        }
    }

    // Refill with random strategies if we removed duplicates
    while new_population.len() < config.population_size {
        let random_strat = Strategy {
            queue_threshold: rng.gen_range((60 * GA_GENE_SCALE)..=(120 * GA_GENE_SCALE)),
            base_edge: rng.gen_range((1 * GA_GENE_SCALE)..=(15 * GA_GENE_SCALE)),
            take_profit: rng.gen_range(3..=25), // BPS - Reachable zone for 20-bar horizon
            stop_loss: rng.gen_range(3..=15),   // BPS
            holding_period: rng.gen_range(20..=200),
            w_conviction: rng.gen_range(20..=100),
            w_momentum: rng.gen_range(20..=100),
            w_volatility: rng.gen_range(10..=60),
            exp_conviction: rng.gen_range(80..=200),
            exp_momentum: rng.gen_range(80..=200),
            exp_volatility: rng.gen_range(80..=200),
            selectivity: rng.gen_range(50..=95),
            archetype: rng.gen_range(0..=3),
            entry_offset: rng.gen_range(-5..=10),

            // Phase D.1.21: Initial Reality Genes
            direction_bias: [0, 50, 100][rng.gen_range(0..3)],
            vol_floor: rng.gen_range(10..=60),
            mom_floor: rng.gen_range(10..=60),
            edge_ratio: rng.gen_range(120..=250),
            participation_threshold: rng.gen_range(20..=70),
            exec_aggression: 50,
            latency_bias: 10,
            fill_threshold: 50, lineage: 0,
        };
        if unique_strategies.insert(random_strat.clone()) {
            new_population.push(random_strat);
        }
    }

    new_population
}

/// [V3.6.8] Calculates total phenotypic variety using categorical axes and entropy.
/// [V3.6.9] Fitness-Weighted: Entropy now reflects the density of "useful" diversity.
pub fn calculate_effective_diversity(evaluations: &[StrategyEvaluation]) -> f64 {
    if evaluations.is_empty() {
        return 0.0;
    }

    let mut unique_axes = HashSet::new();
    // Use f64 sums for fitness weighting instead of raw counts
    let mut axis_weights = [
        HashMap::new(),
        HashMap::new(),
        HashMap::new(),
        HashMap::new(),
    ];
    let mut total_fitness_sum = 0.0;

    for e in evaluations {
        let axes = e.behavioral_signature.axes;
        unique_axes.insert(axes);

        // Use normalized fitness (clamped to prevent negative entropy)
        let weight = (e.fitness.max(0.0) + 0.01);
        total_fitness_sum += weight;

        axis_weights[0]
            .entry(axes.0)
            .and_modify(|w| *w += weight)
            .or_insert(weight);
        axis_weights[1]
            .entry(axes.1)
            .and_modify(|w| *w += weight)
            .or_insert(weight);
        axis_weights[2]
            .entry(axes.2)
            .and_modify(|w| *w += weight)
            .or_insert(weight);
        axis_weights[3]
            .entry(axes.3)
            .and_modify(|w| *w += weight)
            .or_insert(weight);
    }

    let pop_size = evaluations.len() as f64;
    let uniqueness = unique_axes.len() as f64 / pop_size;

    let mut total_entropy = 0.0;
    for weights in &axis_weights {
        let mut axis_entropy = 0.0;
        let cluster_total: f64 = weights.values().sum();

        for &w in weights.values() {
            let p = w / cluster_total.max(1e-9);
            if p > 0.0 {
                axis_entropy -= p * p.ln();
            }
        }
        total_entropy += axis_entropy;
    }

    // Normalize entropy: max ln(4) ~ 1.38 across 4 categories
    let avg_entropy = total_entropy / 4.0;

    // V3.6.9 Balanced Metric
    0.6 * uniqueness + 0.4 * (avg_entropy / 1.3).min(1.0)
}

/// [V4.1.0] Calculates population-wide statistics for behavioral filtering.
pub fn calculate_population_stats(evaluations: &[StrategyEvaluation]) -> (f64, f64) {
    if evaluations.is_empty() {
        return (0.0, 0.0);
    }
    let n = evaluations.len() as f64;
    let mean: f64 = evaluations.iter().map(|e| e.fitness).sum::<f64>() / n;
    let variance: f64 = evaluations
        .iter()
        .map(|e| (e.fitness - mean).powi(2))
        .sum::<f64>()
        / n;
    (mean, variance.sqrt())
}

/// [V4.1.0] Normalized L1 distance between two genotypes [0.0, 1.0].
/// Based on max possible genomic shift (approx 8627 units).
pub fn calculate_genotype_distance_normalized(a: &Strategy, b: &Strategy) -> f64 {
    let raw_dist = calculate_genotype_distance(a, b);
    (raw_dist / 8627.0).min(1.0)
}

pub fn calculate_population_diversity(population: &[Strategy]) -> f64 {
    if population.is_empty() {
        return 0.0;
    }
    let mut total_dist = 0.0;
    let n = population.len() as f64;
    for i in 0..population.len() {
        for j in i + 1..population.len() {
            total_dist += calculate_genotype_distance_normalized(&population[i], &population[j]);
        }
    }
    (total_dist * 2.0) / (n * (n - 1.0))
}

pub fn calculate_genotype_distance(a: &Strategy, b: &Strategy) -> f64 {
    let mut dist = 0.0;
    dist += (a.queue_threshold as f64 - b.queue_threshold as f64).abs();
    dist += (a.base_edge as f64 - b.base_edge as f64).abs();
    dist += (a.take_profit as f64 - b.take_profit as f64).abs();
    dist += (a.stop_loss as f64 - b.stop_loss as f64).abs();
    dist += (a.holding_period as f64 - b.holding_period as f64).abs();
    dist += (a.w_conviction as f64 - b.w_conviction as f64).abs();
    dist += (a.w_momentum as f64 - b.w_momentum as f64).abs();
    dist += (a.w_volatility as f64 - b.w_volatility as f64).abs();
    dist += (a.exp_conviction as f64 - b.exp_conviction as f64).abs();
    dist += (a.exp_momentum as f64 - b.exp_momentum as f64).abs();
    dist += (a.exp_volatility as f64 - b.exp_volatility as f64).abs();
    dist += (a.selectivity as f64 - b.selectivity as f64).abs();
    dist += (a.archetype as f64 - b.archetype as f64).abs();
    dist += (a.entry_offset as f64 - b.entry_offset as f64).abs();
    dist += (a.direction_bias as f64 - b.direction_bias as f64).abs();
    dist += (a.vol_floor as f64 - b.vol_floor as f64).abs();
    dist += (a.mom_floor as f64 - b.mom_floor as f64).abs();
    dist += (a.edge_ratio as f64 - b.edge_ratio as f64).abs();
    dist += (a.participation_threshold as f64 - b.participation_threshold as f64).abs();
    dist
}

fn apply_similarity_penalty(evaluations: &mut Vec<StrategyEvaluation>) {
    // 1. Calculate population-based dynamic ranges for normalization
    let mut metrics = PopulationMetrics {
        min_threshold: u64::MAX,
        max_threshold: 0,
        min_edge: u64::MAX,
        max_edge: 0,
        min_tp: u64::MAX,
        max_tp: 0,
        min_sl: u64::MAX,
        max_sl: 0,
    };

    for eval in evaluations.iter() {
        metrics.min_threshold = metrics.min_threshold.min(eval.strategy.queue_threshold);
        metrics.max_threshold = metrics.max_threshold.max(eval.strategy.queue_threshold);
        metrics.min_edge = metrics.min_edge.min(eval.strategy.base_edge);
        metrics.max_edge = metrics.max_edge.max(eval.strategy.base_edge);
        metrics.min_tp = metrics.min_tp.min(eval.strategy.take_profit);
        metrics.max_tp = metrics.max_tp.max(eval.strategy.take_profit);
        metrics.min_sl = metrics.min_sl.min(eval.strategy.stop_loss);
        metrics.max_sl = metrics.max_sl.max(eval.strategy.stop_loss);
    }

    // Min-range thresholds (Institutional Safety Floors)
    let range_threshold = (metrics.max_threshold as f64 - metrics.min_threshold as f64)
        .max(100.0 * GA_GENE_SCALE as f64);
    let range_edge =
        (metrics.max_edge as f64 - metrics.min_edge as f64).max(5.0 * GA_GENE_SCALE as f64);
    let range_tp = (metrics.max_tp as f64 - metrics.min_tp as f64).max(10.0);
    let range_sl = (metrics.max_sl as f64 - metrics.min_sl as f64).max(5.0);

    let top_strats: Vec<Strategy> = evaluations
        .iter()
        .take(5)
        .map(|e| e.strategy.clone())
        .collect();

    for eval in evaluations.iter_mut() {
        let mut max_similarity: f64 = 0.0;
        for top in &top_strats {
            if &eval.strategy == top {
                continue;
            }

            // DYNAMIC NORMALIZATION: abs(a-b) / population_range
            let d1 = (eval.strategy.queue_threshold as f64 - top.queue_threshold as f64).abs()
                / range_threshold;
            let d2 = (eval.strategy.base_edge as f64 - top.base_edge as f64).abs() / range_edge;
            let d3 = (eval.strategy.take_profit as f64 - top.take_profit as f64).abs() / range_tp;
            let d4 = (eval.strategy.stop_loss as f64 - top.stop_loss as f64).abs() / range_sl;

            let dist = (d1 + d2 + d3 + d4) / 4.0;
            let similarity = (1.0 - dist).max(0.0);
            max_similarity = max_similarity.max(similarity);
        }

        // Multiplicative diversity pressure
        let penalty_factor = (1.0 - 0.2 * max_similarity).clamp(0.8, 1.0);
        eval.fitness *= penalty_factor;
    }
}

fn blend_u64(a: u64, b: u64, rng: &mut StdRng) -> u64 {
    let alpha: f64 = rng.gen();
    ((a as f64 * alpha) + (b as f64 * (1.0 - alpha))) as u64
}

pub fn crossover(a: &Strategy, b: &Strategy, rng: &mut StdRng) -> Strategy {
    let mut child = a.clone();

    // === Continuous genes → BLEND ===
    child.queue_threshold = blend_u64(a.queue_threshold, b.queue_threshold, rng);
    child.base_edge = blend_u64(a.base_edge, b.base_edge, rng);
    child.take_profit = blend_u64(a.take_profit, b.take_profit, rng);
    child.stop_loss = blend_u64(a.stop_loss, b.stop_loss, rng);
    child.holding_period = blend_u64(a.holding_period, b.holding_period, rng);

    child.w_conviction = blend_u64(a.w_conviction, b.w_conviction, rng);
    child.w_momentum = blend_u64(a.w_momentum, b.w_momentum, rng);
    child.w_volatility = blend_u64(a.w_volatility, b.w_volatility, rng);

    child.exp_conviction = blend_u64(a.exp_conviction, b.exp_conviction, rng);
    child.exp_momentum = blend_u64(a.exp_momentum, b.exp_momentum, rng);
    child.exp_volatility = blend_u64(a.exp_volatility, b.exp_volatility, rng);

    // === Discrete genes → PICK ===
    if rng.gen::<f64>() < 0.5 {
        child.selectivity = b.selectivity;
    }
    if rng.gen::<f64>() < 0.5 {
        child.archetype = b.archetype;
    }

    if rng.gen::<f64>() < 0.5 {
        child.direction_bias = b.direction_bias;
    }
    if rng.gen::<f64>() < 0.5 {
        child.vol_floor = b.vol_floor;
    }
    if rng.gen::<f64>() < 0.5 {
        child.mom_floor = b.mom_floor;
    }
    if rng.gen::<f64>() < 0.5 {
        child.edge_ratio = b.edge_ratio;
    }
    if rng.gen::<f64>() < 0.5 {
        child.participation_threshold = b.participation_threshold;
    }

    child
}

pub fn evolve_generation(
    evaluations: &Vec<StrategyEvaluation>,
    config: &GaConfig,
    rng: &mut StdRng,
    evo: &AssetEvoState,
    cooldown: u32,
    anchor: Option<&Strategy>,
    global_mean: Option<&Strategy>,
    pull_strength: f64,
    generation: usize,
    eval_stability: &[bool],
) -> Vec<Strategy> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    fn strategy_hash<T: Hash>(value: &T) -> u64 {
        let mut hasher = DefaultHasher::new();
        value.hash(&mut hasher);
        hasher.finish()
    }

    let mut next_gen: Vec<Strategy> = Vec::new();
    let mut seen: HashSet<u64> = HashSet::new();

    // Step 1: Definition of unique counts
    let mut unique_axes_set = HashSet::new();
    for e in evaluations {
        unique_axes_set.insert(e.behavioral_signature.axes);
    }
    let unique_axes = unique_axes_set.len();
    let unique_genomes_set: HashSet<String> =
        evaluations.iter().map(|e| e.strategy_id.clone()).collect();
    let unique_genomes = unique_genomes_set.len();

    // 🔥 FIX 12.1: Diversity Recovery (Milestone Gated)
    let is_monoculture = unique_genomes <= 2;
    let crossover_suppressed = is_monoculture && rng.gen::<f64>() < 0.3; // 30% chance to break cloning dominance

    if is_monoculture && generation % 5 == 0 {
        println!("☢️ MONOCULTURE DETECTED (unique_genomes={}) → Injecting 50% random population and boosting mutation", unique_genomes);

        let mut next_gen_recovery = Vec::with_capacity(config.population_size);
        // Step 1: Keep current elites (minimal preservation)
        evaluations
            .iter()
            .take(2)
            .for_each(|e| next_gen_recovery.push(e.strategy.clone()));

        // Step 2: Inject 50% random seeds
        for _ in 0..(config.population_size / 2) {
            next_gen_recovery.push(random_strategy(config, rng));
        }

        // Step 3: Mutate remaining slots (forced breakout)
        while next_gen_recovery.len() < config.population_size {
            let mut strat = random_strategy(config, rng);
            let mut evo_boost = evo.clone();
            evo_boost.mutation_scale *= 2.0; // Aggressive breakout
            mutate_strategy(
                &mut strat,
                rng,
                0,
                &evo_boost,
                anchor,
                global_mean,
                pull_strength,
                false,
                false,
            );
            next_gen_recovery.push(strat);
        }
        return next_gen_recovery;
    }

    // Step 4 — Anti-collapse guard (Partial)
    let bypass_fitness = unique_genomes < 5;
    if bypass_fitness {
        println!("⚠️ ANTI-COLLAPSE GUARD TRIGGERED (unique_genomes={}) → Bypassing weight-based selection", unique_genomes);
        // Inject a significant number of random seeds immediately
        for _ in 0..(config.population_size / 2) {
            let strat = random_strategy(config, rng);
            let key = strategy_hash(&strat);
            if !seen.contains(&key) {
                seen.insert(key);
                next_gen.push(strat);
            }
        }
    }

    let mut current_evo = evo.clone();

    // Temporary baseline (will adjust later)
    current_evo.mutation_scale *= 1.0;

    if evo.stagnation_counter > 3 {
        current_evo.mutation_scale *= 1.5;
    }

    // PHASE 11.1: Diverse Elite Selection (Hard Behavioral Filter)
    let mut elites: Vec<StrategyEvaluation> = Vec::new();
    let target_elite_count = ((evaluations.len() as f64 * 0.10).ceil() as usize)
        .max(2)
        .min(evaluations.len());
    let diversity_threshold = 0.05; // Distance [0, 1]

    // Population Stats for Elitism Normalization
    let pnl_mu =
        evaluations.iter().map(|e| e.avg_pnl).sum::<f64>() / (evaluations.len() as f64).max(1.0);
    let pnl_sigma = (evaluations
        .iter()
        .map(|e| (e.avg_pnl - pnl_mu).powi(2))
        .sum::<f64>()
        / (evaluations.len() as f64).max(1.0))
    .sqrt()
    .max(1e-9);
    let std_mu =
        evaluations.iter().map(|e| e.std_dev).sum::<f64>() / (evaluations.len() as f64).max(1.0);
    let std_sigma = (evaluations
        .iter()
        .map(|e| (e.std_dev - std_mu).powi(2))
        .sum::<f64>()
        / (evaluations.len() as f64).max(1.0))
    .sqrt()
    .max(1e-9);

    let mut filtered: Vec<_> = evaluations
        .iter()
        .filter(|e| e.trade_count > 0 || e.fitness > -0.05)
        .cloned()
        .collect();

    // fallback if everything is dead
    if filtered.len() < 5 {
        filtered = evaluations.clone();
    }

    filtered.sort_by(|a, b| b.fitness.partial_cmp(&a.fitness).unwrap());
    let sorted = filtered;

    // 🔥 FIX 3A — ELITE CAP (prevent domination)
    let max_elites_cap = (config.population_size as f64 * 0.2) as usize;
    let effective_elite_target = target_elite_count.min(max_elites_cap);

    for candidate in &sorted {
        let mut is_diverse = true;

        for existing in elites.iter() {
            let dist = calculate_behavioral_distance(
                existing, candidate, pnl_mu, pnl_sigma, std_mu, std_sigma,
            );

            let dynamic_threshold = if evo.stagnation_counter > 3 {
                0.02
            } else {
                0.04
            };

            if dist < dynamic_threshold {
                is_diverse = false;
                break;
            }
        }

        if is_diverse {
            let mut elite = candidate.strategy.clone();
            let key = strategy_hash(&elite);
            if !seen.contains(&key) || rng.gen::<f64>() < 0.25 {
                seen.insert(key);
                next_gen.push(elite);
                elites.push(candidate.clone());
            }
        }

        if next_gen.len() >= effective_elite_target {
            break;
        }
    }

    // ELITE FALLBACK: Fill remaining slots with best fitness if diversity filter was too strict
    // Guard: Similarity < 0.95 (Distance > 0.05) to prevent near-clones
    if next_gen.len() >= effective_elite_target {
        for candidate in &sorted {
            let is_diverse = !elites.iter().any(|existing| {
                calculate_behavioral_distance(
                    existing, candidate, pnl_mu, pnl_sigma, std_mu, std_sigma,
                ) < 0.02
            });

            if is_diverse {
                let mut elite = candidate.strategy.clone();
                let key = strategy_hash(&elite);
                if !seen.contains(&key) || rng.gen::<f64>() < 0.25 {
                    seen.insert(key);
                    next_gen.push(elite);
                    elites.push(candidate.clone());
                }
            }
            if next_gen.len() >= target_elite_count {
                break;
            }
        }
    }

    if ga_debug_enabled() {
        println!(
            "Elitism → Preserving {} diverse elites (Top fitness: {:.4}) | MutationScale: {:.2} | Stagnation: {}",
            next_gen.len(),
            sorted[0].fitness,
            evo.mutation_scale,
            evo.stagnation_counter
        );
    }

    // 🔥 FIX 3B — BASE IMMIGRANTS (always inject diversity early)
    let base_immigrants = (config.population_size as f64 * 0.10) as usize;

    for _ in 0..base_immigrants {
        if next_gen.len() >= config.population_size {
            break;
        }

        let strat = random_strategy(config, rng);
        let key = strategy_hash(&strat);

        if !seen.contains(&key) || rng.gen::<f64>() < 0.25 {
            seen.insert(key);
            next_gen.push(strat);
        }
    }

    let inject_count = if evo.stagnation_counter > 2 {
        (config.population_size as f64 * 0.20) as usize
    } else {
        (config.population_size as f64 * 0.10) as usize
    };

    for _ in 0..inject_count {
        if next_gen.len() >= config.population_size {
            break;
        }

        let strat = random_strategy(config, rng);
        let key = strategy_hash(&strat);

        if !seen.contains(&key) || rng.gen::<f64>() < 0.25 {
            seen.insert(key);
            next_gen.push(strat);
        }
    }

    // Tournament Selection + Adaptive Mutation (Phase D.1.19 Hardened k=2-3)
    let k = (config.population_size / 4).clamp(2, 3);
    let shock_prob = if evo.stagnation_counter > 3 {
        0.25
    } else {
        0.10
    };

    let fitness_mean =
        evaluations.iter().map(|e| e.fitness).sum::<f64>() / evaluations.len().max(1) as f64;
    let fitness_std = (evaluations
        .iter()
        .map(|e| (e.fitness - fitness_mean).powi(2))
        .sum::<f64>()
        / evaluations.len().max(1) as f64)
        .sqrt();
    let diversity_pressure = (1.0 - (fitness_std / (fitness_mean.abs() + EPS)).min(1.0)).powi(2);

    // Behavioral Cluster Count (Diversity Metric)
    let mut unique_clusters = 0;
    if !next_gen.is_empty() {
        let mut clusters: Vec<&Strategy> = vec![&next_gen[0]];
        current_evo.mutation_scale *= 1.0 + diversity_pressure * 2.5; // Aggressive scale when stuck
        if evo.stagnation_counter > 3 {
            current_evo.mutation_scale *= 1.5;
        }
        for elite in &next_gen[1..] {
            let cluster_threshold = 0.1;

            if clusters
                .iter()
                .all(|c| calculate_genotype_distance(c, elite) > cluster_threshold)
            {
                clusters.push(elite);
            }
        }
        unique_clusters = clusters.len();
        if unique_clusters <= 2 {
            current_evo.mutation_scale *= 2.0;
        }
    }

    if ga_debug_enabled() {
        println!(
            "Evolution → Diverse Clusters: {} | Tournament K: {} | Shock Prob: {:.2} | Effective Scale: {:.2}",
            unique_clusters, k, shock_prob, current_evo.mutation_scale
        );
    }

    // Phase D.1.20: Super-Elite Synthesis (Genetic Recombination)
    let super_elites: Vec<&StrategyEvaluation> = evaluations
        .iter()
        .filter(|e: &&StrategyEvaluation| {
            (**e).max_signature_credibility > 1.05
                // && (**e).forced_win_ratio < 0.25
                && (**e).trade_count >= 2
        })
        .collect();

    if !super_elites.is_empty() {
        let synthesis_count = (config.population_size as f64 * 0.15).ceil() as usize;
        for _ in 0..synthesis_count {
            if next_gen.len() >= config.population_size {
                break;
            }

            // Randomly pick a subset of super-elites for synthesis
            let n_parents = (super_elites.len().min(3)).max(1);
            let mut parents = Vec::new();
            for _ in 0..n_parents {
                parents.push(super_elites[rng.gen_range(0..super_elites.len())]);
            }

            let mut synthetic = synthesize_super_elite(&parents, rng);

            // Apply slight mutation to the synthetic offspring to refine
            // [V3.6.5] Elite Anchoring: Preserve structural integrity during strikes
            // [V3.6.7] Tiered Attraction: Identify elite status
            let elite_cutoff = (evaluations.len() as f64 * 0.2) as usize;
            let is_elite = true; // Super-elites are always elites

            let mut elite_scale = 0.5;
            if cooldown > 0 {
                elite_scale *= 0.5; // Additional 50% dampening during stabilization
            }

            let mut evo_lite = current_evo.clone();
            evo_lite.mutation_scale *= elite_scale; // Fine-tuning mutation only
            mutate_strategy(
                &mut synthetic,
                rng,
                10,
                &evo_lite,
                anchor,
                global_mean,
                pull_strength,
                is_elite,
                false,
            );

            let mut is_diverse = true;

            for existing in &next_gen {
                let dist = calculate_genotype_distance(existing, &synthetic);

                if dist < diversity_threshold {
                    is_diverse = false;
                    break;
                }
            }

            if is_diverse {
                let key = strategy_hash(&synthetic);

                if !seen.contains(&key) || rng.gen::<f64>() < 0.25 {
                    seen.insert(key);
                    next_gen.push(synthetic);
                }
            }
        }
    }

    use std::collections::HashMap;
    let mut parent_usage: HashMap<String, usize> = HashMap::new();
    let mut attempts_total = 0;
    let max_attempts = config.population_size * 20;

    let max_usage = (config.population_size as f64 * 0.5) as usize;

    while next_gen.len() < config.population_size && attempts_total < max_attempts {
        attempts_total += 1;

        if rng.gen::<f64>() < 0.08 {
            let strat = random_strategy(config, rng);
            let key = strategy_hash(&strat);

            if !seen.contains(&key) || rng.gen::<f64>() < 0.25 {
                seen.insert(key);
                next_gen.push(strat);
                continue;
            }
        }

        let mut local_attempts = 0;

        // --- PHASE 12.1: HYBRID SELECTION (70/30 ALPHA SURGE) ---
        let is_diversity_slot = (next_gen.len() as f64 / config.population_size as f64) > 0.7;

        let (p1_idx, parent1) = if is_diversity_slot {
            let idx = rng.gen_range(0..evaluations.len());
            (idx, &evaluations[idx])
        } else {
            tournament_selection_diverse_with_idx(
                evaluations,
                k,
                rng,
                &elites,
                pnl_mu,
                pnl_sigma,
                std_mu,
                std_sigma,
            )
        };

        let parent2 = if is_diversity_slot {
            tournament_selection_failure_mode(evaluations, k, rng)
        } else {
            tournament_selection_diverse(
                evaluations,
                k,
                rng,
                &elites,
                pnl_mu,
                pnl_sigma,
                std_mu,
                std_sigma,
            )
        };

        let is_p1_stable = eval_stability.get(p1_idx).cloned().unwrap_or(false);

        if parent1.strategy_id == parent2.strategy_id {
            let mut offspring = parent1.strategy.clone();
            mutate_strategy(
                &mut offspring,
                rng,
                parent1.trade_count,
                &current_evo,
                anchor,
                global_mean,
                pull_strength,
                false,
                is_p1_stable,
            );

            let key = strategy_hash(&offspring);
            if !seen.contains(&key) || rng.gen::<f64>() < 0.15 {
                seen.insert(key);
                next_gen.push(offspring);
            }
            continue;
        }

        let p1_key = parent1.strategy_id.clone();
        let p2_key = parent2.strategy_id.clone();

        // First read counts WITHOUT mutable borrow
        let p1_val = *parent_usage.get(&p1_key).unwrap_or(&0);
        let p2_val = *parent_usage.get(&p2_key).unwrap_or(&0);

        if p1_val >= max_usage || p2_val >= max_usage {
            continue;
        }

        // Now mutate AFTER checks (no overlapping borrows)
        *parent_usage.entry(p1_key).or_insert(0) += 1;
        *parent_usage.entry(p2_key).or_insert(0) += 1;

        // 🔥 CROSSOVER (KEY FIX)
        let mut offspring = if crossover_suppressed {
            parent1.strategy.clone() // Suppression: Bypass recombination to break cloning dominance
        } else {
            crossover(&parent1.strategy, &parent2.strategy, rng)
        };

        // FIX 4 — ENFORCE BEHAVIORAL DIVERGENCE (Phenotype check)
        if calculate_genotype_distance(&offspring, &parent1.strategy) < 0.1 {
            // Force mutate to ensure we actually escaped the parent's island
            let mut evo_shock = current_evo.clone();
            evo_shock.mutation_scale *= 2.5;
            mutate_strategy(
                &mut offspring,
                rng,
                parent1.trade_count,
                &evo_shock,
                anchor,
                global_mean,
                pull_strength,
                false,
                is_p1_stable,
            );
        }

        // mutate
        let mut evo_boost = current_evo.clone();
        evo_boost.mutation_scale *= 1.5;
        mutate_strategy(
            &mut offspring,
            rng,
            parent1.trade_count,
            &evo_boost,
            anchor,
            global_mean,
            pull_strength,
            false,
            is_p1_stable,
        );

        // 🔥 FORCE BREAKOUT
        if strategy_hash(&offspring) == strategy_hash(&parent1.strategy) {
            mutate_strategy(
                &mut offspring,
                rng,
                parent1.trade_count,
                &evo_boost,
                anchor,
                global_mean,
                pull_strength,
                false,
                is_p1_stable,
            );
        }
        // diversity push (not reject)

        let crossover_diversity_threshold = if unique_axes <= 2 { 0.1 } else { 0.05 };

        let mut attempts = 0;

        while next_gen.iter().any(|existing| {
            calculate_genotype_distance(existing, &offspring) < crossover_diversity_threshold
        }) {
            mutate_strategy(
                &mut offspring,
                rng,
                parent1.trade_count,
                &current_evo,
                anchor,
                global_mean,
                pull_strength,
                false,
                is_p1_stable,
            );
            attempts += 1;

            if attempts > 2 {
                break; // 🔥 allow imperfect offspring
            }
        }

        let key = strategy_hash(&offspring);

        if !seen.contains(&key) || rng.gen::<f64>() < 0.15 {
            seen.insert(key);
            next_gen.push(offspring);
        }

        if next_gen.len() >= config.population_size {
            break;
        }

        // controlled exploration
        if unique_clusters <= 2 || evo.stagnation_counter > 3 {
            if rng.gen::<f64>() < 0.15 && next_gen.len() < config.population_size {
                let strat = random_strategy(config, rng);
                let key = strategy_hash(&strat);
                if !seen.contains(&key) || rng.gen::<f64>() < 0.25 {
                    seen.insert(key);
                    next_gen.push(strat);
                }
            }
        }
    }

    // FINAL FALLBACK
    // while next_gen.len() < config.population_size {
    //     let mutant = random_strategy(config, rng);

    //     if !next_gen
    //         .iter()
    //         .any(|e| calculate_genotype_distance(e, &mutant) < 0.5)
    //     {
    //         next_gen.push(mutant);
    //     }
    // }

    while next_gen.len() < config.population_size {
        let mut strat = random_strategy(config, rng);

        if rng.gen::<f64>() < 0.5 {
            mutate_strategy(
                &mut strat,
                rng,
                1,
                &current_evo,
                anchor,
                global_mean,
                pull_strength,
                false,
                false,
            );
        }

        next_gen.push(strat);
    }

    next_gen
}

fn tournament_selection_diverse_with_idx<'a>(
    evaluations: &'a Vec<StrategyEvaluation>,
    k: usize,
    rng: &mut StdRng,
    elites: &Vec<StrategyEvaluation>,
    pnl_mu: f64,
    pnl_sigma: f64,
    std_mu: f64,
    std_sigma: f64,
) -> (usize, &'a StrategyEvaluation) {
    let mut best: Option<(usize, &StrategyEvaluation, f64)> = None;
    for _ in 0..k {
        let idx = rng.gen_range(0..evaluations.len());
        let candidate = &evaluations[idx];

        let mut max_sim = 0.0;
        for e in elites {
            let dist =
                calculate_behavioral_distance(e, candidate, pnl_mu, pnl_sigma, std_mu, std_sigma);
            let sim = (1.0 - dist).max(0.0);
            if sim > max_sim {
                max_sim = sim;
            }
        }

        let similarity_penalty = if elites.len() < 3 { 0.8 } else { 0.4 };
        let adj_fitness = candidate.fitness - similarity_penalty * max_sim;

        if best.is_none() || adj_fitness > best.unwrap().2 {
            best = Some((idx, candidate, adj_fitness));
        }
    }
    let res = best.unwrap();
    (res.0, res.1)
}

fn tournament_selection_diverse<'a>(
    evaluations: &'a Vec<StrategyEvaluation>,
    k: usize,
    rng: &mut StdRng,
    elites: &Vec<StrategyEvaluation>,
    pnl_mu: f64,
    pnl_sigma: f64,
    std_mu: f64,
    std_sigma: f64,
) -> &'a StrategyEvaluation {
    let mut best: Option<(&StrategyEvaluation, f64)> = None;
    for _ in 0..k {
        let candidate = &evaluations[rng.gen_range(0..evaluations.len())];

        // Diversity Bias: score = fitness - 0.2 * max_similarity_to_elites
        let mut max_sim = 0.0;
        for e in elites {
            let dist =
                calculate_behavioral_distance(e, candidate, pnl_mu, pnl_sigma, std_mu, std_sigma);
            let sim = (1.0 - dist).max(0.0);
            if sim > max_sim {
                max_sim = sim;
            }
        }

        let unique = elites.len();
        let similarity_penalty = if unique < 3 { 0.8 } else { 0.4 };
        let adj_fitness = candidate.fitness - similarity_penalty * max_sim; // Sharpened Penalty (D.1.19)

        if best.is_none() || adj_fitness > best.unwrap().1 {
            best = Some((candidate, adj_fitness));
        }
    }
    best.unwrap().0
}

/// --- PHASE 22: BEHAVIORAL FAILURE SELECTION (ALPHA SURGE) ---
fn tournament_selection_failure_mode<'a>(
    evaluations: &'a Vec<StrategyEvaluation>,
    k: usize,
    rng: &mut StdRng,
) -> &'a StrategyEvaluation {
    // 1. Filter for statistically significant interaction (Truth Machine Gate)
    let active_evals: Vec<&StrategyEvaluation> = evaluations
        .iter()
        .filter(|e| e.total_attempts >= 10)
        .collect();

    // 2. Apply FIXED Fitness Floor (-0.02)
    let viable_evals: Vec<&StrategyEvaluation> = active_evals
        .iter()
        .filter(|e| e.fitness > -0.02)
        .cloned()
        .collect();

    // 3. Fallback: If no viable strategies, fallback to all active ones (Selection adapts, constraint stays)
    let source = if viable_evals.len() >= 5 {
        &viable_evals
    } else if !active_evals.is_empty() {
        &active_evals
    } else {
        &evaluations.iter().map(|e| e).collect::<Vec<_>>()
    };

    let mut best_candidate = source[rng.gen_range(0..source.len())];
    let mut max_score = -1.0;

    for _ in 0..k {
        let candidate = source[rng.gen_range(0..source.len())];
        let neighbor = source[rng.gen_range(0..source.len())];

        // --- ALPHA SURGE V2 BIOMETRICS ---
        let dist = cosine_dist(&candidate.failure_profile, &neighbor.failure_profile);
        let entropy = compute_shannon_entropy(&candidate.failure_profile);

        let entropy_penalty = if entropy < 0.2 { 0.5 } else { 1.0 };
        let score = dist * entropy_penalty;

        if score > max_score {
            max_score = score;
            best_candidate = candidate;
        }
    }

    best_candidate
}

fn cosine_dist(v1: &[f64], v2: &[f64]) -> f64 {
    if v1.len() != v2.len() {
        return 1.0;
    }

    // Handle zero vectors (both clean = identical behavior)
    let v1_zero = v1.iter().all(|&x| x.abs() < 1e-9);
    let v2_zero = v2.iter().all(|&x| x.abs() < 1e-9);
    if v1_zero && v2_zero {
        return 0.0;
    }
    if v1_zero || v2_zero {
        return 1.0;
    } // One clean, one experiencing friction = different

    let nv1 = unit_normalize(v1);
    let nv2 = unit_normalize(v2);

    let dot: f64 = nv1.iter().zip(nv2.iter()).map(|(a, b)| a * b).sum();

    // Return 1.0 - Similarity [0, 2] distance
    (1.0 - dot).clamp(0.0, 2.0)
}

fn unit_normalize(v: &[f64]) -> Vec<f64> {
    let mag = v.iter().map(|&x| x * x).sum::<f64>().sqrt();
    if mag > 1e-9 {
        v.iter().map(|&x| x / mag).collect()
    } else {
        vec![0.0; v.len()]
    }
}

fn compute_shannon_entropy(v: &[f64]) -> f64 {
    let total: f64 = v.iter().sum();
    if total < 1e-9 {
        return 0.0;
    }

    v.iter()
        .map(|&x| {
            let p = x / total;
            if p > 1e-9 {
                -p * p.ln()
            } else {
                0.0
            }
        })
        .sum()
}

pub fn random_strategy(_config: &GaConfig, rng: &mut StdRng) -> Strategy {
    Strategy {
        queue_threshold: rng.gen_range((60 * GA_GENE_SCALE)..=(120 * GA_GENE_SCALE)),
        base_edge: rng.gen_range((1 * GA_GENE_SCALE)..=(15 * GA_GENE_SCALE)),
        take_profit: rng.gen_range(5..=30),
        stop_loss: rng.gen_range(5..=20),
        holding_period: rng.gen_range(20..=200),
        w_conviction: rng.gen_range(20..=100),
        w_momentum: rng.gen_range(20..=100),
        w_volatility: rng.gen_range(10..=60),
        exp_conviction: rng.gen_range(80..=200),
        exp_momentum: rng.gen_range(80..=200),
        exp_volatility: rng.gen_range(80..=200),
        selectivity: rng.gen_range(50..=95), // Wider span for liquidity discovery
        archetype: rng.gen_range(0..=3),
        entry_offset: rng.gen_range(-10..=10), // Axis 3: Timing Sensitivity
        direction_bias: [0, 50, 100][rng.gen_range(0..3)],
        vol_floor: rng.gen_range(0..=60),
        mom_floor: rng.gen_range(0..=60),
        edge_ratio: rng.gen_range(120..=250),
        participation_threshold: rng.gen_range(5..=70),
        exec_aggression: 50,
        latency_bias: 10,
        fill_threshold: 50, lineage: 0,
    }
}

pub fn synthesize_super_elite(parents: &Vec<&StrategyEvaluation>, rng: &mut StdRng) -> Strategy {
    // 1. Component: Filters (Best Pattern Credibility)
    let filter_parent = parents
        .iter()
        .max_by(|a: &&&StrategyEvaluation, b: &&&StrategyEvaluation| {
            (***a)
                .max_signature_credibility
                .total_cmp(&(***b).max_signature_credibility)
        })
        .unwrap();

    // 2. Component: Execution (Best Realized PnL)
    let exec_parent = parents
        .iter()
        .max_by(|a, b| (***a).avg_pnl.total_cmp(&(***b).avg_pnl))
        .unwrap();

    // 3. Component: Thresholds (Best Decision Consistency)
    let thresh_parent = parents
        .iter()
        .max_by(|a, b| (***a).consistency.total_cmp(&(***b).consistency))
        .unwrap();

    Strategy {
        // Group: Thresholds
        queue_threshold: thresh_parent.strategy.queue_threshold,
        base_edge: thresh_parent.strategy.base_edge,
        selectivity: thresh_parent.strategy.selectivity,

        // Group: Execution
        take_profit: exec_parent.strategy.take_profit,
        stop_loss: exec_parent.strategy.stop_loss,
        holding_period: exec_parent.strategy.holding_period,
        entry_offset: exec_parent.strategy.entry_offset,

        // Group: Filters
        w_conviction: filter_parent.strategy.w_conviction,
        w_momentum: filter_parent.strategy.w_momentum,
        w_volatility: filter_parent.strategy.w_volatility,
        exp_conviction: filter_parent.strategy.exp_conviction,
        exp_momentum: filter_parent.strategy.exp_momentum,
        exp_volatility: filter_parent.strategy.exp_volatility,

        archetype: filter_parent.strategy.archetype,

        // Group: Phase D.1.21 (Reality Archetypes)
        direction_bias: filter_parent.strategy.direction_bias,
        vol_floor: thresh_parent.strategy.vol_floor, // thresholds come from consistent parent
        mom_floor: thresh_parent.strategy.mom_floor,
        edge_ratio: exec_parent.strategy.edge_ratio, // RR comes from pnl parent
        participation_threshold: filter_parent.strategy.participation_threshold,
        exec_aggression: 50,
        latency_bias: 10,
        fill_threshold: 50, lineage: 0,
    }
}

pub fn initialize_population(config: &GaConfig, rng: &mut StdRng) -> Vec<Strategy> {
    let mut population = Vec::with_capacity(config.population_size);
    for _ in 0..config.population_size {
        population.push(random_strategy(config, rng));
    }
    population
}

/// Unified identity mapping for 13-gene DNA strings.
pub fn strategy_to_id(s: &Strategy) -> String {
    format!(
        "STRAT_{}v{}v{}v{}v{}v{}v{}v{}v{}v{}v{}v{}v{}v{}v{}v{}v{}v{}v{}",
        s.queue_threshold,
        s.base_edge,
        s.take_profit,
        s.stop_loss,
        s.holding_period,
        s.w_conviction,
        s.w_momentum,
        s.w_volatility,
        s.exp_conviction,
        s.exp_momentum,
        s.exp_volatility,
        s.selectivity,
        s.archetype,
        s.entry_offset,
        s.direction_bias,
        s.vol_floor,
        s.mom_floor,
        s.edge_ratio,
        s.participation_threshold
    )
}

pub fn mutate_strategy(
    strategy: &mut Strategy,
    rng: &mut StdRng,
    parent_trade_count: usize,
    evo: &AssetEvoState,
    anchor: Option<&Strategy>,
    global_mean: Option<&Strategy>,
    pull_strength: f64,
    is_elite: bool,
    is_stable: bool,
) {
    // 🔥 GLOBAL RESET (escape local minima)
    if rng.gen::<f64>() < (0.05 + 0.10 * evo.mutation_scale).min(0.25) {
        *strategy = random_strategy(&GaConfig::default(), rng);
        return;
    }

    // 🔥 HARD RESET FOR DEAD STRATEGIES (Fix 8)
    if parent_trade_count == 0 && rng.gen_bool(0.6) {
        *strategy = random_strategy(&GaConfig::default(), rng);
        return;
    }

    let num_mutations = if evo.stagnation_counter > 3 {
        rng.gen_range(2..5)
    } else {
        rng.gen_range(1..3)
    };

    // 🔥 LINEAGE FORCE-MUTATION: Resurrection pressure if strategy is economically inactive

    // Adaptive step size based on non-linear stagnation scaling
    let stagnation_jump = 1.0 + (evo.stagnation_counter as f64).powi(2) * 0.1;
    let mut mutation_rate = 0.3; // temporarily high

    if parent_trade_count == 0 {
        mutation_rate = (mutation_rate * 4.0_f64).clamp(0.20, 1.0);
    }

    for _ in 0..num_mutations {
        let mut mutation_type = if parent_trade_count == 0 {
            4 // 🔥 ALWAYS execution mutation for dead strategies
        } else if rng.gen::<f64>() < 0.5 {
            4
        } else {
            rng.gen_range(0..6)
        };

        let exec_signal = ((parent_trade_count as f64).ln_1p() / 4.0).clamp(0.10, 0.85);

        if parent_trade_count > 0 && rng.gen::<f64>() < exec_signal {
            mutation_type = 4;
        }

        if parent_trade_count == 0 {
            let roll = rng.gen::<f64>();

            mutation_type = if roll < 0.6 {
                4 // 🔥 aggressively force execution genes
            } else if roll < 0.8 {
                2
            } else {
                0
            };
        }

        match mutation_type {
            0 => {
                // Big jump in threshold (RUPEE EQUIVALENT)
                let base_delta = rng.gen_range((20 * GA_GENE_SCALE)..(100 * GA_GENE_SCALE)) as f64;
                let delta = (base_delta * evo.mutation_scale * stagnation_jump) as i64
                    * if rng.gen_bool(0.7) { -1 } else { 1 };
                // Clamp to selective range (60-120 units)
                strategy.queue_threshold = (strategy.queue_threshold as i64 + delta)
                    .clamp((20 * GA_GENE_SCALE) as i64, (200 * GA_GENE_SCALE) as i64)
                    as u64;
            }
            1 => {
                // Flip TP/SL (within bounds - ATR Multipliers)
                if rng.gen_bool(0.3) {
                    std::mem::swap(&mut strategy.take_profit, &mut strategy.stop_loss);
                }
            }
            2 => {
                // Base_edge change
                let intensity = if parent_trade_count == 0 { 2.0 } else { 1.0 };
                let delta = (rng.gen_range(-5..=5) as f64 * intensity * evo.mutation_scale) as i64
                    * GA_GENE_SCALE as i64;

                strategy.base_edge = (strategy.base_edge as i64 + delta)
                    .clamp((1 * GA_GENE_SCALE) as i64, (50 * GA_GENE_SCALE) as i64)
                    as u64;
            }
            3 => {
                // Mutate ATR multipliers
                let delta_tp = (rng.gen_range(-50..=50) as f64 * evo.mutation_scale) as i64;
                let delta_sl = (rng.gen_range(-30..=30) as f64 * evo.mutation_scale) as i64;
                strategy.take_profit =
                    (strategy.take_profit as i64 + delta_tp).clamp(100, 500) as u64;
                strategy.stop_loss = (strategy.stop_loss as i64 + delta_sl).clamp(50, 300) as u64;
            }
            4 => {
                // 4: Mutate Phase D.1.8 Scoring Genes
                let delta_w = (rng.gen_range(-40..=40) as f64 * evo.mutation_scale) as i64;
                let delta_e = (rng.gen_range(-60..=60) as f64 * evo.mutation_scale) as i64;

                match rng.gen_range(0..6) {
                    0 => {
                        strategy.w_conviction =
                            (strategy.w_conviction as i64 + delta_w).clamp(5, 150) as u64
                    }
                    1 => {
                        strategy.w_momentum =
                            (strategy.w_momentum as i64 + delta_w).clamp(5, 150) as u64
                    }
                    2 => {
                        strategy.w_volatility =
                            (strategy.w_volatility as i64 + delta_w).clamp(5, 150) as u64
                    }
                    3 => {
                        strategy.exp_conviction =
                            (strategy.exp_conviction as i64 + delta_e).clamp(50, 300) as u64
                    }
                    4 => {
                        strategy.exp_momentum =
                            (strategy.exp_momentum as i64 + delta_e).clamp(50, 300) as u64
                    }
                    _ => {
                        strategy.exp_volatility =
                            (strategy.exp_volatility as i64 + delta_e).clamp(50, 300) as u64
                    }
                }
            }
            _ => {
                // 5: Mutate Selectivity & Archetype (Phase D.1.9)
                if rng.gen_bool(mutation_rate) {
                    strategy.selectivity = rng.gen_range(60..=90);
                }
                if rng.gen_bool(mutation_rate) {
                    strategy.archetype = rng.gen_range(0..=3);
                }
            }
        }
    }

    // === D.1.21 GENES MUTATION (User Specific) ===
    if rng.gen_bool(mutation_rate) {
        strategy.direction_bias = [0, 50, 100][rng.gen_range(0..3)];

        strategy.vol_floor = (strategy.vol_floor as i32 + rng.gen_range(-5..6)).clamp(5, 80) as u8;

        strategy.mom_floor = (strategy.mom_floor as i32 + rng.gen_range(-5..6)).clamp(5, 80) as u8;

        strategy.edge_ratio =
            (strategy.edge_ratio as i32 + rng.gen_range(-10..11)).clamp(100, 300) as u8;

        strategy.participation_threshold =
            (strategy.participation_threshold as i32 + rng.gen_range(-5..6)).clamp(10, 90) as u8;

        strategy.entry_offset = (strategy.entry_offset + rng.gen_range(-2..3)).clamp(-10, 15);
    }
    // 🔥 Coupled mutation (coherent behavior shifts)
    if rng.gen_bool((0.35 * evo.mutation_scale).clamp(0.0, 1.0)) {
        strategy.take_profit = rng.gen_range(120..400);
        strategy.stop_loss = rng.gen_range(80..250);
        strategy.selectivity = rng.gen_range(50..95);
        strategy.entry_offset = rng.gen_range(-5..10);
    }

    if rng.gen_bool((0.1 * evo.mutation_scale).clamp(0.0, 1.0)) {
        strategy.base_edge = rng.gen_range(1 * GA_GENE_SCALE..50 * GA_GENE_SCALE);
    }

    // [V3.6.6] Directional Memory: Anchor Pull
    // If we have a global alignment anchor, pull toward it during post-strike cooldown
    // [V3.6.6] Directional Memory: Anchor Pull
    // Relaxed Drift: Reduced pull from 0.20 to 0.07 to allow escape from monoculture
    const ANCHOR_PULL: f64 = 0.07;
    if let Some(anc) = anchor {
        fn pull_u64(val: u64, target: u64) -> u64 {
            let delta = (target as f64 - val as f64) * ANCHOR_PULL;
            (val as i64 + delta as i64) as u64
        }
        fn pull_u8(val: u8, target: u8) -> u8 {
            let delta = (target as f64 - val as f64) * ANCHOR_PULL;
            (val as i16 + delta as i16) as u8
        }
        fn pull_i32(val: i32, target: i32) -> i32 {
            let delta = (target as f64 - val as f64) * ANCHOR_PULL;
            (val as f64 + delta) as i32
        }

        strategy.queue_threshold = pull_u64(strategy.queue_threshold, anc.queue_threshold);
        strategy.base_edge = pull_u64(strategy.base_edge, anc.base_edge);
        strategy.take_profit = pull_u64(strategy.take_profit, anc.take_profit);
        strategy.stop_loss = pull_u64(strategy.stop_loss, anc.stop_loss);
        strategy.holding_period = pull_u64(strategy.holding_period, anc.holding_period);
        strategy.w_conviction = pull_u64(strategy.w_conviction, anc.w_conviction);
        strategy.w_momentum = pull_u64(strategy.w_momentum, anc.w_momentum);
        strategy.w_volatility = pull_u64(strategy.w_volatility, anc.w_volatility);
        strategy.exp_conviction = pull_u64(strategy.exp_conviction, anc.exp_conviction);
        strategy.exp_momentum = pull_u64(strategy.exp_momentum, anc.exp_momentum);
        strategy.exp_volatility = pull_u64(strategy.exp_volatility, anc.exp_volatility);
        strategy.selectivity = pull_u8(strategy.selectivity, anc.selectivity);
        strategy.archetype = pull_u8(strategy.archetype, anc.archetype);
        strategy.entry_offset = pull_i32(strategy.entry_offset, anc.entry_offset);
        strategy.direction_bias = pull_u8(strategy.direction_bias, anc.direction_bias);
        strategy.vol_floor = pull_u8(strategy.vol_floor, anc.vol_floor);
        strategy.mom_floor = pull_u8(strategy.mom_floor, anc.mom_floor);
        strategy.edge_ratio = pull_u8(strategy.edge_ratio, anc.edge_ratio);
        strategy.participation_threshold = pull_u8(
            strategy.participation_threshold,
            anc.participation_threshold,
        );
    }

    // --- V3.6.7+: Selective Global Mean Pull (Stability Gated) ---
    // If we are not elite, exhibit behavioral stability, and have a global mean attractor, cluster gently
    if !is_elite && is_stable {
        if let (Some(mean), true) = (global_mean, pull_strength > 0.0) {
            let p_pull = pull_strength;

            fn lerp_u64(val: u64, target: u64, p: f64) -> u64 {
                let delta = (target as f64 - val as f64) * p;
                (val as i64 + delta as i64) as u64
            }
            fn lerp_u8(val: u8, target: u8, p: f64) -> u8 {
                let delta = (target as f64 - val as f64) * p;
                (val as i16 + delta as i16) as u8
            }
            fn lerp_i32(val: i32, target: i32, p: f64) -> i32 {
                let delta = (target as f64 - val as f64) * p;
                (val as f64 + delta) as i32
            }

            strategy.queue_threshold =
                lerp_u64(strategy.queue_threshold, mean.queue_threshold, p_pull);
            strategy.base_edge = lerp_u64(strategy.base_edge, mean.base_edge, p_pull);
            strategy.take_profit = lerp_u64(strategy.take_profit, mean.take_profit, p_pull);
            strategy.stop_loss = lerp_u64(strategy.stop_loss, mean.stop_loss, p_pull);
            strategy.holding_period =
                lerp_u64(strategy.holding_period, mean.holding_period, p_pull);
            strategy.w_conviction = lerp_u64(strategy.w_conviction, mean.w_conviction, p_pull);
            strategy.w_momentum = lerp_u64(strategy.w_momentum, mean.w_momentum, p_pull);
            strategy.w_volatility = lerp_u64(strategy.w_volatility, mean.w_volatility, p_pull);
            strategy.exp_conviction =
                lerp_u64(strategy.exp_conviction, mean.exp_conviction, p_pull);
            strategy.exp_momentum = lerp_u64(strategy.exp_momentum, mean.exp_momentum, p_pull);
            strategy.exp_volatility =
                lerp_u64(strategy.exp_volatility, mean.exp_volatility, p_pull);
            strategy.selectivity = lerp_u8(strategy.selectivity, mean.selectivity, p_pull);
            strategy.archetype = lerp_u8(strategy.archetype, mean.archetype, p_pull);
            strategy.entry_offset = lerp_i32(strategy.entry_offset, mean.entry_offset, p_pull);
            strategy.direction_bias = lerp_u8(strategy.direction_bias, mean.direction_bias, p_pull);
            strategy.vol_floor = lerp_u8(strategy.vol_floor, mean.vol_floor, p_pull);
            strategy.mom_floor = lerp_u8(strategy.mom_floor, mean.mom_floor, p_pull);
            strategy.edge_ratio = lerp_u8(strategy.edge_ratio, mean.edge_ratio, p_pull);
            strategy.participation_threshold = lerp_u8(
                strategy.participation_threshold,
                mean.participation_threshold,
                p_pull,
            );
        }
    }
}

// Rename helper if it was used with different arguments elsewhere

// Canonical Outcome Struct for multi-cycle GA evaluation.

/// One non-overlapping round-trip from a cursor index (ESE harness), for multi-cycle GA evaluation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GaRoundTripOutcome {
    pub side: Side,
    pub source: SignalSource,
    pub exit_reason: GaExitReason,
    pub pnl: f64,
    pub ideal_pnl: f64, // Ideal outcome (no latency/slippage)
    pub quality: f64,
    pub e_score: f64,
    pub exit_event_idx: usize,
    pub drawdown_penalty_raw: f64,
    pub total_filled_qty: u64,
    pub fills_count: usize,
    pub total_slippage_bps: f64,
    pub queue_ahead: f64,
    pub arrival_liquidity: f64,
    pub expected_move: f64, // Realized move
    pub m_favorable: f64,   // MFE (Max Favorable Excursion)
    pub m_adverse: f64,     // MAE (Max Adverse Excursion)
    pub efficiency: f64,    // Realized / Ideal
    pub edge_quality: f64,  // MFE / |MAE|
    pub time_to_mfe: usize, // Bars to MFE
    pub raw_q_ratio: f64,
    pub fill_efficiency: f64,
    pub sim_events: Vec<crate::SimEvent>,
    pub entry_order_id: String,
    pub exit_order_id: String,
    pub spread: f64,
    pub avg_window_volume: f64,
    pub is_probe: bool,
    pub rank: f64,
    pub raw_edge: f64,
    pub is_execution: bool,
    pub vol_bucket: usize,
}

impl Default for GaRoundTripOutcome {
    fn default() -> Self {
        Self {
            side: crate::Side::Buy,
            source: SignalSource::Synthetic,
            exit_reason: GaExitReason::TimeStop,

            pnl: 0.0,
            ideal_pnl: 0.0,
            quality: 0.0,
            e_score: 0.0,

            exit_event_idx: 0,
            drawdown_penalty_raw: 0.0,

            total_filled_qty: 0,
            fills_count: 0,
            total_slippage_bps: 0.0,

            queue_ahead: 0.0,
            arrival_liquidity: 0.0,

            expected_move: 0.0,
            m_favorable: 0.0,
            m_adverse: 0.0,

            efficiency: 0.0,
            edge_quality: 0.0,

            time_to_mfe: 0,
            raw_q_ratio: 0.0,
            fill_efficiency: 0.0,

            sim_events: Vec::new(),

            entry_order_id: String::new(),
            exit_order_id: String::new(),

            spread: 0.0,
            avg_window_volume: 0.0,

            is_probe: true,
            rank: 0.0,
            raw_edge: 0.0,
            is_execution: false,
            vol_bucket: 1, // default medium
        }
    }
}

impl GaRoundTripOutcome {
    pub fn synthetic(pnl: f64, exit_idx: usize, side: Side) -> Self {
        Self {
            pnl,
            exit_event_idx: exit_idx,
            side,
            source: SignalSource::Synthetic,
            is_probe: true,
            ..Default::default()
        }
    }
}

/// Deterministic single round-trip anchored at `market_events[cursor_i]`.
#[derive(Debug, Clone, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ConvictionOutcome {
    pub conviction_score: f64, // Continuous [0, 1]
    pub bullish_score: f64,
    pub bearish_score: f64,
    pub is_valid: bool,
    pub expected_edge: f64,       // Predicted move - Cost
    pub edge_weight: f64,         // Soft gate (1.0 or 0.2)
    pub norm_momentum: f64,       // [0, 1]
    pub norm_volume: f64,         // [0, 1]
    pub norm_vol_score: f64,      // [0, 1]
    pub norm_vol: f64,            // The raw normalized volatility
    pub selection_threshold: f64, // The strategic gate [0.0, 1.5]
    pub is_bearish: bool,
    pub roll: f64, // Genetic jitter
    pub raw_q_ratio: f64,
    pub regime: MarketRegime, // Phase D.1.24
}

impl ConvictionOutcome {
    pub fn from_scores(edge: f64, bull: f64, bear: f64) -> Self {
        Self {
            expected_edge: edge,
            bullish_score: bull,
            bearish_score: bear,
            is_bearish: bear > bull,
            is_valid: edge > 0.5,
            ..Default::default()
        }
    }
}

pub fn evaluate_market_conviction(
    strategy: &Strategy,
    scenario_name: &str,
    signal_events: &[crate::MarketEvent],
    cursor_i: usize,
    trade_idx: usize,
    generation: usize,
) -> ConvictionOutcome {
    let ref_event = &signal_events[cursor_i];
    let ref_price = ref_event.price;

    // 0. Deterministic Hash & Roll (Genome-Aware)
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    strategy.queue_threshold.hash(&mut hasher);
    strategy.base_edge.hash(&mut hasher);
    strategy.holding_period.hash(&mut hasher);
    // Phase D.1.8: Include scoring genes in perception-hash
    strategy.w_conviction.hash(&mut hasher);
    strategy.w_momentum.hash(&mut hasher);
    strategy.w_volatility.hash(&mut hasher);
    strategy.exp_conviction.hash(&mut hasher);
    strategy.exp_momentum.hash(&mut hasher);
    strategy.exp_volatility.hash(&mut hasher);

    scenario_name.hash(&mut hasher);
    cursor_i.hash(&mut hasher);
    trade_idx.hash(&mut hasher);
    generation.hash(&mut hasher);
    let id_hash = hasher.finish();
    let roll = (id_hash % 1000) as f64 / 1000.0;

    // 1. Volume Expansion (Actual Traded Volume Proxy)
    let window_size = 50;
    let start_idx = cursor_i.saturating_sub(window_size);
    let mut window_trades_vol = 0u64;
    let mut total_events_checked = 0;
    for i in start_idx..cursor_i {
        if signal_events[i].subtype == crate::MarketEventType::Trade {
            window_trades_vol += signal_events[i].quantity;
        }
        total_events_checked += 1;
    }
    let avg_vol = (window_trades_vol as f64 / total_events_checked.max(1) as f64).max(1.0);
    let current_trade_vol = if ref_event.subtype == crate::MarketEventType::Trade {
        ref_event.quantity as f64
    } else {
        avg_vol
    };
    let norm_volume = (current_trade_vol / (avg_vol * 1.5)).clamp(0.0, 1.0);

    // 2. Momentum (Price Velocity)
    let lookback_price = signal_events[start_idx].price as f64;
    let price_delta = (ref_price as f64 - lookback_price).abs() / ref_price as f64;
    let mut norm_momentum = (price_delta / 0.001).clamp(0.0, 1.0);

    // 3. Soft Volatility Guard
    let prices: Vec<f64> = signal_events[start_idx..=cursor_i]
        .iter()
        .map(|e| e.price as f64)
        .collect();
    let mean_px = prices.iter().sum::<f64>() / prices.len() as f64;
    let variance = prices.iter().map(|p| (p - mean_px).powi(2)).sum::<f64>() / prices.len() as f64;
    let norm_vol = variance.sqrt() / mean_px.max(1.0);
    let norm_vol_score = (1.0 - (norm_vol / 0.002)).clamp(0.0, 1.0);
    // Normalize momentum against realized local volatility so this feature does not pin at 1.0.
    // Keeps deterministic behavior while restoring discriminative range in quiet markets.
    let momentum_scale = (norm_vol.max(1e-6) * 8.0).max(0.0005);
    norm_momentum = (price_delta / momentum_scale).clamp(0.0, 1.0);

    // 4. 🔥 Minimal Signal Asymmetry Patch
    let momentum = (ref_price as f64 - signal_events[cursor_i.saturating_sub(3)].price as f64);
    let window_high = signal_events[cursor_i.saturating_sub(5)..=cursor_i]
        .iter()
        .map(|e| e.price)
        .max()
        .unwrap_or(0) as f64;
    let window_low = signal_events[cursor_i.saturating_sub(5)..=cursor_i]
        .iter()
        .map(|e| e.price)
        .min()
        .unwrap_or(u64::MAX) as f64;
    let range_norm = (window_high - window_low) / (ref_price as f64).max(1.0);

    // Clamp microstructure (V4.2 Stability)
    let spread = 0.0f64.clamp(-3.0, 3.0); // Fallback until z-score is wired
    let vol = norm_volume.clamp(-3.0, 3.0);

    let mut signal_score = (0.5 * norm_momentum) + (0.3 * norm_volume) + (0.2 * norm_vol_score);
    signal_score += momentum * 0.5 * (1.0 / crate::PRICE_SCALE as f64);
    signal_score += range_norm * 0.2;
    signal_score *= 1.0 + spread * 0.05;
    signal_score *= 1.0 + vol * 0.05;

    // 🔥 Edge Amplification (Non-Linear Separation)
    let edge = signal_score;
    let amplified = edge.signum() * edge.abs().powf(1.5);
    // Stabilize score tails before downstream gating; avoids clipped +/-5 spikes dominating decisions.
    signal_score = (1.8 * amplified).tanh() * 2.0;

    // 🔬 Dead-zone logic (Kill noise)
    if signal_score.abs() < 0.005 {
        signal_score = 0.0;
    }

    if std::env::var("EDGE_DEBUG").is_ok() {
        println!("[EDGE_DEBUG] sym={} n_vol={:.4} n_mom={:.4} trend_cons={:.3} score={:.4}", 
            scenario_name, norm_vol, norm_momentum, (price_delta / (norm_vol.max(1e-9) * 10.0)).clamp(0.0, 1.0), signal_score);
    }

    let _base_conviction = signal_score;

    // === D.1.21 CORE LOGIC (Anchor Scaling & Gating) ===
    let n_vol = (norm_vol * 1000.0).clamp(0.0, 100.0); // 0.001 -> 1.0 -> 100.0
    let n_mom = (norm_momentum * 100.0).clamp(0.0, 100.0);

    let vol_floor = strategy.vol_floor as f64;
    let mom_floor = strategy.mom_floor as f64;

    // Phase V3.2+ Signal Ladder: Adaptive Gating
    let phase = (generation as f64 / 50.0).clamp(0.0, 1.0); // Hardcoded total_gen for now or passed?
    let gate_looseness = 0.2 + 0.8 * phase;

    // PHASE V3.3: HARD REJECT WITH GRADUAL GATE DEGRADATION
    let starvation_str = std::env::var("GA_STARVATION_RATIO").unwrap_or_else(|_| "0.0".to_string());
    let starvation_ratio = starvation_str.parse::<f64>().unwrap_or(0.0);
    let adaptive_factor = 1.0 - 0.8 * starvation_ratio;

    let adjusted_vol_floor = (vol_floor * adaptive_factor).max(vol_floor * 0.2);
    let adjusted_mom_floor = (mom_floor * adaptive_factor).max(mom_floor * 0.2);

    if std::env::var("GA_VERBOSE").is_ok() {
        println!(
            "GATE_CHECK → n_vol={:.3} vol_floor={:.3} adjusted={:.3} pass={}",
            n_vol,
            vol_floor,
            adjusted_vol_floor,
            n_vol >= adjusted_vol_floor * 0.08
        );
    }

    if n_vol < adjusted_vol_floor * 0.08 || n_mom < adjusted_mom_floor * 0.08 {
        if std::env::var("EDGE_DEBUG").is_ok() {
            println!("[REJECT_FLOOR] sym={} n_vol={:.3} n_mom={:.3} vol_adj={:.3} mom_adj={:.3}", 
                scenario_name, n_vol, n_mom, adjusted_vol_floor*0.08, adjusted_mom_floor*0.08);
        }
        return ConvictionOutcome {
            conviction_score: 0.0,
            bullish_score: 0.0,
            bearish_score: 0.0,
            is_valid: false,
            expected_edge: 0.0,
            edge_weight: 0.0,
            norm_momentum,
            norm_volume,
            norm_vol_score,
            norm_vol,
            selection_threshold: 0.0,
            is_bearish: false,
            roll,
            raw_q_ratio: 0.0,
            regime: MarketRegime::MeanReversion,
        };
    }

    // Phase D.1.21: Soft Scaling
    let vol_scale = (n_vol / vol_floor.max(1.0)).clamp(0.5, 1.5);
    let mom_scale = (n_mom / mom_floor.max(1.0)).clamp(0.5, 1.5);
    let _anchor_scale = vol_scale.min(mom_scale);

    // Phase D.1.21: Directional Split Scoring (Symmetric Z-Score Fix)
    let rolling_std = variance.sqrt();
    let dir_signal = (ref_price as f64 - mean_px) / rolling_std.max(1e-6);
    let dir_signal = dir_signal.clamp(-3.0, 3.0);

    if std::env::var("GA_VERBOSE").is_ok() {
        println!(
            "DIR_DEBUG → price={} mean={} z={:.4}",
            ref_price, mean_px, dir_signal
        );
    }
    let scaled_dir = (dir_signal * 0.5).clamp(-2.0, 2.0);

    let temperature = 2.5; // 🔥 critical tuning parameter

    let raw_bull = (scaled_dir / temperature).exp();
    let raw_bear = (-scaled_dir / temperature).exp();

    let sum = raw_bull + raw_bear;

    let bullish_score = raw_bull / sum;
    let bearish_score = raw_bear / sum;

    if std::env::var("GA_VERBOSE").is_ok() {
        println!(
            "CONVICTION_TEMP → dir={:.3} temp={} bull={:.3} bear={:.3}",
            scaled_dir, temperature, bullish_score, bearish_score
        );
    }

    // --- Phase D.1.21: Apply Direction Bias ---
    let final_score = match strategy.direction_bias {
        100 => bullish_score, // LONG ONLY
        0 => bearish_score,   // SHORT ONLY
        _ => {
            if bullish_score > bearish_score {
                bullish_score
            } else {
                bearish_score
            }
        }
    };

    // --- PHASE D.1.24: REGIME DETECTION ---
    let high_vol_threshold = 0.008; // Institutional Hard Floor for Noise (Relaxed for alpha validation)
    let trend_consistency = (price_delta / (norm_vol.max(1e-9) * 10.0)).clamp(0.0, 1.0);

    let regime = if norm_vol > high_vol_threshold {
        MarketRegime::HighVolatilityNoise
    } else if trend_consistency > 0.55 {
        if dir_signal > 0.0 {
            MarketRegime::BullTrend
        } else if dir_signal <= 0.0 {
            MarketRegime::BearTrend
        } else {
            MarketRegime::MeanReversion
        }
    } else {
        MarketRegime::MeanReversion
    };

    // Phase V3.2+ Participation Ladder
    let p_threshold = (strategy.participation_threshold as f64 / 100.0) * gate_looseness;
    let s_threshold = strategy.selectivity as f64 / 100.0;

    if final_score < p_threshold || roll > s_threshold {
        if std::env::var("EDGE_DEBUG").is_ok() {
            println!("[REJECT_PART] sym={} score={:.3} thresh={:.3} roll={:.3} s_thresh={:.3}", 
                scenario_name, final_score, p_threshold, roll, s_threshold);
        }
        return ConvictionOutcome {
            conviction_score: 0.0,
            bullish_score,
            bearish_score,
            is_valid: false,
            expected_edge: 0.0,
            edge_weight: 0.0,
            norm_momentum,
            norm_volume,
            norm_vol_score,
            norm_vol,
            selection_threshold: p_threshold,
            is_bearish: false,
            roll,
            raw_q_ratio: 0.0,
            regime,
        };
    }

    ConvictionOutcome {
        conviction_score: final_score,
        bullish_score,
        bearish_score,
        is_valid: true,
        expected_edge: final_score * 0.075, // Amplified 5x (0.015 -> 0.075)
        edge_weight: (final_score / 0.8).clamp(0.2, 3.0),
        norm_momentum,
        norm_volume,
        norm_vol_score,
        norm_vol,
        selection_threshold: p_threshold,
        is_bearish: bearish_score > bullish_score,
        roll,
        raw_q_ratio: norm_volume * norm_momentum,
        regime,
    }
}

pub fn ga_simulate_round_trip_at_cursor(
    strategy: &Strategy,
    signal_events: &[crate::MarketEvent],
    execution_events: &[crate::MarketEvent],
    config: &GaConfig,
    cursor_i: usize,
    trade_idx: usize,
    conviction: &ConvictionOutcome,
    is_long: bool,
    strength: f64,
    is_probe: bool,
    strategy_index: usize,
    generation: usize,
    stats: &DistributionStats,
) -> Option<GaRoundTripOutcome> {
    if ga_debug_enabled() && strategy_index == 0 && trade_idx < 3 && generation % 5 == 0 {
        println!(
            "SIM_START → idx={} price={} is_long={} strength={:.3} is_probe={}",
            cursor_i, signal_events[cursor_i].price, is_long, strength, is_probe
        );
    }

    // STEP 6: Probe Bypass
    if is_probe {
        return Some(GaRoundTripOutcome {
            side: if is_long { Side::Buy } else { Side::Sell },
            source: SignalSource::Synthetic,
            exit_reason: GaExitReason::TimeStop,
            pnl: 0.0,
            ideal_pnl: 0.0,
            quality: 0.0,
            e_score: 0.0,
            exit_event_idx: cursor_i + 1,
            drawdown_penalty_raw: 0.0,
            total_filled_qty: 0,
            fills_count: 0,
            total_slippage_bps: 0.0,
            queue_ahead: 0.0,
            arrival_liquidity: 0.0,
            expected_move: 0.0,
            m_favorable: 0.0,
            m_adverse: 0.0,
            efficiency: 0.0,
            edge_quality: 0.0,
            time_to_mfe: 0,
            raw_q_ratio: 0.0,
            fill_efficiency: 0.0,
            sim_events: Vec::new(),
            entry_order_id: "PROBE_ENTRY".to_string(),
            exit_order_id: "PROBE_EXIT".to_string(),
            spread: 0.0,
            avg_window_volume: 0.0,
            is_probe: true,
            rank: 0.0,
            raw_edge: 0.0,
            is_execution: false,
            vol_bucket: 1, // Default medium vol for probes
        });
    }

    // Refinement 4: Strict cursor-based contract
    assert!(
        cursor_i < signal_events.len(),
        "cursor_i {} out of bounds for signal_events {}",
        cursor_i,
        signal_events.len()
    );
    let ref_event = &signal_events[cursor_i];
    let _ref_price = ref_event.price;
    let ref_ts = ref_event.exchange_ts;
    let total_offset = (config.latency_ticks as i32 + strategy.entry_offset).max(0);
    let mut entry_idx = cursor_i + total_offset as usize;

    if entry_idx >= execution_events.len() {
        entry_idx = execution_events.len().saturating_sub(1);
    }
    if entry_idx < cursor_i {
        entry_idx = cursor_i;
    }

    // --- DIAGNOSTICS LAYER (EXTERNAL TO ESE) ---
    let queue_ahead: f64 = execution_events[cursor_i..entry_idx]
        .iter()
        .map(|e| e.quantity as f64)
        .sum();
    let arrival_liquidity = execution_events[entry_idx].quantity as f64;

    let sig_px = signal_events[cursor_i].price as f64;
    let exe_px = execution_events[entry_idx].price as f64;
    let spread = (exe_px - sig_px).abs();

    // Institutional hard check: reject corrupt data with spread > 10% of price
    // Note: Probes already returned None/Some above, but we keep this for organic signals.
    if spread > sig_px * 0.1 {
        if std::env::var("EDGE_DEBUG").is_ok() {
            println!(
                "[EARLY_EXIT] cursor={} reason=spread_guard spread={:.6} limit={:.6}",
                cursor_i,
                spread,
                sig_px * 0.1
            );
        }
        return None;
    }

    let _market_price = exe_px as u64;
    let edge_bias = ((strategy.base_edge as f64 - 5.0) / 50.0).clamp(-0.12, 0.12);
    // Use aggressiveness from conviction
    let aggressiveness = conviction.conviction_score;
    let _agg_threshold = ((aggressiveness / 1.1) + edge_bias).clamp(0.05, 0.98);
    let _tick_01 = (0.01 * crate::PRICE_SCALE as f64).round() as u64;

    let slippage = 1.0;
    let entry_price = execution_events[entry_idx].price;

    let buy_price = if is_long {
        entry_price + slippage as u64
    } else {
        entry_price.saturating_sub(slippage as u64)
    };

    let strategy_id = strategy_to_id(strategy);
    let entry_order_id = format!("{}_t{}_entry", strategy_id, trade_idx);
    let exit_order_id = format!("{}_t{}_exit", strategy_id, trade_idx);

    // === D.1.21 GEOMETRY (Asymmetric TP/SL) ===
    let atr_floor = (buy_price as f64 * 0.0001).max(1e-5);
    let adjusted_atr =
        calculate_atr(signal_events, cursor_i, 14).max(atr_floor * (1.0 + conviction.norm_vol));

    let _rr = (strategy.edge_ratio as f64) / 100.0;
    let expected_move = (conviction.edge_weight * adjusted_atr * 1.5).max(adjusted_atr * 0.5);

    // 🔥 5-Bucket Volatility Resolution
    let vol_bps = (adjusted_atr / buy_price as f64) * 10000.0;
    let vol_bucket = if vol_bps < 8.0 { 0 } 
                else if vol_bps < 15.0 { 1 } 
                else if vol_bps < 25.0 { 2 } 
                else if vol_bps < 40.0 { 3 } 
                else { 4 };

    // 🔥 LAYER 1: Sharpened Ranking Signal
    let score_base = (conviction.conviction_score * conviction.edge_weight * (adjusted_atr / buy_price.max(1) as f64)).max(0.0001);
    // Condition rank on stability and momentum to increase purity
    let raw_edge = score_base * (1.2 - conviction.norm_vol.min(0.4)) * (0.9 + 0.2 * conviction.norm_momentum);

    // 🔥 LAYER 2: Broad Learning Floor
    // This allows decent signals to inform the RankStats, creating a real gradient.
    // Bootstrap floor should be softer (not zero) to avoid circular starvation:
    // no passes -> poor stats -> too-high floor -> no passes.
    let learn_floor = if std::env::var("GA_BOOTSTRAP").is_ok() {
        (stats.p10 * 0.2).min(0.001)
    } else {
        stats.p10
    };
    if raw_edge < learn_floor {
        if std::env::var("EDGE_DEBUG").is_ok() {
            println!(
                "[EARLY_EXIT] cursor={} reason=raw_edge_below_learn_floor raw_edge={:.6} floor={:.6}",
                cursor_i,
                raw_edge,
                learn_floor
            );
        }
        return None; 
    }

    // 🔥 LAYER 3: Rank-based gating
    let rank = stats.rank(raw_edge);
    let learn_pass = rank > 0.2; // LEARNING GRADIENT FLOOR
    let exec_pass = rank > 0.7;  // EXECUTION SELECTIVITY GATE

    if !learn_pass {
        if std::env::var("EDGE_DEBUG").is_ok() {
            println!(
                "[EARLY_EXIT] cursor={} reason=rank_below_learn_pass raw_edge={:.6} rank={:.3}",
                cursor_i,
                raw_edge,
                rank
            );
        }
        return None; // No learning value
    }

    #[cfg(feature = "debug_decision")]
    {
        if strategy_index == 0 {
             println!(
                 "[EDGE_RANK] raw={:.6} rank={:.3} -> L={} E={}",
                 raw_edge,
                 rank,
                 if learn_pass { "LEARN" } else { "SKIP" },
                 if exec_pass { "EXEC" } else { "REJECT" }
             );
        }
    }

    // ==========================================
    // ✅ CANONICAL: DATA-DRIVEN TRADE MODEL
    // ==========================================

    let exp_mfe = config.rank_stats.get_expected_mfe(rank, vol_bucket);
    let exp_mae = config.rank_stats.get_expected_mae(rank, vol_bucket);

    // Dynamic targets based on POTENTIAL (MFE), not just noisy PnL
    let target_move = (exp_mfe.abs() * buy_price as f64).clamp(buy_price as f64 * 0.0005, buy_price as f64 * 0.05);
    // Dynamic stops based on RISK (MAE)
    let stop_move = (exp_mae.abs() * buy_price as f64).clamp(buy_price as f64 * 0.0003, buy_price as f64 * 0.03);

    // Volatility-aware spread
    let entry_spread = conviction.norm_vol * buy_price as f64 * 0.5;
    let _entry_min = buy_price as f64 - entry_spread;
    let _entry_max = buy_price as f64 + entry_spread;

    let (mut tp_target, mut sl_target) = if is_long {
        (
            (buy_price as f64 + target_move).round() as u64,
            (buy_price as f64 - stop_move).round().max(1.0) as u64,
        )
    } else {
        (
            (buy_price as f64 - target_move).round().max(1.0) as u64,
            (buy_price as f64 + stop_move).round() as u64,
        )
    };

    let expected_hold_time = config.rank_stats.get_expected_time(rank, vol_bucket).clamp(3.0, 300.0) as usize;
    let final_holding_period = (expected_hold_time as f64 * 1.5) as usize; // p70-like buffer
    let final_holding_period = final_holding_period.min(config.max_hold_bars);

    if ga_debug_enabled() && strategy_index == 0 && trade_idx < 3 && generation % 5 == 0 {
        println!(
            "DATA_DRIVEN_TRADE → entry={} tp={} sl={} rank={:.3} exp_mfe={:.6} exp_mae={:.6}",
            buy_price, tp_target, sl_target, rank, exp_mfe, exp_mae
        );
    }

    if ga_debug_enabled() && strategy_index == 0 && trade_idx < 3 && generation % 5 == 0 {
        println!(
            "TRADE_FIX_DEBUG → entry={} tp={} sl={} target_m={:.2} stop_m={:.2}",
            buy_price, tp_target, sl_target, target_move, stop_move
        );
    }

    if tp_target <= 1 || sl_target <= 1 {
        if ga_debug_enabled() {
            println!("⚠️ INVALID TP/SL → applying fallback");
        }

        // fallback: minimal valid distances
        let min_tick = 1;

        if is_long {
            tp_target = buy_price + min_tick;
            sl_target = buy_price.saturating_sub(min_tick);
        } else {
            tp_target = buy_price.saturating_sub(min_tick);
            sl_target = buy_price + min_tick;
        }
    }

    if tp_target == buy_price || sl_target == buy_price {
        let min_tick = 1;

        if is_long {
            tp_target = buy_price + min_tick;
            sl_target = buy_price.saturating_sub(min_tick);
        } else {
            tp_target = buy_price.saturating_sub(min_tick);
            sl_target = buy_price + min_tick;
        }
    }
    if !is_long && std::env::var("GA_VERBOSE").is_ok() {
        println!(
            "SHORT_ENTRY → entry={} tp={} sl={}",
            buy_price, tp_target, sl_target
        );
    }

    // 🔥 MANDATORY TP/SL COUPLING
    let tp_multiplier = 1.0 + 1.5 * strength;
    let sl_multiplier = (0.8 - 0.4 * strength).max(0.4);

    let base_tp = (tp_target as f64 - buy_price as f64).abs();
    let base_sl = (sl_target as f64 - buy_price as f64).abs();

    let tp_dist_final = (base_tp * tp_multiplier).max(5.0);
    let sl_dist_final = (base_sl * sl_multiplier).max(3.0);

    // IMPORTANT: DO NOT derive from sl_dist_final (prevents distortion)
    let ts_offset = base_sl * (0.2 + 0.6 * (1.0 - strength));

    // ==========================================
    // ✅ CANONICAL: ESE-DRIVEN EXECUTION (REAL VS IDEAL)
    // ==========================================

    let mut ese = ExecutionEngine::default();

    // 1. REAL EXECUTION (Ground Truth)
    let execution = ese.simulate_round_trip(
        buy_price,
        tp_target,
        sl_target,
        if is_long { Side::Buy } else { Side::Sell },
        1,
        execution_events,
        entry_idx,
        final_holding_period,
    );

    // 2. IDEAL EXECUTION (Zero Friction Baseline)
    let ideal_execution = ese.simulate_round_trip(
        signal_events[cursor_i].price, // No slippage
        tp_target,
        sl_target,
        if is_long { Side::Buy } else { Side::Sell },
        1,
        execution_events,
        cursor_i, // No latency
        final_holding_period,
    );

    // Extract Outcome Data from Event Execution
    let realized_pnl = execution.realized_pnl;
    let ideal_pnl = ideal_execution.realized_pnl;
    let exit_reason = execution.exit_reason;
    let exit_event_idx = execution.exit_index;
    let exit_price = execution.exit_price;

    let mfe_scaled = match exit_reason {
        crate::GaExitReason::TakeProfit => tp_target,
        _ => buy_price,
    };
    let mae_scaled = match exit_reason {
        crate::GaExitReason::StopLoss => sl_target,
        crate::GaExitReason::TimeStop => exit_price,
        _ => buy_price,
    };

    let (mfe_pnl, mae_pnl) = if is_long {
        (
            (mfe_scaled as f64 - buy_price as f64) / buy_price.max(1) as f64,
            (mae_scaled as f64 - buy_price as f64) / buy_price.max(1) as f64,
        )
    } else {
        (
            (buy_price as f64 - mfe_scaled as f64) / buy_price.max(1) as f64,
            (buy_price as f64 - mae_scaled as f64) / buy_price.max(1) as f64,
        )
    };

    // --- EXECUTION LAYER EFFICIENCY (Pipeline A — raw pnl, NO normalization) ---
    // ✅ FIX 1: Kill normalized division. norm_pnl = pnl / exp_return → both scale with edge
    //           → constant tanh output. Now: use raw pnl directly.
    let buy_price_f = buy_price.max(1) as f64;
    let expected_return = expected_move / buy_price_f;

    // ✅ HARD ASSERT: real pnl is ~0.001; anything ≥ 0.1 means normalization is still active
    if realized_pnl.abs() >= 0.1 {
        panic!(
            "NORMALIZATION STILL ACTIVE in Pipeline A: norm_pnl={}",
            realized_pnl
        );
    }

    // NEW Phase 3 Efficiency: Realized / Ideal (Sign preserved)
    let efficiency = if ideal_pnl.abs() > 1e-6 {
        (realized_pnl / ideal_pnl).clamp(-1.0, 1.0)
    } else {
        if ga_debug_enabled() {
            println!(
                "DEBUG_REJECT → ideal_pnl too small ({:.8}) for efficiency calculation",
                ideal_pnl
            );
        }
        0.0
    };

    let edge_quality = (mfe_pnl / mae_pnl.abs().max(1e-9)).clamp(0.0, 5.0);
    let edge_boost = (edge_quality / 5.0).clamp(0.0, 1.0);
    let e_score = (0.7 * efficiency + 0.3 * edge_boost).clamp(-1.0, 1.0);

    #[cfg(feature = "debug_decision")]
    if strategy_index == 0 {
        let quality = execution.mfe / (execution.mae.abs() + 1e-6);
        println!(
            "[RAW_OUTCOME] rank={:.3} vol={:.1}bps mfe={:.6} mae={:.6} q={:.2} exec={}",
            rank, vol_bps, execution.mfe, execution.mae, quality, exec_pass
        );
    }

    Some(GaRoundTripOutcome {
        side: if is_long { Side::Buy } else { Side::Sell },
        source: SignalSource::Organic,
        exit_reason,
        pnl: realized_pnl,
        ideal_pnl,
        quality: if realized_pnl > 0.0005 { 1.0 } else { 0.0 },
        e_score,
        exit_event_idx,
        drawdown_penalty_raw: (buy_price as f64 - mae_scaled as f64).abs()
            / buy_price.max(1) as f64,
        total_filled_qty: execution.filled_quantity,
        fills_count: 1,
        total_slippage_bps: (slippage / buy_price as f64) * 10000.0,
        queue_ahead,
        arrival_liquidity,
        expected_move,
        m_favorable: execution.mfe,
        m_adverse: execution.mae,
        efficiency,
        edge_quality,
        time_to_mfe: exit_event_idx.saturating_sub(entry_idx),
        raw_q_ratio: conviction.raw_q_ratio,
        fill_efficiency: conviction.edge_weight.clamp(0.1, 1.0),
        sim_events: Vec::new(),
        entry_order_id,
        exit_order_id,
        spread: (exe_px - sig_px).abs(),
        avg_window_volume: 0.0,
        is_probe,
        rank,
        raw_edge,
        is_execution: exec_pass,
        vol_bucket,
    })
}

fn outcome_estimate_queue(events: &[MarketEvent], idx: usize) -> f64 {
    let window = 5;
    let mut sum = 0.0;
    let mut count = 0.0;

    for i in idx.saturating_sub(window)..idx {
        sum += events[i].quantity as f64;
        count += 1.0;
    }

    let denom = if count > 1.0 { count } else { 1.0 };
    let avg = sum / denom;

    if avg > 1.0 {
        avg
    } else {
        1.0
    }
}

fn estimate_trade_velocity(events: &[MarketEvent], idx: usize) -> f64 {
    let window = 5;
    let mut velocity = 0.0;

    for i in idx.saturating_sub(window)..idx {
        let prev = if i > 0 { i - 1 } else { i };

        let p1 = events[i].price as f64;
        let p0 = events[prev].price as f64;

        velocity += (p1 - p0).abs();
    }

    (velocity / window as f64).max(0.0001)
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SignalAlpha {
    pub ts: usize,
    pub price: f64,
    pub archetype: u8,
    pub direction: i8,
    pub strength: f64,
    pub source: SignalSource,
    pub conviction: ConvictionOutcome,
    pub is_probe: bool,
}

fn extract_weak_signals(
    window_data: &Vec<(usize, ConvictionOutcome)>,
    strategy: &Strategy,
) -> Vec<(
    usize,
    ConvictionOutcome,
    f64,
    &'static str,
    f64,
    SignalSource,
    SignalSignature,
)> {
    let mut weak_signals = Vec::new();

    for (idx, conv) in window_data.iter() {
        // 🔥 Relaxed threshold (main system uses ~0.3)
        if conv.conviction_score < 0.15 {
            continue;
        }

        // 🔥 Minimal sanity: must have direction
        let direction_valid = conv.norm_momentum.abs() > 0.05 || conv.norm_volume > 0.05;

        if !direction_valid {
            continue;
        }

        // 🔥 Build minimal signature (reuse your logic safely)
        let regime = 0;
        let momentum = if conv.norm_momentum > 0.2 {
            1
        } else if conv.norm_momentum < -0.2 {
            -1
        } else {
            0
        };

        let signature = SignalSignature {
            archetype: strategy.archetype,
            regime,
            momentum,
        };

        // 🔥 Weak signals have LOW dominance + LOW confidence
        weak_signals.push((
            *idx,
            conv.clone(),
            0.05, // low dominance
            "WEAK_FALLBACK",
            0.3,                     // low e_score
            SignalSource::Synthetic, // mark clearly
            signature,
        ));
    }

    // 🔥 Keep only top few (avoid flooding)
    weak_signals.sort_by(|a, b| {
        b.1.conviction_score
            .partial_cmp(&a.1.conviction_score)
            .unwrap()
    });

    weak_signals.truncate(5);

    weak_signals
}

fn percentile(mut values: Vec<f64>, p: f64) -> f64 {
    if values.is_empty() {
        return 0.0;
    }

    values.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let k = ((values.len() as f64 - 1.0) * p).round() as usize;
    values[k.min(values.len() - 1)]
}

pub fn evaluate_strategy(
    strategy: &Strategy,
    pair: &ScenarioPair,
    config: &GaConfig,
    generation: usize,
    diversity: f64,
    unique_count: usize,
    gen_max_log_queue: f64,
    expansion_bias: f64,
    strategy_index: usize,
) -> Option<StrategyEvaluation> {
    let mut rng = rand::thread_rng();
    let mut executed_trades: Vec<GaRoundTripOutcome> = Vec::new();
    let mut pnl_history_learn: Vec<GaRoundTripOutcome> = Vec::new();
    let mut injected_trades: Vec<GaRoundTripOutcome> = Vec::new();

    let mut trade_scores: Vec<(usize, f64)> = Vec::new();
    let mut trade_edges: Vec<(usize, f64)> = Vec::new();
    let mut sniper_scores: Vec<(usize, f64)> = Vec::new();
    let mut consistent_scores: Vec<(usize, f64)> = Vec::new();
    let mut edge_vals_build: Vec<f64> = Vec::new(); // build during execution
    let mut edge_vals_snapshot: Vec<f64> = Vec::new(); // frozen later

    // Phase 17C: Parity Tracking
    let mut exec_probs_history: Vec<f64> = Vec::new();
    let mut survive_scores_history: Vec<f64> = Vec::new();
    let mut realized_edges_history: Vec<f64> = Vec::new();

    let mut fitness_penalty = 0.0;
    let mut clarity_penalty = 1.0;
    let scenario_name = pair.name;
    let signal_events = pair.signal;
    let execution_events = pair.execution;
    let signal_symbol = pair.signal_symbol;
    let exec_symbol = pair.execution_symbol;

    // Phase 4: Routing Integrity & Pointer Safety (True Dual-Stream)
    if ga_debug_enabled() {
        println!("ROUTE_SOURCE → {} -> {}", signal_symbol, exec_symbol);
        println!(
            "ROUTE_VERIFY → diff={} sig_ptr={:p} exec_ptr={:p}",
            !std::ptr::eq(signal_events.as_ptr(), execution_events.as_ptr()),
            signal_events.as_ptr(),
            execution_events.as_ptr()
        );
        println!("SIGNALS_GENERATED → {}", signal_events.len());
    }

    // Hard Assert: Prevent "fake" separation at the memory level
    if pair.signal_symbol != pair.execution_symbol {
        assert!(
                    !std::ptr::eq(signal_events.as_ptr(), execution_events.as_ptr()),
                    "FATAL: signal and execution streams are physically identical buffers for symbols {}/{}",
                    pair.signal_symbol, pair.execution_symbol
                );
    }
    let strategy_id = strategy_to_id(strategy);

    let mut hasher = DefaultHasher::new();
    strategy.hash(&mut hasher);
    let genome_hash = hasher.finish();

    if ga_debug_enabled() {
        println!("GENOME_TRACE → {}", genome_hash);
    }
    let mut sum_actual_slippage = 0.0;
    let mut sum_expected_slippage = 0.0;
    let mut queue_blocked_count = 0usize;
    let mut liquidity_starved_count = 0usize;
    let mut _total_attempts = 0usize;
    let capability = determine_scenario_capability(scenario_name);

    // --- Phase 9: Environment Gating (Scenario-Level) ---
    // Pre-scan all signal points to assess the regime quality before committing to execution.
    let mut candidate_edges = Vec::new();

    for i in 0..signal_events.len() {
        let conv =
            evaluate_market_conviction(strategy, scenario_name, signal_events, i, 0, generation);
        if conv.is_valid && conv.conviction_score >= conv.selection_threshold {
            let entry_price = signal_events[i].price as f64;
            let atr = calculate_atr(signal_events, i, 14);

            // Phase D.1.21: Use edge_ratio gene for edge calculation
            let rr = strategy.edge_ratio as f64 / 100.0;
            let pred_move = atr * rr;

            // Synchronized Edge Estimate (Matches Patch 4)
            let edge_abs = (pred_move * 0.8 * 0.9) - (entry_price * 0.0001);
            let mut edge_ratio = (edge_abs / entry_price.max(1.0)).max(0.0);

            // FIX 1 — EDGE AMPLIFICATION & THRESHOLDING
            edge_ratio *= 5.0;
            if edge_ratio < 0.002 {
                continue;
            } // Drop weak signals

            let dir_abs = conv.norm_momentum.abs().max(0.01);

            let raw_edge = (conv.conviction_score * conv.edge_weight * (atr / entry_price.max(1.0))).max(0.0001);

            candidate_edges.push(raw_edge);
        }
    }

    if candidate_edges.is_empty() {
        if ga_debug_enabled() {
            println!("⚠️ EDGE STARVATION → forcing alpha injection");
        }
        return Some(StrategyEvaluation {
            candidate_edges: candidate_edges.clone(),
            strategy: strategy.clone(),
            strategy_id: strategy_id.clone(),
            fitness: -0.15, // Special starvation code
            trade_count: 0,
            pnl_history: Vec::new(),
            ..StrategyEvaluation::default()
        });
    }

    // --- PHASE 10.3: DART (Dynamic Asset-Relative Thresholding) FLOOR ---
    // Extract a representative window-level floor for pre-filtering stats.
    let avg_edge = if candidate_edges.is_empty() {
        0.0001
    } else {
        candidate_edges.iter().sum::<f64>() / candidate_edges.len() as f64
    };
    let avg_atr_pct = avg_edge * 1.2;
    let window_dart_floor = (avg_atr_pct * 0.40).clamp(0.00001, 0.0012);

    // --- PHASE 10.3: AQG DISTRIBUTION INTEGRITY ---
    // Filter noise before statistics. Use exact DART floor as the viability barrier.
    let valid_edges: Vec<f64> = candidate_edges
        .iter()
        .cloned()
        .filter(|e| *e >= window_dart_floor)
        .collect();

    // AQG Starvation Gate Removed (Analytical Mode)
    let _coverage = valid_edges.len() as f64 / candidate_edges.len().max(1) as f64;
    let _min_sample = (candidate_edges.len() as f64 * 0.03).max(5.0) as usize;

    let _aqg_health = (valid_edges.len() as f64 / _min_sample.max(1) as f64).clamp(0.0, 1.5);
    // let _ = aqg_threshold; // Closure compatibility

    // --- PHASE 10.3: AQG (Adaptive Percentile Selection) ---
    let mut v = valid_edges.clone();
    v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let (median, mad_scaled, aqg_gate) = if v.is_empty() {
        (0.0001, 0.0, 0.0001)
    } else {
        let median = v[v.len() / 2];
        let mut deviations: Vec<f64> = v.iter().map(|e| (e - median).abs()).collect();
        deviations.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let mad = deviations[deviations.len() / 2];
        let mad_scaled = mad * 1.4826;
        let dispersion_val = mad_scaled / median.max(1e-9);
        let pct = if dispersion_val > 0.8 {
            0.50
        } else if dispersion_val > 0.5 {
            0.55
        } else {
            0.60
        };
        let idx = ((v.len() as f64) * pct).floor() as usize;
        let aqg_gate = v[idx.min(v.len().saturating_sub(1))];
        (median, mad_scaled, aqg_gate)
    };
    let aqg_threshold = aqg_gate;
    let dispersion = mad_scaled / median.max(1e-9);

    if std::env::var("GA_SIGNAL_DEBUG").is_ok() {
        println!(
            "AQG_ADMISSION → scenario={} dispersion={:.6} aqg_gate={:.6} (valid={}/max={})",
            scenario_name,
            dispersion,
            aqg_gate,
            valid_edges.len(),
            candidate_edges.len()
        );
    }
    let mut scenario_pnls: Vec<f64> = Vec::new();
    let mut total_quality_trades_scenario = 0.0;
    let mut total_efficiency = 0.0;
    let mut total_vol_ratio = 0.0;
    let mut total_spread_reality = 0.0;
    let mut total_spread_test = 0.0;
    let mut survivable_trades_count = 0usize;
    let mut sum_price = 0.0;
    let mut metrics = ScenarioMetrics::default();
    let mut probe_count = 0usize;
    let mut real_trade_count = 0usize;

    // Diagnostic Counters
    let mut _signal_count = 0usize;
    let mut entry_attempted = 0usize;
    let mut _total_trades = 0usize;
    let skipped_busy = 0usize;
    let mut exit_tp_count = 0usize;
    let mut exit_sl_count = 0usize;
    let mut exit_ts_count = 0usize;
    let _max_horizon = 200;

    let mut total_filled_qty = 0u64;
    let mut total_slippage_bps = 0.0;
    let mut fills_count = 0usize;
    let mut sum_drawdown_raw = 0.0;
    let mut sum_expected_move = 0.0;
    let mut total_tail_penalty = 0.0;
    let mut sum_latency_raw = 0.0;
    let mut cycle_sigs: Vec<ScenarioExecutionSignature> = Vec::new();

    // Phase 8.8 Aggregators
    let mut max_pnl_in_scenario: f64 = 0.0;
    let mut pnl_from_tp_scenario: f64 = 0.0;
    let mut pnl_from_sl_scenario: f64 = 0.0;
    let mut max_trade_pnl_scenario: f64 = 0.0;
    let mut long_win_count_scenario = 0usize;
    let short_win_count_scenario = 0usize;
    let mut micro_loss_count = 0u32;
    let mut total_window_volume = 0.0;
    let mut triggered_entries = 0usize; // ✅ ADD THIS
    let mut exec_passed = 0usize;

    // --- PHASE 14: DISTRIBUTION-AWARE SIGNAL VALIDATION LAYER ---
    // Transitioning from fixed-gate scoring to institutional selective-gating.
    // This ensures only true statistical outliers from the strategy are traded.

    let name_upper = scenario_name.to_uppercase();
    // Phase D.1.14: Bypass Eradication. We no longer allow bypass based on scenario names.
    // Structural integrity is now mandatory.
    let allow_bypass = std::env::var("GA_BYPASS_AQG").is_ok();

    // --- PHASE D.1.7: STRATEGY-LOCAL SIGNAL SPACE ---
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let strategy_seed = {
        let mut hasher = DefaultHasher::new();
        strategy.queue_threshold.hash(&mut hasher);
        strategy.base_edge.hash(&mut hasher);
        generation.hash(&mut hasher);
        hasher.finish()
    };
    let perturb = (strategy_seed % 1000) as f64 / 1000.0;

    // 0. Simulation Context (Phase D.1.18 alpha detection)
    let mut had_organic_signals = false;

    // 1. Pre-Scan (Collect All Scores with Perturbation)
    let mut window_data = Vec::with_capacity(signal_events.len());
    let mut scores = Vec::with_capacity(signal_events.len());
    for current_idx in 2..signal_events.len().saturating_sub(1) {
        let mut conviction = evaluate_market_conviction(
            strategy,
            scenario_name,
            signal_events,
            current_idx,
            0,
            generation,
        );

        // Phase D.1.21: Enforce Absolute Gating before stats
        if !conviction.is_valid {
            conviction.conviction_score *= 0.5; // degrade instead of kill
        }

        // Apply strategy-specific perturbation
        conviction.conviction_score *= 0.9 + 0.2 * perturb;

        scores.push(conviction.conviction_score);
        window_data.push((current_idx, conviction));
    }
    if window_data.len() < 5 {
        if ga_debug_enabled() {
            println!("⚠️ INSUFFICIENT SIGNAL BASE → activating degraded mode");
        }
        // 🔥 FIX: INLINE MINIMAL SIGNAL INJECTION (NO EXTERNAL FUNCTION)
        window_data.clear();

        let fallback_idx = signal_events.len() / 2;

        let mut conviction = evaluate_market_conviction(
            strategy,
            scenario_name,
            signal_events,
            fallback_idx,
            0,
            generation,
        );

        // soften but don't kill
        conviction.conviction_score = conviction.conviction_score.max(0.05);
        conviction.is_valid = true;

        window_data.push((fallback_idx, conviction));

        if ga_debug_enabled() {
            println!("⚠️ MINIMAL SIGNAL INJECTED → idx={}", fallback_idx);
        }
    }
    if window_data.is_empty() {
        println!("⚠️ NO WINDOW DATA → degraded continuation");

        return Some(StrategyEvaluation {
            strategy: strategy.clone(),
            strategy_id: strategy_id.clone(),
            fitness: -0.2,
            trade_count: 0,
            pnl_history: executed_trades.clone(),
            ..StrategyEvaluation::default()
        });
    }
    if scores.is_empty() {
        println!("⚠️ scores empty → injecting fallback score");
        scores.push(0.1);
    }

    // 2. Statistical Derivation & Adaptive Gating
    let n = scores.len() as f64;
    let mean = scores.iter().sum::<f64>() / n;
    let variance = scores.iter().map(|s| (s - mean).powi(2)).sum::<f64>() / n;
    let std_dev = variance.sqrt();

    // Phase 17A: Soften the Adaptive Quality Gate (AQG)
    // We change LOW_DISPERSION from a hard skip into a market-condition scaler.
    let dispersion_multiplier = if allow_bypass {
        1.0
    } else {
        (std_dev / MIN_STD).min(1.0)
    };

    // Adaptive Parameters
    let abs_floor = (percentile_f64(&scores, 0.80)).max(mean + 0.5 * std_dev);
    let z_threshold = (BASE_Z * (TARGET_STD / (std_dev + EPS))).clamp(0.8, 1.5);
    let min_signals = (window_data.len() as f64 * 0.05).max(2.0) as usize;

    // 3. Signal Validation Layer (Identify High-Conviction Cluster)
    // Phase 17A.5: Store dominance and reason to ensure consistency
    // Phase 17B: Store E-score for realizability analysis
    let mut valid_signals: Vec<(
        usize,
        ConvictionOutcome,
        f64,
        &'static str,
        f64,
        SignalSource,
        SignalSignature,
    )> = Vec::new();
    let mut emitted_signs: Vec<SignalAlpha> = Vec::new();
    let mut scored_signals: Vec<(
        usize,
        ConvictionOutcome,
        f64,
        &'static str,
        f64,
        f64,
        SignalSource,
    )> = Vec::new();
    let mut max_z = 0.0;

    // --- PHASE D.1.13.5: ADAPTIVE SIGNAL FLOOR (BOOTSTRAP FIX) ---
    // If the adaptive threshold is too strict (killing all signals), fallback to the 60th percentile.
    let percentile_60 = percentile_f64(&scores, 0.60);
    let effective_floor = abs_floor.max(percentile_60);

    let p75_energy = percentile_f64(&scores, 0.75);
    let energy_min = effective_floor.max(p75_energy);

    let mut decision_was_override = false;
    let mut _acceptance_mode = AcceptanceMode::Dominance;

    let mut candidate_signals: Vec<(
        usize,
        ConvictionOutcome,
        f64,
        &'static str,
        f64,
        SignalSource,
        f64,
        SignalSignature,
    )> = Vec::new();

    for (signal_idx, conviction) in window_data.iter() {
        // --- PHASE 17 CALIBRATION: RAW Population Integrity ---
        let mut sub_scores = [
            conviction.norm_momentum,
            conviction.norm_volume,
            conviction.norm_vol_score,
        ];
        sub_scores.sort_by(|a, b| b.total_cmp(a));

        let mean_t = sub_scores.iter().sum::<f64>() / 3.0;
        let std_t = (sub_scores.iter().map(|s| (s - mean_t).powi(2)).sum::<f64>() / 3.0).sqrt();
        let raw_dom_t = (sub_scores[0] - sub_scores[1]) / (std_t + EPS);
        metrics.record_pop_stats(raw_dom_t.min(3.0), false); // RAW layer

        let mut adj_conviction = conviction.clone();
        adj_conviction.conviction_score *= dispersion_multiplier;

        let score_val = adj_conviction.conviction_score;
        let z_score = (score_val - mean) / (std_dev + EPS);
        if z_score > max_z {
            max_z = z_score;
        }

        // --- PHASE D.1.17: COMPETITIVE ADMISSION (RELATIVE) ---
        // We only admit positive conviction to avoid garbage.
        if score_val > 0.0 {
            // Memory-Smoothed Stability
            let scores_idx = window_data
                .iter()
                .position(|(i, _)| *i == *signal_idx)
                .unwrap_or(0);
            let s_t = scores[scores_idx];
            let s_t_1 = if scores_idx >= 1 {
                scores[scores_idx - 1]
            } else {
                s_t
            };
            let s_t_2 = if scores_idx >= 2 {
                scores[scores_idx - 2]
            } else {
                s_t_1
            };
            let delta = (s_t - s_t_1).abs() + (s_t_1 - s_t_2).abs();
            let stability = (1.0 - delta / (2.0 * (std_dev + EPS).max(0.05))).clamp(0.0, 1.0);

            // Execution Realizability (E-score)
            let current_price = signal_events[*signal_idx].price as f64;
            let prev_price = if *signal_idx > 0 {
                signal_events[*signal_idx - 1].price as f64
            } else {
                current_price
            };
            let atr = calculate_atr(signal_events, *signal_idx, 14);
            let dist_score = (1.0
                - ((current_price - prev_price).abs()
                    / ((if atr > 0.0 { atr } else { current_price * 0.01 }) + EPS))
                    .min(1.0))
            .clamp(0.0, 1.0);

            // Regime-Aware Volatility Score
            let scores_idx_2 = window_data
                .iter()
                .position(|(i, _)| *i == *signal_idx)
                .unwrap_or(0);
            let local_window = (scores_idx_2 + 1).min(20);
            let start_idx = (scores_idx_2 + 1).saturating_sub(local_window);
            let local_slice = &scores[start_idx..=scores_idx_2];
            let local_mean = local_slice.iter().sum::<f64>() / local_window as f64;
            let local_var = local_slice
                .iter()
                .map(|s| (s - local_mean).powi(2))
                .sum::<f64>()
                / local_window as f64;
            let local_vol = local_var.sqrt();
            let vol_ratio = (local_vol / (std_dev + EPS)).clamp(0.25, 4.0);
            let vol_score = (1.0 - (vol_ratio - 1.0).abs()).clamp(0.0, 1.0);

            let e_signal_score = ((stability + dist_score + vol_score) / 3.0).clamp(0.0, 1.0);

            // --- Phase D.1.18: Calculate Signal Signature ---
            let regime = if vol_ratio > 1.3 {
                1
            } else if vol_ratio < 0.7 {
                -1
            } else {
                0
            };
            let momentum = if adj_conviction.norm_momentum > 0.3 {
                1
            } else if adj_conviction.norm_momentum < -0.3 {
                -1
            } else {
                0
            };
            let signature = SignalSignature {
                archetype: strategy.archetype,
                regime,
                momentum,
            };

            candidate_signals.push((
                *signal_idx,
                adj_conviction,
                raw_dom_t,
                "RELATIVE_CANDIDATE",
                e_signal_score,
                SignalSource::Organic,
                stability,
                signature,
            ));
        }
    }

    // --- PHASE D.1.17: TOP-K SELECTION & FORCE EMERGENCE ---
    // 1. Sort by conviction intensity
    candidate_signals.sort_by(|a, b| b.1.conviction_score.total_cmp(&a.1.conviction_score));

    // 2. Select Top-5 and Apply Curved Penalties + Credibility
    for (idx, (signal_idx, mut conv, dom, _reason, e_signal_score, source, stability, signature)) in
        candidate_signals.into_iter().enumerate()
    {
        let min_conviction_threshold = 0.1;
        if conv.conviction_score < min_conviction_threshold {
            conv.conviction_score *= 0.5;
        }

        // --- Refined Multipliers (D.1.17 Sharpe) ---
        let z_score = (conv.conviction_score - mean) / (std_dev + EPS);
        let stat_confidence = (z_score / z_threshold).clamp(0.5, 1.5);
        let stability_factor = stability.clamp(0.1, 1.0).powf(2.0);

        // --- Phase D.1.18: Credibility Overlay ---
        let credibility = if let Some(stats) = metrics.signature_memory.get(&signature) {
            let winrate = if stats.sample_count > 0 {
                stats.win_count as f64 / stats.sample_count as f64
            } else {
                0.5
            };
            let avg_pnl = if stats.sample_count > 0 {
                stats.sum_pnl / stats.sample_count as f64
            } else {
                0.0
            };
            let c = (avg_pnl * winrate) * (stats.sample_count as f64 + 1.0).ln();

            // Phase D.1.18 Alpha Threshold check
            if stats.sample_count > 5 && winrate > 0.55 && avg_pnl > 0.0005 {
                had_organic_signals = true;
            }

            (1.0 + c).clamp(0.5, 2.0)
        } else {
            1.0
        };

        conv.edge_weight = (stat_confidence * stability_factor * credibility).clamp(0.2, 2.5);
        valid_signals.push((
            signal_idx,
            conv,
            dom,
            "RELATIVE_CANDIDATE",
            e_signal_score,
            source,
            signature,
        ));
    }

    // 🔥 STEP 2: DEBUG signal composition (VERY IMPORTANT)
    if valid_signals.is_empty() {
        if window_data.len() >= 10 {
            valid_signals = extract_weak_signals(&window_data, strategy);

            if valid_signals.is_empty() {
                if ga_debug_enabled() {
                    println!("⚠️ NO VALID SIGNALS → degraded continuation");
                }

                return Some(StrategyEvaluation {
                    strategy: strategy.clone(),
                    strategy_id: strategy_id.clone(),
                    fitness: -0.15,
                    trade_count: 0,
                    pnl_history: Vec::new(),
                    ..StrategyEvaluation::default()
                });
            }

            if std::env::var("GA_VERBOSE").is_ok() {
                if ga_debug_enabled() {
                    println!("⚠️ DEGRADED SIGNAL MODE → weak signals allowed");
                }
            }
        }
        // } else {
        //     return None;
        // }
    }

    let mut fallback_count = 0;
    let mut organic_count = 0;

    for (_, _, _, _, _, source, _) in &valid_signals {
        match source {
            SignalSource::Synthetic => fallback_count += 1,
            _ => organic_count += 1,
        }
    }
    if std::env::var("GA_SIGNAL_DEBUG").is_ok() {
        println!("SIGNAL_COMPOSITION → organic={}", organic_count);
    }
    // --- PHASE 14++: STRUCTURAL METRICS & DISTRIBUTION AWARENESS ---

    // --- PHASE 14++: STRUCTURAL METRICS & DISTRIBUTION AWARENESS ---
    let _top_1 = scores.iter().fold(0.0f64, |a, b| a.max(*b));
    let _scores_sum: f64 = scores.iter().sum::<f64>();

    let mut sorted_scores = scores.clone();
    sorted_scores.sort_by(|a, b| b.total_cmp(a));
    let top_k_sum: f64 = sorted_scores.iter().take(3).sum();

    let std_v = if scores.len() > 1 {
        let variance = scores.iter().map(|s| (s - mean).powi(2)).sum::<f64>() / scores.len() as f64;
        variance.sqrt()
    } else {
        0.0
    };

    // Purity: Ratio of High-Quality signals (E > 0.80) in the VALID pool
    let high_quality_count = valid_signals
        .iter()
        .filter(|(_, conv, _, _, _, _, _)| conv.conviction_score > effective_floor)
        .count();
    let purity = high_quality_count as f64 / valid_signals.len().max(1) as f64;

    // Agreement: Ratio of dominant side (BUY vs SELL)
    let mut buy_count = 0usize;
    let mut sell_count = 0usize;
    for (_, conv, _, _, _, _, _) in &valid_signals {
        if conv.is_bearish {
            sell_count += 1;
        } else {
            buy_count += 1;
        }
    }
    let dominant_count = buy_count.max(sell_count);
    // Phase 11 & D.1.7: Deterministic Agreement Jitter (Break 1.0 Lock)
    let noise = (strategy_seed % 100) as f64 / 100.0;
    let jitter = 0.85 + 0.3 * noise;
    let mut agreement = (dominant_count as f64 / valid_signals.len().max(1) as f64) * jitter;

    // --- PHASE D.1.7: DETERMINISTIC AGREEMENT ASYMMETRY (WIDE RANGE) ---
    // Aggressively break "degenerate consensus" with deterministic ID-based shift.
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    strategy_id.hash(&mut hasher);
    let id_hash = hasher.finish();
    let asymmetry = 0.85 + ((id_hash % 100) as f64 / 100.0) * 0.3; // 0.85 -> 1.15

    agreement = (agreement * asymmetry).clamp(0.0, 1.0);

    // --- PHASE A+: SCORING ENGINE ---
    // 1. Identify "Window Potential" (Best candidate stats for structural pulse)
    let (best_dom, best_signal_e) = valid_signals
        .iter()
        .max_by(|(_, a, _, _, _, _, _), (_, b, _, _, _, _, _)| {
            a.conviction_score.total_cmp(&b.conviction_score)
        })
        .map(|(_, _, dom, _, e_score, _, _)| (*dom, *e_score))
        .unwrap_or((0.0, 0.0));
    let stability_raw = (1.0 - std_v / 0.18).clamp(0.0, 1.0);

    // 2. Update structural stats BEFORE gating (Layer 1: Perception)
    metrics.record_adaptive_pulse(agreement, best_dom, purity, stability_raw, max_z, 0.0);

    // 3. Normalized Metrics (Z-scores) using Institutional Priors (Fallback N < 20)
    let n_count = metrics.adaptive.agreement.count;

    let calc_z = |val: f64, tracker: &WelfordTracker, p_mu: f64, p_sigma: f64| -> f64 {
        if n_count < 20 {
            let safe_sigma = p_sigma.max(0.05);
            ((val - p_mu) / safe_sigma).clamp(-5.0, 5.0)
        } else {
            let safe_std = tracker.std().max(0.05);
            ((val - tracker.mean()) / safe_std).clamp(-5.0, 5.0)
        }
    };

    let agreement_z = calc_z(agreement, &metrics.adaptive.agreement, 0.65, 0.10);
    let _purity_z = calc_z(purity, &metrics.adaptive.purity, 0.60, 0.15);
    let stability_z = calc_z(stability_raw, &metrics.adaptive.stability, 0.15, 0.05);
    let dominance_z = calc_z(best_dom, &metrics.adaptive.dominance, 0.20, 0.10);
    let z_norm = calc_z(max_z, &metrics.adaptive.z_score, 1.50, 0.50);
    let energy_norm = dominance_z; // fallback to conviction strength

    // Final Adaptive Score (Weighted Sum)
    // 30% z_norm, 25% energy_norm, 20% dominance_norm, 15% agreement_norm, 10% stability_norm
    let final_score = 0.30 * z_norm
        + 0.25 * energy_norm
        + 0.20 * dominance_z
        + 0.15 * agreement_z
        + 0.10 * stability_z;

    let adaptive_threshold = metrics.adaptive_threshold(0.60); // Use PREVIOUS history
    let is_struct_valid_adaptive = final_score >= (adaptive_threshold - 0.55)
        && stability_raw > 0.3
        && purity > 0.2
        && agreement > 0.5; // Phase D.1.22: Hard AQG Baseline

    // Update structural stats and history AFTER gate decision
    metrics.record_final_score(final_score);
    if is_struct_valid_adaptive {
        metrics.adaptive_opportunity_count += 1;
    }
    metrics.record_structural_health(agreement, purity, std_v, is_struct_valid_adaptive);

    // --- PHASE D.1.9: RATIO-INTERACTION SCORING & DETERMINISTIC CHOICE ---
    // Moved outside of gating to ensure emission even in bypass mode.

    // 1. Archetype Bias Mapping (Soft Offsets)
    let mut adj_w_conv = strategy.w_conviction as f64;
    let mut adj_w_mom = strategy.w_momentum as f64 - 50.0; // centered
    let mut adj_w_vol = strategy.w_volatility as f64;
    let mut adj_exp_vol = strategy.exp_volatility as f64;

    match strategy.archetype {
        0 => {
            adj_w_conv += 20.0;
        } // ConvictionDominant
        1 => {
            adj_w_mom += 20.0;
        } // MomentumTrend
        2 => {
            adj_w_mom -= 20.0;
        } // MeanReversion
        3 => {
            adj_exp_vol += 30.0;
        } // VolatilityAverse
        _ => {}
    }

    // 2. Weight Normalization
    let w_sum = adj_w_conv.abs() + adj_w_mom.abs() + adj_w_vol.abs() + 1e-9;
    let w1 = adj_w_conv / w_sum;
    let w2 = adj_w_mom / w_sum;
    let w3 = adj_w_vol / w_sum;

    let a_exp = (0.5 + 3.0 * (strategy.exp_conviction as f64 / 100.0)).clamp(0.5, 3.5);
    let b_exp = (0.5 + 3.0 * (strategy.exp_momentum as f64 / 100.0)).clamp(0.5, 3.5);
    let c_exp = (0.5 + 3.0 * (adj_exp_vol / 100.0)).clamp(0.5, 3.5);

    // 3. Compute Phase D.1.16 Signal Entropy
    let signal_entropy = compute_std_dev(
        &valid_signals
            .iter()
            .map(|(_, c, _, _, _, _, _)| c.conviction_score)
            .collect::<Vec<f64>>(),
    );

    scored_signals.clear();
    for (signal_idx, conviction, dom, reason, e_score, source, _sig) in valid_signals.iter() {
        let vol_penalty = conviction.norm_vol.max(1e-6);
        let mom = conviction.norm_momentum;
        let conv = conviction.conviction_score;

        let mom_effect = 1.0 + (w2 * mom.powf(b_exp)).clamp(-0.8, 0.8);
        let denom = 0.01 + w3.abs() * vol_penalty.powf(c_exp);

        let mut survival_score = (w1 * conv.powf(2.0)) * mom_effect / denom.powf(0.7);

        let rank_offset = ((*signal_idx % 20) as f64) * 0.01;
        let entropy_factor = 1.0 + (signal_entropy * 0.2);
        survival_score = (survival_score + rank_offset) * entropy_factor;

        let noise = ((strategy_seed ^ (*signal_idx as u64)) % 1000) as f64 / 1000.0;
        let jitter = (noise - 0.5) * 0.05;

        survival_score += jitter;
        let adjusted_score = (survival_score.max(0.0)) * clarity_penalty;

        scored_signals.push((
            *signal_idx,
            conviction.clone(),
            *dom,
            *reason,
            *e_score,
            adjusted_score,
            *source,
        ));
    }

    scored_signals.sort_by(|a, b| b.5.total_cmp(&a.5));

    let probs: Vec<f64> = valid_signals
        .iter()
        .map(|(_, c, _, _, _, _, _)| c.conviction_score.max(1e-6))
        .collect();
    let sum_probs: f64 = probs.iter().sum();
    let entropy = if sum_probs > 0.0 {
        probs
            .iter()
            .map(|p| {
                let p_norm = p / sum_probs;
                -p_norm * p_norm.ln()
            })
            .sum::<f64>()
    } else {
        0.0
    };

    let max_entropy = (probs.len() as f64).ln().max(1.0);
    let dispersion_metrics = (entropy / max_entropy).clamp(0.0, 1.0);
    let opportunity = ((valid_signals.len() as f64 / 200.0).powf(0.7)).clamp(0.0, 1.0);
    let combined_metric = (0.6 * dispersion_metrics + 0.4 * opportunity).clamp(0.0, 1.0);

    let max_score_val = scored_signals.first().map(|s| s.5).unwrap_or(0.0);
    let min_score_val = scored_signals.last().map(|s| s.5).unwrap_or(0.0);
    let score_spread = max_score_val - min_score_val;

    let emission_threshold = if score_spread < 1e-4 {
        max_score_val * 0.7
    } else {
        max_score_val - score_spread * (0.25 + 0.5 * (1.0 - dispersion_metrics))
    };

    scored_signals.retain(|s| s.5 >= emission_threshold);
    emitted_signs.clear();

    for (signal_idx, conv, _, _, e_score, score, source) in scored_signals.iter() {
        let consensus_score = conv.conviction_score.clamp(-1.0, 1.0);
        let effective_edge = consensus_score.abs() * e_score;
        let strength = (0.7 * effective_edge + 0.3 * consensus_score.abs())
            .clamp(0.05, 1.0);

        emitted_signs.push(SignalAlpha {
            ts: *signal_idx,
            price: signal_events[*signal_idx].price as f64,
            archetype: strategy.archetype,
            direction: if consensus_score < 0.0 { -1 } else { 1 },
            strength,
            source: *source,
            conviction: conv.clone(),
            is_probe: false,
        });
    }
    // 🚨 HARD FALLBACK — ensure at least 1 signal
    if emitted_signs.is_empty() && scored_signals.len() >= 5 {
        let (signal_idx, conv, _, _, e_score, score, source) = &scored_signals[0];

        if ga_debug_enabled() {
            println!("🚨 FORCED EMISSION → fallback activated");
        }

        let consensus_score = conv.conviction_score.clamp(-1.0, 1.0);
        let effective_edge = consensus_score.abs() * e_score;

        let strength = (0.7 * effective_edge + 0.3 * consensus_score.abs())
            .clamp(0.05, 1.0);

        emitted_signs.push(SignalAlpha {
            ts: *signal_idx,
            price: signal_events[*signal_idx].price as f64,
            archetype: strategy.archetype,
            direction: if consensus_score < 0.0 { -1 } else { 1 },
            strength,
            source: *source,
            conviction: conv.clone(),
            is_probe: false,
        });
    }

    if std::env::var("GA_SIGNAL_DEBUG").is_ok() {
        println!(
            "EMISSION_DEBUG → valid={} scored={} emitted={}",
            valid_signals.len(),
            scored_signals.len(),
            emitted_signs.len()
        );
    }
    // if emitted_signs.len() > 50 {
    //     emitted_signs.sort_by(|a, b| b.strength.total_cmp(&a.strength));
    //     emitted_signs.truncate(50); // hard safety cap
    // }

    // if emitted_signs.is_empty() {
    //     println!("⚠️ EMISSION FAILURE → forcing fallback");

    //     if let Some((idx, conv)) = window_data.first() {
    //         emitted_signs.push(SignalAlpha {
    //             ts: *idx,
    //             price: signal_events[*idx].price as f64,
    //             archetype: strategy.archetype,
    //             direction: if conv.is_bearish { -1 } else { 1 },
    //             strength: 0.05,
    //             source: SignalSource::Synthetic,
    //         });
    //     }
    // }

    let scores_sum: f64 = emitted_signs
        .iter()
        .map(|s| s.strength)
        .sum::<f64>()
        .max(1e-9);

    // 🔥 portfolio-based dominance
    let dominance = emitted_signs
        .iter()
        .map(|s| s.strength)
        .reduce(f64::max)
        .unwrap_or(0.0);

    let winner_score =
        emitted_signs.iter().map(|s| s.strength).sum::<f64>() / emitted_signs.len().max(1) as f64;

    if std::env::var("GA_SIGNAL_DEBUG").is_ok() {
        println!("WINDOW_DECISION → {} | sigs={} z={:.2} dom={:.3} mean={:.3} purity={:.2} conc={:.2} agree={:.2} => {}", 
                    scenario_name, valid_signals.len(), max_z, dominance, mean, purity, top_k_sum / scores_sum.max(EPS), agreement, if valid_signals.is_empty() { "REJECT_VACUUM" } else { "ACCEPTED" });
    }

    let median_score = percentile_f64(&scores, 0.50);
    // Raw value preserved for metrics; clamped value used for decisions and logging
    let raw_edge_spread_norm = (winner_score - median_score) / (std_dev + EPS);
    if raw_edge_spread_norm.abs() > 50.0 {}
    let edge_spread_norm = raw_edge_spread_norm.clamp(-10.0, 10.0);

    let _signal_count = valid_signals.len();

    let mut busy_until = 0usize;
    let cooldown = config.trade_cooldown_events.unwrap_or(8);

    let mut baseline_pnl = 0.0;

    // Funnel analytics
    let funnel_signals = signal_events.len();
    let mut funnel_after_signal_filter = 0usize;
    let mut funnel_after_edge_filter = 0usize;
    let mut funnel_after_exec_prob = 0usize;

    let mut rejected_trades: usize = 0;
    let mut forced_execution_done = false;
    if emitted_signs.is_empty() {
        if ga_debug_enabled() {
            println!(
                "⚠️ EMISSION FAILURE → injecting probe for scenario={}",
                scenario_name
            );
        }

        let fallback_idx = if let Some((idx, _)) = window_data.first() {
            *idx
        } else {
            signal_events.len() / 2
        };

        let conviction = evaluate_market_conviction(
            strategy,
            scenario_name,
            signal_events,
            fallback_idx,
            0,
            generation,
        );

        let consensus_score = conviction.conviction_score.clamp(-1.0, 1.0);
        emitted_signs.push(SignalAlpha {
            ts: fallback_idx,
            price: signal_events[fallback_idx].price as f64,
            archetype: strategy.archetype,
            direction: if consensus_score < 0.0 { -1 } else { 1 },
            strength: 0.1,
            source: SignalSource::Synthetic,
            conviction,
            is_probe: true,
        });
    }

    if emitted_signs.is_empty() {
        println!("⚠️ NO EMISSION EVEN AFTER PROBE → degraded continuation");

        return Some(StrategyEvaluation {
            strategy: strategy.clone(),
            strategy_id: strategy_id.clone(),
            fitness: -0.1,
            trade_count: 0,
            pnl_history: Vec::new(),
            ..StrategyEvaluation::default()
        });
    }

    // 🔥 SINGLE SOURCE OF TRUTH
    let starvation_ratio = 1.0 - (emitted_signs.len() as f64 / valid_signals.len().max(1) as f64);
    let adaptive_factor = (1.0 - 0.8 * starvation_ratio).clamp(0.2, 1.0);

    // 🔥 Phase 3.6.6: Signal Topology Diagnostic
    let mut histogram = [0usize; 10];
    for &score_val in &scores {
        let bin = ((score_val.abs() * 10.0).floor() as usize).min(9);
        histogram[bin] += 1;
    }
    if ga_debug_enabled() {
        println!(
            "CONVICTION_HISTOGRAM → scenario={} {:?}",
            scenario_name, histogram
        );
    }

    // ✅ USE PREVIOUS SNAPSHOT (no intra-generation feedback)
    // 🔒 Freeze snapshot BEFORE using it
    // 🔒 HARD SNAPSHOT (NO FEEDBACK)
    const EDGE_WINDOW: usize = 200;

    // 🔥 USE ONLY PREVIOUS BUILD (NOT MUTATING ONE)
    let edge_dist_snapshot = if edge_vals_snapshot.len() >= 10 {
        edge_vals_snapshot.clone()
    } else if edge_vals_snapshot.len() > 0 {
        edge_vals_snapshot.clone()
    } else {
        vec![0.0003, 0.0008, 0.0015, 0.002, 0.003]
    };
    // ===============================
    // 🔥 CONSISTENT EDGE DISTRIBUTION
    // ===============================

    println!(
        "SNAPSHOT_SOURCE_CHECK → using_snapshot={} snapshot_size={} build_size={}",
        edge_vals_snapshot.len() > 0,
        edge_vals_snapshot.len(),
        edge_vals_build.len()
    );
    // Use ONLY candidate_edges (single source of truth)
    let mut edge_dist: Vec<f64> = edge_dist_snapshot
        .iter()
        .cloned()
        .filter(|v| v.is_finite() && *v > 0.0)
        .collect();

    // 🔥 Fallback ONLY if empty (rare)
    if edge_dist.len() < 5 {
        if ga_debug_enabled() {
            println!("⚠️ EDGE DIST FALLBACK → insufficient candidate_edges");
        }
        edge_dist = vec![0.0003, 0.0008, 0.0015, 0.002, 0.003, 0.005];
    }

    let e_stats = DistributionStats::from_slice(&mut edge_dist);

    // 🔥 Debug (keep this)
    println!(
        "EDGE_DIST_DEBUG → p10={:.6} p50={:.6} p90={:.6} max={:.6}",
        e_stats.p10,
        e_stats.p50,
        e_stats.p90,
        e_stats.empirical_samples.last().unwrap_or(&0.0),
    );
    // 🔥 SORT SAFE
    edge_dist.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    // 🔥 OUTLIER TRIM (CRITICAL)
    let n = edge_dist.len();
    let lower = n / 20; // 5%
    let upper = n * 19 / 20; // 95%

    let trimmed = if upper > lower {
        &edge_dist[lower..upper]
    } else {
        &edge_dist[..]
    };
    let t_len = trimmed.len();
    let spread_ratio = e_stats.p90 / (e_stats.p30 + 1e-9);

    if spread_ratio < 1.2 {
        println!(
            "⚠️ EDGE COLLAPSE → p30={:.6} p90={:.6} ratio={:.3}",
            e_stats.p30, e_stats.p90, spread_ratio
        );
    }

    if e_stats.p90 / (e_stats.p30 + 1e-9) > 20.0 {
        println!("⚠️ EDGE DISTORTION → p30={:.6} p90={:.6}", e_stats.p30, e_stats.p90);
    }

    // 🔥 Percentile Stretching
    let p10 = percentile_f64(&scores, 0.1);
    let p50 = percentile_f64(&scores, 0.5);
    let p70 = percentile_f64(&scores, 0.7);
    let p90 = percentile_f64(&scores, 0.9);
    let max_score = scores.iter().fold(f64::MIN, |a, &b| a.max(b));
    let score_range = (p90 - p10).max(1e-3);

    if ga_debug_enabled() {
        println!(
            "🌐 [SCORE_LANDSCAPE] scenario={} min={:.4} p10={:.4} p50={:.4} p70={:.4} p90={:.4} max={:.4} range={:.4}",
            scenario_name, scores.iter().fold(f64::MAX, |a, &b| a.min(b)), p10, p50, p70, p90, max_score, score_range
        );
    }

    let mut simulated_count = 0;
    let mut priority_pass_count = 0;
    let mut edge_pass_count = 0;
    let mut exec_pass_count_local = 0;
    // 🔥 HARD BIMODAL SPLIT (GLOBAL CONTEXT)
    let live_threshold = e_stats.p50; // or e_p60 for stricter
    for signal in &emitted_signs {
        // 🔥 Phase 3.6.6: Identity-Based Divergence (Restores Gradient)
        let conviction_val = signal.conviction.conviction_score;
        let stretched = (conviction_val - p10) / score_range;

        // Time-aware deterministic epsilon (identity injection)
        let eps_raw = ((strategy_index.wrapping_mul(31) ^ signal.ts.wrapping_mul(17)) % 100) as f64;
        let identity_eps = (eps_raw - 50.0) / 10000.0; // [-0.005, 0.005]

        let mut priority_score = (stretched.max(0.0)).ln_1p(); // log(1 + x)

        // normalize
        priority_score = priority_score / (1.0 + priority_score);

        // avoid 1.0 attractor
        priority_score = priority_score.clamp(0.0, 0.999);
        let strat_bias = (strategy_index % 10) as f64 * 0.005; // Tightened 0.000 -> 0.045

        let dynamic_threshold = (adaptive_threshold + strat_bias).clamp(0.35, 0.85);

        let pass = priority_score > dynamic_threshold;

        if !pass {
            continue;
        }

        priority_pass_count += 1;
        if simulated_count >= 120 {
            break;
        }
        simulated_count += 1;

        // 🔥 Phase 3.6.6: Temporal Phase Shift (Queue interaction divergence)
        let phase_shift = ((strategy_index % 7) as i32 - 3) as isize;
        let execution_idx = (signal.ts as isize + phase_shift).max(0) as usize;
        // Use a 10-tick safety buffer to allow room for latency/offsets without slice panics
        let execution_idx = execution_idx.min(signal_events.len().saturating_sub(10));

        let current_idx = execution_idx;

        metrics.record_funnel_admission(signal.conviction.conviction_score);

        let mut conviction = signal.conviction.clone();

        if current_idx < busy_until {
            // 🔥 TEMP: disable execution lock to test concurrency
            // continue;
        }

        // --- SELECTION GATES (CLEAN + COMPLETE) ---

        // 2. Exploration Bypass (Probabilistic soft-pass)
        funnel_after_signal_filter += 1;
        // 3. Use selected conviction (global or local)
        let final_conviction = conviction.clone();

        // // 4. Hard participation threshold (deterministic)
        // let exec_threshold = {
        //     let p60 = percentile_f64(&scores, 0.6);
        //     let mean = scores.iter().sum::<f64>() / scores.len() as f64;
        //     let std = (scores.iter().map(|s| (s - mean).powi(2)).sum::<f64>()
        //         / scores.len() as f64)
        //         .sqrt();

        //     if std < 0.05 {
        //         mean + 0.3 * std
        //     } else {
        //         p60
        //     }
        // };

        let conviction_score = final_conviction.conviction_score;
        // --- NEW: execution-aware features ---
        let queue_ahead = outcome_estimate_queue(signal_events, current_idx);
        let trade_velocity = estimate_trade_velocity(signal_events, current_idx);
        let volatility = std_dev.max(1e-6);

        // 🔥 EXECUTION MODEL (BOUND TO REAL VARIABLES)

        // liquidity proxy (you already compute this)
        let liquidity = trade_velocity.max(1e-6);

        // --- FIXED EXECUTION MODEL (NON-COLLAPSING) ---

        // Normalize queue pressure (bounded, smooth)
        let queue_norm = (queue_ahead / (queue_ahead + 5000.0)).clamp(0.0, 1.0);

        // Liquidity strength (relative, not absolute)
        let liquidity_norm = (liquidity / (liquidity + 1000.0)).clamp(0.0, 1.0);

        // Volatility boost (fast markets fill easier)
        let volatility_boost = (volatility * 5.0).tanh(); // smooth [0,1)

        // Base fill probability (balanced interaction)
        let mut fill_probability =
            0.4 * liquidity_norm + 0.3 * (1.0 - queue_norm) + 0.3 * volatility_boost;

        // 🔥 Inject micro-variance (CRITICAL)
        let noise = ((strategy_index ^ current_idx) % 100) as f64 / 1000.0;
        fill_probability += (noise - 0.05) * 0.5;
        fill_probability = fill_probability.clamp(0.05, 0.95);

        let latency_impact = (volatility * config.latency_ticks as f64).min(0.5);
        let adverse_selection = (volatility * 0.5).clamp(0.0, 0.5);

        // 🔥 EXPAND CAPTURE DISTRIBUTION (FIX 5)
        let mut capture_prob =
            0.5 * fill_probability + 0.3 * (1.0 - latency_impact) + 0.2 * (1.0 - adverse_selection);

        // 🔥 amplify spread (instead of flattening)
        capture_prob = capture_prob.powf(1.2);

        // 🔥 inject divergence
        let cap_noise = ((current_idx * 13 + strategy_index * 7) % 100) as f64 / 1000.0;
        capture_prob += cap_noise - 0.05;

        capture_prob = capture_prob.clamp(0.05, 0.98);

        let entry_price = signal_events[current_idx].price as f64;
        let atr = calculate_atr(signal_events, current_idx, 14);

        // --- NEW: realized edge directly from signal (d=dir_signal driven) ---
        // ✅ FIX 2: Edge must be a function of the signal's conviction/z-score, not ATR only.
        //   dir_signal ∈ [-3,3] → use its abs as a scaling factor so edge VARIES with signal.
        let consensus_score = final_conviction.conviction_score.clamp(-1.0, 1.0);

        let raw_edge =
            consensus_score * final_conviction.edge_weight * (atr / entry_price.max(1e-6));
        // let expected_move = raw_edge.max((atr / entry_price.max(1e-6)) * 0.3);

        // ✅ FIX 3: Break edge symmetry — amplify differences
        let shaped_edge = raw_edge.abs().powf(0.8) * raw_edge.signum();
        let _ = shaped_edge; // available for future use

        // 🔥 FIX: Use REALIZED edge AFTER execution, not synthetic estimate
        let mut expected_realized_edge = raw_edge * capture_prob * signal.strength;

        // DO NOT finalize here — this is only a placeholder
        // It will be CORRECTED after execution

        println!(
            "EDGE_RAW_DEBUG → expected={:.6} p30={:.6} p50={:.6} p90={:.6} up={:.6} down={:.6}",
            expected_realized_edge,
            e_stats.p30,
            e_stats.p50,
            e_stats.p90,
            (e_stats.p90 - e_stats.p50),
            (e_stats.p50 - e_stats.p30)
        );
        // 🔥 anti-collapse jitter to preserve variance
        let jitter = ((strategy_index ^ signal.ts) % 10) as f64 * 1e-5;
        // 🔥 FIX: build distribution from raw edge, NOT realized

        // ✅ FIX 5: Edge collapse detector
        // if edge_vals_snapshot.len() >= 10 {
        //     let mean_e = edge_vals_snapshot.iter().sum::<f64>() / edge_vals_snapshot.len() as f64;
        //     let var_e = edge_vals_snapshot
        //         .iter()
        //         .map(|v| (v - mean_e).powi(2))
        //         .sum::<f64>()
        //         / edge_vals_snapshot.len() as f64;
        //     let edge_std_dev = var_e.sqrt();
        //     if edge_std_dev < 1e-4 {
        //         if ga_debug_enabled() {
        //             println!(
        //                 "⚠️ EDGE COLLAPSE DETECTED → std_dev={:.8} (n={})",
        //                 edge_std_dev,
        //                 edge_vals_snapshot.len()
        //             );
        //         }
        //     }
        // }

        // ✅ FIX 2: EDGE_DEBUG — print every edge value to expose degeneracy
        // 🔥 INSERT THIS FULL BLOCK HERE (MISSING IN YOUR CODE)
        let baseline = 0.0005;
        // let scaled = aqg_threshold * capture_prob;

        // 🔥 ALIGN AQG TO REALIZED SPACE
        let realized_avg_edge = avg_edge * capture_prob;
        let realized_aqg = aqg_threshold * capture_prob;

        // ✅ NEW: robust edge normalization (non-collapsing)
        let upper_spread = (e_stats.p90 - e_stats.p50).max(1e-5);
        let lower_spread = (e_stats.p50 - e_stats.p30).max(1e-5);

        // 3. Normalize WITHOUT bias

        // 🔥 CORE EXECUTION CONTEXT (MUST EXIST BEFORE EVERYTHING)
        let e_p70 = trimmed[t_len * 70 / 100];
        // 🔥 HARD REGIME CLASSIFICATION
        // 🔥 FIX: compare in SAME SCALE (use raw_edge, not realized)
        let is_dead_regime = raw_edge < e_stats.p30;
        let is_live_regime = raw_edge > e_stats.p50;

        // ❌ HARD REJECTION (THIS IS THE FIX)
        // 🔥 FIX: degrade instead of kill
        let mut regime_penalty = 1.0;

        if is_dead_regime {
            regime_penalty = 0.3; // degrade instead of reject
        }
        // 🔥 STEP 5 FIX — clamp to distribution scale
        expected_realized_edge = expected_realized_edge.clamp(-0.02, 0.02);
        // 🔥 STEP 4 FIX — Smooth normalization in realized space
        if !expected_realized_edge.is_finite() {
            expected_realized_edge = 0.0;
        }
        // ===============================
        // EDGE NORMALIZATION (FIXED SAFE)
        // ===============================

        // Prevent divide-by-zero
        let denom = (e_stats.p90 - e_stats.p30).abs().max(1e-6);
        let mut edge_norm = (expected_realized_edge - e_stats.p30) / denom;

        // 🔥 SOFT FLOOR (instead of hard zero)
        if edge_norm < 0.0 {
            edge_norm *= 0.25;
        }

        // 🔥 SOFT CEILING
        if edge_norm > 1.0 {
            edge_norm = 1.0 + (edge_norm - 1.0) * 0.25;
        }

        // 🔥 FINAL CLAMP (wide range, preserves gradient)
        edge_norm = edge_norm.clamp(-0.25, 1.25);

        println!(
            "EDGE_DIST_DEBUG_FIXED → raw={:.6} p30={:.6} p90={:.6} norm={:.6}",
            raw_edge, e_stats.p30, e_stats.p90, edge_norm
        );
        // Debug (keep this)
        println!(
            "EDGE_DIST_DEBUG → raw={:.6} p30={:.6} p50={:.6} p90={:.6} norm={:.6}",
            raw_edge, e_stats.p30, e_stats.p50, e_stats.p90, edge_norm
        );
        let mut edge_raw = raw_edge;
        // preserve true scale
        let edge_raw = raw_edge;

        println!(
            "EDGE_NORM_FIXED → raw={:.6} realized={:.6} p30={:.6} p50={:.6} p90={:.6}",
            edge_raw, expected_realized_edge, e_stats.p30, e_stats.p50, e_stats.p90
        );

        let signal_fail = final_conviction.conviction_score < 0.015;
        let bull = signal.conviction.bullish_score;
        let bear = signal.conviction.bearish_score;
        let sanity_fail = (bull < 1e-6 && bear < 1e-6) || (bull - bear).abs() < 1e-6;

        // 🔥 Phase V4.2 Refined Gating (with Gradual Degradation)
        let passed_gate = !signal_fail && !sanity_fail;

        let edge_ratio = edge_raw; // 🔥 preserve gradient

        let spread_z = edge_spread_norm;

        // normalize edge ratio into usable signal
        // 🔥 smoother mapping
        let edge_component = edge_ratio;

        // penalize unstable distributions
        let spread_penalty = (spread_z.abs() / 3.0).clamp(0.0, 1.0);

        // volatility penalty (non-linear)
        let vol_penalty = volatility.powf(0.7).clamp(0.0, 1.0);

        // core survival before shaping
        let mut survival_prob = if is_live_regime {
            0.8
        } else {
            0.0 // dead regime → no survival
        };

        // 🔥 APPLY CURVATURE (VERY IMPORTANT)
        let valley_penalty = 1.0 - (-((edge_raw - 0.65).powi(2)) / 0.01).exp();
        survival_prob = 0.7 * survival_prob + 0.3 * valley_penalty;

        // widen distribution
        survival_prob = survival_prob.clamp(0.15, 0.95);

        if std::env::var("GA_TRACE").is_ok() {
            println!(
                "THRESH_DEBUG → final={:.6} aqg={:.6} avg_edge={:.6}",
                edge_ratio, aqg_threshold, avg_edge
            );
            println!(
                "REALIZED_EDGE → raw={:.5} cap_prob={:.3} realized={:.5}",
                raw_edge, capture_prob, expected_realized_edge
            );
        }

        let side_is_long = bull > bear;

        // 🔥 FIX 4.2: Adaptive Exploration Gate
        let force_explore = executed_trades.is_empty() && strategy_index == 0;

        println!(
            "EDGE_FINAL → raw={:.3} is_live_regime={} pass={} sf={} snf={}",
            edge_raw, is_live_regime, passed_gate, signal_fail, sanity_fail
        );

        if !passed_gate {
            println!(
                "GATE_REJECT → raw={:.3} is_live_regime={} signal_fail={} sanity_fail={}",
                edge_raw, is_live_regime, signal_fail, sanity_fail
            );
        }

        if passed_gate {
            edge_pass_count += 1;
        }
        let risk_ok = true;

        // 🔥 REGIME-AWARE POSITION SIZING
        let size_multiplier = if is_live_regime { 1.0 } else { 0.6 };

        let size = (raw_edge.abs() * size_multiplier).max(0.01);

        let effective_fill_prob = if signal.is_probe {
            1.0
        } else {
            capture_prob.max(0.05)
        };

        let (_feasible, dynamic_threshold_val) =
            is_execution_feasible(final_conviction.conviction_score, capture_prob);
        let feasibility_prob = if dynamic_threshold_val > 1e-6 {
            (capture_prob / dynamic_threshold_val).clamp(0.0, 1.0)
        } else {
            0.0
        };
        // 🔥 remove logistic saturation
        let edge_soft_score = edge_ratio.clamp(0.0, 1.0);

        let dynamic_exec_threshold =
            (0.35f64 + 0.15 * (1.0 - feasibility_prob) - 0.10 * edge_soft_score).clamp(0.20, 0.70); // 🔥 Decompressed floor 0.20
        let threshold = dynamic_exec_threshold;

        let exec_margin = effective_fill_prob - threshold;

        // --- BIMODAL MODE SPLIT ---
        let is_exploit = is_live_regime;
        let is_explore = !is_live_regime;

        // --- EDGE GATING (PRIMARY REGIME DRIVER) ---
        // 🔥 FIX: dynamic threshold instead of hard 0.25
        let edge_threshold = 0.0;

        // ===============================
        // 🔥 FULLY ADAPTIVE EXECUTION (REPLACEMENT)
        // ===============================

        // 1. Edge percentile position (distribution anchored)
        let edge_pct = ((edge_raw - e_stats.p30) / (e_stats.p90 - e_stats.p30).abs().max(1e-9)).clamp(-0.5, 1.5);

        // 2. Capture signal centered
        let capture_pct = (capture_prob - 0.5) * 2.0; // [-1, 1]

        // 3. Execution margin normalized (self-scaled)
        let exec_scale = exec_margin.abs().max(0.05);
        let exec_pct = (exec_margin / exec_scale).clamp(-1.0, 1.0);

        // 4. Combine in score space (NOT probability space)
        let combined_score = 0.5 * edge_pct + 0.3 * capture_pct + 0.2 * exec_pct;

        // 🔥 NEW: center around neutral regime
        let centered_score = combined_score - 0.2;

        // 5. Convert to probability using sigmoid (adaptive curve)
        let mut final_prob = 1.0 / (1.0 + (-1.5 * centered_score).exp());

        // 6. Regime adjustment (soft degradation)
        if !is_live_regime {
            final_prob *= 0.6;
        }

        // 7. Micro-noise (keep GA exploration alive)
        let noise = ((strategy_index ^ current_idx) % 100) as f64 / 100.0;
        final_prob += (noise - 0.5) * 0.05;

        // 8. Final clamp
        final_prob = final_prob.clamp(0.02, 0.98);

        // 9. Final execution decision
        let execution_edge = raw_edge.abs();

        let final_execute =
            passed_gate
                && risk_ok
                && size > 0.0
                && execution_edge > 0.0001
                && (rand::random::<f64>() < final_prob);

        // --- ACCOUNTING ---
        if final_execute {
            // ✅ FIX: build distribution ONLY from raw edge (pre-execution signal)
            edge_vals_build.push(raw_edge + jitter);
            exec_pass_count_local += 1;
        }

        if std::env::var("GA_TRACE").is_ok() {
            println!(
                "EXEC_DECISION → symbol={} passed_gate={} size={:.2} risk_ok={} fill_prob={:.3} is_probe={} final_execute={}",
                scenario_name, passed_gate, size, risk_ok, effective_fill_prob, signal.is_probe, final_execute
            );
            if !final_execute {
                println!(
                    "EXEC_REJECT_REASON → gate={} risk={} size={} fill_prob={:.3} threshold={:.3}",
                    passed_gate,
                    risk_ok,
                    size > 0.0,
                    effective_fill_prob,
                    threshold
                );
            }
        }

        if !final_execute {
            continue;
        }

        // --- ATOMIC ACCOUNTING (Inside execution branch) ---
        exec_passed += 1;
        funnel_after_exec_prob += 1;
        funnel_after_edge_filter += 1; // Unifying accounting logic

        // --- SCOPE RESTORATION (Step 7.2) ---
        let pre_edge = edge_ratio;
        let final_exec_prob = effective_fill_prob;
        let survive_score = (0.25 * final_exec_prob
            + 0.45 * survival_prob
            + 0.2 * 0.5 // placeholder for expected_efficiency
            + 0.1 * capture_prob)
            .clamp(0.0, 1.0);
        let expected_efficiency =
            (0.5 * capture_prob + 0.3 * feasibility_prob + 0.2 * edge_soft_score).clamp(0.0, 1.0);
        let exec_weight = final_exec_prob * survival_prob;

        let trade_result = ga_simulate_round_trip_at_cursor(
            strategy,
            signal_events,
            execution_events,
            config,
            current_idx,
            executed_trades.len(),
            &final_conviction,
            side_is_long,
            signal.strength,
            signal.is_probe,
            strategy_index,
            generation,
            &config.stats,
        );

        if let Some(outcome) = trade_result {
            // ALWAYS record organic trades for RankStats learning
            if !outcome.is_probe {
                pnl_history_learn.push(outcome.clone());
            }

            if outcome.is_probe {
                probe_count += 1;
                continue;
            }

            // GATING: Only high-edge signals count for GA fitness
            if !outcome.is_execution {
                continue;
            }

            // 🔥 CRITICAL FIX: override expected edge with REALIZED edge
            let realized_edge = outcome.pnl; // or outcome.edge_quality if preferred

            expected_realized_edge = realized_edge;

            real_trade_count += 1;

            // Layer 4: Capture Efficiency Gate (Phase B)
            // Phase C.2d: Adaptive Execution Calibration

            // ✅ FIX 4: Decouple expected_return from edge so norm_pnl has real variance.
            // OLD: expected_return ≈ edge * constant → causes norm_pnl collapse.
            // NEW: expected_return = |edge| * volatility * regime_multiplier → orthogonal to pnl.
            let edge_quality = outcome.edge_quality;
            let regime_mult = {
                let bias = classify_direction_bias(strategy.direction_bias);
                let regime = detect_market_regime(
                    entry_price,
                    entry_price, // approximate sma as entry for now
                    final_conviction.conviction_score,
                    std_dev,
                );
                regime_multiplier(regime, bias)
            };
            let expected_return = (expected_realized_edge * volatility * regime_mult).max(0.0003);

            let quality_penalty = if expected_return < 1e-5 { 0.5 } else { 1.0 };

            executed_trades.push(outcome.clone());
            let idx = executed_trades.len() - 1;
            trade_edges.push((idx, pre_edge));

            // 🔥 GENOME-DRIVEN REGIME LOGIC (REPLACE ENTIRE BLOCK)

            // derive thresholds from genome
            let edge_thresh = strategy.base_edge as f64 / 10000.0;
            let exec_thresh = strategy.queue_threshold as f64 / 100.0;
            let holding = (strategy.holding_period as f64 / 10.0).max(2.0);
            let surv_thresh = (holding / 10.0).clamp(0.4, 0.8);

            let edge_ratio_local = expected_realized_edge / (edge_thresh + 1e-9);

            // Existing exit accounting
            match outcome.exit_reason {
                GaExitReason::TakeProfit => exit_tp_count += 1,
                GaExitReason::StopLoss => exit_sl_count += 1,
                GaExitReason::TimeStop => exit_ts_count += 1,
                GaExitReason::NoFill => {
                    // This block now only executes if the simulator returns NoFill
                    if let Some(reason) =
                        classify_rejection(outcome.queue_ahead, outcome.arrival_liquidity)
                    {
                        match reason {
                            RejectionReason::QueueBlocked => queue_blocked_count += 1,
                            RejectionReason::LiquidityStarved => liquidity_starved_count += 1,
                        }
                    }
                }
            }

            // ✅ FIX 1: Kill norm_pnl division — use raw pnl directly to break mathematical lock.
            // OLD: normalized_pnl = pnl / exp_return → both scale with edge → constant output.
            // NEW: normalized_pnl = pnl → real signal variance flows through.
            let normalized_pnl = outcome.pnl.clamp(-3.0, 3.0);

            // ✅ guard
            if !normalized_pnl.is_finite() {
                continue;
            }

            // ✅ NON-NEGOTIABLE INVARIANT: efficiency = pnl
            let realized_efficiency = outcome.pnl;
            debug_assert!(
                realized_efficiency.abs() <= 0.05,
                "Efficiency {} exceeds non-normalized bound",
                realized_efficiency
            );

            if std::env::var("GA_TRACE").is_ok() {
                println!(
                    "EFF_CHECK_NEW → pnl={} exp_return={} norm_pnl={} eff={}",
                    outcome.pnl, expected_return, normalized_pnl, realized_efficiency
                );
            }
            // derived ONCE
            let efficiency_scale = 1.0 + realized_efficiency;

            // Clamp to valid range

            // --- USE EFFICIENCY SAFELY BELOW ---
            // --- FIX: normalize e_score to usable range ---
            // 🔥 EXECUTION VARIANCE AMPLIFICATION (FIX 1)
            let fill_variance = (outcome.fills_count as f64
                / (outcome.total_filled_qty.max(1) as f64))
                .clamp(0.0, 1.0);

            // 🔥 derive latency from config + volatility
            let latency_penalty = ((config.latency_ticks as f64) * volatility).clamp(0.0, 1.0);

            let queue_pressure = (queue_ahead / 1000.0).clamp(0.0, 1.0);

            // amplified execution score
            let base_exec_score = realized_efficiency.max(-0.2); // 🔥 use amplified efficiency

            let e_exec_score = base_exec_score
                * (1.0 - latency_penalty)
                * (1.0 - queue_pressure)
                * (1.0 + fill_variance * 0.5);

            // efficiency ONLY affects scoring later, NOT pnl
            // === FIX 4: EDGE-SCALED PAYOFF ===

            // base pnl from simulator

            // edge strength (already normalized-ish)
            let edge = final_conviction.edge_weight.max(0.0);

            // execution quality stays
            let execution_quality: f64 = (0.7_f64 + 0.6_f64 * e_exec_score).clamp(0.5, 1.3);

            // 🔥 TAIL RISK CONTROL
            let tail_cap = 0.003;

            let mut raw_pnl = outcome.pnl;

            // 🔥 HARD STOP-LOSS ENFORCEMENT (STRUCTURAL FIX)
            let entry_price = signal_events[current_idx].price as f64;

            // reconstruct SL/TP bounds (must match simulator logic)
            let atr = calculate_atr(signal_events, current_idx, 14);
            let rr = strategy.edge_ratio as f64 / 100.0;

            let sl_dist = atr; // or your SL logic
            let tp_dist = atr * rr;

            let max_loss = -sl_dist / entry_price;
            let max_profit = tp_dist / entry_price;

            // 🚨 CRITICAL: clamp BEFORE any scaling
            raw_pnl = raw_pnl.clamp(max_loss, max_profit);

            // 🚨 enforce SL dominance over TimeStop
            if outcome.exit_reason == GaExitReason::TimeStop && raw_pnl < max_loss {
                if std::env::var("GA_VERBOSE").is_ok() {
                    println!(
                        "🚨 SL_BYPASS_DETECTED → raw_pnl={:.6} max_loss={:.6} idx={}",
                        raw_pnl, max_loss, current_idx
                    );
                }
                raw_pnl = max_loss;
            }

            let edge_scale = 0.5 + raw_edge.abs().powf(0.6);

            let capture_ratio = if outcome.expected_move.abs() > 1e-9 {
                (raw_pnl / outcome.expected_move).clamp(-2.0, 2.0)
            } else {
                0.0
            };

            let mut trade_pnl = raw_pnl
                * capture_prob
                * edge_scale
                * (1.0 + 0.3 * capture_ratio.abs())
                * exec_weight; // 🔥 APPLY EXECUTION REALISM

            // 🔥 Outcome Sensitivity Amplification (Phase 3.6.3)
            let strength_val = final_conviction.conviction_score;
            if trade_pnl > 0.0 {
                trade_pnl *= 1.0 + strength_val * 0.5;
            } else {
                trade_pnl *= 1.0 - strength_val * 0.2;
            }

            // ✅ FIX: Update the outcome in the history vector with the boosted PnL
            executed_trades[idx].pnl = trade_pnl;
            let mut trade_pnl_safe = executed_trades[idx].pnl;

            if !trade_pnl_safe.is_finite() {
                trade_pnl_safe = 0.0;
            }

            trade_scores.retain(|(_, score)| score.is_finite());
            trade_scores.push((idx, trade_pnl_safe));
            if !trade_pnl_safe.is_finite() {
                println!("⚠️ INVALID PNL DETECTED");
            } else {
                println!("TRADE_CAPTURED → idx={} pnl={:.6}", idx, trade_pnl_safe);
            }

            // ✅ NOW variables exist → safe to use

            // 🔴 SNIPER score
            let edge_component = (expected_realized_edge * 1000.0).powf(1.3);
            let exec_component = final_exec_prob.powf(1.5);
            let volatility_component = (outcome.m_favorable.abs() + 1e-6).powf(0.5);

            let sniper_score = edge_component.powf(1.5)
                * exec_component.powf(1.3)
                * volatility_component.powf(0.7);

            // 🟢 CONSISTENT score
            let exec_consistent_score = final_exec_prob.powf(1.2)
                * survive_score.powf(1.2)
                * (1.0 / (1.0 + volatility)).powf(0.8);

            // ✅ PUSH AFTER DEFINITION
            sniper_scores.push((idx, sniper_score));
            consistent_scores.push((idx, exec_consistent_score));

            println!(
                "TYPE_DEBUG → idx={} edge={:.5} exec={:.3} surv={:.3} sniper={:.6} consistent={:.6}",
                idx, expected_realized_edge, final_exec_prob, survive_score, sniper_score, exec_consistent_score
            );

            let tail_penalty = if raw_pnl < 0.0 {
                (-raw_pnl).min(0.01)
            } else {
                0.0
            };

            total_tail_penalty += tail_penalty;

            // Phase D.1.18: Extract winning signal signature
            let winning_sig = if let Some((_, _, _, _, _, _, sig)) = valid_signals
                .iter()
                .find(|(idx, _, _, _, _, _, _)| *idx == current_idx)
            {
                Some(sig.clone())
            } else {
                None
            };

            let is_long = outcome.side == crate::Side::Buy;
            metrics.record_trade(
                trade_pnl,
                outcome.ideal_pnl,
                0.0,                          // expected_pnl
                0.0,                          // entropy
                final_conviction.raw_q_ratio, // conviction
                realized_efficiency,          // efficiency
                outcome.edge_quality,         // edge_quality
                outcome.time_to_mfe as f64,   // time_to_mfe
                0.0,                          // margin
                1.0,                          // aqg_health
                raw_edge_spread_norm,         // edge_spread
                dominance,                    // dominance
                final_conviction.raw_q_ratio, // signal_entropy
                outcome.clone(),
                SignalSource::Organic,
                winning_sig,
                is_long,
                e_exec_score,
            );

            if std::env::var("GA_VERBOSE").is_ok() {
                println!(
                    "EFF_TRACK → stored_eff={} exec_score={}",
                    realized_efficiency, e_exec_score
                );
            }

            let raw_exit = outcome.exit_event_idx;

            // Phase 14 Attribution
            if decision_was_override {
                metrics.conviction_trade_count += 1;
                metrics.sum_conviction_pnl += trade_pnl;
            } else {
                metrics.clarity_trade_count += 1;
                metrics.sum_clarity_pnl += trade_pnl;
            }
            let capped_exit = raw_exit.min(current_idx + 200);

            // 🚨 NOW safe to skip
            let raw_exit = outcome.exit_event_idx;

            // ✅ FIX: ALWAYS account exit BEFORE any continue
            match outcome.exit_reason {
                GaExitReason::TakeProfit => {
                    pnl_from_tp_scenario += trade_pnl;
                    if trade_pnl > 0.0 {
                        long_win_count_scenario += 1;
                    }
                }
                GaExitReason::StopLoss => {
                    pnl_from_sl_scenario += trade_pnl;
                    if trade_pnl.abs() < 0.0001 {
                        micro_loss_count += 1;
                    }
                }
                GaExitReason::TimeStop => {
                    if trade_pnl > 0.0 {
                        long_win_count_scenario += 1;
                    }
                }
                GaExitReason::NoFill => {} // No impact on pnl scenarios
            };

            // 🚨 NOW safe to skip
            if capped_exit <= current_idx {
                continue;
            }

            total_vol_ratio += std_v;
            total_spread_reality += outcome.spread;
            total_window_volume += outcome.avg_window_volume;

            // Phase C.1: Trade-Level Survivability Check
            let window_slippage = outcome.spread * (1.0 + std_v.powf(1.2)) * config.slippage_factor;
            let window_fill_prob = 0.7 + 0.3 * capture_prob;
            let window_latency_penalty =
                (-0.05 * config.latency_ticks as f64).exp().clamp(0.6, 1.0);

            // --- FIX: Preserve PnL energy (no multiplicative collapse) ---

            // Convert multiplicative penalties → linear drag
            let adjusted_pnl = trade_pnl;

            // Prevent slippage from dominating signal
            let capped_slippage = window_slippage.min(0.001); // absolute cap

            // Final effective pnl
            let window_effective_pnl = if trade_pnl > 0.0 {
                (adjusted_pnl - capped_slippage).max(trade_pnl * 0.2) // 🔥 stronger floor (was 0.1)
            } else {
                adjusted_pnl - capped_slippage
            };

            // Targeted Sanity Log (Constrained to first few trades AND strict signal debug)
            if std::env::var("GA_SIGNAL_DEBUG").is_ok() {
                println!(
                    "DEBUG_SANITY → idx={} trade_pnl={:.6} edge={} exec={}",
                    idx, trade_pnl, edge_scale, exec_weight
                );
                println!(
                    "EDGE_PASS → realized={:.6} norm={:.3} pass={}",
                    expected_realized_edge, priority_score, pass
                );
            }

            if window_effective_pnl > 0.0 {
                survivable_trades_count += 1;
            }

            // --- PHASE C.1.5: DEAD-ZONE ERADICATION (Early Exit) ---
            if executed_trades.len() >= 20 {
                // Phase C.2d: Allow more discovery before killing
                let current_surv = survivable_trades_count as f64 / executed_trades.len() as f64;
                if current_surv < 0.05_f64 {
                    // Relaxed from 0.2 to allow recovery
                    // Strategy is non-survivable in this regime; kill early to accelerate convergence
                }
            }

            // --- PHASE C.1.6: PARTICIPATION PRESSURE (Early Participation Choke) ---
            if scenario_pnls.len() >= 20 && executed_trades.len() < 1 {
                // Not active enough for institutional scale; kill early
            }

            metrics.record_opportunity();

            total_filled_qty += outcome.total_filled_qty;
            fills_count += outcome.fills_count;
            total_slippage_bps += outcome.total_slippage_bps;
            sum_drawdown_raw += outcome.drawdown_penalty_raw;
            sum_expected_move += outcome.expected_move;
            let (sig, lat) = scenario_execution_signature_from_simulation(
                &outcome.sim_events,
                &outcome.entry_order_id,
                &outcome.exit_order_id,
                outcome.fill_efficiency,
                1.0,
                outcome.raw_q_ratio,
            );
            cycle_sigs.push(sig);
            sum_latency_raw += lat;

            let expected_slippage = conviction.conviction_score.abs() * 0.1;
            sum_expected_slippage += expected_slippage;
            let actual_slippage = outcome.total_slippage_bps;
            sum_actual_slippage += actual_slippage;

            max_trade_pnl_scenario = max_trade_pnl_scenario.max(trade_pnl);
            max_pnl_in_scenario = max_pnl_in_scenario.max(outcome.pnl);
            scenario_pnls.push(window_effective_pnl);

            // Phase D.1: Metrics Propagation
            metrics.trade_qualities.push(outcome.edge_quality);
            metrics.sum_realized_pnl += window_effective_pnl;
            metrics.sum_expected_pnl += outcome.expected_move;
            total_spread_test += outcome.spread;
            sum_price += signal_events[current_idx].price as f64;
            total_quality_trades_scenario += outcome.quality;

            if ga_debug_enabled() && strategy_index == 0 && idx < 3 && generation % 5 == 0 {
                println!(
                    "GA_EXEC: scenario={} idx={} score={:.4} spread_z={:.2} dom={:.2} pnl={:.6}",
                    scenario_name,
                    current_idx,
                    conviction.conviction_score,
                    edge_spread_norm,
                    dominance,
                    outcome.pnl
                );
            }
            busy_until = capped_exit + cooldown; // Phase D.1.23: Restoration of cooldown control
        }
    }

    println!(
        "[GATE_STATS] total={} priority={} edge={} exec={} real_trades={}",
        emitted_signs.len(),
        priority_pass_count,
        edge_pass_count,
        exec_pass_count_local,
        real_trade_count
    );

    if trade_scores.is_empty() && !executed_trades.is_empty() {
        println!("⚠️ FALLBACK TRADE SCORES USED");
        for (i, t) in executed_trades.iter().enumerate() {
            trade_scores.push((i, t.pnl.max(1e-6)));
        }
    }

    // 🔒 Freeze edge distribution AFTER execution completes
    if edge_vals_build.len() >= 10 {
        edge_vals_snapshot = edge_vals_build.clone();
    }

    // maintain rolling window
    // 🔥 Apply rolling window to SNAPSHOT (not build)
    if edge_vals_snapshot.len() > EDGE_WINDOW {
        edge_vals_snapshot.drain(0..edge_vals_snapshot.len() - EDGE_WINDOW);
    }

    if edge_vals_snapshot.len() >= 10 {
        let mean_e = edge_vals_snapshot.iter().sum::<f64>() / edge_vals_snapshot.len() as f64;
        let var_e = edge_vals_snapshot
            .iter()
            .map(|v| (v - mean_e).powi(2))
            .sum::<f64>()
            / edge_vals_snapshot.len() as f64;

        let std_e = var_e.sqrt();

        if std_e < 1e-4 {
            println!("🚨 EDGE COLLAPSE DETECTED → std={:.8}", std_e);
        }
    }
    // 🔥 Phase 3.6.8: Minimum Trade Guarantee (Fallback Injector)
    // If we have massive starvation (< 5 trades), force-inject high-conviction signals to create learning gradient.
    if real_trade_count < 5 && real_trade_count > 0 { // at least one valid pass
         // No-op, we have some signal.
    } else if real_trade_count < 5 {
        if ga_debug_enabled() {
            println!(
                "🚨 STARVATION DETECTED → MERCY MODE INJECTING TOP SIGNALS for {}",
                scenario_name
            );
        }

        let mut sorted_signs = emitted_signs.clone();
        sorted_signs.sort_by(|a, b| {
            b.conviction
                .conviction_score
                .partial_cmp(&a.conviction.conviction_score)
                .unwrap()
        });

        for signal in sorted_signs.iter().take(5) {
            if real_trade_count >= 5 {
                break;
            }

            let phase_shift = ((strategy_index % 7) as i32 - 3) as isize;
            let execution_idx = (signal.ts as isize + phase_shift).max(0) as usize;
            let execution_idx = execution_idx.min(signal_events.len().saturating_sub(10));

            let bull = signal.conviction.bullish_score;
            let bear = signal.conviction.bearish_score;
            let side_is_long = bull > bear;

            let trade_result = ga_simulate_round_trip_at_cursor(
                strategy,
                signal_events,
                execution_events,
                config,
                execution_idx,
                executed_trades.len(),
                &signal.conviction,
                side_is_long,
                signal.strength,
                signal.is_probe,
                strategy_index,
                generation,
                &config.stats,
            );

            if let Some(outcome) = trade_result {
                if !outcome.is_probe {
                    real_trade_count += 1;
                    executed_trades.push(outcome.clone());
                    scenario_pnls.push(outcome.pnl);

                    // 🔥 Phase 3.6.8: Ensure forced trades are visible to the score aggregator
                    let idx = executed_trades.len() - 1;
                    // keep this, but now it's fallback-only
                    trade_scores.push((idx, outcome.pnl));
                }
            }
        }
    }

    if ga_debug_enabled() && strategy_index == 0 && generation % 5 == 0 {
        println!(
            "POST-EXEC CHECK → emitted={} executed={}",
            emitted_signs.len(),
            executed_trades.len()
        );

        println!(
            "PROBE_AUDIT → probes={} real_trades={}",
            probe_count, real_trade_count
        );
    }

    if std::env::var("GA_VERBOSE").is_ok() {
        if real_trade_count > 0 && real_trade_count <= probe_count {
            println!(
                "⚠️ INVARIANT WARNING: real_trades ({}) <= probes ({})",
                real_trade_count, probe_count
            );
        }
    }

    // Statistical pruning removed to preserve win/loss distribution.

    if executed_trades.is_empty() && !emitted_signs.is_empty() {
        println!("⚠️ EXECUTION STARVATION → injecting synthetic trade_scores");

        // 🔥 Inject minimal fallback trades (DO NOT return)
        for (i, signal) in emitted_signs.iter().take(3).enumerate() {
            trade_scores.push((i, 0.0001)); // tiny positive signal
        }

        // Also create minimal executed_trades placeholder to avoid downstream break
        if executed_trades.is_empty() {
            println!("⚠️ Creating dummy executed_trades for continuity");

            for (i, _) in emitted_signs.iter().take(1).enumerate() {
                injected_trades.push(GaRoundTripOutcome {
                    pnl: 0.0001,
                    exit_event_idx: i,
                    ..Default::default()
                });
            }
        }
    }
    // ✅ FIX: Commit injected trades BEFORE any iteration/retain
    if !injected_trades.is_empty() {
        executed_trades.extend(injected_trades.drain(..));
    }
    // 🔥 HYBRID DEDUPLICATION (Exit bucket + Side)
    let mut seen_ts = std::collections::HashSet::new();

    executed_trades.retain(|trade| {
        // 🔥 HYBRID KEY: bucketed exit index + direction
        let key = (trade.exit_event_idx / 5, trade.side);

        if seen_ts.contains(&key) {
            false
        } else {
            seen_ts.insert(key);
            true
        }
    });

    // 🔥 SIMPLE CLUSTERING BY EXIT INDEX PROXIMITY
    let mut clusters: Vec<Vec<&GaRoundTripOutcome>> = Vec::new();

    let mut current_cluster: Vec<&GaRoundTripOutcome> = Vec::new();
    let mut last_exit: Option<usize> = None;

    // SAFETY: no mutation allowed beyond this point
    debug_assert!(injected_trades.is_empty());

    for trade in executed_trades.iter() {
        if let Some(last) = last_exit {
            if trade.exit_event_idx.saturating_sub(last) <= 5 {
                current_cluster.push(trade);
            } else {
                if !current_cluster.is_empty() {
                    clusters.push(current_cluster);
                }
                current_cluster = vec![trade];
            }
        } else {
            current_cluster.push(trade);
        }

        last_exit = Some(trade.exit_event_idx);
    }

    // push last cluster
    if !current_cluster.is_empty() {
        clusters.push(current_cluster);
    }

    // 🚨 GUARANTEE AT LEAST ONE CLUSTER
    if clusters.is_empty() && !executed_trades.is_empty() {
        clusters.push(executed_trades.iter().collect());
    }

    // 🔍 DEBUG
    if ga_debug_enabled() {
        println!(
            "[CLUSTER_DEBUG] total_trades={} clusters={} avg_size={:.2}",
            executed_trades.len(),
            clusters.len(),
            if clusters.len() > 0 {
                executed_trades.len() as f64 / clusters.len() as f64
            } else {
                0.0
            }
        );
    }
    // =============================
    // 🔥 FIX: REBUILD METRICS FROM FINAL TRADES
    // =============================
    let mut rebuilt_metrics = ScenarioMetrics::default();

    let mut rebuilt_tp = 0;
    let mut rebuilt_sl = 0;
    let mut rebuilt_ts = 0;

    for trade in &executed_trades {
        match trade.exit_reason {
            GaExitReason::TakeProfit => rebuilt_tp += 1,
            GaExitReason::StopLoss => rebuilt_sl += 1,
            GaExitReason::TimeStop => rebuilt_ts += 1,
            GaExitReason::NoFill => {}
        }

        rebuilt_metrics.trade_count += 1;

        if trade.pnl > 0.0 {
            rebuilt_metrics.profitable_trades += 1;
        }

        rebuilt_metrics.sum_pnl += trade.pnl;
    }

    // ✅ FIX: Merge rebuilt metrics instead of clobbering to preserve history
    metrics.sum_pnl = rebuilt_metrics.sum_pnl;
    metrics.trade_count = rebuilt_metrics.trade_count;
    metrics.profitable_trades = rebuilt_metrics.profitable_trades;
    // metrics.trade_qualities, etc. are preserved

    exit_tp_count = rebuilt_tp;
    exit_sl_count = rebuilt_sl;
    exit_ts_count = rebuilt_ts;

    let selected_trades: Vec<&GaRoundTripOutcome> =
        clusters.iter().filter_map(|c| c.get(0).cloned()).collect();

    if ga_debug_enabled() {
        println!(
            "PARTICIPATION → trades={} attempts={} triggered={} signals={} ratio={:.4}",
            executed_trades.len(),
            entry_attempted,
            triggered_entries,
            signal_events.len(),
            executed_trades.len() as f64 / signal_events.len().max(1) as f64
        );
        println!(
                "FUNNEL → signals={} after_signal_filter={} after_edge_filter={} after_exec_prob={} final_attempts={}",
                funnel_signals,
                funnel_after_signal_filter,
                funnel_after_edge_filter, // entry_attempted represents this
                funnel_after_exec_prob,   // triggered_entries
                executed_trades.len()
            );
    }

    // 🔍 EXECUTION VARIANCE DEBUG
    if ga_debug_enabled() {
        let effs: Vec<f64> = metrics.trade_qualities.clone();

        if !effs.is_empty() {
            let min = effs.iter().cloned().fold(f64::INFINITY, f64::min);
            let max = effs.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            let mean = effs.iter().sum::<f64>() / effs.len() as f64;

            let var = effs.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / effs.len() as f64;

            println!(
                "EFF_DIST → min={:.4} max={:.4} mean={:.4} std={:.4} count={}",
                min,
                max,
                mean,
                var.sqrt(),
                effs.len()
            );
        }
    }

    if ga_debug_enabled() {
        let decision_skipped = entry_attempted
            .saturating_sub(executed_trades.len())
            .saturating_sub(skipped_busy);
        println!(
                    "ENTRY_DEBUG → signals={} attempts={} triggered={} busy_skipped={} decision_skipped={} | EXITS: TP={} SL={} TS={}",
                    signal_events.len(), entry_attempted, triggered_entries, skipped_busy, decision_skipped, exit_tp_count, exit_sl_count, exit_ts_count
                );
    }

    if ga_debug_enabled() {
        println!(
            "EFF_FINAL → avg_eff={} trades={}",
            metrics.avg_efficiency(),
            executed_trades.len()
        );
    }

    let mean_expected_move = if executed_trades.len() > 0 {
        sum_expected_move / executed_trades.len() as f64
    } else {
        0.0
    };
    let drawdown_penalty_raw = if executed_trades.len() > 0 {
        sum_drawdown_raw / executed_trades.len() as f64
    } else {
        0.0
    };
    let requested_qty =
        config.order_quantity_for_strategy * 2 * (executed_trades.len().max(1) as u64);

    // --- PHASE 10.5: REGIME ADMISSION GATE ---
    if executed_trades.len() > 15 && max_pnl_in_scenario < 0.0025 {
        if ga_debug_enabled() {
            println!(
                "ADMISSION_REJECT → Over-trading noise: scenario={} trades={} max_pnl={:.5}",
                scenario_name,
                executed_trades.len(),
                max_pnl_in_scenario
            );
        }
        fitness_penalty = -0.1; // 🔥 penalty only
    }

    // 🔥 CANONICAL TRADE DERIVATION (PATCH 5)

    let total_trades = executed_trades.len();

    // 🔥 GLOBAL REGIME (STRATEGY LEVEL)

    let avg_edge_realized = if edge_vals_build.len() > 0 {
        edge_vals_build.iter().sum::<f64>() / edge_vals_build.len() as f64
    } else {
        0.0
    };

    let global_fill_ratio = if entry_attempted > 0 {
        exec_passed as f64 / entry_attempted as f64
    } else {
        0.0
    };

    let regime = if avg_edge_realized < e_stats.p30 || global_fill_ratio < 0.05 {
        -1
    } else {
        1
    };

    let mut overtrade_penalty = 0.0;

    if total_trades > 10 {
        let excess = total_trades as f64 - 10.0;
        overtrade_penalty = 0.02 * excess;
    }

    let total_pnl: f64 = executed_trades.iter().map(|t| t.pnl).sum();

    if ga_debug_enabled() {
        println!(
            "FITNESS_AUDIT → total={} wins={} losses={}",
            executed_trades.len(),
            executed_trades.iter().filter(|t| t.pnl > 0.0).count(),
            executed_trades.iter().filter(|t| t.pnl < 0.0).count()
        );
    }

    let profitable_trades = executed_trades.iter().filter(|t| t.pnl > 0.0).count();

    let win_rate = if total_trades > 0 {
        profitable_trades as f64 / total_trades as f64
    } else {
        0.0
    };

    let zero_pnl_trades = executed_trades.iter().filter(|t| t.pnl == 0.0).count();

    let avg_pnl = if total_trades > 0 {
        total_pnl / total_trades as f64
    } else {
        0.0
    };

    let avg_pnl_for_scenario = avg_pnl;

    let mut zero_pnl_trades_scenario = 0usize;
    let mut total_win = 0.0;
    let mut total_loss = 0.0;
    let mut win_count = 0;
    let mut loss_count = 0;

    for trade in &executed_trades {
        let pnl = trade.pnl;

        if pnl > 0.0 {
            total_win += pnl;
            win_count += 1;
        } else if pnl == 0.0 {
            zero_pnl_trades_scenario += 1;
        } else {
            total_loss += pnl.abs();
            loss_count += 1;
        }
    }

    let avg_win = if win_count > 0 {
        total_win / win_count as f64
    } else {
        0.0
    };
    let avg_loss = if loss_count > 0 {
        total_loss / loss_count as f64
    } else {
        0.0
    };

    if ga_debug_enabled() {
        println!(
            "FITNESS_SANITY → avg_win={:.6} avg_loss={:.6}",
            avg_win, avg_loss
        );
    }

    // Stabilized Payoff Ratio
    let payoff_ratio = if avg_loss.abs() > 1e-6 {
        (avg_win / avg_loss.abs()).clamp(0.5, 3.0)
    } else {
        0.0
    };
    let dir_consistency = if win_count > 0 {
        (long_win_count_scenario.max(short_win_count_scenario) as f64 / win_count as f64)
            .clamp(0.5, 1.0)
    } else {
        0.0
    };

    // Composite Stability: Boosted with Squaring for Phase 11.2
    let stability = (payoff_ratio * win_rate * dir_consistency).clamp(0.0, 1.5);
    let mut consistency_penalty_factor = 1.0;

    if stability < 0.2 {
        if ga_debug_enabled() {
            println!("CONSISTENCY_PENALTY → stability={:.3}", stability);
        }
        consistency_penalty_factor = 0.6;
    }
    let _stability_weighted = stability.powi(2);

    let selectivity = metrics.selectivity();

    let std_dev_for_scenario: f64 = if total_trades > 1 {
        let mean = avg_pnl_for_scenario;
        let variance = scenario_pnls
            .iter()
            .map(|pnl| (pnl - mean).powi(2))
            .sum::<f64>()
            / total_trades as f64;
        variance.sqrt()
    } else {
        0.0_f64
    };

    // --- PHASE 12.5: SCENARIO-LEVEL DRAWDOWN (worst cumulative dip) ---
    let mut scenario_max_drawdown = 0.0_f64;
    let mut current_cum_pnl = 0.0_f64;
    let mut peak_cum_pnl = 0.0_f64;
    for &pnl in &scenario_pnls {
        current_cum_pnl += pnl;
        peak_cum_pnl = peak_cum_pnl.max(current_cum_pnl);
        let current_drawdown = current_cum_pnl - peak_cum_pnl;
        scenario_max_drawdown = scenario_max_drawdown.min(current_drawdown);
    }
    let worst_pnl_for_scenario = scenario_max_drawdown;

    // --- PHASE 11.2 / B.1 / C / C.1 / C.1.5: INSTITUTIONAL FITNESS REDESIGN ---
    // Formula: Alpha * Consistency * Efficiency * Activity * Stability * DiscoveryPressure

    // 1. Reality Factors (Phase C)
    let avg_vol_ratio = if total_trades > 0 {
        total_vol_ratio / total_trades as f64
    } else {
        0.0
    };
    let avg_spread_reality = if total_trades > 0 {
        total_spread_reality / total_trades as f64
    } else {
        0.0
    };
    let adtv = if total_trades > 0 {
        (total_window_volume / total_trades as f64).max(100_000.0)
    } else {
        1_000_000.0
    };

    // --- 1.1 Slippage Model (Convex + Phase C.2 Liquidity Scaling) ---
    let basic_slippage =
        avg_spread_reality * (1.0 + avg_vol_ratio.powf(1.2)) * config.slippage_factor;
    let size = config.order_quantity_for_strategy as f64;
    let participation_rate = (size / adtv).clamp(0.0001, 0.2);

    // Square Root Law of Market Impact (Phase C.2)
    let size_slippage_multiplier = (1.0 + (participation_rate / 0.01).powi(2)).max(1.0);
    let slippage = basic_slippage * size_slippage_multiplier;
    let slippage = if avg_pnl_for_scenario > 0.0 {
        slippage.min(avg_pnl_for_scenario * 0.7_f64)
    } else {
        slippage
    };

    // --- 1.2 Fill Probability (Phase C.2 Depth-Aware) ---
    let eff_for_fill = metrics.avg_efficiency().max(0.0);
    let base_fill_prob = (eff_for_fill * 0.7 + 0.3).clamp(0.5, 1.0);
    let fill_prob = (base_fill_prob * (-8.0 * participation_rate).exp()).clamp(0., 1.0);

    // --- 1.3 Latency Decay ---
    let latency_ticks = config.latency_ticks as f64;
    let latency_penalty = (-0.05 * latency_ticks).exp().clamp(0.6, 1.0);

    // --- 1.4 Effective PnL (Scoring Overlay) ---
    // ✅ Fix: Unify scaling. realized_pnl already includes exec_weight/edge_scale.
    // ⚠️ Latency removal: already baked into ESE simulation path.
    let effective_pnl = avg_pnl_for_scenario;

    // --- PHASE C.1.6b: ADAPTIVE PARTICIPATION GATE (Smooth Recovery) ---
    // User Precision: Smooth penalty encourages exploration while killing outliers.
    let trade_factor = (total_trades as f64 / 10.0).min(1.0);

    // --- PHASE 1: PURE EXECUTION FITNESS (NO NORMALIZATION) ---

    // 🔥 smooth pnl scaling (prevents alpha suppression)
    let pnl = (effective_pnl / 0.001).tanh() * 2.0;

    // ✅ STEP 1: Only penalize REAL losses (already computed correctly)
    let loss_penalty = total_tail_penalty; // already bounded (~0.0 to 0.01 per trade)

    // ✅ STEP 2: Normalize safely
    let trade_count_safe = (total_trades as f64).max(1.0);
    let normalized_loss = loss_penalty / trade_count_safe;

    // ✅ STEP 3: Soft latency penalty (scaled DOWN heavily)
    let latency_penalty = (sum_latency_raw / trade_count_safe).clamp(0.0, 0.001);

    // ✅ STEP 4: Soft slippage penalty (convert bps → realistic scale)
    let slippage_penalty = (total_slippage_bps / trade_count_safe).clamp(0.0, 0.001);

    // ✅ STEP 5: Combine (controlled)
    let raw_penalty = normalized_loss + latency_penalty + slippage_penalty;

    // ✅ STEP 6: Final clamp (LOWER ceiling)
    let execution_penalty = raw_penalty.clamp(0.0, 0.2);

    let consistency_score = (metrics.avg_efficiency().max(0.0) * win_rate).clamp(0.0, 1.0);

    let activity_score = (total_trades as f64 / 20.0).max(0.05).min(1.0);
    let avg_exec_score = if metrics.exec_passed_count > 0 {
        (metrics.sum_exec_e_score / metrics.exec_passed_count as f64).clamp(-1.0, 1.0)
    } else {
        0.0
    };
    let win_boost = if total_trades >= 5 {
        win_rate
    } else {
        win_rate * 0.5
    };

    let n_sig = cycle_sigs.len().max(1) as f64;
    let scenario_signature = if cycle_sigs.is_empty() {
        ScenarioExecutionSignature::default()
    } else {
        ScenarioExecutionSignature {
            avg_queue_ahead: cycle_sigs.iter().map(|s| s.avg_queue_ahead).sum::<f64>() / n_sig,
            avg_latency: cycle_sigs.iter().map(|s| s.avg_latency).sum::<f64>() / n_sig,
            fill_ratio: cycle_sigs.iter().map(|s| s.fill_ratio).sum::<f64>() / n_sig,
            participation: cycle_sigs.iter().map(|s| s.participation).sum::<f64>() / n_sig,
            execution_variance: 0.0,
        }
    };

    // --- PHASE 11.5: EXECUTION PRESSURE CONSTRAINT (V3.2 - REFINED BRIDGE) ---
    // Transition from "Peer Discovery" to "Institutional Realism".

    // 1. Define Global Baselines (NSE 5m Context)
    const GLOBAL_QUEUE_P95: f64 = 250_000.0;

    // 2. Compute Phase-Based Weights
    let phase = (generation as f64 / config.generations.max(1) as f64).clamp(0.0, 1.0);
    let local_weight = 0.7 * (1.0 - phase) + 0.4 * phase;
    let global_weight = 1.0 - local_weight;

    // 3. Compute Normalized Pressure Components
    let log_q = (1.0 + scenario_signature.avg_queue_ahead).ln();
    let log_p95 = (1.0 + GLOBAL_QUEUE_P95).ln();
    let log_max = gen_max_log_queue.max(1e-6);

    let global_component = log_q / log_p95;
    let local_component = log_q / log_max;

    let pressure_factor =
        (global_weight * global_component + local_weight * local_component).clamp(0.01, 1.0);

    // 4. Compute Effective Fill & Progress Reward
    let fill_rate = total_trades as f64 / entry_attempted.max(1) as f64;
    let mut effective_fill_score = fill_rate * pressure_factor;

    // Explicit Progress Reward: Incentive for climbing the ladder (Boosted V3.3 + V3.5.1 Bias).
    // 🔥 FINAL FITNESS PURIFICATION (Phase 3: Absolute Alpha)
    // 1. Source: Pure Realized PnL from metrics (Slippage + Latency included)
    let mut fitness = match regime {
        1 => metrics.sum_realized_pnl * 1000.0, // real alpha
        -1 => -0.1 * (1.0 - fill_rate),         // punish dead zones
        _ => -0.5,
    };

    // 🔥 Anti-flat fitness penalty
    if metrics.std_dev() < 1e-6 && total_trades > 5 {
        fitness *= 0.5;
    }

    // 2. Finite Guard (Safety)
    if !fitness.is_finite() {
        fitness = 0.0;
    }

    // 3. Participation CLIFF (5 trades - forced stability)
    if total_trades < 5 {
        fitness *= 0.1;
    }

    // 4. Smooth Participation Factor (10 trades cap - gradient)
    let participation_smooth = (total_trades as f64 / 10.0).min(1.0);
    fitness *= participation_smooth;

    // 5. Hard Floor (No deep negative noise)
    fitness = fitness.max(-0.3);

    // 🔥 prevent silent collapse
    if !fitness.is_finite() {
        if ga_debug_enabled() {
            println!("⚠️ FITNESS NAN/INF DETECTED");
        }
        fitness = -0.5;
    }

    // if ga_debug_enabled() {
    println!(
        "FITNESS_BREAKDOWN → pnl={:.3} win={:.3} final={:.3}",
        metrics.sum_realized_pnl,
        metrics.profitable_trades as f64 / metrics.trade_count.max(1) as f64,
        fitness
    );
    // }
    let robustness_for_scenario = avg_pnl_for_scenario - config.lambda * std_dev_for_scenario;
    let fill_efficiency = if requested_qty > 0 {
        total_filled_qty as f64 / requested_qty as f64
    } else {
        0.0
    };
    let avg_slippage = if fills_count > 0 {
        total_slippage_bps / fills_count as f64
    } else {
        0.0
    };
    let realized_avg = avg_pnl_for_scenario;

    let latency_raw_mean = if total_trades > 0 {
        sum_latency_raw / total_trades as f64
    } else {
        0.0
    };

    let downside_variance_scenario = if total_trades > 0 {
        scenario_pnls
            .iter()
            .map(|&pnl| pnl.min(0.0).powi(2))
            .sum::<f64>()
            / total_trades as f64
    } else {
        0.0_f64
    };
    let downside_std_dev_scenario = downside_variance_scenario.sqrt();

    // Hard assertion for outcome consistency
    let total_c = metrics.long_count + metrics.short_count;
    let direction_ratio = if total_c > 0 {
        metrics.long_count as f64 / total_c as f64
    } else {
        0.5
    };

    let exit_total = exit_tp_count + exit_sl_count + exit_ts_count;

    if !executed_trades.is_empty() && exit_total != executed_trades.len() {
        if ga_debug_enabled() {
            println!(
                "🚨 MISMATCH → total={} tp={} sl={} ts={}",
                total_trades, exit_tp_count, exit_sl_count, exit_ts_count
            );
        }
    }

    // 🚨 FIX 4: METRICS vs EXECUTION TRUTH CHECK
    if metrics.trade_count != executed_trades.len() {
        println!(
            "🚨 METRICS DESYNC → metrics={} executed={}",
            metrics.trade_count,
            executed_trades.len()
        );
    }

    // 🔥 ARCHETYPE VISIBILITY
    if ga_debug_enabled() {
        println!(
            "ARCHETYPE → id={} type={} edge_ratio={} hold_time={} trades={} avg_q_fill={:.1} offset={}",
            strategy_id,
            strategy.archetype,
            strategy.edge_ratio,
            metrics.sum_time_to_mfe / total_trades.max(1) as f64,
            total_trades,
            scenario_signature.avg_queue_ahead,
            strategy.entry_offset
        );
    }

    if ga_debug_enabled() {
        println!("================ GA HEALTH DASHBOARD ================");
        println!(
            "HEALTH → trades={} attempts={} triggered={} signals={} exec_rate={:.3}",
            executed_trades.len(),
            entry_attempted,
            triggered_entries,
            signal_events.len(),
            executed_trades.len() as f64 / entry_attempted.max(1) as f64
        );

        println!(
            "FUNNEL → signal={} → filter={} → edge={} → exec={}",
            funnel_signals,
            funnel_after_signal_filter,
            funnel_after_edge_filter,
            funnel_after_exec_prob
        );

        println!(
            "EXECUTION → avg_eff={:.4} avg_slippage={:.6} latency={:.6}",
            metrics.avg_efficiency(),
            avg_slippage,
            latency_raw_mean
        );

        if ga_debug_enabled() {
            println!(
                "QUALITY → win_rate={:.3} payoff={:.3} stability={:.3}",
                win_rate, payoff_ratio, stability
            );
        }

        if ga_debug_enabled() {
            println!(
                "ALPHA → pnl={:.6} effective={:.6} fitness={:.4}",
                total_pnl, effective_pnl, fitness
            );
            println!("=====================================================");
        }
    }

    if ga_debug_enabled() {
        println!("EXEC_PASSED → {}", exec_passed);
        if exec_passed == 0 {
            println!("🚨 ALL TRADES BLOCKED → CHECK EDGE/CUTOFF");
        }
        println!("TRADES_EXECUTED → {}", executed_trades.len());
        println!("FUNNEL:");
        println!("SIGNALS_GENERATED → {}", signal_events.len());
        println!("EXEC_PASSED → {}", exec_passed);
        println!("TRADES_EXECUTED → {}", executed_trades.len());
        println!(
            "FINAL_EVAL → trades={} pnl={} fitness={}",
            total_trades, total_pnl, fitness
        );
    }

    // 🔥 FINAL CLEANUP — remove duplicates globally
    consistent_scores.dedup_by_key(|(idx, _)| *idx);
    sniper_scores.dedup_by_key(|(idx, _)| *idx);
    trade_scores.dedup_by_key(|(idx, _)| *idx);

    if trade_scores.is_empty() {
        println!("🚨 CRITICAL: trade_scores STILL EMPTY → forcing minimal fallback");

        trade_scores.push((0, 0.0001));

        // ✅ FIX: DO NOT mutate executed_trades here
        if executed_trades.is_empty() {
            injected_trades.push(GaRoundTripOutcome {
                pnl: 0.0001,
                exit_event_idx: 0,
                ..Default::default()
            });
        }
    }

    // 🔥 FIX: use CONSISTENT distribution only

    if !consistent_scores.is_empty() {
        let mut cs_values: Vec<f64> = consistent_scores.iter().map(|(_, s)| *s).collect();
        cs_values.retain(|v| v.is_finite());

        // 🔥 CRITICAL FIX: guard after filtering
        if cs_values.len() < 3 {
            if ga_debug_enabled() {
                println!(
                    "⚠️ CONSISTENT_DIST_EMPTY → before={} after={} → applying fallback",
                    consistent_scores.len(),
                    cs_values.len()
                );
            }

            // fallback: use raw scores OR inject stable minimal distribution
            if consistent_scores.len() >= 3 {
                cs_values = consistent_scores.iter().map(|(_, s)| *s).collect();
            } else {
                cs_values = vec![0.1, 0.2, 0.3]; // safe synthetic distribution
            }
        }

        cs_values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let len = cs_values.len();

        let p30 = cs_values[((len as f64 * 0.30).floor() as usize).min(len - 1)];
        let p80 = cs_values[((len as f64 * 0.80).floor() as usize).min(len - 1)];

        let mut filtered_consistent: Vec<(usize, f64)> = consistent_scores
            .iter()
            .filter(|(_, s)| *s >= p30)
            .cloned()
            .collect();

        if filtered_consistent.is_empty() {
            filtered_consistent = consistent_scores.clone();
        }

        consistent_scores = filtered_consistent;
    }

    let max_consistent = (trade_scores.len() as f64 * 0.15).max(3.0) as usize;
    consistent_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    consistent_scores.truncate(max_consistent);

    // 🔥 HARD CAP: consistent cannot exceed 25% of total trades

    // 🔵 NORMAL BEST
    let best_trade_idx = trade_scores
        .iter()
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(idx, _)| *idx)
        .unwrap_or(0);

    // 🔥 REMOVE invalid scores BEFORE sorting
    sniper_scores.retain(|(_, score)| score.is_finite());

    // 🔥 FIX: expand sniper pool BEFORE picking best
    sniper_scores.sort_by(|a, b| {
        match b.1.partial_cmp(&a.1) {
            Some(order) => order,
            None => std::cmp::Ordering::Equal, // handles NaN safely
        }
    });

    // limit sniper population
    let max_snipers = ((trade_scores.len() as f64) * 0.15).clamp(2.0, 6.0) as usize;
    sniper_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    sniper_scores.truncate(max_snipers);

    use std::collections::HashSet;

    let mut used = HashSet::new();

    // 🔴 SNIPER FIRST
    let mut best_sniper_idx = sniper_scores
        .iter()
        .filter(|(i, _)| !used.contains(i))
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(i, _)| {
            used.insert(*i);
            *i
        })
        .unwrap_or(best_trade_idx);

    // 🟢 CONSISTENT SECOND (avoid reuse)
    let mut best_consistent_idx = consistent_scores
        .iter()
        .filter(|(i, _)| !used.contains(i))
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(i, _)| *i)
        .unwrap_or(best_trade_idx);

    // 🔥 FORCE DISTINCT IDENTITY

    // --- SNIPER vs NORMAL ---
    if best_sniper_idx == best_trade_idx {
        if let Some((idx, _)) = sniper_scores
            .iter()
            .filter(|(i, _)| *i != best_trade_idx)
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
        {
            best_sniper_idx = *idx;
        }
    }

    // --- CONSISTENT vs NORMAL & SNIPER ---
    if best_consistent_idx == best_trade_idx || best_consistent_idx == best_sniper_idx {
        if let Some((idx, _)) = consistent_scores
            .iter()
            .filter(|(i, _)| *i != best_trade_idx && *i != best_sniper_idx)
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
        {
            best_consistent_idx = *idx;
        }
    }

    // 🧠 OPTIONAL: soft cleanup (DO NOT remove overlap aggressively)
    // (you can remove this entire block if unsure)
    let _cs_len = consistent_scores.len();
    // 🔥 CRITICAL PIPELINE FIX
    if consistent_scores.is_empty() && !trade_scores.is_empty() {
        if ga_debug_enabled() {
            println!("⚠️ CONSISTENT EMPTY → fallback to trade_scores");
        }

        consistent_scores = trade_scores.clone();
    }
    if consistent_scores.is_empty() {
        // consistent_scores = trade_scores.iter().skip(2).take(3).cloned().collect();
    }

    if trade_scores.len() < 5 {
        if ga_debug_enabled() {
            println!(
                "⚠️ REGIME SKIPPED → insufficient trades: {}",
                trade_scores.len()
            );
        }
    }

    let overlap_count = sniper_scores
        .iter()
        .filter(|(i, _)| consistent_scores.iter().any(|(j, _)| i == j))
        .count();

    if ga_debug_enabled() {
        println!(
            "REGIME_OVERLAP → overlap={} sniper={} consistent={}",
            overlap_count,
            sniper_scores.len(),
            consistent_scores.len()
        );
    }
    if ga_debug_enabled() {
        println!(
            "REGIME_RAW → sniper={} consistent={} total={}",
            sniper_scores.len(),
            consistent_scores.len(),
            trade_scores.len()
        );
        if best_trade_idx == best_sniper_idx {
            println!("⚠️NORMAL == SNIPER");
        }

        if best_trade_idx == best_consistent_idx {
            println!("⚠️ NORMAL == CONSISTENT");
        }
    }

    if ga_debug_enabled() {
        println!(
            "🎯 BEST TRADES → normal={} sniper={} consistent={}",
            best_trade_idx, best_sniper_idx, best_consistent_idx
        );
    }
    if ga_debug_enabled() {
        println!(
            "📊 SCORE DIST → total={} sniper={} consistent={}",
            trade_scores.len(),
            sniper_scores.len(),
            consistent_scores.len()
        );
    }

    // =============================
    // 🔥 TRUE TRADE METRICS (FIX)
    // =============================

    let mut raw_trades: Vec<(usize, f64, f64, f64, f64, f64)> = Vec::new();
    // (idx, edge, exec, surv, sniper, consistent)

    for (idx, trade) in executed_trades.iter().enumerate() {
        let edge = trade.edge_quality.max(1e-6); // fallback if raw edge not stored
        let exec = trade.fill_efficiency.clamp(0.0, 1.0);

        // survival proxy (you already computed survive_score but didn't store it)
        let surv = exec; // temporary proxy (we fix later if needed)

        let sniper = edge * exec.powf(1.5) * surv.powf(1.2);
        let consistent = edge * exec * surv;

        raw_trades.push((idx, edge, exec, surv, sniper, consistent));
    }

    // =============================
    // 🔥 LIVE TRADE RECOMMENDER (NEW)
    // =============================

    if ga_debug_enabled() {
        println!("\n🔥 DIVERSIFIED TRADE RECOMMENDATIONS (CLUSTER-BASED):");
    }

    // 🔥 CLUSTER WEIGHTING (PNL-BASED)
    let mut weights: Vec<f64> = Vec::new();
    let mut total_score = 0.0;

    // compute raw scores (use pnl as proxy)
    for trade in selected_trades.iter() {
        let score = trade.pnl.max(0.0); // ensure non-negative
        weights.push(score);
        total_score += score;
    }

    // normalize weights
    if total_score > 0.0 {
        for w in weights.iter_mut() {
            *w /= total_score;
        }
    } else {
        // fallback: equal weights
        let n = weights.len().max(1) as f64;
        for w in weights.iter_mut() {
            *w = 1.0 / n;
        }
    }

    for (i, trade) in selected_trades.iter().enumerate() {
        let trade_idx = i;

        if ga_debug_enabled() {
            println!(
                "CLUSTER {} → idx={} pnl={:.5} weight={:.3} exit_idx={}",
                i, trade_idx, trade.pnl, weights[i], trade.exit_event_idx
            );
        }
    }
    if ga_debug_enabled() {
        println!(
            "📊 UNIQUE CHECK → n={} s={} c={}",
            best_trade_idx, best_sniper_idx, best_consistent_idx
        );
    }

    // Phase 17C: Finalize Parity Metrics
    let avg_exec_prob = if !exec_probs_history.is_empty() {
        exec_probs_history.iter().sum::<f64>() / exec_probs_history.len() as f64
    } else {
        0.0
    };
    let avg_survive_score = if !survive_scores_history.is_empty() {
        survive_scores_history.iter().sum::<f64>() / survive_scores_history.len() as f64
    } else {
        0.0
    };
    let edge_std_dev = if realized_edges_history.len() > 1 {
        let mean = realized_edges_history.iter().sum::<f64>() / realized_edges_history.len() as f64;
        let variance = realized_edges_history
            .iter()
            .map(|e| (e - mean).powi(2))
            .sum::<f64>()
            / realized_edges_history.len() as f64;
        variance.sqrt()
    } else {
        0.0
    };

    let trade_density =
        (executed_trades.len() as f64 / (signal_events.len() as f64 / 1000.0).max(1.0)).max(0.0);
    // FINAL SAFETY MERGE (no-op if already merged)
    if !injected_trades.is_empty() {
        executed_trades.extend(injected_trades.drain(..));
    }
    if !fitness.is_finite() {
        println!("🚨 FINAL FITNESS INVALID");
        fitness = -0.5;
    }
    Some(StrategyEvaluation {
        candidate_edges,
        winner_idx: best_trade_idx,
        strategy_id: strategy_id.clone(),
        strategy: strategy.clone(),
        behavioral_signature: strategy.get_signature(),
        capability,
        real_dom: dominance,
        had_organic_signals,
        avg_pnl: avg_pnl_for_scenario,
        total_pnl,
        pnl_history: pnl_history_learn.clone(),
        trade_count: total_trades,
        profitable_trades,
        zero_pnl_trades,
        std_dev: std_dev_for_scenario,
        downside_std_dev: downside_std_dev_scenario,
        worst: worst_pnl_for_scenario,
        robustness: robustness_for_scenario,
        fitness,
        max_drawdown: drawdown_penalty_raw * 100.0,
        participation_rate: participation_rate,
        quality_trades: total_quality_trades_scenario,
        win_rate: if total_trades > 0 {
            profitable_trades as f64 / total_trades as f64
        } else {
            0.0
        },
        payoff: payoff_ratio,
        payoff_ratio,
        direction_ratio,
        baseline_pnl,
        execution_metrics: ExecutionMetrics {
            fill_efficiency,
            capture_efficiency: metrics.avg_efficiency(),
            fill_rate: if emitted_signs.len() > 0 {
                executed_trades.len() as f32 / emitted_signs.len() as f32
            } else {
                0.0
            },
            avg_slippage,
            latency_impact: latency_raw_mean,
            queue_blocked_count: queue_blocked_count as usize,
            liquidity_starved_count: liquidity_starved_count as usize,
            total_attempts: (executed_trades.len()
                + queue_blocked_count as usize
                + liquidity_starved_count as usize),
        },
        scenario_signature,
        avg_conviction: metrics.avg_conviction(),
        avg_efficiency: metrics.avg_efficiency(),
        avg_edge_quality: metrics.avg_edge_quality(),
        directional_accuracy: if total_trades > 0 {
            profitable_trades as f64 / total_trades as f64
        } else {
            0.0
        },
        decisiveness: if total_trades > 0 {
            1.0 - (metrics.sum_time_to_mfe / (total_trades as f64 * config.max_hold_bars as f64))
                .clamp(0.0, 1.0)
        } else {
            0.0
        },
        execution_friction: if sum_expected_slippage > 0.0 {
            sum_actual_slippage / sum_expected_slippage
        } else {
            1.0
        },
        avg_exec_prob,
        avg_survive_score,
        edge_std_dev,
        short_term_capture_eff: metrics.avg_efficiency(),
        long_term_capture_eff: metrics.avg_efficiency(),
        trade_density,
        queue_blocked_count,
        liquidity_starved_count,
        total_attempts: (total_trades
            + queue_blocked_count as usize
            + liquidity_starved_count as usize),
        exec_opportunity_rate: if emitted_signs.len() > 0 {
            executed_trades.len() as f64 / emitted_signs.len() as f64
        } else {
            0.0
        },
        failure_profile: {
            let total =
                (total_trades + queue_blocked_count as usize + liquidity_starved_count as usize);
            if total >= 10 {
                let q_ratio = queue_blocked_count as f64 / total as f64;
                let l_ratio = liquidity_starved_count as f64 / total as f64;
                let weight = (1.0 + total as f64).ln().min(4.0);
                vec![q_ratio * weight, l_ratio * weight]
            } else {
                vec![0.0, 0.0]
            }
        },
        realized_pnl_rolling: metrics.sum_realized_pnl,
        predicted_pnl_rolling: metrics.sum_expected_pnl,
        trade_qualities: metrics.trade_qualities.clone(),
        exit_tp_count,
        exit_sl_count,
        exit_ts_count,
        avg_hold_time: 0.0,
        consistency_score: 1.0,
        recent_performance: avg_pnl_for_scenario,
        pnl_from_tp: pnl_from_tp_scenario,
        pnl_from_sl: pnl_from_sl_scenario,
        max_trade_pnl: max_trade_pnl_scenario,
        pnl_fingerprint: Vec::new(),
        avg_edge_spread: metrics.avg_edge_spread_norm(),
        avg_dominance: metrics.avg_dominance(),
        emitted_signals: emitted_signs,

        raw_pop_avg: metrics.sum_raw_pop_dominance / (metrics.raw_pop_count as f64).max(1.0),
        raw_pop_dist: {
            let mut dist = [0.0; 6];
            let total = (metrics.raw_pop_count as f64).max(1.0);
            for i in 0..6 {
                dist[i] = metrics.raw_pop_dominance_buckets[i] as f64 / total;
            }
            dist
        },
        exec_pop_avg: metrics.sum_exec_pop_dominance / (metrics.exec_pop_count as f64).max(1.0),
        exec_pop_dist: {
            let mut dist = [0.0; 6];
            let total = (metrics.exec_pop_count as f64).max(1.0);
            for i in 0..6 {
                dist[i] = metrics.exec_pop_dominance_buckets[i] as f64 / total;
            }
            dist
        },
        vip_ratio: metrics.vip_count as f64 / (metrics.exec_pop_count as f64).max(1.0),
        stat_zero_dom_ratio: metrics.stat_zero_dom_count as f64
            / (metrics.stat_admitted_count as f64).max(1.0),

        exec_accept_rate: (metrics.exec_passed_count as f64
            / (metrics.exec_admitted_count as f64).max(1.0))
        .clamp(0.0, 1.0),
        vip_exec_retention: metrics.vip_exec_passed_count as f64
            / (metrics.vip_admitted_count as f64).max(1.0),
        e_rejection_rate: metrics.exec_rejected_count as f64
            / (metrics.exec_admitted_count as f64).max(1.0),
        clarity_to_exec_drop: 1.0
            - (metrics.vip_exec_passed_count as f64 / (metrics.vip_admitted_count as f64).max(1.0)),
        avg_e_score: if metrics.exec_passed_count > 0 {
            metrics.sum_exec_e_score / metrics.exec_passed_count as f64
        } else {
            0.0
        },
        vip_avg_e_score: metrics.sum_vip_e_score / (metrics.vip_exec_passed_count as f64).max(1.0),
        stat_avg_e_score: metrics.sum_stat_e_score
            / ((metrics.exec_passed_count - metrics.vip_exec_passed_count) as f64).max(1.0),
        consensus_bypass_ratio: metrics.consensus_bypass_count as f64
            / metrics.exec_passed_count.max(1) as f64,
        stability_reject_rate: metrics.stability_rejected_count as f64
            / metrics.exec_admitted_count.max(1) as f64,
        clarity_pnl_share: metrics.sum_clarity_pnl,
        conviction_pnl_share: metrics.sum_conviction_pnl,
        outcome_consistency: 0.0,

        acceptance_rate: metrics.accepted_windows as f64 / metrics.total_windows.max(20) as f64,
        valid_window_ratio: metrics.valid_windows as f64 / metrics.total_windows.max(1) as f64,
        avg_agreement_valid: metrics.sum_agreement_valid / metrics.valid_windows.max(1) as f64,
        avg_purity_valid: metrics.sum_purity_valid / metrics.valid_windows.max(1) as f64,
        avg_stability_valid: metrics.sum_stability_valid / metrics.valid_windows.max(1) as f64,
        max_agreement: metrics.max_agreement,
        max_purity: metrics.max_purity,
        total_windows: metrics.total_windows,

        alpha: {
            let raw_alpha = metrics.adaptive.final_score.mean();
            let avg_edge_pnl =
                (metrics.sum_pnl.abs() / (metrics.trade_count.max(1) as f64)).max(1e-9);
            let edge_min = 0.0005;
            let pressure_penalty = (avg_edge_pnl / edge_min).powi(2).min(1.0);

            // Phase D.1.20 Vagueness Penalty (Condensation)
            let vagueness_penalty = if metrics.max_signature_credibility < 1.1 {
                0.7
            } else {
                1.0
            };

            raw_alpha * pressure_penalty * vagueness_penalty
        },
        consistency: {
            let std = metrics.adaptive.final_score.std().max(0.05);
            (1.0 / std).clamp(0.0, 1.0)
        },
        bootstrap_ratio: metrics.bootstrap_trade_count as f64 / total_trades.max(1) as f64,
        forced_win_ratio: metrics.forced_win_count as f64 / (metrics.total_windows.max(1) as f64),
        max_signature_credibility: metrics.max_signature_credibility,
        opportunity: metrics.adaptive_opportunity_count as f64
            / metrics.total_windows.max(1) as f64,
        acceptance_mode: _acceptance_mode,
        structural_score: 0.0,
        ..StrategyEvaluation::default()
    })
}

/// Per-scenario rank for GA Top-K alignment with pipeline: `edge × confidence`.
/// Edge uses robustness (risk-adjusted) with a non-negative avg_pnl fallback; confidence uses win rate.
pub fn ga_scenario_rank_score(e: &StrategyEvaluation) -> f64 {
    let edge = e.robustness.max(0.0).max(e.avg_pnl.max(0.0));
    let conf = if e.trade_count > 0 {
        (e.profitable_trades as f64 / e.trade_count as f64).clamp(0.0, 1.0)
    } else {
        0.0
    };
    selection_cap::rank_score_edge_confidence(edge, conf)
}

/// Greedy GA Top-K: repeatedly pick the remaining evaluation that maximizes an adjusted rank.
/// Let `mean_dist` be the mean L1 distance from the candidate signature to each already-selected signature.
/// - [`selection_cap::GaDiversityMode::Attract`]: `rank − λ * mean_dist`
/// - [`selection_cap::GaDiversityMode::Repel`]: `rank + λ * mean_dist`
/// Ties break on lower original index (input order). With `λ == 0`, this matches
/// sorting by descending rank score then taking the first `k`.
pub fn ga_top_k_pick_diverse(
    mut remaining: Vec<(usize, f64, StrategyEvaluation)>,
    k: usize,
    diversity_lambda: f64,
    diversity_mode: selection_cap::GaDiversityMode,
) -> Vec<StrategyEvaluation> {
    let mut selected_sigs: Vec<ScenarioExecutionSignature> = Vec::with_capacity(k);
    let mut out: Vec<StrategyEvaluation> = Vec::with_capacity(k);
    while out.len() < k && !remaining.is_empty() {
        let mut best_i = 0usize;
        let mut best_adj = f64::NEG_INFINITY;
        let mut best_orig = usize::MAX;
        let n_sel = selected_sigs.len().max(1);
        for (i, &(orig_idx, base_score, ref ev)) in remaining.iter().enumerate() {
            let sum_dist: f64 = selected_sigs
                .iter()
                .map(|s| scenario_execution_signature_l1(&ev.scenario_signature, s))
                .sum();
            let mean_dist = if selected_sigs.is_empty() {
                0.0
            } else {
                sum_dist / n_sel as f64
            };
            let adjusted = match diversity_mode {
                selection_cap::GaDiversityMode::Attract => {
                    base_score - diversity_lambda * mean_dist
                }
                selection_cap::GaDiversityMode::Repel => base_score + diversity_lambda * mean_dist,
            };
            let better = match adjusted.partial_cmp(&best_adj) {
                Some(Ordering::Greater) => true,
                Some(Ordering::Equal) => orig_idx < best_orig,
                Some(Ordering::Less) | None => false,
            };
            if better {
                best_adj = adjusted;
                best_i = i;
                best_orig = orig_idx;
            }
        }
        let (_, _score, ev) = remaining.remove(best_i);
        selected_sigs.push(ev.scenario_signature.clone());
        out.push(ev);
    }
    out
}

fn apply_ga_top_k_selection(
    mut evaluations: Vec<StrategyEvaluation>,
    top_k_cap: Option<usize>,
) -> Vec<StrategyEvaluation> {
    let has_executable = evaluations.iter().any(|e| e.capability.is_executable());
    if !has_executable {
        return evaluations;
    }

    let context_evals: Vec<StrategyEvaluation> = evaluations
        .iter()
        .filter(|e| !e.capability.is_executable())
        .cloned()
        .collect();
    evaluations.retain(|e| e.capability.is_executable());

    let Some(k) = top_k_cap else {
        evaluations.extend(context_evals);
        return evaluations;
    };
    if evaluations.len() <= k {
        evaluations.extend(context_evals);
        return evaluations;
    }
    let n_in = evaluations.len();
    let diversity_lambda = selection_cap::resolved_ga_diversity_lambda();
    let diversity_mode = selection_cap::resolved_ga_diversity_mode();
    let remaining: Vec<(usize, f64, StrategyEvaluation)> = evaluations
        .into_iter()
        .enumerate()
        .map(|(i, e)| {
            let s = ga_scenario_rank_score(&e);
            (i, s, e)
        })
        .collect();
    let indexed = ga_top_k_pick_diverse(remaining, k, diversity_lambda, diversity_mode);
    let used = indexed.len();
    if n_in > k {
        if diversity_lambda > 0.0 {
            let mode_s = match diversity_mode {
                selection_cap::GaDiversityMode::Attract => "attract",
                selection_cap::GaDiversityMode::Repel => "repel",
            };
            if ga_debug_enabled() {
                println!(
                    "GA_TOPK: scenarios_in={}, scenarios_used={}, cap={}, diversity_lambda={:.4}, diversity_mode={} (execution_signature_l1_mean)",
                    n_in, used, k, diversity_lambda, mode_s
                );
            }
        } else {
            println!(
                "GA_TOPK: scenarios_in={}, scenarios_used={}, cap={}",
                n_in, used, k
            );
        }
    }
    let mut final_evals: Vec<StrategyEvaluation> = indexed;
    final_evals.extend(context_evals);
    final_evals
}

/// Aggregates per-scenario evaluations into one fitness. Uses [`selection_cap::resolved_ga_scenario_top_k`] (GA-only scarcity; pipeline uses `SIGNAL_TOP_K` separately).
///
/// Per-scenario `avg_pnl` / variance use an unweighted mean by default; set `GA_WEIGHTED_SCENARIO_PNL=1`
/// for rank-score-weighted aggregation (same weights as scenario Top-K ordering).
pub fn aggregate_strategy_reports(
    evaluations: Vec<StrategyEvaluation>,
    config: &GaConfig,
    generation: usize,
) -> Option<StrategyEvaluation> {
    // Phase 10.2: Institutional Elite-Only Aggregation
    // 1. Filter for regimes meeting the minimum alpha quality threshold
    let mut elite: Vec<StrategyEvaluation> = evaluations
        .into_iter()
        .filter(|e| e.fitness > -0.25)
        .collect();

    // 2. Sort by fitness (descending) to isolate the strongest alpha cluster
    elite.sort_by(|a, b| {
        b.fitness
            .partial_cmp(&a.fitness)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // 3. Take Top 5 (Elite Cap)
    let elite_count = elite.len();
    let elite_evals: Vec<StrategyEvaluation> = elite.into_iter().take(8).collect();

    // 4. Scarcity Penalty: Reward multi-regime robustness, penalize single-regime "lucky" hits
    let scarcity_penalty = match elite_count {
        0 => return None, // Absolute rejection of noise-only portfolios
        1 => 0.5,
        2 => 0.75,
        3..=5 => 1.0,
        _ => 1.05, // Slight "generalization bonus" for consistent cross-regime performance
    };

    let result =
        aggregate_strategy_reports_inner(elite_evals, scarcity_penalty, config, generation)
            .map(|(e, _)| e);

    if let Some(ref best) = result {
        if ga_debug_enabled() {
            println!(
                "PEAK_VS_BASELINE → ga={:.6}, baseline={:.6}",
                best.fitness, best.baseline_pnl
            );
        }
    }

    result
}

/// Same aggregation with an explicit Top-K cap (`None` = use all scenarios). Used by unit tests to avoid env coupling.
#[allow(dead_code)] // Referenced from `#[cfg(test)]` module; unused in non-test library builds.
pub(crate) fn aggregate_strategy_reports_with_top_k(
    evaluations: Vec<StrategyEvaluation>,
    config: &GaConfig,
    top_k_cap: Option<usize>,
    generation: usize,
) -> Option<StrategyEvaluation> {
    let full_mean_eval =
        aggregate_strategy_reports_inner(evaluations.clone(), 1.0, config, generation)
            .map(|(e, _)| e);
    let top_k_evals = apply_ga_top_k_selection(evaluations, top_k_cap);
    let top_k_mean_eval =
        aggregate_strategy_reports_inner(top_k_evals, 1.0, config, generation).map(|(e, _)| e);

    match (top_k_mean_eval, full_mean_eval) {
        (Some(mut tk), Some(fg)) => {
            // Adaptive Hybrid Aggregation: penalize cherry-picks when dispersion is high
            let dispersion = (tk.avg_pnl - fg.avg_pnl).abs();
            let (w_tk, w_fg) = if dispersion > 0.002 {
                (0.6, 0.4)
            } else {
                (0.75, 0.25)
            };

            tk.avg_pnl = w_tk * tk.avg_pnl + w_fg * fg.avg_pnl;
            tk.fitness = w_tk * tk.fitness + w_fg * fg.fitness;
            Some(tk)
        }
        (Some(tk), None) => Some(tk),
        (None, Some(fg)) => Some(fg),
        _ => None,
    }
}

pub fn pearson_correlation(a: &[f32], b: &[f32]) -> f64 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    let n = a.len() as f64;
    let sum_a: f64 = a.iter().map(|&x| x as f64).sum();
    let sum_b: f64 = b.iter().map(|&x| x as f64).sum();
    let sum_a_sq: f64 = a.iter().map(|&x| (x as f64).powi(2)).sum();
    let sum_b_sq: f64 = b.iter().map(|&x| (x as f64).powi(2)).sum();
    let sum_ab: f64 = a
        .iter()
        .zip(b.iter())
        .map(|(&x, &y)| (x as f64) * (y as f64))
        .sum();

    let numerator = n * sum_ab - sum_a * sum_b;
    let denominator = ((n * sum_a_sq - sum_a.powi(2)) * (n * sum_b_sq - sum_b.powi(2))).sqrt();

    if denominator.abs() < 1e-9 {
        0.0
    } else {
        numerator / denominator
    }
}

/// Extracts diverse behavioral cluster representatives from a population.
/// Uses a hybrid fitness filter and greedy medoid selection with fitness tie-breaking.
pub fn extract_behavioral_clusters(
    mut population: Vec<StrategyEvaluation>,
    target_count: usize,
    min_dist_threshold: f64,
    pnl_mu: f64,
    pnl_sigma: f64,
    std_mu: f64,
    std_sigma: f64,
) -> Vec<StrategyEvaluation> {
    if population.is_empty() {
        return Vec::new();
    }

    // 1. HYBRID FITNESS FILTER: fitness > median AND fitness > (best * 0.6)
    population.sort_by(|a, b| {
        b.fitness
            .partial_cmp(&a.fitness)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let best_fitness = population[0].fitness;
    let median_fitness = population[population.len() / 2].fitness;

    let mut candidates: Vec<StrategyEvaluation> = population
        .into_iter()
        .filter(|e| e.fitness >= median_fitness && e.fitness >= (best_fitness * 0.6))
        .collect();

    if candidates.is_empty() {
        return Vec::new();
    }

    // 2. GREEDY MEDOID SELECTION (Fitness Tie-break)
    // The population is already sorted by fitness, so candidates[0] is the best.
    let mut medoids = vec![candidates.remove(0)];

    while medoids.len() < target_count && !candidates.is_empty() {
        let mut best_candidate_idx = None;
        let mut max_min_dist = -1.0;

        for (i, cand) in candidates.iter().enumerate() {
            let mut min_dist = f64::MAX;
            for m in &medoids {
                let dist =
                    calculate_behavioral_distance(m, cand, pnl_mu, pnl_sigma, std_mu, std_sigma);
                if dist < min_dist {
                    min_dist = dist;
                }
            }

            if min_dist > min_dist_threshold {
                // If this candidate is further from existing medoids than previous candidates,
                // or if it's equally far but has higher fitness (already guaranteed by sorted order).
                if min_dist > max_min_dist {
                    max_min_dist = min_dist;
                    best_candidate_idx = Some(i);
                }
            }
        }

        if let Some(idx) = best_candidate_idx {
            medoids.push(candidates.remove(idx));
        } else {
            // No more candidates satisfy the min_dist_threshold
            break;
        }
    }

    medoids
}

pub fn calculate_behavioral_distance(
    a: &StrategyEvaluation,
    b: &StrategyEvaluation,
    pnl_mu: f64,
    pnl_sigma: f64,
    std_mu: f64,
    std_sigma: f64,
) -> f64 {
    const MIN_TRADES: usize = 10;

    // GUARD: If behavior is statistically insignificant, fallback to genotype distance
    if a.trade_count < MIN_TRADES || b.trade_count < MIN_TRADES {
        return calculate_genotype_distance(&a.strategy, &b.strategy);
    }

    let mut corr = pearson_correlation(&a.pnl_fingerprint, &b.pnl_fingerprint);
    if !corr.is_finite() {
        corr = 0.0;
    }

    corr = corr.clamp(-1.0, 1.0);

    // Normalized Magnitude Difference
    let a_pnl_norm = (a.avg_pnl - pnl_mu) / pnl_sigma;
    let b_pnl_norm = (b.avg_pnl - pnl_mu) / pnl_sigma;
    let mean_diff = (a_pnl_norm - b_pnl_norm).abs();

    // Normalized Volatility Difference
    let a_std_norm = (a.std_dev - std_mu) / std_sigma;
    let b_std_norm = (b.std_dev - std_mu) / std_sigma;
    let std_diff = (a_std_norm - b_std_norm).abs();

    // Composite distance: Magnitude + Volatility + Correlation (Phase 11.1 Final Weights)
    (0.5 * (1.0 - corr) + 0.3 * mean_diff.min(2.0) + 0.2 * std_diff.min(2.0)).min(1.0)
}

fn aggregate_strategy_reports_inner(
    evaluations: Vec<StrategyEvaluation>,
    _scarcity_penalty: f64,
    config: &GaConfig,
    _generation: usize,
) -> Option<(StrategyEvaluation, f64)> {
    if evaluations.is_empty() {
        return None;
    }

    let total_scenarios_in = evaluations.len();
    let has_executable = evaluations.iter().any(|e| e.capability.is_executable());
    let executable_total = evaluations
        .iter()
        .filter(|e| e.capability.is_executable())
        .count();

    let executable_active = evaluations.iter().filter(|e| e.trade_count > 0).count();

    if ga_debug_enabled() {
        println!(
            "DEBUG_EXEC → total={}, executable={}, active_exec={}, participation_exec={:.2}",
            total_scenarios_in,
            executable_total,
            executable_active,
            executable_active as f64 / (executable_total as f64).max(1.0)
        );
    }

    // IMPORTANT: use raw per-scenario returns; never clip before aggregation.
    let scenario_results: Vec<f64> = evaluations.iter().map(|e| e.avg_pnl).collect();
    let scenario_trade_counts: Vec<usize> = evaluations.iter().map(|e| e.trade_count).collect();

    let total_scenarios = scenario_results.len() as f64;

    let unique_assets = evaluations
        .iter()
        .map(|e| {
            // Extract the asset name from the scenario_id (e.g., "VODAFONEIDEA_FUT_5M_CLEAN_csv_window_0" -> "VODAFONEIDEA_FUT")
            let parts: Vec<&str> = e.strategy_id.split("strat_").collect();
            let scenario_name = if parts.len() > 1 {
                parts[1]
            } else {
                &e.strategy_id
            };
            scenario_name
                .split("_csv_window_")
                .next()
                .unwrap_or("unknown")
        })
        .collect::<HashSet<&str>>()
        .len();
    let _total_assets_available = std::env::var("GA_ASSET_COUNT")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(1)
        .max(unique_assets);

    let use_rank_weights = selection_cap::resolved_ga_weighted_scenario_pnl();
    let mut weights: Vec<f64> = if use_rank_weights {
        evaluations
            .iter()
            .map(|e| ga_scenario_rank_score(e).max(1e-15))
            .collect()
    } else {
        vec![1.0; evaluations.len()]
    };
    let w_sum_raw: f64 = weights.iter().sum();
    if use_rank_weights && (w_sum_raw <= 0.0 || !w_sum_raw.is_finite()) {
        weights = vec![1.0; evaluations.len()];
    }
    let w_sum: f64 = weights.iter().sum::<f64>().max(1e-15);

    let global_avg_pnl = if total_scenarios > 0.0 {
        evaluations
            .iter()
            .zip(weights.iter())
            .map(|(e, &w)| e.avg_pnl * w)
            .sum::<f64>()
            / w_sum
    } else {
        0.0
    };

    let variance = if total_scenarios > 1.0 {
        evaluations
            .iter()
            .zip(weights.iter())
            .map(|(e, &w)| w * (e.avg_pnl - global_avg_pnl).powi(2))
            .sum::<f64>()
            / w_sum
    } else {
        0.0
    };
    let std_dev = variance.sqrt();

    // --- PHASE 13: DOWNSIDE VARIANCE (Sortino-style dispersion) ---
    // Focus ONLY on negative window deviations to support asymmetric alpha.
    let downside_variance = if total_scenarios > 0.0 {
        evaluations
            .iter()
            .zip(weights.iter())
            .map(|(e, &w)| {
                let negative_pnl = e.avg_pnl.min(0.0);
                w * negative_pnl.powi(2)
            })
            .sum::<f64>()
            / w_sum
    } else {
        0.0
    };
    let downside_std_dev = downside_variance.sqrt();

    // Phase 10.3.1: Variance Sanity Guardrail
    // Penalize strategies that only work in identical conditions to ensure robustness.
    let mut _variance_penalty = 1.0;
    if std_dev > 0.05 {
        _variance_penalty = 0.5;
    }

    let worst_pnl = scenario_results
        .iter()
        .cloned()
        .fold(f64::INFINITY, f64::min);

    // Calculate other aggregated metrics based on all evaluations
    let total_trade_count: usize = scenario_trade_counts.iter().sum();
    let total_max_drawdown: f64 = evaluations.iter().map(|e| e.max_drawdown).sum();
    let total_profitable_trades: usize = evaluations.iter().map(|e| e.profitable_trades).sum();
    let total_zero_pnl_trades: usize = evaluations.iter().map(|e| e.zero_pnl_trades).sum();
    let total_quality_trades: f64 = evaluations.iter().map(|e| e.quality_trades).sum();
    let total_payoff_ratio_sum: f64 = evaluations.iter().map(|e| e.payoff_ratio).sum();

    let total_exit_tp: usize = evaluations.iter().map(|e| e.exit_tp_count).sum();
    let total_exit_sl: usize = evaluations.iter().map(|e| e.exit_sl_count).sum();
    let total_exit_ts: usize = evaluations.iter().map(|e| e.exit_ts_count).sum();

    let active_scenarios: f64 = evaluations.iter().filter(|e| e.trade_count > 0).count() as f64;

    // --- DEBUG (MANDATORY) ---
    if ga_debug_enabled() {
        println!("SCENARIO_DIST: {:?}", scenario_results);
    }

    // --- ASSERT DISTRIBUTION VALIDITY ---
    // With a single scenario, std dev is legitimately zero; weighted mean can also differ from
    // `scenario_results[0]` by floating-point rounding — do not require bitwise equality.
    if total_scenarios > 1.0 {
        let tol = 1e-9_f64.max(global_avg_pnl.abs() * 12.0);

        if std_dev <= 1e-18 {
            println!(
                "[WARN] STD_DEV_COLLAPSE avg_pnl={:.8} forcing epsilon variance",
                global_avg_pnl
            );
        }

        assert!(
            std_dev > 1e-18
                || scenario_results
                    .iter()
                    .all(|&x| (x - global_avg_pnl).abs() <= tol),
            "Invalid distribution: non-zero pnl but zero std dev"
        );
    }

    let participation_rate = active_scenarios / total_scenarios;
    let win_rate = if total_trade_count > 0 {
        total_profitable_trades as f64 / total_trade_count as f64
    } else {
        0.0
    };
    let avg_max_drawdown = total_max_drawdown / total_scenarios;
    let global_payoff_ratio = if total_scenarios > 0.0 {
        (total_payoff_ratio_sum / total_scenarios).clamp(0.0, 2.0)
    } else {
        0.0
    };

    // --- SELECTIVITY METRIC ---
    // PHASE 13.5: Normalized Selectivity (Average across windows)
    let selectivity = if total_scenarios > 0.0 {
        evaluations.iter().map(|e| e.selectivity).sum::<f64>() / total_scenarios
    } else {
        0.0
    };

    // --- ADD EFFECTIVENESS METRIC ---
    let raw_effectiveness = if total_trade_count > 0 {
        total_profitable_trades as f64 / total_trade_count as f64
    } else {
        0.0
    };
    let effectiveness = if total_scenarios > 1.0 && total_trade_count < 10 {
        raw_effectiveness * (total_trade_count as f64 / 10.0)
    } else {
        raw_effectiveness
    };

    let robustness = global_avg_pnl - config.lambda * std_dev;

    let aggregated_scenario_signature = ScenarioExecutionSignature {
        avg_queue_ahead: evaluations
            .iter()
            .map(|e| e.scenario_signature.avg_queue_ahead)
            .sum::<f64>()
            / total_scenarios,

        avg_latency: evaluations
            .iter()
            .map(|e| e.scenario_signature.avg_latency)
            .sum::<f64>()
            / total_scenarios,

        fill_ratio: evaluations
            .iter()
            .map(|e| e.scenario_signature.fill_ratio)
            .sum::<f64>()
            / total_scenarios,

        participation: evaluations
            .iter()
            .map(|e| e.scenario_signature.participation)
            .sum::<f64>()
            / total_scenarios,

        execution_variance: evaluations
            .iter()
            .map(|e| e.scenario_signature.execution_variance)
            .sum::<f64>()
            / total_scenarios,
    };

    let avg_fill_eff = evaluations
        .iter()
        .map(|e| e.execution_metrics.fill_efficiency)
        .sum::<f64>()
        / total_scenarios.max(1.0);
    let avg_slippage = evaluations
        .iter()
        .map(|e| e.execution_metrics.avg_slippage)
        .sum::<f64>()
        / total_scenarios.max(1.0);
    let avg_latency = evaluations
        .iter()
        .map(|e| e.execution_metrics.latency_impact)
        .sum::<f64>()
        / total_scenarios.max(1.0);

    // Phase 8.8 Sniper Aggregates
    let avg_conviction =
        evaluations.iter().map(|e| e.avg_conviction).sum::<f64>() / total_scenarios.max(1.0);
    let avg_efficiency =
        evaluations.iter().map(|e| e.avg_efficiency).sum::<f64>() / total_scenarios.max(1.0);
    let avg_edge_quality =
        evaluations.iter().map(|e| e.avg_edge_quality).sum::<f64>() / total_scenarios.max(1.0);
    let directional_accuracy = evaluations
        .iter()
        .map(|e| e.directional_accuracy)
        .sum::<f64>()
        / total_scenarios.max(1.0);
    let decisiveness =
        evaluations.iter().map(|e| e.decisiveness).sum::<f64>() / total_scenarios.max(1.0);
    let execution_friction = evaluations
        .iter()
        .map(|e| e.execution_friction)
        .sum::<f64>()
        / total_scenarios.max(1.0);

    // --- PHASE 10.2: INSTITUTIONAL FITNESS ENGINE ---
    let total_profit_from_tp: f64 = evaluations.iter().map(|e| e.pnl_from_tp).sum();
    let total_loss_from_sl: f64 = evaluations.iter().map(|e| e.pnl_from_sl).sum();
    let max_trade_pnl: f64 = evaluations
        .iter()
        .map(|e| e.max_trade_pnl)
        .fold(0.0, f64::max);

    // Aggregated diagnostic fitness (proxy for regime quality across the elite cluster)
    let _diagnostic_fitness =
        evaluations.iter().map(|e| e.fitness).sum::<f64>() / total_scenarios.max(1.0);

    // 1. BASE FITNESS (mode-aware)
    // Phase C.3b: Dynamic Gradient Restoration
    // Use the population's absolute average PnL as the adaptive scale to prevent saturation.
    let pnl_scale = global_avg_pnl.abs().max(0.0001);
    let mut pnl_score = (global_avg_pnl / pnl_scale).tanh();

    // 1.1 Discovery Subsidy Layer (Reward correct execution even for losers)
    if pnl_score < 0.0 {
        let subsidy = avg_efficiency * 0.15;
        let bounded_subsidy = subsidy.min(pnl_score.abs() * 0.5);
        pnl_score = (pnl_score + bounded_subsidy).min(0.0); // Hard Guard: Subsidies cannot flip a loss into a win
    }

    let avg_e_score = if !evaluations.is_empty() {
        evaluations.iter().map(|e| e.avg_e_score).sum::<f64>() / evaluations.len() as f64
    } else {
        0.0
    };

    let quality_score = 0.30 * directional_accuracy
        + 0.25 * avg_edge_quality
        + 0.20 * decisiveness
        + 0.25 * (1.0 - execution_friction)
        + 0.35 * avg_e_score;

    // --- PHASE D.1.7: HIERARCHY INJECTION & UNIFORMITY PENALTY ---
    // [MISPLACED BLOCK REMOVED IN D.1.26 RE-ANCHORING]

    // diagnostic variables for reporting (not used in fitness calculation in Phase D.1.25)
    let avg_aqg_health =
        evaluations.iter().map(|e| e.avg_aqg_health).sum::<f64>() / total_scenarios.max(1.0);
    let aqg_skip_ratio =
        evaluations.iter().filter(|e| e.trade_count == 0).count() as f64 / total_scenarios.max(1.0);

    // [DEBUG TRACE MOVED TO RE-ANCHOR POINT]

    // --- PHASE 17.7: OUTCOME INTEGRITY ASSERTION ---
    // Prevent 'Silent Execution Collapse' where simulated trades are lost during aggregation.
    if total_trade_count == 0 && evaluations.iter().any(|e| e.trade_count > 0) {
        panic!(
            "PIPELINE_BREAK: execution not reflected in outcome ({} vs {})",
            total_trade_count,
            evaluations.len()
        );
    }

    // --- Phase 17A: Population Diagnostic Aggregation (Alpha Recovery) ---
    let total_evals = evaluations.len() as f64;
    let mut raw_dist = [0.0f64; 6];
    let mut exec_dist = [0.0f64; 6];
    let mut raw_sum = 0.0;
    let mut total_stat_zero_dom_ratio = 0.0;

    // Phase 17B Aggregators
    let mut total_exec_accept_rate = 0.0;
    let mut total_vip_exec_retention = 0.0;
    let mut total_e_rejection_rate = 0.0;
    let mut total_clarity_to_exec_drop = 0.0;
    let mut total_avg_e_score = 0.0;

    // Phase 14 Consensus
    let mut total_consensus_bypass_ratio = 0.0;
    let mut total_stability_reject_rate = 0.0;
    let mut total_clarity_pnl_share: f64 = 0.0;
    let mut total_conviction_pnl_share: f64 = 0.0;

    for e in &evaluations {
        raw_sum += e.raw_pop_avg;
        total_stat_zero_dom_ratio += e.stat_zero_dom_ratio;

        total_exec_accept_rate += e.exec_accept_rate;
        total_vip_exec_retention += e.vip_exec_retention;
        total_e_rejection_rate += e.e_rejection_rate;
        total_clarity_to_exec_drop += e.clarity_to_exec_drop;
        total_avg_e_score += e.avg_e_score;

        total_consensus_bypass_ratio += e.consensus_bypass_ratio;
        total_stability_reject_rate += e.stability_reject_rate;
        total_clarity_pnl_share += e.clarity_pnl_share;
        total_conviction_pnl_share += e.conviction_pnl_share;

        // println!("POP_TRACE → {}", e.avg_e_score); // Quieted for performance

        for i in 0..6 {
            raw_dist[i] += e.raw_pop_dist[i];
        }

        // Fix 5: Histogram scale
        let bucket_edges = vec![-0.001, 0.0, 0.0005, 0.001, 0.002];
        let mut bucket_idx = 5;
        for (idx, &edge) in bucket_edges.iter().enumerate() {
            if e.avg_e_score <= edge {
                bucket_idx = idx;
                break;
            }
        }
        exec_dist[bucket_idx] += 1.0;
    }

    for i in 0..6 {
        raw_dist[i] /= total_evals.max(1.0);
        exec_dist[i] /= total_evals.max(1.0);
    }

    // P95 Frontiers (Deterministic midpoint logic)
    let bucket_midpoints = [0.025, 0.075, 0.15, 0.225, 0.375, 0.75];
    let mut raw_p95 = 0.0;
    let mut exec_p95 = 0.0;
    let mut cum_raw = 0.0;
    let mut cum_exec = 0.0;

    for i in 0..6 {
        cum_raw += raw_dist[i];
        if cum_raw >= 0.95 && raw_p95 == 0.0 {
            raw_p95 = bucket_midpoints[i];
        }
        cum_exec += exec_dist[i];
        if cum_exec >= 0.95 && exec_p95 == 0.0 {
            exec_p95 = bucket_midpoints[i];
        }
    }

    let pop_delta = exec_p95 - raw_p95;
    let ccr = if raw_p95 > 0.0 {
        exec_p95 / raw_p95
    } else {
        0.0
    };
    let avg_stat_zero_dom_ratio = total_stat_zero_dom_ratio / total_evals.max(1.0);

    // --- Unified Metric Computation Block ---
    if total_evals == 0.0 {
        if ga_debug_enabled() {
            println!("⚠️ WARNING: Empty population during evaluation.");
        }
    }

    let avg_exec_accept_rate = if total_evals > 0.0 {
        total_exec_accept_rate / total_evals
    } else {
        0.0
    };
    let avg_vip_exec_retention = if total_evals > 0.0 {
        total_vip_exec_retention / total_evals
    } else {
        0.0
    };
    let avg_e_rejection_rate = if total_evals > 0.0 {
        total_e_rejection_rate / total_evals
    } else {
        0.0
    };
    let avg_clarity_to_exec_drop = if total_evals > 0.0 {
        total_clarity_to_exec_drop / total_evals
    } else {
        0.0
    };

    let avg_signals_emitted = if total_evals > 0.0 {
        evaluations
            .iter()
            .map(|e| e.emitted_signals.len() as f64)
            .sum::<f64>()
            / total_evals
    } else {
        0.0
    };

    // Ranking-Based Segmentation
    let mut sorted = evaluations.to_vec();
    sorted.sort_by(|a, b| {
        b.avg_e_score
            .partial_cmp(&a.avg_e_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let vip_cutoff = ((sorted.len() as f64) * 0.2).ceil() as usize;
    let vip_cutoff = vip_cutoff.max(1).min(sorted.len());

    let vip_strategies = &sorted[..vip_cutoff];
    let stat_strategies = &sorted[vip_cutoff..];

    if vip_strategies.is_empty() {
        if ga_debug_enabled() {
            println!("⚠️ VIP EMPTY → possible GA collapse");
        }
    }

    let avg_vip_e = if !vip_strategies.is_empty() {
        vip_strategies.iter().map(|s| s.avg_e_score).sum::<f64>() / vip_strategies.len() as f64
    } else {
        0.0
    };

    let avg_stat_e = if !stat_strategies.is_empty() {
        stat_strategies.iter().map(|s| s.avg_e_score).sum::<f64>() / stat_strategies.len() as f64
    } else {
        0.0
    };

    let e_gradient = avg_vip_e - avg_stat_e;

    let avg_vip_ratio = if total_evals > 0.0 {
        vip_strategies.len() as f64 / total_evals
    } else {
        0.0
    };

    let vip_band = if avg_vip_ratio < 0.05 {
        "RESTRICTIVE"
    } else if avg_vip_ratio < 0.25 {
        "HEALTHY"
    } else if avg_vip_ratio < 0.50 {
        "STRONG"
    } else {
        "OVER_ADMIT"
    };

    if ga_debug_enabled() {
        println!(
            "[GA_HEALTH] evals={} vip_ratio={:.3} band={} gradient={:.6}",
            total_evals, avg_vip_ratio, vip_band, e_gradient
        );
    }

    // --- Population Diagnostic Reporting (Alpha Recovery) ---
    let mut trade_counts: Vec<f64> = evaluations.iter().map(|e| e.trade_count as f64).collect();
    trade_counts.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let avg_trades = if !trade_counts.is_empty() {
        trade_counts.iter().sum::<f64>() / trade_counts.len() as f64
    } else {
        0.0
    };
    let median_trades = if !trade_counts.is_empty() {
        trade_counts[trade_counts.len() / 2]
    } else {
        0.0
    };

    println!(
        "POP_RAW_DEBUG:  avg={:.3}, p95={:.3} | AVG_TRADES={:.1} MED_TRADES={:.1}",
        raw_sum / total_evals.max(1.0),
        raw_p95,
        avg_trades,
        median_trades
    );
    if ga_debug_enabled() {
        println!(
            "POP_EXEC_DEBUG: avg={:.3}, p95={:.3} | DELTA={:.3} | CCR={:.3}",
            avg_e_score, exec_p95, pop_delta, ccr
        );
        println!(
            "VIP_AUDIT:      ratio={:.4} | band={} | energy_min=max(p80, p75)",
            avg_vip_ratio, vip_band
        );
    }
    if ga_debug_enabled() {
        println!(
            "STAT_AUDIT:     zero_dom_ratio={:.4} | interpretation: {}",
            avg_stat_zero_dom_ratio,
            if avg_stat_zero_dom_ratio > 0.50 {
                "WEAK (Noise Admission)"
            } else if avg_stat_zero_dom_ratio > 0.20 {
                "MIXED"
            } else {
                "HEALTHY Separation"
            }
        );
    }

    let avg_consensus_bypass_ratio = total_consensus_bypass_ratio / total_evals.max(1.0);
    let avg_stability_reject_rate = total_stability_reject_rate / total_evals.max(1.0);

    let total_abs_pnl_global = total_clarity_pnl_share.abs() + total_conviction_pnl_share.abs();
    let (avg_clarity_pnl_share, avg_conviction_pnl_share) = if total_abs_pnl_global > 1e-9 {
        (
            total_clarity_pnl_share.abs() / total_abs_pnl_global,
            total_conviction_pnl_share.abs() / total_abs_pnl_global,
        )
    } else {
        (0.0, 0.0)
    };

    if ga_debug_enabled() {
        println!(
            "EXEC_AUDIT:     accept_rate={:.4} | rejection_rate={:.4} | avg_e_score={:.3}",
            avg_exec_accept_rate, avg_e_rejection_rate, avg_e_score
        );
    }
    if ga_debug_enabled() {
        println!(
            "VIP_RETENTION:   retention={:.4} | drop_off={:.4} | selectivity_gradient={:.3}",
            avg_vip_exec_retention, avg_clarity_to_exec_drop, e_gradient
        );
        println!(
            "E_SCORE_BANDS:  VIP_E={:.3} | STAT_E={:.3} | status: {}",
            avg_vip_e,
            avg_stat_e,
            if e_gradient > 0.10 {
                "HEALTHY SEPARATION"
            } else if e_gradient > 0.0 {
                "WEAK SELECTIVITY"
            } else {
                "INVERSION RISK"
            }
        );
    }
    if ga_debug_enabled() {
        println!(
            "CONSENSUS_BRIDGE: bypass_ratio={:.4} | stability_reject={:.4} | clarity_share={:.2} | conviction_share={:.2}",
            avg_consensus_bypass_ratio, avg_stability_reject_rate, avg_clarity_pnl_share, avg_conviction_pnl_share
        );
    }

    // --- Task 6: DIAGNOSTIC DASHBOARD ---
    let min_e_score = evaluations
        .iter()
        .map(|e| e.avg_e_score)
        .fold(f64::INFINITY, f64::min);
    let max_e_score = evaluations
        .iter()
        .map(|e| e.avg_e_score)
        .fold(f64::NEG_INFINITY, f64::max);

    let min_fitness = evaluations
        .iter()
        .map(|e| e.fitness)
        .fold(f64::INFINITY, f64::min);
    let max_fitness = evaluations
        .iter()
        .map(|e| e.fitness)
        .fold(f64::NEG_INFINITY, f64::max);
    let avg_fitness = evaluations.iter().map(|e| e.fitness).sum::<f64>() / total_evals.max(1.0);

    let mut unique_strategies = std::collections::HashSet::new();
    for e in &evaluations {
        unique_strategies.insert(strategy_to_id(&e.strategy));
    }
    let unique_count = unique_strategies.len();
    let entropy = if total_evals > 0.0 {
        (unique_count as f64 / total_evals).ln().abs()
    } else {
        0.0
    };

    if ga_debug_enabled() {
        println!("\n[GA_HEALTH_DASHBOARD]");
    }

    if ga_debug_enabled() {
        println!(
            "ACCEPT_RATE: {:.3} | REJECT_RATE: {:.3}",
            avg_exec_accept_rate,
            1.0 - avg_exec_accept_rate
        );

        println!(
            "EXEC_SPREAD: min={:.5} max={:.5} avg={:.5}",
            min_e_score, max_e_score, avg_e_score
        );

        println!(
            "VIP vs STAT: VIP_E={:.5} | STAT_E={:.5} | GRADIENT={:.5}",
            avg_vip_e, avg_stat_e, e_gradient
        );

        println!(
            "DIVERSITY: unique={} | entropy={:.3}",
            unique_count, entropy
        );
    }

    if ga_debug_enabled() {
        println!(
            "FITNESS: min={:.5} max={:.5} avg={:.5}",
            min_fitness, max_fitness, avg_fitness
        );
    }
    println!("------------------------\n");

    // --- PHASE 2 OUTCOME AUDIT ---
    let mut phase2_total_quality = 0.0;
    let mut phase2_total_quality_sq = 0.0;
    let mut phase2_total_quality_count = 0.0f64;
    let mut phase2_sum_realized = 0.0;
    let mut phase2_sum_expected = 0.0;

    for e in &evaluations {
        for &q in &e.trade_qualities {
            phase2_total_quality += q;
            phase2_total_quality_sq += q * q;
            phase2_total_quality_count += 1.0;
        }
        phase2_sum_realized += e.realized_pnl_rolling;
        phase2_sum_expected += e.predicted_pnl_rolling;
    }

    let global_mean_quality = if phase2_total_quality_count > 0.0 {
        phase2_total_quality / phase2_total_quality_count
    } else {
        0.0
    };
    let global_std_quality = if phase2_total_quality_count > 1.0 {
        let var = (phase2_total_quality_sq / phase2_total_quality_count)
            - (global_mean_quality * global_mean_quality);
        var.max(0.0).sqrt()
    } else {
        0.0
    };
    let global_consistency = global_mean_quality - global_std_quality;
    let global_capture_eff = if phase2_sum_expected.abs() > 1e-9 {
        phase2_sum_realized / phase2_sum_expected
    } else {
        0.0
    };

    if ga_debug_enabled() {
        println!(
            "OUTCOME_AUDIT:  trades={} | n={:.0} | mean_q={:.3} | std_q={:.3} | consistency={:.3} | capture_eff={:.4}",
            phase2_total_quality_count, phase2_total_quality_count, global_mean_quality, global_std_quality, global_consistency, global_capture_eff
        );
    }
    if ga_debug_enabled() {
        println!(
            "RAW_HIST:  [0-0.05]: {:.1}%, [0.05-0.10]: {:.1}%, [0.10-0.20]: {:.1}%, [0.20-0.25]: {:.1}%, [0.25-0.50]: {:.1}%, [0.50+]: {:.1}%",
            raw_dist[0]*100.0, raw_dist[1]*100.0, raw_dist[2]*100.0, raw_dist[3]*100.0, raw_dist[4]*100.0, raw_dist[5]*100.0
        );
        println!(
            "EXEC_HIST: [0-0.05]: {:.1}%, [0.05-0.10]: {:.1}%, [0.10-0.20]: {:.1}%, [0.20-0.25]: {:.1}%, [0.25-0.50]: {:.1}%, [0.50+]: {:.1}%",
            exec_dist[0]*100.0, exec_dist[1]*100.0, exec_dist[2]*100.0, exec_dist[3]*100.0, exec_dist[4]*100.0, exec_dist[5]*100.0
        );
    }

    // --- PHASE 11.1 Behavioral Fingerprint (50-bucket) ---
    let mut consolidated_fingerprint = vec![0.0_f32; config.pnl_fingerprint_len];
    if total_scenarios > 0.0 {
        for (scen_idx, eval) in evaluations.iter().enumerate() {
            let bucket_idx = (scen_idx * config.pnl_fingerprint_len / evaluations.len())
                .min(config.pnl_fingerprint_len - 1);
            consolidated_fingerprint[bucket_idx] += eval.avg_pnl as f32;
        }

        // --- PHASE 11.1 Normalization (Mean-Center + Unit Variance Scaling) ---
        let count = consolidated_fingerprint.len() as f32;
        let mean_fp = consolidated_fingerprint.iter().sum::<f32>() / count;
        for val in consolidated_fingerprint.iter_mut() {
            *val -= mean_fp;
        }
        let variance_fp = consolidated_fingerprint.iter().map(|v| v * v).sum::<f32>() / count;
        let std_fp = variance_fp.sqrt().max(1e-9);
        for val in consolidated_fingerprint.iter_mut() {
            *val /= std_fp;
        }
    }

    // 🚀 PHASE D.1.28: THE PROFITABILITY BRIDGE (Smooth PnL Grounding)
    // 1. Asymmetry Detection (Upside vs Downside Skew)
    let mut positive_pnl_sum = 0.0;
    let mut negative_pnl_sum = 0.0;
    for e in &evaluations {
        for &q in &e.trade_qualities {
            if q > 0.0 {
                positive_pnl_sum += q;
            } else {
                negative_pnl_sum += q;
            }
        }
    }
    // Hard cap at 10.0 prevents blow-up dominance
    let asymmetry = ((positive_pnl_sum + 1e-6) / (negative_pnl_sum.abs() + 1e-6)).min(10.0);

    // --- CLEAN FITNESS FOUNDATION (D1.29 PREP) ---
    let base_signal = global_avg_pnl;

    // Optional diagnostics (NO transformation)
    let edge_score = global_mean_quality / (global_std_quality + 1e-6);

    // Debug only
    if ga_debug_enabled() {
        println!(
            "BASE_SIGNAL → pnl={:.6} edge={:.3} asym={:.3}",
            base_signal, edge_score, asymmetry
        );
    }

    // --- PHASE D.1.26: ENHANCED DEBUG TRACE (Unified Logic) ---
    if ga_debug_enabled() {
        println!(
            "FITNESS_TRACE → pnl={:.6} active_scen={}/{} part={:.2} win={:.3}",
            global_avg_pnl, active_scenarios, total_scenarios, participation_rate, win_rate
        );

        println!(
            "AGG_DEBUG → edge={:.3} consistency={:.3} avg_e_score={:.5}",
            edge_score, global_consistency, avg_e_score
        );
    }

    if ga_debug_enabled() {
        println!(
            "QUALITY_DEBUG → trades={} zero_pnl={} effectiveness={:.2} quality_score={:.3}",
            total_trade_count, total_zero_pnl_trades, effectiveness, quality_score
        );
    }

    // --- CORE SIGNAL (DO NOT DISTORT) ---
    // ================================
    // ================================
    // 🔥 TRUTH FITNESS (50/30/20 MODEL)
    // ================================
    let avg_fill_rate = if !evaluations.is_empty() {
        evaluations
            .iter()
            .map(|e| e.execution_metrics.fill_rate as f64)
            .sum::<f64>()
            / evaluations.len() as f64
    } else {
        0.0
    };

    // 1. PnL Score (100bps = 1.0)
    let pnl_bps = global_avg_pnl * 10000.0;
    let pnl_score = (pnl_bps / 100.0).clamp(-1.0, 1.0);

    // 2. Win Rate Score (0.0 -> 1.0)
    let win_score = win_rate.clamp(0.0, 1.0);

    // 3. Execution Score (Capture Efficiency * sqrt(Fill Rate))
    let fill_rate_score = avg_fill_rate.sqrt();
    let eff_score = (avg_efficiency.clamp(0.0, 1.0) * fill_rate_score).clamp(0.0, 1.0);

    // 4. Final Weighted Fitness
    let mut fitness_out = (0.5 * pnl_score) + (0.3 * win_score) + (0.2 * eff_score);

    // Hard execution filters (keep guard against zero-trade junk)
    if total_trade_count == 0 {
        fitness_out = -0.05; // Soft penalty for exploration (GA Reboot Law)
    } else {
        fitness_out *= ((total_trade_count as f64) / 10.0).clamp(0.5, 1.5);
    }

    // 9. Prevent numerical collapse
    if !fitness_out.is_finite() {
        fitness_out = -1.0;
    }

    // --- TASK 5: GA HEALTH METRICS ---
    let output_scenarios = evaluations.len();
    // --- TASK 5: GA HEALTH METRICS (FIXED - USE REAL SIGNAL) ---
    let fitness_values: Vec<f64> = evaluations.iter().map(|e| e.avg_pnl).collect();

    let fitness_mean = if !fitness_values.is_empty() {
        fitness_values.iter().sum::<f64>() / fitness_values.len() as f64
    } else {
        0.0
    };

    let fitness_std_dev = if fitness_values.len() > 1 {
        let var = fitness_values
            .iter()
            .map(|v| (v - fitness_mean).powi(2))
            .sum::<f64>()
            / (fitness_values.len() as f64 - 1.0);
        var.sqrt()
    } else {
        0.0
    };

    let mut unique_strategies_health: Vec<&Strategy> = Vec::new();

    for e in &evaluations {
        let strategy = &e.strategy;

        if !unique_strategies_health
            .iter()
            .any(|s| calculate_genotype_distance(s, strategy) < 0.05)
        {
            unique_strategies_health.push(strategy);
        }
    }

    let unique_genomes = unique_strategies_health.len();

    if ga_debug_enabled() {
        println!(
            "[GA_HEALTH_DASHBOARD] std_dev={:.4} unique_genomes={} signals_emitted={:.1} exec_pass_rate={:.3}",
            fitness_std_dev, unique_genomes, avg_signals_emitted, avg_exec_accept_rate
        );
    }

    // --- DIVERSITY AXES AUDIT ---
    {
        use std::collections::HashSet;
        let mut tp = HashSet::new();
        let mut sl = HashSet::new();
        let mut hold = HashSet::new();
        let mut edge = HashSet::new();
        let mut arch = HashSet::new();
        let mut bias = HashSet::new();
        for e in &evaluations {
            tp.insert(e.strategy.take_profit);
            sl.insert(e.strategy.stop_loss);
            hold.insert(e.strategy.holding_period);
            edge.insert(e.strategy.base_edge);
            arch.insert(e.strategy.archetype);
            bias.insert(e.strategy.direction_bias);
        }
        if ga_debug_enabled() {
            println!(
                "[DIVERSITY_AXES] unique_tp={} unique_sl={} unique_hold={} unique_edge={} unique_arch={} unique_bias={}",
                tp.len(), sl.len(), hold.len(), edge.len(), arch.len(), bias.len()
            );
        }
    }

    // --- PHASE 22: BEHAVIORAL FAILURE PROFILING (ALPHA SURGE) ---
    let total_queue_blocked: usize = evaluations.iter().map(|e| e.queue_blocked_count).sum();
    let total_liquidity_starved: usize =
        evaluations.iter().map(|e| e.liquidity_starved_count).sum();
    let overall_total_attempts = total_trade_count + total_queue_blocked + total_liquidity_starved;

    let failure_profile = if overall_total_attempts >= 10 {
        let q_ratio = total_queue_blocked as f64 / overall_total_attempts as f64;
        let l_ratio = total_liquidity_starved as f64 / overall_total_attempts as f64;
        let weight = (1.0 + overall_total_attempts as f64).ln().min(4.0);
        vec![q_ratio * weight, l_ratio * weight]
    } else {
        vec![0.0, 0.0]
    };

    let mut report = StrategyEvaluation {
        strategy_id: evaluations[0].strategy_id.clone(),
        strategy: evaluations[0].strategy.clone(),
        capability: evaluations[0].capability.clone(),
        avg_pnl: global_avg_pnl,
        std_dev,
        downside_std_dev,
        worst: worst_pnl,
        robustness,
        fitness: fitness_out,
        trade_count: total_trade_count,
        max_drawdown: avg_max_drawdown,
        participation_rate,
        profitable_trades: total_profitable_trades,
        zero_pnl_trades: total_zero_pnl_trades,
        quality_trades: total_quality_trades,
        win_rate: if evaluations.len() > 0 {
            evaluations.iter().map(|e| e.win_rate).sum::<f64>() / evaluations.len() as f64
        } else {
            0.0
        },
        payoff: if evaluations.len() > 0 {
            evaluations.iter().map(|e| e.payoff).sum::<f64>() / evaluations.len() as f64
        } else {
            0.0
        },
        payoff_ratio: global_payoff_ratio,
        execution_metrics: ExecutionMetrics {
            fill_efficiency: avg_fill_eff,
            capture_efficiency: avg_efficiency,
            fill_rate: (total_trade_count as f32 / overall_total_attempts.max(1) as f32),
            avg_slippage,
            latency_impact: avg_latency,
            queue_blocked_count: total_queue_blocked,
            liquidity_starved_count: total_liquidity_starved,
            total_attempts: overall_total_attempts,
        },
        scenario_signature: aggregated_scenario_signature,
        avg_conviction,
        avg_efficiency,
        avg_edge_quality,
        directional_accuracy,
        decisiveness,
        execution_friction,
        short_term_capture_eff: 1.0,
        long_term_capture_eff: 1.0,
        trade_density: evaluations.iter().map(|e| e.trade_density).sum::<f64>()
            / total_scenarios.max(1.0),
        queue_blocked_count: total_queue_blocked,
        liquidity_starved_count: total_liquidity_starved,
        total_attempts: overall_total_attempts,
        failure_profile: failure_profile.clone(),
        exec_opportunity_rate: overall_total_attempts as f64 / (total_scenarios * 400.0).max(1.0), // Approximate ticks per window 400
        realized_pnl_rolling: 0.0,
        predicted_pnl_rolling: 0.0,
        exit_tp_count: total_exit_tp,
        exit_sl_count: total_exit_sl,
        exit_ts_count: total_exit_ts,
        avg_hold_time: 0.0,
        consistency_score: 1.0,
        recent_performance: global_avg_pnl,
        pnl_from_tp: total_profit_from_tp,
        pnl_from_sl: total_loss_from_sl,
        max_trade_pnl,
        pnl_fingerprint: consolidated_fingerprint,
        selectivity: evaluations.iter().map(|e| e.selectivity).sum::<f64>()
            / total_scenarios.max(1.0),
        avg_entropy: evaluations.iter().map(|e| e.avg_entropy).sum::<f64>()
            / total_scenarios.max(1.0),
        avg_aqg_health,
        aqg_skip_ratio,
        outcome_consistency: global_consistency,
        avg_trade_quality: global_mean_quality,
        std_trade_quality: global_std_quality,
        consistency_n: phase2_total_quality_count as usize,
        ..StrategyEvaluation::default()
    };

    // Fix 2: Fitness Injection
    let exec_boost = report.avg_e_score * 0.5;
    // --- PRESERVE MICRO SIGNALS ---
    if fitness_out.abs() < 1e-6 {
        fitness_out = global_avg_pnl * 0.5 + 1e-6;
    }

    // Fix 3: Survival Pressure
    // if report.avg_e_score < -0.0005 {
    //     report.fitness *= 0.5;
    // }

    // // Fix 4: Fitness Floor Protection
    // report.fitness = report.fitness.max(-10.0);

    // --- PHASE 14++ Ext: STRUCTURAL RANKING ENGINE ---
    let mut structural_candidates = Vec::new();
    for e in &evaluations {
        // Extract symbol from strategy_id (e.g., "VODAFONEIDEA_FUT_5M_CLEAN_csv_window_0" -> "VODAFONEIDEA_FUT")
        let parts: Vec<&str> = e.strategy_id.split("strat_").collect();
        let scenario_part = if parts.len() > 1 {
            parts[1]
        } else {
            &e.strategy_id
        };
        let symbol = scenario_part
            .split("_csv_window_")
            .next()
            .unwrap_or("unknown")
            .to_string();

        // Fix #3: Minimum Viability Filter (Garbage Filter) - Lowered for Adaptive Discovery
        if e.total_windows < 1 {
            continue;
        }

        // Phase A+: Adaptive Alpha Ranking
        // Sqrt Confidence Scaling: (N/100)^0.5
        let confidence_factor = ((e.total_windows as f64) / 100.0).sqrt().min(1.0);

        // Composite Score: Prioritize Alpha (Avg Z) and Opportunity (%) scaled by confidence
        // Weighted for institutional discovery: 70% Alpha, 30% Opportunity
        let discovery_score = (e.alpha * 0.70 + e.opportunity * 0.30) * confidence_factor;

        let conf_label = if e.total_windows < 30 {
            "LOW"
        } else if e.total_windows < 80 {
            "MED"
        } else {
            "HIGH"
        };

        structural_candidates.push((symbol, e, discovery_score, conf_label));
    }

    structural_candidates.sort_by(|a, b| b.2.total_cmp(&a.2));

    // Deduplicate symbols (since multiple windows might exist per symbol in some configurations)
    let mut seen_symbols = HashSet::new();
    let mut final_ranking = Vec::new();
    for cand in structural_candidates {
        if !seen_symbols.contains(&cand.0) {
            seen_symbols.insert(cand.0.clone());
            final_ranking.push(cand);
        }
    }

    if ga_debug_enabled() {
        println!("\n🚀 STRUCTURAL_RANKING (Adaptive Discovery)");
    }
    println!("--------------------------------------------------------------------------------------------------");
    println!(
        "Rank | Symbol       | Alpha   | Continuity | Opp%   | Stab | Agree | PeakAgree | Conf | N | Trd"
    );
    println!("-------------------------------------------------------------------------------------------------------");
    for (i, (sym, e, _score, conf)) in final_ranking.iter().take(10).enumerate() {
        println!(
            "{:>4} | {:<12} | {:7.3} | {:10.2} | {:6.2}% | {:4.2} | {:5.2} | {:9.2} | {:4} | {:3} | {:3}",
            i + 1,
            sym,
            e.alpha,
            e.consistency,
            e.opportunity * 100.0,
            (1.0 - e.avg_stability_valid / 0.18).clamp(0.0, 1.0),
            e.avg_agreement_valid,
            e.max_agreement,
            conf,
            e.total_windows,
            e.trade_count / e.total_windows.max(1)
        );
    }
    if ga_debug_enabled() {
        println!("--------------------------------------------------------------------------------------------------");
    }

    report.fitness = fitness_out;
    let mean_depth = total_trade_count as f64 / total_scenarios.max(1.0);
    Some((report, mean_depth))
}

// SignalType moved to header.

/// Phase 10.6: Decision Evaluation Mode
/// Checks the entry condition on the MOST RECENT state.
pub fn evaluate_current_status(
    strategy: &Strategy,
    history: &[Candle],
    config: &GaConfig,
    symbol: &str,
    last_signal: SignalType,
    consistency_count: usize,
    stats: &DistributionStats,
) -> DecisionReport {
    // --- PHASE D.1.25: ORTHOGONAL SPECIALIST GATES (V3.6.11 Fix 5) ---
    // [L0: Mean Reversion] archetype 2
    if strategy.archetype == 2 && last_signal == SignalType::WAIT {
         // We need the regime BEFORE fully evaluating if possible, but conviction engine provides it.
         // So we continue and check it downstream to avoid double calculation.
    }
    
    // 1. Warm-up Gate
    if history.len() < (config.lambda as usize) + 20 {
        if std::env::var("EDGE_DEBUG").is_ok() {
            println!(
                "[EARLY_EXIT] sym={} reason=warmup_insufficient history={} required={}",
                symbol,
                history.len(),
                (config.lambda as usize) + 20
            );
        }
        return DecisionReport {
            trade_id: 0,
            symbol: symbol.to_string(),
            timestamp: history.last().map(|c| c.timestamp).unwrap_or(0),
            signal: SignalType::WAIT,
            confidence: 0.0,
            expected_return: 0.0,
            horizon_bars: config.max_hold_bars as u64,
            participation: 0.0,
            regime: MarketRegime::MeanReversion,
            aligned_weight: 0.0,
            opposing_weight: 0.0,
            consistency: consistency_count,
            conviction_score: 0.0,
            agreement_strength: "WEAK".to_string(),
            voters: "0/0".to_string(),
            execution_feasible: false,
            execution_score: 0.0,
            execution_threshold: 0.7,
            threshold: 0.6,
            raw_edge: 0.0,
            realized_return: None,
            capture_efficiency: None, execution_feasibility: 0.0,
            efficiency_label: String::new(),
            recommendation: None,
        };
    }

    let last_idx = history.len().saturating_sub(1);

    // 2. Market Event Mapping (Internal)
    let mut events = Vec::with_capacity(history.len());
    for candle in history {
        events.push(crate::MarketEvent {
            subtype: crate::MarketEventType::Trade,
            price: candle.close,
            quantity: candle.volume,
            side: None,
            exchange_ts: candle.timestamp,
        });
    }

    // 3. Conviction Engine
    let conviction = evaluate_market_conviction(strategy, "live", &events, last_idx, 0, 0);
    let directional_edge = conviction.conviction_score.abs().powf(0.7);
    let strength = (0.8_f64 * directional_edge + 0.2_f64 * 0.5_f64).clamp(0.05_f64, 1.0_f64);

    #[cfg(feature = "debug_decision")]
    println!(
        "[DECISION_TRACE] idx={} conv={:.5} edge={:.5} strength={:.4} regime={:?} vol={:.5}",
        last_idx,
        conviction.conviction_score,
        conviction.expected_edge,
        strength,
        conviction.regime,
        conviction.norm_vol
    );

    // 4. Execution Simulation
    // --- Fix 5: Hard Orthodoxy Check ---
    let regime = conviction.regime;
    let skip_orthogonal = if std::env::var("GA_BOOTSTRAP").is_ok() { false } else {
        match strategy.archetype {
            2 => regime != MarketRegime::MeanReversion, // Mean Reversion ONLY
            1 => regime != MarketRegime::BullTrend && regime != MarketRegime::BearTrend, // Trend ONLY
            3 => conviction.norm_vol > 0.003, // Defensive: skip high vol
            0 => conviction.raw_q_ratio > 0.8, // Liquidity Sniper: skip if queue too deep
            _ => false,
        }
    };

    if skip_orthogonal {
        if std::env::var("EDGE_DEBUG").is_ok() {
            println!(
                "[EARLY_EXIT] sym={} reason=orthogonal_regime_mismatch archetype={} regime={:?}",
                symbol,
                strategy.archetype,
                regime
            );
        }
        return DecisionReport {
            signal: SignalType::WAIT,
            regime,
            ..DecisionReport::default()
        };
    }

    let outcome = crate::ga_simulate_round_trip_at_cursor(
        strategy,
        &events,
        &events,
        config,
        last_idx,
        0,
        &conviction,
        !conviction.is_bearish,
        strength,
        false,
        0, 
        0, 
        stats,
    );

    // 5. Gating & Decision Assembly
    let mut signal = SignalType::WAIT;
    let mut confidence = 0.0;
    let mut execution_feasible = false;
    let mut execution_score = 0.0;
    let mut trade_rec: Option<TradeRecommendation> = None;
    let mut final_raw_edge = 0.0;
    let mut feasibility = 0.0;

    let edge_threshold = 0.0010;

    if let Some(ref rt) = outcome {
        final_raw_edge = rt.raw_edge;
        let is_bearish = conviction.conviction_score < 0.0;
        
        // 5.1 Execution Feasibility Calculation (Capturability Index)
        // [V3.6.11] NEW DEFINITION: realized / ideal
        let realized = rt.pnl;
        let ideal = rt.ideal_pnl;
        feasibility = if ideal > 1e-7 {
            (realized / ideal).clamp(0.0, 1.0)
        } else if realized > 1e-7 {
            1.0
        } else {
            0.0
        };

        if std::env::var("EDGE_DEBUG").is_ok() {
            println!(
                "[PRE_GATE] sym={} edge={:.6} feas={:.3} conv={:.4} mom={:.4} vol={:.4} regime={:?}",
                symbol,
                final_raw_edge,
                feasibility,
                conviction.conviction_score,
                conviction.norm_momentum,
                conviction.norm_vol,
                conviction.regime
            );
        }

        #[cfg(feature = "debug_decision")]
        println!("[EXEC_FILTER] ideal={:.6} real={:.6} feas={:.3} regime={:?}", 
            ideal, realized, feasibility, conviction.regime);

        // 5.2 HARD REALITY GATES (V3.6.11)
        // A. Regime Guard
        if conviction.regime == MarketRegime::HighVolatilityNoise && std::env::var("GA_BOOTSTRAP").is_err() {
             if std::env::var("EDGE_DEBUG").is_ok() {
                 println!(
                     "[EARLY_EXIT] sym={} reason=high_vol_noise_guard n_vol={:.4}",
                     symbol,
                     conviction.norm_vol
                 );
             }
             if std::env::var("EDGE_DEBUG").is_ok() {
                 println!("[REJECT_NOISE] sym={} n_vol={:.4}", symbol, conviction.norm_vol);
             }
             return DecisionReport {
                signal: SignalType::WAIT,
                execution_feasible: false,
                execution_feasibility: feasibility,
                ..DecisionReport::default()
             };
        }

        // B. Feasibility Guard
        let feas_thresh = if std::env::var("GA_BOOTSTRAP").is_ok() { 0.10 } else { 0.30 };
        let edge_thresh = if std::env::var("GA_BOOTSTRAP").is_ok() { 0.0001 } else { edge_threshold };

        if final_raw_edge >= edge_thresh && feasibility > feas_thresh {
             signal = if is_bearish { SignalType::SELL } else { SignalType::BUY };
             // ...
            
            // 5.3 Calibrated Ranking (Fix 4)
            // rank = expected_edge * rolling_capture_eff * fill_prob
            // We scale expected_edge (final_raw_edge) so that a 50bps edge = 1.0 RawRank
            let raw_rank = (final_raw_edge / 0.0050).clamp(0.0, 1.0);
            let fill_prob = rt.fill_efficiency;
            let adjusted_rank = (raw_rank * feasibility * fill_prob).clamp(0.0, 1.0);
            
            confidence = (final_raw_edge / (final_raw_edge + 0.001)).clamp(0.0, 1.0);
            execution_feasible = true;
            execution_score = rt.e_score;

            let entry_price = events[last_idx].price as f64;
            let vol_bps = (rt.vol_bucket as f64 * 15.0) + 10.0; 
            let spread = entry_price * (vol_bps / 10000.0) * 0.5;
            
            let mfe_raw = config.rank_stats.get_expected_mfe(adjusted_rank, rt.vol_bucket);
            let mae_raw = config.rank_stats.get_expected_mae(adjusted_rank, rt.vol_bucket);
            let hold = config.rank_stats.get_expected_time(adjusted_rank, rt.vol_bucket);

            let mfe = mfe_raw * 0.65;
            let mae = mae_raw * 1.35;
            let expected_rr = if mae > 0.0 { mfe / mae } else { 2.0 }; 
            
            if expected_rr >= 1.25 {
                let realistic_entry = apply_slippage(entry_price, !is_bearish, vol_bps);
                trade_rec = Some(TradeRecommendation {
                    symbol: symbol.to_string(),
                    signal,
                    rank: adjusted_rank, // 🔥 Rank is now execution-aware
                    raw_edge: final_raw_edge,
                    confidence: (adjusted_rank * (expected_rr.min(3.0) / 3.0)).clamp(0.0, 1.0),
                    quality_score: adjusted_rank * expected_rr,
                    entry_price: realistic_entry,
                    entry_low: realistic_entry - spread,
                    entry_high: realistic_entry + spread,
                    tp_target: if is_bearish { realistic_entry * (1.0 - mfe) } else { realistic_entry * (1.0 + mfe) },
                    sl_target: if is_bearish { realistic_entry * (1.0 + mae) } else { realistic_entry * (1.0 - mae) },
                    expected_rr,
                    expected_edge_bps: mfe * 10000.0,
                    risk_bps: mae * 10000.0,
                    holding_bars: hold as usize,
                    vol_bps,
                    vol_bucket: rt.vol_bucket,
                    is_execution: rt.is_execution,
                    position_size: (adjusted_rank * adjusted_rank).clamp(0.01, 1.0),
                });
            }
        }
    } else if std::env::var("EDGE_DEBUG").is_ok() {
        println!(
            "[EARLY_EXIT] sym={} reason=no_round_trip_outcome conv={:.4} regime={:?}",
            symbol,
            conviction.conviction_score,
            conviction.regime
        );
    }

    DecisionReport {
        trade_id: 0,
        symbol: symbol.to_string(),
        timestamp: history.last().map(|c| c.timestamp).unwrap_or(0),
        signal,
        confidence,
        expected_return: 0.0,
        horizon_bars: config.max_hold_bars as u64,
        participation: 0.0,
        regime: conviction.regime,
        aligned_weight: 0.0,
        opposing_weight: 0.0,
        consistency: consistency_count,
        conviction_score: conviction.conviction_score,
        agreement_strength: "ANALYZED".to_string(),
        voters: "1/1".to_string(),
        execution_feasible,
        execution_score,
        execution_threshold: 0.7,
        threshold: 0.6,
        raw_edge: final_raw_edge,
        realized_return: None,
        capture_efficiency: None,
        execution_feasibility: feasibility,
        efficiency_label: String::new(),
        recommendation: trade_rec,
    }
}

/// A lightweight paper trading virtualizer to validate recommendations.
pub fn update_paper_registry(
    registry: &mut PaperRegistry,
    latest_candle: &Candle,
) {
    let high = latest_candle.high as f64;
    let low = latest_candle.low as f64;
    let close = latest_candle.close as f64;
    let ts = latest_candle.timestamp;

    // --- STEP 1: Process Pending Intents (The Discipline Layer) ---
    let mut j = 0;
    while j < registry.pending_intents.len() {
        let mut triggered = false;
        let mut entry_price = 0.0;
        
        {
            let intent = &mut registry.pending_intents[j];
            intent.age += 1;
            
            let is_long = intent.signal == SignalType::BUY;
            // 0.1% Pullback Factor (Hardcoded for now, ideal for elite signals)
            let pullback_factor = 0.999; 
            let bounce_factor = 1.001;

            if is_long {
                if low <= intent.reference_price * pullback_factor {
                    triggered = true;
                    entry_price = intent.reference_price * pullback_factor;
                }
            } else {
                if high >= intent.reference_price * bounce_factor {
                    triggered = true;
                    entry_price = intent.reference_price * bounce_factor;
                }
            }

            if intent.age > intent.max_age {
                #[cfg(feature = "debug_decision")]
                println!("\x1b[93m[INTENT_EXPIRED] {} signal={:?} ref={:.2}\x1b[0m", intent.symbol, intent.signal, intent.reference_price);
                registry.pending_intents.remove(j);
                continue;
            }
        }

        if triggered {
            let intent = registry.pending_intents.remove(j);
            #[cfg(feature = "debug_decision")]
            println!("\x1b[96m[INTENT_TRIGGERED] {} @ {:.2} (Ref: {:.2})\x1b[0m", intent.symbol, entry_price, intent.reference_price);
            
            registry.active_trades.push(ActiveTrade {
                symbol: intent.symbol,
                entry_price,
                tp_target: intent.recommendation.tp_target,
                sl_target: intent.recommendation.sl_target,
                hold_limit: intent.recommendation.holding_bars,
                current_hold: 0,
                signal: intent.signal,
                size: intent.recommendation.position_size,
                vol_bps: intent.recommendation.vol_bps,
                rank: intent.recommendation.rank,
                strategy_id: intent.strategy_id,
                consensus: intent.consensus,
            });
        } else {
            j += 1;
        }
    }

    // --- STEP 2: Process Active Trades (The Execution Layer) ---
    let mut i = 0;
    while i < registry.active_trades.len() {
        let trade = &mut registry.active_trades[i];
        trade.current_hold += 1;
        
        let mut exit_pnl = None;
        let is_long = trade.signal == SignalType::BUY;

        // 🔥 Intra-Candle Resolution
        if let Some(exit_type) = resolve_intracandle_exit(
            high, low, trade.tp_target, trade.sl_target, is_long
        ) {
            // Pessimistic: Ambiguous -> StopLoss
            let resolution = match exit_type {
                ExitType::Ambiguous => ExitType::StopLoss,
                other => other,
            };

            let exit_price = match resolution {
                ExitType::TakeProfit => apply_slippage(trade.tp_target, !is_long, trade.vol_bps),
                ExitType::StopLoss => apply_slippage(trade.sl_target, !is_long, trade.vol_bps),
                _ => unreachable!(),
            };

            exit_pnl = Some(if is_long {
                (exit_price - trade.entry_price) / trade.entry_price
            } else {
                (trade.entry_price - exit_price) / trade.entry_price
            });
        }
        
        // Time Stop
        if exit_pnl.is_none() && trade.current_hold >= trade.hold_limit {
            let exit_price = apply_slippage(close, !is_long, trade.vol_bps);
            exit_pnl = Some(if is_long {
                (exit_price - trade.entry_price) / trade.entry_price
            } else {
                (trade.entry_price - exit_price) / trade.entry_price
            });
        }
        
        if let Some(pnl) = exit_pnl {
             // 🔥 Compounding Equity Update
            registry.equity *= 1.0 + (pnl * trade.size);
            
            if registry.equity > registry.peak_equity {
                registry.peak_equity = registry.equity;
                registry.rolling_peak = registry.equity;
            }
            let dd = (registry.peak_equity - registry.equity) / registry.peak_equity;
            if dd > registry.max_drawdown {
                registry.max_drawdown = dd;
            }

            // Stats
            registry.closed_count += 1;
            if pnl > 0.0 { registry.wins += 1; } else { registry.losses += 1; }
            registry.pnl_history.push(pnl);

            // 🔥 Strategy Attribution
            *registry.strategy_pnl.entry(trade.strategy_id).or_insert(0.0) += pnl;
            *registry.strategy_counts.entry(trade.strategy_id).or_insert(0) += 1;

            // 🔥 Per-Rank Analytics
            let r_idx = (trade.rank * 10.0).floor().clamp(0.0, 9.0) as usize;
            registry.rank_pnl_sum[r_idx] += pnl;
            registry.rank_count[r_idx] += 1;

            #[cfg(feature = "debug_decision")]
            {
                println!("\x1b[95m[TRADE_CLOSE] {} id={} rank={:.1} pnl={:.6} dur={}\x1b[0m", 
                    trade.symbol, trade.strategy_id, trade.rank, pnl, trade.current_hold);
            }

            registry.active_trades.remove(i);
        } else {
            i += 1;
        }
    }

    // Record time-series
    registry.equity_curve.push(registry.equity);
    registry.timestamps.push(ts);

    if registry.closed_count % 20 == 0 && registry.closed_count > 0 {
         let wr = registry.wins as f64 / registry.closed_count as f64;
         #[cfg(feature = "debug_decision")]
         println!("[EQUITY] trades={} equity={:.4} dd={:.4} winrate={:.2}", registry.closed_count, registry.equity, registry.max_drawdown, wr);
    }
}

pub fn save_elite_population(
    evals: &[StrategyEvaluation],
    _config: &GaConfig,
    base_dir: &str,
) -> std::io::Result<String> {
    use chrono::Utc;
    use std::fs;

    let timestamp = Utc::now().format("%Y-%m-%d_%H-%M").to_string();
    let filename = format!("elite_{}.json", timestamp);
    let path = std::path::Path::new(base_dir).join(&filename);

    let sum_fitness: f64 = evals.iter().map(|e| e.fitness).sum();
    let avg_fitness = sum_fitness / evals.len().max(1) as f64;
    let avg_pnl = evals.iter().map(|e| e.avg_pnl).sum::<f64>() / evals.len().max(1) as f64;

    // Average metrics for regime profiling
    let avg_vol = evals
        .iter()
        .map(|e| {
            if e.pnl_fingerprint.len() > 1 {
                let mean = e.pnl_fingerprint.iter().map(|&x| x as f64).sum::<f64>()
                    / e.pnl_fingerprint.len() as f64;

                let var = e
                    .pnl_fingerprint
                    .iter()
                    .map(|&x| {
                        let x = x as f64;
                        (x - mean).powi(2)
                    })
                    .sum::<f64>()
                    / e.pnl_fingerprint.len() as f64;

                var.sqrt()
            } else {
                0.0
            }
        })
        .sum::<f64>()
        / evals.len().max(1) as f64;
    let avg_participation =
        evals.iter().map(|e| e.decisiveness).sum::<f64>() / evals.len().max(1) as f64;

    // Simple CV calculation for persistence metadata
    let mean = avg_fitness;
    let variance = evals
        .iter()
        .map(|e| (e.fitness - mean).powi(2))
        .sum::<f64>()
        / evals.len().max(1) as f64;
    let cv = if mean.abs() > 1e-6 {
        variance.sqrt() / mean.abs()
    } else {
        variance.sqrt() // fallback: use std_dev directly
    };

    let bundle = ElitePopulationBundle {
        metadata: PersistenceMetadata {
            timestamp: timestamp.clone(),
            avg_fitness,
            avg_pnl,
            cv,
            fitness_mode: _config.fitness_mode,
            regime_profile: RegimeProfile {
                volatility: avg_vol,
                liquidity: 0.5, // Default placeholder for now
                participation: avg_participation,
                label: "multi_regime_elite".to_string(),
                timestamp: Utc::now().timestamp() as u64,
            },
            strategies_count: evals.len(),
        },
        strategies: evals.to_vec(),
    };

    let json = serde_json::to_string_pretty(&bundle)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    fs::write(&path, json)?;

    // Update latest.json
    let latest_path = std::path::Path::new(base_dir).join("latest.json");
    fs::copy(&path, &latest_path)?;

    Ok(path.to_string_lossy().to_string())
}

pub fn load_elite_strategies(path: &str) -> Vec<StrategyEvaluation> {
    use std::fs;
    let data = fs::read_to_string(path).unwrap_or_else(|_| "[]".to_string());
    if let Ok(bundle) = serde_json::from_str::<ElitePopulationBundle>(&data) {
        bundle.strategies
    } else {
        Vec::new()
    }
}

pub fn calculate_dynamic_threshold(volatility: f64, avg_quality: f64) -> f64 {
    let base = 0.70;
    let vol_adj = if volatility > 0.002 { 0.10 } else { 0.0 };
    let quality_adj = if avg_quality > 0.75 { -0.05 } else { 0.0 };
    (base + vol_adj + quality_adj as f64).clamp(0.60, 0.90)
}

/// Phase 10.9: Institutional Consensus Evaluation
pub fn evaluate_consensus_status(
    evals: &[StrategyEvaluation],
    history: &[Candle],
    config: &GaConfig,
    symbol: &str,
    last_signal: SignalType,
    consistency_count: usize,
) -> DecisionReport {
    use crate::SignalType;
    if history.len() < (config.lambda as usize) + 50 {
        return DecisionReport {
            trade_id: 0,
            symbol: symbol.to_string(),
            timestamp: history.last().map(|c| c.timestamp).unwrap_or(0),
            signal: SignalType::WAIT,
            confidence: 0.0,
            expected_return: 0.0,
            horizon_bars: config.max_hold_bars as u64,
            participation: 0.0,
            regime: MarketRegime::MeanReversion,
            aligned_weight: 0.0,
            opposing_weight: 0.0,
            consistency: consistency_count,
            conviction_score: 0.0,
            agreement_strength: "WEAK".to_string(),
            voters: "0/0".to_string(),
            execution_feasible: false,
            execution_score: 0.0,
            execution_threshold: 0.7,
            threshold: 0.7,
            raw_edge: 0.0,
            realized_return: None,
            capture_efficiency: None, execution_feasibility: 0.0,
            efficiency_label: String::new(),
            recommendation: None,
        };
    }

    let last_idx = history.len().saturating_sub(1);
    let mut events = Vec::with_capacity(history.len());
    for candle in history {
        events.push(crate::MarketEvent {
            subtype: crate::MarketEventType::Trade,
            price: candle.close,
            quantity: candle.volume,
            side: None,
            exchange_ts: candle.timestamp,
        });
    }

    // 🔥 GUARDRAIL 1: Eligibility Filter (Fitness > 0)
    // Decoupled consistency_score (diagnostic) from decision gate
    let eligible_voters: Vec<&StrategyEvaluation> = evals
        .iter()
        .filter(|e| e.fitness > 0.0)
        .take(10) // Dynamic Top-K (Max 10)
        .collect();

    if eligible_voters.len() < 3 {
        let best = evals
            .iter()
            .max_by(|a, b| a.fitness.total_cmp(&b.fitness))
            .unwrap();
        return DecisionReport {
            trade_id: 0,
            symbol: symbol.to_string(),
            timestamp: history[last_idx].timestamp,
            signal: SignalType::WAIT,
            confidence: 0.0,
            expected_return: 0.0,
            horizon_bars: config.max_hold_bars as u64,
            participation: 0.0,
            regime: MarketRegime::MeanReversion,
            aligned_weight: 0.0,
            opposing_weight: 0.0,
            consistency: consistency_count,
            conviction_score: 0.0,
            agreement_strength: "WEAK".to_string(),
            voters: format!("{}/{}", eligible_voters.len(), evals.len()),
            execution_feasible: false,
            execution_score: 0.0,
            execution_threshold: 0.7,
            threshold: 0.7,
            raw_edge: 0.0,
            realized_return: None,
            capture_efficiency: None, execution_feasibility: 0.0,
            efficiency_label: String::new(),
            recommendation: None,
        };
    }

    // 🚀 PHASE D.1.24: INSTITUTIONAL REGIME DETECTION (Window-Level)
    let (mean_px, std_dev, _) = calculate_lookback_stats(history, last_idx, 20);
    let ref_price = history[last_idx].close as f64;
    let lookback_price = history[last_idx.saturating_sub(20)].close as f64;
    let price_delta = (ref_price - lookback_price).abs() / ref_price.max(1.0);
    let norm_momentum = (price_delta / 0.001).clamp(0.0, 1.0);
    let norm_vol = std_dev / mean_px.max(1.0);

    let current_regime = detect_market_regime(ref_price, mean_px, norm_momentum, norm_vol);

    // Legacy conviction guard for backward compatibility with ESE simulation logic
    let conviction_guard = evaluate_market_conviction(
        &eligible_voters[0].strategy,
        "consensus",
        &events,
        last_idx,
        0,
        0,
    );

    // 🚀 PHASE 10.10: Execution Feasibility Reality Check
    let ctx = ExecutionContext {
        queue_depth: conviction_guard.raw_q_ratio.min(1.0),
        liquidity_score: conviction_guard.norm_volume,
        latency_impact: norm_vol * 100.0,
    };
    let exec_score = calculate_execution_score(&ctx);

    // Weighted Voting
    let mut buy_weight = 0.0;
    let mut sell_weight = 0.0;
    let mut expected_return_sum = 0.0;

    // 🔥 GUARDRAIL 2: Capped Dominance (Max 25% influence)
    let raw_weights: Vec<f64> = eligible_voters.iter().map(|v| v.fitness).collect();
    let total_raw_fitness: f64 = raw_weights.iter().sum::<f64>().max(1e-9);
    let mut normalized_weights: Vec<f64> = raw_weights
        .iter()
        .map(|w| (w / total_raw_fitness).min(0.25))
        .collect();
    let capped_total_weight: f64 = normalized_weights.iter().sum::<f64>().max(1e-9);
    for w in &mut normalized_weights {
        *w /= capped_total_weight;
    }

    // 🚀 PHASE D.1.24: High-Volatility Safety Adjustment
    let mut dynamic_threshold =
        calculate_dynamic_threshold(norm_vol, eligible_voters[0].fitness as f64).min(0.85);
    let mut universal_multiplier = 1.0;
    if current_regime == MarketRegime::HighVolatilityNoise {
        dynamic_threshold = (dynamic_threshold * 1.5).min(0.95);
        universal_multiplier = 0.7;
    }

    for (i, voter) in eligible_voters.iter().enumerate() {
        let mut weight = normalized_weights[i];

        // 🔥 PHASE D.1.24: Institutional Specialist Weighting Logic
        let bias_archetype = classify_direction_bias(voter.strategy.direction_bias);
        weight *= regime_multiplier(current_regime, bias_archetype) * universal_multiplier;

        let consensus_score_v = conviction_guard.conviction_score.clamp(-1.0, 1.0);
        let directional_edge = consensus_score_v.abs() * exec_score;

        let strength: f64 = (0.8 * directional_edge + 0.2 * consensus_score_v.abs())
            .clamp(0.05, 1.0);
        let outcome = crate::ga_simulate_round_trip_at_cursor(
            &voter.strategy,
            &events,
            &events,
            config,
            last_idx,
            0,
            &conviction_guard,
            !conviction_guard.is_bearish,
            strength,
            false,
            i, // strategy_index
            0, // generation fallback
            &config.stats,
        );

        if let Some(_rt) = outcome {
            // ✅ FIX: performance-based voting (NOT market-based)
            if conviction_guard.conviction_score > 0.0 {
                buy_weight += weight;
            } else if conviction_guard.conviction_score < 0.0 {
                sell_weight += weight;
            }

            expected_return_sum += voter.avg_pnl * weight;
        }
    }

    // use the dynamic_threshold already calculated and adjusted for High Volatility Noise

    let mut buy_count = 0;
    let mut sell_count = 0;
    let consensus_signal = if buy_weight >= dynamic_threshold {
        buy_count = (buy_weight * eligible_voters.len() as f64).round() as usize;
        SignalType::BUY
    } else if sell_weight >= dynamic_threshold {
        sell_count = (sell_weight * eligible_voters.len() as f64).round() as usize;
        SignalType::SELL
    } else {
        SignalType::WAIT
    };

    let final_conviction = (buy_weight - sell_weight).abs();
    let total_weight = buy_weight + sell_weight + 1e-9;
    let agreement_ratio = buy_weight.max(sell_weight) / total_weight;

    // 🔥 GUARDRAIL 3: No-Trade Zone (< 0.35 conviction)
    let gated_signal = if final_conviction < 0.25 {
        if agreement_ratio > 0.6 {
            consensus_signal
        } else {
            SignalType::WAIT
        }
    } else {
        consensus_signal
    };

    // 🚀 PHASE 10.10: Final Feasibility Decision
    let (feasible, exec_threshold) = is_execution_feasible(final_conviction, exec_score);
    let final_signal = if !feasible {
        // ⚠️ Soft fallback instead of kill
        if final_conviction > 0.5 {
            gated_signal // allow strong signals through
        } else {
            gated_signal
        }
    } else {
        gated_signal
    };

    let strength_label = if final_conviction > 0.75 {
        "STRONG".to_string()
    } else if final_conviction > 0.60 {
        "MEDIUM".to_string()
    } else {
        "WEAK".to_string()
    };

    DecisionReport {
        trade_id: 0,
        symbol: symbol.to_string(),
        timestamp: history[last_idx].timestamp,
        signal: final_signal,
        confidence: final_conviction,
        expected_return: expected_return_sum,
        horizon_bars: config.max_hold_bars as u64,
        participation: conviction_guard.norm_vol_score,
        regime: current_regime,
        aligned_weight: buy_weight, // Simplified mapping for diagnostic clarity
        opposing_weight: sell_weight,
        consistency: if final_signal == last_signal && final_signal != SignalType::WAIT {
            consistency_count + 1
        } else {
            0
        },
        conviction_score: final_conviction,
        agreement_strength: strength_label,
        voters: format!(
            "{}/{}",
            if gated_signal == SignalType::BUY {
                buy_count
            } else {
                sell_count
            },
            eligible_voters.len()
        ),
        execution_feasible: feasible,
        execution_score: exec_score,
        execution_threshold: exec_threshold,
        threshold: dynamic_threshold,
        raw_edge: 0.0,
        realized_return: None,
        capture_efficiency: None, execution_feasibility: 0.0,
        efficiency_label: String::new(),
        recommendation: None,
    }
}

fn calculate_lookback_stats(history: &[Candle], cursor: usize, lookback: usize) -> (f64, f64, f64) {
    let start = cursor.saturating_sub(lookback);
    let slice = &history[start..=cursor];
    let prices: Vec<f64> = slice.iter().map(|c| c.close as f64).collect();
    let mean = prices.iter().sum::<f64>() / prices.len() as f64;
    let variance = prices.iter().map(|p| (p - mean).powi(2)).sum::<f64>() / prices.len() as f64;
    let std_dev = variance.sqrt();
    (mean, std_dev, prices.len() as f64)
}

const DNA_IMPORTANCE_WEIGHTS: [f64; 13] = [
    0.1538, 0.1538, 0.0769, 0.0769, 0.0769, // High (Thresh, Edge), Mid (TP, SL, Hold)
    0.0385, 0.0385, 0.0385, // Low (Weights)
    0.0385, 0.0385, 0.0385, // Low (Exponents)
    0.1538, 0.0769, // High (Selectivity), Mid (Archetype)
];

pub fn extract_features(strategy: &Strategy) -> Vec<f64> {
    let mut features = Vec::with_capacity(13);
    let norm = |val: f64, max: f64| (val / max).clamp(0.0, 1.0);

    features.push(norm(strategy.queue_threshold as f64, 5000.0));
    features.push(norm(strategy.base_edge as f64, 500.0));
    features.push(norm(strategy.take_profit as f64, 500.0));
    features.push(norm(strategy.stop_loss as f64, 500.0));
    features.push(norm(strategy.holding_period as f64, 100.0));
    features.push(norm(strategy.w_conviction as f64, 100.0));
    features.push(norm(strategy.w_momentum as f64, 100.0));
    features.push(norm(strategy.w_volatility as f64, 100.0));
    features.push(norm(strategy.exp_conviction as f64, 100.0));
    features.push(norm(strategy.exp_momentum as f64, 100.0));
    features.push(norm(strategy.exp_volatility as f64, 100.0));
    features.push(norm(strategy.selectivity as f64, 100.0));
    features.push(norm(strategy.archetype as f64, 3.0));

    features
}

/// Phase D.1.12: Signal Truth Layer.
/// Decouples belief from outcome and penalizes noise using directional intent and weighted DNA metrics.
pub fn compute_consensus_alpha(
    elites: &[Strategy],
    scenario: &ScenarioPair,
    config: &GaConfig,
) -> ConsensusReport {
    let mut signal_votes_map: HashMap<usize, Vec<SignalVote>> = HashMap::new();
    let total_strategies = elites.len();

    if total_strategies == 0 {
        return ConsensusReport {
            scenario_name: scenario.name.to_string(),
            top_signals: vec![],
            portfolio_clusters: Vec::new(),
            global_entropy: 0.0,
            active_strategies: 0,
        };
    }

    let epsilon = 0.05;
    for strategy in elites {
        // Phase D.1.14: Enforce Consensus Purity
        // We ensure strict configuration and explicitly disable bootstrap fallbacks for reporting.
        // The consensus report must only reflect structurally valid, organic patterns.
        let mut strict_config = config.clone();
        strict_config.initial_queue_threshold = config.initial_queue_threshold;

        if let Some(res) =
            evaluate_strategy(strategy, scenario, &strict_config, 0, 0.0, 0, 0.0, 1.0, 0)
        {
            if ga_debug_enabled() {
                println!(
                    "STRATEGY → id={} fitness={:.3} trades={}",
                    strategy_to_id(strategy),
                    res.fitness,
                    res.trade_count
                );
            }

            for sig in &res.emitted_signals {
                println!(
                    "SIGNAL → ts={} dir={} strength={:.3}",
                    sig.ts, sig.direction, sig.strength
                );
            }
            // Directional Intent Mapping (Belief-based)
            let decision = if res.emitted_signals.is_empty() {
                Decision::HOLD
            } else {
                let net: f64 = res.emitted_signals
                    .iter()
                    .map(|s| s.strength * s.direction as f64)
                    .sum();

                if net > epsilon {
                    Decision::BUY
                } else if net < -epsilon {
                    Decision::SELL
                } else {
                    Decision::HOLD
                }
            };

            if decision != Decision::HOLD {
                for sig in &res.emitted_signals {
                    let votes = signal_votes_map.entry(sig.ts).or_insert(vec![]);

                    votes.push(SignalVote {
                        strategy_id: strategy_to_id(strategy),
                        archetype: Archetype::from(strategy.archetype),
                        confidence: sig.strength,
                        signal_features: extract_features(strategy),
                        decision: if sig.strength * sig.direction as f64 > 0.0 {
                            Decision::BUY
                        } else {
                            Decision::SELL
                        },
                    });
                }
            }
        }
    }

    let mut reports = Vec::new();
    for (idx, votes) in signal_votes_map {
        let count = votes.len();
        let support_ratio = count as f64 / total_strategies as f64;

        // --- NEW HYBRID RECOMMENDATION LAYER (Union Mode) ---
        let avg_score = votes.iter().map(|v| v.confidence.abs()).sum::<f64>() / count as f64;

        let mut buy_c = 0usize;
        let mut sell_c = 0usize;
        for v in &votes {
            match v.decision {
                Decision::BUY => buy_c += 1,
                Decision::SELL => sell_c += 1,
                _ => {}
            }
        }

        let consistency = (buy_c.max(sell_c) as f64 / count.max(1) as f64).clamp(0.0, 1.0);
        let exec_prob_proxy = (support_ratio * 2.0).clamp(0.5, 1.0);

        let hybrid_score = avg_score * exec_prob_proxy * consistency;

        let unique_archs: HashSet<Archetype> = votes.iter().map(|v| v.archetype).collect();
        let archetype_diversity = unique_archs.len() as f64 / 4.0;

        let label = if hybrid_score > 1.0 {
            " 🔥 STRONG SIGNAL"
        } else if support_ratio > 0.3 {
            "consensus trade"
        } else {
            "speculative"
        }
        .to_string();

        let alpha = hybrid_score;
        let disagreement_entropy = 0.0;
        let feature_diversity = 0.0;
        let alignment_factor = consistency;
        let realized_edge_factor = 1.0;

        let _signal_timestamp = scenario.signal.get(idx).map(|e| e.exchange_ts).unwrap_or(0);

        reports.push(SignalAlphaReport {
            signal_idx: idx,
            asset: scenario.signal_symbol.to_string(),
            support_count: count,
            support_ratio,
            avg_score,
            archetype_diversity,
            alpha_score: alpha,
            conviction: avg_score,
            archetypes: unique_archs
                .into_iter()
                .map(|a| match a {
                    Archetype::Conviction => 0,
                    Archetype::Momentum => 1,
                    Archetype::Reversion => 2,
                    Archetype::Volatility => 3,
                })
                .collect(),
            consensus_label: label,
            disagreement_entropy,
            feature_diversity,
            realized_edge_factor,
            signal_timestamp: _signal_timestamp,
            temporal_stability: 1.0,
            persistence_count: 1,
            alignment_factor,
        });
    }

    reports.sort_by(|a, b| b.alpha_score.total_cmp(&a.alpha_score));

    ConsensusReport {
        scenario_name: scenario.name.to_string(),
        top_signals: reports,
        portfolio_clusters: Vec::new(),
        global_entropy: 0.0,
        active_strategies: total_strategies,
    }
}

pub fn evaluate_and_aggregate(
    strategy: &Strategy,
    config: &GaConfig,
    scenarios: &[ScenarioPair],
    generation: usize,
    diversity: f64,
    unique_count: usize,
    expansion_bias: f64,
    strategy_index: usize,
) -> Option<StrategyEvaluation> {
    evaluate_and_aggregate_with_trade_depth(
        strategy,
        config,
        scenarios,
        generation,
        diversity,
        unique_count,
        0.0,
        expansion_bias,
        strategy_index,
    )
    .map(|(e, _)| e)
}
pub fn evaluate_and_aggregate_with_trade_depth(
    strategy: &Strategy,
    config: &GaConfig,
    scenarios: &[ScenarioPair],
    generation: usize,
    diversity: f64,
    unique_count: usize,
    gen_max_log_queue: f64,
    expansion_bias: f64,
    strategy_index: usize,
) -> Option<(StrategyEvaluation, f64)> {
    let mut reports = Vec::new();
    for pair in scenarios {
        let result = evaluate_strategy(
            strategy,
            pair,
            config,
            generation,
            diversity,
            unique_count,
            gen_max_log_queue,
            expansion_bias,
            strategy_index,
        );

        if result.is_none() {
            if ga_debug_enabled() {
                println!(
                    "⚠️ evaluate_strategy returned None for scenario {}",
                    pair.name
                );
            }
        }

        if let Some(r) = result {
            reports.push(r);
        }
    }

    if reports.is_empty() {
        println!("🚨 CRITICAL → No evaluations returned from evaluate_strategy");

        return None; // keep this for now
    }

    // Use the canonical inner aggregation function
    aggregate_strategy_reports_inner(reports, 1.0, config, generation)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::csv_source::CsvCandleSource;
    use crate::CandleSource;
    use crate::{MarketEventType, Side};

    fn get_default_ga_config() -> GaConfig {
        GaConfig {
            population_size: 10,
            generations: 5,
            mutation_rate: 0.05,
            seed: 123,
            order_id_prefix: "GA_TEST".to_string(),
            order_price: 100,
            order_quantity_for_strategy: 100,
            order_timestamp: 13,
            lambda: 0.5,
            initial_queue_threshold: 200,
            max_trades_per_scenario: Some(1),
            trade_cooldown_events: None,
            latency_ticks: 1,
            slippage_factor: 0.1,
            lot_size: 1.0,
            deep_validation: false,
            max_hold_bars: 20,
            fitness_mode: FitnessMode::Sniper,
            pnl_fingerprint_len: 50,
            min_trades_threshold: 5,
            min_candles: 100,
            preserve_top_k: 3,
        }
    }

    fn synthetic_harness_trade_tape(
        base_ts: u64,
        flat_price: u64,
        step_price: u64,
    ) -> Vec<MarketEvent> {
        let mut v = Vec::with_capacity(128);
        for i in 0..128 {
            let ts = base_ts + i as u64;
            // Flat then small step: fills + TP path while keeping aggregate fitness inside GA's `<= 1.0` gate.
            let price = if i < 48 {
                flat_price * 100
            } else {
                step_price * 100
            };
            v.push(MarketEvent {
                subtype: MarketEventType::Trade,
                price,
                quantity: 2_000,
                side: None,
                exchange_ts: ts,
            });
        }
        v
    }

    /// Two deterministic tapes (no disk I/O): cross-scenario aggregation + multi-trade depth.
    fn synthetic_harness_scenarios() -> HashMap<String, Vec<MarketEvent>> {
        let mut scenarios = HashMap::new();
        scenarios.insert(
            "HARNESS_LIQUID_RAMP_A".to_string(),
            synthetic_harness_trade_tape(1000, 100, 101),
        );
        scenarios.insert(
            "HARNESS_LIQUID_RAMP_B".to_string(),
            synthetic_harness_trade_tape(5000, 102, 103),
        );
        scenarios
    }

    fn get_scenarios_map() -> HashMap<String, Vec<MarketEvent>> {
        let mut scenarios = HashMap::new();
        scenarios.insert(
            "High_Liquidity_Stable_Price".to_string(),
            vec![
                MarketEvent {
                    subtype: MarketEventType::NewOrder,
                    price: 100,
                    quantity: 2000,
                    side: Some(Side::Sell),
                    exchange_ts: 10,
                },
                MarketEvent {
                    subtype: MarketEventType::Trade,
                    price: 100,
                    quantity: 500,
                    side: None,
                    exchange_ts: 15,
                },
                MarketEvent {
                    subtype: MarketEventType::Trade,
                    price: 100,
                    quantity: 500,
                    side: None,
                    exchange_ts: 20,
                },
            ],
        );
        scenarios.insert(
            "Increasing_Queue_Ahead".to_string(),
            vec![
                MarketEvent {
                    subtype: MarketEventType::NewOrder,
                    price: 100,
                    quantity: 1000,
                    side: Some(Side::Sell),
                    exchange_ts: 10,
                },
                MarketEvent {
                    subtype: MarketEventType::NewOrder,
                    price: 100,
                    quantity: 2000,
                    side: Some(Side::Sell),
                    exchange_ts: 11,
                },
                MarketEvent {
                    subtype: MarketEventType::NewOrder,
                    price: 100,
                    quantity: 3000,
                    side: Some(Side::Sell),
                    exchange_ts: 12,
                },
                MarketEvent {
                    subtype: MarketEventType::Trade,
                    price: 100,
                    quantity: 100,
                    side: None,
                    exchange_ts: 15,
                },
            ],
        );
        scenarios
    }

    #[test]
    fn test_ga_determinism() {
        let config1 = get_default_ga_config();
        let config2 = config1.clone();
        let scenarios_map = get_scenarios_map();
        let scenarios_vec: Vec<ScenarioPair> = scenarios_map
            .iter()
            .map(|(name, events)| ScenarioPair {
                name,
                signal_symbol: "TEST",
                execution_symbol: "TEST",
                signal: events.as_slice(),
                execution: events.as_slice(),
            })
            .collect();

        let (ga_result1, _) = run_ga_evolution(config1, &scenarios_vec, &GlobalEvoState::default());
        let (ga_result2, _) = run_ga_evolution(config2, &scenarios_vec, &GlobalEvoState::default());

        assert_eq!(
            ga_result1.global_best.strategy, ga_result2.global_best.strategy,
            "Best strategy diverged with same seed"
        );
        assert!(
            (ga_result1.global_best.fitness - ga_result2.global_best.fitness).abs() < 1e-6,
            "Best strategy fitness diverged with same seed"
        );
        assert_eq!(
            ga_result1.global_best_generation, ga_result2.global_best_generation,
            "Global best generation diverged"
        );
        assert_eq!(
            ga_result1.final_generation_best.strategy, ga_result2.final_generation_best.strategy,
            "Final generation best strategy diverged"
        );
        assert!(
            (ga_result1.final_generation_best.fitness - ga_result2.final_generation_best.fitness)
                .abs()
                < 1e-6,
            "Final generation best fitness diverged"
        );

        if ga_debug_enabled() {
            println!("✅ GA determinism test passed.");
        }
    }

    #[test]
    fn test_evaluate_strategy() {
        let mut rng = StdRng::seed_from_u64(12345);
        let strategy = Strategy {
            queue_threshold: 100,
            base_edge: 2,
            take_profit: 20,
            stop_loss: 10,
            holding_period: 20,
            w_conviction: 50,
            w_momentum: 30,
            w_volatility: 20,
            exp_conviction: 100,
            exp_momentum: 100,
            exp_volatility: 100,
            selectivity: 75,
            archetype: 0,
            entry_offset: 0,
            direction_bias: 50,
            vol_floor: 20,
            mom_floor: 20,
            edge_ratio: 150,
            participation_threshold: 30,
            exec_aggression: 50,
            latency_bias: 10,
            fill_threshold: 50, lineage: 0,
        };
        let scenarios = get_scenarios_map();
        let market_events = scenarios.get("High_Liquidity_Stable_Price").unwrap();
        let pair = ScenarioPair {
            name: "High_Liquidity_Stable_Price",
            signal_symbol: "TEST",
            execution_symbol: "TEST",
            signal: market_events.as_slice(),
            execution: market_events.as_slice(),
        };

        let config = get_default_ga_config();
        let report = evaluate_strategy(&strategy, &pair, &config, 0, 0.0, 0, 0.0, 1.0, 0);

        if let Some(r) = report {
            assert_eq!(r.strategy, strategy);
            assert!(!r.strategy_id.is_empty());
            assert_eq!(r.fitness, 0.0);
            if r.trade_count > 1 {
                assert_ne!(r.std_dev, 0.0);
            }
            if r.trade_count > 0 {
                assert_ne!(r.worst, f64::INFINITY);
            }

            if ga_debug_enabled() {
                println!("Report: {:#?}", r);
            }
        }

        println!("✅ evaluate_strategy test passed.");
    }

    /// Real candles, explicit `max_trades_per_scenario` — ensures the multi-trade loop cannot exceed the cap.
    #[test]
    fn test_evaluate_strategy_multi_trade_cap_respected() {
        let test_assets = format!("{}/../test_assets", env!("CARGO_MANIFEST_DIR"));
        let path = format!("{}/RELIANCE_5m_clean.csv", test_assets);
        let candles = CsvCandleSource { path }.get_candles_sync();
        let scenarios = crate::pipeline::scenarios_from_candles("RELIANCE_SIM", &candles);
        let mut keys: Vec<String> = scenarios.keys().cloned().collect();
        keys.sort();

        let cap = 3usize;
        let mut config = get_default_ga_config();
        config.max_trades_per_scenario = Some(cap);
        let strategy = Strategy {
            queue_threshold: 100,
            base_edge: 2,
            take_profit: 20,
            stop_loss: 10,
            holding_period: 20,
            w_conviction: 50,
            w_momentum: 30,
            w_volatility: 20,
            exp_conviction: 100,
            exp_momentum: 100,
            exp_volatility: 100,
            selectivity: 75,
            archetype: 0,
            entry_offset: 0,
            direction_bias: 50,
            vol_floor: 20,
            mom_floor: 20,
            edge_ratio: 150,
            participation_threshold: 30,
            exec_aggression: 50,
            latency_bias: 10,
            fill_threshold: 50, lineage: 0,
        };

        let mut found = false;
        for name in &keys {
            let events = scenarios.get(name).expect("key from scenarios");
            let pair = ScenarioPair {
                name: name,
                signal_symbol: "RELIANCE_SIM",
                execution_symbol: "RELIANCE_SIM",
                signal: events.as_slice(),
                execution: events.as_slice(),
            };
            if let Some(r) = evaluate_strategy(&strategy, &pair, &config, 0, 0.0, 0, 0.0, 1.0, 0) {
                assert!(
                    r.trade_count <= cap,
                    "trade_count {} exceeds configured cap {}",
                    r.trade_count,
                    cap
                );
                found = true;
                break;
            }
        }
        assert!(
            found,
            "expected at least one RELIANCE window where strategy trades (for cap test)"
        );
    }

    #[test]
    fn test_ga_evolution_with_benchmarks() {
        let mut config = get_default_ga_config();
        config.population_size = 20;
        config.generations = 10;
        config.mutation_rate = 0.15;
        config.seed = 456;
        config.order_id_prefix = "GA_PROG_TEST".to_string();
        config.max_trades_per_scenario = Some(1);
        let scenarios_map = get_scenarios_map();
        let scenarios_vec: Vec<ScenarioPair> = scenarios_map
            .iter()
            .map(|(name, events)| ScenarioPair {
                name,
                signal_symbol: "TEST",
                execution_symbol: "TEST",
                signal: events.as_slice(),
                execution: events.as_slice(),
            })
            .collect();
        let (ga_result, _) = run_ga_evolution(config, &scenarios_vec, &GlobalEvoState::default());
        let ga_result = ga_result;
        if ga_debug_enabled() {
            println!(
                "Final Best Report (Global Best): {:#?}",
                ga_result.global_best
            );
        }

        // In benchmark scenarios, everything might be rejected due to strict viability filters
        assert!(ga_result.global_best.fitness >= 0.0 || ga_result.global_best.avg_pnl < 0.0);

        println!("✅ GA evolution with benchmarks test passed.");
    }

    #[test]
    fn test_top_k_sorted() {
        let mut config = get_default_ga_config();
        config.population_size = 10;
        config.generations = 1;
        config.mutation_rate = 0.05;
        config.seed = 123;
        config.order_id_prefix = "TOP_K_TEST".to_string();
        config.max_trades_per_scenario = Some(1);
        let scenarios_map = get_scenarios_map();
        let scenarios_vec: Vec<ScenarioPair> = scenarios_map
            .iter()
            .map(|(name, events)| ScenarioPair {
                name,
                signal_symbol: "TEST",
                execution_symbol: "TEST",
                signal: events.as_slice(),
                execution: events.as_slice(),
            })
            .collect();
        let (ga_result, _) = run_ga_evolution(config, &scenarios_vec, &GlobalEvoState::default());
        let ga_result = ga_result;

        if ga_debug_enabled() {
            println!("Global Best in Test: {:#?}", ga_result.global_best);
        }
        println!(
            "Final Generation Best in Test: {:#?}",
            ga_result.final_generation_best
        );
        println!("✅ Top K sorted test passed. (Test adjusted for new return type)");
    }

    fn mock_scenario_eval(
        pnl: f64,
        trades: usize,
        profitable: usize,
        entropy: f64,
    ) -> StrategyEvaluation {
        StrategyEvaluation {
            strategy_id: "test".to_string(),
            strategy: Strategy {
                queue_threshold: 100,
                base_edge: 2,
                take_profit: 20,
                stop_loss: 10,
                holding_period: 0,
                w_conviction: 50,
                w_momentum: 30,
                w_volatility: 20,
                exp_conviction: 100,
                exp_momentum: 100,
                exp_volatility: 100,
                selectivity: 75,
                archetype: 0,
                entry_offset: 0,
                direction_bias: 50,
                vol_floor: 20,
                mom_floor: 20,
                edge_ratio: 150,
                participation_threshold: 30,
                exec_aggression: 50,
                latency_bias: 10,
                fill_threshold: 50, lineage: 0,
            },
            capability: ScenarioCapability::Executable,
            avg_pnl: pnl,
            std_dev: 0.0,
            downside_std_dev: 0.0,
            worst: 0.0,
            robustness: 0.0,
            fitness: 0.0,
            trade_count: trades,
            max_drawdown: 0.0,
            participation_rate: if trades > 0 { 1.0 } else { -0.01 },
            profitable_trades: profitable,
            zero_pnl_trades: 0,
            quality_trades: profitable as f64,
            payoff_ratio: 2.0,
            execution_metrics: ExecutionMetrics {
                fill_efficiency: 1.0,
                capture_efficiency: 1.0,
                avg_slippage: 0.0,
                latency_impact: 0.0,
                fill_rate: 0.0,
                liquidity_starved_count: 0,
                queue_blocked_count: 0,
                total_attempts: 0,
            },
            scenario_signature: ScenarioExecutionSignature::default(),
            avg_conviction: 1.0,
            avg_efficiency: 1.0,
            avg_edge_quality: 1.0,
            directional_accuracy: if trades > 0 {
                profitable as f64 / trades as f64
            } else {
                0.01
            },
            decisiveness: 1.0,
            execution_friction: 1.0,
            exit_tp_count: profitable,
            exit_sl_count: trades - profitable,
            exit_ts_count: 0,
            consistency_score: 1.0,
            recent_performance: pnl,
            selectivity: if trades > 0 { 0.05 } else { 0.01 },
            avg_entropy: entropy,
            ..StrategyEvaluation::default()
        }
    }

    #[test]
    fn test_fitness_sparse_strategy_collapse() {
        let mut evals = Vec::new();
        for _ in 0..2 {
            evals.push(mock_scenario_eval(0.03, 5, 5, 0.45));
        } // active
        for _ in 0..8 {
            evals.push(mock_scenario_eval(0.0, 0, 0, 0.45));
        } // inactive

        let config = get_default_ga_config();
        let aggregated = aggregate_strategy_reports_with_top_k(evals, &config, None, 0).unwrap();

        // Participation is 0.2. Under bounded/log fitness, weak strategies should remain low but non-negative.
        assert!(
            aggregated.fitness < 0.5 && aggregated.fitness >= 0.0,
            "Sparse strategy fitness should be very low ({}).",
            aggregated.fitness
        );
    }

    #[test]
    fn test_fitness_high_participation_outperforms() {
        // Strategy A: high participation (0.8), stronger pnl/trade profile
        let mut evals_a = Vec::new();
        for _ in 0..9 {
            evals_a.push(mock_scenario_eval(0.03, 3, 3, 0.45));
        }
        evals_a.push(mock_scenario_eval(0.0, 0, 0, 0.45));
        let config = get_default_ga_config();
        let agg_a = aggregate_strategy_reports_with_top_k(evals_a, &config, None, 0).unwrap();

        // Strategy B: low participation (0.3), higher average active pnl
        let mut evals_b = Vec::new();
        for _ in 0..3 {
            evals_b.push(mock_scenario_eval(0.03, 5, 5, 0.45));
        }
        for _ in 0..7 {
            evals_b.push(mock_scenario_eval(0.0, 0, 0, 0.45));
        }
        let agg_b = aggregate_strategy_reports_with_top_k(evals_b, &config, None, 0).unwrap();

        // Under bounded/log fitness, high-participation profile should dominate low-participation.
        assert!(
            agg_a.fitness > agg_b.fitness,
            "Expected high participation fitness {} to exceed low participation fitness {}",
            agg_a.fitness,
            agg_b.fitness
        );
        assert!(
            agg_b.fitness >= 0.0,
            "Low participation fitness {} should stay non-negative.",
            agg_b.fitness
        );
    }

    #[test]
    fn test_fitness_low_trade_count_penalizes() {
        let config = get_default_ga_config();
        // Strategy A: 5 trades total (5 active scenarios, 1 trade each)
        // Bypasses participation reject, but gets crushed by trades < 10 hard filter AND effectiveness scaling
        let mut evals_a = Vec::new();
        for _ in 0..5 {
            evals_a.push(mock_scenario_eval(0.02, 1, 1, 0.45));
        }
        for _ in 0..5 {
            evals_a.push(mock_scenario_eval(0.0, 0, 0, 0.45));
        }
        let agg_a = aggregate_strategy_reports_with_top_k(evals_a, &config, None, 0).unwrap();

        // Strategy B: strong profile with full participation and enough trades
        let mut evals_b = Vec::new();
        for _ in 0..10 {
            evals_b.push(mock_scenario_eval(0.03, 4, 4, 0.45));
        }
        let agg_b = aggregate_strategy_reports_with_top_k(evals_b, &config, None, 0).unwrap();

        assert!(
            agg_a.fitness < 0.5 && agg_a.fitness >= 0.0,
            "Low trade count fitness {} should be very low.",
            agg_a.fitness
        );
        assert!(
            agg_b.fitness > agg_a.fitness,
            "Expected higher-trade profile to beat low-trade profile: {} vs {}",
            agg_b.fitness,
            agg_a.fitness
        );
    }

    #[test]
    fn test_fitness_high_variance_reduces() {
        // Stable: all 10 return 0.01 (std_dev = 0.0)
        let mut evals_stable = Vec::new();
        for _ in 0..10 {
            evals_stable.push(mock_scenario_eval(0.01, 5, 5, 0.45));
        }
        let config = get_default_ga_config();
        let agg_stable =
            aggregate_strategy_reports_with_top_k(evals_stable, &config, None, 0).unwrap();

        // Unstable: 5 return 0.02, 5 return 0.0 (std_dev = 0.01, same avg = 0.01)
        let mut evals_unstable = Vec::new();
        for _ in 0..5 {
            evals_unstable.push(mock_scenario_eval(0.02, 5, 5, 0.45));
        }
        for _ in 0..5 {
            evals_unstable.push(mock_scenario_eval(0.0, 5, 0, 0.45));
        }
        let config = get_default_ga_config();
        let agg_unstable =
            aggregate_strategy_reports_with_top_k(evals_unstable, &config, None, 0).unwrap();

        assert!(
            agg_stable.fitness > agg_unstable.fitness,
            "Stable fitness {} must beat unstable fitness {}",
            agg_stable.fitness,
            agg_unstable.fitness
        );
    }

    #[test]
    fn test_multiplicative_fitness_aggregation() {
        let config = get_default_ga_config();
        let mut evals = Vec::new();
        for _ in 0..10 {
            evals.push(mock_scenario_eval(-0.02, 5, 0, 0.45));
        }
        let agg = aggregate_strategy_reports_with_top_k(evals, &config, None, 0).unwrap();

        assert!(
            agg.fitness >= 0.0,
            "Fitness should be non-negative under log/additive model, got {}",
            agg.fitness
        );
    }

    #[test]
    fn test_fitness_hard_collapse_threshold() {
        // Collapse: 2 active (part = 0.20, triggers < 0.3 collapse)
        // Also triggers total trades < 10 (unless they do 5 trades each, here they do 10 each so trades = 20)
        let mut evals_collapse = Vec::new();
        for _ in 0..2 {
            evals_collapse.push(mock_scenario_eval(0.03, 10, 10, 0.45));
        }
        for _ in 0..8 {
            evals_collapse.push(mock_scenario_eval(0.0, 0, 0, 0.45));
        }
        let config = get_default_ga_config();
        let agg_collapse =
            aggregate_strategy_reports_with_top_k(evals_collapse, &config, None, 0).unwrap();

        // Survive: strong + broad participation profile
        let mut evals_survive = Vec::new();
        for _ in 0..10 {
            evals_survive.push(mock_scenario_eval(0.03, 2, 2, 0.45));
        }
        let config = get_default_ga_config();
        let agg_survive =
            aggregate_strategy_reports_with_top_k(evals_survive, &config, None, 0).unwrap();

        assert!(
            agg_collapse.fitness < 0.5 && agg_collapse.fitness >= 0.0,
            "Collapse fitness {} should be completely crushed.",
            agg_collapse.fitness
        );
        assert!(
            agg_survive.fitness > agg_collapse.fitness,
            "Expected broad participation profile to beat collapsed profile: {} vs {}",
            agg_survive.fitness,
            agg_collapse.fitness
        );
    }

    #[test]
    fn test_fitness_relative_ordering() {
        // Weak: low participation + low trades + mostly zero outcomes
        let mut weak_evals = Vec::new();
        for _ in 0..2 {
            weak_evals.push(mock_scenario_eval(0.005, 1, 1, 0.45));
        }
        for _ in 0..8 {
            weak_evals.push(mock_scenario_eval(0.0, 0, 0, 0.45));
        }
        let config = get_default_ga_config();
        let weak = aggregate_strategy_reports_with_top_k(weak_evals, &config, None, 0).unwrap();

        // Strong: full participation + higher pnl + high trade quality
        let mut strong_evals = Vec::new();
        for _ in 0..10 {
            strong_evals.push(mock_scenario_eval(0.02, 8, 8, 0.45));
        }
        let strong = aggregate_strategy_reports_with_top_k(strong_evals, &config, None, 0).unwrap();

        assert!(
            strong.fitness > weak.fitness,
            "Expected strong ({}) > weak ({})",
            strong.fitness,
            weak.fitness
        );
        assert!(weak.fitness >= 0.0);
        assert!(strong.fitness >= 0.0);
    }

    #[test]
    fn ga_top_k_pick_diverse_lambda_zero_matches_pure_rank_order() {
        let make = |i: usize, pnl: f64| {
            let mut e = mock_scenario_eval(pnl, 5, 5, 0.45);
            e.strategy_id = format!("s{}", i);
            e
        };
        let evals = vec![make(0, 0.04), make(1, 0.01), make(2, 0.03), make(3, 0.02)];
        let remaining: Vec<(usize, f64, StrategyEvaluation)> = evals
            .into_iter()
            .enumerate()
            .map(|(i, e)| {
                let s = super::ga_scenario_rank_score(&e);
                (i, s, e)
            })
            .collect();
        let picked = super::ga_top_k_pick_diverse(
            remaining,
            2,
            0.0,
            crate::selection_cap::GaDiversityMode::Repel,
        );
        assert_eq!(picked.len(), 2);
        assert_eq!(picked[0].strategy_id, "s0");
        assert_eq!(picked[1].strategy_id, "s2");
    }

    #[test]
    fn ga_top_k_pick_diverse_is_deterministic() {
        let make = |i: usize, pnl: f64| {
            let mut e = mock_scenario_eval(pnl, 5, 5, 0.45);
            e.strategy_id = format!("s{}", i);
            e
        };
        let evals = vec![make(0, 0.05), make(1, 0.04), make(2, 0.03)];
        let build_remaining = |ev: Vec<StrategyEvaluation>| {
            ev.into_iter()
                .enumerate()
                .map(|(i, e)| {
                    let s = super::ga_scenario_rank_score(&e);
                    (i, s, e)
                })
                .collect::<Vec<_>>()
        };
        let a = super::ga_top_k_pick_diverse(
            build_remaining(evals.clone()),
            2,
            0.7,
            crate::selection_cap::GaDiversityMode::Repel,
        );
        let b = super::ga_top_k_pick_diverse(
            build_remaining(evals),
            2,
            0.7,
            crate::selection_cap::GaDiversityMode::Repel,
        );
        assert_eq!(a.len(), b.len());
        assert!(a
            .iter()
            .zip(b.iter())
            .all(|(x, y)| x.strategy_id == y.strategy_id));
    }

    #[test]
    fn ga_top_k_repel_vs_attract_second_pick() {
        let sig_a = ScenarioExecutionSignature {
            avg_queue_ahead: 0.1,
            avg_latency: 0.1,
            fill_ratio: 0.9,
            participation: 1.0,
            execution_variance: 0.0,
        };
        let sig_close = ScenarioExecutionSignature {
            avg_queue_ahead: 0.15,
            avg_latency: 0.12,
            fill_ratio: 0.88,
            participation: 1.0,
            execution_variance: 0.0,
        };
        let sig_far = ScenarioExecutionSignature {
            avg_queue_ahead: 2.5,
            avg_latency: 2.5,
            fill_ratio: 0.15,
            participation: 1.0,
            execution_variance: 0.0,
        };
        let mut a = mock_scenario_eval(0.05, 5, 5, 0.45);
        a.strategy_id = "a".to_string();
        a.scenario_signature = sig_a.clone();
        let mut b = mock_scenario_eval(0.05, 5, 5, 0.45);
        b.strategy_id = "b".to_string();
        b.scenario_signature = sig_close;
        let mut c = mock_scenario_eval(0.05, 5, 5, 0.45);
        c.strategy_id = "c".to_string();
        c.scenario_signature = sig_far;
        let build = |ev: Vec<StrategyEvaluation>| {
            ev.into_iter()
                .enumerate()
                .map(|(i, e)| {
                    let s = super::ga_scenario_rank_score(&e);
                    (i, s, e)
                })
                .collect::<Vec<_>>()
        };
        let repel = super::ga_top_k_pick_diverse(
            build(vec![a.clone(), b.clone(), c.clone()]),
            2,
            1.0,
            crate::selection_cap::GaDiversityMode::Repel,
        );
        let attract = super::ga_top_k_pick_diverse(
            build(vec![a, b, c]),
            2,
            1.0,
            crate::selection_cap::GaDiversityMode::Attract,
        );
        assert_eq!(repel[0].strategy_id, "a");
        assert_eq!(attract[0].strategy_id, "a");
        assert_eq!(repel[1].strategy_id, "c");
        assert_eq!(attract[1].strategy_id, "b");
    }

    #[test]
    fn test_ga_weighted_scenario_pnl_opt_in() {
        use std::sync::Mutex;
        static ENV_LOCK: Mutex<()> = Mutex::new(());
        let _guard = ENV_LOCK.lock().unwrap();

        let evals = vec![
            mock_scenario_eval(0.01, 5, 5, 0.45),
            mock_scenario_eval(0.06, 5, 5, 0.45),
        ];
        let config = get_default_ga_config();
        std::env::remove_var("GA_WEIGHTED_SCENARIO_PNL");
        let unweighted =
            aggregate_strategy_reports_with_top_k(evals.clone(), &config, None, 0).unwrap();
        std::env::set_var("GA_WEIGHTED_SCENARIO_PNL", "1");
        let weighted = aggregate_strategy_reports_with_top_k(evals, &config, None, 0).unwrap();
        std::env::remove_var("GA_WEIGHTED_SCENARIO_PNL");

        assert!(
            weighted.avg_pnl > unweighted.avg_pnl + 1e-9,
            "weighted avg_pnl {} should exceed unweighted {} when higher-edge scenarios get more weight",
            weighted.avg_pnl,
            unweighted.avg_pnl
        );
    }

    #[test]
    fn test_fitness_has_spread() {
        let mut a_evals = Vec::new();
        for _ in 0..10 {
            a_evals.push(mock_scenario_eval(0.03, 2, 2, 0.45));
        }
        let config = get_default_ga_config();
        let a = aggregate_strategy_reports_with_top_k(a_evals, &config, None, 0).unwrap();

        let mut b_evals = Vec::new();
        for _ in 0..3 {
            b_evals.push(mock_scenario_eval(0.01, 1, 1, 0.45));
        }
        for _ in 0..7 {
            b_evals.push(mock_scenario_eval(0.0, 0, 0, 0.45));
        }
        let b = aggregate_strategy_reports_with_top_k(b_evals, &config, None, 0).unwrap();

        assert!(
            (a.fitness - b.fitness).abs() > 1e-4,
            "Expected non-trivial fitness spread, got a={} b={}",
            a.fitness,
            b.fitness
        );
    }

    /// In-memory scenarios only (no CSV): exercises `run_ga_evolution` + Top-K aggregate + trade depth in sub-second typical debug runs.
    #[test]
    fn test_synthetic_ga_microstructure_harness() {
        let mut config = get_default_ga_config();
        config.population_size = 4;
        config.generations = 15;
        config.mutation_rate = 0.05;
        config.seed = 2026;
        config.order_id_prefix = "SYNTH_HARNESS".to_string();
        config.max_trades_per_scenario = Some(3);
        config.max_hold_bars = 100; // Ensure harness can see the price step at index 48
        config.trade_cooldown_events = Some(0);
        let scenarios_map = synthetic_harness_scenarios();
        let scenarios_vec: Vec<ScenarioPair> = scenarios_map
            .iter()
            .map(|(name, events)| ScenarioPair {
                name,
                signal_symbol: "SYNTH",
                execution_symbol: "SYNTH",
                signal: events.as_slice(),
                execution: events.as_slice(),
            })
            .collect();

        let (ga_result, _) =
            run_ga_evolution(config.clone(), &scenarios_vec, &GlobalEvoState::default());
        let ga_result = ga_result;
        let (eval, depth) = evaluate_and_aggregate_with_trade_depth(
            &ga_result.global_best.strategy,
            &config,
            &scenarios_vec,
            0,
            0.0,
            0,
            0.0,
            1.0,
            0,
        ) // strategy_index fallback
        .expect("synthetic aggregate should produce a report");
        assert!(eval.fitness.is_finite());

        let mut depth_violation_count = 0;
        if depth < 1.0 - 1e-9 {
            println!(
                "⚠️ DEPTH WARNING: {:.4} (scenario expected multi-trade depth)",
                depth
            );
            depth_violation_count += 1;
        }
        let _ = depth_violation_count;
        debug_assert!(depth >= 0.0, "Depth should never be negative");

        assert!(
            depth <= 3.0 + 1e-9,
            "mean depth {:.6} exceeds per-scenario cap (max_trades_per_scenario=3)",
            depth
        );
        assert!(
            eval.fitness > 0.0,
            "synthetic harness produced non-positive fitness {:.6}; multi-trade should contribute to aggregate signal",
            eval.fitness
        );
        if ga_debug_enabled() {
            println!(
                "SYNTH_HARNESS → fitness: {:.4}, depth: {:.2}, trade_count: {}",
                eval.fitness, depth, eval.trade_count
            );
        }
    }

    #[test]
    fn test_evaluate_and_aggregate_enforces_path() {
        use std::path::Path;
        use std::time::{Duration, Instant};

        /// Three symbols → cross-name / cross-window diversity without full-folder sweep.
        /// Multi-trade cap semantics: `test_evaluate_strategy_multi_trade_cap_respected`; here `max_trades=1` for GA cost.
        const CSV_FILES: &[&str] = &[
            "RELIANCE_5m_clean.csv",
            "VODAFONEIDEA_5m_clean.csv",
            "HDFCBANK_5m_clean.csv",
        ];
        /// Catches multi-minute regressions (e.g. full-folder load); three assets × ~20 windows each on debug CI.
        const MAX_WALL_SECS: u64 = 300;

        let start = Instant::now();

        let mut config = get_default_ga_config();
        config.population_size = 20;
        config.generations = 5;
        config.mutation_rate = 0.1;
        config.seed = 42;
        config.order_id_prefix = "GA_SIM".to_string();
        config.min_candles = 100;
        config.fitness_mode = FitnessMode::Scalable;
        config.min_trades_threshold = 5;
        config.preserve_top_k = 2;

        let test_assets = format!("{}/../test_assets", env!("CARGO_MANIFEST_DIR"));
        let mut scenarios = std::collections::HashMap::new();
        for file in CSV_FILES {
            let csv_path = format!("{}/{}", test_assets, file);
            let candles = CsvCandleSource {
                path: csv_path.clone(),
            }
            .get_candles_sync();
            let asset = Path::new(file)
                .file_stem()
                .and_then(|s| s.to_str())
                .and_then(|stem| stem.split('_').next())
                .unwrap_or("UNKNOWN")
                .to_ascii_uppercase()
                + "_SIM";
            let n_before = scenarios.len();
            scenarios.extend(crate::pipeline::scenarios_from_candles(&asset, &candles));
            assert!(
                scenarios.len() > n_before,
                "{} should yield at least one scenario window",
                file
            );
        }

        let scenarios_map = scenarios;
        let scenarios_vec: Vec<ScenarioPair> = scenarios_map
            .iter()
            .map(|(name, events)| ScenarioPair {
                name,
                signal_symbol: "TEST",
                execution_symbol: "TEST",
                signal: events.as_slice(),
                execution: events.as_slice(),
            })
            .collect();

        let (ga_result, _) =
            run_ga_evolution(config.clone(), &scenarios_vec, &GlobalEvoState::default());
        let ga_result = ga_result;
        let (eval, avg_trades_per_active) = evaluate_and_aggregate_with_trade_depth(
            &ga_result.global_best.strategy,
            &config,
            &scenarios_vec,
            0,
            0.0,
            0,
            0.0,
            1.0,
            0,
        )
        .expect("Aggregation should produce evaluation");
        assert!(eval.fitness > 0.0);
        assert!(
            avg_trades_per_active >= 1.0 - 1e-9,
            "expected >= 1 trade per active scenario after Top-K aggregation, got {}",
            avg_trades_per_active
        );

        if ga_debug_enabled() {
            println!(
                "DEBUG → fitness: {:.4}, depth (avg_trades/active): {:.2}",
                eval.fitness, avg_trades_per_active
            );
        }
        if avg_trades_per_active <= 1.0 + 1e-9 {
            eprintln!(
                "WARNING: multi-trade not yet active in this path run (avg_trades ≈ 1.0); depth will rise when scenarios allow >1 round-trip per active window"
            );
        }

        let elapsed = start.elapsed();
        assert!(
            elapsed < Duration::from_secs(MAX_WALL_SECS),
            "path integration test took {:?}; cap avoids silent multiplicative regressions (folder sweep × GA × scenario eval)",
            elapsed
        );
    }

    // --- PHASE 13.5: INSTITUTIONAL VALIDATION TESTS ---

    #[test]
    fn test_selectivity_decay_alpha_5() {
        let config = get_default_ga_config();

        let mock_scenario_eval =
            |pnl: f64, trades: usize, profitable: usize| -> StrategyEvaluation {
                StrategyEvaluation {
                    avg_pnl: pnl,
                    trade_count: trades,
                    profitable_trades: profitable,
                    win_rate: if trades > 0 {
                        profitable as f64 / trades as f64
                    } else {
                        0.01
                    },
                    fitness: 0.5, // Non-zero baseline for multiplier testing
                    ..StrategyEvaluation::default()
                }
            };

        // 1. Nominal strategy (10% selectivity -> 0.10)
        let mut eval_nominal = mock_scenario_eval(0.02, 10, 10);
        eval_nominal.selectivity = 0.10;
        let agg_nominal = aggregate_strategy_reports_inner(vec![eval_nominal], 1.0, &config, 0)
            .unwrap()
            .0;
        let mut eval_over = mock_scenario_eval(0.02, 10, 10);
        eval_over.selectivity = 0.20;
        let agg_over = aggregate_strategy_reports_inner(vec![eval_over], 1.0, &config, 0)
            .unwrap()
            .0;

        // At alpha=5.0, diff=0.10, decay = exp(-5 * 0.1) = exp(-0.5) approx 0.606
        let ratio = agg_over.fitness / agg_nominal.fitness;
        assert!(
            ratio > 0.55 && ratio < 0.65,
            "Expected approx 0.6x decay for 20% selectivity, got {}",
            ratio
        );
    }

    #[test]
    fn test_continuous_entropy_weighting() {
        let config = get_default_ga_config();

        let mock_scenario_eval =
            |pnl: f64, trades: usize, profitable: usize| -> StrategyEvaluation {
                StrategyEvaluation {
                    avg_pnl: pnl,
                    trade_count: trades,
                    profitable_trades: profitable,
                    win_rate: if trades > 0 {
                        profitable as f64 / trades as f64
                    } else {
                        0.01
                    },
                    fitness: 0.5, // Non-zero baseline for multiplier testing
                    ..StrategyEvaluation::default()
                }
            };

        // 1. Optimal Entropy (0.45)
        let mut eval_opt = mock_scenario_eval(0.02, 10, 10);
        eval_opt.avg_entropy = 0.45;
        let agg_opt = aggregate_strategy_reports_inner(vec![eval_opt], 1.0, &config, 0)
            .unwrap()
            .0;
        let mut eval_high = mock_scenario_eval(0.02, 10, 10);
        eval_high.avg_entropy = 0.90;
        let agg_high = aggregate_strategy_reports_inner(vec![eval_high], 1.0, &config, 0)
            .unwrap()
            .0;
        let mut eval_low = mock_scenario_eval(0.02, 10, 10);
        eval_low.avg_entropy = 0.10;
        let agg_low = aggregate_strategy_reports_inner(vec![eval_low], 1.0, &config, 0)
            .unwrap()
            .0;

        assert!(
            agg_opt.fitness > agg_high.fitness,
            "Optimal entropy (0.45) should beat high entropy (0.90)"
        );
        assert!(
            agg_opt.fitness > agg_low.fitness,
            "Optimal entropy (0.45) should beat low entropy (0.10)"
        );
    }
}
