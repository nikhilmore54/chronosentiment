#!/usr/bin/env python3
"""
RP-608-OFX — Crypto volume/aggressor-direction mark from durable aggTrade journal.

Research-only. Does NOT replace baseline L1 OF_SUP / ±0.30 / reverse-OF paper.
Writes ofx_volume_flow_snapshot.json beside the aggTrade capture root.
"""

from __future__ import annotations

import json
import os
import time
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, List, Optional

WORKSPACE = Path(__file__).resolve().parent.parent
import sys

sys.path.insert(0, str(WORKSPACE))

from research.rp608_ofx.executed_flow import (  # noqa: E402
    DEFAULT_WINDOW_MS,
    aggregate_executed_flow,
    trades_from_aggtrade_records,
)
from research.rp608_ofx.executed_flow.volume_mark import (  # noqa: E402
    OFX_VOLUME_IMBALANCE_THRESHOLD,
    inspect_ofx_volume_book,
)

SNAPSHOT_NAME = "ofx_volume_flow_snapshot.json"
PROTOCOL_ID = "RP-608-OFX"
PHASE = "CRYPTO_VOLUME_FLOW_MARK"


def _load_recent_journal_rows(path: Path, *, max_rows: int = 20_000) -> List[Dict[str, Any]]:
    if not path.exists():
        return []
    # Efficient-ish tail: read all if small; for large files read last N lines
    try:
        size = path.stat().st_size
    except OSError:
        return []
    if size <= 8_000_000:
        lines = path.read_text(encoding="utf-8").splitlines()
    else:
        # last ~2MB
        with path.open("rb") as f:
            f.seek(max(0, size - 2_000_000))
            chunk = f.read().decode("utf-8", errors="ignore")
        lines = chunk.splitlines()[1:]  # drop partial first line
    rows: List[Dict[str, Any]] = []
    for line in lines[-max_rows:]:
        line = line.strip()
        if not line:
            continue
        try:
            rows.append(json.loads(line))
        except json.JSONDecodeError:
            continue
    return rows


def _rows_to_aggtrade_shaped(rows: List[Dict[str, Any]], symbol: str) -> List[Dict[str, Any]]:
    shaped: List[Dict[str, Any]] = []
    for r in rows:
        raw = r.get("raw_ref") if isinstance(r.get("raw_ref"), dict) else {}
        shaped.append(
            {
                "s": r.get("symbol") or raw.get("s") or symbol,
                "a": r.get("agg_trade_id") if r.get("agg_trade_id") is not None else raw.get("a"),
                "p": r.get("price") if r.get("price") is not None else raw.get("p"),
                "q": r.get("quantity") if r.get("quantity") is not None else raw.get("q"),
                "m": r.get("is_buyer_maker") if r.get("is_buyer_maker") is not None else raw.get("m"),
                "E": r.get("event_time_ms") or raw.get("E"),
                "T": r.get("trade_time_ms") or raw.get("T"),
            }
        )
    return shaped


def build_ofx_volume_flow_snapshot(
    *,
    journal_path: Path,
    symbol: str = "BTCUSDT",
    window_ms: int = DEFAULT_WINDOW_MS,
    now_ms: Optional[int] = None,
    primary_side: str = "LONG",
    imbalance_threshold: float = OFX_VOLUME_IMBALANCE_THRESHOLD,
) -> Dict[str, Any]:
    now = int(now_ms) if now_ms is not None else int(time.time() * 1000)
    window_ms = max(1, int(window_ms))
    t0 = now - window_ms
    rows = _load_recent_journal_rows(journal_path)
    trades = trades_from_aggtrade_records(
        _rows_to_aggtrade_shaped(rows, symbol), default_symbol=symbol
    )
    result = aggregate_executed_flow(
        trades,
        instrument_id=symbol.upper(),
        window_start_ms=t0,
        window_end_ms=now + 1,
        source="binance_aggTrade_journal",
    )
    insp = inspect_ofx_volume_book(
        result, primary_side=primary_side, imbalance_threshold=imbalance_threshold
    )
    # Dominant aggressor label for observability (not institutional)
    if result.total_classified_qty > 0:
        if result.signed_delta_qty > 0:
            dominant = "BUY_AGGRESSOR"
        elif result.signed_delta_qty < 0:
            dominant = "SELL_AGGRESSOR"
        else:
            dominant = "BALANCED"
    else:
        dominant = "UNAVAILABLE"

    return {
        "protocol_id": PROTOCOL_ID,
        "phase": PHASE,
        "construct": "EXECUTED_AGGRESSOR_VOLUME",
        "baseline_of_inert": True,
        "live_order_authority_pct": 0.0,
        "symbol": symbol.upper(),
        "as_of_utc_ms": now,
        "as_of_iso": datetime.fromtimestamp(now / 1000.0, tz=timezone.utc).isoformat(),
        "window_ms": window_ms,
        "window_start_ms": t0,
        "window_end_ms": now + 1,
        "journal_path": str(journal_path),
        "journal_rows_loaded": len(rows),
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
            "Volume/direction OFX mark from executed aggressor qty. "
            "Independent of candle colour and baseline L1 OF_SUP."
        ),
    }


def write_ofx_volume_flow_snapshot(capture_root: Path, snapshot: Dict[str, Any]) -> Path:
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


def refresh_ofx_volume_flow_mark(
    capture_root: Path,
    *,
    symbol: str = "BTCUSDT",
    window_ms: int = DEFAULT_WINDOW_MS,
    now_ms: Optional[int] = None,
) -> Dict[str, Any]:
    journal = Path(capture_root) / "aggtrades.jsonl"
    snap = build_ofx_volume_flow_snapshot(
        journal_path=journal,
        symbol=symbol,
        window_ms=window_ms,
        now_ms=now_ms,
    )
    write_ofx_volume_flow_snapshot(capture_root, snap)
    return snap
