#!/usr/bin/env python3
"""
emit_rp614_gate4_trade_disposition_ledger.py

Generates the exhaustive trade-level disposition ledger for all 7,264 RP-605 input trades:
- trade_id, symbol, side, entry_timestamp, exit_timestamp, observed_bars, expected_bars, gap_duration_sec, disposition, grid_eligibility
- Emits live_capture/shadow_validation/rp614_gate4_trade_disposition_ledger.csv
"""

import json
import csv
from pathlib import Path
from datetime import datetime, timezone

WORKSPACE_DIR = Path(__file__).resolve().parent.parent
DATA_DIR = WORKSPACE_DIR / "live_capture" / "shadow_validation"
CANDLE_DIR = WORKSPACE_DIR / "intraday_capture" / "yahoo_cache_1m"

JOURNAL_FILE = DATA_DIR / "rp605_nse_baseline_journal.jsonl"
OUT_LEDGER_CSV = DATA_DIR / "rp614_gate4_trade_disposition_ledger.csv"

def main():
    print("=" * 80)
    print("📋 GENERATING RP-614 GATE 4 TRADE DISPOSITION LEDGER (N = 7,264)")
    print("=" * 80)

    with open(JOURNAL_FILE, "r") as f:
        trades = [json.loads(line) for line in f if line.strip()]

    symbols = sorted(list(set(t["symbol"] for t in trades)))
    candle_cache = {}
    for sym in symbols:
        file_path = CANDLE_DIR / f"{sym}.json"
        if not file_path.exists() and not sym.endswith(".NS"):
            file_path = CANDLE_DIR / f"{sym}.NS.json"
        with open(file_path, "r") as f:
            candles = json.load(f)
            candle_cache[sym] = sorted(candles, key=lambda x: x["timestamp"])

    records = []
    dispositions_count = {
        "COMPLETE_OBSERVATION": 0,
        "INCOMPLETE_IN_TRADE_GAP": 0,
        "NO_BAR_DATA": 0
    }

    for t in trades:
        sym = t["symbol"]
        candles = candle_cache.get(sym, [])
        entry_ts_dt = datetime.fromisoformat(t["entry_timestamp"].replace("Z", "+00:00")).astimezone(timezone.utc)
        entry_epoch = int(entry_ts_dt.timestamp())
        exit_ts_dt = datetime.fromisoformat(t["exit_timestamp"].replace("Z", "+00:00")).astimezone(timezone.utc)
        exit_epoch = int(exit_ts_dt.timestamp())

        trade_bars = [c for c in candles if entry_epoch <= c["timestamp"] <= exit_epoch]
        expected_trade_minutes = max(1, int((exit_epoch - entry_epoch) / 60))

        if not trade_bars:
            disp = "NO_BAR_DATA"
            grid_eligible = False
            gap_duration = exit_epoch - entry_epoch
        elif len(trade_bars) < expected_trade_minutes:
            disp = "INCOMPLETE_IN_TRADE_GAP"
            grid_eligible = False
            gap_duration = (expected_trade_minutes - len(trade_bars)) * 60
        else:
            disp = "COMPLETE_OBSERVATION"
            grid_eligible = True
            gap_duration = 0

        dispositions_count[disp] += 1
        records.append({
            "trade_id": t["trade_id"],
            "session_date": entry_ts_dt.strftime("%Y-%m-%d"),
            "symbol": sym,
            "side": t["side"],
            "entry_timestamp_utc": t["entry_timestamp"],
            "exit_timestamp_utc": t["exit_timestamp"],
            "entry_price": float(t["entry_price"]),
            "exit_price": float(t["exit_price"]),
            "quantity": float(t["quantity"]),
            "gross_pnl": float(t["pnl"]),
            "observed_holding_sec": exit_epoch - entry_epoch,
            "observed_bars_count": len(trade_bars),
            "expected_bars_count": expected_trade_minutes,
            "gap_duration_sec": gap_duration,
            "disposition": disp,
            "grid_eligible": grid_eligible
        })

    with open(OUT_LEDGER_CSV, "w", newline="", encoding="utf-8") as f:
        writer = csv.DictWriter(f, fieldnames=list(records[0].keys()))
        writer.writeheader()
        writer.writerows(records)

    print(f"💾 Emitted {len(records):,} trade disposition records to: {OUT_LEDGER_CSV}")
    print("\n--- POPULATION SUMMARY ---")
    for k, v in dispositions_count.items():
        print(f"  • {k:28s}: {v:,} ({v / len(trades) * 100:.2f}%)")
    print(f"  • Grid-Eligible Complete Pop : {dispositions_count['COMPLETE_OBSERVATION']:,} ({dispositions_count['COMPLETE_OBSERVATION']/len(trades)*100:.2f}%)")
    print("=" * 80)

if __name__ == "__main__":
    main()
