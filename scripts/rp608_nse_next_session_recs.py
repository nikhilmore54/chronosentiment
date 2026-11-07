#!/usr/bin/env python3
"""
RP-608 NSE — next-session intraday watchlist (post-close).

After continuous session close, rank the RP-607 **classified** universe
(102 of census 119; excludes C_Quarantined) for *next session* intraday
observation using today's completed 15m bars (fetch-backed).

Governance: 0.0% order authority. Not executable overnight / into CAS.
Status: WATCH_NEXT_SESSION — actionable only after next-session warmup
(bar ≥5, within MAX_NEW_ENTRY_BAR), subject to live traj15 / quote gates.
"""

from __future__ import annotations

import json
import math
from dataclasses import asdict, dataclass
from datetime import date, datetime, timedelta, timezone
from pathlib import Path
from typing import Any, Dict, List, Optional, Sequence, Set

from scripts.dynamic_trade_boundaries import compute_dynamic_boundaries
from scripts.rp608_order_flow_mark import (
    ORDER_FLOW_IMBALANCE_THRESHOLD,
    ORDER_FLOW_LABEL,
    classify_order_flow_support,
    l1_imbalance,
    micro_vs_mid_bps,
)

PROTOCOL_ID = "RP-608-NSE-NEXT-SESSION-001"
LIVE_ORDER_AUTHORITY_PCT = 0.0
TRAJ15_ENTRY_THRESHOLD = 0.05
TOP_N_DEFAULT = 20
MIN_BARS = 5
LEVELS_POLICY_ID = "RP608_DYNAMIC_OHLCV_STRUCTURE_V1"
LEVELS_ROLE = "INDICATIVE_WATCHLIST_ONLY"
DEFAULT_FRICTION_BPS = 12.0
# Entry band as fractions of structural risk distance (ref − stop for LONG).
ENTRY_PULLBACK_FRAC = 0.35
ENTRY_CHASE_FRAC = 0.25
ENTRY_STOP_BUFFER_FRAC = 0.05

# Align with close-window adaptation
MAX_NEW_ENTRY_BAR = 19
MANDATORY_SQUARE_OFF_BAR = 23

LEVEL_STATUS_VALID = "VALID"
LEVEL_STATUS_WAITING = "WAITING"
LEVEL_STATUS_INVALIDATED = "INVALIDATED"
LEVEL_STATUS_STALE = "STALE_SUPPRESSED"


@dataclass
class NextSessionCandidate:
    symbol: str
    tradingsymbol: str
    side: str
    decision: str
    status: str
    score: float
    traj15_eod: float
    session_return_bps: float
    close_price: float
    session_bars: int
    thesis: str
    actionable_from_session_date: str
    actionable_from_bar: int
    live_order_authority: float = LIVE_ORDER_AUTHORITY_PCT
    protocol_id: str = PROTOCOL_ID
    horizon_class: str = "INTRADAY"
    horizon_id: str = "H0_NSE_NEXT_SESSION_WATCH"
    source: str = "RP-608-NSE-EOD-FETCH"
    order_flow_supported: bool = False
    order_flow_label: Optional[str] = None
    order_flow_imbalance: Optional[float] = None
    order_flow_source: str = "none"
    recommended_at_utc: Optional[str] = None
    # Indicative structure levels (NOT executed / NOT paper entry).
    levels: Optional[Dict[str, Any]] = None
    level_status: Optional[str] = None
    entry_low: Optional[float] = None
    entry_high: Optional[float] = None
    stop_price: Optional[float] = None
    target_price: Optional[float] = None
    risk_reward_display: Optional[str] = None
    levels_role: str = LEVELS_ROLE
    not_executed_entry: bool = True
    not_paper_trade_entry: bool = True


def _traj15(closes: Sequence[float]) -> float:
    if len(closes) < 2:
        return 0.0
    window = list(closes[-5:]) if len(closes) >= 5 else list(closes)
    if len(window) < 2 or window[0] <= 0:
        return 0.0
    ret = (window[-1] / window[0]) - 1.0
    denom = 0.01 * math.sqrt(max(len(window) - 1, 1))
    if denom <= 0:
        return 0.0
    return float(math.tanh(ret / denom))


def _session_return_bps(closes: Sequence[float]) -> float:
    if len(closes) < 2 or closes[0] <= 0:
        return 0.0
    return ((float(closes[-1]) / float(closes[0])) - 1.0) * 10000.0


def next_weekday_session(d: date, holidays: Optional[Set[date]] = None) -> date:
    """Next Mon–Fri session date, skipping optional exchange holidays."""
    hol = holidays or set()
    cur = d + timedelta(days=1)
    for _ in range(14):
        if cur.weekday() < 5 and cur not in hol:
            return cur
        cur += timedelta(days=1)
    return d + timedelta(days=1)


def bar_closes_from_history(bars: Sequence[Any]) -> List[float]:
    out: List[float] = []
    for b in bars:
        if isinstance(b, dict):
            c = b.get("close")
        else:
            c = getattr(b, "close", None)
        if c is None:
            continue
        try:
            out.append(float(c))
        except (TypeError, ValueError):
            continue
    return out


def _bar_ts_ms(bar: Any) -> Optional[int]:
    try:
        if isinstance(bar, dict):
            ts = bar.get("timestamp_utc_ms")
        else:
            ts = getattr(bar, "timestamp_utc_ms", None)
        return int(ts) if ts is not None else None
    except (TypeError, ValueError):
        return None


def compute_indicative_watch_levels(
    bars: Sequence[Any],
    *,
    side: str = "LONG",
    reference_price: Optional[float] = None,
    friction_bps: float = DEFAULT_FRICTION_BPS,
    calculated_at_utc: Optional[str] = None,
) -> Dict[str, Any]:
    """
    Indicative entry range / stop / target / R:R for next-session watch rows.

    Reuses RP608_DYNAMIC_OHLCV_STRUCTURE_V1 at an *indicative* reference (last close).
    Not an executed paper/live entry price. Authority remains 0%.
    """
    side_u = str(side or "LONG").upper()
    closes = bar_closes_from_history(bars)
    if reference_price is None:
        if not closes:
            return {
                "ok": False,
                "reason": "NO_REFERENCE_PRICE",
                "levels_role": LEVELS_ROLE,
                "policy_id": LEVELS_POLICY_ID,
            }
        reference_price = float(closes[-1])
    ref = float(reference_price)
    if ref <= 0 or not math.isfinite(ref):
        return {
            "ok": False,
            "reason": "INVALID_REFERENCE_PRICE",
            "levels_role": LEVELS_ROLE,
            "policy_id": LEVELS_POLICY_ID,
        }

    bounds = compute_dynamic_boundaries(
        bars,
        side=side_u,
        entry_price=ref,
        friction_bps=max(1.0, float(friction_bps)),
    )
    if bounds.get("stop_price") is None or bounds.get("target_price") is None:
        return {
            "ok": False,
            "reason": bounds.get("reason") or "DYNAMIC_UNAVAILABLE",
            "levels_role": LEVELS_ROLE,
            "policy_id": LEVELS_POLICY_ID,
            "reference_price": round(ref, 4),
            "not_executed_entry": True,
            "not_paper_trade_entry": True,
        }

    stop = float(bounds["stop_price"])
    target = float(bounds["target_price"])
    if side_u == "LONG":
        risk = max(ref - stop, 1e-9)
        reward = max(target - ref, 0.0)
        entry_low = ref - ENTRY_PULLBACK_FRAC * risk
        entry_high = ref + ENTRY_CHASE_FRAC * risk
        # Keep band above stop and below target.
        floor = stop + ENTRY_STOP_BUFFER_FRAC * risk
        entry_low = max(entry_low, floor)
        entry_high = min(entry_high, target - ENTRY_STOP_BUFFER_FRAC * max(reward, risk))
        if entry_high < entry_low:
            entry_low, entry_high = ref, ref
    else:
        risk = max(stop - ref, 1e-9)
        reward = max(ref - target, 0.0)
        entry_high = ref + ENTRY_PULLBACK_FRAC * risk
        entry_low = ref - ENTRY_CHASE_FRAC * risk
        ceil = stop - ENTRY_STOP_BUFFER_FRAC * risk
        entry_high = min(entry_high, ceil)
        entry_low = max(entry_low, target + ENTRY_STOP_BUFFER_FRAC * max(reward, risk))
        if entry_high < entry_low:
            entry_low, entry_high = ref, ref

    gross_rr = reward / risk if risk > 0 else None
    net_rr = bounds.get("net_rr")
    last_bar_ts = _bar_ts_ms(bars[-1]) if bars else None
    calc_at = calculated_at_utc or datetime.now(timezone.utc).isoformat().replace("+00:00", "Z")

    return {
        "ok": bool(bounds.get("ok")),
        "reason": bounds.get("reason") or "OK",
        "levels_role": LEVELS_ROLE,
        "policy_id": str(bounds.get("policy_id") or LEVELS_POLICY_ID),
        "not_executed_entry": True,
        "not_paper_trade_entry": True,
        "reference_price": round(ref, 4),
        "entry_low": round(float(entry_low), 4),
        "entry_high": round(float(entry_high), 4),
        "stop_price": round(stop, 4),
        "target_price": round(target, 4),
        "stop_bps": bounds.get("stop_bps"),
        "target_bps": bounds.get("target_bps"),
        "risk_reward_gross": None if gross_rr is None else round(float(gross_rr), 4),
        "risk_reward_net": None if net_rr is None else round(float(net_rr), 4),
        "risk_reward_display": (
            None
            if gross_rr is None
            else f"1 : {gross_rr:.2f}"
        ),
        "atr_14": bounds.get("atr_14"),
        "trend_label": bounds.get("trend_label"),
        "structure_pivot": bounds.get("structure_pivot"),
        "friction_bps": bounds.get("friction_bps"),
        "levels_calculated_at_utc": calc_at,
        "levels_source_bar_timestamp_utc_ms": last_bar_ts,
        "structure_generation": 1,
        "invalidation_latched": False,
        "levels_provenance": {
            "policy_id": str(bounds.get("policy_id") or LEVELS_POLICY_ID),
            "reference": "last_completed_bar_close",
            "entry_band": {
                "pullback_frac_of_risk": ENTRY_PULLBACK_FRAC,
                "chase_frac_of_risk": ENTRY_CHASE_FRAC,
                "stop_buffer_frac": ENTRY_STOP_BUFFER_FRAC,
            },
            "authority_pct": LIVE_ORDER_AUTHORITY_PCT,
        },
    }


def classify_watch_level_status(
    *,
    side: str,
    last_price: Optional[float],
    entry_low: Optional[float],
    entry_high: Optional[float],
    stop_price: Optional[float],
    feed_stale: bool = False,
    reconcile_pending: bool = False,
    session_actionable: bool = True,
    invalidation_latched: bool = False,
) -> Dict[str, Any]:
    """
    Live validity of the indicative entry band (price-event path).

    Safeguard: once INVALIDATED for the current structure_generation, VALID is
    forbidden until a fresh completed-bar structural evaluation clears the latch.
    """
    # Pre-actionable next session: WAITING wins (do not conflate with live stale).
    if not session_actionable:
        return {
            "level_status": LEVEL_STATUS_WAITING,
            "level_status_reason": "NEXT_SESSION_NOT_ACTIONABLE_YET",
            "live_validity_suppressed": False,
            "invalidation_latched": bool(invalidation_latched),
        }
    if feed_stale or reconcile_pending:
        return {
            "level_status": LEVEL_STATUS_STALE,
            "level_status_reason": (
                "RECONCILE_PENDING" if reconcile_pending and not feed_stale else "FEED_STALE"
            ),
            "live_validity_suppressed": True,
            "invalidation_latched": bool(invalidation_latched),
        }
    if last_price is None or entry_low is None or entry_high is None or stop_price is None:
        return {
            "level_status": LEVEL_STATUS_WAITING,
            "level_status_reason": "PRICE_OR_LEVELS_UNAVAILABLE",
            "live_validity_suppressed": False,
            "invalidation_latched": bool(invalidation_latched),
        }
    px = float(last_price)
    lo = float(entry_low)
    hi = float(entry_high)
    stop = float(stop_price)
    side_u = str(side or "LONG").upper()
    latched = bool(invalidation_latched)

    def _pack(status: str, reason: str, *, latch: Optional[bool] = None) -> Dict[str, Any]:
        return {
            "level_status": status,
            "level_status_reason": reason,
            "live_validity_suppressed": False,
            "invalidation_latched": latched if latch is None else bool(latch),
        }

    if side_u == "LONG":
        if px <= stop:
            return _pack(
                LEVEL_STATUS_INVALIDATED,
                "PRICE_AT_OR_THROUGH_STOP",
                latch=True,
            )
        if lo <= px <= hi:
            if latched:
                return _pack(
                    LEVEL_STATUS_INVALIDATED,
                    "LATCHED_INVALID_REQUIRES_FRESH_STRUCTURE",
                    latch=True,
                )
            return _pack(LEVEL_STATUS_VALID, "PRICE_INSIDE_ENTRY_RANGE")
        if px > hi:
            return _pack(
                LEVEL_STATUS_WAITING,
                "PRICE_ABOVE_ENTRY_RANGE_WAIT_PULLBACK",
            )
        return _pack(LEVEL_STATUS_WAITING, "PRICE_BELOW_ENTRY_RANGE")

    # SHORT
    if px >= stop:
        return _pack(
            LEVEL_STATUS_INVALIDATED,
            "PRICE_AT_OR_THROUGH_STOP",
            latch=True,
        )
    if lo <= px <= hi:
        if latched:
            return _pack(
                LEVEL_STATUS_INVALIDATED,
                "LATCHED_INVALID_REQUIRES_FRESH_STRUCTURE",
                latch=True,
            )
        return _pack(LEVEL_STATUS_VALID, "PRICE_INSIDE_ENTRY_RANGE")
    if px < lo:
        return _pack(
            LEVEL_STATUS_WAITING,
            "PRICE_BELOW_ENTRY_RANGE_WAIT_BOUNCE",
        )
    return _pack(LEVEL_STATUS_WAITING, "PRICE_ABOVE_ENTRY_RANGE")


def enrich_recommendation_levels_live(
    recommendations: Sequence[Dict[str, Any]],
    *,
    price_history: Optional[Dict[str, Sequence[Any]]] = None,
    live_prices: Optional[Dict[str, float]] = None,
    feed_stale_by_symbol: Optional[Dict[str, bool]] = None,
    reconcile_pending: bool = False,
    session_actionable: bool = True,
    now_utc: Optional[datetime] = None,
    recalculate_structure: bool = False,
) -> List[Dict[str, Any]]:
    """
    Overlay live level_status (and optional structure refresh) onto watch rows.

    Structural refresh is authorized only when recalculate_structure=True (completed
    15m bar path). Presentation pollers must pass recalculate_structure=False.
    """
    now = now_utc or datetime.now(timezone.utc)
    now_iso = now.isoformat().replace("+00:00", "Z")
    live_prices = live_prices or {}
    feed_stale_by_symbol = feed_stale_by_symbol or {}
    price_history = price_history or {}
    out: List[Dict[str, Any]] = []
    for raw in recommendations:
        row = dict(raw)
        sym = str(row.get("symbol") or "")
        side = str(row.get("side") or "LONG")
        bars = price_history.get(sym) or []
        levels = row.get("levels") if isinstance(row.get("levels"), dict) else None
        prior_gen = int((levels or {}).get("structure_generation") or 1)
        prior_bar_ts = (levels or {}).get("levels_source_bar_timestamp_utc_ms")
        prior_latched = bool((levels or {}).get("invalidation_latched"))

        if recalculate_structure and bars:
            fresh = compute_indicative_watch_levels(
                bars,
                side=side,
                reference_price=None,
                calculated_at_utc=now_iso,
            )
            new_bar_ts = fresh.get("levels_source_bar_timestamp_utc_ms")
            # Only accept as a new structural generation when the completed bar advances.
            if fresh.get("ok") and new_bar_ts is not None and new_bar_ts != prior_bar_ts:
                fresh = dict(fresh)
                fresh["structure_generation"] = prior_gen + 1
                fresh["invalidation_latched"] = False  # latch clear on fresh structure
                levels = fresh
                prior_latched = False
                row["levels"] = levels
            elif levels is None and fresh.get("ok"):
                levels = fresh
                row["levels"] = levels

        if not levels:
            out.append(row)
            continue
        last_px = live_prices.get(sym)
        if last_px is None and row.get("close_price") is not None:
            try:
                last_px = float(row["close_price"])
            except (TypeError, ValueError):
                last_px = None
        status = classify_watch_level_status(
            side=side,
            last_price=last_px,
            entry_low=levels.get("entry_low"),
            entry_high=levels.get("entry_high"),
            stop_price=levels.get("stop_price"),
            feed_stale=bool(feed_stale_by_symbol.get(sym, False)),
            reconcile_pending=bool(reconcile_pending),
            session_actionable=session_actionable,
            invalidation_latched=prior_latched,
        )
        levels = dict(levels)
        levels["level_status"] = status["level_status"]
        levels["level_status_reason"] = status["level_status_reason"]
        levels["live_validity_suppressed"] = status["live_validity_suppressed"]
        levels["invalidation_latched"] = bool(status.get("invalidation_latched"))
        levels["structure_generation"] = int(levels.get("structure_generation") or prior_gen or 1)
        levels["last_price_for_status"] = None if last_px is None else round(float(last_px), 4)
        levels["level_status_asof_utc"] = now_iso
        row["levels"] = levels
        row["level_status"] = status["level_status"]
        row["invalidation_latched"] = levels["invalidation_latched"]
        row["entry_low"] = levels.get("entry_low")
        row["entry_high"] = levels.get("entry_high")
        row["stop_price"] = levels.get("stop_price")
        row["target_price"] = levels.get("target_price")
        row["risk_reward_display"] = levels.get("risk_reward_display")
        out.append(row)
    return out


def apply_price_event_to_watch_levels(
    levels: Dict[str, Any],
    *,
    side: str,
    last_price: float,
    feed_stale: bool = False,
    reconcile_pending: bool = False,
    session_actionable: bool = True,
    asof_utc: Optional[str] = None,
) -> Dict[str, Any]:
    """Push-path validity update only — does not recompute structure."""
    status = classify_watch_level_status(
        side=side,
        last_price=last_price,
        entry_low=levels.get("entry_low"),
        entry_high=levels.get("entry_high"),
        stop_price=levels.get("stop_price"),
        feed_stale=feed_stale,
        reconcile_pending=reconcile_pending,
        session_actionable=session_actionable,
        invalidation_latched=bool(levels.get("invalidation_latched")),
    )
    out = dict(levels)
    out["level_status"] = status["level_status"]
    out["level_status_reason"] = status["level_status_reason"]
    out["live_validity_suppressed"] = status["live_validity_suppressed"]
    out["invalidation_latched"] = bool(status.get("invalidation_latched"))
    out["last_price_for_status"] = round(float(last_price), 4)
    out["level_status_asof_utc"] = asof_utc or datetime.now(timezone.utc).isoformat().replace(
        "+00:00", "Z"
    )
    return out


def l1_snapshots_from_latest_evaluations(path: Path) -> Dict[str, Dict[str, Any]]:
    """
    Build per-symbol L1 snapshots from rp608_nse_latest_evaluations.json
    (data_quality.bid_qty / ask_qty written at last continuous bar).
    """
    out: Dict[str, Dict[str, Any]] = {}
    if not path.exists():
        return out
    try:
        raw = json.loads(path.read_text(encoding="utf-8"))
    except Exception:
        return out
    evaluations = raw.get("evaluations") if isinstance(raw, dict) else None
    if not isinstance(evaluations, dict):
        return out
    for sym, ev in evaluations.items():
        if isinstance(ev, list):
            ev = ev[-1] if ev else None
        if not isinstance(ev, dict):
            continue
        dq = ev.get("data_quality") if isinstance(ev.get("data_quality"), dict) else {}
        try:
            bid_qty = float(dq.get("bid_qty") or 0.0)
            ask_qty = float(dq.get("ask_qty") or 0.0)
        except (TypeError, ValueError):
            continue
        snap: Dict[str, Any] = {
            "bid_qty": bid_qty,
            "ask_qty": ask_qty,
            "source": "latest_evaluations_l1",
        }
        bid_p = ev.get("bid_price", ev.get("best_bid"))
        ask_p = ev.get("ask_price", ev.get("best_ask"))
        try:
            if bid_p is not None:
                snap["bid_price"] = float(bid_p)
            if ask_p is not None:
                snap["ask_price"] = float(ask_p)
        except (TypeError, ValueError):
            pass
        out[str(sym)] = snap
        # Also index by tradingsymbol without .NS
        ts = str(sym).replace(".NS", "")
        out[ts] = snap
    return out


def _lookup_l1(
    l1_by_symbol: Optional[Dict[str, Dict[str, Any]]],
    symbol: str,
) -> Optional[Dict[str, Any]]:
    if not l1_by_symbol:
        return None
    if symbol in l1_by_symbol:
        return l1_by_symbol[symbol]
    ts = str(symbol).replace(".NS", "")
    if ts in l1_by_symbol:
        return l1_by_symbol[ts]
    dotted = ts if str(symbol).endswith(".NS") else f"{symbol}.NS"
    return l1_by_symbol.get(dotted)


def load_rp607_classified_symbols(path: Path) -> Set[str]:
    """
    Return RP-607 classified symbols (valid / non-quarantined).
    Census remains 119; actionable watch universe is typically 102.
    """
    out: Set[str] = set()
    if not path.exists():
        return out
    try:
        raw = json.loads(path.read_text(encoding="utf-8"))
    except Exception:
        return out
    profiles = raw.get("profiles") if isinstance(raw, dict) else None
    if not isinstance(profiles, dict):
        return out
    for sym, rec in profiles.items():
        if not isinstance(rec, dict):
            continue
        comp = str(rec.get("compartment") or "")
        valid = rec.get("valid")
        quarantined = (
            valid is False
            or "Quarant" in comp
            or bool(rec.get("quarantine_reason"))
        )
        if quarantined:
            continue
        out.add(str(sym))
        out.add(str(sym).replace(".NS", ""))
        ts = rec.get("tradingsymbol")
        if ts:
            out.add(str(ts))
    return out


def _symbol_allowed(symbol: str, eligible: Optional[Set[str]]) -> bool:
    if eligible is None:
        return True
    if not eligible:
        return False
    if symbol in eligible:
        return True
    ts = str(symbol).replace(".NS", "")
    if ts in eligible:
        return True
    return f"{ts}.NS" in eligible


def build_next_session_recommendations(
    price_history: Dict[str, Sequence[Any]],
    *,
    from_session_date: str,
    next_session_date: str,
    top_n: int = TOP_N_DEFAULT,
    now_utc: Optional[datetime] = None,
    l1_by_symbol: Optional[Dict[str, Dict[str, Any]]] = None,
    eligible_symbols: Optional[Set[str]] = None,
    universe_label: str = "RP-607-classified-102",
) -> Dict[str, Any]:
    """
    Rank symbols for next-session LONG watch from today's completed bars.
    Deterministic: same histories (+ same L1 + same eligibility) → same list / OF marks.
    When eligible_symbols is set (RP-607 classified), quarantined names are skipped.
    """
    now = now_utc or datetime.now(timezone.utc)
    emitted_utc = now.isoformat().replace("+00:00", "Z")
    candidates: List[NextSessionCandidate] = []
    skipped_ineligible = 0

    for sym, bars in sorted(price_history.items()):
        if not _symbol_allowed(str(sym), eligible_symbols):
            skipped_ineligible += 1
            continue
        closes = bar_closes_from_history(bars)
        if len(closes) < MIN_BARS:
            continue
        traj = _traj15(closes)
        sess_bps = _session_return_bps(closes)
        # Rank score: emphasize end-of-day traj15 (same family as live entry),
        # with session return as soft tie-break (scaled).
        score = traj + (sess_bps / 10000.0) * 0.25
        if traj < TRAJ15_ENTRY_THRESHOLD and sess_bps < 25.0:
            continue
        side = "LONG"
        of = classify_order_flow_support(side, _lookup_l1(l1_by_symbol, str(sym)))
        of_note = ""
        if of["order_flow_supported"]:
            of_note = (
                f"; L1 order-flow confirms LONG "
                f"(imbalance={of['order_flow_imbalance']:+.3f} ≥ {ORDER_FLOW_IMBALANCE_THRESHOLD:.2f})"
            )
        thesis = (
            f"EOD traj15={traj:+.4f}, session={sess_bps:+.1f} bps over {len(closes)} bars — "
            f"watch for next-session intraday continuation after warmup{of_note}"
        )
        levels = compute_indicative_watch_levels(
            bars,
            side=side,
            reference_price=float(closes[-1]),
            calculated_at_utc=emitted_utc,
        )
        # Post-close stamp: not yet actionable next session → WAITING.
        status_blk = classify_watch_level_status(
            side=side,
            last_price=float(closes[-1]),
            entry_low=levels.get("entry_low"),
            entry_high=levels.get("entry_high"),
            stop_price=levels.get("stop_price"),
            feed_stale=False,
            session_actionable=False,
        )
        if levels.get("ok"):
            levels = dict(levels)
            levels["level_status"] = status_blk["level_status"]
            levels["level_status_reason"] = status_blk["level_status_reason"]
            levels["live_validity_suppressed"] = status_blk["live_validity_suppressed"]
            levels["level_status_asof_utc"] = emitted_utc
        candidates.append(
            NextSessionCandidate(
                symbol=str(sym),
                tradingsymbol=str(sym).replace(".NS", ""),
                side=side,
                decision="WATCH_NEXT_SESSION",
                status="WATCH_NEXT_SESSION",
                score=round(score, 6),
                traj15_eod=round(traj, 6),
                session_return_bps=round(sess_bps, 4),
                close_price=round(closes[-1], 4),
                session_bars=len(closes),
                thesis=thesis,
                actionable_from_session_date=next_session_date,
                actionable_from_bar=MIN_BARS,
                order_flow_supported=bool(of["order_flow_supported"]),
                order_flow_label=of["order_flow_label"],
                order_flow_imbalance=of["order_flow_imbalance"],
                order_flow_source=str(of["order_flow_source"]),
                recommended_at_utc=emitted_utc,
                levels=levels,
                level_status=status_blk["level_status"],
                entry_low=levels.get("entry_low"),
                entry_high=levels.get("entry_high"),
                stop_price=levels.get("stop_price"),
                target_price=levels.get("target_price"),
                risk_reward_display=levels.get("risk_reward_display"),
            )
        )

    candidates.sort(key=lambda c: (-c.score, c.symbol))
    top = candidates[: max(0, int(top_n))]
    n_of = sum(1 for c in top if c.order_flow_supported)

    return {
        "protocol_id": PROTOCOL_ID,
        "live_order_authority": LIVE_ORDER_AUTHORITY_PCT,
        "emitted_utc": emitted_utc,
        "recommended_at_utc": emitted_utc,
        "from_session_date": from_session_date,
        "next_session_date": next_session_date,
        "closing_window_ist": "15:00-15:15",
        "mandatory_square_off_bar": MANDATORY_SQUARE_OFF_BAR,
        "max_new_entry_bar_next_session": MAX_NEW_ENTRY_BAR,
        "order_flow_imbalance_threshold": ORDER_FLOW_IMBALANCE_THRESHOLD,
        "n_order_flow_supported": n_of,
        "universe_label": universe_label if eligible_symbols is not None else "full_price_history",
        "universe_eligible_n": (
            len({s for s in eligible_symbols if str(s).endswith(".NS")})
            if eligible_symbols is not None
            else None
        ),
        "skipped_ineligible": skipped_ineligible if eligible_symbols is not None else 0,
        "note": (
            "Post-close next-session intraday watchlist from today's kite/15m fetches. "
            "Universe: RP-607 classified (102) — census 119 minus 17 quarantined. "
            "Not executable overnight or in CAS. Next session: re-validate traj15 + L1 "
            f"after bar {MIN_BARS}; no new entries after bar {MAX_NEW_ENTRY_BAR}; "
            f"square-off bar {MANDATORY_SQUARE_OFF_BAR} (15:00). "
            f"ORDER_FLOW_SUPPORTED = L1 TOB imbalance agrees with side "
            f"(|imb|≥{ORDER_FLOW_IMBALANCE_THRESHOLD:.2f}); unmarked otherwise. "
            f"Indicative entry/stop/target from {LEVELS_POLICY_ID} — "
            "watchlist guidance only, not executed or paper-trade entry."
        ),
        "levels_policy_id": LEVELS_POLICY_ID,
        "levels_role": LEVELS_ROLE,
        "levels_informational_only": True,
        "universe_scored": len(candidates),
        "n_recommendations": len(top),
        "recommendations": [asdict(c) for c in top],
    }


def write_next_session_recommendations(path: Path, payload: Dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    tmp = path.with_suffix(".tmp")
    tmp.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
    tmp.replace(path)


def load_price_history_json(path: Path) -> Dict[str, List[Dict[str, Any]]]:
    if not path.exists():
        return {}
    try:
        raw = json.loads(path.read_text(encoding="utf-8"))
    except Exception:
        return {}
    if not isinstance(raw, dict):
        return {}
    out: Dict[str, List[Dict[str, Any]]] = {}
    for k, v in raw.items():
        if isinstance(v, list):
            out[str(k)] = [x for x in v if isinstance(x, dict)]
    return out
