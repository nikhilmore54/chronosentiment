#!/usr/bin/env python3
"""
ChronoSentiment — Cross-Asset Data Downloader
Downloads data for non-crypto assets and maps them onto the engine's
expected symbol namespace (BTC-USD/ETH-USD/SOL-USD) to enable frozen replay.

The architecture is NOT modified. We simply feed different price data
through the same pipeline to test ecological law universality.
"""

import json, time, argparse
from datetime import datetime, timedelta
import pandas as pd
import yfinance as yf
from pathlib import Path

_ROOT = Path(__file__).resolve().parents[1]

ASSET_PROFILES = {
    "equities": {
        "tickers": ["SPY", "QQQ", "NVDA"],
        "label": "US Equities",
    },
    "fx": {
        "tickers": ["EURUSD=X", "GBPUSD=X", "USDJPY=X"],
        "label": "Foreign Exchange",
    },
    "commodities": {
        "tickers": ["GC=F", "CL=F", "SI=F"],
        "label": "Commodities",
    },
}

# Map external tickers → engine symbols (engine expects exactly these)
ENGINE_SYMBOLS = ["BTC-USD", "ETH-USD", "SOL-USD"]

def main():
    parser = argparse.ArgumentParser(description="Download Cross-Asset Data")
    parser.add_argument("--profile", type=str, required=True, choices=ASSET_PROFILES.keys())
    parser.add_argument("--days", type=int, default=30)
    parser.add_argument("--interval", type=str, default="5m")
    args = parser.parse_args()

    profile = ASSET_PROFILES[args.profile]
    tickers = profile["tickers"]
    output_file = _ROOT / "archive" / f"xasset_{args.profile}_{args.days}d_{args.interval}.jsonl"

    print(f"🌍 CROSS-ASSET DOWNLOAD — Architecture FROZEN")
    print(f"📊 Profile: {profile['label']}")
    print(f"🎯 Tickers: {tickers}")
    print(f"📅 Period: {args.days} days @ {args.interval}")
    print(f"🔄 Mapping: {' → '.join(f'{t}→{e}' for t, e in zip(tickers, ENGINE_SYMBOLS))}")
    print()

    end_date = datetime.now()
    start_date = end_date - timedelta(days=args.days)

    all_data = {t: pd.DataFrame() for t in tickers}

    current_end = end_date
    current_start = max(start_date, current_end - timedelta(days=7))

    while current_end > start_date:
        print(f"  📥 {current_start.strftime('%Y-%m-%d')} → {current_end.strftime('%Y-%m-%d')}...")
        try:
            df = yf.download(
                tickers=tickers, start=current_start, end=current_end,
                interval=args.interval, auto_adjust=True, progress=False, threads=False
            )
            if not df.empty:
                if isinstance(df.columns, pd.MultiIndex):
                    for t in tickers:
                        if t in df.columns.get_level_values(1):
                            all_data[t] = pd.concat([all_data[t], df.xs(t, level=1, axis=1)])
                else:
                    # Single ticker case
                    all_data[tickers[0]] = pd.concat([all_data[tickers[0]], df])
        except Exception as e:
            print(f"  ⚠️ {e}")

        current_end = current_start
        current_start = max(start_date, current_end - timedelta(days=7))
        time.sleep(1)

    # Build timeline with mapped symbols
    timeline = {}
    for i, ticker in enumerate(tickers):
        mapped_sym = ENGINE_SYMBOLS[i]
        df = all_data[ticker]
        if df.empty:
            print(f"  ⚠️ No data for {ticker}")
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
                "symbol": mapped_sym, "timestamp": unix_ts,
                "open": float(row["Open"]), "high": float(row["High"]),
                "low": float(row["Low"]), "close": float(row["Close"]),
                "volume": float(row.get("Volume", 0))
            })

    sorted_ts = sorted(timeline.keys())
    if not sorted_ts:
        print("❌ No data retrieved!")
        return

    print(f"\n💾 {len(sorted_ts)} timesteps → {output_file}")
    output_file.parent.mkdir(parents=True, exist_ok=True)
    with open(output_file, 'w') as f:
        for ts in sorted_ts:
            f.write(json.dumps(timeline[ts]) + "\n")

    print(f"✅ {profile['label']}: {tickers[0]}→BTC-USD, {tickers[1]}→ETH-USD, {tickers[2]}→SOL-USD")
    print(f"🧪 Replay: python3 scripts/replay_from_file.py --file {output_file}")

if __name__ == "__main__":
    main()
