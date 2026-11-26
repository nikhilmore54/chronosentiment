#!/usr/bin/env python3
"""
scripts/run_rp608_event_daemon.py

RP-608-EVENT-001: Parallel Multi-Timescale Tick-Level Shadow Decision Daemon.
Strictly Read-Only (0.0% Order Authority) with Dedicated Isolated Ledger Output.

Connects concurrently to:
  1. wss://stream.binance.com:9443/ws/btcusdt@kline_15m (Completed 15m Candles to maintain 150-bar rolling buffer)
  2. wss://stream.binance.com:9443/ws/btcusdt@bookTicker (Continuous Top-of-Book Quotes)

Evaluates 4 Genuinely Independent Lookback Configurations in Parallel:
  - Candidate A: 16 bars (4 hours)
  - Candidate B: 32 bars (8 hours)
  - Candidate C: 64 bars (16 hours)
  - Candidate D: 120 bars (30 hours)

Isolation Invariants:
  - Output restricted exclusively to: live_capture/shadow_validation/crypto/event_driven/
  - Zero access or modification to canonical observer files (rp608_decision_cards.jsonl, rp608_checkpoint.json)
  - Strictly 0.0% Order Authority (Read-Only public WebSockets; no exchange credentials)
  - Diagnostic telemetry written to event_telemetry.jsonl; recommendation ledgers free of routine NO_TRADE spam.
"""

from __future__ import annotations

import asyncio
import json
import os
import signal
import ssl
import sys
import time
import urllib.request
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, List, Optional

import websockets

WORKSPACE_DIR = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(WORKSPACE_DIR))

from scripts.rp608_crypto_event_engine import (
    MultiLookbackCoordinator,
    CompletedCandle,
    CANDIDATE_CONFIGS,
    POLICY_FAMILY,
    EVENT_DIR,
    MAX_VALID_SPREAD_BPS,
    MAX_QUOTE_AGE_MS,
    MIN_REFRACTORY_MS,
    MIN_HISTORY_WARMUP_SEC
)

EVENT_OUTPUT_DIR = WORKSPACE_DIR / "live_capture" / "shadow_validation" / "crypto" / "event_driven"
CANONICAL_DIR = WORKSPACE_DIR / "live_capture" / "shadow_validation" / "crypto"
TELEMETRY_PATH = EVENT_OUTPUT_DIR / "event_telemetry.jsonl"

BINANCE_KLINE_WS = "wss://stream.binance.com:9443/ws/btcusdt@kline_15m"
BINANCE_BOOK_TICKER_WS = "wss://stream.binance.com:9443/ws/btcusdt@bookTicker"
BINANCE_REST_KLINES = "https://api.binance.com/api/v3/klines?symbol=BTCUSDT&interval=15m&limit=150"


def enforce_isolation_invariants():
    """Verifies that the daemon output path does not collide with canonical files."""
    EVENT_OUTPUT_DIR.mkdir(parents=True, exist_ok=True)
    canonical_files = [
        CANONICAL_DIR / "rp608_decision_cards.jsonl",
        CANONICAL_DIR / "rp608_checkpoint.json",
        CANONICAL_DIR / "rp608_shadow_telemetry.jsonl",
        CANONICAL_DIR / "rp608_trade_journal.jsonl"
    ]
    for cf in canonical_files:
        assert cf.parent != EVENT_OUTPUT_DIR, f"FATAL: Isolation breach! Event output points to canonical dir {cf.parent}"
        assert not str(EVENT_OUTPUT_DIR).startswith(str(cf)), "FATAL: Path collision detected!"
    print(f"[RP-608-EVENT] Isolation Invariant Verified: Dedicated Output Dir -> {EVENT_OUTPUT_DIR}")


def fetch_historical_candles(limit: int = 150) -> List[CompletedCandle]:
    """Fetches completed 15m candles from Binance public REST endpoint to pre-warm lookback buffers."""
    print(f"[RP-608-EVENT] Pre-warming candle context buffer ({limit} completed 15m bars)...")
    url = f"https://api.binance.com/api/v3/klines?symbol=BTCUSDT&interval=15m&limit={limit}"
    req = urllib.request.Request(url, headers={"User-Agent": "RP608-Event-Observer/1.0"})
    candles: List[CompletedCandle] = []

    try:
        ctx = ssl.create_default_context()
        with urllib.request.urlopen(req, timeout=10, context=ctx) as resp:
            data = json.loads(resp.read().decode("utf-8"))

        now_ms = int(time.time() * 1000)
        # Last element may be an in-progress bar, so only take completed ones (close_time < now_ms)
        for idx, item in enumerate(data):
            open_time_ms = int(item[0])
            open_p = float(item[1])
            high_p = float(item[2])
            low_p = float(item[3])
            close_p = float(item[4])
            vol = float(item[5])
            close_time_ms = int(item[6])

            if close_time_ms < now_ms:
                candles.append(CompletedCandle(
                    bar_idx=idx + 1,
                    open_time_ms=open_time_ms,
                    close_time_ms=close_time_ms,
                    open_price=open_p,
                    high_price=high_p,
                    low_price=low_p,
                    close_price=close_p,
                    volume=vol
                ))
        print(f"[RP-608-EVENT] Successfully loaded {len(candles)} completed 15m bars into context buffer.")
    except Exception as exc:
        print(f"[RP-608-EVENT] Warning: REST warmup failed ({exc}). Will warm up incrementally via kline stream.")

    return candles


class EventDaemon:
    def __init__(self, output_dir: Path = EVENT_OUTPUT_DIR):
        self.output_dir = output_dir
        self.output_dir.mkdir(parents=True, exist_ok=True)
        self.coordinator = MultiLookbackCoordinator(base_dir=self.output_dir)
        self.running = False
        self.telemetry_path = self.output_dir / "event_telemetry.jsonl"
        self.heartbeat_counter = 0

    async def run_kline_stream(self):
        """Maintains WebSocket for completed 15m candles."""
        while self.running:
            try:
                print(f"[RP-608-EVENT] Connecting to Binance Kline Stream: {BINANCE_KLINE_WS}")
                async with websockets.connect(BINANCE_KLINE_WS, ping_interval=20, ping_timeout=10) as ws:
                    print("[RP-608-EVENT] Kline WebSocket Connected.")
                    while self.running:
                        msg = await ws.recv()
                        data = json.loads(msg)
                        k = data.get("k", {})
                        if k.get("x", False):  # Candle is closed
                            candle = CompletedCandle(
                                bar_idx=self.coordinator.candle_buf.count() + 1,
                                open_time_ms=int(k["t"]),
                                close_time_ms=int(k["T"]),
                                open_price=float(k["o"]),
                                high_price=float(k["h"]),
                                low_price=float(k["l"]),
                                close_price=float(k["c"]),
                                volume=float(k["v"])
                            )
                            self.coordinator.ingest_completed_candle(candle)
                            print(f"[RP-608-EVENT] Ingested Closed 15m Candle: close={candle.close_price:.2f} | Buffer Count: {self.coordinator.candle_buf.count()}")
            except asyncio.CancelledError:
                break
            except Exception as e:
                print(f"[RP-608-EVENT] Kline WebSocket Error: {e}. Reconnecting in 5s...")
                await asyncio.sleep(5)

    async def run_book_ticker_stream(self):
        """Maintains WebSocket for real-time bookTicker quotes."""
        while self.running:
            try:
                print(f"[RP-608-EVENT] Connecting to Binance BookTicker Stream: {BINANCE_BOOK_TICKER_WS}")
                async with websockets.connect(BINANCE_BOOK_TICKER_WS, ping_interval=20, ping_timeout=10) as ws:
                    print("[RP-608-EVENT] BookTicker WebSocket Connected.")
                    self.coordinator.handle_reconnect(reason="WS_CONNECTED")
                    while self.running:
                        msg = await ws.recv()
                        recv_ms = int(time.time() * 1000)
                        raw = json.loads(msg)
                        # Process quote across all 4 candidate lookbacks
                        self.coordinator.process_quote_event(raw, receive_time_ms=recv_ms)
            except asyncio.CancelledError:
                break
            except Exception as e:
                print(f"[RP-608-EVENT] BookTicker WebSocket Disconnect: {e}. Resetting state and reconnecting in 3s...")
                self.coordinator.handle_reconnect(reason="WS_DISCONNECT")
                await asyncio.sleep(3)

    async def run_telemetry_logger(self):
        """Periodically logs diagnostic telemetry (every 60s) without polluting recommendation ledgers."""
        while self.running:
            try:
                await asyncio.sleep(60)
                self.heartbeat_counter += 1
                now_utc = datetime.now(timezone.utc).isoformat()
                now_ms = int(time.time() * 1000)

                stats_summary = self.coordinator.get_summary_stats()
                telemetry_record = {
                    "heartbeat_id": self.heartbeat_counter,
                    "timestamp_utc": now_utc,
                    "timestamp_ms": now_ms,
                    "order_authority": 0.0,
                    "candle_buffer_count": stats_summary["candle_count"],
                    "candidates": {
                        bars: {
                            "candidate_id": cdata["candidate_id"],
                            "warmed_up": cdata["warmed_up"],
                            "quotes_received": cdata["stats"]["quotes_received"],
                            "quotes_valid": cdata["stats"]["quotes_valid"],
                            "quotes_rejected": cdata["stats"]["quotes_rejected"],
                            "recs_emitted": cdata["stats"]["recs_emitted"],
                            "recs_rate_limited": cdata["stats"]["recs_rate_limited"],
                            "recs_counter_trend_filtered": cdata["stats"]["recs_counter_trend_filtered"],
                            "stream_reconnect_count": cdata["stats"]["stream_reconnect_count"]
                        }
                        for bars, cdata in stats_summary["candidates"].items()
                    }
                }

                with open(self.telemetry_path, "a", encoding="utf-8") as f:
                    f.write(json.dumps(telemetry_record) + "\n")

            except asyncio.CancelledError:
                break
            except Exception as e:
                print(f"[RP-608-EVENT] Telemetry Logging Error: {e}")

    async def start(self):
        self.running = True
        enforce_isolation_invariants()

        # Pre-seed candle buffer
        initial_candles = fetch_historical_candles(limit=150)
        for c in initial_candles:
            self.coordinator.ingest_completed_candle(c)

        print("\n" + "=" * 80)
        print("  RP-608-EVENT-001: 4-CANDIDATE SHADOW DECISION DAEMON STARTED")
        print(f"  Output Directory: {self.output_dir}")
        print("  Order Authority: Strictly 0.0% (Read-Only)")
        print("  Candidate Configurations:")
        for bars, eng in self.coordinator.engines.items():
            print(f"    - {eng.candidate_id}: {eng.candidate_name} | Lookback: {bars} bars | Hash: {eng.config_hash}")
        print("=" * 80 + "\n")

        # Launch concurrent background tasks
        await asyncio.gather(
            self.run_kline_stream(),
            self.run_book_ticker_stream(),
            self.run_telemetry_logger()
        )

    def stop(self):
        print("\n[RP-608-EVENT] Stopping daemon cleanly...")
        self.running = False


def main():
    daemon = EventDaemon()

    loop = asyncio.new_event_loop()
    asyncio.set_event_loop(loop)

    def handle_sig(sig, frame):
        daemon.stop()
        for task in asyncio.all_tasks(loop):
            task.cancel()

    signal.signal(signal.SIGINT, handle_sig)
    signal.signal(signal.SIGTERM, handle_sig)

    try:
        loop.run_until_complete(daemon.start())
    except (asyncio.CancelledError, KeyboardInterrupt):
        pass
    finally:
        print("[RP-608-EVENT] Daemon stopped.")


if __name__ == "__main__":
    main()
