#!/usr/bin/env python3
"""
ChronoSentiment — OOS Historical Data Downloader
Downloads a SPECIFIC date range for out-of-sample validation.
The architecture is FROZEN — this data must NOT influence any parameters.
"""

import json, time, argparse
from datetime import datetime, timedelta
import pandas as pd
import yfinance as yf
from pathlib import Path

_ROOT = Path(__file__).resolve().parents[1]
SYMBOLS = ["BTC-USD", "ETH-USD", "SOL-USD"]

def main():
    parser = argparse.ArgumentParser(description="Download OOS Historical Data")
    parser.add_argument("--start", type=str, required=True, help="Start date YYYY-MM-DD")
    parser.add_argument("--end", type=str, required=True, help="End date YYYY-MM-DD")
    parser.add_argument("--interval", type=str, default="1m", help="Candle interval")
    parser.add_argument("--output", type=str, default=None, help="Output filename")
    args = parser.parse_args()

    start_date = datetime.strptime(args.start, "%Y-%m-%d")
    end_date = datetime.strptime(args.end, "%Y-%m-%d")
    
    label = f"oos_{args.start}_{args.end}_{args.interval}"
    output_file = _ROOT / "archive" / (args.output or f"{label}.jsonl")
    
    days = (end_date - start_date).days
    print(f"🧪 OOS DOWNLOAD — Architecture is FROZEN")
    print(f"📅 Window: {args.start} → {args.end} ({days} days)")
    print(f"📊 Interval: {args.interval}")
    print(f"🎯 Assets: {SYMBOLS}")
    print()
    
    # Yahoo Finance limits: 1m = max 30 days back, must chunk 7-day windows
    all_data = {sym: pd.DataFrame() for sym in SYMBOLS}
    
    current_end = end_date
    current_start = max(start_date, current_end - timedelta(days=7))
    
    while current_end > start_date:
        print(f"  📥 Fetching {current_start.strftime('%Y-%m-%d')} → {current_end.strftime('%Y-%m-%d')}...")
        try:
            df = yf.download(
                tickers=SYMBOLS,
                start=current_start,
                end=current_end,
                interval=args.interval,
                auto_adjust=True,
                progress=False,
                threads=False
            )
            if not df.empty and isinstance(df.columns, pd.MultiIndex):
                for sym in SYMBOLS:
                    if sym in df.columns.get_level_values(1):
                        sym_df = df.xs(sym, level=1, axis=1)
                        all_data[sym] = pd.concat([all_data[sym], sym_df])
        except Exception as e:
            print(f"  ⚠️ Error: {e}")
            
        current_end = current_start
        current_start = max(start_date, current_end - timedelta(days=7))
        time.sleep(1)
    
    # Build synchronized timeline
    timeline = {}
    for sym in SYMBOLS:
        df = all_data[sym]
        if df.empty:
            continue
        df = df[~df.index.duplicated(keep='first')].sort_index()
        for ts, row in df.iterrows():
            try:
                unix_ts = int(ts.timestamp())
            except:
                continue
            if pd.isna(row['Close']):
                continue
            if unix_ts not in timeline:
                timeline[unix_ts] = []
            timeline[unix_ts].append({
                "symbol": sym, "timestamp": unix_ts,
                "open": float(row["Open"]), "high": float(row["High"]),
                "low": float(row["Low"]), "close": float(row["Close"]),
                "volume": float(row.get("Volume", 0))
            })
    
    sorted_ts = sorted(timeline.keys())
    if not sorted_ts:
        print("❌ No data retrieved! Check that the date range is within Yahoo Finance limits.")
        return
    
    print(f"\n💾 Saving {len(sorted_ts)} timesteps → {output_file}")
    output_file.parent.mkdir(parents=True, exist_ok=True)
    with open(output_file, 'w') as f:
        for ts in sorted_ts:
            f.write(json.dumps(timeline[ts]) + "\n")
    
    first_dt = datetime.utcfromtimestamp(sorted_ts[0])
    last_dt = datetime.utcfromtimestamp(sorted_ts[-1])
    print(f"✅ OOS data: {first_dt} → {last_dt}")
    print(f"🧪 Replay with: python3 scripts/replay_from_file.py --file {output_file}")

if __name__ == "__main__":
    main()
