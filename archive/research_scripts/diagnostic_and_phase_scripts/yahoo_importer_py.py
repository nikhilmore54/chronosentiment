#!/usr/bin/env python3
import argparse
import json
import hashlib
import os
from pathlib import Path
import yfinance as yf

def main():
    parser = argparse.ArgumentParser(description="Yahoo Finance importer (Python)")
    parser.add_argument("--symbol", required=True, help="Yahoo ticker (e.g., '^GDAXI', 'HSBA.L')")
    parser.add_argument("--interval", default="1m", help="Data interval (e.g., 1m, 5m, 1h)")
    parser.add_argument("--name", required=True, help="Capture name used for output directory")
    args = parser.parse_args()

    # Fetch last 7 days (max for 1m interval)
    data = yf.download(tickers=args.symbol, period="7d", interval=args.interval, progress=False)
    if data.empty:
        print(f"[FAIL] No data returned for {args.symbol}")
        exit(1)

    # Prepare output directory
    base_dir = Path("chronology/historical") / args.name
    base_dir.mkdir(parents=True, exist_ok=True)

    # Use first timestamp as start identifier
    start_ts = int(data.index[0].timestamp() * 1000)
    file_path = base_dir / f"{args.symbol.replace('^','').replace('.','').lower()}_{start_ts}.jsonl"
    manifest_path = base_dir / f"{args.name}_{start_ts}_manifest.json"

    hasher = hashlib.sha256()
    tick_count = 0
    with file_path.open("w", encoding="utf-8") as f:
        for ts, row in data.iterrows():
            tick = {
                "symbol": args.symbol,
                "timestamp": int(ts.timestamp() * 1000),
                "price": float(row["Close"].iloc[0] if hasattr(row["Close"], "iloc") else row["Close"]),
                "volume": float(row["Volume"].iloc[0] if hasattr(row["Volume"], "iloc") else row["Volume"]),
                "is_buyer_maker": False,
            }
            line = json.dumps(tick)
            f.write(line + "\n")
            hasher.update((line + "\n").encode("utf-8"))
            tick_count += 1

    manifest = {
        "substrate": args.symbol,
        "capture_start": start_ts,
        "capture_end": int(data.index[-1].timestamp() * 1000),
        "total_ticks": tick_count,
        "chronology_hash": hasher.hexdigest(),
        "gaps": [],
        "provenance": "Yahoo Finance via yfinance",
    }
    with manifest_path.open("w", encoding="utf-8") as mf:
        json.dump(manifest, mf, indent=2)

    print(f"✅ Yahoo Historical Capture Complete: {tick_count} ticks")
    print(f"   Hash: {hasher.hexdigest()}")
    print(f"   Directory: {base_dir}")

if __name__ == "__main__":
    main()
