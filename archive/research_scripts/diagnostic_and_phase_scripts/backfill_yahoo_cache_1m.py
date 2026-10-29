#!/usr/bin/env python3
"""
Backfill intraday_capture/yahoo_cache_1m/<TICKER>.NS.json
with 1-minute bars for a given date, merging with existing data.

Usage:
  python3 scripts/backfill_yahoo_cache_1m.py --date 2026-09-18 [--tickers AXISBANK_NS,...]
"""
import argparse
import json
import os
import sys
import time
import datetime

try:
    import yfinance as yf
except ImportError:
    print("yfinance not installed. Run: pip install yfinance", file=sys.stderr)
    sys.exit(1)

CACHE_DIR = "intraday_capture/yahoo_cache_1m"


def brief_ticker_to_yahoo(ticker: str) -> str:
    """AXISBANK_NS -> AXISBANK.NS"""
    return ticker.replace("_NS", ".NS").replace("_BO", ".BO")


def fetch_bars(yahoo_symbol: str, date_str: str):
    """Fetch 1m bars for a specific date via yfinance."""
    # yfinance interval=1m supports max 7 days of history
    # We fetch start=date, end=date+1day to get exactly that day
    date = datetime.date.fromisoformat(date_str)
    start = date_str
    end = (date + datetime.timedelta(days=1)).isoformat()
    try:
        tk = yf.Ticker(yahoo_symbol)
        df = tk.history(start=start, end=end, interval="1m", auto_adjust=False)
        if df is None or len(df) == 0:
            return []
        records = []
        for ts_idx, row in df.iterrows():
            # Handle timezone-aware index
            if hasattr(ts_idx, "timestamp"):
                ts_unix = int(ts_idx.timestamp())
            else:
                ts_unix = int(ts_idx.to_pydatetime().timestamp())
            records.append({
                "timestamp": ts_unix,
                "open": float(row.get("Open", 0.0)),
                "high": float(row.get("High", 0.0)),
                "low": float(row.get("Low", 0.0)),
                "close": float(row.get("Close", 0.0)),
                "adj_close": float(row.get("Adj Close", row.get("Close", 0.0))),
                "volume": float(row.get("Volume", 0.0)),
            })
        return records
    except Exception as e:
        print(f"  ERROR fetching {yahoo_symbol}: {e}", file=sys.stderr)
        return []


def merge_and_save(cache_path: str, new_bars: list):
    """Merge new_bars into existing cache file (deduplicate by timestamp)."""
    existing = []
    if os.path.exists(cache_path):
        with open(cache_path) as f:
            existing = json.load(f)
    existing_by_ts = {r["timestamp"]: r for r in existing}
    for bar in new_bars:
        existing_by_ts[bar["timestamp"]] = bar
    merged = sorted(existing_by_ts.values(), key=lambda r: r["timestamp"])
    with open(cache_path, "w") as f:
        json.dump(merged, f)
    return len(merged), len(new_bars)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--date", required=True, help="Date to backfill (YYYY-MM-DD)")
    ap.add_argument("--tickers", default="", help="Comma-separated brief tickers (default: all in cache dir)")
    ap.add_argument("--delay", type=float, default=1.0, help="Delay between fetches (seconds)")
    args = ap.parse_args()

    os.makedirs(CACHE_DIR, exist_ok=True)

    if args.tickers:
        brief_tickers = [t.strip() for t in args.tickers.split(",") if t.strip()]
    else:
        # All existing .json files in the cache
        brief_tickers = []
        for fname in sorted(os.listdir(CACHE_DIR)):
            if fname.endswith(".json"):
                # AXISBANK.NS.json -> AXISBANK_NS
                brief_tickers.append(fname.replace(".NS.json", "_NS").replace(".BO.json", "_BO"))

    print(f"Backfilling {len(brief_tickers)} tickers for {args.date}")
    ok, skipped, failed = 0, 0, []

    for i, ticker in enumerate(brief_tickers, 1):
        yahoo_sym = brief_ticker_to_yahoo(ticker)
        cache_path = os.path.join(CACHE_DIR, f"{yahoo_sym}.json")
        print(f"[{i:3d}/{len(brief_tickers)}] {yahoo_sym} ...", end=" ", flush=True)
        bars = fetch_bars(yahoo_sym, args.date)
        if not bars:
            print("NO DATA")
            failed.append(yahoo_sym)
        else:
            total, added = merge_and_save(cache_path, bars)
            print(f"{added} new bars merged (cache total: {total})")
            ok += 1
        if args.delay > 0:
            time.sleep(args.delay)

    print(f"\nDone. OK={ok}  NoData={len(failed)}")
    if failed:
        print("No data for:", ", ".join(failed))


if __name__ == "__main__":
    main()
