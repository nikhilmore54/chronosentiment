"""
Binance aggTrade → DIRECT aggressor ExecutedTrade (OFX research-only).

Exchange semantics (Binance):
  isBuyerMaker / m == true  → buyer is maker → **sell aggressor**
  isBuyerMaker / m == false → buyer is taker → **buy aggressor**

This is DIRECT source evidence. Not resting L1. Not candle volume.
"""

from __future__ import annotations

from typing import Any, Dict, Iterable, List, Optional

from .aggregate import SIDE_DIRECT, SIDE_UNAVAILABLE, ExecutedTrade, SideClass


def parse_binance_agg_trade(
    row: Dict[str, Any],
    *,
    default_symbol: str = "BTCUSDT",
) -> Optional[ExecutedTrade]:
    """
    Parse one Binance aggTrade WS/REST object or NormalizedTick
    ({symbol, timestamp, price, volume, is_buyer_maker}).
    """
    if not isinstance(row, dict):
        return None

    symbol = str(
        row.get("s")
        or row.get("symbol")
        or row.get("S")
        or default_symbol
        or ""
    ).upper()
    if not symbol:
        return None

    # Timestamps: event time E (ms) preferred; T trade time; timestamp normalized.
    # Use first non-None key — `.get("E", default)` does not skip an explicit null.
    ts = None
    for key in ("E", "T", "timestamp", "event_time", "trade_time"):
        if row.get(key) is not None:
            ts = row.get(key)
            break
    try:
        timestamp_ms = int(ts)
    except (TypeError, ValueError):
        return None
    if timestamp_ms <= 0:
        return None

    try:
        price = float(row.get("p") if row.get("p") is not None else row.get("price") or 0.0)
    except (TypeError, ValueError):
        return None

    qty_raw = row.get("q") if row.get("q") is not None else row.get("quantity", row.get("volume"))
    try:
        quantity = float(qty_raw)
    except (TypeError, ValueError):
        return None

    if "m" in row or "isBuyerMaker" in row or "is_buyer_maker" in row:
        raw_m = row.get("m", row.get("isBuyerMaker", row.get("is_buyer_maker")))
        if isinstance(raw_m, str):
            is_buyer_maker = raw_m.strip().lower() in {"1", "true", "yes"}
        else:
            is_buyer_maker = bool(raw_m)
        if is_buyer_maker:
            side = SideClass(
                aggressor="SELL",
                classification=SIDE_DIRECT,
                evidence="binance_aggTrade.isBuyerMaker=true → sell aggressor",
            )
        else:
            side = SideClass(
                aggressor="BUY",
                classification=SIDE_DIRECT,
                evidence="binance_aggTrade.isBuyerMaker=false → buy aggressor",
            )
    else:
        side = SideClass(
            aggressor="UNKNOWN",
            classification=SIDE_UNAVAILABLE,
            evidence="missing isBuyerMaker/m — qty not signed",
        )

    seq = row.get("a", row.get("aggTradeId", row.get("aggregate_trade_id")))
    sequence_id = str(seq) if seq is not None else None

    return ExecutedTrade(
        instrument_id=symbol,
        timestamp_ms=timestamp_ms,
        price=price,
        quantity=quantity,
        side=side,
        source="binance_aggTrade",
        sequence_id=sequence_id,
        raw_ref=sequence_id,
    )


def trades_from_aggtrade_records(
    rows: Iterable[Dict[str, Any]],
    *,
    default_symbol: str = "BTCUSDT",
) -> List[ExecutedTrade]:
    out: List[ExecutedTrade] = []
    for row in rows:
        t = parse_binance_agg_trade(row, default_symbol=default_symbol)
        if t is not None:
            out.append(t)
    return out
