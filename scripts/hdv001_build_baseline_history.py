#!/usr/bin/env python3
"""
HDV-001-F Baseline History Cache Builder
==========================================
Builds a separate pre-development lookback cache for Baseline C (momentum).

The primary HDV-001-B cache covers 2026-07-14 to 2026-08-13.
Baseline C requires 20 NSE sessions before the earliest decision (2026-07-14).
This script fetches 2026-06-01 to 2026-07-13 for all 52 instruments.

Output:
  datasets/hdv001/hdv001_baseline_history_v1/  (per-instrument JSON)
  datasets/hdv001/hdv001_baseline_history_manifest.json

This cache is SEPARATE from hdv001_price_cache_v1 and must not be merged
with it. The primary cache remains immutable.
"""

import json
import sys
from datetime import date, datetime, timezone
from pathlib import Path

try:
    import yfinance as yf
except ImportError:
    print("ERROR: yfinance not installed. Run: pip install yfinance")
    sys.exit(1)

# ── paths ─────────────────────────────────────────────────────────────────────
WORKSPACE   = Path(__file__).resolve().parent.parent
CACHE_DIR   = WORKSPACE / "datasets" / "hdv001" / "hdv001_baseline_history_v1"
MANIFEST    = WORKSPACE / "datasets" / "hdv001" / "hdv001_baseline_history_manifest.json"

# ── constants ─────────────────────────────────────────────────────────────────
FETCH_START = "2026-06-01"
FETCH_END   = "2026-07-14"   # exclusive in yfinance (last bar = 2026-07-13)

NSE_UNIVERSE = [
    "ADANIENT.NS", "ADANIPORTS.NS", "ASIANPAINT.NS", "AXISBANK.NS",
    "BAJAJFINSV.NS", "BAJFINANCE.NS", "BHARTIARTL.NS", "BPCL.NS",
    "CIPLA.NS", "COALINDIA.NS", "DIVISLAB.NS", "DRREDDY.NS",
    "EICHERMOT.NS", "GRASIM.NS", "HCLTECH.NS", "HDFCBANK.NS",
    "HDFCLIFE.NS", "HEROMOTOCO.NS", "HINDALCO.NS", "HINDUNILVR.NS",
    "ICICIBANK.NS", "IDEA.NS", "INDUSINDBK.NS", "INFY.NS",
    "ITC.NS", "JSWSTEEL.NS", "KOTAKBANK.NS", "LT.NS",
    "M&M.NS", "MAHABANK.NS", "MARUTI.NS", "NESTLEIND.NS",
    "NTPC.NS", "ONGC.NS", "PIDILITIND.NS", "POWERGRID.NS",
    "RELIANCE.NS", "SBILIFE.NS", "SBIN.NS", "SHREECEM.NS",
    "SUNPHARMA.NS", "TATACONSUM.NS", "TATASTEEL.NS", "TCS.NS",
    "TECHM.NS", "TITAN.NS", "TMPV.NS", "ULTRACEMCO.NS",
    "UNITDSPR.NS", "UPL.NS", "VEDL.NS", "WIPRO.NS",
]

def symbol_to_filename(symbol: str) -> str:
    return symbol.replace("&M", "ANDM").replace(".", "_") + ".json"

def main():
    print("=" * 70)
    print("HDV-001-F BASELINE HISTORY CACHE BUILDER")
    print("=" * 70)
    print(f"Fetch window: {FETCH_START} to {FETCH_END} (exclusive)")
    print(f"Universe: {len(NSE_UNIVERSE)} instruments")
    print()

    CACHE_DIR.mkdir(parents=True, exist_ok=True)

    results = []
    ok = warn = fail = 0

    for symbol in NSE_UNIVERSE:
        try:
            ticker = yf.Ticker(symbol)
            df = ticker.history(
                start=FETCH_START,
                end=FETCH_END,
                interval="1d",
                auto_adjust=True,
            )
            if df.empty:
                print(f"  WARN  {symbol}: no data returned")
                warn += 1
                results.append({"symbol": symbol, "status": "WARN", "n_bars": 0})
                continue

            bars = []
            for ts, row in df.iterrows():
                bar_date = ts.date() if hasattr(ts, "date") else ts
                bars.append({
                    "date":   str(bar_date),
                    "open":   round(float(row["Open"]),  4),
                    "high":   round(float(row["High"]),  4),
                    "low":    round(float(row["Low"]),   4),
                    "close":  round(float(row["Close"]), 4),
                    "volume": int(row["Volume"]),
                })

            n = len(bars)
            fname = symbol_to_filename(symbol)
            out = {
                "symbol":     symbol,
                "fetch_start": FETCH_START,
                "fetch_end":   FETCH_END,
                "n_bars":      n,
                "bars":        bars,
            }
            (CACHE_DIR / fname).write_text(json.dumps(out, indent=2))
            print(f"  OK    {symbol}: {n} bars  ({bars[0]['date']} → {bars[-1]['date']})")
            ok += 1
            results.append({"symbol": symbol, "status": "OK", "n_bars": n,
                            "first": bars[0]["date"], "last": bars[-1]["date"]})

        except Exception as e:
            print(f"  FAIL  {symbol}: {e}")
            fail += 1
            results.append({"symbol": symbol, "status": "FAIL", "error": str(e)})

    manifest = {
        "version":    "hdv001_baseline_history_v1",
        "built_at":   datetime.now(timezone.utc).isoformat(),
        "fetch_start": FETCH_START,
        "fetch_end":   FETCH_END,
        "n_instruments": len(NSE_UNIVERSE),
        "ok": ok, "warn": warn, "fail": fail,
        "instruments": results,
    }
    MANIFEST.write_text(json.dumps(manifest, indent=2))

    print()
    print(f"Summary: {ok} OK, {warn} WARN, {fail} FAIL")
    print(f"Manifest: {MANIFEST.relative_to(WORKSPACE)}")

    if fail > 0:
        print("ERROR: some instruments failed. Do not proceed to baseline run.")
        sys.exit(1)
    print("\nBaseline history cache COMPLETE.")
    sys.exit(0)

if __name__ == "__main__":
    main()