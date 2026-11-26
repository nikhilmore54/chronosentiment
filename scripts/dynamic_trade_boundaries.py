#!/usr/bin/env python3
"""
RP-608 Dynamic Trade Boundaries (deterministic OHLCV-structure policy).

Computes stop-loss and target from completed-bar history — not fixed ±1%/1.5%.
Uses ATR, swing pivots, and regime/trend efficiency. Same bars → same boundaries.

Governance (chronosentiment-core + RP-608-REGIME-BREAKOUT-001):
  - Arm A frozen baseline retains fixed exits for fair comparison.
  - This module is the experimental dynamic exit policy (entry-time freeze).
  - No randomness; no online weight updates; bounded and validated.
  - Live order authority remains 0.0% in shadow research.

Live updates (PROPOSED→VALIDATED→APPLIED) follow
rp608_dynamic_trade_boundary_management_specification.md and are optional.
"""

from __future__ import annotations

import math
from typing import Any, Dict, List, Optional, Sequence, Tuple

from scripts.market_regime_classifier import (
    adx_indicator,
    calculate_atr_series,
    classify_market_regime,
    kaufman_efficiency_ratio,
)


# Bounded parameter envelope (validated ranges)
ATR_STOP_MULT_MIN, ATR_STOP_MULT_MAX = 0.75, 2.50
ATR_TARGET_MULT_MIN, ATR_TARGET_MULT_MAX = 1.00, 4.00
STOP_BPS_FLOOR, STOP_BPS_CEIL = 40.0, 250.0
TARGET_BPS_FLOOR, TARGET_BPS_CEIL = 60.0, 400.0
MIN_NET_RR = 1.0


def _hlc(bar: Any) -> Optional[Tuple[float, float, float]]:
    try:
        if isinstance(bar, dict):
            h, l, c = float(bar["high"]), float(bar["low"]), float(bar["close"])
        else:
            h, l, c = float(bar.high), float(bar.low), float(bar.close)
        if not all(math.isfinite(x) and x > 0 for x in (h, l, c)):
            return None
        if l > h:
            return None
        return h, l, c
    except Exception:
        return None


def _clamp(x: float, lo: float, hi: float) -> float:
    return max(lo, min(hi, x))


def recent_swing_pivot(
    bars: Sequence[Any],
    side: str,
    lookback: int = 20,
    pivot: int = 2,
) -> Optional[float]:
    """
    Most recent swing low (LONG stop anchor) or swing high (SHORT stop anchor)
    within lookback completed bars. Pivot = local extremum over ±pivot neighbors.
    """
    if bars is None or len(bars) < (2 * pivot + 1):
        return None
    window = list(bars[-(lookback + pivot):]) if len(bars) > lookback + pivot else list(bars)
    extremes: List[Tuple[int, float]] = []
    for i in range(pivot, len(window) - pivot):
        hlc = _hlc(window[i])
        if hlc is None:
            continue
        h, l, _ = hlc
        if side == "LONG":
            ok = all(
                (_hlc(window[i - k]) or (0, l + 1, 0))[1] >= l
                and (_hlc(window[i + k]) or (0, l + 1, 0))[1] >= l
                for k in range(1, pivot + 1)
            )
            if ok:
                extremes.append((i, l))
        else:
            ok = all(
                (_hlc(window[i - k]) or (h - 1, 0, 0))[0] <= h
                and (_hlc(window[i + k]) or (h - 1, 0, 0))[0] <= h
                for k in range(1, pivot + 1)
            )
            if ok:
                extremes.append((i, h))
    if not extremes:
        return None
    return extremes[-1][1]


def trend_scale_from_er_adx(er: Optional[float], adx: Optional[float]) -> Dict[str, float]:
    """
    Stronger trend → wider target multiple, slightly tighter relative stop vs ATR.
    Weak trend → compressed target (often fails Net R:R → stand aside).
    """
    er_v = 0.0 if er is None else float(er)
    adx_v = 0.0 if adx is None else float(adx)
    # stop_mult in ATR units
    if er_v >= 0.40 and adx_v >= 25.0:
        return {"stop_atr_mult": 1.25, "target_atr_mult": 2.50, "label": "STRONG_TREND"}
    if er_v >= 0.25 and adx_v >= 20.0:
        return {"stop_atr_mult": 1.50, "target_atr_mult": 2.00, "label": "DEVELOPING_TREND"}
    return {"stop_atr_mult": 1.00, "target_atr_mult": 1.25, "label": "WEAK_OR_CHOP"}


def compute_dynamic_boundaries(
    bars: Sequence[Any],
    *,
    side: str,
    entry_price: float,
    regime: Optional[str] = None,
    friction_bps: float = 12.0,
) -> Dict[str, Any]:
    """
    Entry-time dynamic stop & target from completed OHLCV pattern features.

    Stop = max(ATR-based, structure swing) distance on the adverse side, clamped.
    Target = ATR-scaled reward in trend direction, clamped; must clear Net R:R >= 1.
    """
    assert side in ("LONG", "SHORT")
    if entry_price is None or entry_price <= 0 or not math.isfinite(entry_price):
        return {"ok": False, "reason": "INVALID_ENTRY_PRICE"}

    atr_series = calculate_atr_series(bars, period=14)
    if atr_series is None or not atr_series:
        return {"ok": False, "reason": "INSUFFICIENT_ATR_HISTORY"}
    atr = float(atr_series[-1])
    if atr <= 0 or not math.isfinite(atr):
        return {"ok": False, "reason": "INVALID_ATR"}

    closes = []
    for b in bars:
        hlc = _hlc(b)
        if hlc is None:
            return {"ok": False, "reason": "CORRUPT_BAR"}
        closes.append(hlc[2])

    er = kaufman_efficiency_ratio(closes, n=16)
    adx_data = adx_indicator(bars, period=14)
    adx = adx_data.get("adx") if adx_data else None
    plus_di = adx_data.get("plus_di") if adx_data else None
    minus_di = adx_data.get("minus_di") if adx_data else None

    # Optional regime override from caller; else classify
    if regime is None:
        regime = classify_market_regime(bars).get("regime")

    scale = trend_scale_from_er_adx(er, adx)
    stop_mult = _clamp(scale["stop_atr_mult"], ATR_STOP_MULT_MIN, ATR_STOP_MULT_MAX)
    target_mult = _clamp(scale["target_atr_mult"], ATR_TARGET_MULT_MIN, ATR_TARGET_MULT_MAX)

    # SIDEWAYS: refuse expansive targets (stand-aside via Net R:R failure often)
    if regime == "SIDEWAYS":
        target_mult = min(target_mult, 1.25)
        stop_mult = min(stop_mult, 1.00)

    atr_stop_dist = atr * stop_mult
    atr_target_dist = atr * target_mult

    swing = recent_swing_pivot(bars, side=side, lookback=20, pivot=2)
    structure_stop = None
    if swing is not None:
        if side == "LONG" and swing < entry_price:
            structure_stop = swing - 0.1 * atr  # small buffer beyond swing
        elif side == "SHORT" and swing > entry_price:
            structure_stop = swing + 0.1 * atr

    if side == "LONG":
        atr_stop_px = entry_price - atr_stop_dist
        stop_px = max(atr_stop_px, structure_stop) if structure_stop is not None else atr_stop_px
        # Prefer the tighter of ATR and structure when structure is closer (less risk)
        if structure_stop is not None:
            stop_px = max(atr_stop_px, structure_stop)  # both below entry; max = tighter
        target_px = entry_price + atr_target_dist
        # Pattern extension: prior swing high as soft target ceiling boost if beyond ATR target
        prior_high = recent_swing_pivot(bars, side="SHORT", lookback=30, pivot=2)
        if prior_high is not None and prior_high > target_px:
            # Blend: allow up to prior high but still within TARGET_BPS_CEIL
            target_px = min(prior_high, entry_price * (1.0 + TARGET_BPS_CEIL / 10_000.0))
    else:
        atr_stop_px = entry_price + atr_stop_dist
        stop_px = min(atr_stop_px, structure_stop) if structure_stop is not None else atr_stop_px
        if structure_stop is not None:
            stop_px = min(atr_stop_px, structure_stop)
        target_px = entry_price - atr_target_dist
        prior_low = recent_swing_pivot(bars, side="LONG", lookback=30, pivot=2)
        if prior_low is not None and prior_low < target_px:
            target_px = max(prior_low, entry_price * (1.0 - TARGET_BPS_CEIL / 10_000.0))

    stop_bps = abs(entry_price - stop_px) / entry_price * 10_000.0
    target_bps = abs(target_px - entry_price) / entry_price * 10_000.0
    stop_bps = _clamp(stop_bps, STOP_BPS_FLOOR, STOP_BPS_CEIL)
    target_bps = _clamp(target_bps, TARGET_BPS_FLOOR, TARGET_BPS_CEIL)

    # Re-apply clamped distances to prices
    if side == "LONG":
        stop_px = entry_price * (1.0 - stop_bps / 10_000.0)
        target_px = entry_price * (1.0 + target_bps / 10_000.0)
    else:
        stop_px = entry_price * (1.0 + stop_bps / 10_000.0)
        target_px = entry_price * (1.0 - target_bps / 10_000.0)

    net_rr = (target_bps - friction_bps) / (stop_bps + friction_bps + 1e-9)
    # ATR/structure bounds stand alone; regime label is observational (UNAVAILABLE must not
    # force Arm A fallback when Net R:R clears — early-session ATR warmup is common).
    ok = net_rr >= MIN_NET_RR and bool(atr > 0)

    return {
        "ok": ok,
        "reason": "OK" if ok else ("INSUFFICIENT_NET_RR" if net_rr < MIN_NET_RR else "INVALID_ATR"),
        "side": side,
        "entry_price": round(entry_price, 6),
        "stop_price": round(stop_px, 6),
        "target_price": round(target_px, 6),
        "stop_bps": round(stop_bps, 4),
        "target_bps": round(target_bps, 4),
        "net_rr": round(net_rr, 4),
        "atr_14": round(atr, 6),
        "er_16": None if er is None else round(er, 4),
        "adx_14": None if adx is None else round(adx, 2),
        "plus_di": plus_di,
        "minus_di": minus_di,
        "regime": regime,
        "trend_label": scale["label"],
        "stop_atr_mult": stop_mult,
        "target_atr_mult": target_mult,
        "structure_pivot": None if swing is None else round(swing, 6),
        "friction_bps": friction_bps,
        "policy_id": "RP608_DYNAMIC_OHLCV_STRUCTURE_V1",
        "frozen_at_entry": True,
    }


def propose_boundary_update(
    *,
    side: str,
    entry_price: float,
    current_stop: float,
    current_target: float,
    bars: Sequence[Any],
    last_price: float,
) -> Dict[str, Any]:
    """
    Optional live recommendation (PROPOSED). Does not apply itself.
    Monotonic stop tightening only; target may extend with structure.
    """
    fresh = compute_dynamic_boundaries(
        bars, side=side, entry_price=entry_price, friction_bps=12.0
    )
    if not fresh.get("ok"):
        return {"state": "REJECTED", "reason": fresh.get("reason"), "proposal": None}

    new_stop = float(fresh["stop_price"])
    new_target = float(fresh["target_price"])

    if side == "LONG":
        # Stop may only rise; never below last price
        if new_stop <= current_stop:
            new_stop = current_stop
        if new_stop >= last_price:
            return {"state": "REJECTED", "reason": "STOP_CROSSED_OR_INVALID", "proposal": None}
        if new_target < current_target:
            new_target = current_target  # do not cut target on weak updates without explicit policy
    else:
        if new_stop >= current_stop:
            new_stop = current_stop
        if new_stop <= last_price:
            return {"state": "REJECTED", "reason": "STOP_CROSSED_OR_INVALID", "proposal": None}
        if new_target > current_target:
            new_target = current_target

    if new_stop == current_stop and new_target == current_target:
        return {"state": "REJECTED", "reason": "NO_MATERIAL_CHANGE", "proposal": None}

    return {
        "state": "PROPOSED",
        "reason": "STRUCTURE_ATR_REFRESH",
        "proposal": {
            "stop_price": round(new_stop, 6),
            "target_price": round(new_target, 6),
            "from_stop": current_stop,
            "from_target": current_target,
            "features": {
                "atr_14": fresh.get("atr_14"),
                "er_16": fresh.get("er_16"),
                "adx_14": fresh.get("adx_14"),
                "trend_label": fresh.get("trend_label"),
            },
        },
    }
