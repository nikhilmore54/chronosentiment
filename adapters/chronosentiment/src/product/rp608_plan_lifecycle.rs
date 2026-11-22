//! RP-608 crypto/NSE plan lifecycle manage — Rust cutover (Gate 5 Adaptive TP + Gate 2X).
//!
//! Owns: Adaptive TP extend/lock, hard SL/TP (max extensions), Gate 2X Protect/Exit apply.
//! Does **not** own: WS I/O, entry signals, OF ±0.30 paper (still shadow-side).
//! Books never pooled. Aligned with `.cursor/rules/chronosentiment-core.mdc` §3c.

use serde::{Deserialize, Serialize};

use super::coralys_state_bridge::CoralysPositionAction;
use super::gate2x_decide::Gate2xDecision;
use super::gate2x_features::{
    Gate2xFeatureSnapshot, Gate2xParams, ProtectFloorMode, VolSurgeBucket,
};
use super::gate2x_position_manage::{Gate2xBookKind, Gate2xPositionManager};

pub const PLAN_LIFECYCLE_PROTOCOL: &str = "RP-608-PLAN-LIFECYCLE-MANAGE-001";
pub const ADAPTIVE_TP_CRYPTO: &str = "RP-608-GATE5-CRYPTO-ADAPTIVE-TP-001";
pub const ADAPTIVE_TP_NSE: &str = "RP-608-GATE5-NSE-ADAPTIVE-TP-001";
pub const TRAILING_SL_PROTOCOL: &str = "RP-608-TRAILING-SL-001";

fn round_tick(px: f64) -> f64 {
    (px * 100.0).round() / 100.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanLifecycleConfig {
    pub adaptive_tp_enabled: bool,
    pub adaptive_tp_extend_atr_mult: f64,
    pub adaptive_tp_max_extensions: u32,
    /// Fallback % of entry when atr_14 missing (NSE Arm A).
    pub adaptive_tp_extend_pct: f64,
    /// Favorable-only SL trail (never worse than original stop).
    #[serde(default = "default_true")]
    pub trailing_sl_enabled: bool,
    /// Trail distance = atr_mult × ATR(14) behind favorable extreme.
    #[serde(default = "default_trail_atr_mult")]
    pub trailing_sl_atr_mult: f64,
    /// Arm trail only after favorable move ≥ this fraction of initial risk.
    #[serde(default = "default_trail_arm_frac")]
    pub trailing_sl_arm_risk_frac: f64,
    pub round_trip_friction_bps: f64,
    pub meaningful_floor_bps: f64,
    pub early_sl_path_scratch_enabled: bool,
    pub early_sl_path_scratch_max_hold_s: f64,
    pub early_sl_path_friction_bps_side: f64,
    pub ambiguous_tie_break_tp: bool,
}

fn default_true() -> bool {
    true
}
fn default_trail_atr_mult() -> f64 {
    0.5
}
fn default_trail_arm_frac() -> f64 {
    0.30
}

impl Default for PlanLifecycleConfig {
    fn default() -> Self {
        Self {
            adaptive_tp_enabled: true,
            adaptive_tp_extend_atr_mult: 0.5,
            adaptive_tp_max_extensions: 3,
            adaptive_tp_extend_pct: 0.005,
            trailing_sl_enabled: true,
            trailing_sl_atr_mult: 0.5,
            trailing_sl_arm_risk_frac: 0.30,
            round_trip_friction_bps: 12.0,
            meaningful_floor_bps: 5.0,
            early_sl_path_scratch_enabled: true,
            early_sl_path_scratch_max_hold_s: 120.0,
            early_sl_path_friction_bps_side: 6.0,
            ambiguous_tie_break_tp: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanQuote {
    pub best_bid_price: f64,
    pub best_ask_price: f64,
    #[serde(default)]
    pub low_price: Option<f64>,
    #[serde(default)]
    pub high_price: Option<f64>,
    #[serde(default)]
    pub now_ms: Option<i64>,
    #[serde(default)]
    pub vol_surge: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenPlanState {
    pub plan_id: String,
    pub instrument: String,
    pub direction: String, // LONG | SHORT
    pub entry_price: f64,
    pub stop_loss_price: f64,
    pub profit_target_price: f64,
    pub atr_14: f64,
    pub simulated_quantity: f64,
    pub simulated_notional_usdt: f64,
    pub entry_time_ms: i64,
    pub expiry_time_ms: i64,
    /// Immutable opening stop — trail may only tighten vs this bound.
    #[serde(default)]
    pub original_stop_loss_price: Option<f64>,
    #[serde(default)]
    pub tp_level: u32,
    #[serde(default)]
    pub adaptive_tp_armed: bool,
    #[serde(default)]
    pub trailing_sl_armed: bool,
    #[serde(default)]
    pub mfe_exec_price: Option<f64>,
    #[serde(default)]
    pub mae_exec_price: Option<f64>,
    #[serde(default)]
    pub gate2x_peak_mfe: Option<f64>,
    #[serde(default)]
    pub gate2x_peak_tp_frac: Option<f64>,
    #[serde(default)]
    pub gate2x_mfe_armed: bool,
    #[serde(default)]
    pub gate2x_soft_breach_since_ms: Option<i64>,
    #[serde(default)]
    pub path_h_adverse_seen: bool,
    #[serde(default)]
    pub path_h_revisit_seen: bool,
    #[serde(default)]
    pub exit_vol_surge: Option<String>,
    #[serde(default)]
    pub book: Option<String>, // crypto | nse
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LifecycleCloseReason {
    ProfitTargetHit,
    StopLossHit,
    /// Industry-style label when a *trailed* stop (moved from original) is hit.
    /// Distinct from [`StopLossHit`] (initial / unmoved protective stop).
    TrailingStopHit,
    EarlySlPathScratch,
    Gate2xMfeGivebackProtect,
    MaxHorizonExpired,
    AmbiguousStopTarget,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecycleTickResult {
    pub protocol: String,
    pub closed: bool,
    pub plan: OpenPlanState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exit_reason: Option<LifecycleCloseReason>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exit_reason_detail: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exit_price: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub net_pnl: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gate2x_action: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ambiguous_resolved_as: Option<String>,
}

/// In-process plan lifecycle + Gate 2X manager (one book).
pub struct PlanLifecycleManager {
    gate2x: Gate2xPositionManager,
    cfg: PlanLifecycleConfig,
    book: Gate2xBookKind,
}

impl PlanLifecycleManager {
    pub fn load(
        evidence_dir: &str,
        book: Gate2xBookKind,
        cfg: PlanLifecycleConfig,
    ) -> Result<Self, String> {
        Ok(Self {
            gate2x: Gate2xPositionManager::load(evidence_dir, book)?,
            cfg,
            book,
        })
    }

    pub fn reset_position_latch(&mut self) {
        self.gate2x.reset_position_latch();
    }

    pub fn manage_tick(&mut self, mut plan: OpenPlanState, quote: PlanQuote) -> LifecycleTickResult {
        let now_ms = quote.now_ms.unwrap_or_else(|| {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as i64)
                .unwrap_or(0)
        });
        let direction = plan.direction.to_ascii_uppercase();
        let bid = quote.best_bid_price;
        let ask = quote.best_ask_price;
        let low = quote.low_price.unwrap_or_else(|| bid.min(ask));
        let high = quote.high_price.unwrap_or_else(|| bid.max(ask));
        let exec = if direction == "LONG" { bid } else { ask };

        // MFE / MAE marks
        if direction == "LONG" {
            let fav = bid.max(high);
            let adv = bid.min(low);
            plan.mfe_exec_price = Some(round_tick(match plan.mfe_exec_price {
                Some(p) => p.max(fav),
                None => fav,
            }));
            plan.mae_exec_price = Some(round_tick(match plan.mae_exec_price {
                Some(p) => p.min(adv),
                None => adv,
            }));
        } else {
            let fav = ask.min(low);
            let adv = ask.max(high);
            plan.mfe_exec_price = Some(round_tick(match plan.mfe_exec_price {
                Some(p) => p.min(fav),
                None => fav,
            }));
            plan.mae_exec_price = Some(round_tick(match plan.mae_exec_price {
                Some(p) => p.max(adv),
                None => adv,
            }));
        }

        // Gate 5 H120 path stamps (mid)
        let mid = 0.5 * (bid + ask);
        let band = plan.entry_price * (self.cfg.early_sl_path_friction_bps_side / 10000.0);
        if direction == "LONG" {
            if mid < plan.entry_price - band {
                plan.path_h_adverse_seen = true;
            }
            if plan.path_h_adverse_seen && mid >= plan.entry_price - band {
                plan.path_h_revisit_seen = true;
            }
        } else {
            if mid > plan.entry_price + band {
                plan.path_h_adverse_seen = true;
            }
            if plan.path_h_adverse_seen && mid <= plan.entry_price + band {
                plan.path_h_revisit_seen = true;
            }
        }

        if let Some(ref vs) = quote.vol_surge {
            plan.exit_vol_surge = Some(vs.to_ascii_uppercase());
        }

        // Favorable-only trailing SL (never worse than original), then Adaptive TP.
        self.apply_trailing_sl(&mut plan, &direction, exec);
        self.apply_adaptive_tp(&mut plan, &direction, bid, ask, low, high, now_ms);

        // Gate 2X
        let g2x = self.run_gate2x(&mut plan, exec, now_ms);
        if matches!(
            g2x.action,
            CoralysPositionAction::Protect | CoralysPositionAction::Exit
        ) {
            let friction = plan.simulated_notional_usdt
                * (self.cfg.round_trip_friction_bps / 10000.0);
            let gross = if direction == "LONG" {
                (exec - plan.entry_price) * plan.simulated_quantity
            } else {
                (plan.entry_price - exec) * plan.simulated_quantity
            };
            self.gate2x.reset_position_latch();
            return LifecycleTickResult {
                protocol: PLAN_LIFECYCLE_PROTOCOL.into(),
                closed: true,
                plan,
                exit_reason: Some(LifecycleCloseReason::Gate2xMfeGivebackProtect),
                exit_reason_detail: Some(g2x.reason.clone()),
                exit_price: Some(round_tick(exec)),
                net_pnl: Some(((gross - friction) * 100.0).round() / 100.0),
                gate2x_action: Some(format!("{:?}", g2x.action)),
                ambiguous_resolved_as: None,
            };
        }

        let tp = plan.profit_target_price;
        let sl = plan.stop_loss_price;
        let mfe = plan.mfe_exec_price.unwrap_or(exec);
        let mae = plan.mae_exec_price.unwrap_or(exec);
        let mut hit_tp = false;
        let mut hit_sl = false;
        if direction == "LONG" {
            hit_tp = bid >= tp || high >= tp || mfe >= tp;
            hit_sl = bid <= sl || low <= sl || mae <= sl;
        } else {
            hit_tp = ask <= tp || low <= tp || mfe <= tp;
            hit_sl = ask >= sl || high >= sl || mae >= sl;
        }

        let max_ext = self.cfg.adaptive_tp_max_extensions;
        if self.cfg.adaptive_tp_enabled && hit_tp && !hit_sl && plan.tp_level < max_ext {
            hit_tp = false;
        }
        let mut ambiguous_as: Option<String> = None;
        if hit_tp && hit_sl {
            if plan.tp_level >= 1 {
                hit_tp = false;
                ambiguous_as = Some("SL_ADAPTIVE_LOCK".into());
            } else if self.cfg.ambiguous_tie_break_tp {
                hit_sl = false;
                ambiguous_as = Some("TP".into());
            } else {
                hit_tp = false;
                ambiguous_as = Some("SL".into());
            }
        }

        let hold_s = if plan.entry_time_ms > 0 {
            (now_ms - plan.entry_time_ms) as f64 / 1000.0
        } else {
            0.0
        };

        if hit_sl {
            let path_ok = plan.path_h_adverse_seen && plan.path_h_revisit_seen;
            if self.cfg.early_sl_path_scratch_enabled
                && hold_s < self.cfg.early_sl_path_scratch_max_hold_s
                && path_ok
            {
                let friction = plan.simulated_notional_usdt
                    * (self.cfg.round_trip_friction_bps / 10000.0);
                let scratch_px = round_tick(plan.entry_price);
                self.gate2x.reset_position_latch();
                return LifecycleTickResult {
                    protocol: PLAN_LIFECYCLE_PROTOCOL.into(),
                    closed: true,
                    plan,
                    exit_reason: Some(LifecycleCloseReason::EarlySlPathScratch),
                    exit_reason_detail: Some("EARLY_SL_PATH_SCRATCH_GATE5_H120".into()),
                    exit_price: Some(scratch_px),
                    net_pnl: Some((-friction * 100.0).round() / 100.0),
                    gate2x_action: Some(format!("{:?}", g2x.action)),
                    ambiguous_resolved_as: ambiguous_as,
                };
            }
            let exit_px = sl;
            let friction = (plan.simulated_notional_usdt + exit_px * plan.simulated_quantity)
                * (self.cfg.round_trip_friction_bps / 10000.0);
            let gross = if direction == "LONG" {
                (exit_px - plan.entry_price) * plan.simulated_quantity
            } else {
                (plan.entry_price - exit_px) * plan.simulated_quantity
            };
            let orig_sl = plan
                .original_stop_loss_price
                .unwrap_or(plan.stop_loss_price);
            let trailed = plan.trailing_sl_armed
                && (plan.stop_loss_price - orig_sl).abs() > 1e-9;
            let (reason, detail) = if ambiguous_as.is_some() {
                (LifecycleCloseReason::AmbiguousStopTarget, None)
            } else if trailed {
                // Standard brokerage/OMS style: trailing stop fill ≠ initial stop loss.
                (
                    LifecycleCloseReason::TrailingStopHit,
                    Some("TRAILING_STOP_HIT".into()),
                )
            } else {
                (LifecycleCloseReason::StopLossHit, None)
            };
            self.gate2x.reset_position_latch();
            return LifecycleTickResult {
                protocol: PLAN_LIFECYCLE_PROTOCOL.into(),
                closed: true,
                plan,
                exit_reason: Some(reason),
                exit_reason_detail: detail,
                exit_price: Some(round_tick(exit_px)),
                net_pnl: Some(((gross - friction) * 100.0).round() / 100.0),
                gate2x_action: Some(format!("{:?}", g2x.action)),
                ambiguous_resolved_as: ambiguous_as,
            };
        }

        if hit_tp {
            let exit_px = tp;
            let friction = (plan.simulated_notional_usdt + exit_px * plan.simulated_quantity)
                * (self.cfg.round_trip_friction_bps / 10000.0);
            let gross = if direction == "LONG" {
                (exit_px - plan.entry_price) * plan.simulated_quantity
            } else {
                (plan.entry_price - exit_px) * plan.simulated_quantity
            };
            let detail = if self.cfg.adaptive_tp_enabled && plan.tp_level >= max_ext {
                Some("ADAPTIVE_TP_MAX_EXTENSIONS".into())
            } else {
                None
            };
            self.gate2x.reset_position_latch();
            return LifecycleTickResult {
                protocol: PLAN_LIFECYCLE_PROTOCOL.into(),
                closed: true,
                plan,
                exit_reason: Some(LifecycleCloseReason::ProfitTargetHit),
                exit_reason_detail: detail,
                exit_price: Some(round_tick(exit_px)),
                net_pnl: Some(((gross - friction) * 100.0).round() / 100.0),
                gate2x_action: Some(format!("{:?}", g2x.action)),
                ambiguous_resolved_as: ambiguous_as,
            };
        }

        if now_ms >= plan.expiry_time_ms {
            let friction = (plan.simulated_notional_usdt + exec * plan.simulated_quantity)
                * (self.cfg.round_trip_friction_bps / 10000.0);
            let gross = if direction == "LONG" {
                (exec - plan.entry_price) * plan.simulated_quantity
            } else {
                (plan.entry_price - exec) * plan.simulated_quantity
            };
            self.gate2x.reset_position_latch();
            return LifecycleTickResult {
                protocol: PLAN_LIFECYCLE_PROTOCOL.into(),
                closed: true,
                plan,
                exit_reason: Some(LifecycleCloseReason::MaxHorizonExpired),
                exit_reason_detail: Some("MAX_HORIZON_60M_EXPIRED".into()),
                exit_price: Some(round_tick(exec)),
                net_pnl: Some(((gross - friction) * 100.0).round() / 100.0),
                gate2x_action: Some(format!("{:?}", g2x.action)),
                ambiguous_resolved_as: None,
            };
        }

        LifecycleTickResult {
            protocol: PLAN_LIFECYCLE_PROTOCOL.into(),
            closed: false,
            plan,
            exit_reason: None,
            exit_reason_detail: None,
            exit_price: None,
            net_pnl: None,
            gate2x_action: Some(format!("{:?}", g2x.action)),
            ambiguous_resolved_as: None,
        }
    }

    /// Trail stop with favorable ticks. LONG: SL only rises; SHORT: SL only falls.
    /// Bound: never beyond `original_stop_loss_price` (never more damage than open risk).
    fn apply_trailing_sl(&self, plan: &mut OpenPlanState, direction: &str, exec: f64) {
        if !self.cfg.trailing_sl_enabled {
            return;
        }
        let orig = plan
            .original_stop_loss_price
            .unwrap_or(plan.stop_loss_price);
        plan.original_stop_loss_price = Some(orig);

        let entry = plan.entry_price;
        let risk = if direction == "LONG" {
            entry - orig
        } else {
            orig - entry
        };
        if risk <= 0.0 {
            return;
        }

        let mfe_px = plan.mfe_exec_price.unwrap_or(exec);
        let fav = if direction == "LONG" {
            mfe_px - entry
        } else {
            entry - mfe_px
        };
        let arm = risk * self.cfg.trailing_sl_arm_risk_frac.max(0.0);
        if fav < arm {
            return;
        }

        let trail_dist = if plan.atr_14 > 0.0 {
            self.cfg.trailing_sl_atr_mult * plan.atr_14
        } else {
            risk * 0.5
        };
        if trail_dist <= 0.0 {
            return;
        }

        let proposed = if direction == "LONG" {
            round_tick(mfe_px - trail_dist)
        } else {
            round_tick(mfe_px + trail_dist)
        };

        let mut sl = plan.stop_loss_price;
        if direction == "LONG" {
            // Tighten up only; never below original; keep below MFE (no instant stop).
            let cand = proposed.max(orig);
            if cand > sl && cand < mfe_px {
                sl = cand;
            }
        } else {
            // Tighten down only; never above original; keep above MFE.
            let cand = proposed.min(orig);
            if cand < sl && cand > mfe_px {
                sl = cand;
            }
        }
        if (sl - plan.stop_loss_price).abs() > 1e-9 {
            plan.stop_loss_price = sl;
            plan.trailing_sl_armed = true;
        }
    }

    fn apply_adaptive_tp(
        &self,
        plan: &mut OpenPlanState,
        direction: &str,
        bid: f64,
        ask: f64,
        low: f64,
        high: f64,
        now_ms: i64,
    ) {
        if !self.cfg.adaptive_tp_enabled {
            return;
        }
        let max_ext = self.cfg.adaptive_tp_max_extensions;
        if plan.tp_level >= max_ext {
            return;
        }
        let step = if plan.atr_14 > 0.0 {
            self.cfg.adaptive_tp_extend_atr_mult * plan.atr_14
        } else {
            self.cfg.adaptive_tp_extend_pct * plan.entry_price
        };
        if step <= 0.0 {
            return;
        }
        let mfe = plan.mfe_exec_price.unwrap_or(if direction == "LONG" {
            bid
        } else {
            ask
        });
        let mut tp = plan.profit_target_price;
        let mut sl = plan.stop_loss_price;
        let mut level = plan.tp_level;
        let mut extended = false;
        while level < max_ext {
            let still_hit = if direction == "LONG" {
                bid >= tp || high >= tp || mfe >= tp
            } else {
                ask <= tp || low <= tp || mfe <= tp
            };
            if !still_hit {
                break;
            }
            let prior_tp = tp;
            if direction == "LONG" {
                tp = round_tick(prior_tp + step);
                let new_sl = round_tick(prior_tp);
                if new_sl > sl {
                    sl = new_sl;
                }
            } else {
                tp = round_tick(prior_tp - step);
                let new_sl = round_tick(prior_tp);
                if new_sl < sl {
                    sl = new_sl;
                }
            }
            level += 1;
            extended = true;
        }
        if extended {
            plan.profit_target_price = tp;
            plan.stop_loss_price = sl;
            plan.tp_level = level;
            plan.adaptive_tp_armed = true;
            let _ = now_ms;
            let _ = self.book; // book-specific protocol stamped by caller if needed
        }
    }

    fn run_gate2x(
        &mut self,
        plan: &mut OpenPlanState,
        exec: f64,
        now_ms: i64,
    ) -> Gate2xDecision {
        let qty = plan.simulated_quantity.max(0.0);
        let entry = plan.entry_price;
        let direction = plan.direction.to_ascii_uppercase();
        let mfe_px = plan.mfe_exec_price.unwrap_or(exec);
        let peak_mfe = if direction == "LONG" {
            (mfe_px - entry) * qty
        } else {
            (entry - mfe_px) * qty
        }
        .max(0.0);
        let prev_peak = plan.gate2x_peak_mfe.unwrap_or(0.0);
        let peak_mfe = peak_mfe.max(prev_peak);
        plan.gate2x_peak_mfe = Some(peak_mfe);

        let tp_span = if direction == "LONG" {
            plan.profit_target_price - entry
        } else {
            entry - plan.profit_target_price
        };
        let tp_frac = if tp_span > 0.0 {
            if direction == "LONG" {
                (mfe_px - entry) / tp_span
            } else {
                (entry - mfe_px) / tp_span
            }
        } else {
            0.0
        };
        let prev_tf = plan.gate2x_peak_tp_frac.unwrap_or(0.0);
        let peak_tp_frac = tp_frac.max(prev_tf);
        plan.gate2x_peak_tp_frac = Some(peak_tp_frac);

        let cur_mfe = if direction == "LONG" {
            (exec - entry) * qty
        } else {
            (entry - exec) * qty
        };

        let notional = if plan.simulated_notional_usdt > 0.0 {
            plan.simulated_notional_usdt
        } else {
            entry * qty
        };
        let arm_threshold = notional
            * (self
                .cfg
                .round_trip_friction_bps
                .max(self.cfg.meaningful_floor_bps)
                / 10000.0);

        let hold_s = if plan.entry_time_ms > 0 {
            (now_ms - plan.entry_time_ms) as f64 / 1000.0
        } else {
            0.0
        };

        let mut params = match self.book {
            Gate2xBookKind::Crypto => Gate2xParams::crypto_defaults(),
            Gate2xBookKind::Nse => Gate2xParams::nse_defaults(),
        };
        params.protect_floor = ProtectFloorMode::CostClear;

        let snap = Gate2xFeatureSnapshot {
            decision_id: plan.plan_id.clone(),
            instrument: plan.instrument.clone(),
            direction: direction.clone(),
            reference_price: Some(entry),
            peak_mfe,
            current_mfe: cur_mfe,
            peak_tp_frac,
            hold_s,
            arm_threshold,
            already_armed: plan.gate2x_mfe_armed,
            vol_surge: VolSurgeBucket::parse(
                plan.exit_vol_surge.as_deref().unwrap_or("UNK"),
            ),
            soft_breach_since_ms: plan.gate2x_soft_breach_since_ms,
            now_ms,
            params,
            train_vol_median: 0.001,
            tp_level: plan.tp_level,
            adaptive_armed: plan.adaptive_tp_armed || plan.tp_level >= 1,
        };

        let d = self.gate2x.manage_tick(snap);
        plan.gate2x_mfe_armed = d.soft_breach.armed || plan.gate2x_mfe_armed;
        plan.gate2x_soft_breach_since_ms = d.soft_breach.soft_breach_since_ms;
        d
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn write_store(dir: &std::path::Path) {
        let path = dir.join("BTCUSDT.jsonl");
        let mut f = std::fs::File::create(&path).unwrap();
        for i in 0..40 {
            let outcome = if i % 3 == 0 {
                "TARGET_BEFORE_RISK"
            } else {
                "RISK_BEFORE_TARGET"
            };
            let line = format!(
                "{{\"ticker\":\"BTCUSDT\",\"trend\":\"Bearish\",\"momentum\":\"Negative\",\"volatility\":\"present\",\"direction\":\"SHORT\",\"relative_volume_20\":2.2,\"mfe_pct\":[0.1,0.2,0.3,0.4,0.5,0.5,0.5,0.5,0.5,0.5],\"mae_pct\":[-0.1,-0.1,-0.2,-0.2,-0.3,-0.3,-0.3,-0.3,-0.3,-0.3],\"outcome\":\"{outcome}\",\"sessions_to_outcome\":3.0}}\n"
            );
            f.write_all(line.as_bytes()).unwrap();
        }
    }

    fn short_plan(entry: f64, atr: f64) -> OpenPlanState {
        let tp = round_tick(entry - 1.5 * atr);
        let sl = round_tick(entry + 1.0 * atr);
        OpenPlanState {
            plan_id: "PLAN-TEST".into(),
            instrument: "BTCUSDT".into(),
            direction: "SHORT".into(),
            entry_price: entry,
            stop_loss_price: sl,
            profit_target_price: tp,
            atr_14: atr,
            simulated_quantity: 0.01,
            simulated_notional_usdt: 1000.0,
            entry_time_ms: 1_000_000,
            expiry_time_ms: 1_000_000 + 3_600_000,
            original_stop_loss_price: Some(sl),
            tp_level: 0,
            adaptive_tp_armed: false,
            trailing_sl_armed: false,
            mfe_exec_price: None,
            mae_exec_price: None,
            gate2x_peak_mfe: None,
            gate2x_peak_tp_frac: None,
            gate2x_mfe_armed: false,
            gate2x_soft_breach_since_ms: None,
            path_h_adverse_seen: false,
            path_h_revisit_seen: false,
            exit_vol_surge: Some("UNK".into()),
            book: Some("crypto".into()),
        }
    }

    #[test]
    fn trailing_sl_short_tightens_never_worse_than_original() {
        let dir = std::env::temp_dir().join("rp608_plan_lc_trail");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        write_store(&dir);
        let mut cfg = PlanLifecycleConfig::default();
        cfg.adaptive_tp_enabled = false; // isolate trail
        cfg.trailing_sl_enabled = true;
        cfg.trailing_sl_atr_mult = 0.5;
        cfg.trailing_sl_arm_risk_frac = 0.30;
        let mut mgr =
            PlanLifecycleManager::load(dir.to_str().unwrap(), Gate2xBookKind::Crypto, cfg)
                .unwrap();
        let entry = 100.0;
        let atr = 10.0;
        let mut plan = short_plan(entry, atr);
        let orig_sl = plan.stop_loss_price; // 110
        // Favorable move: ask/low down to 95 → fav=5 = 50% of risk 10 → armed.
        // Keep high below trailed SL so we do not trip STOP_LOSS this tick.
        let q = PlanQuote {
            best_bid_price: 94.9,
            best_ask_price: 95.0,
            low_price: Some(94.5),
            high_price: Some(98.0),
            now_ms: Some(1_000_000 + 30_000),
            vol_surge: Some("UNK".into()),
        };
        let r = mgr.manage_tick(plan, q);
        assert!(!r.closed);
        assert!(r.plan.trailing_sl_armed);
        assert!(r.plan.stop_loss_price < orig_sl);
        assert!(r.plan.stop_loss_price <= orig_sl);
        // proposed ≈ mfe(94.5) + 0.5*atr = 94.5+5 = 99.5
        assert!((r.plan.stop_loss_price - 99.5).abs() < 0.05);
        assert_eq!(r.plan.original_stop_loss_price, Some(orig_sl));

        // Adverse tick must not loosen SL back toward original
        plan = r.plan;
        let locked = plan.stop_loss_price;
        let q2 = PlanQuote {
            best_bid_price: 101.0,
            best_ask_price: 101.2,
            low_price: Some(94.5),
            high_price: Some(101.5),
            now_ms: Some(1_000_000 + 60_000),
            vol_surge: Some("UNK".into()),
        };
        let r2 = mgr.manage_tick(plan, q2);
        assert!(r2.plan.stop_loss_price <= locked + 1e-9);
        assert!(r2.plan.stop_loss_price <= orig_sl + 1e-9);
    }

    #[test]
    fn trailing_stop_hit_uses_industry_exit_reason() {
        let dir = std::env::temp_dir().join("rp608_plan_lc_trail_hit");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        write_store(&dir);
        let mut cfg = PlanLifecycleConfig::default();
        cfg.adaptive_tp_enabled = false;
        cfg.trailing_sl_enabled = true;
        cfg.trailing_sl_atr_mult = 0.5;
        cfg.trailing_sl_arm_risk_frac = 0.30;
        cfg.early_sl_path_scratch_enabled = false;
        let mut mgr =
            PlanLifecycleManager::load(dir.to_str().unwrap(), Gate2xBookKind::Crypto, cfg)
                .unwrap();
        let entry = 100.0;
        let atr = 10.0;
        let mut plan = short_plan(entry, atr);
        let orig_sl = plan.stop_loss_price;
        // Arm trail (favorable)
        let q = PlanQuote {
            best_bid_price: 94.9,
            best_ask_price: 95.0,
            low_price: Some(94.5),
            high_price: Some(98.0),
            now_ms: Some(1_000_000 + 30_000),
            vol_surge: Some("UNK".into()),
        };
        let r = mgr.manage_tick(plan, q);
        assert!(!r.closed);
        assert!(r.plan.trailing_sl_armed);
        assert!(r.plan.stop_loss_price < orig_sl);
        plan = r.plan;
        let trail = plan.stop_loss_price;
        // Trip the trailed stop (not the original): ask/high through trail
        let q_hit = PlanQuote {
            best_bid_price: trail + 0.2,
            best_ask_price: trail + 0.4,
            low_price: Some(94.5),
            high_price: Some(trail + 0.5),
            now_ms: Some(1_000_000 + 60_000),
            vol_surge: Some("UNK".into()),
        };
        let r_hit = mgr.manage_tick(plan, q_hit);
        assert!(r_hit.closed);
        match r_hit.exit_reason {
            Some(LifecycleCloseReason::TrailingStopHit) => {}
            other => panic!("expected TrailingStopHit, got {:?}", other),
        }
        assert_eq!(
            r_hit.exit_reason_detail.as_deref(),
            Some("TRAILING_STOP_HIT")
        );
    }

    #[test]
    fn adaptive_tp_extends_short_on_first_touch() {
        let dir = std::env::temp_dir().join("rp608_plan_lc_adapt");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        write_store(&dir);
        let mut mgr = PlanLifecycleManager::load(
            dir.to_str().unwrap(),
            Gate2xBookKind::Crypto,
            PlanLifecycleConfig::default(),
        )
        .unwrap();
        let plan = short_plan(80170.0, 264.43);
        let tp0 = plan.profit_target_price;
        let q = PlanQuote {
            best_bid_price: tp0 - 1.0,
            best_ask_price: tp0 - 0.5,
            low_price: None,
            high_price: None,
            now_ms: Some(1_000_000 + 5_000),
            vol_surge: Some("UNK".into()),
        };
        let r = mgr.manage_tick(plan, q);
        assert!(!r.closed);
        assert_eq!(r.plan.tp_level, 1);
        assert!(r.plan.adaptive_tp_armed);
        assert!((r.plan.stop_loss_price - tp0).abs() < 0.02);
    }
}
