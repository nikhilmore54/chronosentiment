#!/usr/bin/env python3
"""
RP-608-REGIME-BREAKOUT-001 Phase 3: Event-driven Donchian breakout lifecycle.

Deterministic state machine with corrected event order:
  stale -> spread -> CROSSING/QUALIFY -> else WATCH -> retreat/expiry.

Live order authority: 0.0%. Pure functions only.
"""

from __future__ import annotations

import hashlib
from dataclasses import dataclass
from typing import Any, Dict, List, Optional, Sequence, Tuple

TERMINAL = frozenset({"TRADE_QUALIFIED", "INVALIDATED", "EXPIRED"})
ACTIVE = frozenset({"WATCH", "BREAKOUT_DETECTED"})


def donchian_20(bars: Sequence[Any], exclude_forming: bool = True) -> Optional[Tuple[float, float]]:
    """
    Donchian high/low from completed bars t-20..t-1.
    If exclude_forming, uses bars[:-1][-20:] (drops last/forming).
    """
    if bars is None:
        return None
    hist = list(bars[:-1]) if exclude_forming and len(bars) >= 1 else list(bars)
    window = hist[-20:]
    if len(window) < 20:
        return None
    highs, lows = [], []
    for b in window:
        h = b["high"] if isinstance(b, dict) else getattr(b, "high", None)
        l = b["low"] if isinstance(b, dict) else getattr(b, "low", None)
        if h is None or l is None:
            return None
        highs.append(float(h))
        lows.append(float(l))
    return max(highs), min(lows)


def pack_bars_for_breakout_eval(
    completed_including_signal: Sequence[Any],
) -> List[Any]:
    """
    Canonical packing for quote-vs-Donchian evaluation.

    Input: rolling completed bars whose last element is the signal bar t.
    Output: prior completed (…, t-1) + [signal as forming/evaluation candle].

    donchian_20(..., exclude_forming=True) then uses t-20..t-1 only — the signal
    bar must NOT inflate the boundary (Gate 4 packing defect).
    """
    bars = list(completed_including_signal)
    if not bars:
        return []
    return bars[:-1] + [bars[-1]]


def half_spread_bps(ask: float, bid: float) -> Optional[float]:
    if ask <= 0 or bid <= 0 or ask <= bid:
        return None
    mid = (ask + bid) / 2.0
    return ((ask - bid) / (2.0 * mid)) * 10_000.0


def delta_buffer(mid: float, ask: float, bid: float) -> float:
    full_spread = ask - bid
    bps_component = mid * (2.0 / 10_000.0)
    return max(bps_component, 0.5 * full_spread)


def dedupe_key(session_date: str, symbol: str, boundary_price: float, direction: str) -> str:
    raw = f"{session_date}|{symbol}|{boundary_price:.8f}|{direction}"
    return hashlib.sha256(raw.encode("utf-8")).hexdigest()


def net_reward_risk(
    target_distance: float,
    stop_distance: float,
    friction: float,
) -> float:
    denom = stop_distance + friction
    if denom <= 0:
        return 0.0
    return (target_distance - friction) / denom


@dataclass
class LifecycleState:
    state: str = "IDLE"
    direction: Optional[str] = None
    boundary_price: Optional[float] = None
    reason: Optional[str] = None
    dedupe: Optional[str] = None
    net_rr: Optional[float] = None


def evaluate_quote(
    *,
    state: LifecycleState,
    regime: str,
    bars_completed_plus_forming: Sequence[Any],
    ask: float,
    bid: float,
    quote_age_ms: float,
    session_date: str,
    symbol: str,
    half_spread_cap_bps: float,
    target_distance: float,
    stop_distance: float,
    friction: float,
    session_expired: bool = False,
    approach_pct: float = 0.0025,
) -> LifecycleState:
    """
    One deterministic evaluation step. Crossing is evaluated before WATCH.
    """
    out = LifecycleState(
        state=state.state,
        direction=state.direction,
        boundary_price=state.boundary_price,
        reason=state.reason,
        dedupe=state.dedupe,
        net_rr=state.net_rr,
    )
    if out.state in TERMINAL:
        return out

    if session_expired:
        if out.state in ACTIVE or out.state == "IDLE":
            if out.state in ACTIVE:
                out.state = "EXPIRED"
                out.reason = "session_or_bar_expiry"
        return out

    # 1. Stale
    if quote_age_ms > 500:
        if out.state in ACTIVE:
            out.state = "INVALIDATED"
            out.reason = "stale"
        return out

    hs = half_spread_bps(ask, bid)
    if hs is None:
        if out.state in ACTIVE:
            out.state = "INVALIDATED"
            out.reason = "invalid_book"
        return out

    # 2. Spread
    if hs > half_spread_cap_bps:
        if out.state in ACTIVE:
            out.state = "INVALIDATED"
            out.reason = "spread"
        return out

    bounds = donchian_20(bars_completed_plus_forming, exclude_forming=True)
    if bounds is None:
        return out
    upper, lower = bounds
    mid = (ask + bid) / 2.0
    buf = delta_buffer(mid, ask, bid)

    long_cross = ask > upper + buf
    short_cross = bid < lower - buf

    # 3. Crossing before WATCH
    if long_cross or short_cross:
        direction = "LONG" if long_cross else "SHORT"
        boundary = upper if long_cross else lower
        out.state = "BREAKOUT_DETECTED"
        out.direction = direction
        out.boundary_price = boundary
        out.dedupe = dedupe_key(session_date, symbol, boundary, direction)
        rr = net_reward_risk(target_distance, stop_distance, friction)
        out.net_rr = round(rr, 4)
        if rr >= 1.0:
            out.state = "TRADE_QUALIFIED"
            out.reason = "net_rr_ok"
        else:
            out.state = "INVALIDATED"
            out.reason = "insufficient_rr"
        return out

    # 4. WATCH approach
    approach_long = abs(ask - upper) / upper <= approach_pct if upper > 0 else False
    approach_short = abs(bid - lower) / lower <= approach_pct if lower > 0 else False
    if (approach_long or approach_short) and regime in ("TRANSITIONING", "DIRECTIONAL"):
        out.state = "WATCH"
        out.direction = "LONG" if approach_long else "SHORT"
        out.boundary_price = upper if approach_long else lower
        out.reason = "approach"
        out.dedupe = dedupe_key(session_date, symbol, out.boundary_price, out.direction)
        return out

    # 5. Retreat for active states
    if out.state in ACTIVE:
        out.state = "INVALIDATED"
        out.reason = "retreat"
    return out
