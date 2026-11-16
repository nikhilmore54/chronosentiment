"""
NSE Stage 0D inferred-aggressor ticks → DERIVED_VALIDATED ExecutedTrade.

Kite does **not** supply exchange aggressor side. Labels are LTP-vs-quote
inference (`BUY_INITIATED_CANDIDATE` / `SELL_INITIATED_CANDIDATE`) with
confidence. This feed is never DIRECT.

Default: only MEDIUM confidence (prior-quote path) is emitted as
DERIVED_VALIDATED for signing. LOW / AMBIGUOUS / UNAVAILABLE → UNKNOWN
with UNAVAILABLE classification (qty not signed).

Not resting L1 imbalance. Not unsigned candle volume.
"""

from __future__ import annotations

from typing import Any, Dict, Iterable, List, Optional, Set

from .aggregate import SIDE_DERIVED_VALIDATED, SIDE_UNAVAILABLE, ExecutedTrade, SideClass

_CONF_RANK = {"NONE": 0, "LOW": 1, "MEDIUM": 2, "HIGH": 3}


def _conf_ok(conf: str, minimum: str) -> bool:
    return _CONF_RANK.get(str(conf).upper(), 0) >= _CONF_RANK.get(str(minimum).upper(), 0)


def parse_nse_inferred_tick(
    row: Dict[str, Any],
    *,
    min_confidence: str = "MEDIUM",
    require_trade_like: bool = True,
) -> Optional[ExecutedTrade]:
    if not isinstance(row, dict):
        return None

    if require_trade_like and not bool(row.get("trade_like")):
        return None

    sym = row.get("symbol")
    tok = row.get("instrument_token")
    instrument_id = str(sym) if sym else (str(tok) if tok is not None else "")
    if not instrument_id:
        return None

    ts = row.get("receipt_utc_ms") or row.get("exchange_timestamp_utc_ms")
    try:
        timestamp_ms = int(ts)
    except (TypeError, ValueError):
        return None
    if timestamp_ms <= 0:
        return None

    try:
        price = float(row.get("last_price") or 0.0)
    except (TypeError, ValueError):
        price = 0.0

    try:
        quantity = float(row.get("last_traded_quantity") or 0.0)
    except (TypeError, ValueError):
        return None

    prim = row.get("aggressor_inference_primary") or {}
    label = str(prim.get("label") or "UNAVAILABLE")
    conf = str(prim.get("confidence") or "NONE")
    reason = str(prim.get("reason") or "")

    if label.startswith("BUY") and _conf_ok(conf, min_confidence):
        side = SideClass(
            aggressor="BUY",
            classification=SIDE_DERIVED_VALIDATED,
            evidence=f"nse_stage0d {label} conf={conf} ({reason})",
        )
    elif label.startswith("SELL") and _conf_ok(conf, min_confidence):
        side = SideClass(
            aggressor="SELL",
            classification=SIDE_DERIVED_VALIDATED,
            evidence=f"nse_stage0d {label} conf={conf} ({reason})",
        )
    else:
        side = SideClass(
            aggressor="UNKNOWN",
            classification=SIDE_UNAVAILABLE,
            evidence=f"nse_stage0d not signable label={label} conf={conf}",
        )

    seq_parts = [
        str(tok) if tok is not None else instrument_id,
        str(timestamp_ms),
        str(row.get("last_trade_time_sec") or ""),
        str(row.get("volume") or ""),
        str(quantity),
    ]
    sequence_id = "|".join(seq_parts)

    return ExecutedTrade(
        instrument_id=instrument_id,
        timestamp_ms=timestamp_ms,
        price=price,
        quantity=quantity,
        side=side,
        source="nse_ofx_stage0d_inferred",
        sequence_id=sequence_id,
        raw_ref=sequence_id,
    )


def trades_from_nse_inferred_ticks(
    rows: Iterable[Dict[str, Any]],
    *,
    min_confidence: str = "MEDIUM",
    require_trade_like: bool = True,
    instrument_ids: Optional[Set[str]] = None,
) -> List[ExecutedTrade]:
    out: List[ExecutedTrade] = []
    for row in rows:
        t = parse_nse_inferred_tick(
            row, min_confidence=min_confidence, require_trade_like=require_trade_like
        )
        if t is None:
            continue
        if instrument_ids is not None and t.instrument_id not in instrument_ids:
            continue
        out.append(t)
    return out
