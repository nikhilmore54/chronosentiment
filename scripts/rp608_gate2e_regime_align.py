#!/usr/bin/env python3
"""
RP-608 Gate 2E — directional regime alignment + DI skew standaside.

Protocols:
  RP-608-GATE2E-REGIME-ALIGN-001  (DIRECTIONAL adverse)
  RP-608-GATE2E-DECISION-TUNE-001 (SIDEWAYS/TRANSITIONING DI skew)

Authority: 0.0%

Rules (deterministic):
  1) DIRECTIONAL (Kaufman ER / ADX / VolRatio):
       BULLISH (+DI >= -DI) → block SHORT
       BEARISH (+DI <  -DI) → block LONG
  2) SIDEWAYS / TRANSITIONING with large DI skew (|+DI - -DI| >= floor):
       same adverse-side block (DI majority fights proposed side)
  3) UNAVAILABLE / weak DI skew → no block

Not a universal N-day trend filter; not H2 session bans; OF ±0.30 / stops Gate 5 frozen.

PRIMARY performance books: on by default.
BASELINE / Gate2X r50 twins: off by default (CF control).
DI skew guard: RP608_GATE2E_DI_SKEW (default on) · RP608_GATE2E_DI_SKEW_MIN_ABS (default 10).
"""
from __future__ import annotations

import os
from typing import Any, Dict, Optional, Tuple

PROTOCOL_ID = "RP-608-GATE2E-REGIME-ALIGN-001"
PROTOCOL_ID_DECISION_TUNE = "RP-608-GATE2E-DECISION-TUNE-001"
DEFAULT_DI_SKEW_MIN_ABS = 10.0


def _env_bool(name: str, default: bool) -> bool:
    raw = os.environ.get(name)
    if raw is None or str(raw).strip() == "":
        return default
    return str(raw).strip().lower() not in {"0", "false", "no", "off"}


def gate2e_regime_align_enabled() -> bool:
    return _env_bool("RP608_GATE2E_REGIME_ALIGN", True)


def gate2e_regime_align_baseline_enabled() -> bool:
    """BASELINE / r50 twin books stay ungated unless explicitly enabled."""
    return _env_bool("RP608_GATE2E_REGIME_ALIGN_BASELINE", False)


def gate2e_di_skew_enabled() -> bool:
    """SIDEWAYS/TRANSITIONING DI-skew guard (decision-engine tune)."""
    return _env_bool("RP608_GATE2E_DI_SKEW", True)


def gate2e_di_skew_min_abs() -> float:
    raw = os.environ.get("RP608_GATE2E_DI_SKEW_MIN_ABS")
    if raw is None or str(raw).strip() == "":
        return float(DEFAULT_DI_SKEW_MIN_ABS)
    try:
        return max(0.0, float(raw))
    except (TypeError, ValueError):
        return float(DEFAULT_DI_SKEW_MIN_ABS)


def regime_align_applies_to_book(book_id: Optional[str]) -> bool:
    if not gate2e_regime_align_enabled():
        return False
    bid = str(book_id or "").upper()
    if any(
        tok in bid
        for tok in (
            "BASELINE",
            "GATE2X_R50",
            "G2X_R50",
            "FLOOR_LOCK",
            "GATE2X_FLOOR",
            "TREND_AGREE",
            # P1a SCI-clean twin: calibrator owns abstention; DI/regime stack off
            # unless RP608_GATE2E_REGIME_ALIGN_BASELINE explicitly enables.
            "SCI_ADAPTER_REC_CLEAN",
            "SCI_CLEAN",
        )
    ):
        return gate2e_regime_align_baseline_enabled()
    return True


def _di_pair(
    regime: Optional[Dict[str, Any]],
    plus_di: Optional[float],
    minus_di: Optional[float],
) -> Tuple[Optional[float], Optional[float]]:
    p = plus_di
    m = minus_di
    if isinstance(regime, dict):
        if p is None and regime.get("plus_di") is not None:
            try:
                p = float(regime["plus_di"])
            except (TypeError, ValueError):
                p = None
        if m is None and regime.get("minus_di") is not None:
            try:
                m = float(regime["minus_di"])
            except (TypeError, ValueError):
                m = None
    return p, m


def trend_dir_from_regime(regime: Optional[Dict[str, Any]]) -> Optional[str]:
    if not isinstance(regime, dict):
        return None
    explicit = regime.get("trend_dir") or regime.get("directional_trend")
    if explicit in ("BULLISH", "BEARISH"):
        return str(explicit)
    reason = str(regime.get("reason") or "")
    if "BULLISH" in reason:
        return "BULLISH"
    if "BEARISH" in reason:
        return "BEARISH"
    p, m = _di_pair(regime, None, None)
    if p is None or m is None:
        return None
    return "BULLISH" if p >= m else "BEARISH"


def evaluate_regime_align(
    *,
    direction: str,
    regime: Optional[Dict[str, Any]] = None,
    regime_state: Optional[str] = None,
    plus_di: Optional[float] = None,
    minus_di: Optional[float] = None,
    book_id: Optional[str] = None,
) -> Dict[str, Any]:
    """
    Returns a stamp. `blocks_entry` on DIRECTIONAL adverse, or SIDEWAYS/
    TRANSITIONING with large DI skew fighting the proposed side.
    """
    d = str(direction or "").strip().upper()
    applies = regime_align_applies_to_book(book_id)
    state = str(
        regime_state
        or (regime.get("regime") if isinstance(regime, dict) else None)
        or "UNAVAILABLE"
    ).upper()
    p_di, m_di = _di_pair(regime, plus_di, minus_di)
    trend = trend_dir_from_regime(regime) if isinstance(regime, dict) else None
    if trend is None and p_di is not None and m_di is not None:
        trend = "BULLISH" if float(p_di) >= float(m_di) else "BEARISH"

    di_skew = None
    if p_di is not None and m_di is not None:
        try:
            di_skew = abs(float(p_di) - float(m_di))
        except (TypeError, ValueError):
            di_skew = None

    stamp: Dict[str, Any] = {
        "protocol_id": PROTOCOL_ID,
        "decision_tune_protocol_id": PROTOCOL_ID_DECISION_TUNE,
        "gate_enabled": bool(applies),
        "di_skew_enabled": bool(gate2e_di_skew_enabled()),
        "di_skew_min_abs": float(gate2e_di_skew_min_abs()),
        "di_skew": di_skew,
        "plus_di": p_di,
        "minus_di": m_di,
        "regime_state": state,
        "trend_dir": trend,
        "direction": d if d in ("LONG", "SHORT") else None,
        "book_id": book_id,
        "blocks_entry": False,
        "block_reason": None,
        "align": None,
        "guard": None,
    }
    if d not in ("LONG", "SHORT"):
        stamp["block_reason"] = "NO_DIRECTION"
        return stamp

    adverse = trend in ("BULLISH", "BEARISH") and (
        (trend == "BULLISH" and d == "SHORT")
        or (trend == "BEARISH" and d == "LONG")
    )

    # 1) DIRECTIONAL adverse
    if state == "DIRECTIONAL" and trend in ("BULLISH", "BEARISH"):
        stamp["guard"] = "DIRECTIONAL"
        stamp["align"] = "ADVERSE" if adverse else "ALIGNED"
        if adverse and applies:
            stamp["blocks_entry"] = True
            stamp["block_reason"] = f"ADVERSE_TO_DIRECTIONAL_{trend}"
        return stamp

    # 2) SIDEWAYS / TRANSITIONING DI skew (decision-engine tune)
    if (
        state in ("SIDEWAYS", "TRANSITIONING")
        and gate2e_di_skew_enabled()
        and trend in ("BULLISH", "BEARISH")
        and di_skew is not None
        and di_skew >= float(gate2e_di_skew_min_abs())
    ):
        stamp["guard"] = "DI_SKEW"
        stamp["protocol_id"] = PROTOCOL_ID_DECISION_TUNE
        stamp["align"] = "ADVERSE" if adverse else "ALIGNED"
        if adverse and applies:
            stamp["blocks_entry"] = True
            stamp["block_reason"] = f"ADVERSE_TO_DI_SKEW_{trend}"
        return stamp

    stamp["align"] = "NO_DIRECTIONAL_REGIME"
    stamp["guard"] = None
    return stamp


def regime_dict_from_card(card: Optional[Dict[str, Any]]) -> Dict[str, Any]:
    """Rebuild a minimal regime dict from attach_regime_telemetry card fields."""
    if not isinstance(card, dict):
        return {"regime": "UNAVAILABLE"}
    return {
        "regime": card.get("regime_state") or "UNAVAILABLE",
        "reason": card.get("regime_reason"),
        "er": card.get("directional_efficiency_er"),
        "vol_ratio": card.get("relative_volatility"),
        "adx": card.get("adx_14"),
        "plus_di": card.get("plus_di"),
        "minus_di": card.get("minus_di"),
        "trend_dir": card.get("regime_trend_dir"),
    }
