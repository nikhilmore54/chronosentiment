#!/usr/bin/env python3
"""
scripts/market_regime_classifier.py

Pure, deterministic market regime classifier for RP-608.
Supports both Binance Crypto (15m BTC/USDT) and NSE Cash Equities (15m 119 scrips).

Implements Stage 1 of RP-608-REGIME-BREAKOUT-001:
  1. Directional Efficiency: Kaufman Efficiency Ratio (ER_16)
  2. Relative Volatility: ATR_14 vs Trailing 48-bar Median (VolRatio_48)
  3. Directional Trend Strength: Wilder's ADX_14

Deterministic 4-Step Precedence:
  1. UNAVAILABLE: Insufficient history or corrupt data. Zero fabrication.
  2. SIDEWAYS: ER < 0.25 OR ADX < 20.0 OR VolRatio < 0.80.
  3. DIRECTIONAL: ER >= 0.40 AND ADX >= 25.0 AND 0.80 <= VolRatio <= 1.80.
  4. TRANSITIONING: All remaining valid parameter combinations.
"""

from __future__ import annotations

import math
from typing import Any, Dict, List, Optional, Sequence, Union


def kaufman_efficiency_ratio(
    closes: Sequence[float],
    n: int = 16
) -> Optional[float]:
    """
    Computes Kaufman Efficiency Ratio over lookback n.
    ER = |C_t - C_{t-n}| / Sum(|C_{t-i} - C_{t-i-1}| for i in 0..n-1).

    Returns:
        float in [0.0, 1.0], or None if len(closes) < n + 1.
    """
    if closes is None or len(closes) < n + 1 or n < 1:
        return None

    for c in closes[-n - 1:]:
        if c is None or not math.isfinite(c) or c <= 0:
            return None

    net_displacement = abs(closes[-1] - closes[-1 - n])
    path_length = sum(
        abs(closes[-1 - i] - closes[-2 - i])
        for i in range(n)
    )

    if path_length == 0.0:
        return 0.0

    er = net_displacement / path_length
    return round(min(1.0, max(0.0, er)), 4)


def _extract_hlc(bar: Any) -> Optional[tuple[float, float, float]]:
    """Extracts (high, low, close) from dataclass, dict, or object."""
    try:
        if isinstance(bar, dict):
            h = float(bar.get("high", 0.0))
            l = float(bar.get("low", 0.0))
            c = float(bar.get("close", 0.0))
        else:
            h = float(getattr(bar, "high", 0.0))
            l = float(getattr(bar, "low", 0.0))
            c = float(getattr(bar, "close", 0.0))
        if not (math.isfinite(h) and math.isfinite(l) and math.isfinite(c)):
            return None
        if h <= 0 or l <= 0 or c <= 0 or l > h:
            return None
        return h, l, c
    except Exception:
        return None


def calculate_atr_series(
    bars: Sequence[Any],
    period: int = 14
) -> Optional[List[float]]:
    """
    Computes Wilder's ATR series across historical bars.
    Returns list of ATR values aligned with bars (first period-1 values are None).
    """
    if bars is None or len(bars) < period + 1 or period < 1:
        return None

    hlc_list = [_extract_hlc(b) for b in bars]
    if any(x is None for x in hlc_list):
        return None

    # Compute True Range
    tr_list: List[float] = [hlc_list[0][0] - hlc_list[0][1]]  # First bar H - L
    for i in range(1, len(hlc_list)):
        curr_h, curr_l, _ = hlc_list[i]
        _, _, prev_c = hlc_list[i - 1]
        tr = max(
            curr_h - curr_l,
            abs(curr_h - prev_c),
            abs(curr_l - prev_c)
        )
        tr_list.append(tr)

    # Wilder's Smoothing for ATR
    atr_series: List[Optional[float]] = [None] * len(bars)
    initial_atr = sum(tr_list[1:period + 1]) / period
    atr_series[period] = initial_atr

    running_atr = initial_atr
    for i in range(period + 1, len(bars)):
        running_atr = (running_atr * (period - 1) + tr_list[i]) / period
        atr_series[i] = running_atr

    return [float(x) for x in atr_series if x is not None]


def relative_volatility_ratio(
    bars: Sequence[Any],
    n_trailing: int = 48,
    atr_period: int = 14
) -> Optional[float]:
    """
    Computes VolRatio = ATR_14(t) / Median(ATR_14(t-k) for k in 0..n_trailing-1).
    Requires at least (atr_period + n_trailing) valid bars.
    """
    required_bars = atr_period + n_trailing
    if bars is None or len(bars) < required_bars:
        return None

    atr_vals = calculate_atr_series(bars, period=atr_period)
    if atr_vals is None or len(atr_vals) < n_trailing:
        return None

    window = atr_vals[-n_trailing:]
    current_atr = window[-1]
    sorted_window = sorted(window)
    mid = len(sorted_window) // 2
    if len(sorted_window) % 2 == 0:
        median_atr = (sorted_window[mid - 1] + sorted_window[mid]) / 2.0
    else:
        median_atr = sorted_window[mid]

    if median_atr <= 0.0:
        return None

    ratio = current_atr / median_atr
    return round(ratio, 4)


def adx_indicator(
    bars: Sequence[Any],
    period: int = 14
) -> Optional[Dict[str, float]]:
    """
    Computes standard Wilder's ADX over the specified period.
    Requires at least (2 * period + 1) bars for stable initialization.
    """
    required_bars = 2 * period + 1
    if bars is None or len(bars) < required_bars or period < 2:
        return None

    hlc_list = [_extract_hlc(b) for b in bars]
    if any(x is None for x in hlc_list):
        return None

    tr_list: List[float] = []
    plus_dm_list: List[float] = []
    minus_dm_list: List[float] = []

    for i in range(1, len(hlc_list)):
        curr_h, curr_l, _ = hlc_list[i]
        prev_h, prev_l, prev_c = hlc_list[i - 1]

        # TR
        tr = max(curr_h - curr_l, abs(curr_h - prev_c), abs(curr_l - prev_c))
        tr_list.append(tr)

        # Directional Movement
        up_move = curr_h - prev_h
        down_move = prev_l - curr_l

        if up_move > down_move and up_move > 0:
            plus_dm_list.append(up_move)
        else:
            plus_dm_list.append(0.0)

        if down_move > up_move and down_move > 0:
            minus_dm_list.append(down_move)
        else:
            minus_dm_list.append(0.0)

    # Initial Wilder sums
    sm_tr = sum(tr_list[:period])
    sm_pdm = sum(plus_dm_list[:period])
    sm_mdm = sum(minus_dm_list[:period])

    dx_list: List[float] = []

    def _calc_dx(p_dm: float, m_dm: float, t_r: float) -> float:
        if t_r <= 0:
            return 0.0
        p_di = 100.0 * (p_dm / t_r)
        m_di = 100.0 * (m_dm / t_r)
        di_sum = p_di + m_di
        if di_sum <= 0:
            return 0.0
        return 100.0 * (abs(p_di - m_di) / di_sum)

    first_dx = _calc_dx(sm_pdm, sm_mdm, sm_tr)
    dx_list.append(first_dx)

    for i in range(period, len(tr_list)):
        sm_tr = sm_tr - (sm_tr / period) + tr_list[i]
        sm_pdm = sm_pdm - (sm_pdm / period) + plus_dm_list[i]
        sm_mdm = sm_mdm - (sm_mdm / period) + minus_dm_list[i]
        dx_list.append(_calc_dx(sm_pdm, sm_mdm, sm_tr))

    if len(dx_list) < period:
        return None

    # Smooth DX to get ADX
    adx = sum(dx_list[:period]) / period
    for i in range(period, len(dx_list)):
        adx = (adx * (period - 1) + dx_list[i]) / period

    p_di = 100.0 * (sm_pdm / sm_tr) if sm_tr > 0 else 0.0
    m_di = 100.0 * (sm_mdm / sm_tr) if sm_tr > 0 else 0.0

    return {
        "adx": round(adx, 2),
        "plus_di": round(p_di, 2),
        "minus_di": round(m_di, 2)
    }


def classify_market_regime(
    bars: Sequence[Any],
    er_period: int = 16,
    vol_trailing: int = 48,
    vol_atr_period: int = 14,
    adx_period: int = 14,
    min_er_directional: float = 0.40,
    max_er_sideways: float = 0.25,
    min_adx_directional: float = 25.0,
    max_adx_sideways: float = 20.0,
    min_vol_ratio: float = 0.80,
    max_vol_ratio: float = 1.80
) -> Dict[str, Any]:
    """
    Classifies market regime following the exhaustive 4-step precedence:
      1. UNAVAILABLE: Insufficient history or corrupt data.
      2. SIDEWAYS: ER < 0.25 OR ADX < 20.0 OR VolRatio < 0.80.
      3. DIRECTIONAL: ER >= 0.40 AND ADX >= 25.0 AND 0.80 <= VolRatio <= 1.80.
      4. TRANSITIONING: All remaining valid parameter combinations.
    """
    if bars is None or len(bars) == 0:
        return {
            "regime": "UNAVAILABLE",
            "reason": "EMPTY_BAR_HISTORY",
            "er": None,
            "vol_ratio": None,
            "adx": None,
            "plus_di": None,
            "minus_di": None
        }

    closes = []
    for b in bars:
        hlc = _extract_hlc(b)
        if hlc is None:
            return {
                "regime": "UNAVAILABLE",
                "reason": "CORRUPT_BAR_DATA",
                "er": None,
                "vol_ratio": None,
                "adx": None,
                "plus_di": None,
                "minus_di": None
            }
        closes.append(hlc[2])

    er = kaufman_efficiency_ratio(closes, n=er_period)
    vol_ratio = relative_volatility_ratio(bars, n_trailing=vol_trailing, atr_period=vol_atr_period)
    adx_data = adx_indicator(bars, period=adx_period)

    # Step 1: Precedence Check for Data Sufficiency
    if er is None or vol_ratio is None or adx_data is None:
        missing = []
        if er is None:
            missing.append(f"ER_{er_period}(need {er_period+1})")
        if vol_ratio is None:
            missing.append(f"VolRatio_{vol_trailing}(need {vol_trailing+vol_atr_period})")
        if adx_data is None:
            missing.append(f"ADX_{adx_period}(need {2*adx_period+1})")
        return {
            "regime": "UNAVAILABLE",
            "reason": f"INSUFFICIENT_WARMUP:{','.join(missing)}:bars={len(bars)}",
            "er": er,
            "vol_ratio": vol_ratio,
            "adx": adx_data.get("adx") if adx_data else None,
            "plus_di": adx_data.get("plus_di") if adx_data else None,
            "minus_di": adx_data.get("minus_di") if adx_data else None
        }

    adx = adx_data["adx"]
    plus_di = adx_data["plus_di"]
    minus_di = adx_data["minus_di"]

    # Step 2: Explicit Sideways Precedence
    if er < max_er_sideways or adx < max_adx_sideways or vol_ratio < min_vol_ratio:
        reasons = []
        if er < max_er_sideways:
            reasons.append(f"LOW_EFFICIENCY({er:.4f}<{max_er_sideways})")
        if adx < max_adx_sideways:
            reasons.append(f"WEAK_ADX({adx:.1f}<{max_adx_sideways})")
        if vol_ratio < min_vol_ratio:
            reasons.append(f"VOL_COMPRESSION({vol_ratio:.4f}<{min_vol_ratio})")
        return {
            "regime": "SIDEWAYS",
            "reason": ";".join(reasons),
            "er": er,
            "vol_ratio": vol_ratio,
            "adx": adx,
            "plus_di": plus_di,
            "minus_di": minus_di
        }

    # Step 3: Confirmed Directional Precedence
    if (er >= min_er_directional and
            adx >= min_adx_directional and
            min_vol_ratio <= vol_ratio <= max_vol_ratio):
        trend_dir = "BULLISH" if plus_di >= minus_di else "BEARISH"
        return {
            "regime": "DIRECTIONAL",
            "reason": f"CONFIRMED_TREND:{trend_dir}(ER={er:.4f},ADX={adx:.1f},VR={vol_ratio:.4f})",
            "er": er,
            "vol_ratio": vol_ratio,
            "adx": adx,
            "plus_di": plus_di,
            "minus_di": minus_di
        }

    # Step 4: Default Remaining Permutations to Transitioning
    trans_reasons = []
    if max_er_sideways <= er < min_er_directional:
        trans_reasons.append(f"MODERATE_EFFICIENCY({er:.4f})")
    if max_adx_sideways <= adx < min_adx_directional:
        trans_reasons.append(f"DEVELOPING_ADX({adx:.1f})")
    if vol_ratio > max_vol_ratio:
        trans_reasons.append(f"VOL_CLIMAX({vol_ratio:.4f}>{max_vol_ratio})")
    if not trans_reasons:
        trans_reasons.append(f"MIXED_SIGNALS(ER={er:.4f},ADX={adx:.1f},VR={vol_ratio:.4f})")

    return {
        "regime": "TRANSITIONING",
        "reason": ";".join(trans_reasons),
        "er": er,
        "vol_ratio": vol_ratio,
        "adx": adx,
        "plus_di": plus_di,
        "minus_di": minus_di
    }
