#!/usr/bin/env python3
"""
rp608_shadow_engine.py

RP-608 — Dynamic Trade Boundary Management Shadow Engine.
Runs as a parallel, read-only shadow execution policy alongside live RP-605 prospective validation.

Strict Architectural Invariants:
1. RP-605 Control Isolation: ZERO feedback into RP-605 daemons, Coralys parameters, entries, exits, or capital.
2. Recommendation Lifecycle Engine: PROPOSED -> VALIDATED -> APPLIED (or REJECTED / EXPIRED).
3. Monotonic Stop-Loss Enforcement: LONG stops move UP only; SHORT stops move DOWN only.
4. Price Validation: Proposed boundaries crossed by current price are REJECTED immediately.
5. Atomic State Transitions: SL + Target update together in one atomic transaction with version history.
6. Contemporaneous Information Constraint: Recommendations driven STRICTLY by running MFE up to evaluation tick.
7. Update Type Disaggregation: Classifies updates as STOP_LOSS_ONLY, TARGET_ONLY, or JOINT_SL_AND_TARGET.
"""

import os
import sys
import json
import math
from pathlib import Path
from datetime import datetime, timezone, timedelta
from collections import defaultdict

WORKSPACE_DIR = Path(__file__).parent.parent
LOG_DIR = WORKSPACE_DIR / "live_capture" / "shadow_validation"

def parse_iso(ts_str):
    if not ts_str or ts_str == "ACTIVE":
        return None
    try:
        ts_str = ts_str.replace("Z", "+00:00")
        return datetime.fromisoformat(ts_str).astimezone(timezone.utc)
    except Exception:
        return None

class RP608ShadowTrade:
    def __init__(self, trade_id, symbol, side, entry_price, quantity, entry_timestamp, initial_sl, initial_target, fee_bps=5.0, composite_id=None):
        self.trade_id = trade_id
        self.composite_id = composite_id or trade_id
        self.symbol = symbol
        self.side = side
        self.entry_price = float(entry_price)
        self.quantity = float(quantity)
        self.entry_timestamp = entry_timestamp
        self.fee_bps = float(fee_bps)
        
        # Active Boundaries (v0)
        self.active_sl = float(initial_sl)
        self.active_target = float(initial_target)
        self.version = 0
        self.boundary_history = [{
            "version": 0,
            "timestamp": entry_timestamp,
            "stop_loss": self.active_sl,
            "target": self.active_target,
            "reason": "INITIAL_ENTRY_FREEZE",
            "update_type": "INITIAL_FREEZE"
        }]
        
        # Recommendation history
        self.recommendations = []
        
        # State tracking
        self.status = "OPEN" # OPEN, CLOSED
        self.exit_timestamp = None
        self.trigger_price = None
        self.exit_price = None
        self.slippage_cost = 0.0
        self.fee_cost = 0.0
        self.exit_reason = None
        self.realized_pnl = 0.0
        
        # Running MFE / MAE tracking (strictly up to evaluation tick)
        self.mfe_price = self.entry_price
        self.mae_price = self.entry_price
        self.mfe_pnl = 0.0
        self.mae_pnl = 0.0

    def update_excursions(self, current_price):
        if self.side == "LONG":
            if current_price > self.mfe_price:
                self.mfe_price = current_price
            if current_price < self.mae_price:
                self.mae_price = current_price
            self.mfe_pnl = (self.mfe_price - self.entry_price) * self.quantity
            self.mae_pnl = (self.mae_price - self.entry_price) * self.quantity
        else:
            if current_price < self.mfe_price:
                self.mfe_price = current_price
            if current_price > self.mae_price:
                self.mae_price = current_price
            self.mfe_pnl = (self.entry_price - self.mfe_price) * self.quantity
            self.mae_pnl = (self.entry_price - self.mae_price) * self.quantity

    def propose_update(self, proposed_sl, proposed_target, timestamp_utc, rationale, policy_id="RP608_SHADOW_POLICY_V1"):
        sl_changed = abs(float(proposed_sl) - self.active_sl) >= 1e-4
        target_changed = abs(float(proposed_target) - self.active_target) >= 1e-4

        if sl_changed and target_changed:
            update_type = "JOINT_SL_AND_TARGET"
        elif sl_changed:
            update_type = "STOP_LOSS_ONLY"
        elif target_changed:
            update_type = "TARGET_ONLY"
        else:
            update_type = "ZERO_DELTA"

        # Calculate Running MFE % up to current tick
        running_mfe_pct = (self.mfe_price - self.entry_price) / self.entry_price if self.side == "LONG" else (self.entry_price - self.mfe_price) / self.entry_price

        rec = {
            "policy_id": policy_id,
            "trade_id": self.trade_id,
            "composite_id": self.composite_id,
            "symbol": self.symbol,
            "side": self.side,
            "timestamp_utc": timestamp_utc,
            "running_mfe_pct": round(running_mfe_pct, 6),
            "current_active_sl": self.active_sl,
            "current_active_target": self.active_target,
            "proposed_sl": float(proposed_sl),
            "proposed_target": float(proposed_target),
            "update_type": update_type,
            "rationale": rationale,
            "state": "PROPOSED"
        }
        return rec

    def validate_recommendation(self, recommendation, current_price):
        p_sl = recommendation["proposed_sl"]
        p_target = recommendation["proposed_target"]
        
        # 1. Monotonicity Safeguard
        if self.side == "LONG":
            if p_sl < self.active_sl:
                recommendation["state"] = "REJECTED"
                recommendation["rejection_reason"] = f"NON_MONOTONIC_SL: proposed {p_sl:.4f} < active {self.active_sl:.4f}"
                self.recommendations.append(recommendation)
                return False
            if current_price <= p_sl:
                recommendation["state"] = "REJECTED"
                recommendation["rejection_reason"] = f"PRICE_CROSSED_PROPOSED_SL: current {current_price:.4f} <= proposed {p_sl:.4f}"
                self.recommendations.append(recommendation)
                return False
        else: # SHORT
            if p_sl > self.active_sl:
                recommendation["state"] = "REJECTED"
                recommendation["rejection_reason"] = f"NON_MONOTONIC_SL: proposed {p_sl:.4f} > active {self.active_sl:.4f}"
                self.recommendations.append(recommendation)
                return False
            if current_price >= p_sl:
                recommendation["state"] = "REJECTED"
                recommendation["rejection_reason"] = f"PRICE_CROSSED_PROPOSED_SL: current {current_price:.4f} >= proposed {p_sl:.4f}"
                self.recommendations.append(recommendation)
                return False
                
        # Zero-delta check
        if abs(p_sl - self.active_sl) < 1e-4 and abs(p_target - self.active_target) < 1e-4:
            recommendation["state"] = "REJECTED"
            recommendation["rejection_reason"] = "ZERO_DELTA_UPDATE"
            self.recommendations.append(recommendation)
            return False

        recommendation["state"] = "VALIDATED"
        return True

    def apply_update(self, recommendation):
        if recommendation["state"] != "VALIDATED":
            return False
            
        self.version += 1
        self.active_sl = recommendation["proposed_sl"]
        self.active_target = recommendation["proposed_target"]
        
        history_entry = {
            "version": self.version,
            "timestamp": recommendation["timestamp_utc"],
            "stop_loss": self.active_sl,
            "target": self.active_target,
            "update_type": recommendation["update_type"],
            "reason": recommendation["rationale"]
        }
        self.boundary_history.append(history_entry)
        recommendation["state"] = "APPLIED"
        self.recommendations.append(recommendation)
        return True

    def check_hypothetical_exit(self, current_price, timestamp_utc, is_control_engine_exit=False, control_exit_price=None):
        if self.status == "CLOSED":
            return False
            
        self.update_excursions(current_price)
        
        entry_val = self.entry_price * self.quantity
        entry_fee = entry_val * (self.fee_bps / 10000.0)

        sl_hit = False
        target_hit = False
        fill_price = current_price

        if self.side == "LONG":
            if current_price <= self.active_sl:
                sl_hit = True
                fill_price = min(current_price, self.active_sl)
            elif current_price >= self.active_target:
                target_hit = True
                fill_price = max(current_price, self.active_target)
        else: # SHORT
            if current_price >= self.active_sl:
                sl_hit = True
                fill_price = max(current_price, self.active_sl)
            elif current_price <= self.active_target:
                target_hit = True
                fill_price = min(current_price, self.active_target)

        if sl_hit:
            self.status = "CLOSED"
            self.exit_timestamp = timestamp_utc
            self.trigger_price = self.active_sl
            self.exit_price = fill_price
            self.exit_reason = "SHADOW_STOP_LOSS_HIT"
            
            exit_val = fill_price * self.quantity
            exit_fee = exit_val * (self.fee_bps / 10000.0)
            self.fee_cost = entry_fee + exit_fee
            self.slippage_cost = abs(fill_price - self.active_sl) * self.quantity
            
            raw_pnl = (fill_price - self.entry_price) * self.quantity if self.side == "LONG" else (self.entry_price - fill_price) * self.quantity
            self.realized_pnl = raw_pnl - self.fee_cost
            return True

        if target_hit:
            self.status = "CLOSED"
            self.exit_timestamp = timestamp_utc
            self.trigger_price = self.active_target
            self.exit_price = fill_price
            self.exit_reason = "SHADOW_TARGET_HIT"
            
            exit_val = fill_price * self.quantity
            exit_fee = exit_val * (self.fee_bps / 10000.0)
            self.fee_cost = entry_fee + exit_fee
            self.slippage_cost = abs(fill_price - self.active_target) * self.quantity
            
            raw_pnl = (fill_price - self.entry_price) * self.quantity if self.side == "LONG" else (self.entry_price - fill_price) * self.quantity
            self.realized_pnl = raw_pnl - self.fee_cost
            return True

        if is_control_engine_exit:
            exec_p = float(control_exit_price) if control_exit_price is not None else current_price
            self.status = "CLOSED"
            self.exit_timestamp = timestamp_utc
            self.trigger_price = exec_p
            self.exit_price = exec_p
            self.exit_reason = "CONTROL_ENGINE_EXIT_COINCIDENT"
            
            exit_val = exec_p * self.quantity
            exit_fee = exit_val * (self.fee_bps / 10000.0)
            self.fee_cost = entry_fee + exit_fee
            self.slippage_cost = 0.0
            
            raw_pnl = (exec_p - self.entry_price) * self.quantity if self.side == "LONG" else (self.entry_price - exec_p) * self.quantity
            self.realized_pnl = raw_pnl - self.fee_cost
            return True

        return False

class RP608ShadowEngine:
    def __init__(self, policy_id="RP608_SHADOW_POLICY_V1"):
        self.policy_id = policy_id
        self.active_trades = {} # composite_id -> RP608ShadowTrade
        self.closed_trades = []
        self.recommendations = []

    def process_entry_event(self, entry_event, default_sl_pct=0.03, default_target_pct=0.06):
        trade_id = entry_event["trade_id"]
        session = entry_event.get("session", "UNKNOWN")
        sym = entry_event["symbol"]
        side = entry_event["side"]
        entry_p = float(entry_event["entry_price"])
        qty = float(entry_event.get("quantity", 1.0))
        entry_ts = entry_event["entry_timestamp"]

        composite_id = f"{session}_{sym}_{side}_{entry_ts}_{trade_id}"
        
        if composite_id in self.active_trades or any(t.composite_id == composite_id for t in self.closed_trades):
            return None

        if "stop_loss" in entry_event and entry_event["stop_loss"] is not None:
            initial_sl = float(entry_event["stop_loss"])
        else:
            sl_pct = default_sl_pct if "USD" not in sym else 0.05
            initial_sl = entry_p * (1.0 - sl_pct) if side == "LONG" else entry_p * (1.0 + sl_pct)

        if "target" in entry_event and entry_event["target"] is not None:
            initial_target = float(entry_event["target"])
        else:
            tgt_pct = default_target_pct if "USD" not in sym else 0.10
            initial_target = entry_p * (1.0 + tgt_pct) if side == "LONG" else entry_p * (1.0 - tgt_pct)

        trade = RP608ShadowTrade(
            trade_id=trade_id,
            composite_id=composite_id,
            symbol=sym,
            side=side,
            entry_price=entry_p,
            quantity=qty,
            entry_timestamp=entry_ts,
            initial_sl=initial_sl,
            initial_target=initial_target
        )
        self.active_trades[composite_id] = trade
        return trade

    def evaluate_tick(self, timestamp_utc, symbol, current_price, is_control_engine_exit=False, control_exit_price=None):
        current_price = float(current_price)
        closed_this_tick = []

        open_ids = list(self.active_trades.keys())
        for cid in open_ids:
            trade = self.active_trades.get(cid)
            if not trade or trade.symbol != symbol:
                continue

            # 1. Update running excursions & check hypothetical exits
            if trade.check_hypothetical_exit(current_price, timestamp_utc, is_control_engine_exit, control_exit_price):
                self.closed_trades.append(trade)
                del self.active_trades[cid]
                closed_this_tick.append(trade)
                continue

            # 2. Candidate Policy Recommendation Generator driven STRICTLY by Running MFE up to current tick
            if trade.mfe_pnl > 0:
                running_mfe_pct = (trade.mfe_price - trade.entry_price) / trade.entry_price if trade.side == "LONG" else (trade.entry_price - trade.mfe_price) / trade.entry_price
                
                if running_mfe_pct >= 0.008: # Lock in 50% of running MFE when running MFE >= 0.8%
                    if trade.side == "LONG":
                        proposed_sl = trade.entry_price + (trade.mfe_price - trade.entry_price) * 0.5
                        proposed_target = trade.active_target
                    else:
                        proposed_sl = trade.entry_price - (trade.entry_price - trade.mfe_price) * 0.5
                        proposed_target = trade.active_target

                    rec = trade.propose_update(
                        proposed_sl=proposed_sl,
                        proposed_target=proposed_target,
                        timestamp_utc=timestamp_utc,
                        rationale=f"PROFIT_LOCK_50PCT_RUNNING_MFE: RunningMFE={running_mfe_pct*100:.2f}%",
                        policy_id=self.policy_id
                    )
                    
                    if trade.validate_recommendation(rec, current_price):
                        trade.apply_update(rec)
                        
                    self.recommendations.append(rec)

        return closed_this_tick

def main():
    print("[RP-608 Shadow Engine] Engine Module Loaded Cleanly with Running MFE Enforcement & Update Disaggregation.")

if __name__ == "__main__":
    main()
