#!/usr/bin/env python3
"""
ChronoSentiment — yfinance candle fetcher
Usage: python3 scripts/fetch_candles.py <symbol> <interval> <n_candles>
Output: JSON array of candles to stdout

Supported intervals: 1m, 5m, 15m, 30m, 1h, 1d, 1wk
Works for: NSE (.NS), BSE (.BO), HKEx (.HK), TSE (.T), US equities, crypto
"""

import sys
import json
import yfinance as yf
import pandas as pd


def interval_to_period(interval: str) -> str:
    """Return the minimum period needed to guarantee enough bars."""
    return {
        "1m":  "5d",
        "5m":  "5d",
        "15m": "5d",
        "30m": "1mo",
        "1h":  "730d",
        "1d":  "2y",
        "1wk": "5y",
    }.get(interval, "60d")


def fetch_latest(symbol: str, interval: str, n_candles: int) -> list[dict]:
    """Fetch latest candles from Yahoo Finance."""
    period = interval_to_period(interval)
    # Use direct download with small window to minimize stale cached responses.
    df = yf.download(
        tickers=symbol,
        period=period,
        interval=interval,
        auto_adjust=True,
        progress=False,
        threads=False,
    )
    if df.empty:
        return []
    if isinstance(df.columns, pd.MultiIndex):
        # yfinance download may return (Price, Ticker) multi-index columns for one symbol.
        df.columns = df.columns.get_level_values(0)

    df = df.tail(n_candles)
    candles: list[dict] = []
    for ts, row in df.iterrows():
        try:
            timestamp = int(ts.timestamp())
        except Exception:
            timestamp = 0
        candles.append(
            {
                "timestamp": timestamp,
                "open": float(row["Open"]),
                "high": float(row["High"]),
                "low": float(row["Low"]),
                "close": float(row["Close"]),
                "volume": int(row.get("Volume", 0)),
            }
        )
    return candles


def main():
    if len(sys.argv) < 4:
        print(json.dumps({"error": "Usage: fetch_candles.py <symbol> <interval> <n_candles>"}), file=sys.stderr)
        sys.exit(1)

    symbol    = sys.argv[1]
    interval  = sys.argv[2]
    n_candles = int(sys.argv[3])
    try:
        candles = fetch_latest(symbol, interval, n_candles)
        if not candles:
            print(f"ERROR: yfinance returned no data for {symbol} @ {interval}", file=sys.stderr)
            print("[]")
            return
        print(json.dumps(candles))

    except Exception as e:
        print(f"ERROR: yfinance exception for {symbol} @ {interval}: {e}", file=sys.stderr)
        print("[]")

if __name__ == "__main__":
    main()
