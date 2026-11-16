#!/usr/bin/env python3
"""
RP-608 NSE Gate-3 warmup policies.

Policies:
  same_session          — live legacy: today's completed bars only
  cross_gap_neutral_v1  — prior session + today; overnight return zeroed in traj15

Authority: 0.0%. Does not change OF ±0.30, G1 window, or Estimand A exit formulas.
"""

from __future__ import annotations

import math
import os
from datetime import datetime, timedelta, timezone
from typing import Any, Dict, List, Optional, Sequence, Tuple

IST = timezone(timedelta(hours=5, minutes=30))
HISTORY_READY_N = 5
TRAJ_THRESHOLD = 0.05

POLICY_SAME_SESSION = "same_session"
POLICY_CROSS_GAP_NEUTRAL_V1 = "cross_gap_neutral_v1"
POLICY_ID_CROSS_GAP_NEUTRAL_V1 = "RP608_G3_CROSS_GAP_NEUTRAL_V1"

DEFAULT_G3_WARMUP_POLICY = os.environ.get(
    "RP608_G3_WARMUP_POLICY", POLICY_CROSS_GAP_NEUTRAL_V1
).strip().lower()


def bar_session_date(bar: Any) -> Optional[str]:
    if bar is None:
        return None
    if isinstance(bar, dict):
        if bar.get("session_date"):
            return str(bar["session_date"])
        ts = bar.get("timestamp_utc_ms")
    else:
        sd = getattr(bar, "session_date", None)
        if sd:
            return str(sd)
        ts = getattr(bar, "timestamp_utc_ms", None)
    if ts is None:
        return None
    try:
        return (
            datetime.fromtimestamp(int(ts) / 1000.0, tz=timezone.utc)
            .astimezone(IST)
            .date()
            .isoformat()
        )
    except (TypeError, ValueError, OSError):
        return None


def bar_ohlc(bar: Any) -> Optional[Tuple[float, float, float, float, int]]:
    try:
        if isinstance(bar, dict):
            o = float(bar["open"])
            h = float(bar["high"])
            l = float(bar["low"])
            c = float(bar["close"])
            ts = int(bar.get("timestamp_utc_ms") or 0)
        else:
            o = float(bar.open)
            h = float(bar.high)
            l = float(bar.low)
            c = float(bar.close)
            ts = int(getattr(bar, "timestamp_utc_ms", 0) or 0)
        if c <= 0 or ts <= 0:
            return None
        return o, h, l, c, ts
    except (TypeError, ValueError, KeyError, AttributeError):
        return None


def traj15_from_closes(closes: Sequence[float]) -> float:
    """Exact daemon same-session traj15 formula."""
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
    return float(math.tanh(ret / denom))


def traj15_gap_neutral(bars: Sequence[Any]) -> Tuple[float, Optional[float]]:
    """
    Traj15 with overnight close→open gap removed.

    Within a session: chain close-to-close.
    At session boundary: replace overnight jump with open→close of the first
    bar of the new session.
    """
    parsed: List[Tuple[str, float, float]] = []
    for b in bars:
        ohlc = bar_ohlc(b)
        sd = bar_session_date(b)
        if ohlc is None or not sd:
            continue
        o, _h, _l, c, _ts = ohlc
        parsed.append((sd, o, c))
    if len(parsed) < 2:
        return 0.0, None

    lookback = min(len(parsed), 15)
    window = parsed[-lookback:]
    adj_closes = [window[0][2]]
    overnight_gap_bps: Optional[float] = None
    for i in range(1, len(window)):
        prev_sd, _po, prev_c = window[i - 1]
        cur_sd, cur_o, cur_c = window[i]
        if cur_sd != prev_sd:
            if prev_c > 0:
                overnight_gap_bps = ((cur_o - prev_c) / prev_c) * 1e4
            if cur_o > 0:
                adj_closes.append(adj_closes[-1] * (cur_c / cur_o))
            else:
                adj_closes.append(adj_closes[-1])
        elif prev_c > 0:
            adj_closes.append(adj_closes[-1] * (cur_c / prev_c))
        else:
            adj_closes.append(adj_closes[-1])
    return traj15_from_closes(adj_closes), overnight_gap_bps


def prior_session_bars_only(
    feature_bars: Sequence[Any],
    *,
    today_session_date: str,
) -> List[Any]:
    """Keep bars from the most recent session strictly before today."""
    by_date: Dict[str, List[Any]] = {}
    for b in feature_bars:
        sd = bar_session_date(b)
        if not sd or sd >= today_session_date:
            continue
        by_date.setdefault(sd, []).append(b)
    if not by_date:
        return []
    prior_date = max(by_date.keys())
    bars = by_date[prior_date]
    bars = sorted(
        bars,
        key=lambda b: int(
            (b.get("timestamp_utc_ms") if isinstance(b, dict) else getattr(b, "timestamp_utc_ms", 0))
            or 0
        ),
    )
    return bars


def build_g3_history(
    *,
    policy: str,
    session_bars: Sequence[Any],
    feature_bars: Optional[Sequence[Any]] = None,
    today_session_date: str,
) -> List[Any]:
    session_list = list(session_bars or [])
    pol = (policy or POLICY_SAME_SESSION).strip().lower()
    if pol in ("", POLICY_SAME_SESSION, "a", "policy_a"):
        return session_list
    prior = prior_session_bars_only(feature_bars or [], today_session_date=today_session_date)
    # Dedupe by timestamp; session bars win on collision.
    by_ts: Dict[int, Any] = {}
    for b in list(prior) + session_list:
        ohlc = bar_ohlc(b)
        if ohlc is None:
            continue
        by_ts[ohlc[4]] = b
    return [by_ts[k] for k in sorted(by_ts.keys())]


def evaluate_g3_momentum(
    *,
    policy: str,
    session_bars: Sequence[Any],
    feature_bars: Optional[Sequence[Any]] = None,
    today_session_date: str,
    history_ready_n: int = HISTORY_READY_N,
    traj_threshold: float = TRAJ_THRESHOLD,
) -> Dict[str, Any]:
    """
    Returns G3 readiness + traj15 under the selected warmup policy.

    Session eligibility still requires ≥1 completed today bar.
    """
    pol = (policy or POLICY_SAME_SESSION).strip().lower()
    session_n = len(session_bars or [])
    hist = build_g3_history(
        policy=pol,
        session_bars=session_bars,
        feature_bars=feature_bars,
        today_session_date=today_session_date,
    )
    gap_bps: Optional[float] = None
    if pol in (POLICY_CROSS_GAP_NEUTRAL_V1, "c", "policy_c", "cross_gap_neutral"):
        traj, gap_bps = traj15_gap_neutral(hist) if len(hist) >= 2 else (0.0, None)
        policy_id = POLICY_ID_CROSS_GAP_NEUTRAL_V1
        score_kind = "calculated_gap_neutral" if len(hist) >= 2 else "cold_start_default_zero"
    else:
        closes = []
        for b in hist:
            ohlc = bar_ohlc(b)
            if ohlc:
                closes.append(ohlc[3])
        traj = traj15_from_closes(closes) if len(closes) >= 2 else 0.0
        policy_id = "RP608_G3_SAME_SESSION"
        score_kind = "calculated" if len(closes) >= 2 else "cold_start_default_zero"
        pol = POLICY_SAME_SESSION

    g3_ready = bool(session_n >= 1 and len(hist) >= int(history_ready_n))
    signal_long = bool(g3_ready and traj >= float(traj_threshold))
    return {
        "g3_warmup_policy": pol,
        "g3_warmup_policy_id": policy_id,
        "g3_ready": g3_ready,
        "g3_history_length": len(hist),
        "g3_session_history_length": session_n,
        "traj15": float(traj),
        "overnight_gap_bps": None if gap_bps is None else float(gap_bps),
        "score_kind": score_kind,
        "signal_long": signal_long,
        "history_ready_n": int(history_ready_n),
        "traj_threshold": float(traj_threshold),
    }
