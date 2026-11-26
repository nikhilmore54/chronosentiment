#!/usr/bin/env python3
"""
audit_rp614_gate4_market_data_quality.py

Full Population Market Data Quality & Trade Window Completeness Audit for RP-614 Gate 4.
Audits:
- All 99 NSE symbols across 4 canonical sessions (2026-09-28, 2026-09-29, 2026-09-30, 2026-10-01)
- Expected schedule: 375 bars per session (09:15 to 15:29 IST = 03:45 to 09:59 UTC)
- OHLC consistency, positive prices, duplicates, volume
- Trade window completeness for all 7,264 canonical trades
- Emits rp614_gate4_data_quality_inventory.json
"""

import json
from pathlib import Path
from datetime import datetime, timezone, timedelta
from typing import Dict, List, Any
import numpy as np

WORKSPACE_DIR = Path(__file__).resolve().parent.parent
DATA_DIR = WORKSPACE_DIR / "live_capture" / "shadow_validation"
CANDLE_DIR = WORKSPACE_DIR / "intraday_capture" / "yahoo_cache_1m"

JOURNAL_FILE = DATA_DIR / "rp605_nse_baseline_journal.jsonl"
OUT_INVENTORY_JSON = DATA_DIR / "rp614_gate4_data_quality_inventory.json"

CANONICAL_SESSIONS = ["2026-09-28", "2026-09-29", "2026-09-30", "2026-10-01"]

def audit_symbol_session_candles(candles: List[Dict[str, Any]], session_date: str) -> Dict[str, Any]:
    start_dt = datetime.fromisoformat(f"{session_date}T03:45:00+00:00")
    expected_epochs = [int((start_dt + timedelta(minutes=i)).timestamp()) for i in range(375)]
    
    observed_map = {}
    duplicates = 0
    invalid_ohlc = 0
    non_positive_px = 0
    
    for c in candles:
        ts = c["timestamp"]
        c_dt = datetime.fromtimestamp(ts, tz=timezone.utc)
        if c_dt.strftime("%Y-%m-%d") != session_date:
            continue
        
        if ts in observed_map:
            duplicates += 1
            continue
            
        o, h, l, cl = float(c["open"]), float(c["high"]), float(c["low"]), float(c["close"])
        if o <= 0 or h <= 0 or l <= 0 or cl <= 0:
            non_positive_px += 1
        if l > o or l > cl or h < o or h < cl or l > h:
            invalid_ohlc += 1
            
        observed_map[ts] = c

    missing_epochs = [ts for ts in expected_epochs if ts not in observed_map]
    
    gap_windows = []
    if missing_epochs:
        cur_gap = [missing_epochs[0]]
        for ts in missing_epochs[1:]:
            if ts == cur_gap[-1] + 60:
                cur_gap.append(ts)
            else:
                gap_windows.append(cur_gap)
                cur_gap = [ts]
        gap_windows.append(cur_gap)

    return {
        "session_date": session_date,
        "expected_bars": len(expected_epochs),
        "observed_bars": len([ts for ts in expected_epochs if ts in observed_map]),
        "missing_bars_count": len(missing_epochs),
        "gap_windows_count": len(gap_windows),
        "duplicate_timestamps": duplicates,
        "invalid_ohlc_count": invalid_ohlc,
        "non_positive_price_count": non_positive_px
    }

def main():
    print("=" * 80)
    print("🔬 RP-614 GATE 4: FULL POPULATION MARKET DATA QUALITY & TRADE AUDIT")
    print("=" * 80)

    with open(JOURNAL_FILE, "r") as f:
        trades = [json.loads(line) for line in f if line.strip()]

    symbols = sorted(list(set(t["symbol"] for t in trades)))
    total_trades = len(trades)
    print(f"📖 Loaded {total_trades:,} canonical trades across {len(symbols)} unique symbols.")

    # 1. Per-symbol per-session candle audit
    symbol_audits = {}
    total_expected_bars = len(symbols) * len(CANONICAL_SESSIONS) * 375
    total_observed_bars = 0
    total_missing_bars = 0
    total_duplicates = 0
    total_invalid_ohlc = 0
    total_non_positive = 0

    candle_cache = {}
    for sym in symbols:
        file_path = CANDLE_DIR / f"{sym}.json"
        if not file_path.exists() and not sym.endswith(".NS"):
            file_path = CANDLE_DIR / f"{sym}.NS.json"
        
        with open(file_path, "r") as f:
            candles = json.load(f)
            candle_cache[sym] = sorted(candles, key=lambda x: x["timestamp"])

        symbol_audits[sym] = {}
        for sess in CANONICAL_SESSIONS:
            res = audit_symbol_session_candles(candle_cache[sym], sess)
            symbol_audits[sym][sess] = res
            total_observed_bars += res["observed_bars"]
            total_missing_bars += res["missing_bars_count"]
            total_duplicates += res["duplicate_timestamps"]
            total_invalid_ohlc += res["invalid_ohlc_count"]
            total_non_positive += res["non_positive_price_count"]

    print("\n--- AGGREGATE MARKET DATA INTEGRITY TOTALS ---")
    print(f"Total Expected Bars (99 symbols x 4 sessions x 375 min): {total_expected_bars:,}")
    print(f"Total Observed Bars:  {total_observed_bars:,} ({total_observed_bars / total_expected_bars * 100:.2f}% coverage)")
    print(f"Total Missing Bars:   {total_missing_bars:,} ({total_missing_bars / total_expected_bars * 100:.2f}% missingness)")
    print(f"Total Duplicate Timestamps: {total_duplicates}")
    print(f"Total Invalid OHLC Records: {total_invalid_ohlc}")
    print(f"Total Non-Positive Prices:  {total_non_positive}")

    # 2. Trade Window Completeness Audit
    # For each trade, check if candle data exists from entry to exit
    trade_dispositions = {
        "COMPLETE_OBSERVATION": 0,
        "INCOMPLETE_IN_TRADE_GAP": 0,
        "DATA_TRUNCATED_PRE_EXIT": 0,
        "NO_BAR_DATA": 0
    }
    
    trade_level_records = []

    for t in trades:
        sym = t["symbol"]
        candles = candle_cache[sym]
        ts_list = [c["timestamp"] for c in candles]
        
        entry_ts_dt = datetime.fromisoformat(t["entry_timestamp"].replace("Z", "+00:00")).astimezone(timezone.utc)
        entry_epoch = int(entry_ts_dt.timestamp())
        exit_ts_dt = datetime.fromisoformat(t["exit_timestamp"].replace("Z", "+00:00")).astimezone(timezone.utc)
        exit_epoch = int(exit_ts_dt.timestamp())
        
        # Check available bars in trade interval
        trade_bars = [c for c in candles if entry_epoch <= c["timestamp"] <= exit_epoch]
        expected_trade_minutes = max(1, int((exit_epoch - entry_epoch) / 60))
        
        if not trade_bars:
            disp = "NO_BAR_DATA"
        elif len(trade_bars) < expected_trade_minutes:
            disp = "INCOMPLETE_IN_TRADE_GAP"
        else:
            disp = "COMPLETE_OBSERVATION"
            
        trade_dispositions[disp] += 1
        trade_level_records.append({
            "trade_id": t["trade_id"],
            "session_date": entry_ts_dt.strftime("%Y-%m-%d"),
            "symbol": sym,
            "side": t["side"],
            "entry_timestamp": t["entry_timestamp"],
            "exit_timestamp": t["exit_timestamp"],
            "observed_bars": len(trade_bars),
            "expected_bars": expected_trade_minutes,
            "disposition": disp
        })

    print("\n--- TRADE-LEVEL OBSERVATION DISPOSITION RECONCILIATION (N = 7,264) ---")
    for disp, cnt in trade_dispositions.items():
        print(f"  • {disp:30s}: {cnt:,} ({cnt / total_trades * 100:.2f}%)")

    assert sum(trade_dispositions.values()) == total_trades, "Trade disposition count mismatch!"

    # 3. Export Inventory JSON
    inventory = {
        "programme": "RP-614 Gate 4 Data Quality & Missing Bar Inventory",
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "audit_scope": {
            "total_canonical_trades": total_trades,
            "unique_symbols": len(symbols),
            "sessions": CANONICAL_SESSIONS,
            "expected_bars_per_session": 375,
            "total_expected_bars": total_expected_bars,
            "total_observed_bars": total_observed_bars,
            "coverage_pct": round(total_observed_bars / total_expected_bars * 100.0, 2),
            "total_missing_bars": total_missing_bars,
            "missingness_pct": round(total_missing_bars / total_expected_bars * 100.0, 2),
            "total_duplicates": total_duplicates,
            "total_invalid_ohlc": total_invalid_ohlc,
            "total_non_positive_prices": total_non_positive
        },
        "trade_window_dispositions": trade_dispositions,
        "symbol_audits": symbol_audits
    }

    with open(OUT_INVENTORY_JSON, "w", encoding="utf-8") as f:
        json.dump(inventory, f, indent=2)
    print(f"\n💾 Saved Gate 4 Data Quality Inventory JSON to: {OUT_INVENTORY_JSON}")
    print("=" * 80)
    print("✅ GATE 4 DATA QUALITY AUDIT COMPLETED")
    print("=" * 80)

if __name__ == "__main__":
    main()
