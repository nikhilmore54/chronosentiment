use crate::ga::{AlphaConsensus, SignalType, TradeRecommendation};
use crate::market_adapter::Candle;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::OnceLock;

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
    /// Linearized symbol update index when this intent was admitted (log / analysis only).
    #[serde(default)]
    pub intent_created_symbol_updates: usize,
    /// Candidate → confirm wait in symbol updates (live_engine confirmation path; 0 if unknown).
    #[serde(default)]
    pub confirm_delta_symbol_updates: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveTrade {
    pub rec_id: u64,
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
    pub expected_edge_bps: f64,
    /// Fixed TP/ladder volatility unit captured at entry for deterministic per-trade thresholds.
    #[serde(default)]
    pub tp_vol_unit: f64,
    pub max_pnl: f64,
    pub min_pnl: f64,
    pub min_pnl_first_10: f64,
    pub bars_to_mfe: usize,
    pub last_mark_pnl: f64,
    pub decay_count: usize,
    pub trailing_armed_seen: bool,
    pub exit_state: AdaptiveExitState,
    /// Bars in pullback since prior bar (`usize::MAX` = not in pullback episode; see end-of-bar tick).
    pub bars_since_pullback: usize,
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
    pub intent_age_at_fill: usize,
    #[serde(default)]
    pub momentum_3: f64,
    #[serde(default)]
    pub vol_5: f64,
    #[serde(default)]
    pub score_std_5: f64,
    #[serde(default)]
    pub partial_tp_done: bool,
    #[serde(default = "default_remaining_fraction")]
    pub remaining_fraction: f64,
    #[serde(default)]
    pub realized_partial_pnl: f64,
    #[serde(default)]
    pub trail_active: bool,
    #[serde(default)]
    pub trail_stop: f64,
    #[serde(default)]
    pub peak_price: f64,
    #[serde(default)]
    pub is_runner: bool,
    pub consensus: Option<AlphaConsensus>,
}

fn default_remaining_fraction() -> f64 {
    1.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClosedTradeObservation {
    pub rank: f64,
    pub vol_bucket: usize,
    pub mfe: f64,
    pub mae_abs: f64,
    pub hold_bars: usize,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum AdaptiveExitState {
    ArmedObserved,
    PullbackCandidate,
    ConfirmedBreak,
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
    pub rank_pnl_sum: [f64; 10],
    pub rank_count: [usize; 10],
    pub vol_pnl_sum: [f64; 5],
    pub vol_count: [usize; 5],
    pub rolling_peak: f64,
    pub adaptation_threshold: usize,
    pub trade_counts_per_strat: HashMap<usize, usize>,
    /// Rolling `(high-low)/close` per symbol for volatility-scaled paper SL (not serialized).
    #[serde(skip)]
    pub symbol_bar_ranges: HashMap<String, VecDeque<f64>>,
    /// Closed-trade observations for online rank-stats warmup (live engine drains each tick).
    #[serde(skip)]
    pub closed_observations: Vec<ClosedTradeObservation>,
    /// Intent lifecycle diagnostics for deterministic entry-state debugging.
    #[serde(default)]
    pub intents_created: usize,
    #[serde(default)]
    pub intents_triggered: usize,
    #[serde(default)]
    pub intents_expired: usize,
    #[serde(default)]
    pub intents_expiry_age_sum: usize,
    /// Shadow-mode trigger volatility samples for online percentile logging (not serialized).
    #[serde(skip)]
    pub shadow_trigger_vol_samples: Vec<f64>,
    /// Shadow-mode trigger momentum samples for online percentile logging (not serialized).
    #[serde(skip)]
    pub shadow_trigger_mom_samples: Vec<f64>,
}

impl PaperRegistry {
    pub fn summary(&self) {
        let win_rate = if self.closed_count > 0 {
            self.wins as f64 / self.closed_count as f64
        } else {
            0.0
        };
        let avg_pnl = if !self.pnl_history.is_empty() {
            self.pnl_history.iter().sum::<f64>() / self.pnl_history.len() as f64
        } else {
            0.0
        };
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
                println!(
                    "Rank {:.1} : count={} avg_pnl={:.6}",
                    r as f64 / 10.0,
                    self.rank_count[r],
                    avg
                );
            }
        }
        println!("---------------------------");
        let avg_expiry_age = if self.intents_expired > 0 {
            self.intents_expiry_age_sum as f64 / self.intents_expired as f64
        } else {
            0.0
        };
        println!(
            "INTENT_STATS created={} triggered={} expired={} pending_final={} avg_expiry_age={:.2}",
            self.intents_created,
            self.intents_triggered,
            self.intents_expired,
            self.pending_intents.len(),
            avg_expiry_age
        );

        let ev_hits = PAPER_EV_LOOKUP_HIT.load(Ordering::Relaxed);
        let ev_miss = PAPER_EV_LOOKUP_MISS.load(Ordering::Relaxed);
        let ev_total = ev_hits + ev_miss;
        if ev_total > 0 {
            println!(
                "[PAPER_EV] lookup hit_rate={:.4} (hits={} misses={})",
                ev_hits as f64 / ev_total as f64,
                ev_hits,
                ev_miss
            );
        }
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
            symbol_bar_ranges: HashMap::new(),
            closed_observations: Vec::new(),
            intents_created: 0,
            intents_triggered: 0,
            intents_expired: 0,
            intents_expiry_age_sum: 0,
            shadow_trigger_vol_samples: Vec::new(),
            shadow_trigger_mom_samples: Vec::new(),
        }
    }
}

fn linear_percentile_f64(sorted_vals: &[f64], p: f64) -> f64 {
    if sorted_vals.is_empty() {
        return 0.0;
    }
    if sorted_vals.len() == 1 {
        return sorted_vals[0];
    }
    let n = sorted_vals.len() as f64;
    let idx = (n - 1.0) * (p / 100.0);
    let lo = idx.floor() as usize;
    let hi = (lo + 1).min(sorted_vals.len() - 1);
    let w = idx - lo as f64;
    sorted_vals[lo] * (1.0 - w) + sorted_vals[hi] * w
}

fn percentile_rank(sorted_vals: &[f64], value: f64) -> f64 {
    if sorted_vals.is_empty() {
        return 0.0;
    }
    let mut le_count = 0usize;
    for v in sorted_vals {
        if *v <= value {
            le_count += 1;
        }
    }
    100.0 * (le_count as f64 / sorted_vals.len() as f64)
}

#[derive(Debug, Clone, Copy)]
pub enum ExitType {
    TakeProfit,
    StopLoss,
    Ambiguous,
}

pub fn apply_slippage(price: f64, is_buy: bool, vol_bps: f64) -> f64 {
    let base_bps = 2.0;
    let dynamic_bps = base_bps + (vol_bps * 0.1).min(10.0);
    let factor = dynamic_bps / 10000.0;
    if is_buy { price * (1.0 + factor) } else { price * (1.0 - factor) }
}

fn paper_vol_bucket_from_bps(vol_bps: f64) -> usize {
    (((vol_bps - 10.0) / 15.0).round().clamp(0.0, 4.0)) as usize
}

/// First bar after open (`current_hold == 1` after increment): suppress intrabar SL so the
/// candle can resolve without worst-case SL-before-recovery (deterministic; TP still allowed).
fn intrabar_exit_respecting_entry_bar(
    current_hold: usize,
    raw: Option<ExitType>,
) -> Option<ExitType> {
    if current_hold == 1 {
        match raw {
            // Entry bar: no worst-case SL / ambiguous intrabar exit; bar close + next bars apply SL.
            Some(ExitType::StopLoss) | Some(ExitType::Ambiguous) => None,
            other => other,
        }
    } else {
        raw
    }
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

/// Average `(high-low)/close` over the deque for `symbol`, else `fallback` (e.g. current bar).
fn paper_avg_bar_range(registry: &PaperRegistry, symbol: &str, fallback: f64) -> f64 {
    registry
        .symbol_bar_ranges
        .get(symbol)
        .filter(|d| !d.is_empty())
        .map(|d| d.iter().sum::<f64>() / d.len() as f64)
        .unwrap_or(fallback)
}

/// Stop price from clamped average range × multiplier (deterministic).
fn paper_vol_sl_price(
    entry: f64,
    is_long: bool,
    avg_range: f64,
    mult: f64,
    range_lo: f64,
    range_hi: f64,
) -> f64 {
    let vol = avg_range.clamp(range_lo, range_hi);
    let dist = mult * vol;
    if is_long {
        entry * (1.0 - dist)
    } else {
        entry * (1.0 + dist)
    }
}

/// Take-profit price from the same clamped range × multiplier (deterministic; TP distance typically ≤ SL distance).
fn paper_vol_tp_price(
    entry: f64,
    is_long: bool,
    avg_range: f64,
    mult: f64,
    range_lo: f64,
    range_hi: f64,
) -> f64 {
    let vol = avg_range.clamp(range_lo, range_hi);
    let dist = mult * vol;
    if is_long {
        entry * (1.0 + dist)
    } else {
        entry * (1.0 - dist)
    }
}

/// After `max_pnl` exceeds `threshold`, ratchet stop to entry so a micro-MFE cannot become a large loss.
fn apply_mfe_breakeven_lock(trade: &mut ActiveTrade, is_long: bool, threshold: f64, enabled: bool) {
    if !enabled || trade.max_pnl <= threshold {
        return;
    }
    if is_long {
        trade.sl_target = trade.sl_target.max(trade.entry_price);
    } else {
        trade.sl_target = trade.sl_target.min(trade.entry_price);
    }
}

/// Close-based and intrabar trail touch (same timing class as TP/SL intrabar). If the bar
/// pierces `trail_stop` but closes back through, exit PnL is priced at the stop (slippage).
fn paper_trail_touch_exit_pnl(
    entry_price: f64,
    trail_stop: f64,
    is_long: bool,
    high: f64,
    low: f64,
    close: f64,
    mark_pnl: f64,
    vol_bps: f64,
) -> Option<f64> {
    let trail_hit_close = if is_long {
        close <= trail_stop
    } else {
        close >= trail_stop
    };
    let trail_hit_intrabar = if is_long {
        low <= trail_stop
    } else {
        high >= trail_stop
    };
    if !trail_hit_close && !trail_hit_intrabar {
        return None;
    }
    if trail_hit_intrabar && !trail_hit_close {
        let px = apply_slippage(trail_stop, !is_long, vol_bps);
        Some(if is_long {
            (px - entry_price) / entry_price
        } else {
            (entry_price - px) / entry_price
        })
    } else {
        Some(mark_pnl)
    }
}

/// One cell from `analysis/ev_table.json` (built by `scripts/build_ev_table.py`).
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct PaperEvCell {
    count: u64,
    p_survive: f64,
    avg_pnl: f64,
    ev: f64,
}

type PaperEvTable = HashMap<String, HashMap<String, PaperEvCell>>;

static PAPER_EV_TABLE_CACHE: OnceLock<Option<PaperEvTable>> = OnceLock::new();

/// Budget for `[PAPER_EV] lookup` lines when `PAPER_EV_LOOKUP_DEBUG` is set (avoids flooding huge replays).
static PAPER_EV_LOOKUP_DEBUG_BUDGET: AtomicUsize = AtomicUsize::new(50);

/// Counts intent-expiry EV key lookups against `PAPER_EV_TABLE_PATH` (cell present vs missing).
static PAPER_EV_LOOKUP_HIT: AtomicUsize = AtomicUsize::new(0);
static PAPER_EV_LOOKUP_MISS: AtomicUsize = AtomicUsize::new(0);

fn paper_ev_table_cached() -> Option<&'static PaperEvTable> {
    PAPER_EV_TABLE_CACHE
        .get_or_init(|| {
            let path = std::env::var("PAPER_EV_TABLE_PATH").ok()?;
            if path.is_empty() {
                return None;
            }
            let data = std::fs::read_to_string(&path).ok()?;
            let table: PaperEvTable = serde_json::from_str(&data).ok()?;
            let n_age = table.len();
            let mut age_keys: Vec<String> = table.keys().cloned().collect();
            age_keys.sort();
            let keys_sample: Vec<String> = age_keys.into_iter().take(5).collect();
            let (pa, pc) = paper_intent_lookup_keys(0, 5, 0.65);
            println!(
                "[PAPER_EV] loaded ev_table from {} ({} age-bucket keys) sample_age_keys={:?} probe(age=0,voters=5,conf=0.65) => {} | {}",
                path, n_age, keys_sample, pa, pc
            );
            Some(table)
        })
        .as_ref()
}

/// Integer bin label (`low_high` or `last_plus`); must match `scripts/build_ev_table.py::bucket_int`.
fn paper_bucket_int(mut v: i32, bins: &[i32]) -> String {
    if bins.len() < 2 {
        return "0_plus".to_string();
    }
    if v < bins[0] {
        v = bins[0];
    }
    for i in 0..bins.len() - 1 {
        if v >= bins[i] && v < bins[i + 1] {
            return format!("{}_{}", bins[i], bins[i + 1]);
        }
    }
    format!("{}_plus", bins[bins.len() - 1])
}

fn paper_intent_lookup_keys(age: usize, _voters: usize, conf: f64) -> (String, String) {
    let conf_cent = (conf * 100.0).round() as i32;
    let conf_cent = conf_cent.clamp(0, 100);
    (
        paper_bucket_int(age as i32, &[0, 2, 5, 10, 20]),
        paper_bucket_int(conf_cent, &[50, 60, 70, 80, 100]),
    )
}

/// Returns true when a loaded EV table has a cell for this state and `ev` is below the threshold.
fn paper_intent_ev_low(
    age: usize,
    voters: usize,
    conf: f64,
    threshold: f64,
    table: &PaperEvTable,
) -> bool {
    let (age_b, conf_b) = paper_intent_lookup_keys(age, voters, conf);
    if std::env::var("PAPER_EV_LOOKUP_DEBUG").is_ok() {
        let left = PAPER_EV_LOOKUP_DEBUG_BUDGET.fetch_sub(1, Ordering::Relaxed);
        if left > 0 {
            println!(
                "[PAPER_EV] lookup age_b={} conf_b={}",
                age_b, conf_b
            );
        }
    }
    let cell = table.get(&age_b).and_then(|m| m.get(&conf_b));
    match cell {
        Some(c) => {
            PAPER_EV_LOOKUP_HIT.fetch_add(1, Ordering::Relaxed);
            c.ev < threshold
        }
        None => {
            PAPER_EV_LOOKUP_MISS.fetch_add(1, Ordering::Relaxed);
            std::env::var("PAPER_EV_MISS_IS_LOW")
                .map(|v| !v.is_empty() && v != "0" && !v.eq_ignore_ascii_case("false"))
                .unwrap_or(false)
        }
    }
}

/// `symbol_linear_updates`: monotonic per-symbol update count after applying `latest_candle` (replay-stable clock).
pub fn update_paper_registry(
    registry: &mut PaperRegistry,
    latest_candle: &Candle,
    symbol: &str,
    symbol_linear_updates: usize,
    trigger_momentum_3: f64,
    trigger_vol_5: f64,
) {
    let exit_mode = std::env::var("PAPER_EXIT_MODE")
        .unwrap_or_else(|_| "default".to_string())
        .to_lowercase();
    let time_exit_only = exit_mode == "time";
    let hybrid_exit = exit_mode == "hybrid";
    let tpsl_only = exit_mode == "tpsl_only";
    let delayed_exit = exit_mode == "delayed";
    let hybrid_delayed_exit = exit_mode == "hybrid_delayed";
    let signal_exit = exit_mode == "signal";
    let trailing_exit = exit_mode == "trailing";
    let trailing_adaptive_exit = exit_mode == "trailing_adaptive";
    // Intrabar TP/SL from recommendation targets (vol-scaled when enabled), not hybrid/trailing/time-only.
    let default_tpsl_path = !time_exit_only
        && !trailing_adaptive_exit
        && !trailing_exit
        && !signal_exit
        && !hybrid_delayed_exit
        && !delayed_exit
        && !hybrid_exit;
    let fixed_hold_bars = std::env::var("PAPER_FIXED_HOLD_BARS")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(30)
        .max(1);
    let tp_min_hold_bars = std::env::var("PAPER_TP_MIN_HOLD")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(1)
        .max(1);
    let partial_tp_min_hold_bars = std::env::var("PAPER_PARTIAL_TP_MIN_HOLD")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(3)
        .max(1);
    let partial_tp_fraction = std::env::var("PAPER_PARTIAL_TP_FRACTION")
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(0.4)
        .clamp(0.0, 0.9);
    let min_hold_bars = std::env::var("PAPER_MIN_HOLD_BARS")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(1)
        .max(1);
    let hybrid_tp = std::env::var("PAPER_HYBRID_TP")
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(0.0030)
        .max(0.0);
    let hybrid_sl = std::env::var("PAPER_HYBRID_SL")
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(-0.0015)
        .min(0.0);
    let delayed_sl = std::env::var("PAPER_DELAYED_SL")
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(-0.0015)
        .min(0.0);
    let delayed_min_bars = std::env::var("PAPER_DELAYED_MIN_BARS")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(10)
        .max(1);
    let adaptive_tp = std::env::var("PAPER_ADAPTIVE_TP")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true") || v.eq_ignore_ascii_case("edge"))
        .unwrap_or(false);
    let signal_decay_rank = std::env::var("PAPER_SIGNAL_DECAY_RANK")
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(0.0005)
        .max(0.0);
    let trailing_dd = std::env::var("PAPER_TRAILING_DD")
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(0.01)
        .max(0.0);
    let trailing_min_bars = std::env::var("PAPER_TRAILING_MIN_BARS")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(10)
        .max(1);
    let trailing_dd_strong = std::env::var("PAPER_TRAILING_DD_STRONG")
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(0.015)
        .max(0.0);
    let trailing_dd_weak = std::env::var("PAPER_TRAILING_DD_WEAK")
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(0.008)
        .max(0.0);
    let trailing_arm_mfe = std::env::var("PAPER_TRAILING_ARM_MFE")
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(0.004)
        .max(0.0);
    let trailing_good_mfe_bars = std::env::var("PAPER_TRAILING_GOOD_MFE_BARS")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(12)
        .max(1);
    let trailing_good_mae_cut = std::env::var("PAPER_TRAILING_GOOD_MAE_CUT")
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(-0.015);
    let trailing_decay_dd = std::env::var("PAPER_TRAILING_DECAY_DD")
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(0.008)
        .max(0.0);
    let trailing_decay_strong_dd = std::env::var("PAPER_TRAILING_DECAY_STRONG_DD")
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(0.013)
        .max(0.0);
    let trailing_decay_weak_confirm = std::env::var("PAPER_TRAILING_DECAY_WEAK_CONFIRM")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(3)
        .max(1);
    let trailing_decay_slope = std::env::var("PAPER_TRAILING_DECAY_SLOPE")
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(0.004)
        .max(0.0);
    let trailing_pullback_timeout_bars = std::env::var("PAPER_TRAILING_PULLBACK_TIMEOUT_BARS")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(6)
        .max(1);
    let trailing_decay_peak_bars = std::env::var("PAPER_TRAILING_DECAY_PEAK_BARS")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(2)
        .max(1);
    let trailing_continuation_peak_bars = std::env::var("PAPER_TRAILING_CONT_PEAK_BARS")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(4)
        .max(1);
    let trailing_arm_bars_loose = std::env::var("PAPER_TRAILING_ARM_BARS_LOOSE")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(7)
        .max(1);
    // Adaptive trailing (K8): recovery veto + weak-path strong-DD gates (PAPER_K83_*).
    let k83_recovery_frac = std::env::var("PAPER_K83_RECOVERY_FRAC")
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(0.6)
        .clamp(0.0, 1.0);
    let k83_early_peak_bars = std::env::var("PAPER_K83_EARLY_PEAK_BARS")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(3)
        .max(1);
    let k83_early_dd_max = std::env::var("PAPER_K83_EARLY_DD_MAX")
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(0.02)
        .max(0.0);
    let k83_peak_hold_frac = std::env::var("PAPER_K83_PEAK_HOLD_FRAC")
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(0.4)
        .clamp(0.0, 1.0);
    let k83_strong_dd_extra = std::env::var("PAPER_K83_STRONG_DD_EXTRA")
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(0.003)
        .max(0.0);
    let k83_weak_path_decay_min = std::env::var("PAPER_K83_WEAK_PATH_DECAY_MIN")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(4)
        .max(1);
    let high = latest_candle.high as f64;
    let low = latest_candle.low as f64;
    let close = latest_candle.close as f64;
    let ts = latest_candle.timestamp;
    let exit_probe = std::env::var("EXIT_PROBE").is_ok();

    let bar_range = (high - low) / close.max(1e-12);
    let vol_range_lookback = std::env::var("PAPER_VOL_RANGE_LOOKBACK")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(5)
        .max(1);
    {
        let dq = registry
            .symbol_bar_ranges
            .entry(symbol.to_string())
            .or_insert_with(VecDeque::new);
        if dq.len() >= vol_range_lookback {
            dq.pop_front();
        }
        dq.push_back(bar_range);
    }
    let paper_vol_sl = std::env::var("PAPER_VOL_SL")
        .map(|v| {
            !(v == "0" || v.eq_ignore_ascii_case("false") || v.eq_ignore_ascii_case("off"))
        })
        .unwrap_or(true);
    let paper_vol_sl_mult = std::env::var("PAPER_VOL_SL_MULT")
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(1.2)
        .max(0.0);
    let paper_vol_range_lo = std::env::var("PAPER_VOL_RANGE_LO")
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(0.0005)
        .max(1e-9);
    let paper_vol_range_hi = std::env::var("PAPER_VOL_RANGE_HI")
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(0.01)
        .max(paper_vol_range_lo);
    // TP uses a tighter high clamp than SL so micro-edge TP can live inside typical MFE.
    let paper_vol_tp_range_hi = std::env::var("PAPER_VOL_TP_RANGE_HI")
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(0.0018)
        .max(paper_vol_range_lo)
        .min(paper_vol_range_hi);
    let paper_mfe_lock = std::env::var("PAPER_MFE_LOCK")
        .map(|v| {
            !(v == "0" || v.eq_ignore_ascii_case("false") || v.eq_ignore_ascii_case("off"))
        })
        .unwrap_or(true);
    let paper_mfe_lock_threshold = std::env::var("PAPER_MFE_LOCK_THRESHOLD")
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(0.0007)
        .max(0.0);
    let paper_vol_tp = std::env::var("PAPER_VOL_TP")
        .map(|v| {
            !(v == "0" || v.eq_ignore_ascii_case("false") || v.eq_ignore_ascii_case("off"))
        })
        .unwrap_or(true);
    // Default < SL multiplier so TP often sits inside one-bar favorable excursion (tune via env).
    let paper_vol_tp_mult = std::env::var("PAPER_VOL_TP_MULT")
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(0.5)
        .max(0.0);
    let paper_tp_strength_gated = std::env::var("PAPER_TP_STRENGTH_GATED")
        .map(|v| !(v == "0" || v.eq_ignore_ascii_case("false") || v.eq_ignore_ascii_case("off")))
        .unwrap_or(true);
    let paper_tp_alpha = std::env::var("PAPER_TP_ALPHA")
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(0.8)
        .clamp(0.5, 1.5);
    let paper_tp_use_touch_fallback = std::env::var("PAPER_TP_USE_TOUCH_FALLBACK")
        .map(|v| !(v == "0" || v.eq_ignore_ascii_case("false") || v.eq_ignore_ascii_case("off")))
        .unwrap_or(false);
    let paper_tp_ladder = std::env::var("PAPER_TP_LADDER")
        .map(|v| !(v == "0" || v.eq_ignore_ascii_case("false") || v.eq_ignore_ascii_case("off")))
        .unwrap_or(false);
    let paper_tp_ladder_t1 = std::env::var("PAPER_TP_LADDER_T1")
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(0.5)
        .clamp(0.1, 3.0);
    let paper_tp_ladder_t2 = std::env::var("PAPER_TP_LADDER_T2")
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(1.0)
        .clamp(0.2, 4.0);
    let paper_tp_ladder_lock1 = std::env::var("PAPER_TP_LADDER_LOCK1")
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(0.0)
        .clamp(-1.0, 3.0);
    let paper_tp_ladder_lock2 = std::env::var("PAPER_TP_LADDER_LOCK2")
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(0.5)
        .clamp(-1.0, 4.0);
    let paper_tp_ladder_allow_tp_at_t2 = std::env::var("PAPER_TP_LADDER_ALLOW_TP_AT_T2")
        .map(|v| !(v == "0" || v.eq_ignore_ascii_case("false") || v.eq_ignore_ascii_case("off")))
        .unwrap_or(false);
    let paper_partial_tp = std::env::var("PAPER_PARTIAL_TP")
        .map(|v| !(v == "0" || v.eq_ignore_ascii_case("false") || v.eq_ignore_ascii_case("off")))
        .unwrap_or(true);
    let paper_partial_tp_min_hold = std::env::var("PAPER_PARTIAL_TP_MIN_HOLD")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(3)
        .max(1);
    let paper_partial_tp_min_ret = std::env::var("PAPER_PARTIAL_TP_MIN_RET")
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(0.0002)
        .max(0.0);
    let paper_partial_tp_fraction = std::env::var("PAPER_PARTIAL_TP_FRACTION")
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(0.4)
        .clamp(0.1, 0.9);
    let paper_trail_enable = std::env::var("PAPER_TRAIL_ENABLE")
        .map(|v| !(v == "0" || v.eq_ignore_ascii_case("false") || v.eq_ignore_ascii_case("off")))
        .unwrap_or(false);
    let paper_trail_type = std::env::var("PAPER_TRAIL_TYPE")
        .unwrap_or_else(|_| "atr".to_string())
        .to_lowercase();
    let paper_trail_atr_mult = std::env::var("PAPER_TRAIL_ATR_MULT")
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(1.5)
        .max(0.1);
    let paper_trail_pct = std::env::var("PAPER_TRAIL_PCT")
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(0.002)
        .max(1e-6);
    let paper_trail_breakeven = std::env::var("PAPER_TRAIL_BREAKEVEN")
        .map(|v| !(v == "0" || v.eq_ignore_ascii_case("false") || v.eq_ignore_ascii_case("off")))
        .unwrap_or(true);
    let paper_runner_mode = std::env::var("PAPER_RUNNER_MODE")
        .map(|v| !(v == "0" || v.eq_ignore_ascii_case("false") || v.eq_ignore_ascii_case("off")))
        .unwrap_or(true);
    let paper_runner_min_hold = std::env::var("PAPER_RUNNER_MIN_HOLD")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(2)
        .max(1);
    let paper_runner_ret_threshold = std::env::var("PAPER_RUNNER_RET_THRESHOLD")
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(0.001)
        .max(0.0);
    let paper_intent_decay_min_score = std::env::var("PAPER_INTENT_DECAY_MIN_SCORE")
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(0.0002)
        .max(0.0);
    let paper_intent_pullback_factor = std::env::var("PAPER_INTENT_PULLBACK_FACTOR")
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(0.999);
    let paper_intent_bounce_factor = std::env::var("PAPER_INTENT_BOUNCE_FACTOR")
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(1.001);
    let paper_ev_expire_threshold = std::env::var("PAPER_EV_EXPIRE_THRESHOLD")
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(0.0002);

    let mut j = 0;
    while j < registry.pending_intents.len() {
        if registry.pending_intents[j].symbol != symbol {
            j += 1;
            continue;
        }
        let mut triggered = false;
        let mut entry_price = 0.0;
        {
            let intent = &mut registry.pending_intents[j];
            intent.age += 1;
            let is_long = intent.signal == SignalType::BUY;
            let pullback_factor = paper_intent_pullback_factor;
            let bounce_factor = paper_intent_bounce_factor;
            let mut touch_trigger = false;
            let mut touch_entry_price = 0.0;
            if is_long {
                if low <= intent.reference_price * pullback_factor {
                    touch_trigger = true;
                    touch_entry_price = intent.reference_price * pullback_factor;
                }
            } else if high >= intent.reference_price * bounce_factor {
                touch_trigger = true;
                touch_entry_price = intent.reference_price * bounce_factor;
            }
            let ev_expire = paper_ev_table_cached().is_some_and(|t| {
                paper_intent_ev_low(
                    intent.age,
                    intent.rec_voters,
                    intent.rec_conf,
                    paper_ev_expire_threshold,
                    t,
                )
            });
            if intent.age > intent.max_age || ev_expire {
                registry.intents_expired = registry.intents_expired.saturating_add(1);
                registry.intents_expiry_age_sum =
                    registry.intents_expiry_age_sum.saturating_add(intent.age);
                registry.pending_intents.remove(j);
                continue;
            }
            if touch_trigger {
                // Time-decayed intent quality: later fills require stronger original rec score.
                let age_ratio =
                    (intent.age as f64 / intent.max_age.max(1) as f64).clamp(0.0, 1.0);
                let decay = 1.0 - age_ratio;
                let adjusted_score = intent.rec_score * decay;
                if adjusted_score >= paper_intent_decay_min_score {
                    triggered = true;
                    entry_price = touch_entry_price;
                }
            }
        }
        if triggered {
            registry.intents_triggered = registry.intents_triggered.saturating_add(1);
            let intent = registry.pending_intents.remove(j);
            let intent_age_updates = symbol_linear_updates.saturating_sub(intent.intent_created_symbol_updates);
            eprintln!(
                "[TRIGGER_METRICS] sym={} rec_id={} confirm_updates={} intent_age_updates={} intent_age_bars={}",
                intent.symbol,
                intent.rec_id,
                intent.confirm_delta_symbol_updates,
                intent_age_updates,
                intent.age
            );
            eprintln!(
                "[TRIGGER_STATE] rec_id={} confirm_updates={} intent_age_updates={} trigger_momentum_3={:.6} trigger_vol_5={:.6}",
                intent.rec_id,
                intent.confirm_delta_symbol_updates,
                intent_age_updates,
                trigger_momentum_3,
                trigger_vol_5
            );
            // Shadow-mode decision telemetry (log-only): compute online percentiles from seen triggers.
            registry.shadow_trigger_vol_samples.push(trigger_vol_5);
            registry.shadow_trigger_mom_samples.push(trigger_momentum_3);
            let mut vol_sorted = registry.shadow_trigger_vol_samples.clone();
            vol_sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            let mut mom_sorted = registry.shadow_trigger_mom_samples.clone();
            mom_sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            let vol_p85 = linear_percentile_f64(&vol_sorted, 85.0);
            let mom_p15 = linear_percentile_f64(&mom_sorted, 15.0);
            let vol_percentile = percentile_rank(&vol_sorted, trigger_vol_5);
            let mom_percentile = percentile_rank(&mom_sorted, trigger_momentum_3);
            let would_block_vol = trigger_vol_5 > vol_p85;
            let would_block_mom = trigger_momentum_3 <= mom_p15;
            let would_block_any = would_block_vol || would_block_mom;
            let reason = match (would_block_vol, would_block_mom) {
                (true, true) => "both",
                (true, false) => "extreme_volatility",
                (false, true) => "extreme_negative_momentum",
                (false, false) => "none",
            };
            eprintln!(
                "[SHADOW_DECISION] rec_id={} would_block_vol={} would_block_mom={} would_block_any={} reason={} vol_percentile={:.2} mom_percentile={:.2}",
                intent.rec_id,
                if would_block_vol { 1 } else { 0 },
                if would_block_mom { 1 } else { 0 },
                if would_block_any { 1 } else { 0 },
                reason,
                vol_percentile,
                mom_percentile
            );
            let is_long_entry = intent.signal == SignalType::BUY;
            let avg_r_entry = paper_avg_bar_range(registry, symbol, bar_range);
            // TP uses the tighter of rolling vs entry bar range so TP can sit inside a typical one-bar move.
            let tp_vol_input = avg_r_entry.min(bar_range);
            let sl_target = if paper_vol_sl {
                paper_vol_sl_price(
                    entry_price,
                    is_long_entry,
                    avg_r_entry,
                    paper_vol_sl_mult,
                    paper_vol_range_lo,
                    paper_vol_range_hi,
                )
            } else {
                intent.recommendation.sl_target
            };
            let tp_target = if paper_vol_tp {
                paper_vol_tp_price(
                    entry_price,
                    is_long_entry,
                    tp_vol_input,
                    paper_vol_tp_mult,
                    paper_vol_range_lo,
                    paper_vol_tp_range_hi,
                )
            } else {
                intent.recommendation.tp_target
            };
            registry.active_trades.push(ActiveTrade {
                rec_id: intent.rec_id,
                symbol: intent.symbol,
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
                tp_vol_unit: tp_vol_input.clamp(paper_vol_range_lo, paper_vol_tp_range_hi),
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
                consensus: intent.consensus,
            });
        } else {
            j += 1;
        }
    }

    let mut i = 0;
    while i < registry.active_trades.len() {
        if registry.active_trades[i].symbol != symbol {
            i += 1;
            continue;
        }
        let tp_vol_base = bar_range
            .min(paper_avg_bar_range(registry, symbol, bar_range))
            .clamp(paper_vol_range_lo, paper_vol_tp_range_hi);
        let trade = &mut registry.active_trades[i];
        trade.current_hold += 1;
        let mut exit_pnl = None;
        let mut exit_tag = "NONE";
        let is_long = trade.signal == SignalType::BUY;
        let bar_best = if is_long {
            (high - trade.entry_price) / trade.entry_price.max(1e-12)
        } else {
            (trade.entry_price - low) / trade.entry_price.max(1e-12)
        };
        let bar_worst = if is_long {
            (low - trade.entry_price) / trade.entry_price.max(1e-12)
        } else {
            (trade.entry_price - high) / trade.entry_price.max(1e-12)
        };
        if bar_best > trade.max_pnl {
            trade.max_pnl = bar_best;
            trade.bars_to_mfe = trade.current_hold;
            trade.decay_count = 0;
        }
        trade.min_pnl = trade.min_pnl.min(bar_worst);
        let mark_exit = apply_slippage(close, !is_long, trade.vol_bps);
        let mark_pnl = if is_long {
            (mark_exit - trade.entry_price) / trade.entry_price
        } else {
            (trade.entry_price - mark_exit) / trade.entry_price
        };
        let momentum_positive = mark_pnl >= trade.last_mark_pnl;
        if paper_runner_mode
            && !trade.is_runner
            && trade.current_hold >= paper_runner_min_hold
            && mark_pnl >= paper_runner_ret_threshold
            && momentum_positive
        {
            trade.is_runner = true;
        }
        // Runners never use take-profit exits; trailing / SL / time handle remainder (see intrabar + hybrid paths).
        let tp_allowed_global = !trade.is_runner;
        if is_long {
            trade.peak_price = trade.peak_price.max(close);
        } else {
            trade.peak_price = trade.peak_price.min(close);
        }

        // Partial TP proxy: after a short hold and sufficient favorable move, lock a fraction of
        // volatility-scaled gains in stop level while keeping the trade alive for full TP window.
        if paper_partial_tp
            && !trade.partial_tp_done
            && trade.current_hold >= paper_partial_tp_min_hold
        {
            if mark_pnl >= paper_partial_tp_min_ret {
                let frac = paper_partial_tp_fraction.clamp(0.0, 0.95);
                trade.realized_partial_pnl += frac * mark_pnl;
                trade.remaining_fraction = (1.0 - frac).max(0.05);
                let lock_ret = (paper_partial_tp_fraction * mark_pnl).max(0.0);
                let lock_price = if is_long {
                    trade.entry_price * (1.0 + lock_ret)
                } else {
                    trade.entry_price * (1.0 - lock_ret)
                };
                trade.sl_target = if is_long {
                    trade.sl_target.max(lock_price)
                } else {
                    trade.sl_target.min(lock_price)
                };
                trade.partial_tp_done = true;
            }
        }
        if paper_trail_enable
            && trade.is_runner
            && (trade.partial_tp_done || trade.current_hold >= 2)
            && trade.remaining_fraction > 0.0
        {
            // Runner classification already encodes strength; arm trail from bar 2 (no second profit gate).
            if trade.current_hold >= 2 {
                trade.trail_active = true;
            }
            if trade.trail_active {
                let candidate_stop = if paper_trail_type == "percent" {
                    if is_long {
                        trade.peak_price * (1.0 - paper_trail_pct)
                    } else {
                        trade.peak_price * (1.0 + paper_trail_pct)
                    }
                } else {
                    let atr_proxy = (trade.tp_vol_unit * trade.entry_price).max(1e-9);
                    if is_long {
                        trade.peak_price - (paper_trail_atr_mult * atr_proxy)
                    } else {
                        trade.peak_price + (paper_trail_atr_mult * atr_proxy)
                    }
                };
                if is_long {
                    trade.trail_stop = trade.trail_stop.max(candidate_stop);
                    if paper_trail_breakeven {
                        trade.trail_stop = trade.trail_stop.max(trade.entry_price);
                    }
                } else {
                    trade.trail_stop = trade.trail_stop.min(candidate_stop);
                    if paper_trail_breakeven {
                        trade.trail_stop = trade.trail_stop.min(trade.entry_price);
                    }
                }
            }
        }
        // Runner + paper trail: evaluate trail before any TP path so close-based trail competes with TP.
        if paper_trail_enable
            && trade.is_runner
            && trade.trail_active
            && trade.remaining_fraction > 0.0
            && trade.current_hold >= min_hold_bars
        {
            if let Some(pnl) = paper_trail_touch_exit_pnl(
                trade.entry_price,
                trade.trail_stop,
                is_long,
                high,
                low,
                close,
                mark_pnl,
                trade.vol_bps,
            ) {
                exit_pnl = Some(pnl);
                exit_tag = "TRAIL";
            }
        }
        if mark_pnl > trade.max_pnl {
            trade.max_pnl = mark_pnl;
            // Track true MFE timing (latest peak), not first improvement.
            trade.bars_to_mfe = trade.current_hold;
            trade.decay_count = 0;
        } else {
            if trade.current_hold > 1 && mark_pnl < trade.last_mark_pnl {
                trade.decay_count = trade.decay_count.saturating_add(1);
            } else {
                trade.decay_count = 0;
            }
        }
        trade.min_pnl = trade.min_pnl.min(mark_pnl);
        if trade.current_hold <= 10 {
            trade.min_pnl_first_10 = trade.min_pnl_first_10.min(mark_pnl);
        }
        let mut tp_touch = 0i32;
        let mut tp_strength_hit = 0i32;
        let trade_tp_vol_unit = if trade.tp_vol_unit > 1e-12 {
            trade.tp_vol_unit
        } else {
            tp_vol_base
        };
        let tp_strength_ret = paper_tp_alpha * trade_tp_vol_unit;
        let t1_ret = paper_tp_ladder_t1 * trade_tp_vol_unit;
        let t2_ret = paper_tp_ladder_t2 * trade_tp_vol_unit;
        let stage1_hit = trade.max_pnl >= t1_ret;
        let stage2_hit = trade.max_pnl >= t2_ret;
        let ladder_tp_enabled = paper_tp_ladder && paper_tp_ladder_allow_tp_at_t2 && stage2_hit;
        let mut stage1_applied = 0i32;
        let mut stage2_applied = 0i32;
        if default_tpsl_path && paper_tp_ladder {
            let prev_sl = trade.sl_target;
            let lock1_price = if is_long {
                trade.entry_price * (1.0 + paper_tp_ladder_lock1 * trade_tp_vol_unit)
            } else {
                trade.entry_price * (1.0 - paper_tp_ladder_lock1 * trade_tp_vol_unit)
            };
            let lock2_price = if is_long {
                trade.entry_price * (1.0 + paper_tp_ladder_lock2 * trade_tp_vol_unit)
            } else {
                trade.entry_price * (1.0 - paper_tp_ladder_lock2 * trade_tp_vol_unit)
            };
            if stage1_hit {
                trade.sl_target = if is_long {
                    trade.sl_target.max(lock1_price)
                } else {
                    trade.sl_target.min(lock1_price)
                };
                if (trade.sl_target - prev_sl).abs() > 1e-12 {
                    stage1_applied = 1;
                }
            }
            let prev_sl_after_t1 = trade.sl_target;
            if stage2_hit {
                trade.sl_target = if is_long {
                    trade.sl_target.max(lock2_price)
                } else {
                    trade.sl_target.min(lock2_price)
                };
                if (trade.sl_target - prev_sl_after_t1).abs() > 1e-12 {
                    stage2_applied = 1;
                }
            }
        }

        // Default TP/SL path: MFE update -> strength-gated TP -> breakeven lock -> intrabar SL/TP.
        if default_tpsl_path && (!paper_tp_ladder || ladder_tp_enabled) {
            let tp_hit_pre = if is_long {
                high >= trade.tp_target
            } else {
                low <= trade.tp_target
            };
            tp_touch = tp_hit_pre as i32;
            let tp_has_strength = trade.max_pnl >= tp_strength_ret;
            tp_strength_hit = tp_has_strength as i32;
            // Strength-gated TP is only valid when TP target comes from the same vol unit.
            // If PAPER_VOL_TP is off (GA TP target), require actual TP touch.
            let strength_gate_active = paper_tp_strength_gated && paper_vol_tp;
            let allow_tp = if strength_gate_active {
                tp_has_strength || (paper_tp_use_touch_fallback && tp_hit_pre)
            } else {
                tp_hit_pre
            };
            if tp_allowed_global && allow_tp {
                let exit_price = apply_slippage(trade.tp_target, !is_long, trade.vol_bps);
                exit_pnl = Some(if is_long {
                    (exit_price - trade.entry_price) / trade.entry_price
                } else {
                    (trade.entry_price - exit_price) / trade.entry_price
                });
                exit_tag = "TP";
            }
        }
        if exit_pnl.is_none() {
            apply_mfe_breakeven_lock(
                trade,
                is_long,
                paper_mfe_lock_threshold,
                paper_mfe_lock,
            );
        }

        let was_pullback_at_bar_open =
            trade.exit_state == AdaptiveExitState::PullbackCandidate;

        if trailing_adaptive_exit {
            let drawdown_from_peak = (trade.max_pnl - mark_pnl).max(0.0);
            let profit_arm = trade.max_pnl >= trailing_arm_mfe;
            let time_arm = trade.current_hold >= trailing_arm_bars_loose;
            let trailing_armed = profit_arm || time_arm;
            if trailing_armed {
                trade.trailing_armed_seen = true;
            }
            let strong_path = trade.bars_to_mfe > 0
                && trade.bars_to_mfe <= trailing_good_mfe_bars
                && trade.min_pnl_first_10 >= trailing_good_mae_cut;
            let dd_threshold = if strong_path { trailing_dd_strong } else { trailing_dd_weak };
            let decay_detected = trade.decay_count >= trailing_decay_peak_bars;
            let bars_since_peak = if trade.bars_to_mfe > 0 {
                trade.current_hold.saturating_sub(trade.bars_to_mfe)
            } else {
                usize::MAX
            };
            // Continuation-aware filter: ignore short pullback recoveries right after a peak.
            let continuation_signal =
                mark_pnl > trade.last_mark_pnl && bars_since_peak <= trailing_continuation_peak_bars;
            let recovery_likely = mark_pnl > k83_recovery_frac * trade.max_pnl
                || (bars_since_peak <= k83_early_peak_bars && drawdown_from_peak < k83_early_dd_max);
            if trailing_armed {
                let weak_decay = decay_detected
                    && drawdown_from_peak >= trailing_decay_dd
                    && !continuation_signal;
                let decay_slope = (trade.last_mark_pnl - mark_pnl).max(0.0);
                let strong_decay = drawdown_from_peak >= trailing_decay_strong_dd
                    && trade.decay_count >= trailing_decay_weak_confirm
                    && !continuation_signal;
                let allow_strong_dd_break = strong_path || !recovery_likely;
                let peak_retention = if trade.max_pnl > 1e-12 {
                    (mark_pnl / trade.max_pnl).clamp(-10.0, 10.0)
                } else {
                    0.0
                };
                let weak_peak_hold = peak_retention > k83_peak_hold_frac;
                let weak_strong_dd_allows_break = !weak_peak_hold
                    && (drawdown_from_peak > trailing_decay_strong_dd + k83_strong_dd_extra
                        || trade.decay_count >= k83_weak_path_decay_min);
                let confirm_strong_dd_break = strong_decay
                    && (strong_path && allow_strong_dd_break || !strong_path && weak_strong_dd_allows_break);
                let slope_break = trade.decay_count >= trailing_decay_weak_confirm
                    && decay_slope >= trailing_decay_slope
                    && !continuation_signal;
                let timeout_break = bars_since_peak > trailing_pullback_timeout_bars;
                // Intra-bar state loop (K8.8 min pullback); weak pullback exit = TRAIL_DD only if armed MFE + past early bars (K9.1).
                const MAX_STATE_ITERS: usize = 16;
                const K87_EARLY_PULLBACK_BARS: usize = 3;
                let mut state_iters = 0usize;
                while state_iters < MAX_STATE_ITERS {
                    state_iters += 1;
                    let prev_state = trade.exit_state;
                    match trade.exit_state {
                        AdaptiveExitState::ConfirmedBreak => break,
                        AdaptiveExitState::ArmedObserved => {
                            // `else if` so weak_decay does not get overwritten by strong-DD in the same arm.
                            if weak_decay {
                                trade.bars_since_pullback = 0;
                                trade.exit_state = AdaptiveExitState::PullbackCandidate;
                            } else if drawdown_from_peak >= dd_threshold && confirm_strong_dd_break {
                                trade.exit_state = AdaptiveExitState::ConfirmedBreak;
                                exit_tag = if strong_path {
                                    "TRAIL_BREAK_STRONG_DD_STRONG_PATH"
                                } else {
                                    "TRAIL_BREAK_STRONG_DD_WEAK_PATH"
                                };
                            }
                        }
                        AdaptiveExitState::PullbackCandidate => {
                            if continuation_signal {
                                trade.bars_since_pullback = usize::MAX;
                                trade.exit_state = AdaptiveExitState::ArmedObserved;
                            } else if strong_path {
                                if confirm_strong_dd_break {
                                    trade.exit_state = AdaptiveExitState::ConfirmedBreak;
                                    exit_tag = "TRAIL_BREAK_STRONG_DD_STRONG_PATH";
                                } else if slope_break {
                                    trade.exit_state = AdaptiveExitState::ConfirmedBreak;
                                    exit_tag = "TRAIL_BREAK_SLOPE_STRONG_PATH";
                                } else if timeout_break {
                                    trade.exit_state = AdaptiveExitState::ConfirmedBreak;
                                    exit_tag = "TRAIL_BREAK_TIMEOUT_STRONG_PATH";
                                }
                            } else {
                                let allow_trailing = trade.max_pnl >= trailing_arm_mfe
                                    && trade.bars_since_pullback > K87_EARLY_PULLBACK_BARS;
                                if allow_trailing && drawdown_from_peak >= trailing_dd_weak {
                                    trade.exit_state = AdaptiveExitState::ConfirmedBreak;
                                    exit_tag = "TRAIL_DD";
                                }
                            }
                        }
                    }
                    if trade.exit_state == AdaptiveExitState::ConfirmedBreak {
                        let min_hold_after_pullback = K87_EARLY_PULLBACK_BARS;
                        let in_early_pullback_hold = !strong_path
                            && trade.bars_since_pullback != usize::MAX
                            && trade.bars_since_pullback < min_hold_after_pullback;
                        if in_early_pullback_hold {
                            trade.exit_state = AdaptiveExitState::PullbackCandidate;
                            exit_tag = "NONE";
                            continue;
                        }
                    }
                    if trade.exit_state == prev_state {
                        break;
                    }
                }
                if trade.exit_state == AdaptiveExitState::ConfirmedBreak {
                    exit_pnl = Some(mark_pnl);
                    if exit_tag == "NONE" {
                        exit_tag = if strong_path {
                            "TRAIL_BREAK_UNCLASSIFIED_STRONG_PATH"
                        } else {
                            "TRAIL_BREAK_UNCLASSIFIED_WEAK_PATH"
                        };
                    }
                }
            }
        } else if trailing_exit {
            let drawdown_from_peak = (trade.max_pnl - mark_pnl).max(0.0);
            if trade.current_hold >= trailing_min_bars && drawdown_from_peak >= trailing_dd {
                exit_pnl = Some(mark_pnl);
                exit_tag = "TRAIL_DD";
            }
        } else if signal_exit {
            if trade.rank < signal_decay_rank {
                exit_pnl = Some(mark_pnl);
                exit_tag = "SIGNAL_DECAY";
            }
        } else if hybrid_delayed_exit {
            let effective_tp = if adaptive_tp {
                let edge_ret = (trade.expected_edge_bps.abs() / 10000.0).max(0.0);
                (edge_ret * 1.5).clamp(0.0010, 0.0020)
            } else {
                hybrid_tp
            };
            if tp_allowed_global && mark_pnl >= effective_tp {
                exit_pnl = Some(mark_pnl);
                exit_tag = "TP";
            } else if trade.current_hold >= delayed_min_bars && mark_pnl <= delayed_sl {
                exit_pnl = Some(mark_pnl);
                exit_tag = "DELAYED_SL";
            }
        } else if delayed_exit {
            if trade.current_hold >= delayed_min_bars && mark_pnl <= delayed_sl {
                exit_pnl = Some(mark_pnl);
                exit_tag = "DELAYED_SL";
            }
        } else if hybrid_exit {
            let mark_exit = apply_slippage(close, !is_long, trade.vol_bps);
            let mark_pnl = if is_long {
                (mark_exit - trade.entry_price) / trade.entry_price
            } else {
                (trade.entry_price - mark_exit) / trade.entry_price
            };
            if tp_allowed_global && mark_pnl >= hybrid_tp {
                exit_pnl = Some(mark_pnl);
                exit_tag = "TP";
            } else if mark_pnl <= hybrid_sl {
                exit_pnl = Some(mark_pnl);
                exit_tag = "SL";
            }
        } else if !time_exit_only {
            if exit_pnl.is_none() {
                let raw_exit = resolve_intracandle_exit(
                    high,
                    low,
                    trade.tp_target,
                    trade.sl_target,
                    is_long,
                );
                if paper_tp_ladder && default_tpsl_path && !ladder_tp_enabled {
                    if let Some(exit_type) =
                        intrabar_exit_respecting_entry_bar(trade.current_hold, raw_exit)
                    {
                        let sl_only = matches!(exit_type, ExitType::StopLoss | ExitType::Ambiguous);
                        if sl_only {
                            let exit_price = apply_slippage(trade.sl_target, !is_long, trade.vol_bps);
                            exit_pnl = Some(if is_long {
                                (exit_price - trade.entry_price) / trade.entry_price
                            } else {
                                (trade.entry_price - exit_price) / trade.entry_price
                            });
                            exit_tag = "SL";
                        }
                    }
                } else {
                    let raw_exit = match raw_exit {
                        Some(ExitType::Ambiguous) => Some(ExitType::TakeProfit),
                        x => x,
                    };
                    if let Some(exit_type) =
                        intrabar_exit_respecting_entry_bar(trade.current_hold, raw_exit)
                    {
                        match exit_type {
                            ExitType::TakeProfit => {
                                // Runners: TP fully disabled; trail precedence is applied earlier this bar.
                                if tp_allowed_global {
                                    let exit_price = apply_slippage(
                                        trade.tp_target,
                                        !is_long,
                                        trade.vol_bps,
                                    );
                                    exit_pnl = Some(if is_long {
                                        (exit_price - trade.entry_price) / trade.entry_price
                                    } else {
                                        (trade.entry_price - exit_price) / trade.entry_price
                                    });
                                    exit_tag = "TP";
                                }
                            }
                            ExitType::StopLoss => {
                                let exit_price = apply_slippage(
                                    trade.sl_target,
                                    !is_long,
                                    trade.vol_bps,
                                );
                                exit_pnl = Some(if is_long {
                                    (exit_price - trade.entry_price) / trade.entry_price
                                } else {
                                    (trade.entry_price - exit_price) / trade.entry_price
                                });
                                exit_tag = "SL";
                            }
                            ExitType::Ambiguous => unreachable!(),
                        }
                    }
                }
            }
        }

        if exit_pnl.is_some() && exit_tag == "TP" && trade.current_hold < tp_min_hold_bars {
            // Deterministic TP hold gate: disable full take-profit before TP_MIN_HOLD bars.
            if !trade.partial_tp_done
                && trade.current_hold >= partial_tp_min_hold_bars
                && partial_tp_fraction > 0.0
            {
                let exit_price = apply_slippage(trade.tp_target, !is_long, trade.vol_bps);
                let partial_pnl = if is_long {
                    (exit_price - trade.entry_price) / trade.entry_price
                } else {
                    (trade.entry_price - exit_price) / trade.entry_price
                };
                let clipped = (trade.size * partial_tp_fraction).min(trade.size).max(0.0);
                if clipped > 0.0 {
                    registry.equity *= 1.0 + (partial_pnl * clipped);
                    trade.size = (trade.size - clipped).max(0.0);
                    trade.partial_tp_done = true;
                    if exit_probe {
                        println!(
                            "[PARTIAL_TP] rec_id={} sym={} hold={} frac={:.3} pnl={:.6} remaining_size={:.3}",
                            trade.rec_id,
                            trade.symbol,
                            trade.current_hold,
                            clipped,
                            partial_pnl,
                            trade.size
                        );
                    }
                }
            }
            exit_pnl = None;
            exit_tag = "";
        }
        if exit_pnl.is_some() && trade.current_hold < min_hold_bars {
            // Deterministic minimum hold: suppress all early exits until minimum bars elapse.
            exit_pnl = None;
            exit_tag = "";
        }
        if paper_trail_enable
            && trade.trail_active
            && trade.remaining_fraction > 0.0
            && exit_pnl.is_none()
            && trade.current_hold >= min_hold_bars
        {
            if let Some(pnl) = paper_trail_touch_exit_pnl(
                trade.entry_price,
                trade.trail_stop,
                is_long,
                high,
                low,
                close,
                mark_pnl,
                trade.vol_bps,
            ) {
                exit_pnl = Some(pnl);
                exit_tag = "TRAIL";
            }
        }

        let allow_time_exit = !tpsl_only;
        let hold_limit = if time_exit_only
            || hybrid_exit
            || delayed_exit
            || hybrid_delayed_exit
            || signal_exit
            || trailing_exit
            || trailing_adaptive_exit
        {
            fixed_hold_bars
        } else {
            trade.hold_limit
        };
        // K10: weak-path adaptive, no arm MFE by bar 3 — exit before TIME (locked; K11.x removed as over-pruning).
        const K10_WEAK_MFE_MIN_HOLD: usize = 3;
        if trailing_adaptive_exit
            && allow_time_exit
            && exit_pnl.is_none()
        {
            let strong_path_exit = trade.bars_to_mfe > 0
                && trade.bars_to_mfe <= trailing_good_mfe_bars
                && trade.min_pnl_first_10 >= trailing_good_mae_cut;
            if !strong_path_exit
                && trade.max_pnl < trailing_arm_mfe
                && trade.current_hold >= K10_WEAK_MFE_MIN_HOLD
            {
                exit_pnl = Some(mark_pnl);
                exit_tag = "EARLY_WEAK_EXIT";
            }
        }
        if allow_time_exit && exit_pnl.is_none() && trade.current_hold >= hold_limit {
            let exit_price = apply_slippage(close, !is_long, trade.vol_bps);
            exit_pnl = Some(if is_long {
                (exit_price - trade.entry_price) / trade.entry_price
            } else {
                (trade.entry_price - exit_price) / trade.entry_price
            });
            exit_tag = "TIME";
        }

        if exit_probe {
            let (tp_hit, sl_hit) = if exit_pnl.is_some() && exit_tag == "TP" {
                (1i32, 0i32)
            } else if exit_pnl.is_some() && exit_tag == "SL" {
                (0i32, 1i32)
            } else if exit_pnl.is_some() {
                (0i32, 0i32)
            } else if default_tpsl_path {
                let tp_pre = if is_long {
                    high >= trade.tp_target
                } else {
                    low <= trade.tp_target
                };
                if tp_pre && (!paper_tp_ladder || ladder_tp_enabled) {
                    (1i32, 0i32)
                } else {
                    let raw_intrabar = resolve_intracandle_exit(
                        high,
                        low,
                        trade.tp_target,
                        trade.sl_target,
                        is_long,
                    );
                    let raw_intrabar = if paper_tp_ladder && !ladder_tp_enabled {
                        raw_intrabar
                    } else {
                        match raw_intrabar {
                            Some(ExitType::Ambiguous) => Some(ExitType::TakeProfit),
                            x => x,
                        }
                    };
                    let intrabar = intrabar_exit_respecting_entry_bar(
                        trade.current_hold,
                        raw_intrabar,
                    );
                    if paper_tp_ladder && !ladder_tp_enabled {
                        match intrabar {
                            Some(ExitType::StopLoss) | Some(ExitType::Ambiguous) => (0i32, 1i32),
                            _ => (0i32, 0i32),
                        }
                    } else {
                        match intrabar {
                            Some(ExitType::TakeProfit) => (1i32, 0i32),
                            Some(ExitType::StopLoss) => (0i32, 1i32),
                            Some(ExitType::Ambiguous) => (1i32, 0i32),
                            None => (0i32, 0i32),
                        }
                    }
                }
            } else {
                let raw_intrabar = resolve_intracandle_exit(
                    high,
                    low,
                    trade.tp_target,
                    trade.sl_target,
                    is_long,
                );
                let intrabar =
                    intrabar_exit_respecting_entry_bar(trade.current_hold, raw_intrabar);
                match intrabar {
                    Some(ExitType::TakeProfit) => (1i32, 0i32),
                    Some(ExitType::StopLoss) => (0i32, 1i32),
                    Some(ExitType::Ambiguous) => (1i32, 1i32),
                    None => (0i32, 0i32),
                }
            };
            let timeout_hit = (exit_tag == "TIME") as i32;
            let trail_lvl = if trailing_adaptive_exit {
                (trade.max_pnl - mark_pnl).max(0.0)
            } else {
                0.0
            };
            let profit_arm = trade.max_pnl >= trailing_arm_mfe;
            let time_arm = trade.current_hold >= trailing_arm_bars_loose;
            let armed_now = (profit_arm || time_arm) as i32;
            let sl_level_ret = if is_long {
                (trade.sl_target - trade.entry_price) / trade.entry_price.max(1e-12)
            } else {
                (trade.entry_price - trade.sl_target) / trade.entry_price.max(1e-12)
            };
            println!(
                "[EXIT_TRACE] rec_id={} sym={} state={:?} armed_seen={} armed_now={} ret_now={:.6} mfe={:.6} mae={:.6} \
                 sl_hit={} tp_hit={} tp_strength_hit={} tp_strength_ret={:.6} tp_touch={} stage1_hit={} stage2_hit={} stage1_applied={} stage2_applied={} sl_level_ret={:.6} timeout={} trail_dd={:.6} exit_tag={} closing={}",
                trade.rec_id,
                trade.symbol,
                trade.exit_state,
                if trade.trailing_armed_seen { 1 } else { 0 },
                armed_now,
                mark_pnl,
                trade.max_pnl,
                trade.min_pnl,
                sl_hit,
                tp_hit,
                tp_strength_hit,
                tp_strength_ret,
                tp_touch,
                stage1_hit as i32,
                stage2_hit as i32,
                stage1_applied,
                stage2_applied,
                sl_level_ret,
                timeout_hit,
                trail_lvl,
                exit_tag,
                exit_pnl.is_some() as i32
            );
        }

        if let Some(pnl_raw) = exit_pnl {
            let pnl = trade.realized_partial_pnl + (trade.remaining_fraction * pnl_raw);
            registry.equity *= 1.0 + (pnl * trade.size);
            if registry.equity > registry.peak_equity {
                registry.peak_equity = registry.equity;
                registry.rolling_peak = registry.equity;
            }
            let dd = (registry.peak_equity - registry.equity) / registry.peak_equity;
            if dd > registry.max_drawdown {
                registry.max_drawdown = dd;
            }
            registry.closed_count += 1;
            if pnl > 0.0 {
                registry.wins += 1;
            } else {
                registry.losses += 1;
            }
            registry.pnl_history.push(pnl);
            *registry.strategy_pnl.entry(trade.strategy_id).or_insert(0.0) += pnl;
            *registry.strategy_counts.entry(trade.strategy_id).or_insert(0) += 1;
            let r_idx = (trade.rank * 10.0).floor().clamp(0.0, 9.0) as usize;
            registry.rank_pnl_sum[r_idx] += pnl;
            registry.rank_count[r_idx] += 1;
            println!(
                "[EXIT] type={} sym={} strategy={} pnl={:.6} dur={}",
                exit_tag, trade.symbol, trade.strategy_id, pnl, trade.current_hold
            );
            let ret_at_exit = if trade.max_pnl > 1e-12 {
                (pnl / trade.max_pnl).clamp(-10.0, 10.0)
            } else {
                0.0
            };
            println!(
                "[TRADE_PATH] rec_id={} sym={} mfe={:.6} mae={:.6} pnl={:.6} ret_at_exit={:.6} edge_bps={:.3} rank={:.4} rec_score={:.6} rec_feas={:.4} rec_conf={:.4} rec_voters={} intent_age={} momentum_3={:.6} vol_5={:.6} score_std_5={:.6} vol_bps={:.2} partial_realized_pnl={:.6} remainder_exit_type={} trail_active={} trail_stop_at_exit={:.6} peak_price_at_exit={:.6} is_runner={} dur={} armed={} state={:?} exit_type={}",
                trade.rec_id,
                trade.symbol,
                trade.max_pnl,
                trade.min_pnl,
                pnl,
                ret_at_exit,
                trade.expected_edge_bps,
                trade.rank,
                trade.rec_score,
                trade.rec_feas,
                trade.rec_conf,
                trade.rec_voters,
                trade.intent_age_at_fill,
                trade.momentum_3,
                trade.vol_5,
                trade.score_std_5,
                trade.vol_bps,
                trade.realized_partial_pnl,
                exit_tag,
                if trade.trail_active { 1 } else { 0 },
                trade.trail_stop,
                trade.peak_price,
                if trade.is_runner { 1 } else { 0 },
                trade.current_hold,
                if trade.trailing_armed_seen { 1 } else { 0 },
                trade.exit_state,
                exit_tag
            );
            registry.closed_observations.push(ClosedTradeObservation {
                rank: trade.rank,
                vol_bucket: paper_vol_bucket_from_bps(trade.vol_bps),
                mfe: trade.max_pnl.max(0.0),
                mae_abs: (-trade.min_pnl).max(0.0),
                hold_bars: trade.current_hold,
            });
            registry.active_trades.remove(i);
        } else {
            trade.last_mark_pnl = mark_pnl;
            if trade.exit_state == AdaptiveExitState::PullbackCandidate
                && was_pullback_at_bar_open
                && trade.bars_since_pullback != usize::MAX
            {
                trade.bars_since_pullback = trade.bars_since_pullback.saturating_add(1);
            }
            i += 1;
        }
    }

    registry.equity_curve.push(registry.equity);
    registry.timestamps.push(ts);
}

pub fn finalize_paper_registry(
    registry: &mut PaperRegistry,
    latest_prices: &HashMap<String, f64>,
) {
    // Pending intents cannot execute once stream ends.
    registry.pending_intents.clear();

    let i = 0;
    while i < registry.active_trades.len() {
        let trade = &registry.active_trades[i];
        let mark_close = latest_prices
            .get(&trade.symbol)
            .copied()
            .unwrap_or(trade.entry_price);
        let is_long = trade.signal == SignalType::BUY;
        let exit_price = apply_slippage(mark_close, !is_long, trade.vol_bps);
        let pnl_raw = if is_long {
            (exit_price - trade.entry_price) / trade.entry_price.max(1e-9)
        } else {
            (trade.entry_price - exit_price) / trade.entry_price.max(1e-9)
        };
        let pnl = trade.realized_partial_pnl + (trade.remaining_fraction * pnl_raw);

        registry.equity *= 1.0 + (pnl * trade.size);
        if registry.equity > registry.peak_equity {
            registry.peak_equity = registry.equity;
            registry.rolling_peak = registry.equity;
        }
        let dd = (registry.peak_equity - registry.equity) / registry.peak_equity.max(1e-9);
        if dd > registry.max_drawdown {
            registry.max_drawdown = dd;
        }

        registry.closed_count += 1;
        if pnl > 0.0 {
            registry.wins += 1;
        } else {
            registry.losses += 1;
        }
        registry.pnl_history.push(pnl);
        *registry.strategy_pnl.entry(trade.strategy_id).or_insert(0.0) += pnl;
        *registry.strategy_counts.entry(trade.strategy_id).or_insert(0) += 1;
        let r_idx = (trade.rank * 10.0).floor().clamp(0.0, 9.0) as usize;
        registry.rank_pnl_sum[r_idx] += pnl;
        registry.rank_count[r_idx] += 1;

        println!(
            "[EXIT] type=FINALIZE_TIME sym={} strategy={} pnl={:.6} dur={}",
            trade.symbol, trade.strategy_id, pnl, trade.current_hold
        );
        let ret_at_exit = if trade.max_pnl > 1e-12 {
            (pnl / trade.max_pnl).clamp(-10.0, 10.0)
        } else {
            0.0
        };
        println!(
            "[TRADE_PATH] rec_id={} sym={} mfe={:.6} mae={:.6} pnl={:.6} ret_at_exit={:.6} edge_bps={:.3} rank={:.4} rec_score={:.6} rec_feas={:.4} rec_conf={:.4} rec_voters={} intent_age={} momentum_3={:.6} vol_5={:.6} score_std_5={:.6} vol_bps={:.2} partial_realized_pnl={:.6} remainder_exit_type=FINALIZE_TIME trail_active={} trail_stop_at_exit={:.6} peak_price_at_exit={:.6} is_runner={} dur={} exit_type=FINALIZE_TIME",
            trade.rec_id,
            trade.symbol,
            trade.max_pnl,
            trade.min_pnl,
            pnl,
            ret_at_exit,
            trade.expected_edge_bps,
            trade.rank,
            trade.rec_score,
            trade.rec_feas,
            trade.rec_conf,
            trade.rec_voters,
            trade.intent_age_at_fill,
            trade.momentum_3,
            trade.vol_5,
            trade.score_std_5,
            trade.vol_bps,
            trade.realized_partial_pnl,
            if trade.trail_active { 1 } else { 0 },
            trade.trail_stop,
            trade.peak_price,
            if trade.is_runner { 1 } else { 0 },
            trade.current_hold
        );
        registry.closed_observations.push(ClosedTradeObservation {
            rank: trade.rank,
            vol_bucket: paper_vol_bucket_from_bps(trade.vol_bps),
            mfe: trade.max_pnl.max(0.0),
            mae_abs: (-trade.min_pnl).max(0.0),
            hold_bars: trade.current_hold,
        });

        registry.active_trades.remove(i);
    }
}

pub fn close_active_trades_for_symbol(
    registry: &mut PaperRegistry,
    symbol: &str,
    latest_candle: &Candle,
    reason: &str,
) -> usize {
    let close = latest_candle.close as f64;
    let mut closed = 0usize;
    let mut i = 0;
    while i < registry.active_trades.len() {
        if registry.active_trades[i].symbol != symbol {
            i += 1;
            continue;
        }
        let trade = &registry.active_trades[i];
        let is_long = trade.signal == SignalType::BUY;
        let exit_price = apply_slippage(close, !is_long, trade.vol_bps);
        let pnl_raw = if is_long {
            (exit_price - trade.entry_price) / trade.entry_price.max(1e-9)
        } else {
            (trade.entry_price - exit_price) / trade.entry_price.max(1e-9)
        };
        let pnl = trade.realized_partial_pnl + (trade.remaining_fraction * pnl_raw);

        registry.equity *= 1.0 + (pnl * trade.size);
        if registry.equity > registry.peak_equity {
            registry.peak_equity = registry.equity;
            registry.rolling_peak = registry.equity;
        }
        let dd = (registry.peak_equity - registry.equity) / registry.peak_equity.max(1e-9);
        if dd > registry.max_drawdown {
            registry.max_drawdown = dd;
        }

        registry.closed_count += 1;
        if pnl > 0.0 {
            registry.wins += 1;
        } else {
            registry.losses += 1;
        }
        registry.pnl_history.push(pnl);
        *registry.strategy_pnl.entry(trade.strategy_id).or_insert(0.0) += pnl;
        *registry.strategy_counts.entry(trade.strategy_id).or_insert(0) += 1;
        let r_idx = (trade.rank * 10.0).floor().clamp(0.0, 9.0) as usize;
        registry.rank_pnl_sum[r_idx] += pnl;
        registry.rank_count[r_idx] += 1;

        println!(
            "[EXIT] type={} sym={} strategy={} pnl={:.6} dur={}",
            reason, trade.symbol, trade.strategy_id, pnl, trade.current_hold
        );
        let ret_at_exit = if trade.max_pnl > 1e-12 {
            (pnl / trade.max_pnl).clamp(-10.0, 10.0)
        } else {
            0.0
        };
        println!(
            "[TRADE_PATH] rec_id={} sym={} mfe={:.6} mae={:.6} pnl={:.6} ret_at_exit={:.6} edge_bps={:.3} rank={:.4} rec_score={:.6} rec_feas={:.4} rec_conf={:.4} rec_voters={} intent_age={} momentum_3={:.6} vol_5={:.6} score_std_5={:.6} vol_bps={:.2} partial_realized_pnl={:.6} remainder_exit_type={} trail_active={} trail_stop_at_exit={:.6} peak_price_at_exit={:.6} is_runner={} dur={} armed={} state={:?} exit_type={}",
            trade.rec_id,
            trade.symbol,
            trade.max_pnl,
            trade.min_pnl,
            pnl,
            ret_at_exit,
            trade.expected_edge_bps,
            trade.rank,
            trade.rec_score,
            trade.rec_feas,
            trade.rec_conf,
            trade.rec_voters,
            trade.intent_age_at_fill,
            trade.momentum_3,
            trade.vol_5,
            trade.score_std_5,
            trade.vol_bps,
            trade.realized_partial_pnl,
            reason,
            if trade.trail_active { 1 } else { 0 },
            trade.trail_stop,
            trade.peak_price,
            if trade.is_runner { 1 } else { 0 },
            trade.current_hold,
            if trade.trailing_armed_seen { 1 } else { 0 },
            trade.exit_state,
            reason
        );
        registry.closed_observations.push(ClosedTradeObservation {
            rank: trade.rank,
            vol_bucket: paper_vol_bucket_from_bps(trade.vol_bps),
            mfe: trade.max_pnl.max(0.0),
            mae_abs: (-trade.min_pnl).max(0.0),
            hold_bars: trade.current_hold,
        });
        registry.active_trades.remove(i);
        closed += 1;
    }
    closed
}
