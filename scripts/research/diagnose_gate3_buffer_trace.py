#!/usr/bin/env python3
"""
scripts/research/diagnose_gate3_buffer_trace.py

Non-destructive diagnostic trace for Gate 3 (Causal Warmup Hurdle: 5 bars).
Traces symbol RELIANCE.NS end-to-end:
1. Incoming observation format and frequency from Kite feed
2. Historical 15m candle availability vs /quote polling snapshot
3. Current persisted buffer state in rp608_nse_bar_history.json & latest_evaluations.json
4. Evaluator Gate 3 logic and buffer length accounting
"""

import json
import time
import sys
from datetime import datetime, timezone, timedelta
from pathlib import Path

WORKSPACE_DIR = Path("/Users/nikhil/ChronoSentiment_MEGA_FINAL")
sys.path.insert(0, str(WORKSPACE_DIR))
NSE_TEST_DIR = WORKSPACE_DIR / "live_capture" / "shadow_validation" / "nse" / "test_run"

EVAL_FILE = NSE_TEST_DIR / "rp608_nse_latest_evaluations.json"
HISTORY_FILE = NSE_TEST_DIR / "rp608_nse_bar_history.json"
CHECKPOINT_FILE = NSE_TEST_DIR / "rp608_nse_checkpoint.json"

SYMBOL = "RELIANCE.NS"


def diagnose_gate3():
    print("=" * 80)
    print(f"  GATE 3 END-TO-END DIAGNOSTIC TRACE FOR {SYMBOL}")
    print("=" * 80)

    # 1. Inspect on-disk evaluation snapshot
    print("\n--- 1. Snapshot Inspection in rp608_nse_latest_evaluations.json ---")
    if EVAL_FILE.exists():
        with open(EVAL_FILE, "r", encoding="utf-8") as f:
            eval_data = json.load(f)
        snapshot_time = eval_data.get("timestamp_iso")
        bar_idx = eval_data.get("bar_idx")
        sym_eval = eval_data.get("evaluations", {}).get(SYMBOL, {})
        hist_len = sym_eval.get("history_length")
        decision_code = sym_eval.get("decision_code")
        explanation = sym_eval.get("explanation")
        print(f"File modified time: {datetime.fromtimestamp(EVAL_FILE.stat().st_mtime, tz=timezone.utc).isoformat()}")
        print(f"Snapshot Timestamp: {snapshot_time} | Session Bar Index: {bar_idx}")
        print(f"Symbol Record: {SYMBOL}")
        print(f"  history_length: {hist_len}")
        print(f"  decision_code: {decision_code}")
        print(f"  explanation: {explanation}")
    else:
        print(f"[!] {EVAL_FILE} does not exist!")

    # 2. Inspect on-disk price history buffer
    print("\n--- 2. Persisted Buffer Inspection in rp608_nse_bar_history.json ---")
    if HISTORY_FILE.exists():
        with open(HISTORY_FILE, "r", encoding="utf-8") as f:
            hist_data = json.load(f)
        sym_bars = hist_data.get(SYMBOL, [])
        print(f"Persisted bars count for {SYMBOL}: {len(sym_bars)}")
        for idx, b in enumerate(sym_bars):
            ts = b.get("timestamp_utc_ms")
            iso = datetime.fromtimestamp(ts / 1000.0, tz=timezone.utc).isoformat() if ts else "N/A"
            print(f"  Bar {idx+1}: Time={iso} Close={b.get('close')} Source={b.get('source')} Completed={b.get('is_completed')}")
    else:
        print(f"[!] {HISTORY_FILE} does not exist on disk!")

    # 3. Live Feed Ingestion Probe for RELIANCE.NS
    print("\n--- 3. Live Feed Probe & Capability Check ---")
    from scripts.run_rp608_nse_shadow_daemon import KiteDataFeedAdapter
    adapter = KiteDataFeedAdapter((SYMBOL,))
    print(f"Adapter Credentials Present: {bool(adapter.access_token)}")
    if adapter.access_token:
        # Check /quote response structure
        feed_res = adapter.fetch_all_symbols()
        sym_res = feed_res.get(SYMBOL, {})
        bars = sym_res.get("bars", [])
        quote = sym_res.get("quote")
        print(f"Live /quote fetched bars count: {len(bars)}")
        if bars:
            b = bars[0]
            print(f"  Poll Bar Type: BarOHLCV, timestamp_utc_ms={b.timestamp_utc_ms}")
            print(f"  Bar OHLC: O={b.open}, H={b.high}, L={b.low}, C={b.close}, V={b.volume}")
            print(f"  Notice: /quote returns DAY cumulative OHLC, NOT a 15-minute completed candle!")

        # Check historical 15m candle capability
        now_utc = datetime.now(timezone.utc)
        ist_offset = timedelta(hours=5, minutes=30)
        now_ist = now_utc + ist_offset
        open_ist = now_ist.replace(hour=9, minute=15, second=0, microsecond=0)
        from_utc = (open_ist - ist_offset).astimezone(timezone.utc)
        to_utc = now_utc

        print(f"\nChecking Kite Connect Historical 15m API for today ({from_utc.isoformat()} to {to_utc.isoformat()}):")
        hist_15m = adapter.fetch_historical_15m_bars(SYMBOL, from_utc, to_utc)
        print(f"Historical 15m bars returned today: {len(hist_15m)}")
        for idx, b in enumerate(hist_15m):
            b_ist = datetime.fromtimestamp(b.timestamp_utc_ms / 1000.0, tz=timezone.utc) + ist_offset
            print(f"  Hist 15m Bar {idx+1}: IST={b_ist.strftime('%H:%M:%S')} O={b.open} H={b.high} L={b.low} C={b.close} V={b.volume}")

    print("\n" + "=" * 80)


if __name__ == "__main__":
    diagnose_gate3()
