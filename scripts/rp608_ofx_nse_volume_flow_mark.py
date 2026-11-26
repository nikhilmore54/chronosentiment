#!/usr/bin/env python3
"""
RP-608-OFX — NSE volume/aggressor-direction mark from Stage 0D inferred ticks.

DERIVED_VALIDATED only (never DIRECT). Research-only.
Does NOT replace baseline L1 OF_SUP / ±0.30 / NSE OF paper.

Uses an in-memory ring buffer (12GB durable journal is not tailed live).
"""

from __future__ import annotations

import json
import os
import threading
import time
from collections import defaultdict, deque
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Deque, Dict, List, Optional

WORKSPACE = Path(__file__).resolve().parent.parent
import sys

sys.path.insert(0, str(WORKSPACE))

from research.rp608_ofx.executed_flow import (  # noqa: E402
    DEFAULT_WINDOW_MS,
    aggregate_executed_flow,
)
from research.rp608_ofx.executed_flow.nse_inferred import (  # noqa: E402
    trades_from_nse_inferred_ticks,
)
from research.rp608_ofx.executed_flow.volume_mark import (  # noqa: E402
    OFX_VOLUME_IMBALANCE_THRESHOLD,
    inspect_ofx_volume_book,
)

SNAPSHOT_NAME = "ofx_nse_volume_flow_snapshot.json"
PROTOCOL_ID = "RP-608-OFX"
PHASE = "NSE_VOLUME_FLOW_MARK_DERIVED"
MAX_ROWS_PER_SYMBOL = 4_000


def canonicalize_nse_symbol(symbol: Optional[str]) -> str:
    """Stable per-scrip key so volume match never pools across symbols."""
    s = str(symbol or "").strip().upper()
    if not s:
        return ""
    if not s.endswith(".NS") and "." not in s:
        s = f"{s}.NS"
    return s


class OfxNseVolumeFlowBuffer:
    """Thread-safe recent inferred-tick buffer — keyed strictly per symbol."""

    def __init__(self, *, max_rows_per_symbol: int = MAX_ROWS_PER_SYMBOL):
        self._lock = threading.Lock()
        self._by_sym: Dict[str, Deque[Dict[str, Any]]] = defaultdict(
            lambda: deque(maxlen=int(max_rows_per_symbol))
        )
        self._n_ingested = 0

    def ingest_row(self, row: Dict[str, Any]) -> None:
        if not isinstance(row, dict):
            return
        if not bool(row.get("trade_like")):
            return
        sym = canonicalize_nse_symbol(row.get("symbol"))
        if not sym:
            return
        # Stamp canonical symbol so downstream instrument_id matches the open book key
        stamped = dict(row)
        stamped["symbol"] = sym
        with self._lock:
            self._by_sym[sym].append(stamped)
            self._n_ingested += 1

    def rows_for(self, symbol: str, *, since_ms: Optional[int] = None) -> List[Dict[str, Any]]:
        key = canonicalize_nse_symbol(symbol)
        if not key:
            return []
        with self._lock:
            rows = list(self._by_sym.get(key, ()))
        if since_ms is None:
            return rows
        out: List[Dict[str, Any]] = []
        for r in rows:
            # Belt-and-suspenders: never leak another symbol's row
            if canonicalize_nse_symbol(r.get("symbol")) != key:
                continue
            ts = r.get("receipt_utc_ms") or r.get("exchange_timestamp_utc_ms") or 0
            try:
                if int(ts) >= int(since_ms):
                    out.append(r)
            except (TypeError, ValueError):
                continue
        return out

    @property
    def n_ingested(self) -> int:
        with self._lock:
            return int(self._n_ingested)


_GLOBAL_BUFFER = OfxNseVolumeFlowBuffer()


def global_nse_volume_flow_buffer() -> OfxNseVolumeFlowBuffer:
    return _GLOBAL_BUFFER


def build_ofx_nse_volume_flow_snapshot(
    *,
    symbol: str,
    buffer: Optional[OfxNseVolumeFlowBuffer] = None,
    window_ms: int = DEFAULT_WINDOW_MS,
    now_ms: Optional[int] = None,
    primary_side: str = "LONG",
    imbalance_threshold: float = OFX_VOLUME_IMBALANCE_THRESHOLD,
) -> Dict[str, Any]:
    now = int(now_ms) if now_ms is not None else int(time.time() * 1000)
    window_ms = max(1, int(window_ms))
    t0 = now - window_ms
    sym = canonicalize_nse_symbol(symbol)
    buf = buffer or _GLOBAL_BUFFER
    rows = buf.rows_for(sym, since_ms=t0)
    trades = trades_from_nse_inferred_ticks(
        rows,
        min_confidence="MEDIUM",
        require_trade_like=True,
        instrument_ids={sym} if sym else None,
    )
    result = aggregate_executed_flow(
        trades,
        instrument_id=sym,
        window_start_ms=t0,
        window_end_ms=now + 1,
        source="nse_ofx_stage0d_inferred",
    )
    # Force provenance label for consumers
    result.side_classification_mode = "DERIVED_VALIDATED"
    insp = inspect_ofx_volume_book(
        result, primary_side=primary_side, imbalance_threshold=imbalance_threshold
    )
    if result.total_classified_qty > 0:
        if result.signed_delta_qty > 0:
            dominant = "BUY_AGGRESSOR_DERIVED"
        elif result.signed_delta_qty < 0:
            dominant = "SELL_AGGRESSOR_DERIVED"
        else:
            dominant = "BALANCED"
    else:
        dominant = "UNAVAILABLE"

    return {
        "protocol_id": PROTOCOL_ID,
        "phase": PHASE,
        "construct": "DERIVED_VALIDATED_AGGRESSOR_VOLUME",
        "side_classification_mode": "DERIVED_VALIDATED",
        "baseline_of_inert": True,
        "live_order_authority_pct": 0.0,
        "symbol": sym,
        "as_of_utc_ms": now,
        "as_of_iso": datetime.fromtimestamp(now / 1000.0, tz=timezone.utc).isoformat(),
        "window_ms": window_ms,
        "window_start_ms": t0,
        "window_end_ms": now + 1,
        "buffer_rows_in_window": len(rows),
        "buffer_n_ingested_total": buf.n_ingested,
        "dominant_aggressor": dominant,
        "signed_delta_qty": result.signed_delta_qty,
        "buy_aggressor_qty": result.buy_aggressor_qty,
        "sell_aggressor_qty": result.sell_aggressor_qty,
        "total_classified_qty": result.total_classified_qty,
        "classification_coverage": result.classification_coverage,
        "data_quality": result.data_quality,
        "status": result.status,
        "inspection": insp,
        "note": (
            "NSE OFX volume mark from Stage 0D inferred aggressor qty (DERIVED_VALIDATED). "
            "Independent of candle colour and baseline L1 OF_SUP."
        ),
    }


def write_ofx_nse_volume_flow_snapshot(capture_root: Path, snapshot: Dict[str, Any]) -> Path:
    capture_root = Path(capture_root)
    capture_root.mkdir(parents=True, exist_ok=True)
    out = capture_root / SNAPSHOT_NAME
    tmp = out.with_suffix(".tmp")
    with open(tmp, "w", encoding="utf-8") as f:
        json.dump(snapshot, f, indent=2)
        f.flush()
        os.fsync(f.fileno())
    os.replace(tmp, out)
    return out


def refresh_ofx_nse_volume_flow_mark(
    capture_root: Path,
    *,
    symbol: str,
    buffer: Optional[OfxNseVolumeFlowBuffer] = None,
    window_ms: int = DEFAULT_WINDOW_MS,
    now_ms: Optional[int] = None,
) -> Dict[str, Any]:
    snap = build_ofx_nse_volume_flow_snapshot(
        symbol=symbol,
        buffer=buffer,
        window_ms=window_ms,
        now_ms=now_ms,
    )
    write_ofx_nse_volume_flow_snapshot(capture_root, snap)
    return snap
