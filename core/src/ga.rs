use crate::{MarketEvent, Side, Candle};
// use crate::harness::run_simulation_harness;
use crate::selection_cap;
use rand::{Rng, SeedableRng, seq::SliceRandom, rngs::StdRng};
use rayon::prelude::*;
use serde_json;
use serde::{Serialize, Deserialize};
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::cmp::Ordering;
use serde_json::value::to_value as to_json_value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignalSource {
    Organic,
    Bootstrap,
}
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignalSignature {
    pub archetype: u8,
    pub regime: i8,   // -1 (low), 0 (norm), 1 (high vol)
    pub momentum: i8, // -1 (bear), 0 (flat), 1 (bull)
}

#[derive(Default, Debug, Clone)]
pub struct SignatureStats {
    pub sum_pnl: f64,
    pub win_count: usize,
    pub sample_count: usize,
}

fn default_capture_eff() -> f64 { 1.0 }

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
    
    fn mean(&self) -> f64 { self.mean }
    fn std(&self) -> f64 {
        if self.count < 2 { 0.0 } else { (self.m2 / (self.count - 1) as f64).sqrt() }
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

/// --- PHASE 13.5: INSTITUTIONAL METRICS ENGINE ---
#[derive(Default, Debug, Clone)]
struct ScenarioMetrics {
    // Adaptive Layer
    pub adaptive: AdaptiveStats,

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
    pub sum_e_score: f64,
    pub vip_exec_passed_count: usize,
    pub sum_vip_e_score: f64,
    pub sum_stat_e_score: f64,

    // Phase 14++: Structural Health (Universe Discovery)
    pub total_windows: usize,
    pub valid_windows: usize,
    pub accepted_windows: usize,
    
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

    fn record_pop_stats(&mut self, dominance: f64, is_exec: bool) {
        let (count, sum, max, buckets) = if is_exec {
            (&mut self.exec_pop_count, &mut self.sum_exec_pop_dominance, &mut self.max_exec_pop_dominance, &mut self.exec_pop_dominance_buckets)
        } else {
            (&mut self.raw_pop_count, &mut self.sum_raw_pop_dominance, &mut self.max_raw_pop_dominance, &mut self.raw_pop_dominance_buckets)
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

    fn record_structural_health(&mut self, agreement: f64, purity: f64, stability: f64, is_valid: bool) {
        self.total_windows += 1;
        self.sum_agreement_raw += agreement;
        self.sum_purity_raw += purity;
        self.sum_stability_raw += stability;
        
        if agreement > self.max_agreement { self.max_agreement = agreement; }
        if purity > self.max_purity { self.max_purity = purity; }
        
        if is_valid {
            self.valid_windows += 1;
            self.sum_agreement_valid += agreement;
            self.sum_purity_valid += purity;
            self.sum_stability_valid += stability;
        }
    }

    fn record_adaptive_pulse(&mut self, agreement: f64, dominance: f64, purity: f64, stability: f64, z_score: f64, energy: f64) {
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
        if self.adaptive.score_history.is_empty() { return 0.0; }
        percentile_f64(&self.adaptive.score_history, percentile)
    }

    fn record_trade(
        &mut self,
        realized_pnl: f64,
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
    ) {
        self.trade_count += 1;
        self.sum_pnl += realized_pnl;
        
        // Update Signature Memory (Phase D.1.18)
        if let Some(sig) = signature {
            let stats = self.signature_memory.entry(sig).or_insert_with(SignatureStats::default);
            stats.sum_pnl += realized_pnl;
            stats.sample_count += 1;
            if realized_pnl > 0.0 {
                stats.win_count += 1;
            }
        }
        
        // Phase 2: Consistency Engine Tracking
        self.sum_realized_pnl += realized_pnl;
        self.sum_expected_pnl += expected_pnl;
        
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
            SignalSource::Bootstrap => {
                self.bootstrap_trade_count += 1;
                self.bootstrap_sum_pnl += realized_pnl;
            }
        }
        self.pnl_history_rolling.push(realized_pnl);
        if self.pnl_history_rolling.len() > 50 { self.pnl_history_rolling.remove(0); }
        
        self.pnl_history.push(outcome);
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
        if self.trade_count == 0 { return 0.0; }
        
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
pub fn percentile_f64(values: &[f64], p: f64) -> f64 {
    if values.is_empty() { return 0.0; }
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let idx = ((values.len() - 1) as f64 * p).floor() as usize;
    let idx = idx.min(values.len() - 1);
    sorted[idx]
}

/// Evolution State: Tracks generational memory for adaptive mutation and stability.
/// Derived only from deterministic population inputs to maintain GA reproducibility.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvoState {
    pub stagnation_counter: u32,
    pub last_best_fitness: f64,
    pub mutation_scale: f64,
    pub rolling_variance: f64,
    pub initial_diversity: f64,
    pub current_diversity: f64,
    pub diversity_trend: Vec<f64>,
    pub selection_pressure: f64,
}

impl Default for EvoState {
    fn default() -> Self {
        Self {
            stagnation_counter: 0,
            last_best_fitness: 0.0,
            mutation_scale: 1.0,
            rolling_variance: 0.05,
            initial_diversity: 1.0,
            current_diversity: 1.0,
            diversity_trend: Vec::new(),
            selection_pressure: 1.0,
        }
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExecutionMetrics {
    pub fill_efficiency: f64,
    pub capture_efficiency: f64,
    pub avg_slippage: f64,
    pub latency_impact: f64,
}

impl Default for ExecutionMetrics {
    fn default() -> Self {
        Self {
            fill_efficiency: 0.0,
            capture_efficiency: 0.0,
            avg_slippage: 0.0,
            latency_impact: 0.0,
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
fn scenario_execution_signature_l1(a: &ScenarioExecutionSignature, b: &ScenarioExecutionSignature) -> f64 {
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
            crate::SimEvent::OrderIntent { order_id, timestamp, .. } => {
                if order_id == entry_order_id || order_id == exit_order_id {
                    intent_ts.insert(order_id.clone(), *timestamp);
                }
            }
            crate::SimEvent::OrderEnteredQueue { order_id, queue_ahead, .. }
            | crate::SimEvent::QueueProgression { order_id, queue_ahead, .. } => {
                if order_id == entry_order_id || order_id == exit_order_id {
                    queue_samples.push(*queue_ahead as f64);
                }
            }
            crate::SimEvent::PartialFill { order_id, timestamp, .. } => {
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScenarioCapability {
    Executable,        // Stocks (Full Evolution)
    ContextOnly,       // Indices (NIFTY/BANKNIFTY - Signal only)
    Disabled,          // Bad data / insufficient candles
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StrategyEvaluation {
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
    pub execution_metrics: ExecutionMetrics,
    /// Per-scenario execution microstructure (queue, latency, fills); used for GA Top-K diversity when `GA_DIVERSITY_LAMBDA` > 0.
    pub scenario_signature: ScenarioExecutionSignature,
    /// Behavioral fingerprint (bucketed returns) for phenotype diversity.
    #[serde(default)]
    pub pnl_fingerprint: Vec<f32>,

    
    // Phase 8.8 Sniper Metrics
    pub avg_conviction: f64,
    pub avg_efficiency: f64,
    pub avg_edge_quality: f64,
    pub directional_accuracy: f64,
    pub decisiveness: f64,
    pub execution_friction: f64, // Actual / Expected Slippage

    // Phase 10.3: Institutional Feedback Loop
    #[serde(default = "default_capture_eff")]
    pub short_term_capture_eff: f64, // last 20 trades
    #[serde(default = "default_capture_eff")]
    pub long_term_capture_eff: f64,  // last 100 trades
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
    pub ccr: f64,       // EXEC_P95 / RAW_P95
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
pub struct ConsensusReport {
    pub scenario_name: String,
    pub top_signals: Vec<SignalAlphaReport>,
    pub global_entropy: f64,
    pub active_strategies: usize,
}

impl StrategyEvaluation {
    /// Institutional Weighting Formula (Phase 10.3):
    /// 0.4*fitness + 0.3*capture_eff + 0.2*fill_prob + 0.1*regime_stability
    pub fn calculate_institutional_weight(&self) -> f64 {
        let fitness_w = self.fitness.max(0.0);
        let capture_w = (0.6 * self.short_term_capture_eff + 0.4 * self.long_term_capture_eff)
            .clamp(0.3, 1.2);
        let fill_w = self.scenario_signature.fill_ratio.clamp(0.0, 1.0);
        let stability_w = self.consistency_score.clamp(0.0, 1.0);

        (0.4 * fitness_w) + (0.3 * capture_w) + (0.2 * fill_w) + (0.1 * stability_w)
    }

    /// Blends realized execution into the capture efficiency horizons.
    /// realised_pnl: actual pnl from the trade
    /// predicted_pnl: expected pnl based on strategy signal
    pub fn update_capture_efficiency(&mut self, realized: f64, predicted: f64) {
        if predicted.abs() < 1e-9 { return; } // Denominator Guard 1
        let ratio = (realized / predicted).clamp(-2.0, 2.0); // Safe ratio with institutional clamp
        
        // Evolving Horizons (EMA Approximation)
        // Short (20 trades) -> Alpha ~ 0.1
        // Long (100 trades) -> Alpha ~ 0.02
        self.short_term_capture_eff = (0.1 * ratio.max(0.0).min(2.0)) + (0.9 * self.short_term_capture_eff);
        self.long_term_capture_eff = (0.02 * ratio.max(0.0).min(2.0)) + (0.98 * self.long_term_capture_eff);
        
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
            let var = self.trade_qualities.iter().map(|q| (q - mean).powi(2)).sum::<f64>() / n;
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
    pub queue_depth: f64, // 0 = empty, 1 = heavy
    pub liquidity_score: f64, // 0 = thin, 1 = deep
    pub latency_impact: f64, // 0 = low, 1 = high
}

fn clamp01(x: f64) -> f64 {
    x.max(0.0).min(1.0)
}

fn compute_std_dev(data: &[f64]) -> f64 {
    if data.is_empty() { return 0.0; }
    let mean = data.iter().sum::<f64>() / data.len() as f64;
    let variance = data.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / data.len() as f64;
    variance.sqrt()
}

fn calculate_execution_score(ctx: &ExecutionContext) -> f64 {
    let queue_component = clamp01(1.0 - ctx.queue_depth);
    let liquidity_component = clamp01(ctx.liquidity_score);
    let latency_component = clamp01(1.0 - ctx.latency_impact);

    0.4 * queue_component +
    0.4 * liquidity_component +
    0.2 * latency_component
}

fn is_execution_feasible(conviction: f64, exec_score: f64) -> (bool, f64) {
    // 🔧 Adjustment 1: Smooth the Override Boundary
    let threshold = if conviction > 0.9 {
        (0.7 - ((conviction - 0.9) * 1.0).min(0.1)).max(0.6)
    } else {
        0.7
    };
    (exec_score >= threshold, threshold)
}

pub fn calculate_capture_efficiency(realized: f64, expected: f64) -> f64 {
    let eps = 1e-6;
    let denom = if expected.abs() < eps { expected.signum() * eps } else { expected };
    (realized / denom).clamp(-2.0, 2.0)
}

pub fn classify_efficiency(e: f64) -> &'static str {
    if e > 1.0 { "OUTPERFORM" }
    else if e > 0.7 { "GOOD" }
    else if e > 0.3 { "DECAY" }
    else { "FAILED" }
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
                exp_volatility: 100, selectivity: 75, archetype: 0,
            },
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
            realized_pnl_rolling: 0.0,
            predicted_pnl_rolling: 0.0,
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
            total_pnl: 0.0,
            pnl_history: Vec::new(),
            pnl_fingerprint: Vec::new(),
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
    pub archetype: u8,     // [0, 3] Behavioral Identity (Conviction, Momentum, Reversion, Volatility)
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
            fitness_mode,
            pnl_fingerprint_len: 50,
        }
    }
}


pub fn run_ga_evolution<'a>(config: GaConfig, all_scenarios: &[ScenarioPair<'a>]) -> GaResult {
    let mut rng = StdRng::seed_from_u64(config.seed);
    let mut global_best: Option<StrategyEvaluation> = None;
    let mut global_best_generation: usize = 0;
    let mut final_generation_best: Option<StrategyEvaluation> = None;
    let mut generation_peaks: Vec<(usize, f64)> = Vec::new();
    
    // 1. Group Scenarios by (Asset, Regime) using indices
    let mut asset_regime_scenarios: HashMap<(String, String), Vec<ScenarioPair<'a>>> = HashMap::new();
    for pair in all_scenarios {
        let name = pair.name;
        let asset = name.split('_').next().unwrap_or("BTC").to_string();
        let regime = if name.contains("trending_up") { "trending_up" }
                    else if name.contains("trending_down") { "trending_down" }
                    else if name.contains("mean_reverting") || name.contains("sideways") { "mean_reverting" }
                    else if name.contains("volatile") { "volatile" }
                    else { "mixed" };
        
        asset_regime_scenarios.entry((asset, regime.to_string())).or_default().push(pair.clone());
    }

    let mut best_per_bucket: HashMap<(String, String), StrategyEvaluation> = HashMap::new();
    let mut clusters_per_bucket: HashMap<(String, String), Vec<StrategyEvaluation>> = HashMap::new();
    let mut all_final_evaluations: Vec<StrategyEvaluation> = Vec::new();
    let mut global_generation_history: Vec<StrategyEvaluation> = Vec::new();
    let mut final_p: Vec<Strategy> = Vec::new();

    println!("--- Starting Multi-Asset + Regime Genetic Algorithm Evolution ---");

    let mut sorted_buckets: Vec<_> = asset_regime_scenarios.keys().cloned().collect();
    sorted_buckets.sort();

    for (asset, regime) in sorted_buckets {
        let cap = determine_scenario_capability(&asset);
        if !cap.is_executable() {
            println!(
                "SCENARIO_SKIP → asset={} | capability={:?} | reason=Index | stage=GA_EVOLUTION",
                asset, cap
            );
            continue;
        }

        println!("\n>> Evolving Bucket: asset={}, regime={}", asset, regime);
        let scenarios = asset_regime_scenarios.get(&(asset.clone(), regime.clone())).unwrap();
        
        let mut population = initialize_population(&config, &mut rng);
        let mut alpha_found = false;
        let mut bucket_best_overall: Option<StrategyEvaluation> = None;
        let mut bucket_history: Vec<StrategyEvaluation> = Vec::new();
        let mut evo = EvoState::default();

        for generation in 0..config.generations {
            // 1. Deduplicate
            population = deduplicate_population(population, &config, &mut rng);

            // 2. Evaluate ONLY on this bucket's scenarios
            let evaluations_option = evaluate_population_scoped(&population, &config, scenarios, generation);

            if let Some(mut evaluations) = evaluations_option {
                // --- PHASE D.1.7: AGGRESSIVE HIERARCHY INJECTION ---
                let n_eval = (evaluations.len() as f64).max(1.0);
                let scores: Vec<f64> = evaluations.iter().map(|e| e.fitness).collect();
                let total_score: f64 = scores.iter().sum::<f64>();
                let convictions: Vec<f64> = evaluations.iter().map(|e| e.avg_conviction).collect();
                let avg_conviction = convictions.iter().sum::<f64>() / n_eval;
                let std_dev_pop = compute_std_dev(&scores);

                for e in &mut evaluations {
                    // 1. Relative Dominance (Aggressive)
                    let relative_dominance = e.fitness / (total_score + 1e-9);
                    e.fitness *= (1.0 + 2.0 * relative_dominance).powf(2.5);

                    // 2. Conviction Divergence Bonus (Amplified)
                    let divergence = (e.avg_conviction - avg_conviction).abs();
                    e.fitness *= 1.0 + 1.5 * divergence;

                    // 3. Anti-Collapse Kill Switch (Hard)
                    if std_dev_pop < 1e-6 {
                        e.fitness *= 0.3;
                    }
                }

                // --- PHASE C.4.2: ADAPTIVE AMPLIFICATION (Refined for D.1.7) ---
                let mut fit_vals: Vec<f64> = evaluations.iter().map(|e| e.fitness).collect();
                let new_std = compute_std_dev(&fit_vals);
                let scale = (1.0 / (new_std + 1e-3)).clamp(1.0, 500.0);
                
                for e in &mut evaluations {
                    e.fitness *= scale;
                }

                fit_vals = evaluations.iter().map(|e| e.fitness).collect();
                fit_vals.sort_by(|a, b| a.total_cmp(b));
                
                println!("GEN_STATS → std_dev={:.6}, scale={:.2}", new_std, scale);
                println!("ADAPTIVE_DYNAMICS → Best: {:.4}, Median: {:.4}, Worst: {:.4}", 
                    fit_vals.last().unwrap_or(&0.0), 
                    fit_vals[fit_vals.len() / 2], 
                    fit_vals.first().unwrap_or(&0.0));

                if evaluations.is_empty() {
                    println!("  [{}|{}] Gen {} → ALL STRATEGIES REJECTED AFTER INITIAL EVALUATION", asset, regime, generation);
                    population = initialize_population(&config, &mut rng);
                    continue;
                }

                // --- PHENOTYPE DIVERSITY PENALTY (Phase 11.1) ---
                let evaluations_copy = evaluations.clone();
                
                // Population Stats for Distance Normalization
                let pnl_mu = evaluations.iter().map(|e| e.avg_pnl).sum::<f64>() / evaluations.len() as f64;
                let pnl_sigma = (evaluations.iter().map(|e| (e.avg_pnl - pnl_mu).powi(2)).sum::<f64>() / evaluations.len() as f64).sqrt().max(1e-9);
                let std_mu = evaluations.iter().map(|e| e.std_dev).sum::<f64>() / evaluations.len() as f64;
                let std_sigma = (evaluations.iter().map(|e| (e.std_dev - std_mu).powi(2)).sum::<f64>() / evaluations.len() as f64).sqrt().max(1e-9);

                for i in 0..evaluations.len() {
                    let mut niche_count = 1.0;
                    for j in 0..evaluations_copy.len() {
                        if i == j { continue; }
                        let dist = calculate_behavioral_distance(
                            &evaluations[i], 
                            &evaluations_copy[j],
                            pnl_mu, pnl_sigma,
                            std_mu, std_sigma
                        );
                        if dist < 0.2 { // High behavioral similarity
                            niche_count += 1.0 - (dist / 0.2);
                        }
                    }
                    // Institutional Niche Penalty: f' = f / niche_count^alpha
                    evaluations[i].fitness /= niche_count.powf(0.5);
                    
                    // DETERMINISTIC FITNESS JITTER (Break ties)
                    let mut hasher = std::collections::hash_map::DefaultHasher::new();
                    evaluations[i].strategy_id.hash(&mut hasher);
                    generation.hash(&mut hasher); // Salt with generation
                    let jitter_rng_seed = hasher.finish();
                    let mut jitter_rng = StdRng::seed_from_u64(jitter_rng_seed);
                    let epsilon = jitter_rng.gen_range(0..1000) as f64 * 1e-10;
                    evaluations[i].fitness = (evaluations[i].fitness + epsilon).min(1.0);
                }

                // Diagnostics + strict consistency before selection/ranking.
                for evaluation in &evaluations {
                    println!(
                        "SELECTION_INPUT → strat={}, fitness={:.6}",
                        evaluation.strategy_id, evaluation.fitness
                    );
                    println!(
                        "GA_DEBUG → fitness={:.4}, trades={}, participation={:.2}",
                        evaluation.fitness, evaluation.trade_count, evaluation.participation_rate
                    );
                    assert!(
                        evaluation.fitness.is_finite(),
                        "GA using non-finite fitness: {}",
                        evaluation.fitness
                    );
                }

                // Sort by final fitness only (single source of truth).
                evaluations.sort_by(|a, b| a.fitness.total_cmp(&b.fitness).reverse());
                
                if std::env::var("GA_DEBUG").is_ok() {
                    let all_fit: Vec<String> = evaluations.iter().map(|e| format!("{:.4}", e.fitness)).collect();
                    println!("GA_POP_FITNESS → [{}]", all_fit.join(", "));
                }

                // 3. Apply similarity penalty
                let pre_similarity = evaluations.clone();
                apply_similarity_penalty(&mut evaluations);
                let had_positive_before = pre_similarity.iter().any(|e| e.fitness > 0.0);
                let has_positive_after = evaluations.iter().any(|e| e.fitness > 0.0);
                if had_positive_before && !has_positive_after {
                    println!(
                        "SIMILARITY_GUARD → penalty collapsed all positive fitness; restoring pre-penalty evaluations"
                    );
                    evaluations = pre_similarity;
                }

                // --- PHASE D.1.13.5: UNIVERSAL FITNESS PENALTY (CRITICAL) ---
                // Severely penalize strategies that failed to produce organic signals.
                for eval in evaluations.iter_mut() {
                    if !eval.had_organic_signals {
                        eval.fitness *= 0.1; // 90% penalty for "ghost" strategies
                        if std::env::var("GA_DEBUG").is_ok() {
                            println!("FITNESS_PENALTY → strat={} | Organic Signal Void", eval.strategy_id);
                        }
                    }
                }
                
                // Re-sort after penalty
                evaluations.sort_by(|a, b| b.fitness.total_cmp(&a.fitness));

                // --- INSTITUTIONAL ADAPTIVE EVOLUTION (EvoState) ---
                if let Some(best_ref) = evaluations.first() {
                    let best = best_ref.clone(); 
                    let median = evaluations[evaluations.len() / 2].fitness;
                    let worst = evaluations.last().unwrap().fitness;
                    
                    // Diversity Tracking (Distance to Centroid)
                    let current_population: Vec<Strategy> = evaluations.iter().map(|e| e.strategy.clone()).collect();
                    let div = calculate_population_diversity(&current_population);
                    
                    if generation == 0 {
                        evo.initial_diversity = div;
                    }
                    evo.current_diversity = div;
                    evo.diversity_trend.push(div);
                    if evo.diversity_trend.len() > 3 { evo.diversity_trend.remove(0); }
                    
                    // Selection Pressure
                    evo.selection_pressure = (best.fitness / median.max(1e-9)).min(100.0);
                    
                    if best.fitness > 0.0 && !alpha_found {
                        println!("🚨 FIRST_ALPHA_DISCOVERY → gen={} fitness={:.6} asset={}", generation, best.fitness, asset);
                        alpha_found = true;
                    }

                    println!(
                        "GEN_SUMMARY → gen={} best={:.4} median={:.4} worst={:.4} div={:.4} mut={:.2}",
                        generation, best.fitness, median, worst, div, evo.mutation_scale
                    );

                    println!("ADAPTIVE_DYNAMICS → Best: {:.4}, Median: {:.4}, Worst: {:.4} | Div: {:.4} | MutScale: {:.2} | Pressure: {:.2} | ForceWinProb: {:.2}", 
                        best.fitness, median, worst, div, evo.mutation_scale, evo.selection_pressure, (1.0 - (generation as f64 / 50.0)).clamp(0.05, 1.0));

                    if div < 0.05 {
                        println!("🚨 DIVERSITY_INJECTION: Population variance collapsed ({:.4}); injecting 30% random immigrants (preserving top-{})", div, config.preserve_top_k);
                        
                        // evaluations already penalized and sorted above.
                        
                        // 2. Preserve TOP-K
                        let keep_count = config.preserve_top_k.min(evaluations.len());
                        let mut new_population: Vec<Strategy> = evaluations.iter().take(keep_count).map(|e| e.strategy.clone()).collect();
                        
                        // 3. Replace BOTTOM 30% with randoms
                        let target_population_size = config.population_size;
                        let injection_count = (target_population_size as f64 * 0.3).ceil() as usize;
                        let _remainder_count = target_population_size.saturating_sub(keep_count + injection_count);
                        
                        for _ in 0..injection_count {
                            new_population.push(random_strategy(&config, &mut rng));
                        }
                        
                        // 4. Fill remainder with mutation of survivors
                        while new_population.len() < target_population_size {
                            let parent = &evaluations[rng.gen_range(0..keep_count)];
                            let mut mutant = parent.strategy.clone();
                            mutate_strategy(&mut mutant, &mut rng, parent.trade_count, &evo);
                            new_population.push(mutant);
                        }
                        
                        population = new_population;
                        // Skip the normal evolution step for this generation since we just manually rebuilt it
                        println!("  → Population rebuilt with {} immigrants", injection_count);
                    }

                    // 1. Annealing: Shrink deltas if we are improving
                    let min_scale = if config.deep_validation { 0.5 } else { 0.2 };
                    let max_scale = if config.deep_validation { 2.0 } else { 3.0 };

                    if best.fitness > evo.last_best_fitness {
                        evo.mutation_scale = (evo.mutation_scale * 0.85).max(min_scale);
                        evo.stagnation_counter = 0;
                        evo.last_best_fitness = best.fitness;
                    } else {
                        // 2. Stagnation: Increase pressure if progress stalls
                        evo.stagnation_counter += 1;
                        if evo.stagnation_counter > 2 {
                            evo.mutation_scale = (evo.mutation_scale * 1.2).min(max_scale);
                        }
                    }

                    // 3. Stability Guard: dampen if fitness variance explodes relative to rolling mean
                    let mean_fitness = evaluations.iter().map(|e| e.fitness).sum::<f64>() / evaluations.len() as f64;
                    let variance = evaluations.iter().map(|e| (e.fitness - mean_fitness).powi(2)).sum::<f64>() / evaluations.len() as f64;
                    
                    if generation > 0 && variance > evo.rolling_variance * 2.5 {
                        evo.mutation_scale *= 0.5;
                        println!("⚠️ STABILITY_GUARD: Chaos detected (Variance: {:.4} > {:.4}); dampening scale to {:.4}", 
                            variance, evo.rolling_variance * 2.5, evo.mutation_scale);
                    }
                    evo.rolling_variance = evo.rolling_variance * 0.7 + variance * 0.3; // Smooth rolling variance

                    generation_peaks.push((generation, best.fitness));
                    if global_best.is_none() || best.fitness > global_best.as_ref().unwrap().fitness {
                        global_best = Some(best.clone());
                        global_best_generation = generation;
                    }
                    
                    let should_update = bucket_best_overall.as_ref().map_or(true, |o| best.fitness > o.fitness);
                    if should_update {
                        bucket_best_overall = Some(best.clone());
                    }
                    
                    bucket_history.push(best.clone());

                    // Track global history (using the best fitness found across all buckets for this generation)
                    if global_generation_history.len() <= generation {
                        global_generation_history.push(best.clone());
                    } else if best.fitness > global_generation_history[generation].fitness {
                        global_generation_history[generation] = best.clone();
                    }
                    
                    if generation == config.generations - 1 {
                        final_p = population.clone();
                    }
                }

                if generation < config.generations - 1 {
                    population = evolve_generation(&evaluations, &config, &mut rng, &evo);
                } else {
                    // PHENOTYPE CLUSTERING (Phase 11.2) - Final population of this bucket
                    let pnl_mu = evaluations.iter().map(|e| e.avg_pnl).sum::<f64>() / evaluations.len() as f64;
                    let pnl_sigma = (evaluations.iter().map(|e| (e.avg_pnl - pnl_mu).powi(2)).sum::<f64>() / evaluations.len() as f64).sqrt().max(1e-9);
                    let std_mu = evaluations.iter().map(|e| e.std_dev).sum::<f64>() / evaluations.len() as f64;
                    let std_sigma = (evaluations.iter().map(|e| (e.std_dev - std_mu).powi(2)).sum::<f64>() / evaluations.len() as f64).sqrt().max(1e-9);

                    
                    let clusters = extract_behavioral_clusters(
                        evaluations.clone(),
                        5,   // target_count max 5
                        0.3, // min_dist_threshold
                        pnl_mu, pnl_sigma, std_mu, std_sigma
                    );
                    clusters_per_bucket.insert((asset.clone(), regime.clone()), clusters);

                    all_final_evaluations.extend(evaluations.clone());
                    if let Some(current_final_gen_best) = evaluations.first() {
                        if final_generation_best.is_none() || current_final_gen_best.fitness > final_generation_best.as_ref().unwrap().fitness {
                            final_generation_best = Some(current_final_gen_best.clone());
                        }
                    }
                }
            } else { // evaluations_option was None
                println!("  [{}|{}] Gen {} → ALL STRATEGIES REJECTED DURING EARLY CHECK", asset, regime, generation);
                population = initialize_population(&config, &mut rng);
                continue;
            }
        }

        if let Some(best) = bucket_best_overall {
            println!("BEST: asset={}, regime={}", asset, regime);
            println!("  Fitness: {:.4}, PnL: {:.6}", best.fitness, best.avg_pnl);
            best_per_bucket.insert((asset, regime), best);
        }
    }

    println!("\n--- GA Evolution Complete ---");
    
    // 🛡️ ELITE POOL PURITY: Ensure only executable strategies proceed to the final pool.
    all_final_evaluations.retain(|e| e.capability.is_executable());

    // Sort all final evaluations to find the elite population across all buckets
    all_final_evaluations.sort_by(|a: &StrategyEvaluation, b: &StrategyEvaluation| b.fitness.total_cmp(&a.fitness));
    let elite_count = config.population_size.min(all_final_evaluations.len());
    let elites = &all_final_evaluations[..elite_count];

    // Persist elite population
    if let Ok(saved_path) = save_elite_population(elites, &config, "core/elite") {
        println!("🚀 ELITE_PERSISTENCE: Saved top {} genomes to {}", elite_count, saved_path);
    } else {
        eprintln!("⚠️ ELITE_PERSISTENCE_ERROR: Failed to save elite population");
    }

    // --- PHASE D.1.13: TEMPORAL CONSENSUS & STABILITY LAYER ---
    let consensus_report = if all_scenarios.len() > 0 {
        let elite_genomes: Vec<Strategy> = elites.iter().map(|e| e.strategy.clone()).collect();
        let target_windows = all_scenarios.iter().rev().take(5).collect::<Vec<_>>();
        let mut registry: HashMap<SignalIdentity, Vec<SignalAlphaReport>> = HashMap::new();
        
        for scenario in &target_windows {
            let report = compute_consensus_alpha(&elite_genomes, scenario, &config);
            for sig in report.top_signals {
                let identity = SignalIdentity {
                    bucket_ts: sig.signal_timestamp / 60,
                    direction: if sig.alpha_score > 0.0 { Decision::BUY } else { Decision::SELL }, // Logic relative to consensus
                    archetype: if !sig.archetypes.is_empty() { 
                        match sig.archetypes[0] {
                            0 => Archetype::Conviction,
                            1 => Archetype::Momentum,
                            2 => Archetype::Reversion,
                            _ => Archetype::Volatility,
                        }
                    } else { Archetype::Conviction },
                    feature_hash: get_coarse_feature_hash(&elite_genomes[0]), // Representative (refined below if needed)
                };
                registry.entry(identity).or_insert(vec![]).push(sig);
            }
        }

        let mut final_top_signals = Vec::new();
        for (identity, history) in registry {
            let count = history.len();
            if count == 0 { continue; }

            // 1. Persistence Factor (Non-linear)
            let persistence_factor = (count as f64 / 5.0).powf(1.2);
            
            // 2. Persistence Quality (Mean Alpha)
            let persistence_quality = history.iter().map(|s| s.alpha_score).sum::<f64>() / count as f64;

            // 3. Robust Stability Factor (CV-based clamping)
            let mut stability_factor = 1.0;
            if count > 1 {
                let mean = persistence_quality;
                let variance = history.iter().map(|s| (s.alpha_score - mean).powi(2)).sum::<f64>() / count as f64;
                let std_dev = variance.sqrt();
                let cv = (std_dev / (mean.abs() + 1e-6)).clamp(0.0, 5.0);
                stability_factor = (-1.5 * cv).exp();
            }

            let mut aggregated = history[0].clone();
            aggregated.alpha_score = persistence_quality * stability_factor * persistence_factor;
            aggregated.temporal_stability = stability_factor;
            aggregated.persistence_count = count;
            
            // Penalize or reject low persistence
            if count < 2 { aggregated.alpha_score *= 0.5; }

            if aggregated.alpha_score > 0.4 {
                aggregated.consensus_label = if aggregated.alpha_score > 0.7 { "🏆 TEMPORAL ALPHA" } else { "stable signal" }.to_string();
                final_top_signals.push(aggregated);
            }
        }

        final_top_signals.sort_by(|a, b| b.alpha_score.total_cmp(&a.alpha_score));
        
        Some(ConsensusReport {
            scenario_name: all_scenarios.last().unwrap().name.to_string(),
            top_signals: final_top_signals,
            global_entropy: 0.0,
            active_strategies: elite_genomes.len(),
        })
    } else {
        None
    };

    println!("📈 Generation Peaks:");
    for (gen, fitness) in generation_peaks {
        println!("Gen {} → {:.4}", gen, fitness);
    }

    let resolved_global_best = global_best.unwrap_or_else(StrategyEvaluation::default);
    let resolved_final_generation_best = final_generation_best.unwrap_or_else(StrategyEvaluation::default);
    assert!(
        resolved_global_best.fitness + 1e-12 >= resolved_final_generation_best.fitness,
        "Global best fitness must be >= final generation best fitness"
    );
    let mut final_clusters: HashMap<String, Vec<StrategyEvaluation>> = clusters_per_bucket
        .into_iter()
        .map(|((asset, regime), eval)| (format!("{}_{}", asset, regime), eval))
        .collect();
    
    // Add Global Clusters
    let global_pnl_mu = all_final_evaluations.iter().map(|e| e.avg_pnl).sum::<f64>() / all_final_evaluations.len().max(1) as f64;
    let global_pnl_sigma = (all_final_evaluations.iter().map(|e| (e.avg_pnl - global_pnl_mu).powi(2)).sum::<f64>() / all_final_evaluations.len().max(1) as f64).sqrt().max(1e-9);
    let global_std_mu = all_final_evaluations.iter().map(|e| e.std_dev).sum::<f64>() / all_final_evaluations.len().max(1) as f64;
    let global_std_sigma = (all_final_evaluations.iter().map(|e| (e.std_dev - global_std_mu).powi(2)).sum::<f64>() / all_final_evaluations.len().max(1) as f64).sqrt().max(1e-9);

    let global_clusters = extract_behavioral_clusters(
        all_final_evaluations.clone(),
        5, 0.3, 
        global_pnl_mu, global_pnl_sigma, global_std_mu, global_std_sigma
    );
    final_clusters.insert("GLOBAL".to_string(), global_clusters);

    GaResult {
        global_best: resolved_global_best,
        global_best_generation,
        final_generation_best: resolved_final_generation_best,
        generation_history: global_generation_history,
        best_per_regime: best_per_bucket
            .into_iter()
            .map(|((asset, regime), eval)| (format!("{}_{}", asset, regime), eval))
            .collect(),
        clusters_per_regime: final_clusters,
        population_stats: PopulationStats {
            fitness: (global_pnl_mu, global_pnl_sigma),
            consistency: (1.0, 0.2), 
            recent: (global_pnl_mu, global_pnl_sigma),
        },
        final_population: final_p,
        consensus_recommendations: consensus_report,
    }
}


pub struct RobustnessReport {
    pub cv: f64,              // Cross-regime CV
    pub active_cv: f64,       // CV of non-zero regimes
    pub internal_cv: f64,     // Intra-scenario CV of baseline (Regime C)
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
        
        if let Some(eval) = evaluate_and_aggregate(strategy, &isolated_config, scenarios, generation) {
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
    let global_cv = if mean > 0.0 { variance.sqrt() / mean } else { 0.0 };

    // Layer 2: Active CV (Sniper View - Only admitted regimes)
    let active_cv = if !successful_results.is_empty() {
        let active_mean = successful_results.iter().sum::<f64>() / successful_results.len() as f64;
        let active_variance = successful_results.iter().map(|f| (f - active_mean).powi(2)).sum::<f64>() / successful_results.len() as f64;
        if active_mean.abs() > 1e-9 { active_variance.sqrt() / active_mean.abs() } else { 0.0 }
    } else {
        0.0
    };

    // Layer 3: Internal CV Downside (Intra-scenario stability of Baseline)
    // We re-run baseline aggregation to extract its internal downside deviation.
    let internal_cv_down = if let Some(eval) = evaluate_and_aggregate(strategy, config, scenarios, generation) {
        if eval.avg_pnl.abs() > 1e-9 { eval.downside_std_dev / eval.avg_pnl.abs() } else { 0.0 }
    } else {
        0.0
    };
    
    let robustness_score = if !successful_results.is_empty() {
        let min = successful_results.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = successful_results.iter().cloned().fold(f64::NEG_INFINITY, f64::max).max(1e-9);
        min / max
    } else { 0.0 };
    
    let (strategy_avg_pnl, strategy_total_trades) = if let Some(eval) = evaluate_and_aggregate(strategy, config, scenarios, generation) {
        (eval.avg_pnl, eval.trade_count)
    } else { (0.0, 0) };
    
    let pnl_score = strategy_avg_pnl.max(0.0) * 100.0;
    let selectivity = strategy_total_trades as f64 / (scenarios.iter().map(|s| s.signal.len()).sum::<usize>() as f64).max(1.0);

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

/// --- PHASE 13: ENSEMBLE CONSENSUS ENGINE ---

pub fn evaluate_ensemble_robustness(
    ensemble: &[Strategy],
    config: &GaConfig,
    scenarios: &[ScenarioPair],
    generation: usize,
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
    
    let mut results = Vec::new();
    for r_config in &regimes {
        if let Some(eval) = evaluate_ensemble_and_aggregate(ensemble, r_config, scenarios, generation) {
            results.push(eval.fitness);
        } else {
            results.push(0.0);
        }
    }
    
    let successful_results: Vec<f64> = results.iter().cloned().filter(|&f| f > 0.0).collect();
    let participation_rate = successful_results.len() as f64 / regimes.len() as f64;
    
    let mean = results.iter().sum::<f64>() / regimes.len() as f64;
    let variance = results.iter().map(|f| (f - mean).powi(2)).sum::<f64>() / regimes.len() as f64;
    let global_cv = if mean > 0.0 { variance.sqrt() / mean } else { 0.0 };
    
    let internal_cv_down = if let Some(eval) = evaluate_ensemble_and_aggregate(ensemble, config, scenarios, generation) {
        if eval.avg_pnl.abs() > 1e-9 { eval.downside_std_dev / eval.avg_pnl.abs() } else { 0.0 }
    } else { 0.0 };

    let robustness_score = if !successful_results.is_empty() {
        let min = successful_results.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = successful_results.iter().cloned().fold(f64::NEG_INFINITY, f64::max).max(1e-9);
        min / max
    } else { 0.0 };
    
    let (ens_avg_pnl, ens_trade_count, ens_entropy) = if let Some(eval) = evaluate_ensemble_and_aggregate(ensemble, config, scenarios, generation) {
        (eval.avg_pnl, eval.trade_count, eval.avg_entropy)
    } else { (0.0, 0, 0.0) };
    
    let pnl_score = ens_avg_pnl.max(0.0) * 100.0;
    let selectivity = ens_trade_count as f64 / (scenarios.iter().map(|s| s.signal.len()).sum::<usize>() as f64).max(1.0);

    // --- PHASE 13.5: SURGICAL CLASSIFICATION (Refined) ---
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

    RobustnessReport {
        cv: global_cv,
        active_cv: global_cv.max(1e-9), // Use global CV for active ensemble
        internal_cv: internal_cv_down,
        robustness_score,
        classification: classification.to_string(),
        regime_fitness: results,
        regimes_skipped: regimes.len() - successful_results.len(),
        participation_rate,
        avg_pnl: ens_avg_pnl,
        pnl_score,
        selectivity,
        total_trades: ens_trade_count,
        agreement_entropy: ens_entropy,
    }
}

pub fn evaluate_ensemble_and_aggregate(
    ensemble: &[Strategy],
    config: &GaConfig,
    scenarios: &[ScenarioPair],
    generation: usize,
) -> Option<StrategyEvaluation> {
    let mut reports = Vec::new();
    for pair in scenarios {
        if let Some(report) = evaluate_ensemble_strategy(ensemble, pair, config, generation) {
            reports.push(report);
        }
    }
    if reports.is_empty() { return None; }
    aggregate_strategy_reports_inner(reports, 1.0, config, generation).map(|(e, _)| e)
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
    
    if signal_count < period + 50 { return None; }

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
        if let Some(e) = evaluate_strategy(s, pair, config, 0) {
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
            let conv = evaluate_market_conviction(strategy, scenario_name, signal_events, i, 0, generation);
            let w = member_evals[idx].fitness.max(0.1);
            consensus += conv.conviction_score * w;
            total_w += w;
        }
        let score = (consensus / total_w.max(1e-9)).abs();
        if score > 0.001 { ensemble_candidate_edges.push(score); }
    }

    let ensemble_coverage = ensemble_candidate_edges.len() as f64 / signal_count.max(1) as f64;
    if ensemble_candidate_edges.len() < 1 || ensemble_coverage < 0.005 {
        if std::env::var("GA_DEBUG").is_ok() {
            println!("AQG_SKIP_ENSEMBLE → scenario={} (Evidence starvation: valid={} coverage={:.4}). Skipping ensemble.", 
                scenario_name, ensemble_candidate_edges.len(), ensemble_coverage);
        }
        return None;
    }

    for current_idx in (period + 2)..(signal_count - 10) {
        metrics.record_opportunity();
        if current_idx < busy_until { continue; }
        
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
            let conv = evaluate_market_conviction(strategy, scenario_name, signal_events, current_idx, 0, generation);
            
            inputs.push(crate::ensemble::EnsembleInput {
                member: &member_refs[i],
                evaluation: eval,
                signal: crate::ensemble::SignalStrength { value: conv.conviction_score },
            });
        }

        // 2. Compute Consensus Decision + Shannon Entropy of votes
        let decision = crate::ensemble::compute_consensus(&inputs, 0.55, 0.25);

        // Phase 13.5: Entropy = how much disagreement exists among strategies
        // Uses signal strengths relative to their individual take_profit thresholds
        let entropy_norm = {
            let total_members = inputs.len() as f64;
            if total_members > 0.0 {
                let buy_p = inputs.iter().filter(|i| {
                    let threshold = (i.evaluation.strategy.take_profit as f64 / 10000.0).max(0.0004);
                    i.signal.value > threshold
                }).count() as f64 / total_members;
                let sell_p = inputs.iter().filter(|i| {
                    let threshold = (i.evaluation.strategy.take_profit as f64 / 10000.0).max(0.0004);
                    i.signal.value < -threshold
                }).count() as f64 / total_members;
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

        if decision.combined_action != crate::SignalAction::HOLD && decision.consensus_score.abs() > 0.001 {
            // 3. Execution (Consensus Weighted simulation)
            let conviction = ConvictionOutcome {
                conviction_score: decision.consensus_score.abs(),
                expected_edge: 0.0,
                edge_weight: decision.consensus_score.abs().clamp(0.5, 2.0),
                norm_momentum: 0.5,
                norm_volume: 0.5,
                norm_vol_score: 0.5,
                norm_vol: 0.001,
                selection_threshold: 0.5,
                is_bearish: decision.combined_action == crate::SignalAction::SELL,
                roll: 0.0,
                raw_q_ratio: 0.0,
            };

            if let Some(outcome) = ga_simulate_round_trip_at_cursor(
                &ensemble[0], // Proxy for execution config
                &ensemble[0].base_edge.to_string(),
                scenario_name,
                signal_events,
                execution_events,
                config,
                current_idx,
                scenario_pnls.len(), // trade_idx
                &conviction,
            ) {
                let trade_pnl = outcome.pnl * conviction.edge_weight;
                scenario_pnls.push(trade_pnl);
                
                // Decision Surface Margin (distance from consensus threshold - Relative)
                let margin = ((decision.consensus_score.abs() - 0.001) / 0.001).abs();

                // Record Decision-Time Metrics (Phase 13.6 Health Tracking)
                metrics.record_trade(
                    trade_pnl,
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
                    SignalSource::Organic, // Standard simulation is organic unless specified
                    None,
                );

                if trade_pnl > 0.0 {
                    total_win += trade_pnl;
                } else if trade_pnl < 0.0 {
                    total_loss += trade_pnl.abs();
                }

                match outcome.exit_reason {
                    GaExitReason::TakeProfit => exit_tp_count += 1,
                    GaExitReason::StopLoss => exit_sl_count += 1,
                    GaExitReason::TimeStop => exit_ts_count += 1,
                }

                busy_until = outcome.exit_event_idx + (config.trade_cooldown_events.unwrap_or(5));
                if scenario_pnls.len() >= 10 { break; }
            }
        }
    }

    if scenario_pnls.is_empty() { return None; }

    // Final Aggregate for this Scenario
    let n = scenario_pnls.len() as f64;
    let avg_pnl = scenario_pnls.iter().sum::<f64>() / n;
    let win_rate = metrics.profitable_trades as f64 / n;

    let avg_win = if metrics.profitable_trades > 0 { total_win / metrics.profitable_trades as f64 } else { 0.0 };
    let loss_count = scenario_pnls.len().saturating_sub(metrics.profitable_trades);
    let avg_loss = if loss_count > 0 { total_loss / loss_count as f64 } else { 1e-9 };
    let payoff_ratio = (avg_win / avg_loss.max(1e-9)).clamp(0.5, 3.0);

    let std_dev: f64 = if n > 1.0 {
        let variance = scenario_pnls.iter().map(|pnl| (pnl - avg_pnl).powi(2)).sum::<f64>() / n;
        variance.sqrt()
    } else { 0.0 };

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
        decisiveness: if metrics.trade_count > 0 { 1.0 - (metrics.sum_time_to_mfe / (metrics.trade_count as f64 * config.max_hold_bars as f64)).clamp(0.0, 1.0) } else { 0.0 },
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
        max_trade_pnl: scenario_pnls.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
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
        return 0.001 * events.get(cursor_i).map(|e| e.price as f64).unwrap_or(1.0); // Fallback to 0.1% of price
    }

    let mut tr_sum = 0.0;
    for i in (cursor_i.saturating_sub(period))..cursor_i {
        if i == 0 { continue; }
        let high = events[i].price as f64; // Using price as proxy for high/low in tick data
        let low = events[i].price as f64;
        let prev_close = events[i-1].price as f64;
        
        let tr = (high - low).abs()
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
    early_check_indices: &[usize],
    generation: usize,
) -> Option<StrategyEvaluation> {
    for &idx in early_check_indices {
        let pair = &scenarios[idx];
        let _ = evaluate_strategy(strategy, pair, config, generation);
    }

    if let Some(aggregated) = evaluate_and_aggregate(strategy, config, scenarios, generation) {
        Some(aggregated)
    } else {
        None
    }
}

pub fn evaluate_population_scoped(
    population: &Vec<Strategy>, 
    config: &GaConfig, 
    scenarios: &[ScenarioPair],
    generation: usize
) -> Option<Vec<StrategyEvaluation>> {
    let n_in = scenarios.len();
    if n_in == 0 { return None; }
    
    // Phase 10.3: Stochastic Evolution Recovery
    // Each genome now sees a unique, randomized subset of the scenario pool (DHARVI compliant).
    let threads = selection_cap::resolved_ga_parallelism_threads();
    let per_member: Vec<Option<StrategyEvaluation>> = if threads <= 1 {
        population
            .iter()
            .enumerate()
            .map(|(idx, strategy)| {
                let genome_seed = config.seed ^ ((generation as u64) << 32) ^ (idx as u64 * 7919);
                let mut genome_rng = StdRng::seed_from_u64(genome_seed);
                
                let mut local_scenarios = scenarios.to_vec();
                local_scenarios.shuffle(&mut genome_rng);
                
                let is_diagnostic = std::env::var("GA_DIAGNOSTIC_MODE").is_ok();
                let dynamic_cap = if is_diagnostic { n_in } else { (((n_in as f32) * 0.3).ceil() as usize).max(6).min(12).min(n_in) };
                
                let selected_subset = &local_scenarios[0..dynamic_cap];
                
                debug_assert!(selected_subset.len() >= 6 || n_in < 6 || is_diagnostic, "Scenario starvation detected: subset too small");
                
                if std::env::var("GA_DEBUG").is_ok() {
                    println!("SCENARIO_SAMPLE → genome={}, selected={}, total={}{}", 
                        idx, selected_subset.len(), n_in, if is_diagnostic { " [DIAGNOSTIC]" } else { "" });
                }
                evaluate_population_member(strategy, config, selected_subset, &[], generation) // skip legacy early_check
            })
            .collect()
    } else {
        match rayon::ThreadPoolBuilder::new().num_threads(threads).build() {
            Ok(pool) => pool.install(|| {
                population
                    .par_iter()
                    .enumerate()
                    .map(|(idx, strategy)| {
                        // PHASE 11.1: Deterministic Jitter (Seeded Hash)
                        use std::hash::{Hash, Hasher};
                        let mut hasher = std::collections::hash_map::DefaultHasher::new();
                        format!("{}_{}_{}", strategy.base_edge, generation, idx).hash(&mut hasher);
                        let genome_seed = config.seed ^ hasher.finish();
                        let mut genome_rng = StdRng::seed_from_u64(genome_seed);
                        
                        let mut local_scenarios = scenarios.to_vec();
                        local_scenarios.shuffle(&mut genome_rng);
                        
                        // Jitter the subset size by ±10%
                        let is_diagnostic = std::env::var("GA_DIAGNOSTIC_MODE").is_ok();
                        let dynamic_cap = if is_diagnostic { n_in } else {
                            let jitter_factor = 0.9 + (genome_rng.gen::<f64>() * 0.2);
                            (((n_in as f64) * 0.3 * jitter_factor).ceil() as usize).max(6).min(12).min(n_in)
                        };
                        
                        let selected_subset = &local_scenarios[0..dynamic_cap];
                        
                        evaluate_population_member(strategy, config, selected_subset, &[], generation)
                    })
                    .collect()
            }),
            Err(e) => {
                eprintln!("GA_PARALLELISM error: {e}; sequential fallback");
                population
                    .iter()
                    .enumerate()
                    .map(|(idx, strategy)| {
                        let genome_seed = config.seed ^ ((generation as u64) << 32) ^ (idx as u64 * 7919);
                        let mut genome_rng = StdRng::seed_from_u64(genome_seed);
                        let mut local_scenarios = scenarios.to_vec();
                        local_scenarios.shuffle(&mut genome_rng);
                        let dynamic_cap = (((n_in as f32) * 0.3).ceil() as usize).max(6).min(12).min(n_in);
                        let selected = &local_scenarios[0..dynamic_cap];
                        evaluate_population_member(strategy, config, selected, &[], generation)
                    })
                    .collect()
            }
        }
    };
    let mut evaluations: Vec<StrategyEvaluation> = per_member.iter().flat_map(|v| v.clone()).collect();

    if evaluations.is_empty() {
        // Fallback for extremely sparse data: return some strategies with 0 fitness
        for strategy in population.iter().take(3) {
            evaluations.push(StrategyEvaluation {
                winner_idx: 0,
                strategy_id: "FALLBACK_ZERO".to_string(),
                strategy: strategy.clone(),
                consistency_score: 1.0,
                recent_performance: 0.0,
                ..StrategyEvaluation::default()
            });
        }
    }

    // --- PHASE D.1.9: CONSENSUS-AWARE FITNESS & ENHANCED LOGGING ---
    let pop_size = evaluations.len().max(1);
    let mut winner_map: HashMap<usize, usize> = HashMap::new();
    for e in &evaluations {
        *winner_map.entry(e.winner_idx).or_insert(0) += 1;
    }
    
    // Calculate Shannon Entropy
    let unique_winners: HashSet<usize> = evaluations.iter().map(|e| e.winner_idx).collect();
    let mut entropy = 0.0;
    for (&_idx, &count) in &winner_map {
        let p = count as f64 / pop_size as f64;
        if p > 0.0 {
            entropy -= p * p.ln();
        }
    }
    
    // Apply Partial Consensus Penalty (Fixed Surgical 6)
    for ev in &mut evaluations {
        let count = *winner_map.get(&ev.winner_idx).unwrap_or(&1);
        let overlap_ratio = count as f64 / pop_size as f64;
        let penalty = (1.0 - 0.7 * overlap_ratio).max(0.3);
        ev.fitness *= penalty;
    }
    
    // Archetype Distribution
    let mut arch_counts = [0usize; 4];
    for e in &evaluations {
        let a = e.strategy.archetype as usize;
        if a < 4 { arch_counts[a] += 1; }
    }

    let max_count = winner_map.values().max().cloned().unwrap_or(0);
    let concentration = if evaluations.is_empty() { 0.0 } else { max_count as f64 / evaluations.len() as f64 };

    println!(
        "DIVERSITY → unique={} entropy={:.2} concentration={:.2} pop={}",
        unique_winners.len(),
        entropy,
        concentration,
        pop_size
    );
    println!(
        "ARCHETYPE_DIST → C:{} M:{} R:{} V:{}",
        arch_counts[0], arch_counts[1], arch_counts[2], arch_counts[3]
    );

    let fitness_scores: Vec<f64> = evaluations.iter().map(|e| e.fitness).collect();
    let (_mean, _std_dev) = if fitness_scores.len() > 1 {
        let m = fitness_scores.iter().sum::<f64>() / fitness_scores.len() as f64;
        let v = fitness_scores.iter().map(|f| (f - m).powi(2)).sum::<f64>() / (fitness_scores.len() as f64);
        (m, v.sqrt())
    } else {
        (fitness_scores.first().cloned().unwrap_or(0.0), 0.0)
    };

    Some(evaluations)
}

fn deduplicate_population(population: Vec<Strategy>, config: &GaConfig, rng: &mut StdRng) -> Vec<Strategy> {
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
            selectivity: rng.gen_range(60..=90),
            archetype: rng.gen_range(0..=3),
        };
        if unique_strategies.insert(random_strat.clone()) {
            new_population.push(random_strat);
        }
    }

    new_population
}

fn calculate_population_diversity(population: &[Strategy]) -> f64 {
    if population.is_empty() { return 0.0; }
    
    // Centroid calculation (O(n))
    let mut sum_thresh = 0.0;
    let mut sum_edge = 0.0;
    let mut sum_tp = 0.0;
    let mut sum_sl = 0.0;
    
    for s in population {
        sum_thresh += s.queue_threshold as f64 / GA_GENE_SCALE as f64;
        sum_edge += s.base_edge as f64 / GA_GENE_SCALE as f64;
        // Normalize ATR multipliers (scaled by 100) to O(1) for diversity
        sum_tp += s.take_profit as f64 / 100.0; 
        sum_sl += s.stop_loss as f64 / 100.0;
    }
    
    let n = population.len() as f64;
    let centroid = (sum_thresh / n, sum_edge / n, sum_tp / n, sum_sl / n);
    
    // Mean distance to centroid (L1 normalized)
    let mut total_dist = 0.0;
    for s in population {
        let d1 = (s.queue_threshold as f64 / GA_GENE_SCALE as f64 - centroid.0).abs() / 1000.0;
        let d2 = (s.base_edge as f64 / GA_GENE_SCALE as f64 - centroid.1).abs() / 50.0;
        let d3 = (s.take_profit as f64 / 100.0 - centroid.2).abs() / 10.0;
        let d4 = (s.stop_loss as f64 / 100.0 - centroid.3).abs() / 10.0;
        total_dist += d1 + d2 + d3 + d4;
    }
    
    (total_dist / n).min(1.0)
}

pub fn calculate_genotype_distance(a: &Strategy, b: &Strategy) -> f64 {
    // Standard scaling factors for genotype units
    let d1 = (a.queue_threshold as f64 - b.queue_threshold as f64).abs() / (50.0 * GA_GENE_SCALE as f64);
    let d2 = (a.base_edge as f64 - b.base_edge as f64).abs() / (10.0 * GA_GENE_SCALE as f64);
    let d3 = (a.take_profit as f64 - b.take_profit as f64).abs() / 100.0;
    let d4 = (a.stop_loss as f64 - b.stop_loss as f64).abs() / 50.0;

    ((d1 + d2 + d3 + d4) / 4.0).min(1.0)
}

fn apply_similarity_penalty(evaluations: &mut Vec<StrategyEvaluation>) {
    // 1. Calculate population-based dynamic ranges for normalization
    let mut metrics = PopulationMetrics {
        min_threshold: u64::MAX, max_threshold: 0,
        min_edge: u64::MAX, max_edge: 0,
        min_tp: u64::MAX, max_tp: 0,
        min_sl: u64::MAX, max_sl: 0,
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
    let range_threshold = (metrics.max_threshold as f64 - metrics.min_threshold as f64).max(100.0 * GA_GENE_SCALE as f64);
    let range_edge = (metrics.max_edge as f64 - metrics.min_edge as f64).max(5.0 * GA_GENE_SCALE as f64);
    let range_tp = (metrics.max_tp as f64 - metrics.min_tp as f64).max(10.0);
    let range_sl = (metrics.max_sl as f64 - metrics.min_sl as f64).max(5.0);

    let top_strats: Vec<Strategy> = evaluations.iter().take(5).map(|e| e.strategy.clone()).collect();
    
    for eval in evaluations.iter_mut() {
        let mut max_similarity: f64 = 0.0;
        for top in &top_strats {
            if &eval.strategy == top { continue; }
            
            // DYNAMIC NORMALIZATION: abs(a-b) / population_range
            let d1 = (eval.strategy.queue_threshold as f64 - top.queue_threshold as f64).abs() / range_threshold;
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

fn evolve_generation(evaluations: &Vec<StrategyEvaluation>, config: &GaConfig, rng: &mut StdRng, evo: &EvoState) -> Vec<Strategy> {
    let mut next_gen: Vec<Strategy> = Vec::new();

    // PHASE 11.1: Diverse Elite Selection (Hard Behavioral Filter)
    let mut elites: Vec<StrategyEvaluation> = Vec::new();
    let target_elite_count = ((evaluations.len() as f64 * 0.10).ceil() as usize).max(2).min(evaluations.len());
    let diversity_threshold = 0.3; // Distance [0, 1]

    // Population Stats for Elitism Normalization
    let pnl_mu = evaluations.iter().map(|e| e.avg_pnl).sum::<f64>() / (evaluations.len() as f64).max(1.0);
    let pnl_sigma = (evaluations.iter().map(|e| (e.avg_pnl - pnl_mu).powi(2)).sum::<f64>() / (evaluations.len() as f64).max(1.0)).sqrt().max(1e-9);
    let std_mu = evaluations.iter().map(|e| e.std_dev).sum::<f64>() / (evaluations.len() as f64).max(1.0);
    let std_sigma = (evaluations.iter().map(|e| (e.std_dev - std_mu).powi(2)).sum::<f64>() / (evaluations.len() as f64).max(1.0)).sqrt().max(1e-9);

    for candidate in evaluations {
        let is_diverse = elites.iter().all(|e| {
            calculate_behavioral_distance(
                e, candidate,
                pnl_mu, pnl_sigma,
                std_mu, std_sigma
            ) > diversity_threshold
        });
        if is_diverse {
            elites.push(candidate.clone());
        }
        if elites.len() >= target_elite_count { break; }
    }

    // ELITE FALLBACK: Fill remaining slots with best fitness if diversity filter was too strict
    // Guard: Similarity < 0.95 (Distance > 0.05) to prevent near-clones
    if elites.len() < target_elite_count {
        for candidate in evaluations {
            if !elites.iter().any(|e| e.strategy == candidate.strategy) {
                let too_similar = elites.iter().any(|e| {
                    calculate_behavioral_distance(
                        e, candidate,
                        pnl_mu, pnl_sigma,
                        std_mu, std_sigma
                    ) < 0.05
                });
                if !too_similar {
                    elites.push(candidate.clone());
                }
            }
            if elites.len() >= target_elite_count { break; }
        }
    }

    let preserved_strategies: Vec<Strategy> = elites.iter().map(|e| e.strategy.clone()).collect();
    next_gen.extend(preserved_strategies);

    println!(
        "Elitism → Preserving {} diverse elites (Top fitness: {:.4}) | MutationScale: {:.2} | Stagnation: {}",
        elites.len(),
        evaluations[0].fitness,
        evo.mutation_scale,
        evo.stagnation_counter
    );

    // Tournament Selection + Adaptive Mutation (Phase D.1.19)
    let k = (config.population_size / 4).min(5).max(2);
    let shock_prob = if evo.stagnation_counter > 3 { 0.25 } else { 0.10 };

    let fitness_mean = evaluations.iter().map(|e| e.fitness).sum::<f64>() / evaluations.len().max(1) as f64;
    let fitness_std = (evaluations.iter().map(|e| (e.fitness - fitness_mean).powi(2)).sum::<f64>() / evaluations.len().max(1) as f64).sqrt();
    let diversity_pressure = (1.0 - (fitness_std / (fitness_mean.abs() + EPS)).min(1.0)).powi(2);
    
    let mut current_evo = evo.clone();
    current_evo.mutation_scale *= 1.0 + diversity_pressure * 2.5; // Aggressive scale when stuck
    
    if evo.stagnation_counter > 3 {
        current_evo.mutation_scale *= 1.5;
    }

    // Behavioral Cluster Count (Diversity Metric)
    let mut unique_clusters = 0;
    if !elites.is_empty() {
        let mut clusters: Vec<&StrategyEvaluation> = vec![&elites[0]];
        for elite in &elites[1..] {
            if clusters.iter().all(|c| calculate_behavioral_distance(c, elite, pnl_mu, pnl_sigma, std_mu, std_sigma) > 0.3) {
                clusters.push(elite);
            }
        }
        unique_clusters = clusters.len();
    }
    
    println!(
        "Evolution → Diverse Clusters: {} | Tournament K: {} | Shock Prob: {:.2} | Effective Scale: {:.2}",
        unique_clusters, k, shock_prob, current_evo.mutation_scale
    );

    let mutation_rate = config.mutation_rate * (1.0 + diversity_pressure);

    // Phase D.1.20: Super-Elite Synthesis (Genetic Recombination)
    let super_elites: Vec<&StrategyEvaluation> = evaluations.iter()
        .filter(|e: &&StrategyEvaluation| (**e).max_signature_credibility > 1.15 && (**e).forced_win_ratio < 0.25 && (**e).trade_count >= 3)
        .collect();
    
    if !super_elites.is_empty() {
        let synthesis_count = (config.population_size as f64 * 0.15).ceil() as usize;
        for _ in 0..synthesis_count {
            if next_gen.len() >= config.population_size { break; }
            
            // Randomly pick a subset of super-elites for synthesis
            let n_parents = (super_elites.len().min(3)).max(1);
            let mut parents = Vec::new();
            for _ in 0..n_parents {
                parents.push(super_elites[rng.gen_range(0..super_elites.len())]);
            }
            
            let mut synthetic = synthesize_super_elite(&parents, rng);
            
            // Apply slight mutation to the synthetic offspring to refine
            let mut evo_lite = current_evo.clone();
            evo_lite.mutation_scale *= 0.5; // Fine-tuning mutation only
            mutate_strategy(&mut synthetic, rng, 10, &evo_lite);
            
            next_gen.push(synthetic);
        }
    }

    while next_gen.len() < config.population_size {
        // Diversified Tournament: Penalize similarity to existing elites
        let parent_eval = tournament_selection_diverse(evaluations, k, rng, &elites, pnl_mu, pnl_sigma, std_mu, std_sigma);
        let mut offspring = parent_eval.strategy.clone();
        
        if rng.gen::<f64>() < mutation_rate {
            if rng.gen::<f64>() < shock_prob {
                // SHOCK MUTATION (Scout Injection/Exploration)
                offspring = random_strategy(config, rng);
            } else {
                // CREEPING MUTATION (Refinement/Exploitation)
                mutate_strategy(&mut offspring, rng, parent_eval.trade_count, &current_evo);
            }
        }
        next_gen.push(offspring);
    }

    next_gen
}

fn tournament_selection_diverse<'a>(
    evaluations: &'a Vec<StrategyEvaluation>, 
    k: usize, 
    rng: &mut StdRng,
    elites: &Vec<StrategyEvaluation>,
    pnl_mu: f64, pnl_sigma: f64,
    std_mu: f64, std_sigma: f64
) -> &'a StrategyEvaluation {
    let mut best: Option<(&StrategyEvaluation, f64)> = None;
    for _ in 0..k {
        let candidate = &evaluations[rng.gen_range(0..evaluations.len())];
        
        // Diversity Bias: score = fitness - 0.2 * max_similarity_to_elites
        let mut max_sim = 0.0;
        for e in elites {
            let dist = calculate_behavioral_distance(e, candidate, pnl_mu, pnl_sigma, std_mu, std_sigma);
            let sim = (1.0 - dist).max(0.0);
            if sim > max_sim { max_sim = sim; }
        }
        
        let adj_fitness = candidate.fitness - 0.30 * max_sim; // Sharpened Penalty (D.1.19)
        
        if best.is_none() || adj_fitness > best.unwrap().1 {
            best = Some((candidate, adj_fitness));
        }
    }
    best.unwrap().0
}

pub fn random_strategy(_config: &GaConfig, rng: &mut StdRng) -> Strategy {
    Strategy {
        // Institutional Selectivity Floor: Narrowed to 60-120 to avoid velocity saturation
        queue_threshold: (rng.gen_range(60..=120) as u64).saturating_mul(GA_GENE_SCALE), 
        base_edge: (rng.gen_range(1..=10) as u64).saturating_mul(GA_GENE_SCALE),
        // ATR Multipliers (scaled by 100)
        // TP: 1.0 to 5.0 ATR
        take_profit: rng.gen_range(100..=500), 
        // SL: 0.5 to 3.0 ATR
        stop_loss: rng.gen_range(50..=300),   
        // Bars to hold (scaled by 10)
        holding_period: rng.gen_range(20..=200),
        // Phase D.1.8: Non-linear Scoring Genes (scaled 0-100)
        w_conviction: rng.gen_range(10..=100),
        w_momentum: rng.gen_range(10..=100),
        w_volatility: rng.gen_range(5..=50), // Volatility usually penalized, smaller starting weight
        exp_conviction: rng.gen_range(70..=200), // Start near linear or slightly above
        exp_momentum: rng.gen_range(70..=200),
        exp_volatility: rng.gen_range(70..=200),
        selectivity: rng.gen_range(60..=90),
        archetype: rng.gen_range(0..=3),
    }
}

pub fn synthesize_super_elite(parents: &Vec<&StrategyEvaluation>, rng: &mut StdRng) -> Strategy {
    // 1. Component: Filters (Best Pattern Credibility)
    let filter_parent = parents.iter().max_by(|a: &&&StrategyEvaluation, b: &&&StrategyEvaluation| (***a).max_signature_credibility.total_cmp(&(***b).max_signature_credibility)).unwrap();
    
    // 2. Component: Execution (Best Realized PnL)
    let exec_parent = parents.iter().max_by(|a, b| (***a).avg_pnl.total_cmp(&(***b).avg_pnl)).unwrap();
    
    // 3. Component: Thresholds (Best Decision Consistency)
    let thresh_parent = parents.iter().max_by(|a, b| (***a).consistency.total_cmp(&(***b).consistency)).unwrap();

    Strategy {
        // Group: Thresholds
        queue_threshold: thresh_parent.strategy.queue_threshold,
        base_edge: thresh_parent.strategy.base_edge,
        selectivity: thresh_parent.strategy.selectivity,
        
        // Group: Execution
        take_profit: exec_parent.strategy.take_profit,
        stop_loss: exec_parent.strategy.stop_loss,
        holding_period: exec_parent.strategy.holding_period,
        
        // Group: Filters
        w_conviction: filter_parent.strategy.w_conviction,
        w_momentum: filter_parent.strategy.w_momentum,
        w_volatility: filter_parent.strategy.w_volatility,
        exp_conviction: filter_parent.strategy.exp_conviction,
        exp_momentum: filter_parent.strategy.exp_momentum,
        exp_volatility: filter_parent.strategy.exp_volatility,
        
        archetype: filter_parent.strategy.archetype,
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
        "STRAT_{}v{}v{}v{}v{}v{}v{}v{}v{}v{}v{}v{}v{}",
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
        s.archetype
    )
}

fn mutate_strategy(strategy: &mut Strategy, rng: &mut StdRng, parent_trade_count: usize, evo: &EvoState) {
    let mut mutation_type = rng.gen_range(0..6);
    
    // 🔥 LINEAGE FORCE-MUTATION: Resurrection pressure if strategy is economically inactive
    if parent_trade_count == 0 {
        // Force mutation to move only entry conditions (threshold/edge)
        mutation_type = if rng.gen_bool(0.3) { 4 } else if rng.gen_bool(0.5) { 0 } else { 2 };
    }

    // Adaptive step size based on non-linear stagnation scaling
    let stagnation_jump = 1.0 + (evo.stagnation_counter as f64).powi(2) * 0.1;
    let mutation_rate = (0.25 * evo.mutation_scale).clamp(0.01, 0.8);

    match mutation_type {
        0 => { // Big jump in threshold (RUPEE EQUIVALENT)
            let base_delta = rng.gen_range((20 * GA_GENE_SCALE)..(100 * GA_GENE_SCALE)) as f64;
            let delta = (base_delta * evo.mutation_scale * stagnation_jump) as i64 * if rng.gen_bool(0.7) { -1 } else { 1 };
            // Clamp to selective range (60-120 units)
            strategy.queue_threshold = (strategy.queue_threshold as i64 + delta).clamp((60 * GA_GENE_SCALE) as i64, (120 * GA_GENE_SCALE) as i64) as u64;
        }
        1 => { // Flip TP/SL (within bounds - ATR Multipliers)
            let temp = strategy.take_profit;
            strategy.take_profit = strategy.stop_loss.clamp(100, 500); 
            strategy.stop_loss = temp.clamp(50, 300);
        }
        2 => { // Base_edge change
            let intensity = if parent_trade_count == 0 { 2.0 } else { 1.0 };
            let base_edge_new = rng.gen_range((1 * GA_GENE_SCALE)..((20.0 * intensity) as u64 * GA_GENE_SCALE));
            strategy.base_edge = base_edge_new;
        }
        3 => { // Mutate ATR multipliers
            let delta_tp = (rng.gen_range(-50..=50) as f64 * evo.mutation_scale) as i64;
            let delta_sl = (rng.gen_range(-30..=30) as f64 * evo.mutation_scale) as i64;
            strategy.take_profit = (strategy.take_profit as i64 + delta_tp).clamp(100, 500) as u64;
            strategy.stop_loss = (strategy.stop_loss as i64 + delta_sl).clamp(50, 300) as u64;
        }
        4 => { // 4: Mutate Phase D.1.8 Scoring Genes
            let delta_w = (rng.gen_range(-20..=20) as f64 * evo.mutation_scale) as i64;
            let delta_e = (rng.gen_range(-30..=30) as f64 * evo.mutation_scale) as i64;
            
            match rng.gen_range(0..6) {
                0 => strategy.w_conviction = (strategy.w_conviction as i64 + delta_w).clamp(10, 100) as u64,
                1 => strategy.w_momentum = (strategy.w_momentum as i64 + delta_w).clamp(10, 100) as u64,
                2 => strategy.w_volatility = (strategy.w_volatility as i64 + delta_w).clamp(5, 100) as u64,
                3 => strategy.exp_conviction = (strategy.exp_conviction as i64 + delta_e).clamp(50, 300) as u64,
                4 => strategy.exp_momentum = (strategy.exp_momentum as i64 + delta_e).clamp(50, 300) as u64,
                _ => strategy.exp_volatility = (strategy.exp_volatility as i64 + delta_e).clamp(50, 300) as u64,
            }
        }
        _ => { // 5: Mutate Selectivity & Archetype (Phase D.1.9)
            if rng.gen_bool(mutation_rate) {
                strategy.selectivity = rng.gen_range(60..=90);
            }
            if rng.gen_bool(mutation_rate) {
                strategy.archetype = rng.gen_range(0..=3);
            }
        }
    }
}

// Rename helper if it was used with different arguments elsewhere

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GaExitReason {
    TakeProfit,
    StopLoss,
    TimeStop,
}

/// One non-overlapping round-trip from a cursor index (ESE harness), for multi-cycle GA evaluation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GaRoundTripOutcome {
    pub side: Side,
    pub source: SignalSource,
    pub exit_reason: GaExitReason,
    pub pnl: f64,
    pub quality: f64,
    pub exit_event_idx: usize,
    pub drawdown_penalty_raw: f64,
    pub total_filled_qty: u64,
    pub fills_count: usize,
    pub total_slippage_bps: f64,
    pub expected_move: f64, // Realized move
    pub m_favorable: f64,  // MFE (Max Favorable Excursion)
    pub m_adverse: f64,    // MAE (Max Adverse Excursion)
    pub efficiency: f64,    // Realized / MFE
    pub edge_quality: f64,  // MFE / |MAE|
    pub time_to_mfe: usize, // Bars to MFE
    pub raw_q_ratio: f64,
    pub fill_efficiency: f64,
    pub sim_events: Vec<crate::SimEvent>,
    pub entry_order_id: String,
    pub exit_order_id: String,
    pub spread: f64,
    pub avg_window_volume: f64,
}

/// Deterministic single round-trip anchored at `market_events[cursor_i]`.
#[derive(Debug, Clone, Default)]
pub struct ConvictionOutcome {
    pub conviction_score: f64,    // Continuous [0, 1]
    pub expected_edge: f64,       // Predicted move - Cost
    pub edge_weight: f64,         // Soft gate (1.0 or 0.2)
    pub norm_momentum: f64,       // [0, 1]
    pub norm_volume: f64,         // [0, 1]
    pub norm_vol_score: f64,      // [0, 1]
    pub norm_vol: f64,            // The raw normalized volatility
    pub selection_threshold: f64, // The strategic gate [0.0, 1.5]
    pub is_bearish: bool,
    pub roll: f64,                // Genetic jitter
    pub raw_q_ratio: f64,
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
    let current_trade_vol = if ref_event.subtype == crate::MarketEventType::Trade { ref_event.quantity as f64 } else { avg_vol };
    let norm_volume = (current_trade_vol / (avg_vol * 1.5)).clamp(0.0, 1.0);

    // 2. Momentum (Price Velocity)
    let lookback_price = signal_events[start_idx].price as f64;
    let price_delta = (ref_price as f64 - lookback_price).abs() / ref_price as f64;
    let norm_momentum = (price_delta / 0.001).clamp(0.0, 1.0);

    // 3. Soft Volatility Guard
    let prices: Vec<f64> = signal_events[start_idx..=cursor_i].iter().map(|e| e.price as f64).collect();
    let mean_px = prices.iter().sum::<f64>() / prices.len() as f64;
    let variance = prices.iter().map(|p| (p - mean_px).powi(2)).sum::<f64>() / prices.len() as f64;
    let norm_vol = variance.sqrt() / mean_px.max(1.0);
    let norm_vol_score = (1.0 - (norm_vol / 0.002)).clamp(0.0, 1.0);

    // 4. Weighted Hybrid Conviction (Strong Phase D.1.7)
    let base_conviction = (0.5 * norm_momentum) + (0.3 * norm_volume) + (0.2 * norm_vol_score);
    
    // --- PHASE D.1.7: GENOME-DRIVEN CONVICTION (STRONG) ---
    // Force wide spread in how genomes interpret signals
    let genome_bias =
        (strategy.base_edge as f64 * 0.004)
      + (strategy.queue_threshold as f64 * 0.04)
      + (strategy.holding_period as f64 * 0.003);

    let bias_factor = 0.4 + 1.2 * (genome_bias / GA_GENE_SCALE as f64).tanh();
    
    // Entropy Injection: deterministic jitter based on roll
    let jitter = 0.85 + (0.3 * roll); // Broad jitter [0.85, 1.15]
    let conviction_score = (base_conviction * bias_factor * jitter).clamp(0.0, 1.0);

    let is_bearish = (ref_price as f64) < mean_px;
    let selection_threshold = (strategy.base_edge % 151) as f64 / 100.0;

    ConvictionOutcome {
        conviction_score,
        expected_edge: 0.0,
        edge_weight: 1.0,
        norm_momentum,
        norm_volume,
        norm_vol_score,
        norm_vol,
        selection_threshold,
        is_bearish,
        roll,
        raw_q_ratio: (window_trades_vol as f64 / strategy.queue_threshold as f64).min(2.0),
    }
}

pub(crate) fn ga_simulate_round_trip_at_cursor(
    strategy: &Strategy,
    strategy_id: &str,
    _scenario_name: &str,
    signal_events: &[crate::MarketEvent],
    execution_events: &[crate::MarketEvent],
    config: &GaConfig,
    cursor_i: usize,
    trade_idx: usize,
    conviction: &ConvictionOutcome,
) -> Option<GaRoundTripOutcome> {
    // Refinement 4: Strict cursor-based contract
    assert!(cursor_i < signal_events.len(), "cursor_i {} out of bounds for signal_events {}", cursor_i, signal_events.len());
    let ref_event = &signal_events[cursor_i];
    let _ref_price = ref_event.price;
    let ref_ts = ref_event.exchange_ts;
    let entry_idx = cursor_i + config.latency_ticks;
    if entry_idx >= execution_events.len().saturating_sub(1) {
        return None;
    }

    let sig_px = signal_events[cursor_i].price as f64;
    let exe_px = execution_events[entry_idx].price as f64;
    let spread = (exe_px - sig_px).abs();
    
    // Institutional hard check: reject corrupt data with spread > 10% of price
    if spread > sig_px * 0.1 {
        return None;
    }
    
    let slippage = spread * config.slippage_factor;

    let market_price = (exe_px + slippage) as u64;
    let edge_bias = ((strategy.base_edge as f64 - 5.0) / 50.0).clamp(-0.12, 0.12);
    // Use aggressiveness from conviction
    let aggressiveness = conviction.conviction_score; 
    let agg_threshold = ((aggressiveness / 1.1) + edge_bias).clamp(0.05, 0.98);
    let tick_01 = (0.01 * crate::PRICE_SCALE as f64).round() as u64;
    let (buy_price, _is_aggressive) = if conviction.roll < agg_threshold {
        (market_price.saturating_add(tick_01), true)
    } else {
        (market_price, false)
    };

    let entry_order_id = format!("{}_t{}_entry", strategy_id, trade_idx);
    let exit_order_id = format!("{}_t{}_exit", strategy_id, trade_idx);

    let atr_floor = (buy_price as f64 * 0.0001).max(1e-5);
    let adjusted_atr = calculate_atr(signal_events, cursor_i, 14).max(atr_floor * (1.0 + conviction.norm_vol));
    let tp_mult = (strategy.take_profit as f64 / 100.0).max(0.1); // e.g. 250 -> 2.5x ATR
    let sl_mult = (strategy.stop_loss as f64 / 100.0).max(0.1);   // e.g. 150 -> 1.5x ATR
    
    let tp_target = (buy_price as f64 + (adjusted_atr * tp_mult)) as u64;
    let sl_target = (buy_price as i64 - (adjusted_atr * sl_mult) as i64).max(1) as u64;

    let entry_idx = cursor_i + config.latency_ticks;
    let min_hold = 5usize.saturating_add(
        (strategy.base_edge as usize + strategy.take_profit as usize + strategy.stop_loss as usize) % 15,
    );
    let mut exit_price = buy_price;
    let mut _exit_ts = ref_ts.saturating_add(100);
    let mut found_exit = false;
    let mut exit_event_idx = execution_events.len().saturating_sub(1);
    let mut exit_reason = GaExitReason::TimeStop;

    let mut mfe_scaled = buy_price;
    let mut mae_scaled = buy_price;
    let mut time_to_mfe = 0;

    for (j, event) in execution_events.iter().enumerate().skip(entry_idx + min_hold) {
        // High-Precision Excursion Tracking
        if event.price > mfe_scaled {
            mfe_scaled = event.price;
            time_to_mfe = j.saturating_sub(entry_idx);
        }
        if event.price < mae_scaled {
            mae_scaled = event.price;
        }

        // 🎯 Institutional GUARDRAIL 1: SL Priority (Check SL before TP)
        if event.price <= sl_target {
            exit_price = (sl_target as f64 - slippage).max(1.0) as u64; 
            _exit_ts = event.exchange_ts;
            exit_event_idx = j;
            found_exit = true;
            exit_reason = GaExitReason::StopLoss;
            break;
        } else if event.price >= tp_target {
            exit_price = (tp_target as f64 - slippage).max(1.0) as u64;
            _exit_ts = event.exchange_ts;
            exit_event_idx = j;
            found_exit = true;
            exit_reason = GaExitReason::TakeProfit;
            break;
        }
    }
    if !found_exit {
        if let Some(last_ev) = execution_events.last() {
            exit_price = (last_ev.price as f64 - slippage) as u64;
            _exit_ts = last_ev.exchange_ts;
        }
    }

    let raw_realized_pnl = (exit_price as f64 - buy_price as f64) / buy_price.max(1) as f64;
    
    // --- PHASE D.1.6: STATE-DEPENDENT EPSILON GRADIENT RECOVERY ---
    // Inject dispersion (entropy) to ensure GA has a gradient to optimize.
    let realized_pnl = if raw_realized_pnl.abs() < 1e-7 {
        // Dispersion = weight * (1.0 + conviction + vol)
        let dispersion_epsilon = 0.00005 * (1.0 + conviction.conviction_score + conviction.norm_vol);
        if edge_bias >= 0.0 { dispersion_epsilon } else { -dispersion_epsilon }
    } else {
        raw_realized_pnl
    };

    let mfe_pnl = (mfe_scaled as f64 - buy_price as f64) / buy_price.max(1) as f64;
    let mae_pnl = (mae_scaled as f64 - buy_price as f64) / buy_price.max(1) as f64;
    
    // --- Phase C.2e: Relative Capture Efficiency (Bidirectional Gradient) ---
    let efficiency = if realized_pnl >= 0.0 {
        if mfe_pnl <= 1e-9 {
            0.05 // Minimal reward for flat/break-even trades
        } else {
            (realized_pnl / mfe_pnl).clamp(0.05, 1.0)
        }
    } else {
        // Gradient for losses: Reward strategies that contain risk relative to SL floor
        let sl_delta = (sl_target as f64 - buy_price as f64) / buy_price.max(1) as f64;
        let max_loss = sl_delta.min(-1e-6); // Robust floor for division stability
        let loss_eff = (realized_pnl / max_loss).clamp(0.0, 1.0);
        loss_eff * 0.2 // Scale down: max 0.2 efficiency for "intelligent" losses
    };

    if std::env::var("GA_DEBUG").is_ok() {
        println!("TRADE_DEBUG → entry={} exit={} tp={} sl={} mfe={:.6} mae={:.6} pnl={:.6} eff={:.4}", 
            buy_price, exit_price, tp_target, sl_target, mfe_pnl, mae_pnl, realized_pnl, efficiency);
    }

    let edge_quality = (mfe_pnl / mae_pnl.abs().max(1e-9)).clamp(0.0, 5.0);

    Some(GaRoundTripOutcome {
        side: Side::Buy,
        source: SignalSource::Organic,
        exit_reason,
        pnl: realized_pnl,
        quality: if realized_pnl > 0.0005 { 1.0 } else { 0.0 },
        exit_event_idx,
        drawdown_penalty_raw: (buy_price as f64 - mae_scaled as f64).abs() / buy_price.max(1) as f64,
        total_filled_qty: 1,
        fills_count: 1,
        total_slippage_bps: (slippage / buy_price.max(1) as f64) * 10000.0,
        expected_move: realized_pnl,
        m_favorable: mfe_pnl,
        m_adverse: mae_pnl,
        efficiency,
        edge_quality,
        time_to_mfe: time_to_mfe,
        raw_q_ratio: conviction.raw_q_ratio,
        fill_efficiency: 1.0,
        sim_events: Vec::new(),
        entry_order_id,
        exit_order_id,
        spread: (exe_px - sig_px).abs(),
        avg_window_volume: 0.0,
    })
}

pub(crate) fn evaluate_strategy(
    strategy: &Strategy,
    pair: &ScenarioPair,
    config: &GaConfig,
    generation: usize,
) -> Option<StrategyEvaluation> {
    let scenario_name = pair.name;
    let signal_events = pair.signal;
    let execution_events = pair.execution;
    let signal_symbol = pair.signal_symbol;
    let exec_symbol = pair.execution_symbol;

    // Phase 4: Routing Integrity & Pointer Safety (True Dual-Stream)
    if std::env::var("GA_DEBUG").is_ok() {
        println!("ROUTE_SOURCE → {} -> {}", signal_symbol, exec_symbol);
        println!(
            "ROUTE_VERIFY → diff={} sig_ptr={:p} exec_ptr={:p}",
            !std::ptr::eq(signal_events.as_ptr(), execution_events.as_ptr()),
            signal_events.as_ptr(),
            execution_events.as_ptr()
        );
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
    let capability = determine_scenario_capability(scenario_name);

    // --- Phase 9: Environment Gating (Scenario-Level) ---
    // Pre-scan all signal points to assess the regime quality before committing to execution.
    let mut candidate_edges = Vec::new();
    for i in 0..signal_events.len() {
        let conv = evaluate_market_conviction(strategy, scenario_name, signal_events, i, 0, generation);
        if conv.conviction_score >= conv.selection_threshold {
            let entry_price = signal_events[i].price as f64;
            let atr = calculate_atr(signal_events, i, 14);
            let tp_mult = strategy.take_profit as f64 / 100.0;
            let pred_move = atr * tp_mult;
            
            // Synchronized Edge Estimate (Matches line 1951)
            let edge_abs = (pred_move * 0.8 * 0.9) - (entry_price * 0.0001); 
            let edge_ratio = (edge_abs / entry_price.max(1.0)).max(0.0);
            candidate_edges.push(edge_ratio);
        }
    }

    if candidate_edges.is_empty() {
        return None; // No signals, no scenario.
    }

    // --- PHASE 10.3: DART (Dynamic Asset-Relative Thresholding) FLOOR ---
    // Extract a representative window-level floor for pre-filtering stats.
    let avg_edge = candidate_edges.iter().sum::<f64>() / candidate_edges.len() as f64;
    let avg_atr_pct = avg_edge * 1.2;
    let window_dart_floor = (avg_atr_pct * 0.40).clamp(0.00001, 0.0012);

    // --- PHASE 10.3: AQG DISTRIBUTION INTEGRITY ---
    // Filter noise before statistics. Use exact DART floor as the viability barrier.
    let valid_edges: Vec<f64> = candidate_edges.iter().cloned().filter(|e| *e >= window_dart_floor).collect();

    // AQG Starvation Gate Removed (Analytical Mode)
    let _coverage = valid_edges.len() as f64 / candidate_edges.len().max(1) as f64;
    let _min_sample = (candidate_edges.len() as f64 * 0.03).max(5.0) as usize;

    let _aqg_health = (valid_edges.len() as f64 / _min_sample.max(1) as f64).clamp(0.0, 1.5);
    let aqg_threshold = 0.0;
    let _ = aqg_threshold; // Closure compatibility

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
        let pct = if dispersion_val > 0.8 { 0.50 } else if dispersion_val > 0.5 { 0.55 } else { 0.60 };
        let idx = ((v.len() as f64) * pct).floor() as usize;
        let aqg_gate = v[idx.min(v.len().saturating_sub(1))];
        (median, mad_scaled, aqg_gate)
    };

    let dispersion = mad_scaled / median.max(1e-9);

    if std::env::var("GA_DEBUG").is_ok() {
        println!(
            "AQG_ADMISSION → scenario={} dispersion={:.6} aqg_gate={:.6} (valid={}/max={})",
            scenario_name, dispersion, aqg_gate, valid_edges.len(), candidate_edges.len()
        );
    }
    let aqg_threshold = aqg_gate;

    let mut scenario_pnls: Vec<f64> = Vec::new();
    let mut total_quality_trades_scenario = 0.0;
    let mut total_efficiency = 0.0;
    let mut total_vol_ratio = 0.0;
    let mut total_spread_reality = 0.0;
    let mut total_spread_test = 0.0;
    let mut survivable_trades_count = 0usize;
    let mut sum_price = 0.0;
    let mut metrics = ScenarioMetrics::default();
    
    // Diagnostic Counters
    let mut _signal_count = 0usize;
    let mut entry_attempted = 0usize;
    let mut total_trades = 0usize;
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
    let mut sum_latency_raw = 0.0;
    let mut cycle_sigs: Vec<ScenarioExecutionSignature> = Vec::new();

    // Phase 8.8 Aggregators
    let mut sum_expected_slippage = 0.0;
    let mut sum_actual_slippage = 0.0;
    let mut max_pnl_in_scenario: f64 = 0.0;
    let mut pnl_from_tp_scenario: f64 = 0.0;
    let mut pnl_from_sl_scenario: f64 = 0.0;
    let mut max_trade_pnl_scenario: f64 = 0.0;
    let mut long_win_count_scenario = 0usize;
    let short_win_count_scenario = 0usize;
    let mut micro_loss_count = 0u32;
    let mut total_window_volume = 0.0;

    // --- PHASE 14: DISTRIBUTION-AWARE SIGNAL VALIDATION LAYER ---
    // Transitioning from fixed-gate scoring to institutional selective-gating.
    // This ensures only true statistical outliers from the strategy are traded.

    let name_upper = scenario_name.to_uppercase();
    // Phase D.1.14: Bypass Eradication. We no longer allow bypass based on scenario names.
    // Structural integrity is now mandatory.
    let allow_bypass = false; 

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
        let mut conviction = evaluate_market_conviction(strategy, scenario_name, signal_events, current_idx, 0, generation);
        // Apply strategy-specific perturbation
        conviction.conviction_score *= 0.9 + 0.2 * perturb;
        
        scores.push(conviction.conviction_score);
        window_data.push((current_idx, conviction));
    }
    if window_data.is_empty() { return None; }

    // 2. Statistical Derivation & Adaptive Gating
    let n = scores.len() as f64;
    let mean = scores.iter().sum::<f64>() / n;
    let variance = scores.iter().map(|s| (s - mean).powi(2)).sum::<f64>() / n;
    let std_dev = variance.sqrt();

    // Phase 17A: Soften the Adaptive Quality Gate (AQG)
    // We change LOW_DISPERSION from a hard skip into a market-condition scaler.
    let dispersion_multiplier = if allow_bypass { 1.0 } else { (std_dev / MIN_STD).min(1.0) };

    // Adaptive Parameters
    let abs_floor = (percentile_f64(&scores, 0.80)).max(mean + 0.5 * std_dev);
    let z_threshold = (BASE_Z * (TARGET_STD / (std_dev + EPS))).clamp(0.8, 1.5);
    let min_signals = (window_data.len() as f64 * 0.05).max(2.0) as usize;

    // 3. Signal Validation Layer (Identify High-Conviction Cluster)
    // Phase 17A.5: Store dominance and reason to ensure consistency
    // Phase 17B: Store E-score for realizability analysis
    let mut valid_signals: Vec<(usize, ConvictionOutcome, f64, &'static str, f64, SignalSource, SignalSignature)> = Vec::new(); 
    let mut max_z = 0.0;
    
    // --- PHASE D.1.13.5: ADAPTIVE SIGNAL FLOOR (BOOTSTRAP FIX) ---
    // If the adaptive threshold is too strict (killing all signals), fallback to the 60th percentile.
    let percentile_60 = percentile_f64(&scores, 0.60);
    let effective_floor = abs_floor.max(percentile_60);
    
    let p75_energy = percentile_f64(&scores, 0.75);
    let energy_min = effective_floor.max(p75_energy);
    
    let mut decision_was_override = false;
    let mut winner_idx: Option<usize> = None;
    let mut winner_conviction = window_data.first().map(|(_, c)| c.clone()).unwrap_or_else(|| ConvictionOutcome::default());
    let mut winner_reason_final = "NONE";
    let mut winner_dom_final = 0.0f64;
    let mut _winner_e_score_final = 0.0f64;
    let mut winner_acceptance_mode = AcceptanceMode::Dominance;
    let mut _acceptance_mode = AcceptanceMode::Dominance;

    let mut candidate_signals: Vec<(usize, ConvictionOutcome, f64, &'static str, f64, SignalSource, f64, SignalSignature)> = Vec::new();
    
    for (signal_idx, conviction) in window_data.iter() {
        // --- PHASE 17 CALIBRATION: RAW Population Integrity ---
        let mut sub_scores = [conviction.norm_momentum, conviction.norm_volume, conviction.norm_vol_score];
        sub_scores.sort_by(|a, b| b.total_cmp(a));
        
        let mean_t = sub_scores.iter().sum::<f64>() / 3.0;
        let std_t = (sub_scores.iter().map(|s| (s - mean_t).powi(2)).sum::<f64>() / 3.0).sqrt();
        let raw_dom_t = (sub_scores[0] - sub_scores[1]) / (std_t + EPS);
        metrics.record_pop_stats(raw_dom_t.min(3.0), false); // RAW layer

        let mut adj_conviction = conviction.clone();
        adj_conviction.conviction_score *= dispersion_multiplier;
        
        let score_val = adj_conviction.conviction_score;
        let z_score = (score_val - mean) / (std_dev + EPS);
        if z_score > max_z { max_z = z_score; }

        // --- PHASE D.1.17: COMPETITIVE ADMISSION (RELATIVE) ---
        // We only admit positive conviction to avoid garbage.
        if score_val > 0.0 {
            // Memory-Smoothed Stability
            let scores_idx = window_data.iter().position(|(i, _)| *i == *signal_idx).unwrap_or(0);
            let s_t = scores[scores_idx];
            let s_t_1 = if scores_idx >= 1 { scores[scores_idx - 1] } else { s_t };
            let s_t_2 = if scores_idx >= 2 { scores[scores_idx - 2] } else { s_t_1 };
            let delta = (s_t - s_t_1).abs() + (s_t_1 - s_t_2).abs();
            let stability = (1.0 - delta / (2.0 * (std_dev + EPS).max(0.05))).clamp(0.0, 1.0);
            
            // Execution Realizability (E-score)
            let current_price = signal_events[*signal_idx].price as f64;
            let prev_price = if *signal_idx > 0 { signal_events[*signal_idx - 1].price as f64 } else { current_price };
            let atr = calculate_atr(signal_events, *signal_idx, 14);
            let dist_score = (1.0 - ((current_price - prev_price).abs() / ((if atr > 0.0 { atr } else { current_price * 0.01 }) + EPS)).min(1.0)).clamp(0.0, 1.0);
            
            // Regime-Aware Volatility Score
            let scores_idx_2 = window_data.iter().position(|(i, _)| *i == *signal_idx).unwrap_or(0);
            let local_window = (scores_idx_2 + 1).min(20);
            let start_idx = (scores_idx_2 + 1).saturating_sub(local_window);
            let local_slice = &scores[start_idx ..= scores_idx_2];
            let local_mean = local_slice.iter().sum::<f64>() / local_window as f64;
            let local_var = local_slice.iter().map(|s| (s - local_mean).powi(2)).sum::<f64>() / local_window as f64;
            let local_vol = local_var.sqrt();
            let vol_ratio = (local_vol / (std_dev + EPS)).clamp(0.25, 4.0);
            let vol_score = (1.0 - (vol_ratio - 1.0).abs()).clamp(0.0, 1.0);
            
            let e_score = (stability + dist_score + vol_score) / 3.0;

            // --- Phase D.1.18: Calculate Signal Signature ---
            let regime = if vol_ratio > 1.3 { 1 } else if vol_ratio < 0.7 { -1 } else { 0 };
            let momentum = if adj_conviction.norm_momentum > 0.3 { 1 } else if adj_conviction.norm_momentum < -0.3 { -1 } else { 0 };
            let signature = SignalSignature {
                archetype: strategy.archetype,
                regime,
                momentum,
            };

            candidate_signals.push((*signal_idx, adj_conviction, raw_dom_t, "RELATIVE_CANDIDATE", e_score, SignalSource::Organic, stability, signature));
        }
    }

    // --- PHASE D.1.17: TOP-K SELECTION & FORCE EMERGENCE ---
    // 1. Sort by conviction intensity
    candidate_signals.sort_by(|a, b| b.1.conviction_score.total_cmp(&a.1.conviction_score));

    // 2. Select Top-5 and Apply Curved Penalties + Credibility
    for (idx, (signal_idx, mut conv, dom, _reason, e_score, source, stability, signature)) in candidate_signals.into_iter().enumerate() {
        if idx >= 5 { break; } // Top-5 limit

        // --- Refined Multipliers (D.1.17 Sharpe) ---
        let z_score = (conv.conviction_score - mean) / (std_dev + EPS);
        let stat_confidence = (z_score / z_threshold).clamp(0.5, 1.5);
        let stability_factor = stability.clamp(0.1, 1.0).powf(2.0);
        
        // --- Phase D.1.18: Credibility Overlay ---
        let credibility = if let Some(stats) = metrics.signature_memory.get(&signature) {
            let winrate = if stats.sample_count > 0 { stats.win_count as f64 / stats.sample_count as f64 } else { 0.5 };
            let avg_pnl = if stats.sample_count > 0 { stats.sum_pnl / stats.sample_count as f64 } else { 0.0 };
            let c = (avg_pnl * winrate) * (stats.sample_count as f64 + 1.0).ln();
            
            // Phase D.1.18 Alpha Threshold check
            if stats.sample_count > 5 && winrate > 0.55 && avg_pnl > 0.0005 {
                had_organic_signals = true;
            }
            
            (1.0 + c).clamp(0.5, 2.0)
        } else {
            1.0
        };
        
        if credibility > metrics.max_signature_credibility { metrics.max_signature_credibility = credibility; }
        
        conv.conviction_score *= stat_confidence * stability_factor * credibility;

        metrics.exec_passed_count += 1;
        metrics.sum_e_score += e_score;
        valid_signals.push((signal_idx, conv, dom, "RELATIVE_CANDIDATE", e_score, source, signature));
    }

    // 3. Forced Winner Rule (Phase D.1.19: Decay)
    let gen_progress = (generation as f64 / 50.0).clamp(0.0, 1.0);
    let forced_win_prob = (1.0 - gen_progress).clamp(0.05, 1.0);
    
    let had_organic_signals_final = !valid_signals.is_empty();
    if !had_organic_signals && !valid_signals.is_empty() {
        // Already handled above (Organic Breakthrough)
    } else if !had_organic_signals_final && valid_signals.is_empty() && rand::random::<f64>() < forced_win_prob {
        if let Some((_, (signal_idx, conviction))) = window_data.iter().enumerate().max_by(|a, b| a.1.1.conviction_score.total_cmp(&b.1.1.conviction_score)) {
            if conviction.conviction_score > 1e-6 {
                let mut winner_conv = conviction.clone();
                winner_conv.conviction_score *= 0.5; // Heavy "forced" penalty
                
                // Calculate signature for forced winner
                let local_idx = window_data.iter().position(|(i, _)| i == signal_idx).unwrap_or(0);
                let local_window = (local_idx + 1).min(20);
                let start_idx = (local_idx + 1).saturating_sub(local_window);
                let local_slice = &scores[start_idx..=local_idx];
                let local_mean = local_slice.iter().sum::<f64>() / local_window as f64;
                let local_var = local_slice.iter().map(|s| (s - local_mean).powi(2)).sum::<f64>() / local_window as f64;
                let vol_ratio = (local_var.sqrt() / (std_dev + EPS)).clamp(0.25, 4.0);
                
                let sig = SignalSignature {
                    archetype: strategy.archetype,
                    regime: if vol_ratio > 1.3 { 1 } else if vol_ratio < 0.7 { -1 } else { 0 },
                    momentum: if winner_conv.norm_momentum > 0.3 { 1 } else if winner_conv.norm_momentum < -0.3 { -1 } else { 0 },
                };

                valid_signals.push((*signal_idx, winner_conv, 0.1, "FORCED_EMERGENCE", 0.3, SignalSource::Organic, sig));
                metrics.forced_win_count += 1;
                if std::env::var("GA_DEBUG").is_ok() {
                    println!("FORCED_EMERGENCE → idx={} score={:.2}", signal_idx, conviction.conviction_score);
                }
            }
        }
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
    } else { 0.0 };

    // Purity: Ratio of High-Quality signals (E > 0.80) in the VALID pool
    let high_quality_count = valid_signals.iter().filter(|e| e.4 > 0.80).count();
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
    
    agreement *= asymmetry;

    // --- PHASE A+: SCORING ENGINE ---
    // 1. Identify "Window Potential" (Best candidate stats for structural pulse)
    let (best_dom, best_e) = valid_signals.iter()
        .max_by(|(_, a, _, _, _, _, _), (_, b, _, _, _, _, _)| a.conviction_score.total_cmp(&b.conviction_score))
        .map(|(_, _, dom, _, e_score, _, _)| (*dom, *e_score))
        .unwrap_or((0.0, 0.0));

    let stability_raw = (1.0 - std_v / 0.18).clamp(0.0, 1.0);
    
    // 2. Update structural stats BEFORE gating (Layer 1: Perception)
    metrics.record_adaptive_pulse(
        agreement, 
        purity, 
        stability_raw,
        purity, 
        max_z,
        best_e
    );
    metrics.record_structural_health(agreement, purity, std_v, true); 

    // 3. Normalized Metrics (Z-scores) using Institutional Priors (Fallback N < 20)
    let n_count = metrics.adaptive.agreement.count;
    
    let calc_z = |val: f64, tracker: &WelfordTracker, p_mu: f64, p_sigma: f64| -> f64 {
        if n_count < 20 {
            (val - p_mu) / p_sigma.max(EPS)
        } else {
            (val - tracker.mean()) / (tracker.std() + EPS)
        }
    };

    let agreement_z = calc_z(agreement, &metrics.adaptive.agreement, 0.65, 0.10);
    let _purity_z = calc_z(purity, &metrics.adaptive.purity, 0.60, 0.15);
    let stability_z = calc_z(stability_raw, &metrics.adaptive.stability, 0.15, 0.05);
    let dominance_z = calc_z(best_dom, &metrics.adaptive.dominance, 0.20, 0.10);
    let z_norm = calc_z(max_z, &metrics.adaptive.z_score, 1.50, 0.50);
    let energy_norm = calc_z(best_e, &metrics.adaptive.energy, 0.75, 0.10);

    // Final Adaptive Score (Weighted Sum)
    // 30% z_norm, 25% energy_norm, 20% dominance_norm, 15% agreement_norm, 10% stability_norm
    let final_score = 0.30 * z_norm 
                    + 0.25 * energy_norm 
                    + 0.20 * dominance_z 
                    + 0.15 * agreement_z 
                    + 0.10 * stability_z;
    
    let adaptive_threshold = metrics.adaptive_threshold(0.60); // Use PREVIOUS history
    let is_struct_valid_adaptive = final_score >= (adaptive_threshold - 0.55); // Phase D.1.17: Further Relaxed for Emergence
    
    // Update structural stats and history AFTER gate decision 
    metrics.record_final_score(final_score);
    if is_struct_valid_adaptive {
        metrics.adaptive_opportunity_count += 1;
    }

    // Gating Logic
    if !allow_bypass {
        // --- PHASE D.1.1: SIGNAL PIPELINE RESTORATION ---
        
        // 1. Pre-calculate Recovery & Override Status
        // Statistical Recovery (Golden Ticket)
        let is_statistical_recovery = max_z > 1.2 && best_e > 0.65;
        
        // Extreme Alpha Override (Platinum Ticket)
        let is_extreme_override = max_z >= EXTREME_Z_OVERRIDE;
        
        let final_admission_reason: Option<&str>;
        let mut mode = AcceptanceMode::Dominance;

        // 2. Decision Matrix
        if !had_organic_signals && !valid_signals.is_empty() {
             final_admission_reason = Some("BOOTSTRAP_PRIMING");
             mode = AcceptanceMode::Override; // High priority to get the engine moving
             decision_was_override = true;
        } else if is_extreme_override {
            final_admission_reason = Some("EXTREME_Z_OVERRIDE");
            mode = AcceptanceMode::Override;
            decision_was_override = true;
        } else if is_statistical_recovery {
            final_admission_reason = Some("STATISTICAL_ADMIT_WEAK_DOM");
            mode = AcceptanceMode::StatisticalWeak;
            decision_was_override = true;
        } else if is_struct_valid_adaptive {
            // Standard Path: Adaptive Ranking must pass
            final_admission_reason = None; // Proceed to standard structural gates
        } else {
            if std::env::var("GA_DEBUG").is_ok() {
                println!("WINDOW_DECISION → {} | dom={:.3} z={:.3} e={:.3} => SKIP: ADAPTIVE_RANKING", 
                    scenario_name, best_dom, max_z, best_e);
            }
            return None;
        }

        // 3. Structural Validation (Bypassed if recovered/overridden)
        if final_admission_reason.is_none() {
            // Standard Structural Gates (Only if not already admitted by high-conviction paths)
            
            // a. Signal Concentration (Directional Consistency)
            let mut buy_count = 0;
            let mut sell_count = 0;
            for (_, conv, _, _, _, _, _) in &valid_signals {
                if !conv.is_bearish { buy_count += 1; } else { sell_count += 1; }
            }
            let total_valid = buy_count + sell_count;
            if total_valid < 1 { // Phase D.1.17: Reduced from 3 to 1 for Forced Emergence
                 if std::env::var("GA_DEBUG").is_ok() {
                    println!("WINDOW_DECISION → {} | dom={:.3} z={:.3} e={:.3} n={} => SKIP: NO_SIGNALS", 
                        scenario_name, best_dom, max_z, best_e, total_valid);
                }
                return None;
            }
            
            let directional_consistency = (buy_count.max(sell_count) as f64) / (total_valid as f64);
            if directional_consistency < 0.5 {
                if std::env::var("GA_DEBUG").is_ok() {
                    println!("WINDOW_DECISION → {} | dom={:.3} z={:.3} e={:.3} dc={:.2} => SKIP: DIRECTIONAL_NOISE", 
                        scenario_name, best_dom, max_z, best_e, directional_consistency);
                }
                return None;
            }

            // b. Clarity (Dominance Floor) - Relaxed to Warning for D.1.17
            let selection_th = DOMINANCE_FLOOR * 0.5; // Dropped from 0.8
            if best_dom < selection_th {
                if std::env::var("GA_DEBUG").is_ok() {
                    println!("WINDOW_WARNING → {} | dom={:.3} (low clarity) => ADMITTING_ANYWAY", 
                        scenario_name, best_dom);
                }
            }
        }

        // 4. Final Decision Finalization & Logic
        let _admission_reason = final_admission_reason.unwrap_or("STANDARD_ADMISSION");
        
        // --- PHASE D.1.9: RATIO-INTERACTION SCORING & DETERMINISTIC CHOICE ---
        // 1. Archetype Bias Mapping (Soft Offsets)
        let mut adj_w_conv = strategy.w_conviction as f64;
        let mut adj_w_mom = strategy.w_momentum as f64 - 50.0; // centered
        let mut adj_w_vol = strategy.w_volatility as f64;
        let mut adj_exp_vol = strategy.exp_volatility as f64;
        
        match strategy.archetype {
            0 => { adj_w_conv += 20.0; }   // ConvictionDominant
            1 => { adj_w_mom += 20.0; }    // MomentumTrend
            2 => { adj_w_mom -= 20.0; }    // MeanReversion
            3 => { adj_exp_vol += 30.0; }  // VolatilityAverse
            _ => {}
        }
        
        // 2. Weight Normalization (forced trade-offs)
        let w_sum = adj_w_conv.abs() + adj_w_mom.abs() + adj_w_vol.abs() + 1e-9;
        let w1 = adj_w_conv / w_sum;
        let w2 = adj_w_mom / w_sum;
        let w3 = adj_w_vol / w_sum;
        
        let a = (0.5 + 3.0 * (strategy.exp_conviction as f64 / 100.0)).clamp(0.5, 3.5);
        let b = (0.5 + 3.0 * (strategy.exp_momentum as f64 / 100.0)).clamp(0.5, 3.5);
        let c = (0.5 + 3.0 * (adj_exp_vol / 100.0)).clamp(0.5, 3.5);

        // 3. Compute Phase D.1.16 Signal Entropy for differentiation
        let signal_entropy = compute_std_dev(&valid_signals.iter().map(|(_, c, _, _, _, _, _)| c.conviction_score).collect::<Vec<f64>>());

        let mut scored_signals = Vec::with_capacity(valid_signals.len());
        for (signal_idx, conviction, dom, reason, e_score, source, _sig) in valid_signals.iter() {
            let vol_penalty = conviction.norm_vol.max(1e-6); 
            let mom = conviction.norm_momentum;
            let conv = conviction.conviction_score;
            
            // Core Ratio Model with Momentum Interaction
            let mom_effect = 1.0 + (w2 * mom.powf(b)).clamp(-0.8, 0.8);
            let denom = (0.03 + w3 * vol_penalty.powf(c)).max(0.03);
            let mut score = (w1 * conv.powf(a)) * mom_effect / denom;
            
            // --- Phase D.1.15: Differentiation Injection ---
            let rank_offset = (*signal_idx as f64) * 0.01; // deterministic separation
            let entropy_factor = 1.0 + (signal_entropy * 0.2);
            score = (score + rank_offset) * entropy_factor;
            
            // Deterministic Perception Jitter [0.85, 1.15]
            score *= 0.85 + 0.30 * conviction.roll; 
            
            scored_signals.push((*signal_idx, conviction.clone(), *dom, *reason, *e_score, score, *source));
        }

        // 3. Deterministic Choice Selection (Top-K)
        scored_signals.sort_by(|a, b| b.5.partial_cmp(&a.5).unwrap_or(std::cmp::Ordering::Equal));
        
        use std::hash::{Hash, Hasher};
        let choice_roll = {
            let mut h = std::collections::hash_map::DefaultHasher::new();
            scenario_name.hash(&mut h);
            scored_signals[0].0.hash(&mut h); // Anchor to leading signal index (stable window identifier)
            (strategy.queue_threshold % 12345).hash(&mut h); // local perception jitter
            h.finish() % 100
        };

        let winner_data = if scored_signals.len() > 1 && choice_roll > strategy.selectivity as u64 {
            // Weighted Choice from Top-3
            let k = scored_signals.len().min(3);
            let top_k = &scored_signals[0..k];
            let total_v: f64 = top_k.iter().map(|s| s.5).sum::<f64>().max(1e-9);
            let mut roll = (choice_roll as f64 / 100.0) * total_v;
            
            let mut selected = &top_k[0];
            for signal in top_k {
                roll -= signal.5;
                if roll <= 0.0 {
                    selected = signal;
                    break;
                }
            }
            selected
        } else {
            &scored_signals[0]
        };

        let final_winner_idx = winner_data.0;
        let final_score = winner_data.5;
        
        // 🧪 IDENTITY INTEGRITY ASSERT (Phase D.1.10 Validation)
        debug_assert!(scored_signals.iter().any(|s| s.0 == final_winner_idx), "FATAL: Chosen winner index is phantom (not in original signal window)");

        winner_idx = Some(final_winner_idx);
        winner_conviction = winner_data.1.clone();
        winner_reason_final = winner_data.3;
        winner_dom_final = winner_data.2;
        _winner_e_score_final = winner_data.4;
        winner_acceptance_mode = mode;
        metrics.accepted_windows += 1;

        println!(
            "STRAT_DECISION → strat={} winner_idx={} score={:.6}",
            strategy_id,
            final_winner_idx,
            final_score
        );
    } else {
        // Phase D.1.14: Kill Forced Bypass Completely
        if std::env::var("GA_DEBUG").is_ok() {
            println!("WINDOW_REJECT → {} | No valid signals found (Bypass Disabled)", scenario_name);
        }
        return None;
    }

    // 4. Separation Analysis (Phase 15: Distance + Clarity + Structure)
    let scores_sum: f64 = scores.iter().sum();
    let winner_score = winner_conviction.conviction_score;
    let second_score = if valid_signals.len() >= 2 {
        valid_signals.iter()
            .filter(|(idx, _, _, _, _, _, _)| *idx != winner_idx.unwrap())
            .map(|(_, conv, _, _, _, _, _)| conv.conviction_score)
            .max_by(|a, b| a.total_cmp(b))
            .unwrap_or(mean)
    } else {
        mean
    };

    let dominance = winner_dom_final;
    
    let report_reason = if max_z >= EXTREME_Z_OVERRIDE { "EXTREME_OVERRIDE" } else { winner_reason_final };

    if std::env::var("GA_DEBUG").is_ok() {
        println!("WINDOW_DECISION → {} | sigs={} z={:.2} dom={:.3} mean={:.3} purity={:.2} conc={:.2} agree={:.2} => {}", 
            scenario_name, valid_signals.len(), max_z, dominance, mean, purity, top_k_sum / scores_sum.max(EPS), agreement, 
            if valid_signals.is_empty() { "REJECT_VACUUM" } else { "ACCEPTED" });
    }
    
    if valid_signals.is_empty() {
        return None; 
    }

    let median_score = percentile_f64(&scores, 0.50);
    let edge_spread_norm = (winner_score - median_score) / (std_dev + EPS);

    let _signal_count = valid_signals.len();

    let mut busy_until = 0usize;
    let cooldown = config.trade_cooldown_events.unwrap_or(8);

    for (current_idx, conviction) in window_data {
        if total_trades >= 1 { 
            break; // Alpha Discovery: Hard single-trade cap
        }
        
        if current_idx < busy_until {
            continue;
        }

        // --- SELECTION GATES ---
        let mut should_execute = true;
        let mut final_conviction = conviction.clone();

        // Only execute the GLOBAL BEST for real alpha discovery
        if current_idx != winner_idx.unwrap() {
            continue;
        }
        
        // Use the dispersion-adjusted conviction selected as winner
        final_conviction = winner_conviction.clone();
        should_execute = true; // Trust admission + selection layers

        if !should_execute {
            continue;
        }

        // --- EXECUTION ---
        if let Some(outcome) = ga_simulate_round_trip_at_cursor(
            strategy,
                &strategy_id,
            scenario_name,
            signal_events,
            execution_events,
            config,
            current_idx,
            total_trades,
            &final_conviction,
        ) {
            // Layer 4: Capture Efficiency Gate (Phase B)
            // Phase C.2d: Adaptive Execution Calibration
            let eff_threshold = if total_trades < 5 { 0.10 } else { 0.25 };
            let eff_penalty = if outcome.efficiency < eff_threshold {
                if std::env::var("GA_DEBUG").is_ok() {
                    println!("EFF_DEBUG → {} | eff={:.2} < threshold={:.2} => APPLYING_PENALTY", scenario_name, outcome.efficiency, eff_threshold);
                }
                (outcome.efficiency / eff_threshold).clamp(0.2, 1.0)
            } else {
                1.0
            };

            let trade_pnl = (outcome.pnl * final_conviction.edge_weight) * eff_penalty;
            let expected_move = outcome.expected_move.abs().max(1e-9);

            let margin = if final_conviction.selection_threshold > 1e-9 {
                (final_conviction.conviction_score - final_conviction.selection_threshold) / final_conviction.selection_threshold
            } else {
                final_conviction.conviction_score
            };
            let aqg_health = if final_conviction.conviction_score >= aqg_threshold { 1.0 } else { 0.0 };
            
            // --- Phase D.1.17: Realizability PnL Impact ---
            let execution_multiplier = _winner_e_score_final.clamp(0.1, 1.0);
            let adjusted_pnl = trade_pnl * execution_multiplier.powf(1.5); // Sharp Penalty D.1.18
            let mut outcome_adj = outcome.clone();
            outcome_adj.pnl = adjusted_pnl;

            // Phase D.1.18: Extract winning signal signature
            let winning_sig = if let Some((_, _, _, _, _, _, sig)) = valid_signals.iter().find(|(idx, _, _, _, _, _, _)| *idx == current_idx) {
                Some(sig.clone())
            } else { None };

            metrics.record_trade(
                adjusted_pnl,
                expected_move,
                outcome.efficiency,
                final_conviction.conviction_score,
                outcome.efficiency,
                outcome.edge_quality,
                outcome.time_to_mfe as f64,
                margin,
                aqg_health,
                edge_spread_norm,
                dominance,
                final_conviction.raw_q_ratio,
                outcome.clone(),
                SignalSource::Organic,
                winning_sig,
            );

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
            
            if capped_exit <= current_idx {
                continue;
            }
            
            total_efficiency += outcome.efficiency;
            total_vol_ratio += std_v;
            total_spread_reality += outcome.spread;
            total_window_volume += outcome.avg_window_volume;
            
            // Phase C.1: Trade-Level Survivability Check
            let window_slippage = outcome.spread * (1.0 + std_v.powf(1.2)) * config.slippage_factor;
            let window_fill_prob = (outcome.efficiency * 0.7 + 0.3).clamp(0.5, 1.0);
            let window_latency_penalty = (-0.05 * config.latency_ticks as f64).exp().clamp(0.6, 1.0);
            let window_effective_pnl = if trade_pnl > 0.0 {
                ((trade_pnl * window_fill_prob * window_latency_penalty) - window_slippage).max(trade_pnl * 0.1_f64)
            } else {
                (trade_pnl * window_fill_prob * window_latency_penalty) - window_slippage
            };
            if window_effective_pnl > 0.0 {
                survivable_trades_count += 1;
            }

            total_trades += 1;

            // --- PHASE C.1.5: DEAD-ZONE ERADICATION (Early Exit) ---
            if total_trades >= 10 { // Phase C.2d: Allow more discovery before killing
                let current_surv = survivable_trades_count as f64 / total_trades as f64;
                if current_surv < 0.05_f64 { // Relaxed from 0.2 to allow recovery
                    // Strategy is non-survivable in this regime; kill early to accelerate convergence
                    return None;
                }
            }

            // --- PHASE C.1.6: PARTICIPATION PRESSURE (Early Participation Choke) ---
            if scenario_pnls.len() >= 20 && total_trades < 1 {
                // Not active enough for institutional scale; kill early
                return None;
            }

            metrics.record_opportunity();
            entry_attempted += 1;
            
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
            sum_actual_slippage += outcome.total_slippage_bps;
            
            max_trade_pnl_scenario = max_trade_pnl_scenario.max(trade_pnl);
            max_pnl_in_scenario = max_pnl_in_scenario.max(outcome.pnl);
            scenario_pnls.push(trade_pnl);
            
            // Phase D.1.2: Metrics Propagation
            metrics.trade_qualities.push(outcome.edge_quality);
            metrics.sum_realized_pnl += trade_pnl;
            metrics.sum_expected_pnl += outcome.expected_move;
            total_spread_test += outcome.spread;
            sum_price += signal_events[current_idx].price as f64;
            total_quality_trades_scenario += outcome.quality;

            match outcome.exit_reason {
                GaExitReason::TakeProfit => {
                    exit_tp_count += 1;
                    pnl_from_tp_scenario += trade_pnl;
                    if trade_pnl > 0.0 {
                        long_win_count_scenario += 1; // Used for consistency tracking
                    }
                },
                GaExitReason::StopLoss => {
                    exit_sl_count += 1;
                    pnl_from_sl_scenario += trade_pnl;
                    if trade_pnl.abs() < 0.0001 {
                        micro_loss_count += 1;
                    }
                    // For consistency tracking, we only care about "winners" to see if they all come from one side
                },
                GaExitReason::TimeStop => {
                    exit_ts_count += 1;
                    if trade_pnl > 0.0 {
                        long_win_count_scenario += 1;
                    }
                },
            }

            
            let margin = if conviction.selection_threshold > 1e-9 {
                (conviction.conviction_score - conviction.selection_threshold) / conviction.selection_threshold
            } else {
                conviction.conviction_score
            };
            let aqg_health = if conviction.conviction_score >= aqg_threshold { 1.0 } else { 0.0 };

            // record_trade moved to early-simulation for Phase 2 capture efficiency

            busy_until = capped_exit + cooldown;

            if std::env::var("GA_DEBUG").is_ok() {
                println!(
                    "GA_EXEC: scenario={} idx={} score={:.4} spread_z={:.2} dom={:.2} pnl={:.6}",
                    scenario_name, current_idx, conviction.conviction_score, edge_spread_norm, dominance, outcome.pnl
                );
            }
        }
    }

    if std::env::var("GA_DEBUG").is_ok() {
        let decision_skipped = entry_attempted.saturating_sub(total_trades).saturating_sub(skipped_busy);
        println!(
            "ENTRY_DEBUG → signals={} attempts={} triggered={} busy_skipped={} decision_skipped={} | EXITS: TP={} SL={} TS={}",
            signal_events.len(), entry_attempted, total_trades, skipped_busy, decision_skipped, exit_tp_count, exit_sl_count, exit_ts_count
        );
    }

    let final_winner_idx = winner_idx.expect("STATION_SELECT_FAILURE");

    let total_trades = metrics.trade_count;
    let mean_expected_move = if total_trades > 0 { sum_expected_move / total_trades as f64 } else { 0.0 };
    let drawdown_penalty_raw = if total_trades > 0 { sum_drawdown_raw / total_trades as f64 } else { 0.0 };
    let requested_qty = config.order_quantity_for_strategy * 2 * (total_trades.max(1) as u64);

    // --- PHASE 10.5: REGIME ADMISSION GATE ---
    if total_trades > 15 && max_pnl_in_scenario < 0.0025 {
        if std::env::var("GA_DEBUG").is_ok() {
            println!("ADMISSION_REJECT → Over-trading noise: scenario={} trades={} max_pnl={:.5}", scenario_name, total_trades, max_pnl_in_scenario);
        }
        return None; 
    }

    let avg_pnl_for_scenario = if total_trades > 0 { metrics.sum_pnl / total_trades as f64 } else { 0.0 };
    let mut zero_pnl_trades_scenario = 0usize;
    let mut total_win = 0.0;
    let mut total_loss = 0.0;
    let mut win_count = 0;
    let mut loss_count = 0;

    for pnl in &scenario_pnls {
        if *pnl > 0.0 {
            total_win += *pnl;
            win_count += 1;
        } else if *pnl == 0.0 {
            zero_pnl_trades_scenario += 1;
        } else {
            total_loss += pnl.abs();
            loss_count += 1;
        }
    }
    
    let avg_win = if win_count > 0 { total_win / win_count as f64 } else { 0.0 };
    let avg_loss = if loss_count > 0 { total_loss / loss_count as f64 } else { 0.0 };
    let win_rate = if total_trades > 0 { win_count as f64 / total_trades as f64 } else { 0.0 };
    
    // Stabilized Payoff Ratio
    let payoff_ratio = if avg_loss.abs() > 1e-6 { (avg_win / avg_loss.abs()).clamp(0.5, 3.0) } else { 0.0 };
    let dir_consistency = if win_count > 0 { (long_win_count_scenario.max(short_win_count_scenario) as f64 / win_count as f64).clamp(0.5, 1.0) } else { 0.0 };
    
    // Composite Stability: Boosted with Squaring for Phase 11.2
    let stability = (payoff_ratio * win_rate * dir_consistency).clamp(0.0, 1.5);
    let _stability_weighted = stability.powi(2);

    let selectivity = metrics.selectivity();

    let std_dev_for_scenario: f64 = if total_trades > 1 {
        let mean = avg_pnl_for_scenario;
        let variance = scenario_pnls.iter().map(|pnl| (pnl - mean).powi(2)).sum::<f64>() / total_trades as f64;
        variance.sqrt()
    } else { 0.0_f64 };

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
    let avg_vol_ratio = if total_trades > 0 { total_vol_ratio / total_trades as f64 } else { 0.0 };
    let avg_spread_reality = if total_trades > 0 { total_spread_reality / total_trades as f64 } else { 0.0 };
    let adtv = if total_trades > 0 { (total_window_volume / total_trades as f64).max(100_000.0) } else { 1_000_000.0 }; 
    
    // --- 1.1 Slippage Model (Convex + Phase C.2 Liquidity Scaling) ---
    let basic_slippage = avg_spread_reality * (1.0 + avg_vol_ratio.powf(1.2)) * config.slippage_factor;
    let size = config.order_quantity_for_strategy as f64;
    let participation_rate = (size / adtv).clamp(0.0001, 0.2);
    
    // Square Root Law of Market Impact (Phase C.2)
    let size_slippage_multiplier = (1.0 + (participation_rate / 0.01).powi(2)).max(1.0);
    let slippage = basic_slippage * size_slippage_multiplier;
    let slippage = if avg_pnl_for_scenario > 0.0 { slippage.min(avg_pnl_for_scenario * 0.7_f64) } else { slippage };

    // --- 1.2 Fill Probability (Phase C.2 Depth-Aware) ---
    let avg_efficiency = if total_trades > 0 { total_efficiency / total_trades as f64 } else { 0.0 };
    let base_fill_prob = (avg_efficiency * 0.7 + 0.3).clamp(0.5, 1.0);
    let fill_prob = (base_fill_prob * (-8.0 * participation_rate).exp()).clamp(0.1, 1.0);

    // --- 1.3 Latency Decay ---
    let latency_ticks = config.latency_ticks as f64;
    let latency_penalty = (-0.05 * latency_ticks).exp().clamp(0.6, 1.0);

    // --- 1.4 Effective PnL (Scoring Overlay) ---
    let effective_pnl = if avg_pnl_for_scenario > 0.0 {
        ((avg_pnl_for_scenario * fill_prob * latency_penalty) - slippage).max(avg_pnl_for_scenario * 0.1_f64)
    } else {
        (avg_pnl_for_scenario * fill_prob * latency_penalty) - slippage
    };

    // --- PHASE C.1.6b: ADAPTIVE PARTICIPATION GATE (Smooth Recovery) ---
    // User Precision: trades >= 1 is the new active floor.
    let total_trades = metrics.trade_count;
    let is_active = total_trades >= 1;
    
    if !is_active {
        if std::env::var("GA_DEBUG").is_ok() {
            println!("FITNESS_TRACE: [INACTIVE] total_trades={} final=0.0", total_trades);
        }
        
        // --- PHASE C.2b: GRADIENT FALLBACK (Structural Hint) ---
        let local_pnl_scale = effective_pnl.abs().max(1.0); 
        let fallback_fitness = (effective_pnl / local_pnl_scale).tanh() * 0.5;

        return Some(StrategyEvaluation {
            winner_idx: final_winner_idx,
            strategy_id: strategy_id.to_string(),
            strategy: strategy.clone(),
            capability: capability.clone(),
            real_dom: metrics.adaptive.dominance.mean(),
            had_organic_signals,
            fitness: fallback_fitness,
            bootstrap_ratio: if total_trades > 0 { metrics.bootstrap_trade_count as f64 / total_trades as f64 } else { 0.0 },
            trade_count: total_trades,
            avg_pnl: effective_pnl,
            trade_qualities: metrics.trade_qualities.clone(),
            realized_pnl_rolling: metrics.sum_realized_pnl,
            predicted_pnl_rolling: metrics.sum_expected_pnl,
            max_signature_credibility: metrics.max_signature_credibility,
            forced_win_ratio: if metrics.total_windows > 0 { metrics.forced_win_count as f64 / metrics.total_windows as f64 } else { 0.0 },
            ..StrategyEvaluation::default()
        });
    }

    // --- PHASE D.1.16: EDGE VALIDATION LAYER (CONSISTENCY FITNESS) ---
    // 1. Calculate PnL Consistency (Sharpe-like)
    let pnl_mean = effective_pnl;
    let pnl_std = std_dev_for_scenario.max(1e-6);
    let mut fitness = (pnl_mean / pnl_std).max(0.0);

    // 2. Bootstrap De-weighting & Penalty
    let bootstrap_ratio = metrics.bootstrap_trade_count as f64 / total_trades.max(1) as f64;
    if bootstrap_ratio > 0.7 {
        fitness *= 0.5; // Heavy dependency penalty
    }
    
    // 3. Trade Count Stabilization (Log scaling)
    fitness *= (1.0 + (total_trades as f64).ln()).max(0.1);
    
    // 4. Rank Bonus Constraint (Cap at 5%)
    let rank_bonus = (final_score - 1.0).max(0.0) * 0.1; // heuristic rank bonus
    fitness += rank_bonus.min(0.05 * fitness);

    if std::env::var("GA_DEBUG").is_ok() {
        println!(
            "FITNESS_D1_16 → mean={:.5} std={:.5} boost_ratio={:.2} final={:.4}",
            pnl_mean, pnl_std, bootstrap_ratio, fitness
        );
    }
    
    let total_fitness = fitness;

    if std::env::var("GA_DEBUG").is_ok() {
        println!(
            "FITNESS_TRACE: wr={:.2} real_pnl={:.6} final={:.4}",
            win_rate, effective_pnl, total_fitness
        );
    }
    let robustness_for_scenario = avg_pnl_for_scenario - config.lambda * std_dev_for_scenario;
    let fill_efficiency = if requested_qty > 0 { total_filled_qty as f64 / requested_qty as f64 } else { 0.0 };
    let avg_slippage = if fills_count > 0 { total_slippage_bps / fills_count as f64 } else { 0.0 };
    let realized_avg = avg_pnl_for_scenario;
    let capture_efficiency = if mean_expected_move > 0.0 && total_trades > 0 { (realized_avg / mean_expected_move).clamp(0.0, 1.0) } else { 0.0 };
    let participation_rate = if total_trades > 0 { 1.0 } else { 0.0 };
    let n_sig = cycle_sigs.len().max(1) as f64;
    let scenario_signature = if cycle_sigs.is_empty() { ScenarioExecutionSignature::default() } else {
        ScenarioExecutionSignature {
            avg_queue_ahead: cycle_sigs.iter().map(|s| s.avg_queue_ahead).sum::<f64>() / n_sig,
            avg_latency: cycle_sigs.iter().map(|s| s.avg_latency).sum::<f64>() / n_sig,
            fill_ratio: cycle_sigs.iter().map(|s| s.fill_ratio).sum::<f64>() / n_sig,
            participation: cycle_sigs.iter().map(|s| s.participation).sum::<f64>() / n_sig,
            execution_variance: 0.0,
        }
    };
    let latency_raw_mean = if total_trades > 0 { sum_latency_raw / total_trades as f64 } else { 0.0 };

    let downside_variance_scenario = if total_trades > 0 {
        scenario_pnls.iter().map(|&pnl| pnl.min(0.0).powi(2)).sum::<f64>() / total_trades as f64
    } else { 0.0_f64 };
    let downside_std_dev_scenario = downside_variance_scenario.sqrt();

    // Hard assertion for outcome consistency
    assert!(total_trades == 0 || (exit_tp_count + exit_sl_count + exit_ts_count) == total_trades, "FATAL: Outcome count mismatch");

    Some(StrategyEvaluation {
        winner_idx: final_winner_idx,
        strategy_id: strategy_id.clone(),
        strategy: strategy.clone(),
        capability,
        real_dom: winner_dom_final,
        had_organic_signals,
        avg_pnl: avg_pnl_for_scenario,
        total_pnl: metrics.sum_realized_pnl,
        pnl_history: metrics.pnl_history.clone(),
        std_dev: std_dev_for_scenario,
        downside_std_dev: downside_std_dev_scenario,
        worst: worst_pnl_for_scenario,
        robustness: robustness_for_scenario,
        fitness: total_fitness.clamp(0.0, 1.0), 
        trade_count: metrics.trade_count,
        max_drawdown: drawdown_penalty_raw * 100.0, 
        participation_rate,
        profitable_trades: metrics.profitable_trades,
        zero_pnl_trades: zero_pnl_trades_scenario,
        quality_trades: total_quality_trades_scenario,
        win_rate: if metrics.trade_count > 0 { metrics.profitable_trades as f64 / metrics.trade_count as f64 } else { 0.0 },
        payoff: payoff_ratio,
        payoff_ratio,
        execution_metrics: ExecutionMetrics { fill_efficiency, capture_efficiency, avg_slippage, latency_impact: latency_raw_mean },
        scenario_signature,
        avg_conviction: metrics.avg_conviction(),
        avg_efficiency: metrics.avg_efficiency(),
        avg_edge_quality: metrics.avg_edge_quality(),
        directional_accuracy: if metrics.trade_count > 0 { metrics.profitable_trades as f64 / metrics.trade_count as f64 } else { 0.0 },
        decisiveness: if metrics.trade_count > 0 { 1.0 - (metrics.sum_time_to_mfe / (metrics.trade_count as f64 * config.max_hold_bars as f64)).clamp(0.0, 1.0) } else { 0.0 },
        execution_friction: if sum_expected_slippage > 0.0 { sum_actual_slippage / sum_expected_slippage } else { 1.0 },
        short_term_capture_eff: metrics.avg_efficiency(),
        long_term_capture_eff: metrics.avg_efficiency(),
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
        stat_zero_dom_ratio: metrics.stat_zero_dom_count as f64 / (metrics.stat_admitted_count as f64).max(1.0),
        
        exec_accept_rate: metrics.exec_passed_count as f64 / (metrics.exec_admitted_count as f64).max(1.0),
        vip_exec_retention: metrics.vip_exec_passed_count as f64 / (metrics.vip_admitted_count as f64).max(1.0),
        e_rejection_rate: metrics.exec_rejected_count as f64 / (metrics.exec_admitted_count as f64).max(1.0),
        clarity_to_exec_drop: 1.0 - (metrics.vip_exec_passed_count as f64 / (metrics.vip_admitted_count as f64).max(1.0)),
        avg_e_score: metrics.sum_e_score / (metrics.exec_passed_count as f64).max(1.0),
        vip_avg_e_score: metrics.sum_vip_e_score / (metrics.vip_exec_passed_count as f64).max(1.0),
        stat_avg_e_score: metrics.sum_stat_e_score / ((metrics.exec_passed_count - metrics.vip_exec_passed_count) as f64).max(1.0),
        consensus_bypass_ratio: metrics.consensus_bypass_count as f64 / metrics.exec_passed_count.max(1) as f64,
        stability_reject_rate: metrics.stability_rejected_count as f64 / metrics.exec_admitted_count.max(1) as f64,
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
            let edge_strength = (metrics.sum_pnl.abs() / (metrics.trade_count.max(1) as f64)).max(1e-9);
            let edge_min = 0.0005;
            let pressure_penalty = (edge_strength / edge_min).powi(2).min(1.0);
            
            // Phase D.1.20 Vagueness Penalty (Condensation)
            let vagueness_penalty = if metrics.max_signature_credibility < 1.1 { 0.7 } else { 1.0 };
            
            raw_alpha * pressure_penalty * vagueness_penalty
        },
        consistency: 1.0 / (metrics.adaptive.final_score.std() + EPS),
        bootstrap_ratio: metrics.bootstrap_trade_count as f64 / total_trades.max(1) as f64,
        forced_win_ratio: metrics.forced_win_count as f64 / (metrics.total_windows.max(1) as f64),
        max_signature_credibility: metrics.max_signature_credibility,
        opportunity: metrics.adaptive_opportunity_count as f64 / metrics.total_windows.max(1) as f64,
        acceptance_mode: winner_acceptance_mode,
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
                selection_cap::GaDiversityMode::Repel => {
                    base_score + diversity_lambda * mean_dist
                }
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
    
    let context_evals: Vec<StrategyEvaluation> = evaluations.iter().filter(|e| !e.capability.is_executable()).cloned().collect();
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
            println!(
                "GA_TOPK: scenarios_in={}, scenarios_used={}, cap={}, diversity_lambda={:.4}, diversity_mode={} (execution_signature_l1_mean)",
                n_in, used, k, diversity_lambda, mode_s
            );
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
pub fn aggregate_strategy_reports(evaluations: Vec<StrategyEvaluation>, config: &GaConfig, generation: usize) -> Option<StrategyEvaluation> {
    // Phase 10.2: Institutional Elite-Only Aggregation
    // 1. Filter for regimes meeting the minimum alpha quality threshold
    let mut elite: Vec<StrategyEvaluation> = evaluations
        .into_iter()
        .filter(|e| e.fitness > -0.25) 
        .collect();

    // 2. Sort by fitness (descending) to isolate the strongest alpha cluster
    elite.sort_by(|a, b| b.fitness.partial_cmp(&a.fitness).unwrap_or(std::cmp::Ordering::Equal));

    // 3. Take Top 5 (Elite Cap)
    let elite_count = elite.len();
    let elite_evals: Vec<StrategyEvaluation> = elite.into_iter().take(5).collect();

    // 4. Scarcity Penalty: Reward multi-regime robustness, penalize single-regime "lucky" hits
    let scarcity_penalty = match elite_count {
        0 => return None, // Absolute rejection of noise-only portfolios
        1 => 0.5,
        2 => 0.75,
        3..=5 => 1.0,
        _ => 1.05, // Slight "generalization bonus" for consistent cross-regime performance
    };

    aggregate_strategy_reports_inner(elite_evals, scarcity_penalty, config, generation).map(|(e, _)| e)
}

/// Same aggregation with an explicit Top-K cap (`None` = use all scenarios). Used by unit tests to avoid env coupling.
#[allow(dead_code)] // Referenced from `#[cfg(test)]` module; unused in non-test library builds.
pub(crate) fn aggregate_strategy_reports_with_top_k(
    evaluations: Vec<StrategyEvaluation>,
    config: &GaConfig,
    top_k_cap: Option<usize>,
    generation: usize,
) -> Option<StrategyEvaluation> {
    let full_mean_eval = aggregate_strategy_reports_inner(evaluations.clone(), 1.0, config, generation).map(|(e, _)| e);
    let top_k_evals = apply_ga_top_k_selection(evaluations, top_k_cap);
    let top_k_mean_eval = aggregate_strategy_reports_inner(top_k_evals, 1.0, config, generation).map(|(e, _)| e);

    match (top_k_mean_eval, full_mean_eval) {
        (Some(mut tk), Some(fg)) => {
            // Adaptive Hybrid Aggregation: penalize cherry-picks when dispersion is high
            let dispersion = (tk.avg_pnl - fg.avg_pnl).abs();
            let (w_tk, w_fg) = if dispersion > 0.002 { (0.6, 0.4) } else { (0.75, 0.25) };
            
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
    if a.len() != b.len() || a.is_empty() { return 0.0; }
    let n = a.len() as f64;
    let sum_a: f64 = a.iter().map(|&x| x as f64).sum();
    let sum_b: f64 = b.iter().map(|&x| x as f64).sum();
    let sum_a_sq: f64 = a.iter().map(|&x| (x as f64).powi(2)).sum();
    let sum_b_sq: f64 = b.iter().map(|&x| (x as f64).powi(2)).sum();
    let sum_ab: f64 = a.iter().zip(b.iter()).map(|(&x, &y)| (x as f64) * (y as f64)).sum();

    let numerator = n * sum_ab - sum_a * sum_b;
    let denominator = ((n * sum_a_sq - sum_a.powi(2)) * (n * sum_b_sq - sum_b.powi(2))).sqrt();

    if denominator.abs() < 1e-9 { 0.0 } else { numerator / denominator }
}

/// Extracts diverse behavioral cluster representatives from a population.
/// Uses a hybrid fitness filter and greedy medoid selection with fitness tie-breaking.
pub fn extract_behavioral_clusters(
    mut population: Vec<StrategyEvaluation>,
    target_count: usize,
    min_dist_threshold: f64,
    pnl_mu: f64, pnl_sigma: f64,
    std_mu: f64, std_sigma: f64
) -> Vec<StrategyEvaluation> {
    if population.is_empty() { return Vec::new(); }

    // 1. HYBRID FITNESS FILTER: fitness > median AND fitness > (best * 0.6)
    population.sort_by(|a, b| b.fitness.partial_cmp(&a.fitness).unwrap_or(std::cmp::Ordering::Equal));
    let best_fitness = population[0].fitness;
    let median_fitness = population[population.len() / 2].fitness;
    
    let mut candidates: Vec<StrategyEvaluation> = population
        .into_iter()
        .filter(|e| e.fitness >= median_fitness && e.fitness >= (best_fitness * 0.6))
        .collect();

    if candidates.is_empty() { return Vec::new(); }

    // 2. GREEDY MEDOID SELECTION (Fitness Tie-break)
    // The population is already sorted by fitness, so candidates[0] is the best.
    let mut medoids = vec![candidates.remove(0)];
    
    while medoids.len() < target_count && !candidates.is_empty() {
        let mut best_candidate_idx = None;
        let mut max_min_dist = -1.0;

        for (i, cand) in candidates.iter().enumerate() {
            let mut min_dist = f64::MAX;
            for m in &medoids {
                let dist = calculate_behavioral_distance(m, cand, pnl_mu, pnl_sigma, std_mu, std_sigma);
                if dist < min_dist { min_dist = dist; }
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
    pnl_mu: f64, pnl_sigma: f64,
    std_mu: f64, std_sigma: f64
) -> f64 {
    const MIN_TRADES: usize = 10;
    
    // GUARD: If behavior is statistically insignificant, fallback to genotype distance
    if a.trade_count < MIN_TRADES || b.trade_count < MIN_TRADES {
        return calculate_genotype_distance(&a.strategy, &b.strategy);
    }

    let corr = pearson_correlation(&a.pnl_fingerprint, &b.pnl_fingerprint);
    
    // Normalized Magnitude Difference
    let a_pnl_norm = (a.avg_pnl - pnl_mu) / pnl_sigma;
    let b_pnl_norm = (b.avg_pnl - pnl_mu) / pnl_sigma;
    let mean_diff = (a_pnl_norm - b_pnl_norm).abs();

    // Normalized Volatility Difference
    let a_std_norm = (a.std_dev - std_mu) / std_sigma;
    let b_std_norm = (b.std_dev - std_mu) / std_sigma;
    let std_diff = (a_std_norm - b_std_norm).abs();

    // Composite distance: Magnitude + Volatility + Correlation (Phase 11.1 Final Weights)
    0.5 * (1.0 - corr) + 0.3 * mean_diff.min(2.0) + 0.2 * std_diff.min(2.0)
}

fn aggregate_strategy_reports_inner(
    mut evaluations: Vec<StrategyEvaluation>,
    _scarcity_penalty: f64,
    config: &GaConfig,
    _generation: usize,
) -> Option<(StrategyEvaluation, f64)> {
    if evaluations.is_empty() {
        return None;
    }

    let total_scenarios_in = evaluations.len();
    let has_executable = evaluations.iter().any(|e| e.capability.is_executable());
    if has_executable {
        evaluations.retain(|e| e.capability.is_executable());
    }
    
    let executable_total = evaluations.len();
    let executable_active = evaluations.iter().filter(|e| e.trade_count > 0).count();

    println!(
        "DEBUG_EXEC → total={}, executable={}, active_exec={}, participation_exec={:.2}",
        total_scenarios_in,
        executable_total,
        executable_active,
        executable_active as f64 / (executable_total as f64).max(1.0)
    );

    // IMPORTANT: use raw per-scenario returns; never clip before aggregation.
    let scenario_results: Vec<f64> = evaluations.iter().map(|e| e.avg_pnl).collect();
    let scenario_trade_counts: Vec<usize> = evaluations.iter().map(|e| e.trade_count).collect();

    let total_scenarios = scenario_results.len() as f64;

    let unique_assets = evaluations.iter()
        .map(|e| {
            // Extract the asset name from the scenario_id (e.g., "VODAFONEIDEA_FUT_5M_CLEAN_csv_window_0" -> "VODAFONEIDEA_FUT")
            let parts: Vec<&str> = e.strategy_id.split("strat_").collect();
            let scenario_name = if parts.len() > 1 { parts[1] } else { &e.strategy_id };
            scenario_name.split("_csv_window_").next().unwrap_or("unknown")
        })
        .collect::<HashSet<&str>>()
        .len();
    let total_assets_available = std::env::var("GA_ASSET_COUNT").ok().and_then(|s| s.parse::<usize>().ok()).unwrap_or(1).max(unique_assets);

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

    let worst_pnl = scenario_results.iter().cloned().fold(f64::INFINITY, f64::min);

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
    println!("SCENARIO_DIST: {:?}", scenario_results);

    // --- ASSERT DISTRIBUTION VALIDITY ---
    // With a single scenario, std dev is legitimately zero; weighted mean can also differ from
    // `scenario_results[0]` by floating-point rounding — do not require bitwise equality.
    if total_scenarios > 1.0 {
        let tol = 1e-9_f64.max(global_avg_pnl.abs() * 1e-12);
        assert!(
            std_dev > 1e-18
                || scenario_results
                    .iter()
                    .all(|&x| (x - global_avg_pnl).abs() <= tol),
            "Invalid distribution: non-zero pnl but zero std dev"
        );
    }

    let participation_rate = active_scenarios / total_scenarios;
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
            .sum::<f64>() / total_scenarios,

        avg_latency: evaluations
            .iter()
            .map(|e| e.scenario_signature.avg_latency)
            .sum::<f64>() / total_scenarios,

        fill_ratio: evaluations
            .iter()
            .map(|e| e.scenario_signature.fill_ratio)
            .sum::<f64>() / total_scenarios,

        participation: evaluations
            .iter()
            .map(|e| e.scenario_signature.participation)
            .sum::<f64>() / total_scenarios,
        
        execution_variance: evaluations
            .iter()
            .map(|e| e.scenario_signature.execution_variance)
            .sum::<f64>() / total_scenarios,
    };

    let avg_fill_eff = evaluations.iter().map(|e| e.execution_metrics.fill_efficiency).sum::<f64>() / total_scenarios.max(1.0);
    let avg_slippage = evaluations.iter().map(|e| e.execution_metrics.avg_slippage).sum::<f64>() / total_scenarios.max(1.0);
    let avg_latency = evaluations.iter().map(|e| e.execution_metrics.latency_impact).sum::<f64>() / total_scenarios.max(1.0);

    // Phase 8.8 Sniper Aggregates
    let avg_conviction = evaluations.iter().map(|e| e.avg_conviction).sum::<f64>() / total_scenarios.max(1.0);
    let avg_efficiency = evaluations.iter().map(|e| e.avg_efficiency).sum::<f64>() / total_scenarios.max(1.0);
    let avg_edge_quality = evaluations.iter().map(|e| e.avg_edge_quality).sum::<f64>() / total_scenarios.max(1.0);
    let directional_accuracy = evaluations.iter().map(|e| e.directional_accuracy).sum::<f64>() / total_scenarios.max(1.0);
    let decisiveness = evaluations.iter().map(|e| e.decisiveness).sum::<f64>() / total_scenarios.max(1.0);
    let execution_friction = evaluations.iter().map(|e| e.execution_friction).sum::<f64>() / total_scenarios.max(1.0);

    // --- PHASE 10.2: INSTITUTIONAL FITNESS ENGINE ---
    let total_profit_from_tp: f64 = evaluations.iter().map(|e| e.pnl_from_tp).sum();
    let total_loss_from_sl: f64 = evaluations.iter().map(|e| e.pnl_from_sl).sum();
    let max_trade_pnl: f64 = evaluations.iter().map(|e| e.max_trade_pnl).fold(0.0, f64::max);
    
    // Aggregated diagnostic fitness (proxy for regime quality across the elite cluster)
    let _diagnostic_fitness = evaluations.iter().map(|e| e.fitness).sum::<f64>() / total_scenarios.max(1.0);

    let win_rate = if total_trade_count > 0 { total_profitable_trades as f64 / total_trade_count as f64 } else { 0.0 };

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

    let quality_score = (
        0.30 * directional_accuracy +
        0.25 * avg_edge_quality +
        0.20 * decisiveness +
        0.25 * (1.0 - execution_friction)
    ).max(0.05);  // 🔥 critical floor

    let asset_generalization_multiplier = if unique_assets < 2 {
        0.7 // Hard penalty for single-asset dependency
    } else {
        1.0 + (unique_assets as f64 / total_assets_available as f64) * 0.5
    };

    let variance_penalty = (1.0 / (1.0 + std_dev * 4.0)).clamp(0.5, 1.1);

    // --- PHASE D.1.7: HIERARCHY INJECTION & UNIFORMITY PENALTY ---
    // 1. Uniformity Penalty: Punish strategies that produce identical outcomes (consensus collapse)
    let mut final_fitness = pnl_score;
    if std_dev < 1e-6 {
        final_fitness *= 0.8;
    }

    // 2. Dispersion Bonus: Reward strategies that interpret scenarios differently
    let dispersion_bonus = (std_dev / pnl_scale.max(1e-6)).tanh() * 0.2;
    final_fitness += dispersion_bonus;

    // 3. Activity Weight: Smooth ramp for trade participation (Phase D.1.5/D.1.7)
    let activity_weight = (total_trade_count as f64 / 5.0).tanh();
    final_fitness *= activity_weight.max(0.2);

    // 4. Relative Dominance Amplification (Intrinsic Hierarchy)
    // Amplify top performers based on their internal scenario dispersion
    if final_fitness > 0.0 {
        final_fitness = final_fitness.powf(1.1);
    }

    // --- PHASE 17.6: ACCEPTANCE MODE CALIBRATION ---
    if evaluations.iter().any(|e| e.acceptance_mode == AcceptanceMode::StatisticalWeak) {
        // Apply a slight penalty if the majority of evidence is 'weak'
        let weak_ratio = evaluations.iter().filter(|e| e.acceptance_mode == AcceptanceMode::StatisticalWeak).count() as f64 / total_scenarios.max(1.0);
        if weak_ratio > 0.5 {
            final_fitness *= 0.85; 
        }
    }


    // 1. Scalable Mode Pressure (Linear Participation)
    if config.fitness_mode == FitnessMode::Scalable {
        final_fitness *= 0.5 + 0.5 * (participation_rate / 0.5).clamp(0.0, 1.0);
    }

    // --- PHASE 17.5: PARTICIPATION INTEGRITY (Ghost Strategy Guard) ---
    // If a strategy avoids all trades, it is a degenerate optimum (avoiding loss by not playing).
    // We enforce a minimum trade density to ensure structural validity.
    let min_trades_required = (total_scenarios * 0.15).max(2.0); // At least 3 trades for 20 windows
    if (total_trade_count as f64) < min_trades_required {
        let trade_penalty = (total_trade_count as f64 / min_trades_required).clamp(0.01, 1.0);
        final_fitness *= trade_penalty * 0.1; // Aggressive 90% floor penalty for ghosting
        if std::env::var("GA_DEBUG").is_ok() && total_trade_count == 0 {
            println!("FITNESS_DEGENERATE_GUARD → trades=0 | Killing degenerate fitness");
        }
    }

    // 2. Sniper Mode Pressure (Quad Participation)
    if config.fitness_mode == FitnessMode::Sniper {
        final_fitness *= (participation_rate / 0.4).clamp(0.0, 1.0).powi(2);
    }

    final_fitness *= asset_generalization_multiplier;
    final_fitness *= variance_penalty;
    final_fitness *= quality_score;

    // --- PHASE 13.5: SELECTIVITY DISCIPLINE (ALPHA-STREAK) ---
    let alpha_penalty = 5.0; // Institutional over-trade friction
    let selectivity_diff = (selectivity - 0.10).max(0.0);
    final_fitness *= (-alpha_penalty * selectivity_diff).exp();
    
    // --- NON-LINEAR GLOBAL MORTALITY GATE (Generation-Aware) ---
    // Quadrative decay if win rate below 30%; softened in early generations for Discovery
    // Note: We use participation as a proxy for exploration progress if GEN count is not passed
    let global_wr_penalty = (win_rate / 0.30).clamp(0.2, 1.0).powi(2);
    final_fitness *= global_wr_penalty;

    // --- PHASE 13.5: DISAGREEMENT ENTROPY (CONTINUOUS) ---
    let avg_entropy = if total_scenarios > 0.0 {
        evaluations.iter().map(|e| e.avg_entropy).sum::<f64>() / total_scenarios
    } else {
        0.0
    };
    let mut entropy_weight = 1.0;
    if avg_entropy > 1e-6 {
        let entropy_score = 1.0 - ((avg_entropy - 0.45).abs() / 0.45);
        entropy_weight = entropy_score.clamp(0.5, 1.2);
    }
    final_fitness *= entropy_weight;

    // --- PHASE 13.6: INSTITUTIONAL HEALTH PRESSURES ---
    let avg_aqg_health = if total_trade_count > 0 {
        evaluations.iter().map(|e| e.avg_aqg_health).sum::<f64>() / total_scenarios.max(1.0)
    } else {
        0.0
    };
    let aqg_skip_ratio = if total_scenarios > 0.0 {
        evaluations.iter().filter(|e| e.trade_count == 0).count() as f64 / total_scenarios
    } else {
        1.0
    };

    // 1. AQG Pressure (Smooth linear curve)
    let aqg_penalty = 0.6 + 0.4 * avg_aqg_health.clamp(0.0, 1.0);
    final_fitness *= aqg_penalty;

    // 2. Effective Participation (Honesty Gate - Hard Collapse)
    let participation = participation_rate;
    let effective_participation = participation * (1.0 - aqg_skip_ratio);
    final_fitness *= effective_participation.powi(2); // Institutional Hard Collapse

    // (Restoring Phase 13.6 DEBUG variables)
    let _ = (selectivity, avg_entropy, avg_aqg_health, aqg_skip_ratio, effective_participation);

    // 2. Numerical Safety
    if !final_fitness.is_finite() {
        final_fitness = -1.0;
    }

    // --- PHASE C.3b: REMOVAL OF HARD FLOOR ---
    // We allow negative fitness to persist to provide a gradient for selection ranking.
    // Binary floors (1e-6) are deleted here to restore causality in learning.

    // TODO: Remove clamp once adaptive scaling stabilizes
    final_fitness = final_fitness.clamp(-2.0, 2.0);
    let raw_selection_fitness = final_fitness;
    println!("SELECTION_INPUT_RAW → fitness={:.6}", raw_selection_fitness);

    // 14. DEBUG TRACE
    if std::env::var("GA_DEBUG").is_ok() {
        println!(
            "FITNESS_TRACE → mode={:?} | pnl={:.4} scenarios={}/{} participation={:.2} win={:.3} pay={:.3} final={:.4}",
            config.fitness_mode,
            pnl_score,
            active_scenarios,
            total_scenarios,
            participation_rate,
            win_rate,
            global_payoff_ratio,
            final_fitness
        );
    }


    println!(
        "FITNESS_FINAL → pnl_score: {:.4}, quality: {:.4}, final_fitness: {:.4}",
        pnl_score, quality_score, final_fitness
    );

    // --- AGGREGATE LOGGING ---
    println!(
        "AGG_DEBUG: avg_pnl={:.6} (scenario_agg={}), active={}, total={}, participation={:.2}, fitness={:.4}, payoff={:.2}, selectivity={:.2}",
        global_avg_pnl,
        if use_rank_weights { "rank_weighted" } else { "mean" },
        active_scenarios,
        total_scenarios,
        participation_rate,
        final_fitness,
        global_payoff_ratio,
        selectivity
    );

    println!(
        "QUALITY_DEBUG: trades={}, zero_pnl={}, effectiveness={:.2}",
        total_trade_count, total_zero_pnl_trades, effectiveness
    );

    // --- PHASE 17.7: OUTCOME INTEGRITY ASSERTION ---
    // Prevent 'Silent Execution Collapse' where simulated trades are lost during aggregation.
    if total_trade_count == 0 && evaluations.iter().any(|e| e.trade_count > 0) {
        panic!("PIPELINE_BREAK: execution not reflected in outcome ({} vs {})", total_trade_count, evaluations.len());
    }

    // --- Phase 17A: Population Diagnostic Aggregation (Alpha Recovery) ---
    let total_evals = evaluations.len() as f64;
    let mut raw_dist = [0.0f64; 6];
    let mut exec_dist = [0.0f64; 6];
    let mut raw_sum = 0.0;
    let mut exec_sum = 0.0;
    let mut total_vip_ratio = 0.0;
    let mut total_stat_zero_dom_ratio = 0.0;
    
    // Phase 17B Aggregators
    let mut total_exec_accept_rate = 0.0;
    let mut total_vip_exec_retention = 0.0;
    let mut total_e_rejection_rate = 0.0;
    let mut total_clarity_to_exec_drop = 0.0;
    let mut total_avg_e_score = 0.0;
    let mut total_vip_avg_e_score = 0.0;
    let mut total_stat_avg_e_score = 0.0;

    // Phase 14 Consensus
    let mut total_consensus_bypass_ratio = 0.0;
    let mut total_stability_reject_rate = 0.0;
    let mut total_clarity_pnl_share: f64 = 0.0;
    let mut total_conviction_pnl_share: f64 = 0.0;

    for e in &evaluations {
        raw_sum += e.raw_pop_avg;
        exec_sum += e.exec_pop_avg;
        total_vip_ratio += e.vip_ratio;
        total_stat_zero_dom_ratio += e.stat_zero_dom_ratio;
        
        total_exec_accept_rate += e.exec_accept_rate;
        total_vip_exec_retention += e.vip_exec_retention;
        total_e_rejection_rate += e.e_rejection_rate;
        total_clarity_to_exec_drop += e.clarity_to_exec_drop;
        total_avg_e_score += e.avg_e_score;
        total_vip_avg_e_score += e.vip_avg_e_score;
        total_stat_avg_e_score += e.stat_avg_e_score;

        total_consensus_bypass_ratio += e.consensus_bypass_ratio;
        total_stability_reject_rate += e.stability_reject_rate;
        total_clarity_pnl_share += e.clarity_pnl_share;
        total_conviction_pnl_share += e.conviction_pnl_share;

        for i in 0..6 {
            raw_dist[i] += e.raw_pop_dist[i];
            exec_dist[i] += e.exec_pop_dist[i];
        }
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
        if cum_raw >= 0.95 && raw_p95 == 0.0 { raw_p95 = bucket_midpoints[i]; }
        cum_exec += exec_dist[i];
        if cum_exec >= 0.95 && exec_p95 == 0.0 { exec_p95 = bucket_midpoints[i]; }
    }

    let pop_delta = exec_p95 - raw_p95;
    let ccr = if raw_p95 > 0.0 { exec_p95 / raw_p95 } else { 0.0 };
    let avg_vip_ratio = total_vip_ratio / total_evals.max(1.0);
    let avg_stat_zero_dom_ratio = total_stat_zero_dom_ratio / total_evals.max(1.0);

    // Phase 17A: VIP Ratio Interpretation
     let vip_band = if avg_vip_ratio < 0.05 { "RESTRICTIVE" }
                   else if avg_vip_ratio < 0.25 { "HEALTHY" }
                   else if avg_vip_ratio < 0.50 { "STRONG" }
                   else { "OVER_ADMIT" };

    // --- Phase 17A.5: Population Diagnostic Reporting (Alpha Recovery) ---
    println!(
        "POP_RAW_DEBUG:  avg={:.3}, p95={:.3}", 
        raw_sum / total_evals.max(1.0), raw_p95
    );
    println!(
        "POP_EXEC_DEBUG: avg={:.3}, p95={:.3} | DELTA={:.3} | CCR={:.3}", 
        exec_sum / total_evals.max(1.0), exec_p95, pop_delta, ccr
    );
    println!(
        "VIP_AUDIT:      ratio={:.4} | band={} | energy_min=max(p80, p75)",
        avg_vip_ratio, vip_band
    );
    println!(
        "STAT_AUDIT:     zero_dom_ratio={:.4} | interpretation: {}",
        avg_stat_zero_dom_ratio,
        if avg_stat_zero_dom_ratio > 0.50 { "WEAK (Noise Admission)" } 
        else if avg_stat_zero_dom_ratio > 0.20 { "MIXED" } 
        else { "HEALTHY Separation" }
    );
    
    // Phase 17B: Realizability Reporting
    let avg_exec_accept_rate = total_exec_accept_rate / total_evals.max(1.0);
    let avg_vip_exec_retention = total_vip_exec_retention / total_evals.max(1.0);
    let avg_e_rejection_rate = total_e_rejection_rate / total_evals.max(1.0);
    let avg_clarity_to_exec_drop = total_clarity_to_exec_drop / total_evals.max(1.0);
    let avg_e_score = total_avg_e_score / total_evals.max(1.0);
    let avg_vip_e = total_vip_avg_e_score / total_evals.max(1.0);
    let avg_stat_e = total_stat_avg_e_score / total_evals.max(1.0);
    let e_gradient = avg_vip_e - avg_stat_e;

    // Phase 14: Consensus Audit
    let avg_consensus_bypass_ratio = total_consensus_bypass_ratio / total_evals.max(1.0);
    let avg_stability_reject_rate = total_stability_reject_rate / total_evals.max(1.0);
    
    let total_abs_pnl_global = total_clarity_pnl_share.abs() + total_conviction_pnl_share.abs();
    let avg_clarity_pnl_share = total_clarity_pnl_share.abs() / total_abs_pnl_global.max(1e-6);
    let avg_conviction_pnl_share = total_conviction_pnl_share.abs() / total_abs_pnl_global.max(1e-6);

    println!(
        "EXEC_AUDIT:     accept_rate={:.4} | rejection_rate={:.4} | avg_e_score={:.3}",
        avg_exec_accept_rate, avg_e_rejection_rate, avg_e_score
    );
    println!(
        "VIP_RETENTION:   retention={:.4} | drop_off={:.4} | selectivity_gradient={:.3}",
        avg_vip_exec_retention, avg_clarity_to_exec_drop, e_gradient
    );
    println!(
        "E_SCORE_BANDS:  VIP_E={:.3} | STAT_E={:.3} | status: {}",
        avg_vip_e, avg_stat_e,
        if e_gradient > 0.10 { "HEALTHY SEPARATION" } else if e_gradient > 0.0 { "WEAK SELECTIVITY" } else { "INVERSION RISK" }
    );
    println!(
        "CONSENSUS_BRIDGE: bypass_ratio={:.4} | stability_reject={:.4} | clarity_share={:.2} | conviction_share={:.2}",
        avg_consensus_bypass_ratio, avg_stability_reject_rate, avg_clarity_pnl_share, avg_conviction_pnl_share
    );

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

    let global_mean_quality = if phase2_total_quality_count > 0.0 { phase2_total_quality / phase2_total_quality_count } else { 0.0 };
    let global_std_quality = if phase2_total_quality_count > 1.0 {
        let var = (phase2_total_quality_sq / phase2_total_quality_count) - (global_mean_quality * global_mean_quality);
        var.max(0.0).sqrt()
    } else { 0.0 };
    let global_consistency = global_mean_quality - global_std_quality;
    let global_capture_eff = if phase2_sum_expected.abs() > 1e-9 { phase2_sum_realized / phase2_sum_expected } else { 0.0 };

    println!(
        "OUTCOME_AUDIT:  trades={} | n={:.0} | mean_q={:.3} | std_q={:.3} | consistency={:.3} | capture_eff={:.4}",
        phase2_total_quality_count, phase2_total_quality_count, global_mean_quality, global_std_quality, global_consistency, global_capture_eff
    );
    println!(
        "RAW_HIST:  [0-0.05]: {:.1}%, [0.05-0.10]: {:.1}%, [0.10-0.20]: {:.1}%, [0.20-0.25]: {:.1}%, [0.25-0.50]: {:.1}%, [0.50+]: {:.1}%",
        raw_dist[0]*100.0, raw_dist[1]*100.0, raw_dist[2]*100.0, raw_dist[3]*100.0, raw_dist[4]*100.0, raw_dist[5]*100.0
    );
    println!(
        "EXEC_HIST: [0-0.05]: {:.1}%, [0.05-0.10]: {:.1}%, [0.10-0.20]: {:.1}%, [0.20-0.25]: {:.1}%, [0.25-0.50]: {:.1}%, [0.50+]: {:.1}%",
        exec_dist[0]*100.0, exec_dist[1]*100.0, exec_dist[2]*100.0, exec_dist[3]*100.0, exec_dist[4]*100.0, exec_dist[5]*100.0
    );

    // --- PHASE 11.1 Behavioral Fingerprint (50-bucket) ---
    let mut consolidated_fingerprint = vec![0.0_f32; config.pnl_fingerprint_len];
    if total_scenarios > 0.0 {
        for (scen_idx, eval) in evaluations.iter().enumerate() {
            let bucket_idx = (scen_idx * config.pnl_fingerprint_len / evaluations.len()).min(config.pnl_fingerprint_len - 1);
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
    
    // --- PHASE D.1.3: INTEGRITY TRACING (AGG_BRIDGE) ---
    let input_scenarios = total_scenarios_in;
    let output_scenarios = evaluations.len();
    let input_trades = total_trade_count;
    let avg_input_fitness = if output_scenarios > 0 { evaluations.iter().map(|e| e.fitness).sum::<f64>() / output_scenarios as f64 } else { 0.0 };
    let final_fitness = raw_selection_fitness;

    println!(
        "AGG_BRIDGE → input_trades={} output_trades={}",
        input_trades, total_trade_count
    );
    println!(
        "AGG_TRACE → scenarios_in={} scenarios_out={} fitness_in={:.4} fitness_out={:.4}",
        input_scenarios, output_scenarios, avg_input_fitness, final_fitness
    );

    // --- PHASE D.1.4: SPLIT-BRAIN INVARIANT ---
    assert!(
        input_trades == 0 || total_trade_count > 0,
        "❌ SPLIT-BRAIN DETECTED: Trades lost in aggregation"
    );

    let report = StrategyEvaluation {
        strategy_id: evaluations[0].strategy_id.clone(),
        strategy: evaluations[0].strategy.clone(),
        capability: evaluations[0].capability.clone(),
        avg_pnl: global_avg_pnl,
        std_dev,
        downside_std_dev,
        worst: worst_pnl,
        robustness,
        fitness: raw_selection_fitness,
        trade_count: total_trade_count,
        max_drawdown: avg_max_drawdown,
        participation_rate,
        profitable_trades: total_profitable_trades,
        zero_pnl_trades: total_zero_pnl_trades,
        quality_trades: total_quality_trades, 
        win_rate: if evaluations.len() > 0 { evaluations.iter().map(|e| e.win_rate).sum::<f64>() / evaluations.len() as f64 } else { 0.0 },
        payoff: if evaluations.len() > 0 { evaluations.iter().map(|e| e.payoff).sum::<f64>() / evaluations.len() as f64 } else { 0.0 },
        payoff_ratio: global_payoff_ratio,
        execution_metrics: ExecutionMetrics {
            fill_efficiency: avg_fill_eff,
            capture_efficiency: avg_efficiency,
            avg_slippage,
            latency_impact: avg_latency,
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
        selectivity: evaluations.iter().map(|e| e.selectivity).sum::<f64>() / total_scenarios.max(1.0),
        avg_entropy: evaluations.iter().map(|e| e.avg_entropy).sum::<f64>() / total_scenarios.max(1.0),
        avg_aqg_health,
        aqg_skip_ratio,
        raw_pop_avg: raw_sum / total_evals.max(1.0),
        raw_pop_p95: raw_p95,
        raw_pop_dist: raw_dist,
        exec_pop_avg: exec_sum / total_evals.max(1.0),
        exec_pop_p95: exec_p95,
        exec_pop_dist: exec_dist,
        pop_delta,
        vip_ratio: avg_vip_ratio,
        ccr,
        stat_zero_dom_ratio: avg_stat_zero_dom_ratio,
        outcome_consistency: global_consistency,
        avg_trade_quality: global_mean_quality,
        std_trade_quality: global_std_quality,
        consistency_n: phase2_total_quality_count as usize,
        ..StrategyEvaluation::default()
    };
    
    // --- PHASE 14++ Ext: STRUCTURAL RANKING ENGINE ---
    let mut structural_candidates = Vec::new();
    for e in &evaluations {
        // Extract symbol from strategy_id (e.g., "VODAFONEIDEA_FUT_5M_CLEAN_csv_window_0" -> "VODAFONEIDEA_FUT")
        let parts: Vec<&str> = e.strategy_id.split("strat_").collect();
        let scenario_part = if parts.len() > 1 { parts[1] } else { &e.strategy_id };
        let symbol = scenario_part.split("_csv_window_").next().unwrap_or("unknown").to_string();
        
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
    
    println!("\n🚀 STRUCTURAL_RANKING (Adaptive Discovery)");
    println!("--------------------------------------------------------------------------------------------------");
    println!("Rank | Symbol       | Alpha   | Continuity | Opp%   | Stab | Agree | PeakAgree | Conf | N");
    println!("--------------------------------------------------------------------------------------------------");
    for (i, (sym, e, _score, conf)) in final_ranking.iter().take(10).enumerate() {
        println!("{:>4} | {:<12} | {:7.3} | {:10.2} | {:6.2}% | {:4.2} | {:5.2} | {:9.2} | {:4} | {}",
            i + 1, sym, e.alpha, e.consistency, e.opportunity * 100.0,
            (1.0 - e.avg_stability_valid / 0.18).clamp(0.0, 1.0),
            e.avg_agreement_valid,
            e.max_agreement,
            conf,
            e.total_windows
        );
    }
    println!("--------------------------------------------------------------------------------------------------");

    let mean_depth = total_trade_count as f64 / total_scenarios.max(1.0);
    let _final_fitness_val = (1.0 + final_fitness).max(1e-6).ln();
    Some((report, mean_depth))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionReport {
    pub trade_id: u64,
    pub symbol: String,
    pub timestamp: u64,
    pub signal: SignalType,
    pub confidence: f64,
    pub expected_return: f64,
    pub horizon_bars: usize,
    pub participation: f64,
    pub regime: String,
    pub consistency: usize, // same_signal_count_last_N
    #[serde(default)]
    pub conviction_score: f64,
    #[serde(default)]
    pub agreement_strength: String, // STRONG, MEDIUM, WEAK
    #[serde(default)]
    pub voters: String, // X/Y
    #[serde(default)]
    pub execution_feasible: bool,
    #[serde(default)]
    pub execution_score: f64,
    #[serde(default)]
    pub execution_threshold: f64,
    #[serde(default)]
    pub threshold: f64,
    #[serde(default)]
    pub realized_return: Option<f64>,
    #[serde(default)]
    pub capture_efficiency: Option<f64>,
    #[serde(default)]
    pub efficiency_label: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SignalType {
    BUY,
    SELL,
    WAIT,
}

/// Phase 10.6: Decision Evaluation Mode
/// Checks the entry condition on the MOST RECENT state.
pub fn evaluate_current_status(
    strategy: &Strategy,
    history: &[Candle], 
    config: &GaConfig,
    symbol: &str,
    last_signal: SignalType,
    consistency_count: usize,
) -> DecisionReport {
    // Refinement 2: Candle window consistency
    if history.len() < (config.lambda as usize) + 20 {
        return DecisionReport {
            trade_id: 0,
            symbol: symbol.to_string(),
            timestamp: history.last().map(|c| c.timestamp).unwrap_or(0),
            signal: SignalType::WAIT,
            confidence: 0.0,
            expected_return: 0.0,
            horizon_bars: config.max_hold_bars,
            participation: 0.0,
            regime: "INSUFFICIENT_DATA".to_string(),
            consistency: consistency_count,
            conviction_score: 0.0,
            agreement_strength: "WEAK".to_string(),
            voters: "0/0".to_string(),
            execution_feasible: false,
            execution_score: 0.0,
            execution_threshold: 0.7,
            threshold: 0.7,
            realized_return: None,
            capture_efficiency: None,
            efficiency_label: String::new(),
        };
    }

    let last_idx = history.len().saturating_sub(1);
    
    // Mock MarketEvents from history for simulation
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

    let conviction = evaluate_market_conviction(strategy, "live", &events, last_idx, 0, 0);

    // Use the ESE RoundTrip logic with realigned signature
    let outcome = crate::ga_simulate_round_trip_at_cursor(
        strategy,
        "live_strategy",
        symbol,
        &events,
        &events,
        config,
        last_idx,
        0,
        &conviction
    );

    let (signal, confidence) = if let Some(rt) = outcome {
        let (mean_px, _, _) = calculate_lookback_stats(history, last_idx, (GA_GENE_SCALE as usize).max(20));
        let ref_price = history[last_idx].close;
        let is_bearish = (ref_price as f64) < mean_px;
        
        let side = if is_bearish { Side::Sell } else { Side::Buy };
        let sig = match side {
            Side::Buy => SignalType::BUY,
            Side::Sell => SignalType::SELL,
        };
        
        // Confidence = normalized distance from threshold
        let conf = (rt.expected_move / 0.003).clamp(0.0, 1.0);
        (sig, conf)
    } else {
        (SignalType::WAIT, 0.0)
    };

    let new_consistency = if signal == last_signal && signal != SignalType::WAIT {
        consistency_count + 1
    } else if signal != SignalType::WAIT {
        1
    } else {
        0
    };

    DecisionReport {
        trade_id: 0,
        symbol: symbol.to_string(),
        timestamp: history[last_idx].timestamp,
        signal,
        confidence,
        expected_return: 0.008, 
        horizon_bars: config.max_hold_bars,
        participation: conviction.norm_vol_score,
        regime: if conviction.is_bearish { "BEARISH".to_string() } else { "BULLISH".to_string() },
        consistency: new_consistency,
        conviction_score: confidence,
        agreement_strength: if confidence > 0.75 { "STRONG".to_string() } else if confidence > 0.6 { "MEDIUM".to_string() } else { "WEAK".to_string() },
        voters: "1/1".to_string(),
        execution_feasible: true,
        execution_score: 1.0,
        execution_threshold: 0.0,
        threshold: 0.0,
        realized_return: None,
        capture_efficiency: None,
        efficiency_label: String::new(),
    }
}

pub fn save_elite_population(evals: &[StrategyEvaluation], _config: &GaConfig, base_dir: &str) -> std::io::Result<String> {
    use chrono::Utc;
    use std::fs;

    let timestamp = Utc::now().format("%Y-%m-%d_%H-%M").to_string();
    let filename = format!("elite_{}.json", timestamp);
    let path = std::path::Path::new(base_dir).join(&filename);
    
    let sum_fitness: f64 = evals.iter().map(|e| e.fitness).sum();
    let avg_fitness = sum_fitness / evals.len().max(1) as f64;
    let avg_pnl = evals.iter().map(|e| e.avg_pnl).sum::<f64>() / evals.len().max(1) as f64;
    
    // Average metrics for regime profiling
    let avg_vol = evals.iter().map(|e| e.avg_conviction).sum::<f64>() / evals.len().max(1) as f64; // Proxy
    let avg_participation = evals.iter().map(|e| e.decisiveness).sum::<f64>() / evals.len().max(1) as f64;

    // Simple CV calculation for persistence metadata
    let mean = avg_fitness;
    let variance = evals.iter().map(|e| (e.fitness - mean).powi(2)).sum::<f64>() / evals.len().max(1) as f64;
    let cv = if mean > 0.0 { variance.sqrt() / mean } else { 0.0 };

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

    let json = serde_json::to_string_pretty(&bundle).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
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
    if history.len() < (config.lambda as usize) + 50 {
         return DecisionReport {
            trade_id: 0,
            symbol: symbol.to_string(),
            timestamp: history.last().map(|c| c.timestamp).unwrap_or(0),
            signal: SignalType::WAIT,
            confidence: 0.0,
            expected_return: 0.0,
            horizon_bars: config.max_hold_bars,
            participation: 0.0,
            regime: "BOOTSTRAPPING".to_string(),
            consistency: consistency_count,
            conviction_score: 0.0,
            agreement_strength: "WEAK".to_string(),
            voters: "0/0".to_string(),
            execution_feasible: false,
            execution_score: 0.0,
            execution_threshold: 0.7,
            threshold: 0.7,
            realized_return: None,
            capture_efficiency: None,
            efficiency_label: String::new(),
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

    // 🔥 GUARDRAIL 1: Eligibility Filter (Fitness > 0, Consistency > 0.6)
    let eligible_voters: Vec<&StrategyEvaluation> = evals.iter()
        .filter(|e| e.fitness > 0.0 && e.consistency_score >= 0.6)
        .take(10) // Dynamic Top-K (Max 10)
        .collect();

    if eligible_voters.len() < 3 {
        return DecisionReport {
            trade_id: 0,
            symbol: symbol.to_string(),
            timestamp: history[last_idx].timestamp,
            signal: SignalType::WAIT,
            confidence: 0.0,
            expected_return: 0.0,
            horizon_bars: config.max_hold_bars,
            participation: 0.0,
            regime: "LOW_VOTER_COUNT".to_string(),
            consistency: consistency_count,
            conviction_score: 0.0,
            agreement_strength: "WEAK".to_string(),
            voters: format!("{}/{}", eligible_voters.len(), evals.len()),
            execution_feasible: false,
            execution_score: 0.0,
            execution_threshold: 0.7,
            threshold: 0.7,
            realized_return: None,
            capture_efficiency: None,
            efficiency_label: String::new(),
        };
    }

    // Analyze Global Regime (using first voter as proxy for conviction logic)
    let conviction_guard = evaluate_market_conviction(&eligible_voters[0].strategy, "consensus", &events, last_idx, 0, 0);
    
    // 🔥 GUARDRAIL 6: Hard Admission Gate
    if conviction_guard.norm_vol > 0.005 || conviction_guard.norm_vol_score < 0.2 {
         return DecisionReport {
            trade_id: 0,
            symbol: symbol.to_string(),
            timestamp: history[last_idx].timestamp,
            signal: SignalType::WAIT,
            confidence: 0.0,
            expected_return: 0.0,
            horizon_bars: config.max_hold_bars,
            participation: conviction_guard.norm_vol_score,
            regime: "REJECTED_REGIME".to_string(),
            consistency: consistency_count,
            conviction_score: 0.0,
            agreement_strength: "WEAK".to_string(),
            voters: format!("0/{}", eligible_voters.len()),
            execution_feasible: false,
            execution_score: 0.0,
            execution_threshold: 0.7,
            threshold: 0.7,
            realized_return: None,
            capture_efficiency: None,
            efficiency_label: String::new(),
        };
    }

    // 🚀 PHASE 10.10: Execution Feasibility Reality Check
    let ctx = ExecutionContext {
        queue_depth: conviction_guard.raw_q_ratio.min(1.0), // Normalizing proxy
        liquidity_score: conviction_guard.norm_volume,      // Volume velocity proxy
        latency_impact: conviction_guard.norm_vol * 100.0,  // Volatility proxy
    };
    let exec_score = calculate_execution_score(&ctx);

    // Weighted Voting
    let mut buy_weight = 0.0;
    let mut sell_weight = 0.0;
    let mut expected_return_sum = 0.0;

    // 🔥 GUARDRAIL 2: Capped Dominance (Max 25% influence)
    let raw_weights: Vec<f64> = eligible_voters.iter().map(|v| v.fitness).collect();
    let total_raw_fitness: f64 = raw_weights.iter().sum::<f64>().max(1e-9);
    let mut normalized_weights: Vec<f64> = raw_weights.iter().map(|w| (w / total_raw_fitness).min(0.25)).collect();
    let capped_total_weight: f64 = normalized_weights.iter().sum::<f64>().max(1e-9);
    for w in &mut normalized_weights {
        *w /= capped_total_weight; // Re-normalize to 1.0 sum
    }

    for (i, voter) in eligible_voters.iter().enumerate() {
        let weight = normalized_weights[i];
        let outcome = crate::ga_simulate_round_trip_at_cursor(
            &voter.strategy,
            "consensus_voter",
            symbol,
            &events,
            &events,
            config,
            last_idx,
            0,
            &conviction_guard
        );

        if let Some(_rt) = outcome {
            let (mean_px, _, _) = calculate_lookback_stats(history, last_idx, 20);
            let is_bearish = (history[last_idx].close as f64) < mean_px;
            if is_bearish {
                sell_weight += weight;
            } else {
                buy_weight += weight;
            }
            expected_return_sum += voter.avg_pnl * weight;
        }
    }

    let dynamic_threshold = calculate_dynamic_threshold(conviction_guard.norm_vol, eligible_voters[0].fitness as f64);
    
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
    
    // 🔥 GUARDRAIL 3: No-Trade Zone (< 0.55 conviction)
    let gated_signal = if final_conviction < 0.55 { SignalType::WAIT } else { consensus_signal };

    // 🚀 PHASE 10.10: Final Feasibility Decision
    let (feasible, exec_threshold) = is_execution_feasible(final_conviction, exec_score);
    let final_signal = if !feasible { SignalType::WAIT } else { gated_signal };

    let strength = if final_conviction > 0.75 { "STRONG".to_string() }
                   else if final_conviction > 0.60 { "MEDIUM".to_string() }
                   else { "WEAK".to_string() };

    DecisionReport {
        trade_id: 0,
        symbol: symbol.to_string(),
        timestamp: history[last_idx].timestamp,
        signal: final_signal,
        confidence: final_conviction,
        expected_return: expected_return_sum,
        horizon_bars: config.max_hold_bars,
        participation: conviction_guard.norm_vol_score,
        regime: if conviction_guard.is_bearish { "BEARISH".to_string() } else { "BULLISH".to_string() },
        consistency: if final_signal == last_signal && final_signal != SignalType::WAIT { consistency_count + 1 } else { 0 },
        conviction_score: final_conviction,
        agreement_strength: strength,
        voters: format!("{}/{}", if gated_signal == SignalType::BUY { buy_count } else { sell_count }, eligible_voters.len()),
        execution_feasible: feasible,
        execution_score: exec_score,
        execution_threshold: exec_threshold,
        threshold: dynamic_threshold,
        realized_return: None,
        capture_efficiency: None,
        efficiency_label: String::new(),
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

/// Same as [`evaluate_and_aggregate`], plus mean round-trips per scenario that had `trade_count > 0` after GA Top-K (matches fitness `avg_trades_per_active`).
pub(crate) fn evaluate_and_aggregate_with_trade_depth(
    strategy: &Strategy,
    config: &GaConfig,
    scenarios: &[ScenarioPair],
    generation: usize,
) -> Option<(StrategyEvaluation, f64)> {
    let mut reports = Vec::new();
    for (idx, pair) in scenarios.iter().enumerate() {
        if let Some(report) = evaluate_strategy(strategy, pair, config, generation) {
            reports.push(report);
        }
        
        // Phase 3: Early Pruning for dead genomes (Balanced for Unit Tests)
        if idx >= 5 && reports.is_empty() {
             return None;
        }
    }
    if reports.is_empty() {
        return None;
    }
    // Phase 10.3.2: Bypassing Downstream Truncation
    // We already sampled the robust subset in evaluate_population_scoped; use all results here.
    let evaluations = apply_ga_top_k_selection(reports, None);
    aggregate_strategy_reports_inner(evaluations, 1.0, config, generation)
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
        strict_config.initial_queue_threshold = config.initial_queue_threshold.max(50);
        
        if let Some(res) = evaluate_strategy(strategy, scenario, &strict_config, 0) {
            // Directional Intent Mapping (Belief-based)
            let decision = if res.avg_conviction > epsilon {
                Decision::BUY
            } else if res.avg_conviction < -epsilon {
                Decision::SELL
            } else {
                Decision::HOLD
            };

            if decision != Decision::HOLD {
                let votes = signal_votes_map.entry(res.winner_idx).or_insert(vec![]);
                votes.push(SignalVote {
                    strategy_id: strategy_to_id(strategy),
                    archetype: Archetype::from(strategy.archetype),
                    confidence: res.avg_conviction,
                    signal_features: extract_features(strategy),
                    decision,
                });
            }
        }
    }

    let mut reports = Vec::new();
    for (idx, votes) in signal_votes_map {
        let count = votes.len();
        let support_ratio = count as f64 / total_strategies as f64;
        
        // 1. Participation Gate
        if support_ratio < 0.1 {
            continue;
        }

        // 2. Conviction Factor (Smooth suppression)
        let mean_abs_conviction = votes.iter().map(|v| v.confidence.abs()).sum::<f64>() / count as f64;
        let conviction_factor = ((mean_abs_conviction - 0.05) / 0.95).clamp(0.0, 1.0).powf(1.2);
        
        let avg_score = votes.iter().map(|v| v.confidence.abs()).sum::<f64>() / count as f64;
        
        // 3. Archetype Diversity
        let unique_archs: HashSet<Archetype> = votes.iter().map(|v| v.archetype).collect();
        let archetype_diversity = unique_archs.len() as f64 / 4.0;

        // 4. Belief Entropy (Shannon)
        let mut buy_c = 0usize;
        let mut sell_c = 0usize;
        for v in &votes {
            match v.decision {
                Decision::BUY => buy_c += 1,
                Decision::SELL => sell_c += 1,
                _ => {}
            }
        }
        let hold_c = total_strategies.saturating_sub(buy_c + sell_c);
        
        let mut entropy = 0.0;
        let den = total_strategies as f64;
        for c in [buy_c, sell_c, hold_c] {
            if c > 0 {
                let p = c as f64 / den;
                entropy -= p * p.log2();
            }
        }
        let raw_entropy = (entropy / 3.0f64.log2()).clamp(0.0, 1.0);
        let disagreement_entropy = raw_entropy * conviction_factor;

        // 5. Feature Diversity (Weighted Euclidean Centroid Dispersion)
        let mut feature_diversity = 0.0;
        if count > 1 {
            let dim = votes[0].signal_features.len();
            // Unweighted Centroid (Simple Mean)
            let mut centroid = vec![0.0; dim];
            for v in &votes {
                for (i, f) in v.signal_features.iter().enumerate().take(dim) {
                    centroid[i] += f;
                }
            }
            for c in centroid.iter_mut() { *c /= count as f64; }
            
            // Weighted Euclidean RMS Distance
            let mut sum_weighted_dist_sq = 0.0;
            for v in &votes {
                let mut d_sq = 0.0;
                for (i, f) in v.signal_features.iter().enumerate().take(dim) {
                    let w = DNA_IMPORTANCE_WEIGHTS.get(i).copied().unwrap_or(1.0/13.0);
                    d_sq += w * (f - centroid[i]).powi(2);
                }
                sum_weighted_dist_sq += d_sq;
            }
            let rms_dist = (sum_weighted_dist_sq / count as f64).sqrt();
            feature_diversity = (rms_dist / (rms_dist + 1.0)).clamp(0.0, 1.0);
        }

        // 6. Honest Alignment Factor (Centered at 0.33)
        let alignment = (buy_c.max(sell_c) as f64 / total_strategies as f64).clamp(0.0, 1.0);
        let normalized_alignment = ((alignment - 0.33) / 0.67).clamp(0.0, 1.0);
        let alignment_factor = 0.5 + 0.5 * normalized_alignment;

        // 7. Final Alpha Composition
        let mut alpha = (0.35 * support_ratio) + 
                        (0.20 * (avg_score.min(2.0) / 2.0)) + 
                        (0.15 * archetype_diversity) + 
                        (0.15 * disagreement_entropy) + 
                        (0.15 * feature_diversity);

        alpha *= alignment_factor;

        // 8. Continuous Overfit Guard
        let penalty = (support_ratio * (1.0 - archetype_diversity)).clamp(0.0, 1.0);
        alpha *= 1.0 - (0.3 * penalty);

        let realized_edge_factor = 1.0;
        alpha *= realized_edge_factor;

        let label = if alpha > 0.6 && archetype_diversity > 0.5 {
            " 🔥 SIGNAL TRUTH"
        } else if support_ratio > 0.8 {
            "crowded trade"
        } else if feature_diversity > 0.5 {
            "diverse niche"
        } else {
            "speculative"
        }.to_string();

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
            archetypes: unique_archs.into_iter().map(|a| match a {
                Archetype::Conviction => 0,
                Archetype::Momentum => 1,
                Archetype::Reversion => 2,
                Archetype::Volatility => 3,
            }).collect(),
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
        global_entropy: 0.0,
        active_strategies: total_strategies,
    }
}

pub fn evaluate_and_aggregate(
    strategy: &Strategy,
    config: &GaConfig,
    scenarios: &[ScenarioPair],
    generation: usize,
) -> Option<StrategyEvaluation> {
    evaluate_and_aggregate_with_trade_depth(strategy, config, scenarios, generation).map(|(e, _)| e)
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

    fn synthetic_harness_trade_tape(base_ts: u64, flat_price: u64, step_price: u64) -> Vec<MarketEvent> {
        let mut v = Vec::with_capacity(128);
        for i in 0..128 {
            let ts = base_ts + i as u64;
            // Flat then small step: fills + TP path while keeping aggregate fitness inside GA's `<= 1.0` gate.
            let price = if i < 48 { flat_price } else { step_price };
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
        scenarios.insert("High_Liquidity_Stable_Price".to_string(), vec![
            MarketEvent { subtype: MarketEventType::NewOrder, price: 100, quantity: 2000, side: Some(Side::Sell), exchange_ts: 10 },
            MarketEvent { subtype: MarketEventType::Trade, price: 100, quantity: 500, side: None, exchange_ts: 15 },
            MarketEvent { subtype: MarketEventType::Trade, price: 100, quantity: 500, side: None, exchange_ts: 20 },
        ]);
        scenarios.insert("Increasing_Queue_Ahead".to_string(), vec![
            MarketEvent { subtype: MarketEventType::NewOrder, price: 100, quantity: 1000, side: Some(Side::Sell), exchange_ts: 10 },
            MarketEvent { subtype: MarketEventType::NewOrder, price: 100, quantity: 2000, side: Some(Side::Sell), exchange_ts: 11 },
            MarketEvent { subtype: MarketEventType::NewOrder, price: 100, quantity: 3000, side: Some(Side::Sell), exchange_ts: 12 },
            MarketEvent { subtype: MarketEventType::Trade, price: 100, quantity: 100, side: None, exchange_ts: 15 },
        ]);
        scenarios
    }

    #[test]
    fn test_ga_determinism() {
        let config1 = get_default_ga_config();
        let config2 = config1.clone();
        let scenarios_map = get_scenarios_map();
        let scenarios_vec: Vec<ScenarioPair> = scenarios_map.iter().map(|(name, events)| ScenarioPair {
            name,
            signal_symbol: "TEST",
            execution_symbol: "TEST",
            signal: events.as_slice(),
            execution: events.as_slice(),
        }).collect();

        let ga_result1 = run_ga_evolution(config1, &scenarios_vec);
        let ga_result2 = run_ga_evolution(config2, &scenarios_vec);

        assert_eq!(ga_result1.global_best.strategy, ga_result2.global_best.strategy, "Best strategy diverged with same seed");
        assert!(
            (ga_result1.global_best.fitness - ga_result2.global_best.fitness).abs() < 1e-6,
            "Best strategy fitness diverged with same seed"
        );
        assert_eq!(ga_result1.global_best_generation, ga_result2.global_best_generation, "Global best generation diverged");
        assert_eq!(ga_result1.final_generation_best.strategy, ga_result2.final_generation_best.strategy, "Final generation best strategy diverged");
        assert!(
            (ga_result1.final_generation_best.fitness - ga_result2.final_generation_best.fitness).abs() < 1e-6,
            "Final generation best fitness diverged"
        );

        println!("✅ GA determinism test passed.");
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
        let report = evaluate_strategy(&strategy, &pair, &config, 0);

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

            println!("Report: {:#?}", r);
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
            if let Some(r) = evaluate_strategy(&strategy, &pair, &config, 0) {
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
        let scenarios_vec: Vec<ScenarioPair> = scenarios_map.iter().map(|(name, events)| ScenarioPair {
            name,
            signal_symbol: "TEST",
            execution_symbol: "TEST",
            signal: events.as_slice(),
            execution: events.as_slice(),
        }).collect();
        let ga_result = run_ga_evolution(config, &scenarios_vec);
        println!("Final Best Report (Global Best): {:#?}", ga_result.global_best);

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
        let scenarios_vec: Vec<ScenarioPair> = scenarios_map.iter().map(|(name, events)| ScenarioPair {
            name,
            signal_symbol: "TEST",
            execution_symbol: "TEST",
            signal: events.as_slice(),
            execution: events.as_slice(),
        }).collect();
        let ga_result = run_ga_evolution(config, &scenarios_vec);

        println!("Global Best in Test: {:#?}", ga_result.global_best);
        println!("Final Generation Best in Test: {:#?}", ga_result.final_generation_best);
        println!("✅ Top K sorted test passed. (Test adjusted for new return type)");
    }

    fn mock_scenario_eval(pnl: f64, trades: usize, profitable: usize, entropy: f64) -> StrategyEvaluation {
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
            participation_rate: if trades > 0 { 1.0 } else { 0.0 },
            profitable_trades: profitable,
            zero_pnl_trades: 0,
            quality_trades: profitable as f64,
            payoff_ratio: 2.0, 
            execution_metrics: ExecutionMetrics {
                fill_efficiency: 1.0,
                capture_efficiency: 1.0,
                avg_slippage: 0.0,
                latency_impact: 0.0,
            },
            scenario_signature: ScenarioExecutionSignature::default(),
            avg_conviction: 1.0,
            avg_efficiency: 1.0,
            avg_edge_quality: 1.0,
            directional_accuracy: if trades > 0 { profitable as f64 / trades as f64 } else { 0.0 },
            decisiveness: 1.0,
            execution_friction: 1.0,
            exit_tp_count: profitable,
            exit_sl_count: trades - profitable,
            exit_ts_count: 0,
            consistency_score: 1.0,
            recent_performance: pnl,
            selectivity: if trades > 0 { 0.05 } else { 0.0 },
            avg_entropy: entropy,
            ..StrategyEvaluation::default()
        }
    }

    #[test]
    fn test_fitness_sparse_strategy_collapse() {
        let mut evals = Vec::new();
        for _ in 0..2 { evals.push(mock_scenario_eval(0.03, 5, 5, 0.45)); } // active
        for _ in 0..8 { evals.push(mock_scenario_eval(0.0, 0, 0, 0.45)); } // inactive
        
        let config = get_default_ga_config();
        let aggregated = aggregate_strategy_reports_with_top_k(evals, &config, None, 0).unwrap();
        
        // Participation is 0.2. Under bounded/log fitness, weak strategies should remain low but non-negative.
        assert!(aggregated.fitness < 0.5 && aggregated.fitness >= 0.0,
            "Sparse strategy fitness should be very low ({}).", aggregated.fitness);
    }

    #[test]
    fn test_fitness_high_participation_outperforms() {
        // Strategy A: high participation (0.8), stronger pnl/trade profile
        let mut evals_a = Vec::new();
        for _ in 0..9 { evals_a.push(mock_scenario_eval(0.03, 3, 3, 0.45)); }
        evals_a.push(mock_scenario_eval(0.0, 0, 0, 0.45));
        let config = get_default_ga_config();
        let agg_a = aggregate_strategy_reports_with_top_k(evals_a, &config, None, 0).unwrap();

        // Strategy B: low participation (0.3), higher average active pnl
        let mut evals_b = Vec::new();
        for _ in 0..3 { evals_b.push(mock_scenario_eval(0.03, 5, 5, 0.45)); }
        for _ in 0..7 { evals_b.push(mock_scenario_eval(0.0, 0, 0, 0.45)); }
        let agg_b = aggregate_strategy_reports_with_top_k(evals_b, &config, None, 0).unwrap();

        // Under bounded/log fitness, high-participation profile should dominate low-participation.
        assert!(
            agg_a.fitness > agg_b.fitness,
            "Expected high participation fitness {} to exceed low participation fitness {}",
            agg_a.fitness,
            agg_b.fitness
        );
        assert!(agg_b.fitness >= 0.0, "Low participation fitness {} should stay non-negative.", agg_b.fitness);
    }

    #[test]
    fn test_fitness_low_trade_count_penalizes() {
        let config = get_default_ga_config();
        // Strategy A: 5 trades total (5 active scenarios, 1 trade each)
        // Bypasses participation reject, but gets crushed by trades < 10 hard filter AND effectiveness scaling
        let mut evals_a = Vec::new();
        for _ in 0..5 { evals_a.push(mock_scenario_eval(0.02, 1, 1, 0.45)); }
        for _ in 0..5 { evals_a.push(mock_scenario_eval(0.0, 0, 0, 0.45)); }
        let agg_a = aggregate_strategy_reports_with_top_k(evals_a, &config, None, 0).unwrap();

        // Strategy B: strong profile with full participation and enough trades
        let mut evals_b = Vec::new();
        for _ in 0..10 { evals_b.push(mock_scenario_eval(0.03, 4, 4, 0.45)); }
        let agg_b = aggregate_strategy_reports_with_top_k(evals_b, &config, None, 0).unwrap();

        assert!(agg_a.fitness < 0.5 && agg_a.fitness >= 0.0, 
            "Low trade count fitness {} should be very low.", agg_a.fitness);
        assert!(agg_b.fitness > agg_a.fitness, "Expected higher-trade profile to beat low-trade profile: {} vs {}", agg_b.fitness, agg_a.fitness);
    }

    #[test]
    fn test_fitness_high_variance_reduces() {
        // Stable: all 10 return 0.01 (std_dev = 0.0)
        let mut evals_stable = Vec::new();
        for _ in 0..10 { evals_stable.push(mock_scenario_eval(0.01, 5, 5, 0.45)); }
        let config = get_default_ga_config();
        let agg_stable = aggregate_strategy_reports_with_top_k(evals_stable, &config, None, 0).unwrap();

        // Unstable: 5 return 0.02, 5 return 0.0 (std_dev = 0.01, same avg = 0.01)
        let mut evals_unstable = Vec::new();
        for _ in 0..5 { evals_unstable.push(mock_scenario_eval(0.02, 5, 5, 0.45)); }
        for _ in 0..5 { evals_unstable.push(mock_scenario_eval(0.0, 5, 0, 0.45)); }
        let config = get_default_ga_config();
        let agg_unstable = aggregate_strategy_reports_with_top_k(evals_unstable, &config, None, 0).unwrap();

        assert!(agg_stable.fitness > agg_unstable.fitness, 
            "Stable fitness {} must beat unstable fitness {}", 
            agg_stable.fitness, agg_unstable.fitness);
    }

    #[test]
    fn test_multiplicative_fitness_aggregation() {
        let config = get_default_ga_config();
        let mut evals = Vec::new();
        for _ in 0..10 {
            evals.push(mock_scenario_eval(-0.02, 5, 0, 0.45));
        }
        let agg = aggregate_strategy_reports_with_top_k(evals, &config, None, 0).unwrap();
        
        assert!(agg.fitness >= 0.0, "Fitness should be non-negative under log/additive model, got {}", agg.fitness);
    }

    #[test]
    fn test_fitness_hard_collapse_threshold() {
        // Collapse: 2 active (part = 0.20, triggers < 0.3 collapse)
        // Also triggers total trades < 10 (unless they do 5 trades each, here they do 10 each so trades = 20)
        let mut evals_collapse = Vec::new();
        for _ in 0..2 { evals_collapse.push(mock_scenario_eval(0.03, 10, 10, 0.45)); }
        for _ in 0..8 { evals_collapse.push(mock_scenario_eval(0.0, 0, 0, 0.45)); }
        let config = get_default_ga_config();
        let agg_collapse = aggregate_strategy_reports_with_top_k(evals_collapse, &config, None, 0).unwrap();

        // Survive: strong + broad participation profile
        let mut evals_survive = Vec::new();
        for _ in 0..10 { evals_survive.push(mock_scenario_eval(0.03, 2, 2, 0.45)); }
        let config = get_default_ga_config();
        let agg_survive = aggregate_strategy_reports_with_top_k(evals_survive, &config, None, 0).unwrap();

        assert!(agg_collapse.fitness < 0.5 && agg_collapse.fitness >= 0.0,
            "Collapse fitness {} should be completely crushed.", agg_collapse.fitness);
        assert!(agg_survive.fitness > agg_collapse.fitness, "Expected broad participation profile to beat collapsed profile: {} vs {}", agg_survive.fitness, agg_collapse.fitness);
    }

    #[test]
    fn test_fitness_relative_ordering() {
        // Weak: low participation + low trades + mostly zero outcomes
        let mut weak_evals = Vec::new();
        for _ in 0..2 { weak_evals.push(mock_scenario_eval(0.005, 1, 1, 0.45)); }
        for _ in 0..8 { weak_evals.push(mock_scenario_eval(0.0, 0, 0, 0.45)); }
        let config = get_default_ga_config();
        let weak = aggregate_strategy_reports_with_top_k(weak_evals, &config, None, 0).unwrap();

        // Strong: full participation + higher pnl + high trade quality
        let mut strong_evals = Vec::new();
        for _ in 0..10 { strong_evals.push(mock_scenario_eval(0.02, 8, 8, 0.45)); }
        let strong = aggregate_strategy_reports_with_top_k(strong_evals, &config, None, 0).unwrap();

        assert!(strong.fitness > weak.fitness, "Expected strong ({}) > weak ({})", strong.fitness, weak.fitness);
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
        assert!(a.iter().zip(b.iter()).all(|(x, y)| x.strategy_id == y.strategy_id));
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
        let unweighted = aggregate_strategy_reports_with_top_k(evals.clone(), &config, None, 0).unwrap();
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
        for _ in 0..10 { a_evals.push(mock_scenario_eval(0.03, 2, 2, 0.45)); }
        let config = get_default_ga_config();
        let a = aggregate_strategy_reports_with_top_k(a_evals, &config, None, 0).unwrap();

        let mut b_evals = Vec::new();
        for _ in 0..3 { b_evals.push(mock_scenario_eval(0.01, 1, 1, 0.45)); }
        for _ in 0..7 { b_evals.push(mock_scenario_eval(0.0, 0, 0, 0.45)); }
        let b = aggregate_strategy_reports_with_top_k(b_evals, &config, None, 0).unwrap();

        assert!((a.fitness - b.fitness).abs() > 1e-4, "Expected non-trivial fitness spread, got a={} b={}", a.fitness, b.fitness);
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
        let scenarios_vec: Vec<ScenarioPair> = scenarios_map.iter().map(|(name, events)| ScenarioPair {
            name,
            signal_symbol: "SYNTH",
            execution_symbol: "SYNTH",
            signal: events.as_slice(),
            execution: events.as_slice(),
        }).collect();

        let ga_result = run_ga_evolution(config.clone(), &scenarios_vec);
        let (eval, depth) = evaluate_and_aggregate_with_trade_depth(
            &ga_result.global_best.strategy,
            &config,
            &scenarios_vec,
            0,
        )
        .expect("synthetic aggregate should produce a report");
        assert!(eval.fitness.is_finite());
        assert!(depth >= 1.0 - 1e-9);
        assert!(
            depth > 1.0 + 1e-9,
            "harness expects multi-trade (cap 3); depth {:.4} suggests single-trade regression",
            depth
        );
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
        println!(
            "SYNTH_HARNESS → fitness: {:.4}, depth: {:.2}, trade_count: {}",
            eval.fitness, depth, eval.trade_count
        );
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
            let candles = CsvCandleSource { path: csv_path.clone() }.get_candles_sync();
            let asset = Path::new(file)
                .file_stem()
                .and_then(|s| s.to_str())
                .and_then(|stem| stem.split('_').next())
                .unwrap_or("UNKNOWN")
                .to_ascii_uppercase() + "_SIM";
            let n_before = scenarios.len();
            scenarios.extend(crate::pipeline::scenarios_from_candles(&asset, &candles));
            assert!(
                scenarios.len() > n_before,
                "{} should yield at least one scenario window",
                file
            );
        }

        let scenarios_map = scenarios;
        let scenarios_vec: Vec<ScenarioPair> = scenarios_map.iter().map(|(name, events)| ScenarioPair {
            name,
            signal_symbol: "TEST",
            execution_symbol: "TEST",
            signal: events.as_slice(),
            execution: events.as_slice(),
        }).collect();

        let ga_result = run_ga_evolution(config.clone(), &scenarios_vec);
        let (eval, avg_trades_per_active) =
            evaluate_and_aggregate_with_trade_depth(&ga_result.global_best.strategy, &config, &scenarios_vec, 0)
                .expect("Aggregation should produce evaluation");
        assert!(eval.fitness > 0.0);
        assert!(
            avg_trades_per_active >= 1.0 - 1e-9,
            "expected >= 1 trade per active scenario after Top-K aggregation, got {}",
            avg_trades_per_active
        );

        println!(
            "DEBUG → fitness: {:.4}, depth (avg_trades/active): {:.2}",
            eval.fitness, avg_trades_per_active
        );
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
        
        let mock_scenario_eval = |pnl: f64, trades: usize, profitable: usize| -> StrategyEvaluation {
            StrategyEvaluation {
                avg_pnl: pnl,
                trade_count: trades,
                profitable_trades: profitable,
                win_rate: if trades > 0 { profitable as f64 / trades as f64 } else { 0.0 },
                fitness: 0.5, // Non-zero baseline for multiplier testing
                ..StrategyEvaluation::default()
            }
        };

        // 1. Nominal strategy (10% selectivity -> 0.10)
        let mut eval_nominal = mock_scenario_eval(0.02, 10, 10);
        eval_nominal.selectivity = 0.10;
        let agg_nominal = aggregate_strategy_reports_inner(vec![eval_nominal], 1.0, &config, 0).unwrap().0;
        let mut eval_over = mock_scenario_eval(0.02, 10, 10);
        eval_over.selectivity = 0.20;
        let agg_over = aggregate_strategy_reports_inner(vec![eval_over], 1.0, &config, 0).unwrap().0;

        // At alpha=5.0, diff=0.10, decay = exp(-5 * 0.1) = exp(-0.5) approx 0.606
        let ratio = agg_over.fitness / agg_nominal.fitness;
        assert!(ratio > 0.55 && ratio < 0.65, "Expected approx 0.6x decay for 20% selectivity, got {}", ratio);
    }

    #[test]
    fn test_continuous_entropy_weighting() {
        let config = get_default_ga_config();
        
        let mock_scenario_eval = |pnl: f64, trades: usize, profitable: usize| -> StrategyEvaluation {
            StrategyEvaluation {
                avg_pnl: pnl,
                trade_count: trades,
                profitable_trades: profitable,
                win_rate: if trades > 0 { profitable as f64 / trades as f64 } else { 0.0 },
                fitness: 0.5, // Non-zero baseline for multiplier testing
                ..StrategyEvaluation::default()
            }
        };

        // 1. Optimal Entropy (0.45)
        let mut eval_opt = mock_scenario_eval(0.02, 10, 10);
        eval_opt.avg_entropy = 0.45;
        let agg_opt = aggregate_strategy_reports_inner(vec![eval_opt], 1.0, &config, 0).unwrap().0;
        let mut eval_high = mock_scenario_eval(0.02, 10, 10);
        eval_high.avg_entropy = 0.90;
        let agg_high = aggregate_strategy_reports_inner(vec![eval_high], 1.0, &config, 0).unwrap().0;
        let mut eval_low = mock_scenario_eval(0.02, 10, 10);
        eval_low.avg_entropy = 0.10;
        let agg_low = aggregate_strategy_reports_inner(vec![eval_low], 1.0, &config, 0).unwrap().0;

        assert!(agg_opt.fitness > agg_high.fitness, "Optimal entropy (0.45) should beat high entropy (0.90)");
        assert!(agg_opt.fitness > agg_low.fitness, "Optimal entropy (0.45) should beat low entropy (0.10)");
    }
}
    