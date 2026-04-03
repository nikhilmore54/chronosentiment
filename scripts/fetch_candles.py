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

def interval_to_period(interval: str) -> str:
    """Return the minimum period needed to guarantee enough bars."""
    return {
        "1m":  "5d",
        "5m":  "60d",
        "15m": "60d",
        "30m": "60d",
        "1h":  "730d",
        "1d":  "2y",
        "1wk": "5y",
    }.get(interval, "60d")

def main():
    if len(sys.argv) < 4:
        print(json.dumps({"error": "Usage: fetch_candles.py <symbol> <interval> <n_candles>"}), file=sys.stderr)
        sys.exit(1)

    symbol    = sys.argv[1]
    interval  = sys.argv[2]
    n_candles = int(sys.argv[3])
    period    = interval_to_period(interval)

    try:
        ticker = yf.Ticker(symbol)
        df = ticker.history(period=period, interval=interval, auto_adjust=True)

        if df.empty:
            print(f"ERROR: yfinance returned no data for {symbol} @ {interval}", file=sys.stderr)
            print("[]")
            return

        # Keep last n_candles rows
        df = df.tail(n_candles)

        candles = []
        for ts, row in df.iterrows():
            # pandas Timestamp → Unix seconds (UTC)
            try:
                timestamp = int(ts.timestamp())
            except Exception:
                timestamp = 0

            candles.append({
                "timestamp": timestamp,
                "open":      float(row["Open"]),
                "high":      float(row["High"]),
                "low":       float(row["Low"]),
                "close":     float(row["Close"]),
                "volume":    int(row.get("Volume", 0)),
            })

        print(json.dumps(candles))

    except Exception as e:
        print(f"ERROR: yfinance exception for {symbol} @ {interval}: {e}", file=sys.stderr)
        print("[]")

if __name__ == "__main__":
    main()
