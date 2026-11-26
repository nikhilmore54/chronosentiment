#!/usr/bin/env python3
"""
RP-611 Volume Absorption — frozen signal primitives (Phase-1).

Read-only math. No order authority. Thresholds locked at registration.
Does not import or mutate RP-608 / OF / Gate5 behavioural modules.
"""

from __future__ import annotations

from dataclasses import dataclass
from typing import Any, Dict, List, Optional, Tuple

RVOL_MIN = 1.5
BODY_EFF_MAX = 0.35
WICK_MIN = 0.40
SCHEMA_VERSION = "1.0.0"
LIVE_ORDER_AUTHORITY = 0.0


@dataclass(frozen=True)
class Candle:
    open: float
    high: float
    low: float
    close: float
    volume: float
    open_time_ms: int
    is_completed: bool = True
    source: str = ""


def candle_metrics(c: Candle) -> Optional[Dict[str, float]]:
    rng = float(c.high) - float(c.low)
    if rng <= 0:
        return None
    body = abs(float(c.close) - float(c.open))
    upper = (float(c.high) - max(float(c.open), float(c.close))) / rng
    lower = (min(float(c.open), float(c.close)) - float(c.low)) / rng
    return {
        "range": rng,
        "body": body,
        "body_efficiency": body / rng,
        "upper_wick": upper,
        "lower_wick": lower,
        "midpoint": (float(c.high) + float(c.low)) / 2.0,
        "is_red": float(c.close) < float(c.open),
        "is_green": float(c.close) > float(c.open),
    }


def rvol(current_volume: float, prior_volumes: List[float]) -> Optional[float]:
    """RVOL vs mean of previous completed candles (excludes current)."""
    if len(prior_volumes) < 20:
        return None
    window = prior_volumes[-20:]
    mean_v = sum(window) / 20.0
    if mean_v <= 0:
        return None
    return float(current_volume) / mean_v


def classify_absorption(
    c: Candle, prior_volumes: List[float]
) -> Tuple[Optional[str], Dict[str, Any]]:
    """
    Returns (class, detail) where class in {ACCUMULATION_LONG, DISTRIBUTION_SHORT, None}.
    """
    m = candle_metrics(c)
    detail: Dict[str, Any] = {"valid_geometry": m is not None}
    if m is None:
        detail["reject_reason"] = "ZERO_OR_INVALID_RANGE"
        return None, detail
    rv = rvol(float(c.volume), prior_volumes)
    detail.update(m)
    detail["rvol"] = rv
    detail["volume"] = float(c.volume)
    detail["prior_20_mean_volume"] = (
        (sum(prior_volumes[-20:]) / 20.0) if len(prior_volumes) >= 20 else None
    )
    if rv is None:
        detail["reject_reason"] = "INSUFFICIENT_VOLUME_HISTORY"
        return None, detail

    if (
        m["is_red"]
        and rv >= RVOL_MIN
        and m["body_efficiency"] <= BODY_EFF_MAX
        and m["lower_wick"] >= WICK_MIN
    ):
        detail["classification"] = "ACCUMULATION_LONG"
        return "ACCUMULATION_LONG", detail

    if (
        m["is_green"]
        and rv >= RVOL_MIN
        and m["body_efficiency"] <= BODY_EFF_MAX
        and m["upper_wick"] >= WICK_MIN
    ):
        detail["classification"] = "DISTRIBUTION_SHORT"
        return "DISTRIBUTION_SHORT", detail

    detail["reject_reason"] = "NOT_ABSORPTION"
    detail["classification"] = "NEITHER"
    return None, detail


def control_flags(c: Candle, prior_volumes: List[float]) -> Dict[str, bool]:
    """Cohort membership flags (A/B/C geometry side; confirmation applied later)."""
    m = candle_metrics(c)
    rv = rvol(float(c.volume), prior_volumes)
    if m is None:
        return {"A_colour": False, "B_rvol_only": False, "C_absorption": False}
    colour = bool(m["is_red"] or m["is_green"])
    b = bool(rv is not None and rv >= RVOL_MIN)
    cls, _ = classify_absorption(c, prior_volumes)
    return {
        "A_colour": colour,
        "B_rvol_only": b,
        "C_absorption": cls is not None,
    }


def confirm_absorption(
    direction: str, absorption: Candle, next_candle: Candle
) -> Dict[str, Any]:
    mid = (float(absorption.high) + float(absorption.low)) / 2.0
    nxt = float(next_candle.close)
    if direction == "ACCUMULATION_LONG":
        passed = nxt > mid
    else:
        passed = nxt < mid
    return {
        "midpoint": mid,
        "next_close": nxt,
        "passed": passed,
        "reject_reason": None if passed else "CONFIRMATION_FAILED",
    }


def candle_from_nse_bar(bar: Dict[str, Any]) -> Optional[Candle]:
    try:
        if bar.get("is_completed") is False:
            return None
        return Candle(
            open=float(bar["open"]),
            high=float(bar["high"]),
            low=float(bar["low"]),
            close=float(bar["close"]),
            volume=float(bar.get("volume") or 0.0),
            open_time_ms=int(bar["timestamp_utc_ms"]),
            is_completed=True,
            source=str(bar.get("source") or "nse_bar_history"),
        )
    except Exception:
        return None


def candle_from_binance_kline(row: List[Any]) -> Candle:
    # [open_time, open, high, low, close, volume, close_time, ...]
    return Candle(
        open=float(row[1]),
        high=float(row[2]),
        low=float(row[3]),
        close=float(row[4]),
        volume=float(row[5]),
        open_time_ms=int(row[0]),
        is_completed=True,
        source="binance_spot_kline_15m",
    )
