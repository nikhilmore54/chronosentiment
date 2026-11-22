#!/usr/bin/env python3
"""
RP-608-OFX — Crypto Binance aggTrade durable shadow journal (research-only).

Persists raw aggregate-trade events for the existing executed_flow library.
Does NOT alter baseline OF ±0.30, reverse-OF, plan books, or live authority (0%).

Env: RP608_OFX_CRYPTO_AGGTRADE_CAPTURE = 1/0 (default enabled).
"""

from __future__ import annotations

import json
import os
import threading
import time
import uuid
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, Mapping, Optional, Set

PROTOCOL_ID = "RP-608-OFX"
PHASE = "CRYPTO_AGGTRADE_DURABLE_SHADOW_CAPTURE"
DEFAULT_ENV_ENABLE = "RP608_OFX_CRYPTO_AGGTRADE_CAPTURE"
DEFAULT_SYMBOL = "BTCUSDT"
DEFAULT_WS_URL = "wss://stream.binance.com:9443/ws/btcusdt@aggTrade"


def ofx_crypto_aggtrade_capture_enabled(default: bool = True) -> bool:
    raw = os.environ.get(DEFAULT_ENV_ENABLE)
    if raw is None or str(raw).strip() == "":
        return bool(default)
    return str(raw).strip().lower() in {"1", "true", "yes", "on"}


def normalize_aggtrade_event(
    raw: Mapping[str, Any],
    *,
    receipt_utc_ms: Optional[int] = None,
    connection_session_id: Optional[str] = None,
) -> Optional[Dict[str, Any]]:
    """
    Normalize Binance aggTrade WS payload (or already-normalized tick) into a
    durable journal row. Preserves source fields for DIRECT aggressor mapping.
    """
    if not isinstance(raw, dict):
        return None

    symbol = str(raw.get("s") or raw.get("symbol") or DEFAULT_SYMBOL).upper()
    try:
        event_time_ms = int(raw.get("E") if raw.get("E") is not None else raw.get("event_time") or 0)
    except (TypeError, ValueError):
        event_time_ms = 0
    try:
        trade_time_ms = int(raw.get("T") if raw.get("T") is not None else raw.get("trade_time") or event_time_ms or 0)
    except (TypeError, ValueError):
        trade_time_ms = 0
    if trade_time_ms <= 0 and event_time_ms <= 0:
        return None

    price_raw = raw.get("p") if raw.get("p") is not None else raw.get("price")
    qty_raw = raw.get("q") if raw.get("q") is not None else raw.get("quantity", raw.get("volume"))
    try:
        price = float(price_raw)
        quantity = float(qty_raw)
    except (TypeError, ValueError):
        return None
    if quantity <= 0:
        return None

    if "m" in raw or "isBuyerMaker" in raw or "is_buyer_maker" in raw:
        raw_m = raw.get("m", raw.get("isBuyerMaker", raw.get("is_buyer_maker")))
        if isinstance(raw_m, str):
            is_buyer_maker = raw_m.strip().lower() in {"1", "true", "yes"}
        else:
            is_buyer_maker = bool(raw_m)
        m_present = True
    else:
        is_buyer_maker = None
        m_present = False

    agg_id = raw.get("a", raw.get("aggTradeId", raw.get("aggregate_trade_id")))
    try:
        agg_trade_id = int(agg_id) if agg_id is not None else None
    except (TypeError, ValueError):
        agg_trade_id = None

    first_id = raw.get("f", raw.get("first_trade_id"))
    last_id = raw.get("l", raw.get("last_trade_id"))
    recv = int(receipt_utc_ms) if receipt_utc_ms is not None else int(time.time() * 1000)

    # DIRECT mapping (documented): m=false → BUY aggressor; m=true → SELL aggressor
    if m_present:
        aggressor = "SELL" if is_buyer_maker else "BUY"
        side_class = "DIRECT"
    else:
        aggressor = "UNKNOWN"
        side_class = "UNAVAILABLE"

    return {
        "protocol_id": PROTOCOL_ID,
        "phase": PHASE,
        "record_type": "binance_aggtrade",
        "connection_session_id": connection_session_id,
        "symbol": symbol,
        "agg_trade_id": agg_trade_id,
        "first_trade_id": first_id,
        "last_trade_id": last_id,
        "price": price,
        "quantity": quantity,
        "is_buyer_maker": is_buyer_maker,
        "aggressor": aggressor,
        "side_classification": side_class,
        "event_time_ms": event_time_ms or None,
        "trade_time_ms": trade_time_ms or event_time_ms,
        "receipt_utc_ms": recv,
        "receipt_iso": datetime.fromtimestamp(recv / 1000.0, tz=timezone.utc).isoformat(),
        "quote_source": "binance_ws_aggTrade",
        "live_order_authority_pct": 0.0,
        "baseline_of_inert": True,
        "raw_ref": {
            "s": symbol,
            "a": agg_trade_id,
            "p": str(raw.get("p") if raw.get("p") is not None else price),
            "q": str(raw.get("q") if raw.get("q") is not None else quantity),
            "m": is_buyer_maker,
            "E": event_time_ms or None,
            "T": trade_time_ms or None,
        },
    }


class OfxCryptoAggTradeJournal:
    """Append-only research journal; OF-rule inert; separate from bookTicker baseline."""

    def __init__(
        self,
        root: Path,
        *,
        enabled: bool = True,
        symbol: str = DEFAULT_SYMBOL,
        flush_every: int = 32,
        max_seen_ids: int = 50_000,
    ):
        self.root = Path(root)
        self.root.mkdir(parents=True, exist_ok=True)
        self.path = self.root / "aggtrades.jsonl"
        self.coverage_path = self.root / "capture_coverage.json"
        self.enabled = bool(enabled)
        self.symbol = str(symbol).upper()
        self.ws_url = f"wss://stream.binance.com:9443/ws/{self.symbol.lower()}@aggTrade"
        self.flush_every = max(1, int(flush_every))
        self.max_seen_ids = max(1000, int(max_seen_ids))
        self.connection_session_id = str(uuid.uuid4())
        self._lock = threading.Lock()
        self._fh = None
        self._since_flush = 0
        self._seen_ids: Set[int] = set()
        self._last_trade_time_ms: Optional[int] = None
        self._last_agg_id: Optional[int] = None
        self._stats: Dict[str, Any] = {
            "protocol_id": PROTOCOL_ID,
            "phase": PHASE,
            "enabled": self.enabled,
            "symbol": self.symbol,
            "ws_url": self.ws_url,
            "connection_session_id": self.connection_session_id,
            "events_seen": 0,
            "events_written": 0,
            "duplicates_skipped": 0,
            "invalid_skipped": 0,
            "missing_maker_flag": 0,
            "buy_aggressor": 0,
            "sell_aggressor": 0,
            "agg_id_backward": 0,
            "trade_time_non_monotonic": 0,
            "max_gap_ms_observed": 0,
            "last_trade_time_ms": None,
            "last_receipt_utc": None,
            "path": str(self.path),
            "baseline_of_inert": True,
            "live_order_authority_pct": 0.0,
        }
        if self.enabled:
            self._fh = open(self.path, "a", encoding="utf-8")

    def close(self) -> None:
        with self._lock:
            if self._fh is not None:
                try:
                    self._fh.flush()
                    self._fh.close()
                except Exception:
                    pass
                self._fh = None
            self._write_coverage_unlocked()

    def on_raw_event(self, raw: Mapping[str, Any], *, receipt_utc_ms: Optional[int] = None) -> Optional[Dict[str, Any]]:
        """Ingest one WS message. Never raises into the feed path."""
        if not self.enabled:
            return None
        try:
            return self._on_raw_inner(raw, receipt_utc_ms=receipt_utc_ms)
        except Exception:
            return None

    def _on_raw_inner(
        self, raw: Mapping[str, Any], *, receipt_utc_ms: Optional[int] = None
    ) -> Optional[Dict[str, Any]]:
        with self._lock:
            self._stats["events_seen"] += 1
            row = normalize_aggtrade_event(
                raw,
                receipt_utc_ms=receipt_utc_ms,
                connection_session_id=self.connection_session_id,
            )
            if row is None:
                self._stats["invalid_skipped"] += 1
                return None

            if row["symbol"] and row["symbol"] != self.symbol:
                # Allow multi-symbol later; default journal is single-instrument
                pass

            agg_id = row.get("agg_trade_id")
            if agg_id is not None:
                if agg_id in self._seen_ids:
                    self._stats["duplicates_skipped"] += 1
                    return None
                if self._last_agg_id is not None and int(agg_id) < int(self._last_agg_id):
                    self._stats["agg_id_backward"] += 1
                self._seen_ids.add(int(agg_id))
                if len(self._seen_ids) > self.max_seen_ids:
                    # drop arbitrary half to bound memory (ids are roughly increasing)
                    keep = sorted(self._seen_ids)[- self.max_seen_ids // 2 :]
                    self._seen_ids = set(keep)
                self._last_agg_id = int(agg_id)

            tt = int(row.get("trade_time_ms") or 0)
            if self._last_trade_time_ms is not None and tt > 0:
                if tt < self._last_trade_time_ms:
                    self._stats["trade_time_non_monotonic"] += 1
                else:
                    gap = tt - self._last_trade_time_ms
                    if gap > int(self._stats["max_gap_ms_observed"] or 0):
                        self._stats["max_gap_ms_observed"] = gap
            if tt > 0:
                self._last_trade_time_ms = tt
                self._stats["last_trade_time_ms"] = tt

            if row.get("side_classification") != "DIRECT":
                self._stats["missing_maker_flag"] += 1
            elif row.get("aggressor") == "BUY":
                self._stats["buy_aggressor"] += 1
            elif row.get("aggressor") == "SELL":
                self._stats["sell_aggressor"] += 1

            if self._fh is not None:
                self._fh.write(json.dumps(row, separators=(",", ":")) + "\n")
                self._since_flush += 1
                self._stats["events_written"] += 1
                self._stats["last_receipt_utc"] = row.get("receipt_iso")
                if self._since_flush >= self.flush_every:
                    self._fh.flush()
                    self._since_flush = 0
                    self._write_coverage_unlocked()
            return row

    def _write_coverage_unlocked(self) -> None:
        payload = dict(self._stats)
        payload["updated_utc"] = datetime.now(timezone.utc).isoformat()
        tmp = self.coverage_path.with_suffix(".tmp")
        with open(tmp, "w", encoding="utf-8") as f:
            json.dump(payload, f, indent=2)
            f.flush()
            os.fsync(f.fileno())
        os.replace(tmp, self.coverage_path)
