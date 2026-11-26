#!/usr/bin/env python3
"""
Shared L1 order-flow confirmation mark (NSE + Crypto).

Deterministic, confirmatory only — does not change authority (0.0%).
LONG: bid-heavy imbalance; SHORT: ask-heavy. Optional microprice agree when prices exist.
"""

from __future__ import annotations

from typing import Any, Dict, Optional

ORDER_FLOW_IMBALANCE_THRESHOLD = 0.30
ORDER_FLOW_LABEL = "ORDER_FLOW_SUPPORTED"
ORDER_FLOW_EXIT_LABEL = "ORDER_FLOW_EXIT"


def opposite_side(side: str) -> str:
    s = str(side or "").upper()
    if s == "LONG":
        return "SHORT"
    if s == "SHORT":
        return "LONG"
    return ""


def is_order_flow_exit_signal(
    active_side: str,
    l1: Optional[Dict[str, Any]],
    *,
    imbalance_threshold: float = ORDER_FLOW_IMBALANCE_THRESHOLD,
) -> Dict[str, Any]:
    """
    OF exit = opposite-side order-flow confirmation (flow reverse).

    Not time-based. LONG OF exits only when SHORT OF support fires; vice versa.
    """
    opp = opposite_side(active_side)
    if not opp:
        return {
            "exit": False,
            "order_flow_exit_label": None,
            "opposite_side": None,
            "order_flow_imbalance": None,
            "order_flow_source": "none",
        }
    of = classify_order_flow_support(
        opp, l1, imbalance_threshold=imbalance_threshold
    )
    exited = bool(of.get("order_flow_supported"))
    return {
        "exit": exited,
        "order_flow_exit_label": ORDER_FLOW_EXIT_LABEL if exited else None,
        "opposite_side": opp,
        "order_flow_imbalance": of.get("order_flow_imbalance"),
        "order_flow_source": of.get("order_flow_source"),
        "order_flow_label": of.get("order_flow_label"),
    }


def l1_imbalance(bid_qty: float, ask_qty: float) -> Optional[float]:
    """Causal TOB qty imbalance in [-1, +1]. None if book empty."""
    b = float(bid_qty or 0.0)
    a = float(ask_qty or 0.0)
    tot = b + a
    if tot <= 0.0:
        return None
    return (b - a) / tot


def micro_vs_mid_bps(
    bid_price: Optional[float],
    ask_price: Optional[float],
    bid_qty: float,
    ask_qty: float,
) -> Optional[float]:
    """Microprice vs mid in bps; None if L1 prices invalid."""
    try:
        bp = float(bid_price) if bid_price is not None else 0.0
        ap = float(ask_price) if ask_price is not None else 0.0
    except (TypeError, ValueError):
        return None
    if bp <= 0.0 or ap <= 0.0 or ap <= bp:
        return None
    mid = 0.5 * (bp + ap)
    if mid <= 0.0:
        return None
    bq = float(bid_qty or 0.0)
    aq = float(ask_qty or 0.0)
    denom = bq + aq
    if denom <= 0.0:
        micro = mid
    else:
        micro = (ap * bq + bp * aq) / denom
    return ((micro - mid) / mid) * 10000.0


def classify_order_flow_support(
    side: str,
    l1: Optional[Dict[str, Any]],
    *,
    imbalance_threshold: float = ORDER_FLOW_IMBALANCE_THRESHOLD,
) -> Dict[str, Any]:
    """
    Mark LONG/SHORT as ORDER_FLOW_SUPPORTED when L1 TOB agrees with side.

    - LONG: imbalance >= +threshold; SHORT: <= -threshold
    - If bid/ask prices present: also require microprice on same side of mid
    - Unsupported / unavailable → not marked (no negative badge)
    """
    empty = {
        "order_flow_supported": False,
        "order_flow_label": None,
        "order_flow_imbalance": None,
        "order_flow_source": "none",
    }
    if not l1 or not isinstance(l1, dict):
        return empty

    # Accept either qty keys or precomputed imbalance (tick features)
    imb: Optional[float] = None
    if l1.get("imbalance") is not None and l1.get("bid_qty") is None and l1.get("ask_qty") is None:
        try:
            imb = float(l1["imbalance"])
        except (TypeError, ValueError):
            imb = None
    else:
        try:
            bid_qty = float(l1.get("bid_qty") or l1.get("best_bid_qty") or 0.0)
            ask_qty = float(l1.get("ask_qty") or l1.get("best_ask_qty") or 0.0)
        except (TypeError, ValueError):
            return empty
        imb = l1_imbalance(bid_qty, ask_qty)

    if imb is None:
        return {**empty, "order_flow_source": str(l1.get("source") or "unavailable")}

    side_u = str(side or "").upper()
    source = str(l1.get("source") or "l1_tob")

    micro_bps: Optional[float] = None
    if l1.get("micro_vs_mid_bps") is not None:
        try:
            micro_bps = float(l1["micro_vs_mid_bps"])
        except (TypeError, ValueError):
            micro_bps = None
    else:
        try:
            bid_qty = float(l1.get("bid_qty") or l1.get("best_bid_qty") or 0.0)
            ask_qty = float(l1.get("ask_qty") or l1.get("best_ask_qty") or 0.0)
        except (TypeError, ValueError):
            bid_qty = ask_qty = 0.0
        micro_bps = micro_vs_mid_bps(
            l1.get("bid_price", l1.get("best_bid_price")),
            l1.get("ask_price", l1.get("best_ask_price")),
            bid_qty,
            ask_qty,
        )

    if side_u == "LONG":
        qty_ok = imb >= float(imbalance_threshold)
        micro_ok = True if micro_bps is None else (micro_bps >= 0.0)
    elif side_u == "SHORT":
        qty_ok = imb <= -float(imbalance_threshold)
        micro_ok = True if micro_bps is None else (micro_bps <= 0.0)
    else:
        return {
            "order_flow_supported": False,
            "order_flow_label": None,
            "order_flow_imbalance": round(imb, 6),
            "order_flow_source": source,
        }

    supported = bool(qty_ok and micro_ok)
    return {
        "order_flow_supported": supported,
        "order_flow_label": ORDER_FLOW_LABEL if supported else None,
        "order_flow_imbalance": round(imb, 6),
        "order_flow_source": source,
    }


def l1_from_book_ticker(quote: Optional[Dict[str, Any]], *, source: str = "bookTicker") -> Dict[str, Any]:
    """Normalize Binance bookTicker / execution_quote into classify input."""
    if not quote or not isinstance(quote, dict):
        return {"source": "none"}
    return {
        "bid_qty": quote.get("best_bid_qty", quote.get("bid_qty")),
        "ask_qty": quote.get("best_ask_qty", quote.get("ask_qty")),
        "bid_price": quote.get("best_bid_price", quote.get("bid_price")),
        "ask_price": quote.get("best_ask_price", quote.get("ask_price")),
        "source": source,
    }


def l1_from_nse_quote(
    quote: Optional[Any],
    *,
    source: str = "kite_l1",
) -> Dict[str, Any]:
    """
    Normalize NSE TopOfBookQuote / quote dict into classify input.

    Accepts objects with bid_qty/ask_qty/bid_price/ask_price attributes or dicts
    with those keys (also best_bid_* / best_ask_* aliases).
    """
    if quote is None:
        return {"source": "none"}
    if isinstance(quote, dict):
        return l1_from_book_ticker(quote, source=source)

    def _get(name: str, *alts: str) -> Any:
        for key in (name, *alts):
            if hasattr(quote, key):
                return getattr(quote, key)
        return None

    return {
        "bid_qty": _get("bid_qty", "best_bid_qty"),
        "ask_qty": _get("ask_qty", "best_ask_qty"),
        "bid_price": _get("bid_price", "best_bid_price"),
        "ask_price": _get("ask_price", "best_ask_price"),
        "source": source,
    }


def inspect_order_flow_book(
    l1: Optional[Dict[str, Any]],
    *,
    primary_side: str = "LONG",
    imbalance_threshold: float = ORDER_FLOW_IMBALANCE_THRESHOLD,
) -> Dict[str, Any]:
    """
    Observational OF inspection for any symbol (NSE census / crypto).

    Does not gate entries or raise authority. Stamps both LONG and SHORT
    support plus the primary-side mark used for badges.
    """
    side = str(primary_side or "LONG").upper()
    if side not in ("LONG", "SHORT"):
        side = "LONG"

    long_of = classify_order_flow_support(
        "LONG", l1, imbalance_threshold=imbalance_threshold
    )
    short_of = classify_order_flow_support(
        "SHORT", l1, imbalance_threshold=imbalance_threshold
    )
    primary = long_of if side == "LONG" else short_of
    imb = primary.get("order_flow_imbalance")
    if imb is None:
        imb = long_of.get("order_flow_imbalance")
    if imb is None:
        imb = short_of.get("order_flow_imbalance")

    source = (
        primary.get("order_flow_source")
        or long_of.get("order_flow_source")
        or short_of.get("order_flow_source")
        or "none"
    )
    supported = bool(primary.get("order_flow_supported"))
    return {
        "order_flow_supported": supported,
        "order_flow_label": ORDER_FLOW_LABEL if supported else None,
        "order_flow_imbalance": imb,
        "order_flow_source": source,
        "order_flow_side": side,
        "order_flow_long_supported": bool(long_of.get("order_flow_supported")),
        "order_flow_short_supported": bool(short_of.get("order_flow_supported")),
        "order_flow_inspection": True,
        "order_flow_imbalance_threshold": float(imbalance_threshold),
    }


def l1_from_tick_features(features: Optional[Dict[str, Any]], *, source: str = "tick_features") -> Dict[str, Any]:
    """Normalize features_tick snapshot into classify input."""
    if not features or not isinstance(features, dict):
        return {"source": "none"}
    out: Dict[str, Any] = {
        "imbalance": features.get("imbalance"),
        "micro_vs_mid_bps": features.get("micro_vs_mid_bps"),
        "source": source,
    }
    return out
