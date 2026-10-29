#!/usr/bin/env python3
"""
rp612_temporal_trajectory_evaluator.py

RP-612 — Coralys Temporal Trajectory Algorithm Evaluator
Research-Only (0% Execution Authority) · Milestone RP-612-001

Implements the deterministic, read-only shadow evaluator for post-entry
multi-horizon trajectory state evolution under the frozen Coralys framework.
Preserves RP-605 entry decisions and direct 1-to-1 Trade ID pairing.
"""

from __future__ import annotations

import json
import math
import sys
from dataclasses import asdict, dataclass, field
from datetime import datetime, timezone
from enum import Enum
from pathlib import Path
from typing import Any, Dict, List, Optional, Tuple

WORKSPACE_DIR = Path(__file__).parent.parent
sys.path.insert(0, str(WORKSPACE_DIR))
sys.path.insert(0, str(WORKSPACE_DIR / "scripts"))

# Import canonical score calculation - NEVER reimplement or modify
from scripts.run_rp605_nse_prospective_freshness_validation import compute_coralys_scores


class TrajectoryState(str, Enum):
    IMPULSE_CONTINUATION = "IMPULSE_CONTINUATION"
    TRANSIENT_PULLBACK = "TRANSIENT_PULLBACK"
    PERSISTENT_PULLBACK = "PERSISTENT_PULLBACK"
    STRUCTURAL_REVERSAL_WARNING = "STRUCTURAL_REVERSAL_WARNING"
    RAPID_REVERSAL_WARNING = "RAPID_REVERSAL_WARNING"
    PROFIT_HARVEST = "PROFIT_HARVEST"
    EXIT_RECOMMENDED = "EXIT_RECOMMENDED"


class ExitReason(str, Enum):
    RAPID_REVERSAL_ABORT = "RAPID_REVERSAL_ABORT"
    STRUCTURAL_REVERSAL_ABORT = "STRUCTURAL_REVERSAL_ABORT"
    PROFIT_HARVEST_STOP = "PROFIT_HARVEST_STOP"
    RP605_TARGET_PARITY = "RP605_TARGET_PARITY"
    HARD_MAX_EXPIRY = "HARD_MAX_EXPIRY"


@dataclass(frozen=True)
class EvaluatorConfig:
    cadence_sec: int = 60
    s_neutral_boundary: float = 0.0000
    s_recovery_boundary: float = 0.0500
    tau_persist_sec: int = 180       # 3 1-minute bars
    tau_rapid_sec: int = 900         # 15 minutes post-entry
    tau_confirm_sec: int = 60        # 1 bar confirmation
    theta_profit: float = 0.0025     # +25 bps MFE (+0.25%)
    delta_decel: float = 0.0050      # Velocity deceleration threshold in score units / minute
    trailing_giveback_pct: float = 0.40  # 40% giveback of peak MFE
    slippage_bps: float = 2.0
    fee_bps: float = 5.0
    total_friction_bps: float = 7.0


@dataclass
class TrajectoryObservation:
    timestamp_utc: str
    timestamp_epoch: int
    current_price: float
    s15: float
    s30: float
    s1d: float
    s15_tilde: float
    s30_tilde: float
    s1d_tilde: float
    p_severity: int
    s15_velocity_per_min: float
    s15_acceleration_per_min2: float
    tau_adverse_sec: int
    peak_mfe: float
    current_return: float
    state: TrajectoryState


@dataclass
class RP612EvaluationRecord:
    trade_id: str
    symbol: str
    direction: str
    entry_timestamp_utc: str
    entry_timestamp_epoch: int
    entry_price: float
    quantity: float
    
    # RP-605 Control Benchmark
    rp605_exit_timestamp_utc: str
    rp605_exit_timestamp_epoch: int
    rp605_exit_price: float
    rp605_exit_reason: str
    rp605_holding_sec: int
    rp605_gross_pnl: float
    rp605_friction: float
    rp605_net_pnl: float
    rp605_net_paise: int = 0
    
    # RP-612 Counterfactual Shadow
    recommendation_emitted: bool = False
    recommendation_reason: Optional[str] = None
    recommendation_state: Optional[str] = None
    recommendation_timestamp_utc: Optional[str] = None
    recommendation_timestamp_epoch: Optional[int] = None
    simulated_fill_timestamp_utc: Optional[str] = None
    simulated_fill_timestamp_epoch: Optional[int] = None
    simulated_fill_price: Optional[float] = None
    rp612_holding_sec: Optional[int] = None
    rp612_gross_pnl: Optional[float] = None
    rp612_friction: Optional[float] = None
    rp612_net_pnl: Optional[float] = None
    rp612_net_paise: int = 0
    
    # Paired Accounting Comparison
    paired_net_delta: float = 0.0
    paired_paise_delta: int = 0
    loss_truncation_achieved: bool = False
    premature_exit_flag: bool = False
    observations: List[TrajectoryObservation] = field(default_factory=list)


class TemporalTrajectoryStateMachine:
    """
    Deterministic path-dependent finite-state machine evaluating
    a single position lifecycle.
    """

    def __init__(self, direction: str, entry_price: float, config: EvaluatorConfig):
        self.direction = direction.upper()
        self.d = 1.0 if self.direction == "LONG" else -1.0
        self.entry_price = entry_price
        self.config = config
        
        self.state = TrajectoryState.IMPULSE_CONTINUATION
        self.tau_adverse = 0
        self.warning_armed_timestamp: Optional[int] = None
        self.warning_armed_state: Optional[TrajectoryState] = None
        
        self.peak_mfe = 0.0
        self.prev_s15_tilde: Optional[float] = None
        self.prev_velocity_per_min: Optional[float] = None
        
        # Historical cascade timestamps
        self.t_15_down: Optional[int] = None
        self.t_30_down: Optional[int] = None
        self.t_1d_down: Optional[int] = None
        
        self.exit_recommended = False
        self.exit_reason: Optional[ExitReason] = None
        self.exit_timestamp: Optional[int] = None

    def evaluate_bar(
        self,
        timestamp_epoch: int,
        timestamp_utc: str,
        current_price: float,
        price_series: List[float],
        elapsed_sec: int,
    ) -> TrajectoryObservation:
        """
        Evaluates a single completed bar point-in-time.
        """
        # 1. Canonical Coralys Scores (imported from frozen implementation)
        s15, s30, s1d = compute_coralys_scores(price_series)
        s15_tilde = s15 * self.d
        s30_tilde = s30 * self.d
        s1d_tilde = s1d * self.d
        
        # 2. Propagation Severity
        p_severity = (
            (1 if s15_tilde < self.config.s_neutral_boundary else 0)
            + (1 if s30_tilde < self.config.s_neutral_boundary else 0)
            + (1 if s1d_tilde < self.config.s_neutral_boundary else 0)
        )
        
        # Track cascade order
        if s15_tilde < 0 and self.t_15_down is None:
            self.t_15_down = timestamp_epoch
        if s30_tilde < 0 and self.t_30_down is None:
            self.t_30_down = timestamp_epoch
        if s1d_tilde < 0 and self.t_1d_down is None:
            self.t_1d_down = timestamp_epoch
            
        # 3. Trajectory Derivatives (dt_min in minutes)
        dt_min = float(self.config.cadence_sec) / 60.0
        if self.prev_s15_tilde is not None:
            velocity_per_min = (s15_tilde - self.prev_s15_tilde) / dt_min
        else:
            velocity_per_min = 0.0
            
        if self.prev_velocity_per_min is not None:
            acceleration_per_min2 = (velocity_per_min - self.prev_velocity_per_min) / dt_min
        else:
            acceleration_per_min2 = 0.0
            
        self.prev_s15_tilde = s15_tilde
        self.prev_velocity_per_min = velocity_per_min
        
        # 4. Excursion Tracking (in return terms)
        signed_ret = ((current_price - self.entry_price) / self.entry_price) * self.d
        if signed_ret > self.peak_mfe:
            self.peak_mfe = signed_ret
            
        # 5. Dwell Accumulator Update with explicit neutral-state handling
        if p_severity >= 1:
            self.tau_adverse += self.config.cadence_sec
        elif p_severity == 0 and s15_tilde >= self.config.s_recovery_boundary:
            self.tau_adverse = 0  # Confirmed recovery: clean reset
        # Note: If p_severity == 0 and 0.0 <= s15_tilde < s_recovery_boundary (neutral),
        # tau_adverse is paused (holds prior value, neither increments nor resets).
            
        # 6. State Transition Logic (Strict Precedence)
        prior_state = self.state
        new_state = prior_state
        
        if not self.exit_recommended:
            # Candidate Condition Checks
            is_rapid_reversal = (
                p_severity == 3 and elapsed_sec <= self.config.tau_rapid_sec
            )
            is_structural_reversal = (
                p_severity >= 2 and s1d_tilde <= self.config.s_neutral_boundary
            )
            # Profit Harvest Entry: MFE >= theta_profit, velocity decelerating <= -delta_decel, s15 still supportive
            is_profit_harvest_active = (
                self.peak_mfe >= self.config.theta_profit
                and velocity_per_min <= -self.config.delta_decel
                and s15_tilde >= self.config.s_neutral_boundary
            )
            giveback = (self.peak_mfe - signed_ret) if self.peak_mfe > 0 else 0.0
            is_profit_giveback_hit = (
                self.state == TrajectoryState.PROFIT_HARVEST
                and self.peak_mfe >= self.config.theta_profit
                and (giveback / self.peak_mfe) >= self.config.trailing_giveback_pct
            )
            
            # --- EVALUATE TRANSITIONS UNDER DETERMINISTIC PRECEDENCE ---
            
            # Priority 1: Emergency Risk Truncation (Rapid Reversal)
            if is_rapid_reversal:
                if self.state == TrajectoryState.RAPID_REVERSAL_WARNING:
                    if self.warning_armed_timestamp is not None and (timestamp_epoch - self.warning_armed_timestamp) >= self.config.tau_confirm_sec:
                        self.exit_recommended = True
                        self.exit_reason = ExitReason.RAPID_REVERSAL_ABORT
                        self.exit_timestamp = timestamp_epoch
                        new_state = TrajectoryState.EXIT_RECOMMENDED
                else:
                    new_state = TrajectoryState.RAPID_REVERSAL_WARNING
                    self.warning_armed_timestamp = timestamp_epoch
                    self.warning_armed_state = TrajectoryState.RAPID_REVERSAL_WARNING

            # Priority 2: Confirmed Structural Breakdown
            elif is_structural_reversal:
                if self.state == TrajectoryState.STRUCTURAL_REVERSAL_WARNING:
                    if self.warning_armed_timestamp is not None and (timestamp_epoch - self.warning_armed_timestamp) >= self.config.tau_confirm_sec:
                        self.exit_recommended = True
                        self.exit_reason = ExitReason.STRUCTURAL_REVERSAL_ABORT
                        self.exit_timestamp = timestamp_epoch
                        new_state = TrajectoryState.EXIT_RECOMMENDED
                else:
                    new_state = TrajectoryState.STRUCTURAL_REVERSAL_WARNING
                    self.warning_armed_timestamp = timestamp_epoch
                    self.warning_armed_state = TrajectoryState.STRUCTURAL_REVERSAL_WARNING

            # Priority 3: Gain Preservation / Profit Harvest Stop
            elif is_profit_giveback_hit:
                self.exit_recommended = True
                self.exit_reason = ExitReason.PROFIT_HARVEST_STOP
                self.exit_timestamp = timestamp_epoch
                new_state = TrajectoryState.EXIT_RECOMMENDED

            elif is_profit_harvest_active and self.state in {
                TrajectoryState.IMPULSE_CONTINUATION,
                TrajectoryState.TRANSIENT_PULLBACK,
            }:
                new_state = TrajectoryState.PROFIT_HARVEST

            # Priority 4: Transient -> Persistent Escalation
            elif (
                self.state == TrajectoryState.TRANSIENT_PULLBACK
                and self.tau_adverse >= self.config.tau_persist_sec
                and s1d_tilde > self.config.s_neutral_boundary
            ):
                new_state = TrajectoryState.PERSISTENT_PULLBACK

            # Priority 5: Continuation -> Transient Pullback
            elif (
                self.state == TrajectoryState.IMPULSE_CONTINUATION
                and s15_tilde < self.config.s_neutral_boundary
                and s1d_tilde > self.config.s_neutral_boundary
            ):
                new_state = TrajectoryState.TRANSIENT_PULLBACK

            # Priority 6: Recovery / Reset to Continuation
            elif (
                self.state in {TrajectoryState.TRANSIENT_PULLBACK, TrajectoryState.PERSISTENT_PULLBACK, TrajectoryState.PROFIT_HARVEST}
                and s15_tilde >= self.config.s_recovery_boundary
                and s1d_tilde > self.config.s_neutral_boundary
            ):
                new_state = TrajectoryState.IMPULSE_CONTINUATION
                self.warning_armed_timestamp = None
                self.warning_armed_state = None

            # Disarm warning if condition cleared
            if (
                self.state in {TrajectoryState.RAPID_REVERSAL_WARNING, TrajectoryState.STRUCTURAL_REVERSAL_WARNING}
                and not is_rapid_reversal
                and not is_structural_reversal
            ):
                if s1d_tilde > self.config.s_neutral_boundary:
                    new_state = (
                        TrajectoryState.PERSISTENT_PULLBACK
                        if self.tau_adverse >= self.config.tau_persist_sec
                        else TrajectoryState.TRANSIENT_PULLBACK
                    )
                    self.warning_armed_timestamp = None
                    self.warning_armed_state = None

        self.state = new_state

        return TrajectoryObservation(
            timestamp_utc=timestamp_utc,
            timestamp_epoch=timestamp_epoch,
            current_price=current_price,
            s15=s15,
            s30=s30,
            s1d=s1d,
            s15_tilde=s15_tilde,
            s30_tilde=s30_tilde,
            s1d_tilde=s1d_tilde,
            p_severity=p_severity,
            s15_velocity_per_min=velocity_per_min,
            s15_acceleration_per_min2=acceleration_per_min2,
            tau_adverse_sec=self.tau_adverse,
            peak_mfe=self.peak_mfe,
            current_return=signed_ret,
            state=self.state,
        )


def simulate_trade_shadow(
    trade_row: Dict[str, Any],
    bars_timeline: List[Dict[str, Any]],
    config: EvaluatorConfig = EvaluatorConfig(),
) -> RP612EvaluationRecord:
    """
    Evaluates an RP-605 trade against the RP-612 shadow temporal state machine.
    Direct 1-to-1 deterministic pairing by trade_id.
    """
    tid = trade_row["trade_id"]
    sym = trade_row["symbol"]
    side = trade_row["side"].upper()
    entry_px = float(trade_row["entry_price"])
    qty = float(trade_row["quantity"])
    d = 1.0 if side == "LONG" else -1.0
    
    t_entry_epoch = int(trade_row["entry_timestamp_epoch"])
    t_exit_epoch = int(trade_row["exit_timestamp_epoch"])
    rp605_exit_px = float(trade_row["exit_price"])
    rp605_exit_reason = trade_row.get("exit_reason", "RP605_EXIT")
    
    # RP-605 Control Exact Accounting
    rp605_gross = (rp605_exit_px - entry_px) * qty * d
    rp605_notional = entry_px * qty + rp605_exit_px * qty
    rp605_friction = rp605_notional * (config.total_friction_bps / 10000.0)
    rp605_net = rp605_gross - rp605_friction
    rp605_holding_sec = t_exit_epoch - t_entry_epoch
    
    record = RP612EvaluationRecord(
        trade_id=tid,
        symbol=sym,
        direction=side,
        entry_timestamp_utc=trade_row["entry_timestamp_utc"],
        entry_timestamp_epoch=t_entry_epoch,
        entry_price=entry_px,
        quantity=qty,
        rp605_exit_timestamp_utc=trade_row["exit_timestamp_utc"],
        rp605_exit_timestamp_epoch=t_exit_epoch,
        rp605_exit_price=rp605_exit_px,
        rp605_exit_reason=rp605_exit_reason,
        rp605_holding_sec=rp605_holding_sec,
        rp605_gross_pnl=rp605_gross,
        rp605_friction=rp605_friction,
        rp605_net_pnl=rp605_net,
    )
    
    sm = TemporalTrajectoryStateMachine(direction=side, entry_price=entry_px, config=config)
    
    # Process completed bars in chronological order strictly within [t_entry, t_exit]
    for idx, bar in enumerate(bars_timeline):
        bar_epoch = int(bar["timestamp_epoch"])
        if bar_epoch < t_entry_epoch:
            continue
        if bar_epoch > t_exit_epoch:
            break
            
        elapsed = bar_epoch - t_entry_epoch
        obs = sm.evaluate_bar(
            timestamp_epoch=bar_epoch,
            timestamp_utc=bar["timestamp_utc"],
            current_price=float(bar["close"]),
            price_series=bar["history_closes"],
            elapsed_sec=elapsed,
        )
        record.observations.append(obs)
        
        # Check if exit recommendation emitted
        if sm.exit_recommended:
            record.recommendation_emitted = True
            record.recommendation_reason = sm.exit_reason.value if sm.exit_reason else None
            record.recommendation_state = obs.state.value
            record.recommendation_timestamp_utc = bar["timestamp_utc"]
            record.recommendation_timestamp_epoch = bar_epoch
            
            # Simulated fill at next available print (or current bar close + slippage if terminal)
            fill_px = float(bar["close"])
            if d > 0:  # Selling long
                fill_px *= (1.0 - config.slippage_bps / 10000.0)
            else:      # Buying to cover short
                fill_px *= (1.0 + config.slippage_bps / 10000.0)
                
            fill_epoch = bar_epoch + 1
            record.simulated_fill_timestamp_epoch = fill_epoch
            record.simulated_fill_price = fill_px
            record.rp612_holding_sec = max(60, bar_epoch - t_entry_epoch)
            
            rp612_gross = (fill_px - entry_px) * qty * d
            rp612_notional = entry_px * qty + fill_px * qty
            rp612_friction = rp612_notional * (config.total_friction_bps / 10000.0)
            rp612_net = rp612_gross - rp612_friction
            
            record.rp612_gross_pnl = rp612_gross
            record.rp612_friction = rp612_friction
            record.rp612_net_pnl = rp612_net
            record.rp605_net_paise = round(rp605_net * 100)
            record.rp612_net_paise = round(rp612_net * 100)
            record.paired_net_delta = rp612_net - rp605_net
            record.paired_paise_delta = record.rp612_net_paise - record.rp605_net_paise
            
            # Diagnostic Flags
            if rp605_net < 0 and rp612_net > rp605_net:
                record.loss_truncation_achieved = True
            if rp605_gross > 0 and rp612_gross < (0.50 * rp605_gross):
                record.premature_exit_flag = True
            break
            
    # Parity fallback: If no early exit recommended, trade matches RP-605 target exit
    if not record.recommendation_emitted:
        record.recommendation_reason = ExitReason.RP605_TARGET_PARITY.value
        record.recommendation_state = sm.state.value
        record.simulated_fill_timestamp_epoch = t_exit_epoch
        record.simulated_fill_price = rp605_exit_px
        record.rp612_holding_sec = rp605_holding_sec
        record.rp612_gross_pnl = rp605_gross
        record.rp612_friction = rp605_friction
        record.rp612_net_pnl = rp605_net
        record.rp605_net_paise = round(rp605_net * 100)
        record.rp612_net_paise = round(rp605_net * 100)
        record.paired_net_delta = 0.0
        record.paired_paise_delta = 0
        
    return record


if __name__ == "__main__":
    print("RP-612 Coralys Temporal Trajectory Evaluator module loaded.")
