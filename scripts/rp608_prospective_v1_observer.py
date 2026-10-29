#!/usr/bin/env python3
"""
Prospective V1 observational A/B/C recorder.

Arm definitions (matches historical ABC harness on Arm A opportunities):
  Arm A — frozen baseline signal (crypto Policy B; NSE RECOMMEND_LONG)
  Arm B — Arm A ∩ DIRECTIONAL
  Arm C — DIRECTIONAL ∩ TRADE_QUALIFIED  (nested: take_c = take_b ∧ TRADE_QUALIFIED)

Live order authority remains 0.0%. Does not alter Policy B/C actions, thresholds, or exits.
Writes only under artifacts/rp608_regime_breakout/prospective_v1/ when that market is STARTED.
"""

from __future__ import annotations

import json
import os
import tempfile
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, List, Optional, Sequence

from scripts.breakout_lifecycle import LifecycleState, evaluate_quote, pack_bars_for_breakout_eval
from scripts.market_regime_classifier import classify_market_regime
from scripts.rp608_prospective_v1 import (
    IST,
    Market,
    PROSPECTIVE_NS,
    PROTOCOL_ID,
    START_RULE_VERSION,
    assert_write_allowed,
    ensure_namespace,
    is_market_started,
    load_start_event,
)

LIVE_ORDER_AUTHORITY = 0.0
LABEL = "prospective_shadow"

TARGET_DISTANCE = 150.0
STOP_DISTANCE = 100.0
FRICTION = 12.0
CRYPTO_HALF_SPREAD_CAP_BPS = 2.50
NSE_HALF_SPREAD_CAP_BPS = 1.80


def journals_dir() -> Path:
    d = PROSPECTIVE_NS / "journals"
    d.mkdir(parents=True, exist_ok=True)
    return d


def opportunity_journal_path(market: Market) -> Path:
    return journals_dir() / f"{market.value}_abc_opportunities.jsonl"


def recommendation_journal_path(market: Market) -> Path:
    """Full recommendation tape (every evaluated signal/flat), not only Arm A hits."""
    return journals_dir() / f"{market.value}_recommendations.jsonl"


def recommendation_summary_path(market: Market) -> Path:
    return journals_dir() / f"{market.value}_recommendations_summary.json"


def summary_path(market: Market) -> Path:
    return journals_dir() / f"{market.value}_abc_summary.json"


def _bar_dict(bar: Any) -> Dict[str, float]:
    if isinstance(bar, dict):
        return {
            "open": float(bar["open"]),
            "high": float(bar["high"]),
            "low": float(bar["low"]),
            "close": float(bar["close"]),
        }
    return {
        "open": float(getattr(bar, "open")),
        "high": float(getattr(bar, "high")),
        "low": float(getattr(bar, "low")),
        "close": float(getattr(bar, "close")),
    }


def bars_to_ohlc(bars: Sequence[Any]) -> List[Dict[str, float]]:
    return [_bar_dict(b) for b in bars]


def _atomic_update_summary(market: Market, record: Dict[str, Any]) -> None:
    path = summary_path(market)
    assert_write_allowed(path)
    cur: Dict[str, Any] = {}
    if path.exists():
        try:
            cur = json.loads(path.read_text(encoding="utf-8"))
        except Exception:
            cur = {}
    cur.setdefault("protocol_id", PROTOCOL_ID)
    cur.setdefault("start_rule_version", START_RULE_VERSION)
    cur.setdefault("market", market.value)
    cur.setdefault("live_order_authority", LIVE_ORDER_AUTHORITY)
    cur["updated_utc"] = datetime.now(timezone.utc).isoformat()
    cur["n_bars"] = int(cur.get("n_bars") or 0) + 1
    if record.get("arm_a_signal"):
        cur["n_arm_a_opportunities"] = int(cur.get("n_arm_a_opportunities") or 0) + 1
    counts = cur.setdefault("lifecycle_state_counts", {})
    st = record.get("lifecycle_state") or "NONE"
    counts[st] = int(counts.get(st) or 0) + 1
    if record.get("arm_a_signal"):
        cur["arm_b_taken"] = int(cur.get("arm_b_taken") or 0) + (1 if record.get("take_b") else 0)
        cur["arm_c_taken"] = int(cur.get("arm_c_taken") or 0) + (1 if record.get("take_c") else 0)
    cur["trade_qualified_bars"] = int(cur.get("trade_qualified_bars") or 0) + (
        1 if record.get("lifecycle_state") == "TRADE_QUALIFIED" else 0
    )
    cur["directional_bars"] = int(cur.get("directional_bars") or 0) + (
        1 if record.get("regime") == "DIRECTIONAL" else 0
    )
    cur["arm_c_intersection_bars"] = int(cur.get("arm_c_intersection_bars") or 0) + (
        1 if record.get("arm_c_intersection") else 0
    )
    fd, tmp = tempfile.mkstemp(prefix=".tmp_sum_", dir=str(path.parent))
    try:
        with os.fdopen(fd, "w", encoding="utf-8") as fh:
            fh.write(json.dumps(cur, indent=2) + "\n")
            fh.flush()
            os.fsync(fh.fileno())
        os.replace(tmp, path)
    finally:
        if os.path.exists(tmp):
            try:
                os.unlink(tmp)
            except OSError:
                pass


def append_opportunity(market: Market, record: Dict[str, Any]) -> None:
    ensure_namespace()
    path = opportunity_journal_path(market)
    assert_write_allowed(path)
    with path.open("a", encoding="utf-8") as fh:
        fh.write(json.dumps(record, separators=(",", ":")) + "\n")
    _atomic_update_summary(market, record)


def _atomic_update_rec_summary(market: Market, record: Dict[str, Any]) -> None:
    path = recommendation_summary_path(market)
    assert_write_allowed(path)
    cur: Dict[str, Any] = {}
    if path.exists():
        try:
            cur = json.loads(path.read_text(encoding="utf-8"))
        except Exception:
            cur = {}
    cur.setdefault("protocol_id", PROTOCOL_ID)
    cur.setdefault("start_rule_version", START_RULE_VERSION)
    cur.setdefault("market", market.value)
    cur.setdefault("live_order_authority", LIVE_ORDER_AUTHORITY)
    cur["updated_utc"] = datetime.now(timezone.utc).isoformat()
    cur["n_recommendations"] = int(cur.get("n_recommendations") or 0) + 1
    by_sym = cur.setdefault("by_symbol", {})
    sym = str(record.get("symbol") or "?")
    by_sym[sym] = int(by_sym.get(sym) or 0) + 1
    by_dec = cur.setdefault("by_decision", {})
    dec = str(record.get("decision") or record.get("primary_action") or "?")
    by_dec[dec] = int(by_dec.get(dec) or 0) + 1
    if record.get("is_actionable"):
        cur["n_actionable"] = int(cur.get("n_actionable") or 0) + 1
    fd, tmp = tempfile.mkstemp(prefix=".tmp_recsum_", dir=str(path.parent))
    try:
        with os.fdopen(fd, "w", encoding="utf-8") as fh:
            fh.write(json.dumps(cur, indent=2) + "\n")
            fh.flush()
            os.fsync(fh.fileno())
        os.replace(tmp, path)
    finally:
        if os.path.exists(tmp):
            try:
                os.unlink(tmp)
            except OSError:
                pass


def record_recommendation(
    *,
    market: Market,
    symbol: str,
    decision: str,
    session_date: str,
    bar_idx: int,
    close_time_ms: Optional[int] = None,
    reference_price: Optional[float] = None,
    secondary_decision: Optional[str] = None,
    decision_code: Optional[str] = None,
    is_actionable: Optional[bool] = None,
    regime: Optional[str] = None,
    extra: Optional[Dict[str, Any]] = None,
) -> Optional[Dict[str, Any]]:
    """
    Append one row to the full recommendation research tape.
    Journals every evaluation (LONG/SHORT/NO_TRADE/RECOMMEND_LONG/…) when market STARTED.
    0.0% authority — observational only.
    """
    if not is_market_started(market):
        return None

    start_ev = load_start_event(market) or {}
    actionable = is_actionable
    if actionable is None:
        actionable = decision in ("LONG", "SHORT", "RECOMMEND_LONG", "RECOMMEND_SHORT")

    now = datetime.now(timezone.utc)
    record: Dict[str, Any] = {
        "protocol_id": PROTOCOL_ID,
        "start_rule_version": START_RULE_VERSION,
        "label": "prospective_recommendation_tape",
        "market": market.value,
        "symbol": symbol,
        "session_date": session_date,
        "bar_idx": bar_idx,
        "close_time_ms": close_time_ms,
        "recorded_utc": now.isoformat(),
        "recorded_ist": now.astimezone(IST).isoformat(),
        "eligible_session_boundary": start_ev.get("eligible_session_boundary"),
        "live_order_authority": LIVE_ORDER_AUTHORITY,
        "arm_a_frozen": True,
        "decision": decision,
        "primary_action": decision,
        "secondary_decision": secondary_decision,
        "decision_code": decision_code,
        "is_actionable": bool(actionable),
        "reference_price": reference_price,
        "regime": regime,
    }
    if extra:
        record["extra"] = extra

    ensure_namespace()
    path = recommendation_journal_path(market)
    assert_write_allowed(path)
    with path.open("a", encoding="utf-8") as fh:
        fh.write(json.dumps(record, separators=(",", ":")) + "\n")
    _atomic_update_rec_summary(market, record)
    return record


def observe_bar(
    *,
    market: Market,
    completed_bars_including_signal: Sequence[Any],
    ask: Optional[float],
    bid: Optional[float],
    quote_age_ms: Optional[float],
    session_date: str,
    symbol: str,
    bar_idx: int,
    close_time_ms: Optional[int] = None,
    arm_a_signal: bool,
    arm_a_side: Optional[str] = None,
    reference_price: Optional[float] = None,
    extra: Optional[Dict[str, Any]] = None,
) -> Optional[Dict[str, Any]]:
    """
    Evaluate A/B/C dispositions for one completed bar.
    Returns None if market not commenced (nothing written).
    """
    if not is_market_started(market):
        return None

    start_ev = load_start_event(market) or {}
    ohlc = bars_to_ohlc(completed_bars_including_signal)
    if len(ohlc) < 2:
        return None

    regime = classify_market_regime(ohlc)
    regime_label = str((regime or {}).get("regime") or "UNAVAILABLE")

    half_cap = CRYPTO_HALF_SPREAD_CAP_BPS if market == Market.CRYPTO else NSE_HALF_SPREAD_CAP_BPS
    lifecycle_state = "IDLE"
    lifecycle_reason: Optional[str] = None
    lifecycle_direction: Optional[str] = None
    lifecycle_dedupe: Optional[str] = None
    net_rr: Optional[float] = None
    quote_ok = (
        ask is not None
        and bid is not None
        and float(ask) > 0
        and float(bid) > 0
        and float(ask) > float(bid)
        and quote_age_ms is not None
    )

    if quote_ok:
        packed = pack_bars_for_breakout_eval(ohlc)
        lc = evaluate_quote(
            state=LifecycleState(),
            regime=regime_label,
            bars_completed_plus_forming=packed,
            ask=float(ask),
            bid=float(bid),
            quote_age_ms=float(quote_age_ms),
            session_date=session_date,
            symbol=symbol,
            half_spread_cap_bps=half_cap,
            target_distance=TARGET_DISTANCE,
            stop_distance=STOP_DISTANCE,
            friction=FRICTION,
        )
        lifecycle_state = lc.state
        lifecycle_reason = lc.reason
        lifecycle_direction = lc.direction
        lifecycle_dedupe = lc.dedupe
        net_rr = lc.net_rr
    else:
        lifecycle_reason = "invalid_or_missing_executable_quote"

    # Historical ABC: on Arm A opportunities, B = DIRECTIONAL, C = DIRECTIONAL ∩ TQ
    take_b = bool(arm_a_signal) and regime_label == "DIRECTIONAL"
    take_c = take_b and lifecycle_state == "TRADE_QUALIFIED"
    arm_c_intersection = regime_label == "DIRECTIONAL" and lifecycle_state == "TRADE_QUALIFIED"

    if take_c:
        disposition_c = "taken"
    elif not arm_a_signal:
        disposition_c = "arm_a_flat"
    elif regime_label != "DIRECTIONAL":
        disposition_c = "regime_reject"
    else:
        disposition_c = "breakout_reject"

    if take_b:
        disposition_b = "taken"
    elif not arm_a_signal:
        disposition_b = "arm_a_flat"
    else:
        disposition_b = "regime_reject"

    now = datetime.now(timezone.utc)
    record: Dict[str, Any] = {
        "protocol_id": PROTOCOL_ID,
        "start_rule_version": START_RULE_VERSION,
        "label": LABEL,
        "market": market.value,
        "symbol": symbol,
        "session_date": session_date,
        "bar_idx": bar_idx,
        "close_time_ms": close_time_ms,
        "recorded_utc": now.isoformat(),
        "recorded_ist": now.astimezone(IST).isoformat(),
        "eligible_session_boundary": start_ev.get("eligible_session_boundary"),
        "live_order_authority": LIVE_ORDER_AUTHORITY,
        "arm_a_frozen": True,
        "reference_price": reference_price,
        "ask": ask,
        "bid": bid,
        "quote_age_ms": quote_age_ms,
        "quote_ok": quote_ok,
        "regime": regime_label,
        "regime_er": (regime or {}).get("er"),
        "regime_adx": (regime or {}).get("adx"),
        "regime_vol_ratio": (regime or {}).get("vol_ratio"),
        "regime_reason": (regime or {}).get("reason"),
        "arm_a_signal": bool(arm_a_signal),
        "arm_a_side": arm_a_side,
        "take_b": take_b,
        "take_c": take_c,
        "arm_c_intersection": arm_c_intersection,
        "disposition_b": disposition_b,
        "disposition_c": disposition_c,
        "lifecycle_state": lifecycle_state,
        "lifecycle_reason": lifecycle_reason,
        "lifecycle_direction": lifecycle_direction,
        "lifecycle_dedupe": lifecycle_dedupe,
        "net_rr": net_rr,
        "half_spread_cap_bps": half_cap,
        "target_distance": TARGET_DISTANCE,
        "stop_distance": STOP_DISTANCE,
        "friction": FRICTION,
        "packing": "pack_bars_for_breakout_eval",
    }
    if extra:
        record["extra"] = extra

    append_opportunity(market, record)
    return record
