#!/usr/bin/env python3
"""
RP-608-REGIME-BREAKOUT-001 — Frozen Arm A baseline helpers.

Arm A entry/exit/horizon/cost logic is FROZEN. This module exposes pure
snapshot builders used by golden fixtures and enabled-vs-disabled invariance
tests. Regime telemetry fields are observational and must be stripped before
comparing Arm A behavioral equality.

Live order authority: 0.0%.
"""

from __future__ import annotations

import hashlib
import json
import math
from dataclasses import dataclass
from typing import Any, Dict, List, Optional, Sequence, Tuple

# ─── Freeze markers (do not alter numerical constants without new protocol) ───
ARM_A_FROZEN = True
ARM_A_FREEZE_PROTOCOL = "RP-608-REGIME-BREAKOUT-001"
ARM_A_LIVE_ORDER_AUTHORITY = 0.0

# NSE Arm A (mirrors run_rp608_nse_shadow_daemon.py)
NSE_TRAJ15_ENTRY_THRESHOLD = 0.05
NSE_MIN_HISTORY_BARS = 5
NSE_ENTRY_BAR_MIN = 1
NSE_ENTRY_BAR_MAX = 20
NSE_STOP_MULT = 0.99          # -1.0%
NSE_TARGET_MULT = 1.015       # +1.5%
NSE_HORIZON_TARGET_BARS = 4   # 60m
NSE_HORIZON_MAX_BARS = 12     # 180m
NSE_SQUARE_OFF_BAR = 24
NSE_DEFAULT_HALF_SPREAD_BPS = 2.5
NSE_SLIPPAGE_BPS_PER_LEG = 2.0
NSE_NOTIONAL_INR = 100_000.0

# Crypto Arm A Policy B structural constants (fees)
CRYPTO_TAKER_FEE_RATE = 0.0005
CRYPTO_SLIPPAGE_RATE = 0.0001
CRYPTO_DEFAULT_HALF_SPREAD_BPS = 2.5

TELEMETRY_ONLY_KEYS = frozenset({
    "regime_state",
    "directional_efficiency_er",
    "relative_volatility",
    "adx_14",
    "regime_reason",
    "plus_di",
    "minus_di",
    "breakout_state",
    "breakout_boundary_price",
    "net_reward_risk_ratio",
    "regime_telemetry_enabled",
    "market_chop",
    "session_breadth",
    "market_return_bps",
    "market_dispersion_bps",
    "directional_er_hurdle",
})


def strip_telemetry_keys(obj: Any) -> Any:
    """Recursively drop observational telemetry keys for Arm A equality checks."""
    if isinstance(obj, dict):
        return {
            k: strip_telemetry_keys(v)
            for k, v in obj.items()
            if k not in TELEMETRY_ONLY_KEYS
        }
    if isinstance(obj, list):
        return [strip_telemetry_keys(x) for x in obj]
    return obj


def canonical_json(obj: Any) -> str:
    return json.dumps(strip_telemetry_keys(obj), sort_keys=True, separators=(",", ":"))


def fingerprint(obj: Any) -> str:
    return hashlib.sha256(canonical_json(obj).encode("utf-8")).hexdigest()


def nse_traj15(closes: Sequence[float]) -> float:
    if len(closes) < 2:
        return 0.0
    lookback = min(len(closes), 15)
    ret = (closes[-1] - closes[-lookback]) / (closes[-lookback] + 1e-9)
    diffs = [
        (closes[i] - closes[i - 1]) / (closes[i - 1] + 1e-9)
        for i in range(-lookback + 1, 0)
    ]
    vol = (sum(d * d for d in diffs) / len(diffs)) ** 0.5 if diffs else 1e-3
    denom = (vol * (lookback ** 0.5) * 2.5) + 1e-9
    return math.tanh(ret / denom)


def nse_calc_friction(
    buy_px: float,
    sell_px: float,
    qty: float,
    half_spread_bps: float = NSE_DEFAULT_HALF_SPREAD_BPS,
) -> Dict[str, float]:
    """Frozen NSE statutory + spread/slippage friction (INR)."""
    buy_turnover = buy_px * qty
    sell_turnover = sell_px * qty
    total_turnover = buy_turnover + sell_turnover

    brokerage_buy = min(20.0, buy_turnover * 0.0003)
    brokerage_sell = min(20.0, sell_turnover * 0.0003)
    total_brokerage = round(brokerage_buy + brokerage_sell, 2)
    stt = round(sell_turnover * 0.00025, 2)
    exchange_charges = round(total_turnover * 0.0000297, 2)
    sebi_charges = round(total_turnover * 0.000001, 2)
    stamp_duty = round(buy_turnover * 0.00003, 2)
    gst_base = total_brokerage + exchange_charges + sebi_charges
    gst = round(gst_base * 0.18, 2)
    statutory = round(
        total_brokerage + stt + exchange_charges + sebi_charges + stamp_duty + gst, 2
    )
    # Spread crossing + slippage (both legs) in INR
    mid_entry = buy_px
    spread_inr = (half_spread_bps / 10_000.0) * mid_entry * qty * 2.0
    slip_inr = (NSE_SLIPPAGE_BPS_PER_LEG / 10_000.0) * (buy_px + sell_px) * qty
    total = round(statutory + spread_inr + slip_inr, 2)
    return {
        "brokerage": total_brokerage,
        "stt": stt,
        "exchange_charges": exchange_charges,
        "sebi_charges": sebi_charges,
        "stamp_duty": stamp_duty,
        "gst": gst,
        "spread_crossing_inr": round(spread_inr, 2),
        "slippage_inr": round(slip_inr, 2),
        "total_friction_inr": total,
    }


@dataclass
class NseBar:
    bar_idx: int
    open: float
    high: float
    low: float
    close: float


def nse_arm_a_decide_entry(
    closes: Sequence[float],
    bar_idx: int,
    has_open_position: bool = False,
) -> Dict[str, Any]:
    """Frozen NSE Arm A entry decision (no side effects)."""
    traj = nse_traj15(closes) if len(closes) >= 2 else 0.0
    if has_open_position:
        return {
            "decision": "NO_TRADE",
            "decision_code": "EXISTING_POSITION_ACTIVE",
            "traj15": round(traj, 6),
            "signal_entry": False,
        }
    if bar_idx >= NSE_SQUARE_OFF_BAR:
        return {
            "decision": "NO_TRADE",
            "decision_code": "INTRADAY_BOUNDARY_EXCLUDED",
            "traj15": round(traj, 6),
            "signal_entry": False,
        }
    if bar_idx > NSE_ENTRY_BAR_MAX:
        return {
            "decision": "NO_TRADE",
            "decision_code": "INTRADAY_BOUNDARY_EXCLUDED",
            "traj15": round(traj, 6),
            "signal_entry": False,
        }
    if len(closes) < NSE_MIN_HISTORY_BARS:
        return {
            "decision": "NO_TRADE",
            "decision_code": "COLD_START_INSUFFICIENT_HISTORY",
            "traj15": round(traj, 6),
            "signal_entry": False,
        }
    if traj < NSE_TRAJ15_ENTRY_THRESHOLD:
        return {
            "decision": "NO_TRADE",
            "decision_code": "SUB_THRESHOLD_MOMENTUM",
            "traj15": round(traj, 6),
            "signal_entry": False,
        }
    if not (NSE_ENTRY_BAR_MIN <= bar_idx <= NSE_ENTRY_BAR_MAX):
        return {
            "decision": "NO_TRADE",
            "decision_code": "INTRADAY_BOUNDARY_EXCLUDED",
            "traj15": round(traj, 6),
            "signal_entry": False,
        }
    return {
        "decision": "ENTER_LONG",
        "decision_code": "QUALIFIED_MOMENTUM_BREAKOUT",
        "traj15": round(traj, 6),
        "signal_entry": True,
        "side": "LONG",
    }


def nse_arm_a_simulate_trade(
    entry_bar_idx: int,
    entry_price: float,
    forward_bars: Sequence[NseBar],
    half_spread_bps: float = NSE_DEFAULT_HALF_SPREAD_BPS,
) -> Dict[str, Any]:
    """
    Frozen NSE exit path: stop -1%, target +1.5%, 60m if green, max 180m, square-off bar 24.
    """
    qty = max(1.0, math.floor(NSE_NOTIONAL_INR / entry_price))
    mfe = entry_price
    mae = entry_price
    exit_price = entry_price
    exit_bar = entry_bar_idx
    exit_reason = "NO_FORWARD_BARS"

    for bar in forward_bars:
        p = bar.close
        mfe = max(mfe, bar.high)
        mae = min(mae, bar.low)
        bars_held = bar.bar_idx - entry_bar_idx
        sl = entry_price * NSE_STOP_MULT
        tp = entry_price * NSE_TARGET_MULT
        if bar.bar_idx >= NSE_SQUARE_OFF_BAR:
            exit_price, exit_bar, exit_reason = p, bar.bar_idx, "MANDATORY_INTRADAY_SQUARE_OFF"
            break
        if p <= sl or bar.low <= sl:
            exit_price, exit_bar, exit_reason = min(p, sl), bar.bar_idx, "STOP_LOSS"
            break
        if p >= tp or bar.high >= tp:
            exit_price, exit_bar, exit_reason = max(p, tp), bar.bar_idx, "TAKE_PROFIT"
            break
        if bars_held >= NSE_HORIZON_MAX_BARS:
            exit_price, exit_bar, exit_reason = p, bar.bar_idx, "MAX_HOLD_HORIZON_180M"
            break
        if bars_held >= NSE_HORIZON_TARGET_BARS and p > entry_price:
            exit_price, exit_bar, exit_reason = p, bar.bar_idx, "TARGET_HORIZON_60M"
            break
        exit_price, exit_bar, exit_reason = p, bar.bar_idx, "PATH_END"

    holding_bars = max(0, exit_bar - entry_bar_idx)
    gross_pnl = (exit_price - entry_price) * qty
    friction = nse_calc_friction(entry_price, exit_price, qty, half_spread_bps)
    net_pnl = round(gross_pnl - friction["total_friction_inr"], 2)
    gross_bps = ((exit_price - entry_price) / entry_price) * 10_000.0
    net_bps = (net_pnl / (entry_price * qty)) * 10_000.0 if entry_price * qty else 0.0

    return {
        "market": "NSE",
        "side": "LONG",
        "entry_bar_idx": entry_bar_idx,
        "entry_price": round(entry_price, 4),
        "exit_bar_idx": exit_bar,
        "exit_price": round(exit_price, 4),
        "exit_reason": exit_reason,
        "holding_bars": holding_bars,
        "quantity": qty,
        "mfe_price": round(mfe, 4),
        "mae_price": round(mae, 4),
        "gross_pnl_inr": round(gross_pnl, 2),
        "net_pnl_inr": net_pnl,
        "gross_return_bps": round(gross_bps, 4),
        "net_return_bps": round(net_bps, 4),
        "friction": friction,
        "arm_a_frozen": ARM_A_FROZEN,
        "live_order_authority": ARM_A_LIVE_ORDER_AUTHORITY,
    }


def crypto_arm_a_decide(
    action_policy_b: str,
    valid_execution_quote: bool,
) -> Dict[str, Any]:
    """Frozen crypto Arm A: Policy B action is authoritative baseline."""
    if action_policy_b not in ("LONG", "SHORT"):
        return {
            "decision": "NO_TRADE",
            "decision_code": "POLICY_B_NO_TRADE",
            "policy_b_action": action_policy_b,
            "signal_entry": False,
        }
    if not valid_execution_quote:
        return {
            "decision": "NO_TRADE",
            "decision_code": "UNEXECUTABLE_MICROSTRUCTURE_DROP",
            "policy_b_action": action_policy_b,
            "signal_entry": True,
            "taken": False,
        }
    return {
        "decision": f"ENTER_{action_policy_b}",
        "decision_code": "POLICY_B_QUALIFIED",
        "policy_b_action": action_policy_b,
        "signal_entry": True,
        "taken": True,
        "side": action_policy_b,
    }


def crypto_arm_a_simulate_trade(
    side: str,
    entry_price: float,
    forward_closes: Sequence[float],
    stop_bps: float = 100.0,
    target_bps: float = 150.0,
    max_hold_bars: int = 8,
    half_spread_bps: float = CRYPTO_DEFAULT_HALF_SPREAD_BPS,
) -> Dict[str, Any]:
    """Frozen crypto exit path used for golden fixtures (shadow research model)."""
    assert side in ("LONG", "SHORT")
    direction = 1.0 if side == "LONG" else -1.0
    # Executable entry: long pays ask, short hits bid — approximate via half-spread
    entry_fill = entry_price * (1.0 + direction * half_spread_bps / 10_000.0)
    stop = entry_fill * (1.0 - direction * stop_bps / 10_000.0)
    target = entry_fill * (1.0 + direction * target_bps / 10_000.0)

    exit_price = entry_fill
    exit_bar = 0
    exit_reason = "NO_FORWARD_BARS"
    mfe = 0.0
    mae = 0.0

    for i, px in enumerate(forward_closes, start=1):
        signed = direction * (px - entry_fill) / entry_fill
        mfe = max(mfe, signed)
        mae = min(mae, signed)
        if (side == "LONG" and px <= stop) or (side == "SHORT" and px >= stop):
            exit_price, exit_bar, exit_reason = stop, i, "STOP_LOSS"
            break
        if (side == "LONG" and px >= target) or (side == "SHORT" and px <= target):
            exit_price, exit_bar, exit_reason = target, i, "TAKE_PROFIT"
            break
        if i >= max_hold_bars:
            exit_price, exit_bar, exit_reason = px, i, "MAX_HOLD_HORIZON"
            break
        exit_price, exit_bar, exit_reason = px, i, "PATH_END"

    # Exit fill crosses spread opposite way
    exit_fill = exit_price * (1.0 - direction * half_spread_bps / 10_000.0)
    gross_ret = direction * (exit_fill - entry_fill) / entry_fill
    fee = CRYPTO_TAKER_FEE_RATE * 2.0
    slip = CRYPTO_SLIPPAGE_RATE * 2.0
    net_ret = gross_ret - fee - slip
    notional = 100_000.0
    gross_pnl = gross_ret * notional
    net_pnl = net_ret * notional
    friction = {
        "taker_fees_usdt": round(fee * notional, 4),
        "slippage_usdt": round(slip * notional, 4),
        "half_spread_bps": half_spread_bps,
        "total_friction_usdt": round((fee + slip) * notional, 4),
    }
    return {
        "market": "CRYPTO_BTCUSDT",
        "side": side,
        "entry_price": round(entry_fill, 4),
        "exit_price": round(exit_fill, 4),
        "exit_reason": exit_reason,
        "holding_bars": exit_bar,
        "gross_pnl_usdt": round(gross_pnl, 4),
        "net_pnl_usdt": round(net_pnl, 4),
        "gross_return_bps": round(gross_ret * 10_000.0, 4),
        "net_return_bps": round(net_ret * 10_000.0, 4),
        "mfe_frac": round(mfe, 6),
        "mae_frac": round(mae, 6),
        "friction": friction,
        "arm_a_frozen": ARM_A_FROZEN,
        "live_order_authority": ARM_A_LIVE_ORDER_AUTHORITY,
    }


def attach_regime_telemetry(
    card: Dict[str, Any],
    regime: Optional[Dict[str, Any]],
    enabled: bool,
) -> Dict[str, Any]:
    """
    Observational only: copies card and optionally attaches regime fields.
    Must not alter decision/entry/exit/cost keys.
    """
    out = dict(card)
    out["regime_telemetry_enabled"] = bool(enabled)
    if not enabled or regime is None:
        return out
    out["regime_state"] = regime.get("regime")
    out["directional_efficiency_er"] = regime.get("er")
    out["relative_volatility"] = regime.get("vol_ratio")
    out["adx_14"] = regime.get("adx")
    out["regime_reason"] = regime.get("reason")
    out["plus_di"] = regime.get("plus_di")
    out["minus_di"] = regime.get("minus_di")
    return out
