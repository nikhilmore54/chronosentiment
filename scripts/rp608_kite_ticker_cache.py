#!/usr/bin/env python3
"""
RP-608 NSE — continuous Kite WebSocket L1 quote cache (0.0% order authority).

Uses the same binary full-mode protocol as scripts/run_rp608_nse_live_probe.py
(wss://ws.kite.trade), not the kiteconnect SDK.

Limits respected: 1 connection, ≤119 instruments (<< 3000/connection).
"""

from __future__ import annotations

import asyncio
import json
import struct
import threading
import time
from typing import Any, Dict, List, Optional, Sequence

import websockets


def decode_full_mode_packet(pkt: bytes) -> Optional[Dict[str, Any]]:
    """Decode one Kite full-mode quote packet (184 bytes)."""
    if len(pkt) < 184:
        return None
    tok = struct.unpack(">I", pkt[0:4])[0]
    ltp = struct.unpack(">I", pkt[4:8])[0] / 100.0
    vol = struct.unpack(">I", pkt[16:20])[0]
    exch_ts_sec = struct.unpack(">I", pkt[60:64])[0]

    buy_depth = []
    for i in range(5):
        off = 64 + (i * 12)
        b_qty = struct.unpack(">I", pkt[off : off + 4])[0]
        b_price = struct.unpack(">I", pkt[off + 4 : off + 8])[0] / 100.0
        buy_depth.append({"price": b_price, "quantity": b_qty})

    sell_depth = []
    for i in range(5):
        off = 124 + (i * 12)
        s_qty = struct.unpack(">I", pkt[off : off + 4])[0]
        s_price = struct.unpack(">I", pkt[off + 4 : off + 8])[0] / 100.0
        sell_depth.append({"price": s_price, "quantity": s_qty})

    best_bid = float(buy_depth[0]["price"])
    best_ask = float(sell_depth[0]["price"])
    bid_qty = float(buy_depth[0]["quantity"])
    ask_qty = float(sell_depth[0]["quantity"])
    return {
        "instrument_token": int(tok),
        "last_price": float(ltp),
        "volume": float(vol),
        "exchange_timestamp_sec": int(exch_ts_sec),
        "best_bid": best_bid,
        "best_ask": best_ask,
        "bid_qty": bid_qty,
        "ask_qty": ask_qty,
    }


class KiteTickerQuoteCache:
    """Background KiteTicker-style stream → latest L1 by instrument_token."""

    def __init__(
        self,
        *,
        api_key: str,
        access_token: str,
        instrument_tokens: Sequence[int],
        max_quote_age_ms: float = 60_000.0,
    ):
        self.api_key = api_key
        self.access_token = access_token
        self.tokens = sorted({int(t) for t in instrument_tokens if int(t) > 0})
        self.max_quote_age_ms = float(max_quote_age_ms)
        self._latest: Dict[int, Dict[str, Any]] = {}
        self._lock = threading.Lock()
        self._stop = threading.Event()
        self._thread: Optional[threading.Thread] = None
        self._connected = False
        self._last_packet_mono: float = 0.0
        self._packets: int = 0
        self._unique_tokens: int = 0

    @property
    def connected(self) -> bool:
        return self._connected

    def stats(self) -> Dict[str, Any]:
        with self._lock:
            n = len(self._latest)
        return {
            "connected": self._connected,
            "subscribed_tokens": len(self.tokens),
            "cached_tokens": n,
            "packets": self._packets,
            "last_packet_age_sec": (
                None
                if self._last_packet_mono <= 0
                else round(time.monotonic() - self._last_packet_mono, 2)
            ),
        }

    def start(self) -> None:
        if not self.api_key or not self.access_token or not self.tokens:
            return
        if self._thread and self._thread.is_alive():
            return
        self._stop.clear()
        self._thread = threading.Thread(
            target=self._thread_main,
            name="rp608-kite-ticker",
            daemon=True,
        )
        self._thread.start()

    def stop(self) -> None:
        self._stop.set()
        if self._thread and self._thread.is_alive():
            self._thread.join(timeout=5.0)
        self._connected = False

    def get_l1(self, instrument_token: int) -> Optional[Dict[str, Any]]:
        with self._lock:
            row = self._latest.get(int(instrument_token))
            if not row:
                return None
            out = dict(row)
        age = out.get("quote_age_ms")
        if age is not None and float(age) > self.max_quote_age_ms:
            return None
        return out

    def _thread_main(self) -> None:
        while not self._stop.is_set():
            try:
                asyncio.run(self._run_session())
            except Exception:
                self._connected = False
            if self._stop.is_set():
                break
            # reconnect backoff
            self._stop.wait(3.0)

    async def _run_session(self) -> None:
        url = f"wss://ws.kite.trade?api_key={self.api_key}&access_token={self.access_token}"
        async with websockets.connect(url, ping_interval=15, ping_timeout=10) as ws:
            await ws.send(json.dumps({"a": "subscribe", "v": self.tokens}))
            await ws.send(json.dumps({"a": "mode", "v": ["full", self.tokens]}))
            self._connected = True
            while not self._stop.is_set():
                try:
                    msg = await asyncio.wait_for(ws.recv(), timeout=2.0)
                except asyncio.TimeoutError:
                    continue
                if isinstance(msg, str):
                    continue
                if not isinstance(msg, bytes) or len(msg) < 2:
                    continue
                self._ingest_binary_frame(msg)

    def _ingest_binary_frame(self, msg: bytes) -> None:
        num_pkts = struct.unpack(">H", msg[:2])[0]
        offset = 2
        receipt_ms = int(time.time() * 1000)
        updated = False
        for _ in range(num_pkts):
            if offset + 2 > len(msg):
                break
            pkt_len = struct.unpack(">H", msg[offset : offset + 2])[0]
            offset += 2
            pkt = msg[offset : offset + pkt_len]
            offset += pkt_len
            if pkt_len != 184:
                continue
            tick = decode_full_mode_packet(pkt)
            if not tick:
                continue
            tok = int(tick["instrument_token"])
            exch_ms = int(tick["exchange_timestamp_sec"]) * 1000
            tick["receipt_utc_ms"] = receipt_ms
            tick["quote_age_ms"] = max(0.0, float(receipt_ms - exch_ms))
            tick["quote_source"] = "kite_ws_full"
            with self._lock:
                self._latest[tok] = tick
                self._unique_tokens = len(self._latest)
            updated = True
        if updated:
            self._packets += 1
            self._last_packet_mono = time.monotonic()
