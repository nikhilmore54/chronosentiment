#!/usr/bin/env python3
"""
ChronoSentiment — Historical Data Downloader
Downloads crypto data from Yahoo Finance by chunking the requests.
Saves the synchronized output as a JSON Lines file for fast offline replaying.
"""

import argparse
import os
import json
import time
from datetime import datetime, timedelta
import pandas as pd
import yfinance as yf
from pathlib import Path

_ROOT = Path(__file__).resolve().parents[1]
SYMBOLS = ["BTC-USD", "ETH-USD", "SOL-USD"]

def main():
    parser = argparse.ArgumentParser(description="Download Historical Data")
    parser.add_argument("--days", type=int, default=60, help="Number of days to download")
    parser.add_argument("--interval", type=str, default="5m", help="Candle interval (1m, 5m, 1h, 1d)")
    args = parser.parse_args()

    output_file = _ROOT / "archive" / f"history_{args.days}d_{args.interval}.jsonl"

    print(f"🚀 Starting {args.days}-day historical download ({args.interval}) for {SYMBOLS}...")
    
    end_date = datetime.now()
    start_date = end_date - timedelta(days=args.days)
    
    all_data = {sym: pd.DataFrame() for sym in SYMBOLS}
    
    # We chunk the requests into 7-day windows to avoid limits
    current_end = end_date
    current_start = max(start_date, current_end - timedelta(days=7))
    
    while current_end > start_date:
        print(f"📥 Fetching window: {current_start.strftime('%Y-%m-%d')} to {current_end.strftime('%Y-%m-%d')}...")
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
            
            if not df.empty:
                # Handle MultiIndex columns from yfinance
                if isinstance(df.columns, pd.MultiIndex):
                    # We iterate through symbols and extract their specific columns
                    for sym in SYMBOLS:
                        if sym in df.columns.get_level_values(1):
                            # Get cross-section for this symbol
                            sym_df = df.xs(sym, level=1, axis=1)
                            all_data[sym] = pd.concat([all_data[sym], sym_df])
            
        except Exception as e:
            print(f"⚠️ Error fetching window: {e}")
            
        # Move window back
        current_end = current_start
        current_start = max(start_date, current_end - timedelta(days=7))
        time.sleep(1) # Be polite to API
        
    print("🔄 Synchronizing and sorting data...")
    
    # Process into a unified timeline
    timeline = {}
    for sym in SYMBOLS:
        df = all_data[sym]
        if df.empty:
            continue
            
        # Drop duplicates in case windows overlapped
        df = df[~df.index.duplicated(keep='first')]
        df = df.sort_index()
        
        for ts, row in df.iterrows():
            try:
                unix_ts = int(ts.timestamp())
            except:
                continue
                
            # Skip rows with NaN prices
            if pd.isna(row['Close']):
                continue
                
            if unix_ts not in timeline:
                timeline[unix_ts] = []
                
            timeline[unix_ts].append({
                "symbol": sym,
                "timestamp": unix_ts,
                "open": float(row["Open"]),
                "high": float(row["High"]),
                "low": float(row["Low"]),
                "close": float(row["Close"]),
                "volume": float(row.get("Volume", 0))
            })
            
    sorted_timestamps = sorted(timeline.keys())
    
    if not sorted_timestamps:
        print("❌ No data retrieved! Check interval bounds (e.g., 1m is max 30d, 5m is max 60d).")
        return

    # Save to JSONL
    print(f"💾 Saving {len(sorted_timestamps)} synchronized timesteps to {output_file}...")
    output_file.parent.mkdir(parents=True, exist_ok=True)
    
    with open(output_file, 'w') as f:
        for ts in sorted_timestamps:
            f.write(json.dumps(timeline[ts]) + "\n")
            
    print(f"✅ Success! Run 'python3 scripts/replay_from_file.py --file {output_file}' to pipe this into the engine.")

if __name__ == "__main__":
    main()
