#!/usr/bin/env python3
"""
refresh_intraday_cache.py — Bulk refresh of intraday/daily Yahoo Finance caches.

Reads the list of tickers from existing intraday_capture/yahoo_cache_1m/ directory,
fetches fresh bars for each ticker at requested intervals (1m, 5m, 15m, 30m, 1d),
merges with existing cache using NEW FETCH WINS + deduplicate (ticker, timestamp) + chronological sort,
and writes the results back to respective cache directories.

Usage:
    python3 scripts/refresh_intraday_cache.py [--intervals 15m,30m,1d] [--dry-run]
"""

import argparse
import json
import os
import sys
import time
from datetime import datetime, timezone
from pathlib import Path

import yfinance as yf

# ── Constants ─────────────────────────────────────────────────────────────────

REPO_ROOT = Path(__file__).resolve().parent.parent

CACHE_DIRS = {
    "1m":  REPO_ROOT / "intraday_capture" / "yahoo_cache_1m",
    "5m":  REPO_ROOT / "intraday_capture" / "yahoo_cache_5m",
    "15m": REPO_ROOT / "intraday_capture" / "yahoo_cache_15m",
    "30m": REPO_ROOT / "intraday_capture" / "yahoo_cache_30m",
    "1d":  REPO_ROOT / "intraday_capture" / "yahoo_cache_1d",
}

# Yahoo Finance period for each interval
INTERVAL_PERIOD = {
    "1m":  "7d",    # Yahoo retains 1m bars for ~7 days
    "5m":  "60d",   # Yahoo retains 5m bars for ~60 days
    "15m": "60d",   # Yahoo retains 15m bars for ~60 days
    "30m": "60d",   # Yahoo retains 30m bars for ~60 days
    "1d":  "max",   # Yahoo retains daily bars for max history
}

REQUEST_DELAY_SEC = 0.2


def get_tickers_from_cache(cache_dir: Path) -> list[str]:
    """Read ticker list from existing cache directory (TICKER.NS.json files)."""
    if not cache_dir.exists():
        return []
    return sorted(
        f.stem  # e.g. "ADANIENT.NS"
        for f in cache_dir.iterdir()
        if f.suffix == ".json"
    )


def fetch_bars(symbol: str, interval: str, period: str) -> list[dict]:
    """Fetch OHLCV bars from Yahoo Finance. Returns list of bar dicts."""
    ticker = yf.Ticker(symbol)
    df = ticker.history(period=period, interval=interval, auto_adjust=False)
    if df.empty:
        return []
    bars = []
    for ts, row in df.iterrows():
        unix_ts = int(ts.timestamp())
        dt_str = ts.strftime('%Y-%m-%d %H:%M:%S') if interval != "1d" else ts.strftime('%Y-%m-%d')
        bars.append({
            "timestamp": unix_ts,
            "datetime": dt_str,
            "open": float(row["Open"]),
            "high": float(row["High"]),
            "low": float(row["Low"]),
            "close": float(row["Close"]),
            "adj_close": float(row.get("Adj Close", row["Close"])),
            "volume": float(row["Volume"]),
        })
    return bars


def merge_bars_new_fetch_wins(existing_bars: list[dict], fresh_bars: list[dict]) -> list[dict]:
    """
    Merge existing cache with fresh fetch using NEW FETCH WINS protocol.
    Deduplicates by timestamp and sorts chronologically.
    """
    bar_map = {b["timestamp"]: b for b in existing_bars}
    for b in fresh_bars:
        bar_map[b["timestamp"]] = b  # NEW FETCH WINS
    sorted_ts = sorted(bar_map.keys())
    return [bar_map[t] for t in sorted_ts]


def refresh_interval(interval: str, tickers: list[str], dry_run: bool) -> dict:
    """Refresh cache for one interval. Returns summary dict."""
    cache_dir = CACHE_DIRS[interval]
    period = INTERVAL_PERIOD[interval]
    cache_dir.mkdir(parents=True, exist_ok=True)

    n_ok = 0
    n_empty = 0
    n_error = 0

    for i, ticker in enumerate(tickers):
        symbol = ticker
        out_path = cache_dir / f"{symbol}.json"

        existing_bars = []
        if out_path.exists():
            try:
                existing_bars = json.loads(out_path.read_text())
            except Exception:
                existing_bars = []

        if dry_run:
            print(f"  [DRY-RUN] would fetch {symbol} ({interval}) → {out_path.name}")
            n_ok += 1
            continue

        try:
            fresh_bars = fetch_bars(symbol, interval, period)
            if not fresh_bars and not existing_bars:
                print(f"  WARN [{interval}] {symbol}: no bars returned")
                n_empty += 1
            else:
                merged = merge_bars_new_fetch_wins(existing_bars, fresh_bars)
                out_path.write_text(json.dumps(merged, separators=(",", ":")))
                print(f"  OK   [{interval}] {symbol}: {len(fresh_bars)} fresh + {len(existing_bars)} existing → {len(merged)} total bars")
                n_ok += 1
        except Exception as e:
            print(f"  ERR  [{interval}] {symbol}: {e}")
            n_error += 1

        if i < len(tickers) - 1:
            time.sleep(REQUEST_DELAY_SEC)

    return {"interval": interval, "n_ok": n_ok, "n_empty": n_empty, "n_error": n_error}


def main():
    parser = argparse.ArgumentParser(
        description="Bulk refresh intraday/daily Yahoo Finance caches (15m, 30m, 1d)."
    )
    parser.add_argument(
        "--intervals",
        default="15m,30m,1d",
        help="Comma-separated list of intervals to refresh (default: 15m,30m,1d)",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Print what would be fetched without making any requests.",
    )
    args = parser.parse_args()

    intervals = [i.strip() for i in args.intervals.split(",")]
    for iv in intervals:
        if iv not in CACHE_DIRS:
            print(f"ERROR: unknown interval '{iv}'. Valid: {list(CACHE_DIRS.keys())}")
            sys.exit(1)

    now_ist = datetime.now(timezone.utc).strftime("%Y-%m-%d %H:%M UTC")
    print("=" * 60)
    print("  INTRADAY & DAILY CACHE REFRESH")
    print(f"  Run time: {now_ist}")
    print(f"  Intervals: {', '.join(intervals)}")
    print(f"  Dry run: {args.dry_run}")
    print("=" * 60)

    # Get tickers from the 1m cache (canonical source)
    tickers = get_tickers_from_cache(CACHE_DIRS["1m"])
    if not tickers:
        for iv in ("5m", "15m", "30m", "1d"):
            tickers = get_tickers_from_cache(CACHE_DIRS[iv])
            if tickers:
                break
    if not tickers:
        print("ERROR: no tickers found in any cache directory.")
        sys.exit(1)

    print(f"Tickers: {len(tickers)}")
    print()

    summaries = []
    for interval in intervals:
        print(f"── Refreshing {interval} cache ({len(tickers)} tickers) ──")
        summary = refresh_interval(interval, tickers, dry_run=args.dry_run)
        summaries.append(summary)
        print(
            f"  {interval} done: ok={summary['n_ok']} empty={summary['n_empty']} error={summary['n_error']}"
        )
        print()

    print("=" * 60)
    print("SUMMARY")
    for s in summaries:
        status = "OK" if s["n_error"] == 0 and s["n_empty"] == 0 else "WARN"
        print(
            f"  [{status}] {s['interval']}: ok={s['n_ok']} empty={s['n_empty']} error={s['n_error']}"
        )
    print("=" * 60)


if __name__ == "__main__":
    main()