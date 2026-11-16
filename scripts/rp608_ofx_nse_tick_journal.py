#!/usr/bin/env python3
"""
RP-608-OFX Stage 0D — NSE inferred-aggressor tick journal (research-only).

Tees decoded Kite full-mode ticks from the existing single WebSocket cache.
Does NOT alter OF ±0.30, reverse-OF, Estimand A, or live order authority (0%).

Labels are inferred candidates with confidence — not exchange aggressor flags.
"""

from __future__ import annotations

import json
import os
import threading
import time
import uuid
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Callable, Dict, List, Mapping, Optional, Tuple

PROTOCOL_ID = "RP-608-OFX"
PHASE = "STAGE_0D_NSE_INFERRED_AGGRESSOR_TICK_CAPTURE"
DEFAULT_ENV_ENABLE = "RP608_OFX_NSE_TICK_CAPTURE"


def ofx_tick_capture_enabled(default: bool = True) -> bool:
    raw = os.environ.get(DEFAULT_ENV_ENABLE)
    if raw is None or str(raw).strip() == "":
        return bool(default)
    return str(raw).strip().lower() in {"1", "true", "yes", "on"}


def _level5(levels: Any) -> list:
    out = []
    if not isinstance(levels, list):
        return [{"price": 0.0, "quantity": 0.0} for _ in range(5)]
    for i in range(5):
        row = levels[i] if i < len(levels) and isinstance(levels[i], dict) else {}
        try:
            px = float(row.get("price") or 0.0)
        except (TypeError, ValueError):
            px = 0.0
        try:
            qty = float(row.get("quantity") or 0.0)
        except (TypeError, ValueError):
            qty = 0.0
        out.append({"price": round(px, 4), "quantity": round(qty, 4)})
    return out


def classify_trade_like(
    *,
    prev: Optional[Mapping[str, Any]],
    ltp: float,
    ltq: float,
    ltt_sec: int,
    volume: float,
) -> Tuple[str, bool]:
    """Return (update_class, is_trade_like)."""
    if prev is None:
        return "FIRST_OBSERVATION", True
    try:
        p_vol = float(prev.get("volume") or 0.0)
    except (TypeError, ValueError):
        p_vol = 0.0
    p_ltp = float(prev.get("last_price") or 0.0)
    p_ltq = float(prev.get("last_traded_quantity") or 0.0)
    try:
        p_ltt = int(prev.get("last_trade_time_sec") or 0)
    except (TypeError, ValueError):
        p_ltt = 0

    vol_up = volume > p_vol
    lt_changed = (ltp, ltq, ltt_sec) != (p_ltp, p_ltq, p_ltt)
    if vol_up and lt_changed:
        return "TRADE_LIKE_VOLUME_AND_LT_CHANGE", True
    if vol_up:
        return "TRADE_LIKE_VOLUME_INCREASE", True
    if lt_changed:
        return "TRADE_LIKE_LT_CHANGE_NO_VOLUME", True
    return "QUOTE_REPEAT_SAME_LT_AND_VOLUME", False


def infer_aggressor_vs_quote(
    *,
    ltp: float,
    bid: float,
    ask: float,
    quote_age_ms: Optional[float],
    quote_source: str,
) -> Dict[str, Any]:
    """Same-packet or prior-quote LTP vs bid/ask inference (provisional)."""
    out: Dict[str, Any] = {
        "label": "UNAVAILABLE",
        "reason": "uninitialized",
        "confidence": "NONE",
        "quote_source": quote_source,
        "quote_age_ms": quote_age_ms,
        "bid": bid,
        "ask": ask,
        "ltp": ltp,
    }
    if bid is None or ask is None or bid <= 0 or ask <= 0:
        out["reason"] = "missing_quote"
        return out
    if bid >= ask:
        out["reason"] = "crossed_or_locked"
        return out
    if quote_age_ms is not None and float(quote_age_ms) > 60_000.0:
        out["label"] = "UNAVAILABLE"
        out["reason"] = "stale_quote"
        out["confidence"] = "NONE"
        return out
    if ltp >= ask:
        out["label"] = "BUY_INITIATED_CANDIDATE"
        out["reason"] = "ltp_ge_ask"
        out["confidence"] = "LOW" if quote_source == "same_packet" else "MEDIUM"
        return out
    if ltp <= bid:
        out["label"] = "SELL_INITIATED_CANDIDATE"
        out["reason"] = "ltp_le_bid"
        out["confidence"] = "LOW" if quote_source == "same_packet" else "MEDIUM"
        return out
    out["label"] = "AMBIGUOUS"
    out["reason"] = "inside_spread"
    out["confidence"] = "LOW"
    return out


class OfxNseInferredAggressorTickJournal:
    """Append-only research journal; OF-rule inert; single-connection tee consumer."""

    def __init__(
        self,
        root: Path,
        *,
        enabled: bool = True,
        token_to_symbol: Optional[Mapping[int, str]] = None,
        flush_every: int = 32,
    ):
        self.root = Path(root)
        self.root.mkdir(parents=True, exist_ok=True)
        self.path = self.root / "inferred_aggressor_ticks.jsonl"
        self.coverage_path = self.root / "capture_coverage.json"
        self.enabled = bool(enabled)
        self.token_to_symbol = {int(k): str(v) for k, v in (token_to_symbol or {}).items()}
        self.flush_every = max(1, int(flush_every))
        self.connection_session_id = str(uuid.uuid4())
        self._lock = threading.Lock()
        self._prev: Dict[int, Dict[str, Any]] = {}
        self._fh = None
        self._since_flush = 0
        self._row_listeners: List[Callable[[Dict[str, Any]], None]] = []
        self._stats: Dict[str, Any] = {
            "protocol_id": PROTOCOL_ID,
            "phase": PHASE,
            "enabled": self.enabled,
            "connection_session_id": self.connection_session_id,
            "ticks_seen": 0,
            "ticks_written": 0,
            "trade_like": 0,
            "quote_repeat": 0,
            "infer_buy": 0,
            "infer_sell": 0,
            "infer_ambiguous": 0,
            "infer_unavailable": 0,
            "missing_ltq_field": 0,
            "exch_ts_non_monotonic": 0,
            "rows_missing_level_qty": 0,
            "last_receipt_utc": None,
            "path": str(self.path),
        }
        if self.enabled:
            self._fh = open(self.path, "a", encoding="utf-8")

    def set_token_map(self, token_to_symbol: Mapping[int, str]) -> None:
        with self._lock:
            self.token_to_symbol = {int(k): str(v) for k, v in token_to_symbol.items()}

    def register_row_listener(self, fn: Callable[[Dict[str, Any]], None]) -> None:
        """Research tee: called with each inferred row (never raises into feed)."""
        if callable(fn):
            self._row_listeners.append(fn)

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

    def on_tick(self, tick: Mapping[str, Any]) -> None:
        """Callback from KiteTickerQuoteCache (same WS). Never raises into feed path."""
        if not self.enabled:
            return
        try:
            self._on_tick_inner(tick)
        except Exception:
            # Fail closed for research path — never disrupt quote cache / OF.
            return

    def _on_tick_inner(self, tick: Mapping[str, Any]) -> None:
        tok = int(tick.get("instrument_token") or 0)
        if tok <= 0:
            return
        try:
            ltp = float(tick.get("last_price") or 0.0)
        except (TypeError, ValueError):
            ltp = 0.0
        try:
            ltq = float(tick.get("last_traded_quantity") or 0.0)
        except (TypeError, ValueError):
            ltq = 0.0
        try:
            ltt = int(tick.get("last_trade_time_sec") or 0)
        except (TypeError, ValueError):
            ltt = 0
        try:
            volume = float(tick.get("volume") or 0.0)
        except (TypeError, ValueError):
            volume = 0.0
        try:
            exch = int(tick.get("exchange_timestamp_sec") or 0)
        except (TypeError, ValueError):
            exch = 0
        try:
            receipt = int(tick.get("receipt_utc_ms") or time.time() * 1000)
        except (TypeError, ValueError):
            receipt = int(time.time() * 1000)
        try:
            quote_age = float(tick["quote_age_ms"]) if tick.get("quote_age_ms") is not None else None
        except (TypeError, ValueError):
            quote_age = None

        bid = float(tick.get("best_bid") or 0.0)
        ask = float(tick.get("best_ask") or 0.0)
        depth_buy = _level5(tick.get("depth_buy"))
        depth_sell = _level5(tick.get("depth_sell"))
        bid_nz = sum(1 for lv in depth_buy if lv["quantity"] > 0)
        ask_nz = sum(1 for lv in depth_sell if lv["quantity"] > 0)

        with self._lock:
            self._stats["ticks_seen"] += 1
            if "last_traded_quantity" not in tick:
                self._stats["missing_ltq_field"] += 1
            prev = self._prev.get(tok)
            update_class, trade_like = classify_trade_like(
                prev=prev, ltp=ltp, ltq=ltq, ltt_sec=ltt, volume=volume
            )
            if trade_like:
                self._stats["trade_like"] += 1
            else:
                self._stats["quote_repeat"] += 1

            if prev is not None:
                try:
                    prev_exch = int(prev.get("exchange_timestamp_sec") or 0)
                except (TypeError, ValueError):
                    prev_exch = 0
                if exch and prev_exch and exch < prev_exch:
                    self._stats["exch_ts_non_monotonic"] += 1

            same = infer_aggressor_vs_quote(
                ltp=ltp, bid=bid, ask=ask, quote_age_ms=quote_age, quote_source="same_packet"
            )
            prior_inf: Optional[Dict[str, Any]] = None
            if prev is not None and trade_like:
                prior_inf = infer_aggressor_vs_quote(
                    ltp=ltp,
                    bid=float(prev.get("best_bid") or 0.0),
                    ask=float(prev.get("best_ask") or 0.0),
                    quote_age_ms=None,
                    quote_source="prior_packet_quote",
                )

            primary = prior_inf if (prior_inf and prior_inf.get("label") != "UNAVAILABLE") else same
            lab = str(primary.get("label") or "UNAVAILABLE")
            if lab.startswith("BUY"):
                self._stats["infer_buy"] += 1
            elif lab.startswith("SELL"):
                self._stats["infer_sell"] += 1
            elif lab == "AMBIGUOUS":
                self._stats["infer_ambiguous"] += 1
            else:
                self._stats["infer_unavailable"] += 1
            if bid_nz < 5 or ask_nz < 5:
                self._stats["rows_missing_level_qty"] += 1

            row = {
                "protocol_id": PROTOCOL_ID,
                "phase": PHASE,
                "record_type": "inferred_aggressor_tick",
                "connection_session_id": self.connection_session_id,
                "symbol": self.token_to_symbol.get(tok),
                "instrument_token": tok,
                "receipt_utc_ms": receipt,
                "receipt_iso": datetime.fromtimestamp(receipt / 1000.0, tz=timezone.utc).isoformat(),
                "exchange_timestamp_sec": exch or None,
                "exchange_timestamp_utc_ms": (exch * 1000) if exch else None,
                "last_trade_time_sec": ltt or None,
                "quote_age_ms": quote_age,
                "last_price": ltp,
                "last_traded_quantity": ltq,
                "average_price": float(tick.get("average_price") or 0.0) or None,
                "volume": volume,
                "best_bid": bid,
                "best_ask": ask,
                "bid_qty": float(tick.get("bid_qty") or 0.0),
                "ask_qty": float(tick.get("ask_qty") or 0.0),
                "depth_buy": depth_buy,
                "depth_sell": depth_sell,
                "bid_levels_qty_nonzero": bid_nz,
                "ask_levels_qty_nonzero": ask_nz,
                "update_class": update_class,
                "trade_like": bool(trade_like),
                "aggressor_inference_same_packet": same,
                "aggressor_inference_prior_quote": prior_inf,
                "aggressor_inference_primary": primary,
                "inference_note": (
                    "Inferred candidate only — Kite does not supply exchange aggressor side. "
                    "Prior-quote preferred when available; same-packet is LOW confidence."
                ),
                "quote_source": tick.get("quote_source") or "kite_ws_full",
                "live_order_authority_pct": 0.0,
            }

            if self._fh is not None:
                self._fh.write(json.dumps(row, separators=(",", ":")) + "\n")
                self._since_flush += 1
                self._stats["ticks_written"] += 1
                if self._since_flush >= self.flush_every:
                    self._fh.flush()
                    self._since_flush = 0
                    self._write_coverage_unlocked()

            for listener in list(self._row_listeners):
                try:
                    listener(row)
                except Exception:
                    pass

            self._prev[tok] = {
                "last_price": ltp,
                "last_traded_quantity": ltq,
                "last_trade_time_sec": ltt,
                "volume": volume,
                "best_bid": bid,
                "best_ask": ask,
                "exchange_timestamp_sec": exch,
            }
            self._stats["last_receipt_utc"] = row["receipt_iso"]

    def _write_coverage_unlocked(self) -> None:
        self._stats["updated_utc"] = datetime.now(timezone.utc).isoformat()
        try:
            self.coverage_path.write_text(json.dumps(self._stats, indent=2) + "\n", encoding="utf-8")
        except Exception:
            pass
