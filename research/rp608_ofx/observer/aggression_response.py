#!/usr/bin/env python3
"""
RP-608-OFX Stage 1 — aggression → price-response observational helpers.

Research-only. No recommendations, orders, or Stage-2 mechanism labels
(absorption / acceptance / failed auction).
"""

from __future__ import annotations

from dataclasses import dataclass
from typing import Any, Dict, List, Optional, Sequence, Tuple


DEFAULT_HORIZONS_MS: Tuple[int, ...] = (1_000, 5_000, 15_000, 30_000)


def mid_price(bid: float, ask: float, ltp: float) -> Optional[float]:
    if bid > 0 and ask > 0 and ask > bid:
        return 0.5 * (bid + ask)
    if ltp > 0:
        return ltp
    return None


def signed_bps(start: float, end: float) -> Optional[float]:
    if start is None or end is None or start <= 0:
        return None
    return (end - start) / start * 10_000.0


def aggression_sign(label: str) -> int:
    if label == "BUY_INITIATED_CANDIDATE":
        return 1
    if label == "SELL_INITIATED_CANDIDATE":
        return -1
    return 0


def response_class(sign: int, ret_bps: Optional[float], *, flat_bps: float = 2.0) -> str:
    """Descriptive price-response class (not a trading signal)."""
    if ret_bps is None:
        return "INSUFFICIENT_FORWARD"
    if abs(ret_bps) < flat_bps:
        return "FLAT"
    if sign > 0:
        return "ADVANCES_WITH" if ret_bps > 0 else "ADVANCES_AGAINST"
    if sign < 0:
        # sell aggression: with = price down
        return "ADVANCES_WITH" if ret_bps < 0 else "ADVANCES_AGAINST"
    return "UNAVAILABLE"


@dataclass
class TickPoint:
    receipt_utc_ms: int
    ltp: float
    bid: float
    ask: float
    mid: float
    trade_like: bool
    infer_label: str
    infer_confidence: str
    ltq: float
    volume: float
    bid_qty: float
    ask_qty: float
    update_class: str
    symbol: str
    instrument_token: int


def tick_from_row(row: Dict[str, Any]) -> Optional[TickPoint]:
    try:
        receipt = int(row.get("receipt_utc_ms") or 0)
    except (TypeError, ValueError):
        return None
    if receipt <= 0:
        return None
    try:
        ltp = float(row.get("last_price") or 0.0)
        bid = float(row.get("best_bid") or 0.0)
        ask = float(row.get("best_ask") or 0.0)
    except (TypeError, ValueError):
        return None
    mid = mid_price(bid, ask, ltp)
    if mid is None:
        return None
    prim = row.get("aggressor_inference_primary") or {}
    try:
        tok = int(row.get("instrument_token") or 0)
    except (TypeError, ValueError):
        tok = 0
    return TickPoint(
        receipt_utc_ms=receipt,
        ltp=ltp,
        bid=bid,
        ask=ask,
        mid=float(mid),
        trade_like=bool(row.get("trade_like")),
        infer_label=str(prim.get("label") or "UNAVAILABLE"),
        infer_confidence=str(prim.get("confidence") or "NONE"),
        ltq=float(row.get("last_traded_quantity") or 0.0),
        volume=float(row.get("volume") or 0.0),
        bid_qty=float(row.get("bid_qty") or 0.0),
        ask_qty=float(row.get("ask_qty") or 0.0),
        update_class=str(row.get("update_class") or ""),
        symbol=str(row.get("symbol") or ""),
        instrument_token=tok,
    )


def find_forward_mid(points: Sequence[TickPoint], i: int, horizon_ms: int) -> Tuple[Optional[float], Optional[int], bool]:
    """Return (mid, receipt_ms, exact_or_after_found) at/after t0+horizon."""
    t0 = points[i].receipt_utc_ms
    target = t0 + horizon_ms
    # linear scan forward — points are chronological per symbol
    for j in range(i + 1, len(points)):
        if points[j].receipt_utc_ms >= target:
            return points[j].mid, points[j].receipt_utc_ms, True
    return None, None, False


def persistence_same_side(
    points: Sequence[TickPoint],
    i: int,
    *,
    window_ms: int = 5_000,
) -> Dict[str, Any]:
    """Count subsequent trade-like same-side inferences within window."""
    sign0 = aggression_sign(points[i].infer_label)
    if sign0 == 0:
        return {"window_ms": window_ms, "same_side_trade_like": 0, "opposite_trade_like": 0}
    t0 = points[i].receipt_utc_ms
    same = 0
    opp = 0
    for j in range(i + 1, len(points)):
        if points[j].receipt_utc_ms > t0 + window_ms:
            break
        if not points[j].trade_like:
            continue
        s = aggression_sign(points[j].infer_label)
        if s == sign0:
            same += 1
        elif s == -sign0:
            opp += 1
    return {"window_ms": window_ms, "same_side_trade_like": same, "opposite_trade_like": opp}


def liquidity_delta(points: Sequence[TickPoint], i: int, horizon_ms: int = 5_000) -> Dict[str, Any]:
    """L1 displayed qty change after event (not aggressor volume)."""
    t0 = points[i].receipt_utc_ms
    bid0, ask0 = points[i].bid_qty, points[i].ask_qty
    bid1, ask1 = bid0, ask0
    found = False
    for j in range(i + 1, len(points)):
        if points[j].receipt_utc_ms >= t0 + horizon_ms:
            bid1, ask1 = points[j].bid_qty, points[j].ask_qty
            found = True
            break
    if not found:
        return {"horizon_ms": horizon_ms, "available": False}
    return {
        "horizon_ms": horizon_ms,
        "available": True,
        "bid_qty_delta": bid1 - bid0,
        "ask_qty_delta": ask1 - ask0,
    }


def observe_event(
    points: Sequence[TickPoint],
    i: int,
    *,
    horizons_ms: Sequence[int] = DEFAULT_HORIZONS_MS,
    flat_bps: float = 2.0,
) -> Optional[Dict[str, Any]]:
    """Build one Stage-1 observation for a trade-like buy/sell candidate."""
    p = points[i]
    if not p.trade_like:
        return None
    sign = aggression_sign(p.infer_label)
    if sign == 0:
        return None

    responses = {}
    for h in horizons_ms:
        mid1, ts1, ok = find_forward_mid(points, i, h)
        ret = signed_bps(p.mid, mid1) if ok else None
        aligned = None
        if ret is not None:
            aligned = (ret * sign) > 0
        responses[str(h)] = {
            "horizon_ms": h,
            "forward_available": ok,
            "mid0": p.mid,
            "mid1": mid1,
            "return_bps": None if ret is None else round(ret, 4),
            "signed_return_bps": None if ret is None else round(ret * sign, 4),
            "response_class": response_class(sign, ret, flat_bps=flat_bps),
            "aligned_with_aggression": aligned,
            "forward_receipt_utc_ms": ts1,
        }

    # proportionality proxy: |return_bps| / ln(1+ltq) at 5s if available
    r5 = responses.get("5000") or {}
    prop = None
    if r5.get("return_bps") is not None and p.ltq >= 0:
        import math

        denom = math.log1p(max(p.ltq, 0.0))
        if denom > 0:
            prop = abs(float(r5["return_bps"])) / denom

    return {
        "protocol_id": "RP-608-OFX",
        "phase": "STAGE_1_AGGRESSION_RESPONSE_OBSERVER",
        "record_type": "aggression_response_observation",
        "symbol": p.symbol,
        "instrument_token": p.instrument_token,
        "event_receipt_utc_ms": p.receipt_utc_ms,
        "aggression": {
            "label": p.infer_label,
            "confidence": p.infer_confidence,
            "sign": sign,
            "ltq": p.ltq,
            "update_class": p.update_class,
            "ltp": p.ltp,
            "bid": p.bid,
            "ask": p.ask,
            "mid": p.mid,
        },
        "persistence_5s": persistence_same_side(points, i, window_ms=5_000),
        "liquidity_delta_5s": liquidity_delta(points, i, horizon_ms=5_000),
        "price_response": responses,
        "abs_return_per_log_ltq_5s": None if prop is None else round(prop, 6),
        "stage2_mechanism_label": "NOT_ASSIGNED_STAGE_1",
        "live_order_authority_pct": 0.0,
    }


def aggregate_observations(rows: Sequence[Dict[str, Any]], *, horizons_ms: Sequence[int] = DEFAULT_HORIZONS_MS) -> Dict[str, Any]:
    n = len(rows)
    by_side = {"BUY_INITIATED_CANDIDATE": 0, "SELL_INITIATED_CANDIDATE": 0}
    by_conf = {}
    horizon_stats: Dict[str, Dict[str, Any]] = {}
    for h in horizons_ms:
        horizon_stats[str(h)] = {
            "n_with_forward": 0,
            "n_advances_with": 0,
            "n_advances_against": 0,
            "n_flat": 0,
            "n_insufficient": 0,
            "sum_signed_return_bps": 0.0,
            "n_signed": 0,
            "n_aligned": 0,
        }

    for r in rows:
        lab = (r.get("aggression") or {}).get("label")
        if lab in by_side:
            by_side[lab] += 1
        conf = (r.get("aggression") or {}).get("confidence") or "NONE"
        by_conf[conf] = by_conf.get(conf, 0) + 1
        for h, resp in (r.get("price_response") or {}).items():
            st = horizon_stats.get(str(h))
            if not st:
                continue
            cls = resp.get("response_class")
            if not resp.get("forward_available"):
                st["n_insufficient"] += 1
                continue
            st["n_with_forward"] += 1
            if cls == "ADVANCES_WITH":
                st["n_advances_with"] += 1
            elif cls == "ADVANCES_AGAINST":
                st["n_advances_against"] += 1
            elif cls == "FLAT":
                st["n_flat"] += 1
            else:
                st["n_insufficient"] += 1
            sret = resp.get("signed_return_bps")
            if sret is not None:
                st["sum_signed_return_bps"] += float(sret)
                st["n_signed"] += 1
            if resp.get("aligned_with_aggression") is True:
                st["n_aligned"] += 1

    for h, st in horizon_stats.items():
        ns = st["n_signed"] or 0
        st["mean_signed_return_bps"] = round(st["sum_signed_return_bps"] / ns, 4) if ns else None
        nf = st["n_with_forward"] or 0
        st["with_share"] = round(st["n_advances_with"] / nf, 4) if nf else None
        st["against_share"] = round(st["n_advances_against"] / nf, 4) if nf else None
        st["flat_share"] = round(st["n_flat"] / nf, 4) if nf else None
        st["aligned_share"] = round(st["n_aligned"] / nf, 4) if nf else None
        del st["sum_signed_return_bps"]

    # Stability read treats FLAT as first-class (short-horizon noise floor).
    f5 = (horizon_stats.get("5000") or {}).get("flat_share")
    f15 = (horizon_stats.get("15000") or {}).get("flat_share")
    w5 = (horizon_stats.get("5000") or {}).get("with_share")
    w15 = (horizon_stats.get("15000") or {}).get("with_share")
    a5 = (horizon_stats.get("5000") or {}).get("against_share")
    a15 = (horizon_stats.get("15000") or {}).get("against_share")
    if w5 is None or w15 is None or n < 50:
        stability = "INCONCLUSIVE_SAMPLE"
    elif f5 is not None and f15 is not None and f5 >= 0.60 and f15 >= 0.55:
        stability = "DOMINANTLY_FLAT_SHORT_HORIZON"
    elif w5 >= 0.55 and w15 >= 0.55:
        stability = "SUGGESTIVE_ALIGNED_RESPONSE"
    elif a5 is not None and a15 is not None and a5 >= 0.55 and a15 >= 0.55:
        stability = "SUGGESTIVE_AGAINST_RESPONSE"
    else:
        stability = "MIXED_OR_UNSTABLE"

    return {
        "n_events": n,
        "by_aggression_label": by_side,
        "by_confidence": by_conf,
        "horizons": horizon_stats,
        "stability_read": stability,
        "note": (
            "Descriptive only. Inferred aggressor ≠ exchange aggressor. "
            "Not Stage-2 absorption/acceptance/failed-auction labels. "
            "Not a trading edge claim."
        ),
    }
