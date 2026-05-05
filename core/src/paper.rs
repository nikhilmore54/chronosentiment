use crate::ga::{AlphaConsensus, SignalType, TradeRecommendation};
use crate::market_adapter::Candle;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeIntent {
    pub rec_id: u64,
    pub symbol: String,
    pub signal: SignalType,
    pub reference_price: f64,
    pub recommendation: TradeRecommendation,
    pub strategy_id: usize,
    #[serde(default)]
    pub rec_score: f64,
    #[serde(default)]
    pub rec_feas: f64,
    #[serde(default)]
    pub rec_conf: f64,
    #[serde(default)]
    pub rec_voters: usize,
    #[serde(default)]
    pub momentum_3: f64,
    #[serde(default)]
    pub vol_5: f64,
    #[serde(default)]
    pub score_std_5: f64,
    pub consensus: Option<AlphaConsensus>,
    pub age: usize,
    pub max_age: usize,
    pub intent_created_symbol_updates: usize,
    pub confirm_delta_symbol_updates: u32,
    pub immediate_market_fill: bool,
    pub use_recommendation_tpsl: bool,
    pub sketch_risk_span: f64,
    pub mode: String,
    pub birth_price: f64,
    #[serde(default)]
    pub entry_path: String,
    #[serde(default)]
    pub regime: String,
    #[serde(default)]
    pub birth_timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AdaptiveExitState {
    ArmedObserved,
    ActiveTrailing,
    Exited,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ExitType {
    TakeProfit,
    StopLoss,
    Time,
    TrailingStop,
    Halt,
    Manual,
    NoMomentum,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveTrade {
    pub rec_id: u64,
    pub symbol: String,
    pub signal: SignalType,
    pub entry_price: f64,
    pub tp_target: f64,
    pub sl_target: f64,
    pub hold_limit: usize,
    pub current_hold: usize,
    pub size: f64,
    pub vol_bps: f64,
    pub rank: f64,
    pub expected_edge_bps: f64,
    pub tp_vol_unit: f64,
    pub max_pnl: f64,
    pub min_pnl: f64,
    pub min_pnl_first_10: f64,
    pub bars_to_mfe: usize,
    pub last_mark_pnl: f64,
    pub decay_count: usize,
    pub trailing_armed_seen: bool,
    pub exit_state: AdaptiveExitState,
    pub bars_since_pullback: usize,
    pub strategy_id: usize,
    pub rec_score: f64,
    pub rec_feas: f64,
    pub rec_conf: f64,
    pub rec_voters: usize,
    pub intent_age_at_fill: usize,
    pub momentum_3: f64,
    pub vol_5: f64,
    pub score_std_5: f64,
    pub partial_tp_done: bool,
    pub remaining_fraction: f64,
    pub realized_partial_pnl: f64,
    pub trail_active: bool,
    pub trail_stop: f64,
    pub peak_price: f64,
    pub is_runner: bool,
    pub consensus: Option<AlphaConsensus>,
    pub lock_tpsl_levels: bool,
    pub frozen_sl_target: f64,
    pub frozen_tp_target: f64,
    pub from_sketch: bool,
    pub sketch_minute_bucket: i64,
    pub reference_price: f64,
    pub birth_price: f64,
    pub entry_mode: String,
    pub timestamp: u64,
    pub entry_path: String,
    pub regime: String,
    #[serde(default)]
    pub birth_timestamp: u64,
}

#[derive(Serialize, Deserialize)]
pub struct PaperRegistry {
    pub pending_intents: Vec<TradeIntent>,
    pub active_trades: Vec<ActiveTrade>,
    pub closed_count: usize,
    pub pnl_history: Vec<f64>,
    pub closed_observations: Vec<TradeObservation>,
    pub max_concurrent: usize,
    pub stats_p90: HashMap<String, f64>,
    pub stats_p92: HashMap<String, f64>,
    pub stats_p95: HashMap<String, f64>,
    pub stats_buffer: HashMap<String, Vec<f64>>,
    pub intents_triggered: usize,
    pub intents_triggered_buy: usize,
    pub intents_triggered_sell: usize,
    pub intents_created: usize,
    pub intents_expired: usize,
    pub strategy_pnl: HashMap<usize, f64>,
    pub strategy_counts: HashMap<usize, usize>,
    pub positive_excursion_count: usize,
    pub fbpr_count: usize,
    pub iqr_count: usize,
    pub total_entry_drift_bps: f64,
    pub mode_stats: HashMap<String, (usize, usize, usize, usize, f64, f64)>, // mode -> (closed, excursion, fbpr, iqr, drift, pnl)
    pub adaptation_threshold: usize,
    pub shadow_total: usize,
    pub shadow_profitable: usize,
    pub rej_drift: usize,
    pub rej_confirm: usize,
    pub rej_imbalance: usize,
    pub path_impulse_count: usize,
    pub path_micro_count: usize,
    pub path_strategy_count: usize,
    pub path_metrics: HashMap<String, (usize, usize, f64, f64, f64, usize, f64, usize)>, // path -> (closed, wins, total_pnl, win_pnl, loss_pnl, fbpr, mfe_sum, dur_sum)
    pub regime_metrics: HashMap<String, (usize, usize, f64, f64, f64, usize, f64, usize)>, // regime -> (closed, wins, total_pnl, win_pnl, loss_pnl, fbpr, mfe_sum, dur_sum)
    pub probes_emitted: usize,
    pub probes_confirmed: usize,
    pub blocked_signals: usize,
    pub blocked_pnl_sum: f64,
    pub signals_raw_count: usize,
    pub signals_raw_pnl: f64,
    pub signals_random_count: usize,
    pub signals_random_pnl: f64,
    pub signals_pnl_squared_sum: f64, // Variance of accepted trades
    pub signals_raw_pnl_squared_sum: f64, // Variance of filtered baseline
    pub signals_random_pnl_squared_sum: f64, // Variance of random baseline
    pub eos_buckets: HashMap<String, (usize, f64)>, // bucket_id -> (count, pnl_sum)
    pub pnl_samples: Vec<f64>, // For distribution quantiles
    pub regime_distribution: HashMap<String, usize>,
}

impl Default for PaperRegistry {
    fn default() -> Self {
        Self {
            pending_intents: Vec::new(),
            active_trades: Vec::new(),
            closed_count: 0,
            pnl_history: Vec::new(),
            closed_observations: Vec::new(),
            max_concurrent: 0,
            stats_p90: HashMap::new(),
            stats_p92: HashMap::new(),
            stats_p95: HashMap::new(),
            stats_buffer: HashMap::new(),
            intents_triggered: 0,
            intents_triggered_buy: 0,
            intents_triggered_sell: 0,
            intents_created: 0,
            intents_expired: 0,
            strategy_pnl: HashMap::new(),
            strategy_counts: HashMap::new(),
            positive_excursion_count: 0,
            fbpr_count: 0,
            iqr_count: 0,
            total_entry_drift_bps: 0.0,
            mode_stats: HashMap::new(),
            adaptation_threshold: 30,
            shadow_total: 0,
            shadow_profitable: 0,
            rej_drift: 0,
            rej_confirm: 0,
            rej_imbalance: 0,
            path_impulse_count: 0,
            path_micro_count: 0,
            path_strategy_count: 0,
            path_metrics: HashMap::new(),
            regime_metrics: HashMap::new(),
            probes_emitted: 0,
            probes_confirmed: 0,
            blocked_signals: 0,
            blocked_pnl_sum: 0.0,
            signals_raw_count: 0,
            signals_raw_pnl: 0.0,
            signals_random_count: 0,
            signals_random_pnl: 0.0,
            signals_pnl_squared_sum: 0.0,
            signals_raw_pnl_squared_sum: 0.0,
            signals_random_pnl_squared_sum: 0.0,
            eos_buckets: HashMap::new(),
            pnl_samples: Vec::new(),
            regime_distribution: HashMap::new(),
        }
    }
}

impl PaperRegistry {
    pub fn record_rejection(&mut self, reason: &str) {
        match reason {
            "drift" => self.rej_drift += 1,
            "confirm" => self.rej_confirm += 1,
            "imbalance" => self.rej_imbalance += 1,
            _ => {}
        }
    }

    pub fn record_shadow_outcome(&mut self, is_profitable: bool) {
        self.shadow_total += 1;
        if is_profitable {
            self.shadow_profitable += 1;
        }
    }
    pub fn summary(&self) {
        let total_pnl: f64 = self.pnl_history.iter().sum();
        let wins = self.pnl_history.iter().filter(|&&p| p > 0.0).count();
        let win_rate = if self.closed_count > 0 { wins as f64 / self.closed_count as f64 } else { 0.0 };
        let olr = if self.shadow_total > 0 { self.shadow_profitable as f64 / self.shadow_total as f64 } else { 0.0 };
        
        let tp = wins;
        let fp = self.closed_count - wins;
        let fn_count = self.shadow_profitable;
        
        let precision = if (tp + fp) > 0 { tp as f64 / (tp + fp) as f64 } else { 0.0 };
        let recall = if (tp + fn_count) > 0 { tp as f64 / (tp + fn_count) as f64 } else { 0.0 };
        
        let win_sum: f64 = self.pnl_history.iter().filter(|&&p| p > 0.0).sum();
        let loss_sum: f64 = self.pnl_history.iter().filter(|&&p| p <= 0.0).sum();
        let win_count = self.pnl_history.iter().filter(|&&p| p > 0.0).count();
        let loss_count = self.pnl_history.iter().filter(|&&p| p <= 0.0).count();
        
        let avg_win = if win_count > 0 { win_sum / win_count as f64 } else { 0.0 };
        let avg_loss = if loss_count > 0 { loss_sum / loss_count as f64 } else { 0.0 };
        let expectancy = (win_rate * avg_win) + ((1.0 - win_rate) * avg_loss);
        
        let start_time = self.closed_observations.first().map(|o| o.timestamp).unwrap_or(0);
        let end_time = self.closed_observations.last().map(|o| o.timestamp).unwrap_or(0);
        let duration_min = if end_time > start_time { (end_time - start_time) as f64 / 60.0 } else { 0.0 };
        let expectancy_per_min = if duration_min > 0.1 { total_pnl / duration_min } else { 0.0 };

        println!("[PAPER_SUMMARY] closed={} pnl={:.6} win_rate={:.3} precision={:.3} recall={:.3} olr={:.3} exp_per_min={:.6} avg_win={:.6} avg_loss={:.6} expectancy={:.6}", 
            self.closed_count, total_pnl, win_rate, precision, recall, olr, expectancy_per_min, avg_win, avg_loss, expectancy);
        
        println!("[PATH_DISTRIBUTION] impulse={} micro={} strategy={}", self.path_impulse_count, self.path_micro_count, self.path_strategy_count);
        
        for (path, (count, wins, pnl, win_pnl, loss_pnl, fbpr, mfe_sum, dur_sum)) in &self.path_metrics {
            let wr = if *count > 0 { *wins as f64 / *count as f64 } else { 0.0 };
            let fbpr_rate = if *count > 0 { *fbpr as f64 / *count as f64 } else { 0.0 };
            let exp = if *count > 0 { *pnl / *count as f64 } else { 0.0 };
            let avg_mfe = if *count > 0 { *mfe_sum / *count as f64 } else { 0.0 };
            let avg_dur = if *count > 0 { *dur_sum as f64 / *count as f64 } else { 0.0 };
            println!("[PATH_ALPHA] path={} trades={} win_rate={:.3} fbpr={:.3} pnl={:.6} expectancy={:.6} avg_mfe={:.2}bps avg_dur={:.1}", 
                path, count, wr, fbpr_rate, pnl, exp, avg_mfe * 10000.0, avg_dur);
        }

        println!("--- REGIME ANALYSIS ---");
        for (regime, (count, wins, pnl, win_pnl, loss_pnl, fbpr, mfe_sum, dur_sum)) in &self.regime_metrics {
            let wr = if *count > 0 { *wins as f64 / *count as f64 } else { 0.0 };
            let exp = if *count > 0 { *pnl / *count as f64 } else { 0.0 };
            let fbpr_rate = if *count > 0 { *fbpr as f64 / *count as f64 } else { 0.0 };
            println!("[REGIME_ALPHA] regime={} trades={} win_rate={:.3} fbpr={:.3} expectancy={:.6} pnl={:.6}", 
                regime, count, wr, fbpr_rate, exp, pnl);
        }

        let survival_rate = if self.probes_emitted > 0 { self.probes_confirmed as f64 / self.probes_emitted as f64 } else { 0.0 };
        let avg_blocked_pnl = if self.blocked_signals > 0 { self.blocked_pnl_sum / self.blocked_signals as f64 } else { 0.0 };
        println!("[EXECUTION_FIDELITY] probes_emitted={} probes_confirmed={} survival_rate={:.3} blocked_signals={} blocked_counterfactual_pnl={:.6} avg_blocked_pnl={:.6}", 
            self.probes_emitted, self.probes_confirmed, survival_rate, self.blocked_signals, self.blocked_pnl_sum, avg_blocked_pnl);
        
        let alpha_preservation = if self.blocked_pnl_sum < 0.0 { self.blocked_pnl_sum.abs() } else { 0.0 };
        println!("[FILTER_PRECISION] alpha_preservation={:.6} (pnl avoided by blocking)", alpha_preservation);
        
        let accepted_expectancy = if self.closed_count > 0 { total_pnl / self.closed_count as f64 } else { 0.0 };
        let raw_expectancy = if self.signals_raw_count > 0 { self.signals_raw_pnl / self.signals_raw_count as f64 } else { 0.0 };
        let incremental_edge = accepted_expectancy - raw_expectancy;
        
        let accepted_variance = if self.closed_count > 1 {
            (self.signals_pnl_squared_sum / self.closed_count as f64) - (accepted_expectancy * accepted_expectancy)
        } else { 0.0 };
        let accepted_std = accepted_variance.sqrt();
        let t_stat = if accepted_std > 1e-9 && self.closed_count > 0 { 
            incremental_edge / (accepted_std / (self.closed_count as f64).sqrt()) 
        } else { 0.0 };
        
        println!("[SIGMA] accepted_std={:.6} incremental_edge={:.6} t_stat={:.4} (corrected n={})", 
            accepted_std, incremental_edge, t_stat, self.closed_count);

        let random_expectancy = if self.signals_random_count > 0 { self.signals_random_pnl / self.signals_random_count as f64 } else { 0.0 };
        println!("[RANDOM_BASELINE] random_exp={:.6} vs_random_uplift={:.6}", 
            random_expectancy, accepted_expectancy - random_expectancy);
            
        // PNL DISTRIBUTION
        if !self.pnl_samples.is_empty() {
            let mut samples = self.pnl_samples.clone();
            samples.sort_by(|a, b| a.partial_cmp(b).unwrap());
            println!("[PNL_DISTRIBUTION] min={:.6} p25={:.6} p50={:.6} p75={:.6} max={:.6}",
                samples[0], samples[samples.len()/4], samples[samples.len()/2], samples[samples.len()*3/4], samples[samples.len()-1]);
        }
        
        // EOS BUCKETS
        for (bucket, (count, pnl)) in &self.eos_buckets {
            println!("[EOS_BUCKETS] bucket={} trades={} expectancy={:.6}", bucket, count, pnl / (*count as f64).max(1.0));
        }

        println!("[REGIME_DISTRIBUTION] {:?}", self.regime_distribution);
        println!("[REJECTION_PROFILE] drift={} confirm={} imbalance={}", self.rej_drift, self.rej_confirm, self.rej_imbalance);
        
        let avg_drift = if self.closed_count > 0 { self.total_entry_drift_bps / self.closed_count as f64 } else { 0.0 };
        let fbpr_global = if self.closed_count > 0 { self.fbpr_count as f64 / self.closed_count as f64 } else { 0.0 };
        println!("[TIMING_INTEGRITY] avg_drift={:.2}bps (late_entry_bias) FBPR={:.3} (impulse_capture)", 
            avg_drift, fbpr_global);
        
        for (mode, (count, exc, fbp, iq, drift, pnl)) in &self.mode_stats {
            let m_per = if *count > 0 { *exc as f64 / *count as f64 } else { 0.0 };
            let m_fbpr = if *count > 0 { *fbp as f64 / *count as f64 } else { 0.0 };
            let m_iqr = if *count > 0 { *iq as f64 / *count as f64 } else { 0.0 };
            let m_drift = if *count > 0 { *drift / *count as f64 } else { 0.0 };
            println!("[MODE_SUMMARY] mode={} count={} per={:.3} fbpr={:.3} iqr={:.3} drift={:.2} pnl={:.6}", mode, count, m_per, m_fbpr, m_iqr, m_drift, pnl);
        }
    }

    pub fn get_strategy_performance(&self, _strategy_id: usize) -> f64 {
        0.0005f64
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeObservation {
    pub rec_id: u64,
    pub symbol: String,
    pub pnl: f64,
    pub dur: usize,
    pub exit_type: String,
    pub rank: f64,
    pub vol_bucket: usize,
    pub mfe: f64,
    pub mae_abs: f64,
    pub hold_bars: usize,
    pub timestamp: u64,
    pub entry_path: String,
    pub regime: String,
}

pub fn apply_slippage(price: f64, is_buy: bool, vol_bps: f64) -> f64 {
    let target_leg_bps = (vol_bps / 10.0).clamp(1.0, 5.0); 
    let slippage = price * (target_leg_bps / 10000.0);
    if is_buy {
        price + slippage
    } else {
        price - slippage
    }
}

pub fn update_paper_registry(
    registry: &mut PaperRegistry,
    latest_candle: &Candle,
    symbol: &str,
    _symbol_linear_updates: usize,
    _trigger_momentum_3: f64,
    _trigger_vol_5: f64,
) {
    let open = latest_candle.open as f64 / 10000.0;
    let high = latest_candle.high as f64 / 10000.0;
    let low = latest_candle.low as f64 / 10000.0;
    let close = latest_candle.close as f64 / 10000.0;
    let bar_range = (high - low) / open.max(1e-12);

    // 1. Fill Pending Intents (Next-Bar Open)
    let mut j = 0;
    while j < registry.pending_intents.len() {
        let intent = &registry.pending_intents[j];
        if intent.symbol != symbol {
            j += 1;
            continue;
        }

        let fill_px = open;
        let is_long = intent.signal == SignalType::BUY;
        let entry_price = apply_slippage(fill_px, is_long, intent.recommendation.vol_bps);
        
        // Adaptive Unscaling Heuristic
        let mut tp_target = intent.recommendation.tp_target;
        let mut sl_target = intent.recommendation.sl_target;
        if tp_target < entry_price / 100.0 {
            tp_target *= 10000.0;
            sl_target *= 10000.0;
        } else if tp_target > entry_price * 100.0 {
            tp_target /= 10000.0;
            sl_target /= 10000.0;
        }
        
        // Directional Target Enforcement
        if is_long {
            if tp_target <= entry_price { tp_target = entry_price * 1.0020; }
            if sl_target >= entry_price { sl_target = entry_price * 0.9980; }
        } else {
            if tp_target >= entry_price { tp_target = entry_price * 0.9980; }
            if sl_target <= entry_price { sl_target = entry_price * 1.0020; }
        }

        // --- HARD EDGE FILTER ---
        let edge_bps = (tp_target - entry_price).abs() / entry_price.max(1e-6) * 10000.0;
        let slippage_bps = (apply_slippage(1.0, true, intent.recommendation.vol_bps) - 1.0) * 10000.0 * 2.0;
        let expected_edge = edge_bps * intent.rec_conf;
        
        println!("[EV_DASH] sym={} rec_id={} edge_bps={:.1} conf={:.2} exp_edge={:.1} friction={:.1}", 
                 symbol, intent.rec_id, edge_bps, intent.rec_conf, expected_edge, slippage_bps);

        if edge_bps < 1.5 * slippage_bps {
            println!("[SIGNAL_REJECTED] sym={} edge={:.1} < 1.5*friction({:.1}) -> Noise killed.", 
                     symbol, edge_bps, slippage_bps);
            registry.pending_intents.remove(j);
            continue;
        }

        registry.active_trades.push(ActiveTrade {
            rec_id: intent.rec_id,
            symbol: intent.symbol.clone(),
            entry_price,
            tp_target,
            sl_target,
            hold_limit: intent.recommendation.holding_bars,
            current_hold: 0,
            signal: intent.signal,
            size: intent.recommendation.position_size,
            vol_bps: intent.recommendation.vol_bps,
            rank: intent.recommendation.rank,
            expected_edge_bps: intent.recommendation.expected_edge_bps,
            tp_vol_unit: bar_range,
            max_pnl: 0.0,
            min_pnl: 0.0,
            min_pnl_first_10: f64::INFINITY,
            bars_to_mfe: 0,
            last_mark_pnl: 0.0,
            decay_count: 0,
            trailing_armed_seen: false,
            exit_state: AdaptiveExitState::ArmedObserved,
            bars_since_pullback: usize::MAX,
            strategy_id: intent.strategy_id,
            rec_score: intent.rec_score,
            rec_feas: intent.rec_feas,
            rec_conf: intent.rec_conf,
            rec_voters: intent.rec_voters,
            intent_age_at_fill: intent.age,
            momentum_3: intent.momentum_3,
            vol_5: intent.vol_5,
            score_std_5: intent.score_std_5,
            partial_tp_done: false,
            remaining_fraction: 1.0,
            realized_partial_pnl: 0.0,
            trail_active: false,
            trail_stop: entry_price,
            peak_price: entry_price,
            is_runner: false,
            consensus: intent.consensus.clone(),
            lock_tpsl_levels: true,
            frozen_sl_target: sl_target,
            frozen_tp_target: tp_target,
            from_sketch: false,
            sketch_minute_bucket: 0,
            reference_price: intent.reference_price,
            birth_price: intent.reference_price, // Will be updated by confirmed intents
            entry_mode: intent.mode.clone(),
            timestamp: latest_candle.timestamp,
            entry_path: intent.entry_path.clone(),
            regime: intent.regime.clone(),
            birth_timestamp: intent.birth_timestamp,
        });
        
        registry.intents_triggered += 1;
        if is_long { registry.intents_triggered_buy += 1; }
        else { registry.intents_triggered_sell += 1; }
        
        registry.pending_intents.remove(j);
    }

    // 2. Update Active Trades
    let mut i = 0;
    while i < registry.active_trades.len() {
        if registry.active_trades[i].symbol != symbol {
            i += 1;
            continue;
        }

        let mut exit_pnl: Option<f64> = None;
        let mut exit_tag = ExitType::Manual;
        
        {
            let trade = &mut registry.active_trades[i];
            trade.current_hold += 1;

            let is_long = trade.signal == SignalType::BUY;
            let (bar_best, bar_worst) = if is_long {
                ((high - trade.entry_price) / trade.entry_price.max(1e-12), (low - trade.entry_price) / trade.entry_price.max(1e-12))
            } else {
                ((trade.entry_price - low) / trade.entry_price.max(1e-12), (trade.entry_price - high) / trade.entry_price.max(1e-12))
            };

            if bar_best > trade.max_pnl {
                trade.max_pnl = bar_best;
                trade.bars_to_mfe = trade.current_hold;
            }
            trade.min_pnl = trade.min_pnl.min(bar_worst);

            println!("[PAPER_EXCURSION] sym={} rec_id={} entry={:.4} high={:.4} low={:.4} best_bps={:.2} worst_bps={:.2} max_pnl_bps={:.2}",
                symbol, trade.rec_id, trade.entry_price, high, low, bar_best * 10000.0, bar_worst * 10000.0, trade.max_pnl * 10000.0);

            if is_long {
                if high >= trade.tp_target {
                    let slip_tp = apply_slippage(trade.tp_target, false, trade.vol_bps);
                    exit_pnl = Some((slip_tp - trade.entry_price) / trade.entry_price.max(1e-12));
                    exit_tag = ExitType::TakeProfit;
                } else if low <= trade.sl_target {
                    let slip_sl = apply_slippage(trade.sl_target, false, trade.vol_bps);
                    exit_pnl = Some((slip_sl - trade.entry_price) / trade.entry_price.max(1e-12));
                    exit_tag = ExitType::StopLoss;
                }
            } else {
                if low <= trade.tp_target {
                    let slip_tp = apply_slippage(trade.tp_target, true, trade.vol_bps);
                    exit_pnl = Some((trade.entry_price - slip_tp) / trade.entry_price.max(1e-12));
                    exit_tag = ExitType::TakeProfit;
                } else if high >= trade.sl_target {
                    let slip_sl = apply_slippage(trade.sl_target, true, trade.vol_bps);
                    exit_pnl = Some((trade.entry_price - slip_sl) / trade.entry_price.max(1e-12));
                    exit_tag = ExitType::StopLoss;
                }
            }

            // --- EARLY REJECTION: FIRST TICK VALIDATION ---
            if exit_pnl.is_none() && trade.current_hold == 1 {
                let current_pnl = if is_long {
                    (close - trade.entry_price) / trade.entry_price.max(1e-12)
                } else {
                    (trade.entry_price - close) / trade.entry_price.max(1e-12)
                };
                
                if current_pnl <= 0.0 {
                    let exit_price = apply_slippage(close, !is_long, trade.vol_bps);
                    exit_pnl = Some(if is_long {
                        (exit_price - trade.entry_price) / trade.entry_price.max(1e-12)
                    } else {
                        (trade.entry_price - exit_price) / trade.entry_price.max(1e-12)
                    });
                    exit_tag = ExitType::NoMomentum;
                }
            }

            if exit_pnl.is_none() && trade.current_hold >= trade.hold_limit {
                let exit_price = apply_slippage(close, !is_long, trade.vol_bps);
                exit_pnl = Some(if is_long {
                    (exit_price - trade.entry_price) / trade.entry_price.max(1e-12)
                } else {
                    (trade.entry_price - exit_price) / trade.entry_price.max(1e-12)
                });
                exit_tag = ExitType::Time;
            }
        }

        if let Some(pnl) = exit_pnl {
            let trade = registry.active_trades.remove(i);
            registry.record_trade_settlement(trade, pnl, exit_tag, latest_candle.timestamp, close);
        } else {
            i += 1;
        }
    }
}

impl PaperRegistry {
    pub fn force_close_trades_by_symbol(&mut self, symbol: &str, exit_type: ExitType, close_price: f64, timestamp: u64) {
        let mut i = 0;
        while i < self.active_trades.len() {
            if self.active_trades[i].symbol == symbol {
                let trade = self.active_trades.remove(i);
                let is_buy = matches!(trade.signal, SignalType::BUY);
                let slip_price = apply_slippage(close_price, !is_buy, trade.vol_bps);
                let pnl = if is_buy {
                    (slip_price - trade.entry_price) / trade.entry_price.max(1e-12)
                } else {
                    (trade.entry_price - slip_price) / trade.entry_price.max(1e-12)
                };
                self.record_trade_settlement(trade, pnl, exit_type.clone(), timestamp, close_price);
            } else {
                i += 1;
            }
        }
    }

    fn record_trade_settlement(&mut self, trade: ActiveTrade, pnl: f64, exit_tag: ExitType, timestamp: u64, close: f64) {
        self.closed_count += 1;
        self.pnl_history.push(pnl);
        
        if trade.max_pnl > 0.0 {
            self.positive_excursion_count += 1;
        }
        
        let is_fbpr = exit_tag != ExitType::NoMomentum;
        if is_fbpr {
            self.fbpr_count += 1;
        }

        let friction_bps = (apply_slippage(1.0, true, trade.vol_bps) - 1.0) * 10000.0 * 2.0;
        let is_iqr = trade.max_pnl * 10000.0 > 2.0 * friction_bps;
        if is_iqr {
            self.iqr_count += 1;
        }
        
        let drift_bps = (trade.entry_price - trade.birth_price).abs() / trade.birth_price.max(1.0) * 10000.0;
        self.total_entry_drift_bps += drift_bps;

        let entry_mode = trade.entry_mode.clone();
        let stats = self.mode_stats.entry(entry_mode).or_insert((0, 0, 0, 0, 0.0, 0.0));
        stats.0 += 1;
        if trade.max_pnl > 0.0 { stats.1 += 1; }
        if is_fbpr { stats.2 += 1; }
        if is_iqr { stats.3 += 1; }
        stats.4 += drift_bps;
        stats.5 += pnl;

        let current_per = if self.closed_count > 0 {
            self.positive_excursion_count as f64 / self.closed_count as f64
        } else { 0.0 };
        let current_fbpr = if self.closed_count > 0 {
            self.fbpr_count as f64 / self.closed_count as f64
        } else { 0.0 };

        let edge_loss = (trade.entry_price - trade.birth_price).abs() / trade.birth_price * 10000.0;
        println!(
            "[TRUTH_CHECK] sym={} birth={:.4} entry={:.4} exit={:.4} drift={:.2}bps",
            trade.symbol, trade.birth_price, trade.entry_price, close, edge_loss
        );
        let birth_latency = if timestamp > trade.birth_timestamp { timestamp - trade.birth_timestamp } else { 0 };
        println!(
            "[AUDIT_TRADE] sym={} dir={:?} entry={:.4} tp={:.4} sl={:.4} exit={:.4} slip_bps={:.2} conf={:.2} ideal_pnl={:.6} realized_pnl={:.6} edge_loss={:.2}bps capture={:.3} dur={} birth_lat={}ms exit_type={:?} PER={:.3} FBPR={:.3} mode={}",
            trade.symbol, trade.signal, trade.entry_price, trade.tp_target, trade.sl_target, close, 
            friction_bps, 
            trade.rec_conf, trade.max_pnl, pnl, edge_loss, (pnl / trade.max_pnl.max(1e-6)), trade.current_hold, birth_latency, exit_tag,
            current_per, current_fbpr, trade.entry_mode
        );

        self.closed_observations.push(TradeObservation {
            rec_id: trade.rec_id,
            symbol: trade.symbol.clone(),
            pnl,
            dur: trade.current_hold,
            exit_type: format!("{:?}", exit_tag),
            rank: trade.rank,
            vol_bucket: (trade.vol_bps * 100.0) as usize,
            mfe: trade.max_pnl,
            mae_abs: trade.min_pnl.abs(),
            hold_bars: trade.current_hold,
            timestamp: timestamp,
            entry_path: trade.entry_path.clone(),
            regime: trade.regime.clone(),
        });

        if trade.entry_path == "impulse" {
            self.path_impulse_count += 1;
        } else if trade.entry_path == "micro" {
            self.path_micro_count += 1;
        } else if trade.entry_path == "strategy" {
            self.path_strategy_count += 1;
        }
        
        let p_stats = self.path_metrics.entry(trade.entry_path.clone()).or_insert((0, 0, 0.0, 0.0, 0.0, 0, 0.0, 0));
        p_stats.0 += 1; // closed
        if pnl > 0.0 {
            p_stats.1 += 1; // wins
            p_stats.3 += pnl; // win_pnl
        } else {
            p_stats.4 += pnl; // loss_pnl
        }
        p_stats.2 += pnl; // total_pnl
        if is_fbpr {
            p_stats.5 += 1; // fbpr
        }
        p_stats.6 += trade.max_pnl;
        p_stats.7 += trade.current_hold;

        let r_stats = self.regime_metrics.entry(trade.regime.clone()).or_insert((0, 0, 0.0, 0.0, 0.0, 0, 0.0, 0));
        r_stats.0 += 1;
        if pnl > 0.0 { r_stats.1 += 1; r_stats.3 += pnl; } else { r_stats.4 += pnl; }
        r_stats.2 += pnl;
        if is_fbpr { r_stats.5 += 1; }
        r_stats.6 += trade.max_pnl;
        r_stats.7 += trade.current_hold;

        *self.regime_distribution.entry(trade.regime.clone()).or_insert(0) += 1;
    }

    pub fn record_probe_emit(&mut self) {
        self.probes_emitted += 1;
    }

    pub fn record_probe_confirm(&mut self) {
        self.probes_confirmed += 1;
    }

    pub fn record_block(&mut self) {
        self.blocked_signals += 1;
    }

    pub fn record_block_pnl(&mut self, pnl: f64) {
        self.blocked_pnl_sum += pnl;
    }

    pub fn record_raw_signal(&mut self, pnl: f64) {
        self.signals_raw_count += 1;
        self.signals_raw_pnl += pnl;
        self.signals_raw_pnl_squared_sum += pnl * pnl;
    }

    pub fn record_random_signal(&mut self, pnl: f64) {
        self.signals_random_count += 1;
        self.signals_random_pnl += pnl;
        self.signals_random_pnl_squared_sum += pnl * pnl;
    }

    pub fn record_accepted_sample(&mut self, pnl: f64, eos: f64) {
        self.signals_pnl_squared_sum += pnl * pnl;
        self.pnl_samples.push(pnl);
        
        let bucket = if eos < 0.02 { "low" } else if eos < 0.05 { "mid" } else { "high" };
        let entry = self.eos_buckets.entry(bucket.to_string()).or_insert((0, 0.0));
        entry.0 += 1;
        entry.1 += pnl;
    }
}

pub fn resolve_intracandle_exit(_trade: &ActiveTrade, _candle: &Candle) -> Option<(f64, ExitType)> {
    None
}

pub fn finalize_paper_registry(_registry: &mut PaperRegistry, _latest_prices: &HashMap<String, f64>) {}

pub fn close_active_trades_for_symbol(registry: &mut PaperRegistry, symbol: &str, candle: &Candle, _reason: &str) {
    let mut i = 0;
    while i < registry.active_trades.len() {
        if registry.active_trades[i].symbol == symbol {
             let trade = registry.active_trades.remove(i);
             let pnl = (candle.close as f64 / 10000.0 - trade.entry_price) / trade.entry_price;
             registry.pnl_history.push(pnl);
             registry.closed_count += 1;
        } else {
            i += 1;
        }
    }
}

pub fn close_active_sketch_trades_on_side_flip(_registry: &mut PaperRegistry, _symbol: &str, _new_side: SignalType, _minute_bucket: i64, _candle: &Candle) {}
