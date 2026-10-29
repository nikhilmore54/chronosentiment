#!/usr/bin/env python3
"""
RP-608-REGIME-BREAKOUT-001 Phase 2: NSE two-level market context.

Computes equal-weight breadth, market return, dispersion, and MARKET_CHOP.
Under MARKET_CHOP, stock-level DIRECTIONAL requires ER >= 0.50.
Live order authority: 0.0%.
"""

from __future__ import annotations

import math
from typing import Any, Dict, List, Optional, Sequence, Tuple


def _ret_bps(px0: float, px1: float) -> Optional[float]:
    if px0 is None or px1 is None or px0 <= 0 or px1 <= 0:
        return None
    return ((px1 / px0) - 1.0) * 10_000.0


def compute_market_context(
    symbol_returns_bps: Sequence[float],
) -> Dict[str, Any]:
    """
    Equal-weight cross-section from per-symbol session returns (bps).
    Breadth = fraction with return > 0.
    """
    rets = [float(r) for r in symbol_returns_bps if r is not None and math.isfinite(r)]
    n = len(rets)
    if n == 0:
        return {
            "session_breadth": None,
            "market_return_bps": None,
            "market_dispersion_bps": None,
            "n_active": 0,
            "market_chop": False,
            "reason": "NO_ACTIVE_SYMBOLS",
        }
    advancing = sum(1 for r in rets if r > 0.0)
    breadth = advancing / n
    mean_r = sum(rets) / n
    var = sum((r - mean_r) ** 2 for r in rets) / n
    disp = math.sqrt(var)
    chop = (0.40 <= breadth <= 0.60) and (abs(mean_r) < 10.0)
    return {
        "session_breadth": round(breadth, 4),
        "market_return_bps": round(mean_r, 4),
        "market_dispersion_bps": round(disp, 4),
        "n_active": n,
        "market_chop": chop,
        "reason": "MARKET_CHOP" if chop else "MARKET_CLEAR",
    }


def apply_market_chop_to_regime(
    stock_regime: Dict[str, Any],
    market_ctx: Dict[str, Any],
    chop_er_hurdle: float = 0.50,
    base_er_hurdle: float = 0.40,
) -> Dict[str, Any]:
    """
    Precedence: stock UNAVAILABLE/SIDEWAYS unchanged.
    If MARKET_CHOP and stock would be DIRECTIONAL with ER < chop_er_hurdle,
    demote to TRANSITIONING.
    """
    out = dict(stock_regime)
    out["market_chop"] = bool(market_ctx.get("market_chop"))
    out["session_breadth"] = market_ctx.get("session_breadth")
    out["market_return_bps"] = market_ctx.get("market_return_bps")
    out["market_dispersion_bps"] = market_ctx.get("market_dispersion_bps")
    out["directional_er_hurdle"] = (
        chop_er_hurdle if market_ctx.get("market_chop") else base_er_hurdle
    )

    if out.get("regime") in ("UNAVAILABLE", "SIDEWAYS"):
        return out
    if not market_ctx.get("market_chop"):
        return out

    er = out.get("er")
    if out.get("regime") == "DIRECTIONAL" and er is not None and er < chop_er_hurdle:
        out["regime"] = "TRANSITIONING"
        out["reason"] = (
            f"MARKET_CHOP_ER_HURDLE({er:.4f}<{chop_er_hurdle});"
            + str(out.get("reason") or "")
        )
    return out


def session_return_bps_from_bars(
    bars: Sequence[Any],
) -> Optional[float]:
    """Return from first to last close in bps."""
    if not bars or len(bars) < 2:
        return None
    first = bars[0]
    last = bars[-1]
    c0 = first["close"] if isinstance(first, dict) else getattr(first, "close", None)
    c1 = last["close"] if isinstance(last, dict) else getattr(last, "close", None)
    return _ret_bps(float(c0), float(c1)) if c0 and c1 else None
