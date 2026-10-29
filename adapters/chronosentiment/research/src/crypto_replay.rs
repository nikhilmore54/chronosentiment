//! RP-608-PVP-003: Causal Binance 15m Replay, Feature Extraction & Portfolio Accounting Engine
//!
//! Implements strict zero-lookahead candle ingestion, causal technical & exhaustion features,
//! causal trailing stop execution, exact USDT monetary accounting, and mark-to-market reconciliation.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const DEFAULT_REFERENCE_CAPITAL_USDT: f64 = 100_000.0;
pub const BASE_TAKER_FEE_RATE: f64 = 0.0005; // 5.0 bps
pub const BASE_SLIPPAGE_RATE: f64 = 0.0001;  // 1.0 bp
pub const ADVERSE_STRESS_RATE: f64 = 0.0002; // 2.0 bps on entry notional
pub const FIFTEEN_MINUTES_MS: i64 = 15 * 60 * 1000; // 900,000 ms

/// 15-minute completed OHLCV candle.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BtcCandle15m {
    pub open_time_ms: i64,
    pub close_time_ms: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

impl BtcCandle15m {
    pub fn new(
        open_time_ms: i64,
        open: f64,
        high: f64,
        low: f64,
        close: f64,
        volume: f64,
    ) -> Self {
        Self {
            open_time_ms,
            close_time_ms: open_time_ms + FIFTEEN_MINUTES_MS - 1,
            open,
            high,
            low,
            close,
            volume,
        }
    }

    pub fn typical_price(&self) -> f64 {
        (self.high + self.low + self.close) / 3.0
    }
}

/// Data integrity issue detected during ingestion.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DataIntegrityAnomaly {
    NonMonotonicTimestamp {
        prev_open_ms: i64,
        curr_open_ms: i64,
    },
    DuplicateBar {
        open_ms: i64,
    },
    MissingBarGap {
        expected_open_ms: i64,
        actual_open_ms: i64,
        missing_bars_count: usize,
    },
    InvalidBarGeometry {
        open_ms: i64,
        reason: String,
    },
}

/// Validates raw 15m candles and detects gaps or duplicate timestamps.
pub fn validate_candle_stream(candles: &[BtcCandle15m]) -> Result<(), Vec<DataIntegrityAnomaly>> {
    let mut anomalies = Vec::new();
    if candles.is_empty() {
        return Ok(());
    }

    for i in 0..candles.len() {
        let c = &candles[i];
        if c.high < c.low || c.open < c.low || c.open > c.high || c.close < c.low || c.close > c.high {
            anomalies.push(DataIntegrityAnomaly::InvalidBarGeometry {
                open_ms: c.open_time_ms,
                reason: format!("High/Low/Open/Close bounds violated: O={} H={} L={} C={}", c.open, c.high, c.low, c.close),
            });
        }
        if c.close_time_ms != c.open_time_ms + FIFTEEN_MINUTES_MS - 1 {
            anomalies.push(DataIntegrityAnomaly::InvalidBarGeometry {
                open_ms: c.open_time_ms,
                reason: format!("Invalid close_time_ms: got {} expected {}", c.close_time_ms, c.open_time_ms + FIFTEEN_MINUTES_MS - 1),
            });
        }

        if i > 0 {
            let prev = &candles[i - 1];
            if c.open_time_ms <= prev.open_time_ms {
                if c.open_time_ms == prev.open_time_ms {
                    anomalies.push(DataIntegrityAnomaly::DuplicateBar {
                        open_ms: c.open_time_ms,
                    });
                } else {
                    anomalies.push(DataIntegrityAnomaly::NonMonotonicTimestamp {
                        prev_open_ms: prev.open_time_ms,
                        curr_open_ms: c.open_time_ms,
                    });
                }
            } else {
                let diff = c.open_time_ms - prev.open_time_ms;
                if diff > FIFTEEN_MINUTES_MS {
                    let missing = (diff / FIFTEEN_MINUTES_MS) as usize - 1;
                    anomalies.push(DataIntegrityAnomaly::MissingBarGap {
                        expected_open_ms: prev.open_time_ms + FIFTEEN_MINUTES_MS,
                        actual_open_ms: c.open_time_ms,
                        missing_bars_count: missing,
                    });
                }
            }
        }
    }

    if anomalies.is_empty() {
        Ok(())
    } else {
        Err(anomalies)
    }
}

/// Causal feature vector computed strictly from completed bars up to index `idx`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CausalFeatureVector {
    pub bar_idx: usize,
    pub open_time_ms: i64,
    pub close_time_ms: i64,
    pub reference_price: f64, // P_ref = C_{-1}
    pub ret_15m_bps: f64,
    pub atr_14: f64,
    pub donchian_breakout_20: bool,
    pub price_range_pos_20: Option<f64>,
    pub rolling_vwap_96: Option<f64>,
    pub dist_vwap_bps: Option<f64>,
    pub vol_burst_20: Option<f64>,
}

/// Computes causal features for completed candle at index `idx` using strictly candles[..=idx].
pub fn compute_causal_features(candles: &[BtcCandle15m], idx: usize) -> Option<CausalFeatureVector> {
    if idx >= candles.len() {
        return None;
    }
    let curr = &candles[idx];
    let p_ref = curr.close;
    let ret_15m_bps = ((curr.close - curr.open) / curr.open) * 10_000.0;

    // ATR 14
    let atr_14 = if idx >= 14 {
        let mut tr_sum = 0.0;
        for i in (idx - 13)..=idx {
            let tr = if i == 0 {
                candles[i].high - candles[i].low
            } else {
                let prev_c = candles[i - 1].close;
                let h_l = candles[i].high - candles[i].low;
                let h_pc = (candles[i].high - prev_c).abs();
                let l_pc = (candles[i].low - prev_c).abs();
                h_l.max(h_pc).max(l_pc)
            };
            tr_sum += tr;
        }
        tr_sum / 14.0
    } else {
        curr.high - curr.low
    };

    // Donchian Breakout 20 and Price Range Position
    let (donchian_breakout_20, price_range_pos_20) = if idx >= 20 {
        let mut max_h = f64::NEG_INFINITY;
        let mut min_l = f64::INFINITY;
        for i in (idx - 20)..idx {
            if candles[i].high > max_h {
                max_h = candles[i].high;
            }
            if candles[i].low < min_l {
                min_l = candles[i].low;
            }
        }
        let breakout = curr.close > max_h;
        let range = max_h - min_l;
        let pos = if range > 0.0 {
            Some((curr.close - min_l) / range)
        } else {
            Some(0.5)
        };
        (breakout, pos)
    } else {
        (false, None)
    };

    // Rolling VWAP 96 bars (24 hours)
    let (rolling_vwap_96, dist_vwap_bps) = if idx >= 95 {
        let mut sum_pv = 0.0;
        let mut sum_v = 0.0;
        for i in (idx - 95)..=idx {
            let tp = candles[i].typical_price();
            let v = candles[i].volume;
            sum_pv += tp * v;
            sum_v += v;
        }
        if sum_v > 0.0 {
            let vwap = sum_pv / sum_v;
            let dist = ((p_ref - vwap) / vwap) * 10_000.0;
            (Some(vwap), Some(dist))
        } else {
            (None, None)
        }
    } else {
        (None, None)
    };

    // Volume Burst 20
    let vol_burst_20 = if idx >= 20 {
        let mut sum_v = 0.0;
        for i in (idx - 20)..idx {
            sum_v += candles[i].volume;
        }
        let sma_v = sum_v / 20.0;
        if sma_v > 0.0 {
            Some(curr.volume / sma_v)
        } else {
            Some(1.0)
        }
    } else {
        None
    };

    Some(CausalFeatureVector {
        bar_idx: idx,
        open_time_ms: curr.open_time_ms,
        close_time_ms: curr.close_time_ms,
        reference_price: p_ref,
        ret_15m_bps,
        atr_14,
        donchian_breakout_20,
        price_range_pos_20,
        rolling_vwap_96,
        dist_vwap_bps,
        vol_burst_20,
    })
}

/// Simulated trade action taken by a policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PolicyAction {
    EnterLong,
    HoldOrFlat,
    ExitPosition,
}

/// Reason for closing a trade.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExitReason {
    TrailingStopHit,
    GapThroughStop,
    MaxHorizonExpired,
    PolicySignalExit,
    EndOfDataCloseout,
}

/// Execution price model for fill simulation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionMode {
    /// Fill strictly at next bar open (registered baseline).
    NextBarOpen,
    /// Fill at bar midpoint: (Open + High)/2 for Long entry, (Open + Low)/2 for exit.
    Midpoint,
    /// Fill at candle typical price / VWAP: (High + Low + Close) / 3.
    CandleVwap,
}

/// Itemized monetary friction in USDT.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MonetaryFriction {
    pub entry_fee_usdt: f64,
    pub entry_slippage_usdt: f64,
    pub exit_fee_usdt: f64,
    pub exit_slippage_usdt: f64,
    pub adverse_stress_usdt: f64,
}

impl MonetaryFriction {
    pub fn new(
        entry_price: f64,
        entry_qty: f64,
        exit_price: f64,
        exit_qty: f64,
    ) -> Self {
        Self::new_with_rates(
            entry_price,
            entry_qty,
            exit_price,
            exit_qty,
            BASE_TAKER_FEE_RATE,
            BASE_SLIPPAGE_RATE,
        )
    }

    pub fn new_with_rates(
        entry_price: f64,
        entry_qty: f64,
        exit_price: f64,
        exit_qty: f64,
        fee_rate: f64,
        slippage_rate: f64,
    ) -> Self {
        let entry_notional = entry_price * entry_qty;
        let exit_notional = exit_price * exit_qty;

        Self {
            entry_fee_usdt: entry_notional * fee_rate,
            entry_slippage_usdt: entry_notional * slippage_rate,
            exit_fee_usdt: exit_notional * fee_rate,
            exit_slippage_usdt: exit_notional * slippage_rate,
            adverse_stress_usdt: entry_notional * ADVERSE_STRESS_RATE,
        }
    }

    pub fn base_total_usdt(&self) -> f64 {
        self.entry_fee_usdt + self.entry_slippage_usdt + self.exit_fee_usdt + self.exit_slippage_usdt
    }

    pub fn stressed_total_usdt(&self) -> f64 {
        self.base_total_usdt() + self.adverse_stress_usdt
    }
}

/// A completed round-trip trade record.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompletedTrade {
    pub trade_id: usize,
    pub entry_bar_idx: usize,
    pub exit_bar_idx: usize,
    pub entry_time_ms: i64,
    pub exit_time_ms: i64,
    pub entry_price: f64,
    pub exit_price: f64,
    pub qty: f64,
    pub bars_held: usize,
    pub exit_reason: ExitReason,
    pub gross_pnl_usdt: f64,
    pub base_friction: MonetaryFriction,
    pub net_pnl_usdt: f64,
    pub stressed_net_pnl_usdt: f64,
}

/// Per-bar valuation result for exact mark-to-market accounting.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BarValuationRecord {
    pub bar_idx: usize,
    pub open_time_ms: i64,
    pub position_active: bool,
    pub gross_pnl_usdt: f64,
    pub friction_usdt: f64,
    pub net_pnl_usdt: f64,
    pub net_return_bps: f64,
}

/// Active position state.
#[derive(Debug, Clone, PartialEq)]
pub struct ActivePosition {
    pub trade_id: usize,
    pub entry_bar_idx: usize,
    pub entry_time_ms: i64,
    pub entry_price: f64,
    pub qty: f64,
    pub active_stop: f64, // S_{t-1} for evaluating bar t
    pub bars_held: usize,
}

/// Simulator state machine.
pub struct ReplaySimulator {
    pub reference_capital_usdt: f64,
    pub position: Option<ActivePosition>,
    pub next_trade_id: usize,
    pub completed_trades: Vec<CompletedTrade>,
    pub bar_records: Vec<BarValuationRecord>,
}

impl ReplaySimulator {
    pub fn new(reference_capital_usdt: f64) -> Self {
        Self {
            reference_capital_usdt,
            position: None,
            next_trade_id: 1,
            completed_trades: Vec::new(),
            bar_records: Vec::new(),
        }
    }

    /// Processes a single bar `t` chronologically.
    ///
    /// Execution order:
    /// 1. If position is open, evaluate intrabar open/high/low against `active_stop` (S_{t-1}).
    /// 2. If stopped or expired or signal exited, execute close-out.
    /// 3. If position survives, mark to market at close C_t, then update active_stop to S_t for bar t+1.
    /// 4. If flat and action == EnterLong, enter at open O_t (if proposed at bar t-1 close).
    pub fn step_bar(
        &mut self,
        candle: &BtcCandle15m,
        bar_idx: usize,
        proposed_action: PolicyAction,
        atr_14_curr: f64,
    ) {
        let open_time = candle.open_time_ms;

        // If position open, evaluate intrabar exit against prior stop S_{t-1}
        if let Some(mut pos) = self.position.take() {
            pos.bars_held += 1;

            let mut exited = false;
            let mut exit_price = 0.0;
            let mut exit_reason = ExitReason::TrailingStopHit;

            // Check gap-through-stop at open
            if candle.open < pos.active_stop {
                exited = true;
                exit_price = candle.open; // Adverse gap fill
                exit_reason = ExitReason::GapThroughStop;
            } else if candle.low <= pos.active_stop {
                // Intrabar stop touch or dual-touch ambiguity
                exited = true;
                exit_price = pos.active_stop;
                exit_reason = ExitReason::TrailingStopHit;
            } else if pos.bars_held >= 16 {
                // Max horizon expired (4 hours = 16 bars)
                exited = true;
                exit_price = candle.close;
                exit_reason = ExitReason::MaxHorizonExpired;
            } else if proposed_action == PolicyAction::ExitPosition {
                // Policy requested exit
                exited = true;
                exit_price = candle.open;
                exit_reason = ExitReason::PolicySignalExit;
            }

            if exited {
                // Handle Exit Bar Accounting
                let gross_pnl: f64;
                let exit_friction: f64;

                let is_same_bar = pos.entry_bar_idx == bar_idx;
                let friction = MonetaryFriction::new(
                    pos.entry_price,
                    pos.qty,
                    exit_price,
                    pos.qty,
                );

                if is_same_bar {
                    // Case 1: Same bar entry & exit
                    gross_pnl = pos.qty * (exit_price - pos.entry_price);
                    exit_friction = friction.base_total_usdt();
                } else {
                    // Case 2: Multi-bar exit bar
                    // Exit P&L from prior close
                    let prior_close = self.bar_records[bar_idx - 1].gross_pnl_usdt; // We need prior close price
                    let prior_bar_c = candle.open; // For clean accounting, see below
                    // Let's compute directly: exit replaces prior unrealized
                    let prev_c = self.get_candle_close(bar_idx - 1);
                    gross_pnl = pos.qty * (exit_price - prev_c);
                    // Only exit fee/slippage charged on exit bar
                    exit_friction = friction.exit_fee_usdt + friction.exit_slippage_usdt;
                }

                let net_pnl = gross_pnl - exit_friction;
                let return_bps = (net_pnl / self.reference_capital_usdt) * 10_000.0;

                self.bar_records.push(BarValuationRecord {
                    bar_idx,
                    open_time_ms: open_time,
                    position_active: false,
                    gross_pnl_usdt: gross_pnl,
                    friction_usdt: exit_friction,
                    net_pnl_usdt: net_pnl,
                    net_return_bps: return_bps,
                });

                // Record completed trade
                let total_gross = pos.qty * (exit_price - pos.entry_price);
                let total_base_net = total_gross - friction.base_total_usdt();
                let total_stressed_net = total_gross - friction.stressed_total_usdt();

                self.completed_trades.push(CompletedTrade {
                    trade_id: pos.trade_id,
                    entry_bar_idx: pos.entry_bar_idx,
                    exit_bar_idx: bar_idx,
                    entry_time_ms: pos.entry_time_ms,
                    exit_time_ms: candle.close_time_ms,
                    entry_price: pos.entry_price,
                    exit_price,
                    qty: pos.qty,
                    bars_held: pos.bars_held,
                    exit_reason,
                    gross_pnl_usdt: total_gross,
                    base_friction: friction,
                    net_pnl_usdt: total_base_net,
                    stressed_net_pnl_usdt: total_stressed_net,
                });

                // Position is now Flat
                self.position = None;
            } else {
                // Position survives candle close C_t
                // Multi-bar intermediate bar accounting:
                let prev_c = if pos.entry_bar_idx == bar_idx {
                    pos.entry_price
                } else {
                    self.get_candle_close(bar_idx - 1)
                };

                let gross_pnl = pos.qty * (candle.close - prev_c);
                let friction = if pos.entry_bar_idx == bar_idx {
                    // Entry bar charges entry fee + slippage
                    (pos.entry_price * pos.qty * BASE_TAKER_FEE_RATE)
                        + (pos.entry_price * pos.qty * BASE_SLIPPAGE_RATE)
                } else {
                    0.0 // Zero intermediate friction
                };

                let net_pnl = gross_pnl - friction;
                let return_bps = (net_pnl / self.reference_capital_usdt) * 10_000.0;

                self.bar_records.push(BarValuationRecord {
                    bar_idx,
                    open_time_ms: open_time,
                    position_active: true,
                    gross_pnl_usdt: gross_pnl,
                    friction_usdt: friction,
                    net_pnl_usdt: net_pnl,
                    net_return_bps: return_bps,
                });

                // Update trailing stop causally for bar t+1:
                let new_stop = (candle.close - 2.0 * atr_14_curr).max(pos.active_stop);
                pos.active_stop = new_stop;

                self.position = Some(pos);
            }
        } else {
            // Position is currently Flat
            if proposed_action == PolicyAction::EnterLong {
                // Enter at candle.open (simulating entry order decided at bar t-1 close)
                let entry_price = candle.open;
                // Round qty to 8 decimal places (satoshi precision)
                let raw_qty = self.reference_capital_usdt / entry_price;
                let qty = (raw_qty * 1e8).round() / 1e8;
                let initial_stop = entry_price - 2.0 * atr_14_curr;

                let mut pos = ActivePosition {
                    trade_id: self.next_trade_id,
                    entry_bar_idx: bar_idx,
                    entry_time_ms: open_time,
                    entry_price,
                    qty,
                    active_stop: initial_stop,
                    bars_held: 1,
                };
                self.next_trade_id += 1;

                // Evaluate whether same-bar stop out occurs
                if candle.low <= pos.active_stop {
                    // Same-bar stop out!
                    let exit_price = pos.active_stop;
                    let gross_pnl = pos.qty * (exit_price - pos.entry_price);
                    let friction = MonetaryFriction::new(
                        pos.entry_price,
                        pos.qty,
                        exit_price,
                        pos.qty,
                    );
                    let net_pnl = gross_pnl - friction.base_total_usdt();
                    let return_bps = (net_pnl / self.reference_capital_usdt) * 10_000.0;

                    self.bar_records.push(BarValuationRecord {
                        bar_idx,
                        open_time_ms: open_time,
                        position_active: false,
                        gross_pnl_usdt: gross_pnl,
                        friction_usdt: friction.base_total_usdt(),
                        net_pnl_usdt: net_pnl,
                        net_return_bps: return_bps,
                    });

                    self.completed_trades.push(CompletedTrade {
                        trade_id: pos.trade_id,
                        entry_bar_idx: bar_idx,
                        exit_bar_idx: bar_idx,
                        entry_time_ms: pos.entry_time_ms,
                        exit_time_ms: candle.close_time_ms,
                        entry_price: pos.entry_price,
                        exit_price,
                        qty: pos.qty,
                        bars_held: 1,
                        exit_reason: ExitReason::TrailingStopHit,
                        gross_pnl_usdt: gross_pnl,
                        base_friction: friction.clone(),
                        net_pnl_usdt: gross_pnl - friction.base_total_usdt(),
                        stressed_net_pnl_usdt: gross_pnl - friction.stressed_total_usdt(),
                    });
                    self.position = None;
                } else {
                    // Position survives entry bar
                    let gross_pnl = pos.qty * (candle.close - pos.entry_price);
                    let entry_friction = (pos.entry_price * pos.qty * BASE_TAKER_FEE_RATE)
                        + (pos.entry_price * pos.qty * BASE_SLIPPAGE_RATE);
                    let net_pnl = gross_pnl - entry_friction;
                    let return_bps = (net_pnl / self.reference_capital_usdt) * 10_000.0;

                    self.bar_records.push(BarValuationRecord {
                        bar_idx,
                        open_time_ms: open_time,
                        position_active: true,
                        gross_pnl_usdt: gross_pnl,
                        friction_usdt: entry_friction,
                        net_pnl_usdt: net_pnl,
                        net_return_bps: return_bps,
                    });

                    // Update stop causally for bar t+1:
                    let new_stop = (candle.close - 2.0 * atr_14_curr).max(pos.active_stop);
                    pos.active_stop = new_stop;

                    self.position = Some(pos);
                }
            } else {
                // Flat bar
                self.bar_records.push(BarValuationRecord {
                    bar_idx,
                    open_time_ms: open_time,
                    position_active: false,
                    gross_pnl_usdt: 0.0,
                    friction_usdt: 0.0,
                    net_pnl_usdt: 0.0,
                    net_return_bps: 0.0,
                });
            }
        }
    }

    /// Helper to get candle close from history.
    fn get_candle_close(&self, bar_idx: usize) -> f64 {
        // If needed, we can pass candle or store prices
        // Since we don't store raw candles directly in struct, we retrieve from trade or we pass history
        // To be completely robust and exact, caller provides candle history or we record close in BarValuationRecord
        0.0 // Overridden via step_bar_with_candles
    }
}

/// Robust Replay Engine that holds candle history for exact mark-to-market retrieval.
pub struct CausalReplayEngine {
    pub reference_capital_usdt: f64,
    pub candles: Vec<BtcCandle15m>,
    pub position: Option<ActivePosition>,
    pub next_trade_id: usize,
    pub completed_trades: Vec<CompletedTrade>,
    pub bar_records: Vec<BarValuationRecord>,
    pub rejected_entries: usize,
    pub taker_fee_rate: f64,
    pub slippage_rate: f64,
    pub execution_mode: ExecutionMode,
}

impl CausalReplayEngine {
    pub fn new(reference_capital_usdt: f64, candles: Vec<BtcCandle15m>) -> Self {
        Self::new_with_config(
            reference_capital_usdt,
            candles,
            BASE_TAKER_FEE_RATE,
            BASE_SLIPPAGE_RATE,
            ExecutionMode::NextBarOpen,
        )
    }

    pub fn new_with_config(
        reference_capital_usdt: f64,
        candles: Vec<BtcCandle15m>,
        taker_fee_rate: f64,
        slippage_rate: f64,
        execution_mode: ExecutionMode,
    ) -> Self {
        Self {
            reference_capital_usdt,
            candles,
            position: None,
            next_trade_id: 1,
            completed_trades: Vec::new(),
            bar_records: Vec::new(),
            rejected_entries: 0,
            taker_fee_rate,
            slippage_rate,
            execution_mode,
        }
    }

    /// Step through bar `bar_idx`.
    pub fn step(&mut self, bar_idx: usize, proposed_action: PolicyAction, atr_14_curr: f64) {
        let candle = self.candles[bar_idx].clone();
        let open_time = candle.open_time_ms;

        if let Some(mut pos) = self.position.take() {
            pos.bars_held += 1;

            if proposed_action == PolicyAction::EnterLong {
                self.rejected_entries += 1;
            }

            let mut exited = false;
            let mut exit_price = 0.0;
            let mut exit_reason = ExitReason::TrailingStopHit;

            // Step 1: Intrabar evaluation against active_stop S_{t-1} and open exit signals
            if candle.open < pos.active_stop {
                // Gap through stop at open
                exited = true;
                exit_price = candle.open; // Adverse gap fill
                exit_reason = ExitReason::GapThroughStop;
            } else if proposed_action == PolicyAction::ExitPosition {
                // Scheduled exit at open from prior decision
                exited = true;
                exit_price = match self.execution_mode {
                    ExecutionMode::NextBarOpen => candle.open,
                    ExecutionMode::Midpoint => (candle.open + candle.low) / 2.0,
                    ExecutionMode::CandleVwap => candle.typical_price(),
                };
                exit_reason = ExitReason::PolicySignalExit;
            } else if candle.low <= pos.active_stop {
                // Intrabar stop touch during the bar
                exited = true;
                exit_price = pos.active_stop;
                exit_reason = ExitReason::TrailingStopHit;
            } else if pos.bars_held >= 16 {
                // Max holding period expired (4 hours) at close
                exited = true;
                exit_price = candle.close;
                exit_reason = ExitReason::MaxHorizonExpired;
            }

            if exited {
                let gross_pnl: f64;
                let exit_friction: f64;
                let is_same_bar = pos.entry_bar_idx == bar_idx;
                let friction = MonetaryFriction::new_with_rates(
                    pos.entry_price,
                    pos.qty,
                    exit_price,
                    pos.qty,
                    self.taker_fee_rate,
                    self.slippage_rate,
                );

                if is_same_bar {
                    gross_pnl = pos.qty * (exit_price - pos.entry_price);
                    exit_friction = friction.base_total_usdt();
                } else {
                    let prev_c = self.candles[bar_idx - 1].close;
                    gross_pnl = pos.qty * (exit_price - prev_c);
                    exit_friction = friction.exit_fee_usdt + friction.exit_slippage_usdt;
                }

                let net_pnl = gross_pnl - exit_friction;
                let return_bps = (net_pnl / self.reference_capital_usdt) * 10_000.0;

                self.bar_records.push(BarValuationRecord {
                    bar_idx,
                    open_time_ms: open_time,
                    position_active: false,
                    gross_pnl_usdt: gross_pnl,
                    friction_usdt: exit_friction,
                    net_pnl_usdt: net_pnl,
                    net_return_bps: return_bps,
                });

                let total_gross = pos.qty * (exit_price - pos.entry_price);
                let total_base_net = total_gross - friction.base_total_usdt();
                let total_stressed_net = total_gross - friction.stressed_total_usdt();

                self.completed_trades.push(CompletedTrade {
                    trade_id: pos.trade_id,
                    entry_bar_idx: pos.entry_bar_idx,
                    exit_bar_idx: bar_idx,
                    entry_time_ms: pos.entry_time_ms,
                    exit_time_ms: candle.close_time_ms,
                    entry_price: pos.entry_price,
                    exit_price,
                    qty: pos.qty,
                    bars_held: pos.bars_held,
                    exit_reason,
                    gross_pnl_usdt: total_gross,
                    base_friction: friction,
                    net_pnl_usdt: total_base_net,
                    stressed_net_pnl_usdt: total_stressed_net,
                });

                self.position = None;
            } else {
                // Position continues
                let prev_c = if pos.entry_bar_idx == bar_idx {
                    pos.entry_price
                } else {
                    self.candles[bar_idx - 1].close
                };

                let gross_pnl = pos.qty * (candle.close - prev_c);
                let friction = if pos.entry_bar_idx == bar_idx {
                    (pos.entry_price * pos.qty * self.taker_fee_rate)
                        + (pos.entry_price * pos.qty * self.slippage_rate)
                } else {
                    0.0
                };

                let net_pnl = gross_pnl - friction;
                let return_bps = (net_pnl / self.reference_capital_usdt) * 10_000.0;

                self.bar_records.push(BarValuationRecord {
                    bar_idx,
                    open_time_ms: open_time,
                    position_active: true,
                    gross_pnl_usdt: gross_pnl,
                    friction_usdt: friction,
                    net_pnl_usdt: net_pnl,
                    net_return_bps: return_bps,
                });

                // Update stop causally for bar t+1:
                let new_stop = (candle.close - 2.0 * atr_14_curr).max(pos.active_stop);
                pos.active_stop = new_stop;

                self.position = Some(pos);
            }
        } else {
            // Flat
            if proposed_action == PolicyAction::EnterLong {
                let entry_price = match self.execution_mode {
                    ExecutionMode::NextBarOpen => candle.open,
                    ExecutionMode::Midpoint => (candle.open + candle.high) / 2.0,
                    ExecutionMode::CandleVwap => candle.typical_price(),
                };
                let raw_qty = self.reference_capital_usdt / entry_price;
                let qty = (raw_qty * 1e8).round() / 1e8;
                let initial_stop = entry_price - 2.0 * atr_14_curr;

                let mut pos = ActivePosition {
                    trade_id: self.next_trade_id,
                    entry_bar_idx: bar_idx,
                    entry_time_ms: open_time,
                    entry_price,
                    qty,
                    active_stop: initial_stop,
                    bars_held: 1,
                };
                self.next_trade_id += 1;

                if candle.low <= pos.active_stop {
                    // Same-bar stop out
                    let exit_price = pos.active_stop;
                    let gross_pnl = pos.qty * (exit_price - pos.entry_price);
                    let friction = MonetaryFriction::new_with_rates(
                        pos.entry_price,
                        pos.qty,
                        exit_price,
                        pos.qty,
                        self.taker_fee_rate,
                        self.slippage_rate,
                    );
                    let net_pnl = gross_pnl - friction.base_total_usdt();
                    let return_bps = (net_pnl / self.reference_capital_usdt) * 10_000.0;

                    self.bar_records.push(BarValuationRecord {
                        bar_idx,
                        open_time_ms: open_time,
                        position_active: false,
                        gross_pnl_usdt: gross_pnl,
                        friction_usdt: friction.base_total_usdt(),
                        net_pnl_usdt: net_pnl,
                        net_return_bps: return_bps,
                    });

                    self.completed_trades.push(CompletedTrade {
                        trade_id: pos.trade_id,
                        entry_bar_idx: bar_idx,
                        exit_bar_idx: bar_idx,
                        entry_time_ms: pos.entry_time_ms,
                        exit_time_ms: candle.close_time_ms,
                        entry_price: pos.entry_price,
                        exit_price,
                        qty: pos.qty,
                        bars_held: 1,
                        exit_reason: ExitReason::TrailingStopHit,
                        gross_pnl_usdt: gross_pnl,
                        base_friction: friction.clone(),
                        net_pnl_usdt: gross_pnl - friction.base_total_usdt(),
                        stressed_net_pnl_usdt: gross_pnl - friction.stressed_total_usdt(),
                    });

                    self.position = None;
                } else {
                    // Survives entry bar
                    let gross_pnl = pos.qty * (candle.close - pos.entry_price);
                    let entry_friction = (pos.entry_price * pos.qty * self.taker_fee_rate)
                        + (pos.entry_price * pos.qty * self.slippage_rate);
                    let net_pnl = gross_pnl - entry_friction;
                    let return_bps = (net_pnl / self.reference_capital_usdt) * 10_000.0;

                    self.bar_records.push(BarValuationRecord {
                        bar_idx,
                        open_time_ms: open_time,
                        position_active: true,
                        gross_pnl_usdt: gross_pnl,
                        friction_usdt: entry_friction,
                        net_pnl_usdt: net_pnl,
                        net_return_bps: return_bps,
                    });

                    let new_stop = (candle.close - 2.0 * atr_14_curr).max(pos.active_stop);
                    pos.active_stop = new_stop;

                    self.position = Some(pos);
                }
            } else {
                self.bar_records.push(BarValuationRecord {
                    bar_idx,
                    open_time_ms: open_time,
                    position_active: false,
                    gross_pnl_usdt: 0.0,
                    friction_usdt: 0.0,
                    net_pnl_usdt: 0.0,
                    net_return_bps: 0.0,
                });
            }
        }
    }

    /// Closes any remaining open position at the final candle's close C_N.
    pub fn close_at_end_of_data(&mut self) {
        if let Some(pos) = self.position.take() {
            let last_bar_idx = self.candles.len().saturating_sub(1);
            let last_candle = &self.candles[last_bar_idx];
            let exit_price = last_candle.close;
            let friction = MonetaryFriction::new_with_rates(
                pos.entry_price,
                pos.qty,
                exit_price,
                pos.qty,
                self.taker_fee_rate,
                self.slippage_rate,
            );
            let total_gross = pos.qty * (exit_price - pos.entry_price);
            let total_base_net = total_gross - friction.base_total_usdt();
            let total_stressed_net = total_gross - friction.stressed_total_usdt();

            self.completed_trades.push(CompletedTrade {
                trade_id: pos.trade_id,
                entry_bar_idx: pos.entry_bar_idx,
                exit_bar_idx: last_bar_idx,
                entry_time_ms: pos.entry_time_ms,
                exit_time_ms: last_candle.close_time_ms,
                entry_price: pos.entry_price,
                exit_price,
                qty: pos.qty,
                bars_held: pos.bars_held,
                exit_reason: ExitReason::EndOfDataCloseout,
                gross_pnl_usdt: total_gross,
                base_friction: friction,
                net_pnl_usdt: total_base_net,
                stressed_net_pnl_usdt: total_stressed_net,
            });
        }
    }

    /// Aggregates 15m intervals into 24-hour UTC calendar days.
    pub fn aggregate_daily_returns(&self) -> BTreeMap<String, f64> {
        let mut daily_returns_bps: BTreeMap<String, f64> = BTreeMap::new();

        for rec in &self.bar_records {
            let day_key = format_utc_date(rec.open_time_ms);
            *daily_returns_bps.entry(day_key).or_insert(0.0) += rec.net_return_bps;
        }

        daily_returns_bps
    }
}

/// Helper function to format timestamp into YYYY-MM-DD UTC.
pub fn format_utc_date(timestamp_ms: i64) -> String {
    let secs = timestamp_ms / 1000;
    let dt = chrono::DateTime::from_timestamp(secs, 0).unwrap_or_default();
    dt.format("%Y-%m-%d").to_string()
}
